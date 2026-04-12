use axum::{
    extract::{Query, State},
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use anyhow;

use crate::{AppState, AppError};

#[derive(Deserialize)]
pub struct WebhookQuery {
    limit: Option<i64>,
    offset: Option<i64>,
}

#[derive(Serialize, sqlx::FromRow)]
pub struct WebhookDeliveryResponse {
    pub id: Uuid,
    pub payment_id: Uuid,
    pub event_type: String,
    pub url: String,
    pub status: String,
    pub attempts: i32,
    pub max_attempts: i32,
    pub last_attempt_at: Option<DateTime<Utc>>,
    pub next_attempt_at: Option<DateTime<Utc>>,
    pub response_status: Option<i32>,
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize)]
pub struct WebhookListResponse {
    pub data: Vec<WebhookDeliveryResponse>,
    pub pagination: PaginationInfo,
}

#[derive(Serialize)]
pub struct PaginationInfo {
    pub limit: i64,
    pub offset: i64,
    pub total: i64,
}

pub async fn list_webhooks(
    State(state): State<std::sync::Arc<crate::AppState>>,
    Query(query): Query<WebhookQuery>,
) -> Result<Json<WebhookListResponse>, AppError> {
    let limit = query.limit.unwrap_or(50).min(100);
    let offset = query.offset.unwrap_or(0);

    // TODO: Get merchant from authentication context
    // For now, get the test merchant
    let merchant = cryptopay_db::merchants::find_merchant_by_email(&state.db, "test@example.com")
        .await
        .map_err(AppError::Internal)?
        .ok_or(AppError::NotFound("Merchant not found".to_string()))?;

    let webhooks = sqlx::query_as::<_, WebhookDeliveryResponse>(
        r#"
        SELECT id, payment_id, event_type, url, status, attempts, max_attempts,
               last_attempt_at, next_attempt_at, response_status, created_at
        FROM webhook_deliveries
        WHERE payment_id IN (
            SELECT id FROM payments WHERE merchant_id = $1
        )
        ORDER BY created_at DESC
        LIMIT $2 OFFSET $3
        "#,
    )
    .bind(merchant.id)
    .bind(limit)
    .bind(offset)
    .fetch_all(&state.db)
    .await
    .map_err(|e| AppError::Internal(anyhow::anyhow!("Database error: {}", e)))?;

    let total = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(*)
        FROM webhook_deliveries
        WHERE payment_id IN (
            SELECT id FROM payments WHERE merchant_id = $1
        )
        "#,
    )
    .bind(merchant.id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| AppError::Internal(anyhow::anyhow!("Database error: {}", e)))?;

    Ok(Json(WebhookListResponse {
        data: webhooks,
        pagination: PaginationInfo {
            limit,
            offset,
            total,
        },
    }))
}

pub fn routes() -> axum::Router<std::sync::Arc<crate::AppState>> {
    axum::Router::new()
        .route("/v1/webhooks", axum::routing::get(list_webhooks))
}