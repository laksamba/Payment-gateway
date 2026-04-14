pub mod payment;
pub mod money;
pub mod errors;

pub use payment::{Payment, PaymentStatus};
pub use errors::AppError;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Merchant {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub api_key_hash: String,
    pub withdrawal_address: Option<String>,
    pub webhook_url: Option<String>,
    pub webhook_secret: String,
    pub fee_percent: rust_decimal::Decimal,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}
