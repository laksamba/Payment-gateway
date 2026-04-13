use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::fmt;
use uuid::Uuid;

#[derive(Debug, Clone, sqlx::FromRow, Serialize, Deserialize)]
pub struct Payment {
    pub id: Uuid,
    pub merchant_id: Uuid,
    pub amount: Decimal,
    pub currency: String,
    pub deposit_address: String,
    pub status: String,
    pub tx_hash: Option<String>,
    pub confirmations: i32,
    pub required_confirmations: i32,
    pub idempotency_key: Option<String>,
    pub metadata: serde_json::Value,
    pub expires_at: DateTime<Utc>,
    pub confirmed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PaymentStatus {
    Pending,
    Detected,
    Confirming,
    Confirmed,
    Expired,
    Failed,
}

impl fmt::Display for PaymentStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PaymentStatus::Pending => write!(f, "pending"),
            PaymentStatus::Detected => write!(f, "detected"),
            PaymentStatus::Confirming => write!(f, "confirming"),
            PaymentStatus::Confirmed => write!(f, "confirmed"),
            PaymentStatus::Expired => write!(f, "expired"),
            PaymentStatus::Failed => write!(f, "failed"),
        }
    }
}

impl FromStr for PaymentStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "pending" => Ok(PaymentStatus::Pending),
            "detected" => Ok(PaymentStatus::Detected),
            "confirming" => Ok(PaymentStatus::Confirming),
            "confirmed" => Ok(PaymentStatus::Confirmed),
            "expired" => Ok(PaymentStatus::Expired),
            "failed" => Ok(PaymentStatus::Failed),
            _ => Err(format!("unknown status: {}", s)),
        }
    }
}
