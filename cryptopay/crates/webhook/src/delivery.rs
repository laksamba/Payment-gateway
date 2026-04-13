use std::time::Duration;

use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::signing::sign_payload;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookPayload {
    pub event: String,
    pub payment_id: Uuid,
    pub tx_hash: Option<String>,
    pub amount: String,
    pub currency: String,
    pub confirmed_at: Option<DateTime<Utc>>,
    pub metadata: serde_json::Value,
}

#[derive(Debug)]
pub struct WebhookDelivery {
    http: Client,
}

impl Default for WebhookDelivery {
    fn default() -> Self {
        Self::new()
    }
}

impl WebhookDelivery {
    pub fn new() -> Self {
        Self {
            http: Client::new(),
        }
    }

    pub async fn send(
        &self,
        url: &str,
        secret: &str,
        payload: &WebhookPayload,
    ) -> Result<u16, Box<dyn std::error::Error + Send + Sync>> {
        let body = serde_json::to_string(payload)?;
        let signature = sign_payload(secret, &body);
        let response = self.http
            .post(url)
            .header("Content-Type", "application/json")
            .header("X-CryptoPay-Signature", signature)
            .header("X-CryptoPay-Event", &payload.event)
            .timeout(Duration::from_secs(10))
            .body(body)
            .send()
            .await?;
        Ok(response.status().as_u16())
    }

    pub async fn queue_webhook(
        pool: &PgPool,
        payment_id: Uuid,
        event_type: &str,
        url: &str,
        payload: &WebhookPayload,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        sqlx::query(
            r#"
            INSERT INTO webhook_deliveries (payment_id, event_type, url, payload)
            VALUES ($1, $2, $3, $4)
            "#,
        )
        .bind(payment_id)
        .bind(event_type)
        .bind(url)
        .bind(serde_json::to_value(payload)?)
        .execute(pool)
        .await?;
        Ok(())
    }

    pub async fn process_pending_webhooks(
        pool: &PgPool,
        delivery: &WebhookDelivery,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let pending_webhooks = sqlx::query_as::<_, WebhookDeliveryRow>(
            r#"
            SELECT wd.id, wd.payment_id, wd.event_type, wd.url, wd.payload, wd.status, wd.attempts,
                   wd.max_attempts, wd.last_attempt_at, wd.next_attempt_at, wd.response_status,
                   wd.created_at, m.webhook_secret AS merchant_webhook_secret
            FROM webhook_deliveries wd
            INNER JOIN payments p ON p.id = wd.payment_id
            INNER JOIN merchants m ON m.id = p.merchant_id
            WHERE wd.status = 'pending' AND wd.next_attempt_at <= NOW()
            ORDER BY wd.next_attempt_at ASC
            LIMIT 10
            "#,
        )
        .fetch_all(pool)
        .await?;

        for webhook in pending_webhooks {
            let payload: WebhookPayload = serde_json::from_value(webhook.payload.clone())?;
            match delivery
                .send(&webhook.url, &webhook.merchant_webhook_secret, &payload)
                .await
            {
                Ok(status_code) if (200..300).contains(&status_code) => {
                    sqlx::query(
                        r#"
                        UPDATE webhook_deliveries
                        SET status = 'delivered', attempts = attempts + 1, response_status = $1, last_attempt_at = NOW()
                        WHERE id = $2
                        "#,
                    )
                    .bind(status_code as i32)
                    .bind(webhook.id)
                    .execute(pool)
                    .await?;
                }
                Ok(status_code) => {
                    reschedule_webhook(pool, &webhook, Some(status_code as i32)).await?;
                }
                Err(error) => {
                    tracing::warn!("Webhook delivery failed for {}: {}", webhook.id, error);
                    reschedule_webhook(pool, &webhook, None).await?;
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug, sqlx::FromRow)]
pub struct WebhookDeliveryRow {
    pub id: Uuid,
    pub payment_id: Uuid,
    pub event_type: String,
    pub url: String,
    pub payload: serde_json::Value,
    pub status: String,
    pub attempts: i32,
    pub max_attempts: i32,
    pub last_attempt_at: Option<DateTime<Utc>>,
    pub next_attempt_at: DateTime<Utc>,
    pub response_status: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub merchant_webhook_secret: String,
}

async fn reschedule_webhook(
    pool: &PgPool,
    webhook: &WebhookDeliveryRow,
    response_status: Option<i32>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let new_attempts = webhook.attempts + 1;
    let new_status = if new_attempts >= webhook.max_attempts {
        "failed"
    } else {
        "pending"
    };

    let backoff_seconds = 30 * (1 << webhook.attempts);
    let next_attempt = chrono::Utc::now() + chrono::Duration::seconds(backoff_seconds as i64);

    sqlx::query(
        r#"
        UPDATE webhook_deliveries
        SET status = $1, attempts = $2, last_attempt_at = NOW(), next_attempt_at = $3, response_status = $4
        WHERE id = $5
        "#,
    )
    .bind(new_status)
    .bind(new_attempts)
    .bind(next_attempt)
    .bind(response_status)
    .bind(webhook.id)
    .execute(pool)
    .await?;

    Ok(())
}
