use std::sync::Arc;

use axum::{
    extract::State,
    routing::{get, post, put},
    Json, Router,
};
use uuid::Uuid;

use crate::{hash_api_key, AppError, AppState, AuthenticatedMerchant};

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/merchants/register", post(register_merchant))
        .route("/v1/merchants/me", get(get_current_merchant))
        .route("/v1/merchants/me/webhook", put(update_webhook).delete(clear_webhook))
        .route("/v1/merchants/me/rotate-api-key", post(rotate_api_key))
        .route(
            "/v1/merchants/me/rotate-webhook-secret",
            post(rotate_webhook_secret),
        )
        .route("/v1/merchants/settings", post(update_settings))
        .route("/api/merchants", get(get_current_merchant))
}

#[derive(serde::Deserialize)]
struct RegisterRequest {
    name: String,
    email: String,
    password: String,
    webhook_url: Option<String>,
}

#[derive(serde::Serialize)]
struct RegisterResponse {
    merchant_id: uuid::Uuid,
    api_key: String,
    webhook_secret: String,
}

#[derive(serde::Serialize)]
struct MerchantProfileResponse {
    merchant_id: uuid::Uuid,
    name: String,
    email: String,
    withdrawal_address: Option<String>,
    webhook_url: Option<String>,
    fee_percent: String,
    is_active: bool,
    created_at: String,
}

#[derive(serde::Deserialize)]
struct UpdateWebhookRequest {
    webhook_url: String,
}

#[derive(serde::Serialize)]
struct RotateApiKeyResponse {
    api_key: String,
}

#[derive(serde::Serialize)]
struct RotateWebhookSecretResponse {
    webhook_secret: String,
}

#[derive(serde::Deserialize)]
struct UpdateSettingsRequest {
    withdrawal_address: String,
}

#[derive(serde::Serialize)]
struct UpdateSettingsResponse {
    saved: bool,
}

async fn register_merchant(
    State(state): State<Arc<AppState>>,
    Json(req): Json<RegisterRequest>,
) -> Result<Json<RegisterResponse>, AppError> {
    if req.name.trim().is_empty() {
        return Err(AppError::BadRequest("name must not be empty".to_string()));
    }
    if req.email.trim().is_empty() {
        return Err(AppError::BadRequest("email must not be empty".to_string()));
    }
    if req.password.is_empty() {
        return Err(AppError::BadRequest("password must not be empty".to_string()));
    }
    if req.password.len() < 8 {
        return Err(AppError::BadRequest("password must be at least 8 characters".to_string()));
    }
    if let Some(webhook_url) = req.webhook_url.as_deref() {
        validate_webhook_url(webhook_url)?;
    }

    let email = req.email.trim().to_lowercase();

    if cryptopay_db::merchants::find_merchant_by_email(&state.db, &email)
        .await
        .map_err(AppError::Internal)?
        .is_some()
    {
        return Err(AppError::Conflict(format!(
            "email {} already registered",
            email
        )));
    }

    let password_hash = bcrypt::hash(&req.password, bcrypt::DEFAULT_COST)
        .map_err(|e| {
            tracing::error!("bcrypt hash failed: {:?}", e);
            AppError::Internal(anyhow::anyhow!("password hashing failed"))
        })?;

    let api_key = format!("cpay_live_{}", Uuid::new_v4().to_string().replace('-', ""));
    let webhook_secret = Uuid::new_v4().to_string();
    let api_key_hash = hash_api_key(&api_key);

    let merchant = cryptopay_db::merchants::create_merchant(
        &state.db,
        &req.name,
        &email,
        &api_key_hash,
        &webhook_secret,
        req.webhook_url.as_deref(),
        Some(&password_hash),
    )
    .await
    .map_err(AppError::Internal)?;

    Ok(Json(RegisterResponse {
        merchant_id: merchant.id,
        api_key,
        webhook_secret,
    }))
}

async fn get_current_merchant(
    merchant: AuthenticatedMerchant,
) -> Json<MerchantProfileResponse> {
    Json(to_profile_response(&merchant.0))
}

async fn update_webhook(
    State(state): State<Arc<AppState>>,
    merchant: AuthenticatedMerchant,
    Json(req): Json<UpdateWebhookRequest>,
) -> Result<Json<MerchantProfileResponse>, AppError> {
    validate_webhook_url(&req.webhook_url)?;

    let merchant = cryptopay_db::merchants::update_merchant_webhook_url(
        &state.db,
        merchant.0.id,
        Some(req.webhook_url.trim()),
    )
    .await
    .map_err(AppError::Internal)?;

    Ok(Json(to_profile_response(&merchant)))
}

async fn clear_webhook(
    State(state): State<Arc<AppState>>,
    merchant: AuthenticatedMerchant,
) -> Result<Json<MerchantProfileResponse>, AppError> {
    let merchant =
        cryptopay_db::merchants::update_merchant_webhook_url(&state.db, merchant.0.id, None)
            .await
            .map_err(AppError::Internal)?;

    Ok(Json(to_profile_response(&merchant)))
}

async fn rotate_api_key(
    State(state): State<Arc<AppState>>,
    merchant: AuthenticatedMerchant,
) -> Result<Json<RotateApiKeyResponse>, AppError> {
    let api_key = format!("cpay_live_{}", Uuid::new_v4().to_string().replace('-', ""));
    let api_key_hash = hash_api_key(&api_key);

    cryptopay_db::merchants::update_merchant_api_key_hash(&state.db, merchant.0.id, &api_key_hash)
        .await
        .map_err(AppError::Internal)?;

    Ok(Json(RotateApiKeyResponse { api_key }))
}

async fn rotate_webhook_secret(
    State(state): State<Arc<AppState>>,
    merchant: AuthenticatedMerchant,
) -> Result<Json<RotateWebhookSecretResponse>, AppError> {
    let webhook_secret = Uuid::new_v4().to_string();

    cryptopay_db::merchants::update_merchant_webhook_secret(
        &state.db,
        merchant.0.id,
        &webhook_secret,
    )
    .await
    .map_err(AppError::Internal)?;

    Ok(Json(RotateWebhookSecretResponse { webhook_secret }))
}

async fn update_settings(
    State(state): State<Arc<AppState>>,
    merchant: AuthenticatedMerchant,
    Json(req): Json<UpdateSettingsRequest>,
) -> Result<Json<UpdateSettingsResponse>, AppError> {
    let addr = req.withdrawal_address.trim();

    if !addr.starts_with('T') {
        return Err(AppError::BadRequest(
            "withdrawal_address must start with 'T'".to_string(),
        ));
    }

    if addr.len() != 34 {
        return Err(AppError::BadRequest(
            "withdrawal_address must be exactly 34 characters".to_string(),
        ));
    }

    cryptopay_db::merchants::update_withdrawal_address(&state.db, merchant.0.id, addr)
        .await
        .map_err(AppError::Internal)?;

    Ok(Json(UpdateSettingsResponse { saved: true }))
}

fn to_profile_response(merchant: &cryptopay_core::Merchant) -> MerchantProfileResponse {
    MerchantProfileResponse {
        merchant_id: merchant.id,
        name: merchant.name.clone(),
        email: merchant.email.clone(),
        withdrawal_address: merchant.withdrawal_address.clone(),
        webhook_url: merchant.webhook_url.clone(),
        fee_percent: merchant.fee_percent.to_string(),
        is_active: merchant.is_active,
        created_at: merchant.created_at.to_rfc3339(),
    }
}

fn validate_webhook_url(url: &str) -> Result<(), AppError> {
    let trimmed = url.trim();
    if trimmed.is_empty() {
        return Err(AppError::BadRequest(
            "webhook_url must not be empty".to_string(),
        ));
    }

    if !trimmed.starts_with("https://") && !trimmed.starts_with("http://") {
        return Err(AppError::BadRequest(
            "webhook_url must start with http:// or https://".to_string(),
        ));
    }

    Ok(())
}
