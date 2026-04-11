use std::sync::Arc;

use axum::{
    routing::{get, post},
    Router, Json, extract::State,
};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::AppState;
use super::super::AppError;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/merchants/register", post(register_merchant))
        .route("/api/merchants", get(list_merchants))
}

#[derive(serde::Deserialize)]
struct RegisterRequest {
    name: String,
    email: String,
}

#[derive(serde::Serialize)]
struct RegisterResponse {
    merchant_id: uuid::Uuid,
    api_key: String,
    webhook_secret: String,
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

    if cryptopay_db::merchants::find_merchant_by_email(&state.db, &req.email)
        .await
        .map_err(AppError::Internal)?
        .is_some()
    {
        return Err(AppError::Conflict(format!("email {} already registered", req.email)));
    }

    let api_key = format!("cpay_live_{}", Uuid::new_v4().to_string().replace('-', ""));
    let webhook_secret = Uuid::new_v4().to_string();

    let api_key_hash = hex::encode(Sha256::digest(api_key.as_bytes()));

    let merchant = cryptopay_db::merchants::create_merchant(
        &state.db,
        &req.name,
        &req.email,
        &api_key_hash,
        &webhook_secret,
    )
    .await
    .map_err(AppError::Internal)?;

    Ok(Json(RegisterResponse {
        merchant_id: merchant.id,
        api_key,
        webhook_secret,
    }))
}

async fn list_merchants() -> &'static str {
    "merchants"
}
