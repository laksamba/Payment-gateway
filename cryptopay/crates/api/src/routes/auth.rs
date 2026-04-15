use std::sync::Arc;

use axum::{
    extract::State,
    routing::post,
    Json, Router,
};

use crate::{AppError, AppState};

pub fn routes() -> Router<Arc<AppState>> {
    Router::new().route("/v1/auth/login", post(login))
}

#[derive(serde::Deserialize)]
struct LoginRequest {
    email: String,
    password: String,
}

#[derive(serde::Serialize)]
struct LoginResponse {
    api_key: String,
}

async fn login(
    State(state): State<Arc<AppState>>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, AppError> {
    let email = req.email.trim().to_lowercase();

    if email.is_empty() {
        return Err(AppError::BadRequest("email must not be empty".to_string()));
    }
    if req.password.is_empty() {
        return Err(AppError::BadRequest("password must not be empty".to_string()));
    }

    let merchant = cryptopay_db::merchants::find_merchant_by_email(&state.db, &email)
        .await
        .map_err(AppError::Internal)?
        .ok_or_else(|| {
            tracing::warn!("Login failed: no merchant found for email {}", email);
            AppError::Unauthorized
        })?;

    let password_hash = merchant.password_hash.as_deref().ok_or_else(|| {
        tracing::warn!("Login failed: no password set for merchant {}", merchant.id);
        AppError::Unauthorized
    })?;

    // Verify password using bcrypt
    let is_valid = bcrypt::verify(&req.password, password_hash)
        .map_err(|e| {
            tracing::error!("bcrypt verify failed: {:?}", e);
            AppError::Internal(anyhow::anyhow!("password verification failed"))
        })?;

    if !is_valid {
        tracing::warn!("Login failed: invalid password for merchant {}", merchant.id);
        return Err(AppError::Unauthorized);
    }

    if !merchant.is_active {
        return Err(AppError::Unauthorized);
    }

    // NOTE: We don't return the API key in plaintext here for security.
    // The API key is stored hashed, so we need to generate a new one or
    // the frontend needs to use the existing hashed key approach.
    // Actually, looking at the current flow, the API key IS the secret.
    // For password login, we should return something usable.
    // Since we hash the API key for storage, we need to either:
    // 1. Store API key in plaintext (encrypted at minimum)
    // 2. Generate new API key on login
    // Let's generate a new API key on each login for simplicity.
    // The merchant can rotate it later if needed.

    let new_api_key = format!("cpay_live_{}", uuid::Uuid::new_v4().to_string().replace('-', ""));
    let new_api_key_hash = crate::hash_api_key(&new_api_key);

    cryptopay_db::merchants::update_merchant_api_key_hash(&state.db, merchant.id, &new_api_key_hash)
        .await
        .map_err(AppError::Internal)?;

    tracing::info!("Merchant {} logged in successfully", merchant.id);

    Ok(Json(LoginResponse { api_key: new_api_key }))
}