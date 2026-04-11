use axum::{
    extract::{FromRequestParts, State, FromRef},
    http::request::Parts,
};
use sha2::{Digest, Sha256};
use std::sync::Arc;

use crate::{AppError, AppState};

#[derive(Clone)]
pub struct AuthenticatedMerchant(pub cryptopay_core::Merchant);

#[async_trait::async_trait]
impl<S: Send + Sync> FromRequestParts<S> for AuthenticatedMerchant
where
    Arc<AppState>: FromRef<S>,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let State(app_state): State<Arc<AppState>> = State::from_request_parts(parts, state)
            .await
            .map_err(|_| AppError::Internal(anyhow::anyhow!("state not found")))?;

        let api_key = parts
            .headers
            .get("X-API-Key")
            .ok_or(AppError::Unauthorized)?
            .to_str()
            .map_err(|_| AppError::Unauthorized)?;

        let hash = hex::encode(Sha256::digest(api_key.as_bytes()));

        let merchant = cryptopay_db::merchants::find_merchant_by_api_key_hash(&app_state.db, &hash)
            .await
            .map_err(AppError::Internal)?
            .ok_or(AppError::Unauthorized)?;

        Ok(AuthenticatedMerchant(merchant))
    }
}
