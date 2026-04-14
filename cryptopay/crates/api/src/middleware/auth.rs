use std::sync::Arc;

use async_trait::async_trait;
use axum::{
    extract::FromRequestParts,
    http::request::Parts,
};

use crate::{AppError, AppState};

#[derive(Clone)]
pub struct AuthenticatedMerchant(pub cryptopay_core::Merchant);

#[async_trait]
impl FromRequestParts<Arc<AppState>> for AuthenticatedMerchant {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<AppState>,
    ) -> Result<Self, Self::Rejection> {
        let api_key = parts
            .headers
            .get("X-API-Key")
            .ok_or(AppError::Unauthorized)?
            .to_str()
            .map_err(|_| AppError::Unauthorized)?;

        let hash = crate::hash_api_key(api_key);

        let merchant = cryptopay_db::merchants::find_merchant_by_api_key_hash(&state.db, &hash)
            .await
            .map_err(|e| {
                tracing::error!("find_merchant_by_api_key_hash failed: {:?}", e);
                AppError::Internal(e)
            })?
            .ok_or_else(|| {
                tracing::warn!("No merchant found for API key hash");
                AppError::Unauthorized
            })?;

        if !merchant.is_active {
            return Err(AppError::Unauthorized);
        }

        Ok(AuthenticatedMerchant(merchant))
    }
}
