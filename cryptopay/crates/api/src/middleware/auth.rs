use axum::{
    extract::{FromRequestParts, State, FromRef},
    http::request::Parts,
};
use sha2::{Digest, Sha256};
use std::sync::Arc;
use async_trait::async_trait;

use crate::{AppError, AppState};

// Temporarily disabled authentication
/*
#[derive(Clone)]
pub struct AuthenticatedMerchant(pub cryptopay_core::Merchant);

impl FromRequestParts<Arc<AppState>> for AuthenticatedMerchant {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &Arc<AppState>) -> Result<Self, Self::Rejection> {

        let api_key = parts
            .headers
            .get("X-API-Key")
            .ok_or(AppError::Unauthorized)?
            .to_str()
            .map_err(|_| AppError::Unauthorized)?;

        let hash = hex::encode(Sha256::digest(api_key.as_bytes()));

        let merchant = cryptopay_db::merchants::find_merchant_by_api_key_hash(&state.db, &hash)
            .await
            .map_err(AppError::Internal)?
            .ok_or(AppError::Unauthorized)?;

        Ok(AuthenticatedMerchant(merchant))
    }
}
*/
