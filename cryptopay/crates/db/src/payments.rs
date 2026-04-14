use bs58;
use chrono::{Duration, Utc};
use hmac::{Hmac, Mac};
use secp256k1::{PublicKey, Secp256k1, SecretKey};
use sha2::{Digest, Sha256};
use sha3::Keccak256;
use sqlx::PgPool;
use uuid::Uuid;

use cryptopay_core::{Payment, PaymentStatus};

type HmacSha256 = Hmac<Sha256>;

const PAYMENT_COLUMNS: &str = r#"
    id, merchant_id, amount, currency, deposit_address, status,
    tx_hash, confirmations, required_confirmations,
    idempotency_key, metadata, expires_at, confirmed_at, created_at,
    sweep_status, sweep_tx_hash, swept_at, sweep_error
"#;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DerivedDepositWallet {
    pub payment_id: Uuid,
    pub deposit_address: String,
    pub private_key_hex: String,
}

pub fn derive_deposit_wallet(
    master_wallet_private_key: &str,
    payment_id: Uuid,
) -> anyhow::Result<DerivedDepositWallet> {
    let secret_key = derive_child_secret_key(master_wallet_private_key, payment_id)?;
    let secp = Secp256k1::new();
    let public_key = PublicKey::from_secret_key(&secp, &secret_key);
    let uncompressed_public_key = public_key.serialize_uncompressed();
    let hashed_public_key = Keccak256::digest(&uncompressed_public_key[1..]);

    let mut address_bytes = [0u8; 21];
    address_bytes[0] = 0x41;
    address_bytes[1..].copy_from_slice(&hashed_public_key[12..]);

    let checksum = Sha256::digest(Sha256::digest(address_bytes));
    let mut full = address_bytes.to_vec();
    full.extend_from_slice(&checksum[..4]);

    Ok(DerivedDepositWallet {
        payment_id,
        deposit_address: bs58::encode(full).into_string(),
        private_key_hex: hex::encode(secret_key.secret_bytes()),
    })
}

fn derive_child_secret_key(
    master_wallet_private_key: &str,
    payment_id: Uuid,
) -> anyhow::Result<SecretKey> {
    let master_key_bytes = parse_master_private_key(master_wallet_private_key)?;

    for counter in 0u32..=u32::MAX {
        let mut mac =
            HmacSha256::new_from_slice(&master_key_bytes).expect("HMAC can take keys of any size");
        mac.update(b"cryptopay:tron:deposit");
        mac.update(payment_id.as_bytes());
        mac.update(&counter.to_be_bytes());

        let candidate = mac.finalize().into_bytes();
        if let Ok(secret_key) = SecretKey::from_slice(&candidate) {
            return Ok(secret_key);
        }
    }

    Err(anyhow::anyhow!(
        "failed to derive a valid deposit private key for payment {}",
        payment_id
    ))
}

fn parse_master_private_key(master_wallet_private_key: &str) -> anyhow::Result<[u8; 32]> {
    let trimmed = master_wallet_private_key.trim().trim_start_matches("0x");
    let decoded = hex::decode(trimmed)
        .map_err(|error| anyhow::anyhow!("invalid MASTER_WALLET_PRIVATE_KEY hex: {}", error))?;

    if decoded.len() != 32 {
        return Err(anyhow::anyhow!(
            "MASTER_WALLET_PRIVATE_KEY must be exactly 32 bytes"
        ));
    }

    let mut key_bytes = [0u8; 32];
    key_bytes.copy_from_slice(&decoded);
    Ok(key_bytes)
}

pub async fn create_payment(
    pool: &PgPool,
    merchant_id: Uuid,
    amount: rust_decimal::Decimal,
    currency: &str,
    idempotency_key: Option<&str>,
    metadata: serde_json::Value,
    master_wallet_private_key: &str,
) -> anyhow::Result<Payment> {
    let payment_id = Uuid::new_v4();
    let derived_wallet = derive_deposit_wallet(master_wallet_private_key, payment_id)?;
    let expires_at = Utc::now() + Duration::minutes(30);
    let status = PaymentStatus::Pending;

    let query = format!(
        r#"
        INSERT INTO payments (
            id, merchant_id, amount, currency, deposit_address, status,
            idempotency_key, metadata, expires_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        RETURNING {}
        "#,
        PAYMENT_COLUMNS
    );

    let payment = sqlx::query_as::<_, Payment>(&query)
        .bind(payment_id)
        .bind(merchant_id)
        .bind(amount)
        .bind(currency)
        .bind(&derived_wallet.deposit_address)
        .bind(status.to_string())
        .bind(idempotency_key)
        .bind(metadata)
        .bind(expires_at)
        .fetch_one(pool)
        .await?;

    Ok(payment)
}

pub async fn find_payment_by_id(pool: &PgPool, id: Uuid) -> anyhow::Result<Option<Payment>> {
    let query = format!(
        r#"
        SELECT {}
        FROM payments
        WHERE id = $1
        "#,
        PAYMENT_COLUMNS
    );

    let payment = sqlx::query_as::<_, Payment>(&query)
        .bind(id)
        .fetch_optional(pool)
        .await?;

    Ok(payment)
}

pub async fn find_payment_by_id_and_merchant(
    pool: &PgPool,
    id: Uuid,
    merchant_id: Uuid,
) -> anyhow::Result<Option<Payment>> {
    let query = format!(
        r#"
        SELECT {}
        FROM payments
        WHERE id = $1 AND merchant_id = $2
        "#,
        PAYMENT_COLUMNS
    );

    let payment = sqlx::query_as::<_, Payment>(&query)
        .bind(id)
        .bind(merchant_id)
        .fetch_optional(pool)
        .await?;

    Ok(payment)
}

pub async fn find_payment_by_idempotency_key(
    pool: &PgPool,
    merchant_id: Uuid,
    key: &str,
) -> anyhow::Result<Option<Payment>> {
    let query = format!(
        r#"
        SELECT {}
        FROM payments
        WHERE merchant_id = $1 AND idempotency_key = $2
        "#,
        PAYMENT_COLUMNS
    );

    let payment = sqlx::query_as::<_, Payment>(&query)
        .bind(merchant_id)
        .bind(key)
        .fetch_optional(pool)
        .await?;

    Ok(payment)
}

pub async fn list_payments(
    pool: &PgPool,
    merchant_id: Uuid,
    status_filter: Option<&str>,
    limit: i64,
    offset: i64,
) -> anyhow::Result<Vec<Payment>> {
    let mut query = format!(
        r#"
        SELECT {}
        FROM payments
        WHERE merchant_id = $1
        "#,
        PAYMENT_COLUMNS
    );

    let mut bind_count = 1;
    if status_filter.is_some() {
        query.push_str(&format!(" AND status = ${}", bind_count + 1));
        bind_count += 1;
    }

    query.push_str(&format!(
        " ORDER BY created_at DESC LIMIT ${} OFFSET ${}",
        bind_count + 1,
        bind_count + 2
    ));

    let mut sql_query = sqlx::query_as::<_, Payment>(&query).bind(merchant_id);

    if let Some(status) = status_filter {
        sql_query = sql_query.bind(status);
    }

    let payments = sql_query.bind(limit).bind(offset).fetch_all(pool).await?;

    Ok(payments)
}

pub async fn count_payments(
    pool: &PgPool,
    merchant_id: Uuid,
    status_filter: Option<&str>,
) -> anyhow::Result<i64> {
    let mut query = "SELECT COUNT(*) FROM payments WHERE merchant_id = $1".to_string();

    if status_filter.is_some() {
        query.push_str(" AND status = $2");
    }

    let mut sql_query = sqlx::query_scalar::<_, i64>(&query).bind(merchant_id);

    if let Some(status) = status_filter {
        sql_query = sql_query.bind(status);
    }

    let count = sql_query.fetch_one(pool).await?;
    Ok(count)
}

pub async fn list_sweep_candidates(pool: &PgPool, limit: i64) -> anyhow::Result<Vec<Payment>> {
    let query = format!(
        r#"
        SELECT {}
        FROM payments
        WHERE status = 'confirmed' AND sweep_status IN ('not_swept', 'sweep_failed')
        ORDER BY confirmed_at ASC NULLS LAST, created_at ASC
        LIMIT $1
        "#,
        PAYMENT_COLUMNS
    );

    let payments = sqlx::query_as::<_, Payment>(&query)
        .bind(limit.clamp(1, 500))
        .fetch_all(pool)
        .await?;

    Ok(payments)
}

pub async fn mark_payment_swept(
    pool: &PgPool,
    id: Uuid,
    sweep_tx_hash: &str,
) -> anyhow::Result<Payment> {
    let current_payment = find_payment_by_id(pool, id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Payment {} not found", id))?;

    if current_payment.sweep_status == "swept" {
        if current_payment.sweep_tx_hash.as_deref() == Some(sweep_tx_hash) {
            return Ok(current_payment);
        }

        return Err(anyhow::anyhow!(
            "Payment {} is already marked as swept with tx_hash {:?}",
            id,
            current_payment.sweep_tx_hash
        ));
    }

    let query = format!(
        r#"
        UPDATE payments
        SET sweep_status = 'swept', sweep_tx_hash = $1, swept_at = NOW(), sweep_error = NULL
        WHERE id = $2
        RETURNING {}
        "#,
        PAYMENT_COLUMNS
    );

    let payment = sqlx::query_as::<_, Payment>(&query)
        .bind(sweep_tx_hash)
        .bind(id)
        .fetch_one(pool)
        .await?;

    Ok(payment)
}

pub async fn mark_payment_sweep_failed(
    pool: &PgPool,
    id: Uuid,
    reason: &str,
) -> anyhow::Result<Payment> {
    let current_payment = find_payment_by_id(pool, id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Payment {} not found", id))?;

    if current_payment.sweep_status == "swept" {
        return Err(anyhow::anyhow!(
            "Payment {} is already marked as swept and cannot be moved back to sweep_failed",
            id
        ));
    }

    let query = format!(
        r#"
        UPDATE payments
        SET sweep_status = 'sweep_failed', sweep_tx_hash = NULL, swept_at = NULL, sweep_error = $1
        WHERE id = $2
        RETURNING {}
        "#,
        PAYMENT_COLUMNS
    );

    let payment = sqlx::query_as::<_, Payment>(&query)
        .bind(reason)
        .bind(id)
        .fetch_one(pool)
        .await?;

    Ok(payment)
}

pub async fn expire_old_payments(pool: &PgPool) -> anyhow::Result<u64> {
    let result = sqlx::query(
        r#"
        UPDATE payments
        SET status = 'expired'
        WHERE status = 'pending' AND expires_at < NOW()
        "#,
    )
    .execute(pool)
    .await?;

    Ok(result.rows_affected())
}

pub async fn find_pending_payments(pool: &PgPool) -> anyhow::Result<Vec<Payment>> {
    let query = format!(
        r#"
        SELECT {}
        FROM payments
        WHERE status = 'pending' AND expires_at > NOW()
        ORDER BY created_at ASC
        "#,
        PAYMENT_COLUMNS
    );

    let payments = sqlx::query_as::<_, Payment>(&query)
        .fetch_all(pool)
        .await?;

    Ok(payments)
}

pub async fn find_detected_payments(pool: &PgPool) -> anyhow::Result<Vec<Payment>> {
    let query = format!(
        r#"
        SELECT {}
        FROM payments
        WHERE status IN ('detected', 'confirming') AND tx_hash IS NOT NULL
        ORDER BY created_at ASC
        "#,
        PAYMENT_COLUMNS
    );

    let payments = sqlx::query_as::<_, Payment>(&query)
        .fetch_all(pool)
        .await?;

    Ok(payments)
}

pub async fn update_payment_confirmations(
    pool: &PgPool,
    id: Uuid,
    confirmations: i32,
    status: &str,
) -> anyhow::Result<()> {
    sqlx::query(
        r#"
        UPDATE payments
        SET confirmations = $1, status = $2
        WHERE id = $3
        "#,
    )
    .bind(confirmations)
    .bind(status)
    .bind(id)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn mark_payment_confirmed(
    pool: &PgPool,
    id: Uuid,
    confirmations: i32,
) -> anyhow::Result<()> {
    sqlx::query(
        r#"
        UPDATE payments
        SET status = 'confirmed', confirmations = $1, confirmed_at = NOW()
        WHERE id = $2
        "#,
    )
    .bind(confirmations)
    .bind(id)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn mark_payment_detected(
    pool: &PgPool,
    id: Uuid,
    tx_hash: &str,
) -> anyhow::Result<()> {
    let current_payment = find_payment_by_id(pool, id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Payment {} not found", id))?;

    if current_payment.status != "pending" {
        return Err(anyhow::anyhow!(
            "Cannot mark payment {} as detected: current status is '{}' (not 'pending'). Existing tx_hash: {:?}",
            id,
            current_payment.status,
            current_payment.tx_hash
        ));
    }

    let result = sqlx::query(
        r#"
        UPDATE payments
        SET status = 'detected', tx_hash = $1
        WHERE id = $2 AND status = 'pending'
        "#,
    )
    .bind(tx_hash)
    .bind(id)
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(anyhow::anyhow!(
            "Failed to update payment {}: no rows affected (payment may have been updated by another process)",
            id
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use sha2::{Digest, Sha256};
    use uuid::Uuid;

    use super::derive_deposit_wallet;

    #[test]
    fn generated_address_is_deterministic_and_valid_base58check() {
        let master_wallet_private_key =
            "4f3edf983ac636a65a842ce7c78d9aa706d3b113bce036f6a0876fddf2d6a8fe";
        let payment_id =
            Uuid::parse_str("11111111-2222-3333-4444-555555555555").expect("valid uuid");

        let wallet = derive_deposit_wallet(master_wallet_private_key, payment_id)
            .expect("address should be generated");
        let decoded = bs58::decode(&wallet.deposit_address)
            .into_vec()
            .expect("valid base58");

        assert!(wallet.deposit_address.starts_with('T'));
        assert_eq!(decoded.len(), 25);
        assert_eq!(decoded[0], 0x41);
        assert_eq!(wallet.private_key_hex.len(), 64);

        let checksum = Sha256::digest(Sha256::digest(&decoded[..21]));
        assert_eq!(&decoded[21..], &checksum[..4]);
        assert_eq!(
            wallet,
            derive_deposit_wallet(master_wallet_private_key, payment_id)
                .expect("address should be stable")
        );
        assert_ne!(
            wallet.deposit_address,
            derive_deposit_wallet(master_wallet_private_key, Uuid::new_v4())
                .expect("different payment ids must produce different addresses")
                .deposit_address
        );
    }
}
