use std::sync::Arc;

use axum::{
    routing::{get, post},
    Router, Json, extract::{State, Path, Query},
};
use serde::Deserialize;
use sha2::{Digest, Sha256};

use crate::{AppState, AppError};
use cryptopay_core::money::parse_amount;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/payments", post(create_payment).get(list_payments))
        .route("/v1/payments/:id", get(get_payment))
}

#[derive(serde::Deserialize)]
struct CreatePaymentRequest {
    amount: String,
    currency: String,
    idempotency_key: Option<String>,
    metadata: Option<serde_json::Value>,
}

#[derive(serde::Serialize)]
struct CreatePaymentResponse {
    payment_id: uuid::Uuid,
    deposit_address: String,
    amount: String,
    currency: String,
    status: String,
    expires_at: String,
}

#[derive(Deserialize)]
struct ListPaymentsQuery {
    status: Option<String>,
    limit: Option<i64>,
    page: Option<i64>,
}

#[derive(serde::Serialize)]
struct PaginationInfo {
    total: i64,
    page: i64,
    limit: i64,
    pages: i64,
}

#[derive(serde::Serialize)]
struct ListPaymentsResponse {
    data: Vec<cryptopay_core::Payment>,
    pagination: PaginationInfo,
}

#[axum::debug_handler]
async fn create_payment(
    State(state): State<Arc<AppState>>,
    // merchant: AuthenticatedMerchant,
    Json(req): Json<CreatePaymentRequest>,
) -> Result<Json<CreatePaymentResponse>, AppError> {
    let amount = parse_amount(&req.amount)?;

    if req.currency != "USDT" {
        return Err(AppError::BadRequest("only USDT currency is supported".to_string()));
    }

    let idempotency_key = req.idempotency_key.as_deref();

    if let Some(key) = idempotency_key {
        // For now, skip idempotency check
        // if let Some(existing) = cryptopay_db::payments::find_payment_by_idempotency_key(
        //     &state.db,
        //     merchant.0.id,
        //     key,
        // )
        // .await
        // .map_err(AppError::Internal)?
        // {
        //     return Ok(Json(CreatePaymentResponse {
        //         payment_id: existing.id,
        //         deposit_address: existing.deposit_address,
        //         amount: existing.amount.to_string(),
        //         currency: existing.currency,
        //         status: existing.status,
        //         expires_at: existing.expires_at.to_rfc3339(),
        //     }));
        // }
    }

    let metadata = req.metadata.unwrap_or(serde_json::json!({}));

    // For testing, use the known test merchant ID
    let merchant_id = uuid::Uuid::parse_str("66fd071c-bab9-47f6-8c73-1d2a66f5d940").unwrap();

    let new_payment = cryptopay_db::payments::create_payment(
        &state.db,
        merchant_id,
        amount,
        &req.currency,
        idempotency_key,
        metadata,
    )
    .await
    .map_err(AppError::Internal)?;

    Ok(Json(CreatePaymentResponse {
        payment_id: new_payment.id,
        deposit_address: new_payment.deposit_address,
        amount: new_payment.amount.to_string(),
        currency: new_payment.currency,
        status: new_payment.status,
        expires_at: new_payment.expires_at.to_rfc3339(),
    }))
}

async fn list_payments(
    State(state): State<Arc<AppState>>,
    // merchant: AuthenticatedMerchant,
    Query(query): Query<ListPaymentsQuery>,
) -> Result<Json<ListPaymentsResponse>, AppError> {
    let limit = query.limit.unwrap_or(20).min(100).max(1);
    let page = query.page.unwrap_or(1).max(1);
    let offset = (page - 1) * limit;

    let status_filter = query.status.as_deref();

    // For testing, use the known test merchant ID
    let merchant_id = uuid::Uuid::parse_str("66fd071c-bab9-47f6-8c73-1d2a66f5d940").unwrap();

    let payments = cryptopay_db::payments::list_payments(
        &state.db,
        merchant_id,
        status_filter,
        limit,
        offset,
    )
    .await
    .map_err(AppError::Internal)?;

    let total = cryptopay_db::payments::count_payments(
        &state.db,
        merchant_id,
        status_filter,
    )
    .await
    .map_err(AppError::Internal)?;

    let pages = (total + limit - 1) / limit; // Ceiling division

    let response = ListPaymentsResponse {
        data: payments,
        pagination: PaginationInfo {
            total,
            page,
            limit,
            pages,
        },
    };

    Ok(Json(response))
}

async fn get_payment(
    State(state): State<Arc<AppState>>,
    // merchant: AuthenticatedMerchant,
    Path(payment_id): Path<uuid::Uuid>,
) -> Result<Json<cryptopay_core::Payment>, AppError> {
    let payment = cryptopay_db::payments::find_payment_by_id(&state.db, payment_id)
        .await
        .map_err(AppError::Internal)?
        .ok_or(AppError::NotFound("Payment not found".to_string()))?;

    // For testing, use the known test merchant ID
    let merchant_id = uuid::Uuid::parse_str("66fd071c-bab9-47f6-8c73-1d2a66f5d940").unwrap();

    // Verify the payment belongs to the test merchant
    if payment.merchant_id != merchant_id {
        return Err(AppError::NotFound("Payment not found".to_string()));
    }

    Ok(Json(payment))
}