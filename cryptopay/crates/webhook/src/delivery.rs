use std::time::Duration;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use sqlx::PgPool;

use crate::signing::sign_payload;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookPayload {
    pub event: String,           // 'payment.confirmed'
    pub payment_id: Uuid,
    pub tx_hash: Option<String>,
    pub amount: String,          // Decimal formatted as string
    pub currency: String,
    pub confirmed_at: Option<DateTime<Utc>>,
    pub metadata: serde_json::Value,
}

#[derive(Debug)]
pub struct WebhookDelivery {
    http: Client,
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
        secret: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let pending_webhooks = sqlx::query_as::<_, WebhookDeliveryRow>(
            r#"
            SELECT id, payment_id, event_type, url, payload, status, attempts, max_attempts, last_attempt_at, next_attempt_at, response_status, created_at
            FROM webhook_deliveries
            WHERE status = 'pending' AND next_attempt_at <= NOW()
            ORDER BY next_attempt_at ASC
            LIMIT 10
            "#,
        )
        .fetch_all(pool)
        .await?;

        for webhook in pending_webhooks {
            let payload: WebhookPayload = serde_json::from_value(webhook.payload.clone())?;
            let status_code = delivery.send(&webhook.url, secret, &payload).await?;

            if status_code >= 200 && status_code < 300 {
                // Success
                sqlx::query(
                    r#"
                    UPDATE webhook_deliveries
                    SET status = 'delivered', response_status = $1, last_attempt_at = NOW()
                    WHERE id = $2
                    "#,
                )
                .bind(status_code as i32)
                .bind(webhook.id)
                .execute(pool)
                .await?;
            } else {
                // Failure - increment attempts and schedule retry
                let new_attempts = webhook.attempts + 1;
                let new_status = if new_attempts >= webhook.max_attempts {
                    "failed"
                } else {
                    "pending"
                };

                // Exponential backoff: 2^attempts * 30 seconds
                let backoff_seconds = 30 * (1 << webhook.attempts); // 2^attempts
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
                .bind(status_code as i32)
                .bind(webhook.id)
                .execute(pool)
                .await?;
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
}