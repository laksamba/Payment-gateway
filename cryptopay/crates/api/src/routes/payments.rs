use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;

use crate::{AppError, AppState, AuthenticatedMerchant};
use cryptopay_core::{money::parse_amount, Payment, PaymentStatus};

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

#[derive(serde::Serialize)]
struct MerchantPaymentResponse {
    payment_id: uuid::Uuid,
    amount: String,
    currency: String,
    deposit_address: String,
    status: String,
    tx_hash: Option<String>,
    confirmations: i32,
    required_confirmations: i32,
    idempotency_key: Option<String>,
    metadata: serde_json::Value,
    expires_at: String,
    confirmed_at: Option<String>,
    created_at: String,
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
    data: Vec<MerchantPaymentResponse>,
    pagination: PaginationInfo,
}

#[axum::debug_handler]
async fn create_payment(
    State(state): State<Arc<AppState>>,
    merchant: AuthenticatedMerchant,
    Json(req): Json<CreatePaymentRequest>,
) -> Result<Json<CreatePaymentResponse>, AppError> {
    let amount = parse_amount(&req.amount)?;

    if req.currency != "USDT" {
        return Err(AppError::BadRequest(
            "only USDT currency is supported".to_string(),
        ));
    }

    let withdrawal_address = merchant
        .0
        .withdrawal_address
        .as_deref()
        .ok_or_else(|| {
            tracing::warn!("Merchant {} has no withdrawal_address set", merchant.0.id);
            AppError::BadRequest(
                "Set your withdrawal wallet address first via POST /v1/merchants/settings".to_string(),
            )
        })?;

    tracing::info!("Creating payment for merchant {} with withdrawal_address: {}", merchant.0.id, withdrawal_address);

    let idempotency_key = req.idempotency_key.as_deref();
    let metadata = req.metadata.unwrap_or(serde_json::json!({}));

    if let Some(key) = idempotency_key {
        let existing_payment = cryptopay_db::payments::find_payment_by_idempotency_key(
            &state.db,
            merchant.0.id,
            key,
        )
        .await
        .map_err(|error| {
            tracing::error!(
                "Failed to load idempotent payment for merchant {} and key {}: {}",
                merchant.0.id,
                key,
                error
            );
            AppError::Internal(error)
        })?;

        if let Some(existing) = existing_payment {
            if existing.amount != amount
                || existing.currency != req.currency
                || existing.metadata != metadata
            {
                return Err(AppError::Conflict(
                    "idempotency_key already used with different payment parameters".to_string(),
                ));
            }

            return Ok(Json(to_create_payment_response(&existing)));
        }
    }

    let create_payment_result = cryptopay_db::payments::create_payment(
        &state.db,
        merchant.0.id,
        amount,
        &req.currency,
        idempotency_key,
        metadata.clone(),
        withdrawal_address,
    )
    .await;

    match create_payment_result {
        Ok(new_payment) => Ok(Json(to_create_payment_response(&new_payment))),
        Err(err) if is_idempotency_race(&err) && idempotency_key.is_some() => {
            let existing = cryptopay_db::payments::find_payment_by_idempotency_key(
                &state.db,
                merchant.0.id,
                idempotency_key.expect("checked is_some above"),
            )
            .await
            .map_err(|error| {
                tracing::error!(
                    "Failed to reload idempotent payment after unique violation for merchant {}: {}",
                    merchant.0.id,
                    error
                );
                AppError::Internal(error)
            })?
            .ok_or_else(|| AppError::Internal(err))?;

            if existing.amount != amount
                || existing.currency != req.currency
                || existing.metadata != metadata
            {
                return Err(AppError::Conflict(
                    "idempotency_key already used with different payment parameters".to_string(),
                ));
            }

            Ok(Json(to_create_payment_response(&existing)))
        }
        Err(err) => {
            tracing::error!(
                "Failed to create payment for merchant {}: {:?}",
                merchant.0.id,
                &err
            );
            let db_err_msg = err.to_string();
            tracing::error!("DB error detail: {}", db_err_msg);
            Err(AppError::Internal(err))
        }
    }
}

async fn list_payments(
    State(state): State<Arc<AppState>>,
    merchant: AuthenticatedMerchant,
    Query(query): Query<ListPaymentsQuery>,
) -> Result<Json<ListPaymentsResponse>, AppError> {
    let limit = query.limit.unwrap_or(20).clamp(1, 100);
    let page = query.page.unwrap_or(1).max(1);
    let offset = (page - 1) * limit;

    let status_filter = query.status.as_deref();
    if let Some(status) = status_filter {
        status.parse::<PaymentStatus>().map_err(AppError::BadRequest)?;
    }

    tracing::info!("Listing payments for merchant {}: status={:?}, limit={}, offset={}", merchant.0.id, status_filter, limit, offset);

    let payments =
        cryptopay_db::payments::list_payments(&state.db, merchant.0.id, status_filter, limit, offset)
            .await
            .map_err(|e| {
                tracing::error!("list_payments failed for merchant {}: {:?}", merchant.0.id, e);
                AppError::Internal(e)
            })?;

    let total = cryptopay_db::payments::count_payments(&state.db, merchant.0.id, status_filter)
        .await
        .map_err(|e| {
            tracing::error!("count_payments failed for merchant {}: {:?}", merchant.0.id, e);
            AppError::Internal(e)
        })?;

    let pages = (total + limit - 1) / limit;

    Ok(Json(ListPaymentsResponse {
        data: payments.iter().map(to_public_payment_response).collect(),
        pagination: PaginationInfo {
            total,
            page,
            limit,
            pages,
        },
    }))
}

async fn get_payment(
    State(state): State<Arc<AppState>>,
    merchant: AuthenticatedMerchant,
    Path(payment_id): Path<uuid::Uuid>,
) -> Result<Json<MerchantPaymentResponse>, AppError> {
    tracing::info!("Getting payment {} for merchant {}", payment_id, merchant.0.id);
    let payment =
        cryptopay_db::payments::find_payment_by_id_and_merchant(&state.db, payment_id, merchant.0.id)
            .await
            .map_err(|e| {
                tracing::error!("find_payment_by_id_and_merchant failed for payment {}: {:?}", payment_id, e);
                AppError::Internal(e)
            })?
            .ok_or(AppError::NotFound("Payment not found".to_string()))?;

    Ok(Json(to_public_payment_response(&payment)))
}

fn to_create_payment_response(payment: &Payment) -> CreatePaymentResponse {
    CreatePaymentResponse {
        payment_id: payment.id,
        deposit_address: payment.deposit_address.clone(),
        amount: payment.amount.to_string(),
        currency: payment.currency.clone(),
        status: payment.status.clone(),
        expires_at: payment.expires_at.to_rfc3339(),
    }
}

fn to_public_payment_response(payment: &Payment) -> MerchantPaymentResponse {
    MerchantPaymentResponse {
        payment_id: payment.id,
        amount: payment.amount.to_string(),
        currency: payment.currency.clone(),
        deposit_address: payment.deposit_address.clone(),
        status: payment.status.clone(),
        tx_hash: payment.tx_hash.clone(),
        confirmations: payment.confirmations,
        required_confirmations: payment.required_confirmations,
        idempotency_key: payment.idempotency_key.clone(),
        metadata: payment.metadata.clone(),
        expires_at: payment.expires_at.to_rfc3339(),
        confirmed_at: payment.confirmed_at.map(|value| value.to_rfc3339()),
        created_at: payment.created_at.to_rfc3339(),
    }
}

fn is_idempotency_race(error: &anyhow::Error) -> bool {
    matches!(
        error.downcast_ref::<sqlx::Error>(),
        Some(sqlx::Error::Database(db_error)) if db_error.code().as_deref() == Some("23505")
    )
}
