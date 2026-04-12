use bs58;
use chrono::{Duration, Utc};
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use uuid::Uuid;

use cryptopay_core::{Payment, PaymentStatus};

fn generate_deposit_address() -> String {
    let mut address_bytes = [0u8; 21];
    address_bytes[0] = 0x41; // Tron address version byte

    let u1 = Uuid::new_v4();
    let u2 = Uuid::new_v4();
    let uuid1 = u1.as_bytes();
    let uuid2 = u2.as_bytes();
    address_bytes[1..17].copy_from_slice(uuid1);
    address_bytes[17..21].copy_from_slice(&uuid2[..4]);

    let checksum = Sha256::digest(&Sha256::digest(&address_bytes));
    let mut full = address_bytes.to_vec();
    full.extend_from_slice(&checksum[..4]);

    bs58::encode(full).into_string()
}

pub async fn create_payment(
    pool: &PgPool,
    merchant_id: Uuid,
    amount: rust_decimal::Decimal,
    currency: &str,
    idempotency_key: Option<&str>,
    metadata: serde_json::Value,
) -> anyhow::Result<Payment> {
    let deposit_address = generate_deposit_address();
    let expires_at = Utc::now() + Duration::minutes(30);
    let status = PaymentStatus::Pending;

    let payment = sqlx::query_as::<_, Payment>(
        r#"
        INSERT INTO payments (
            merchant_id, amount, currency, deposit_address, status,
            idempotency_key, metadata, expires_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING id, merchant_id, amount, currency, deposit_address, status,
                  tx_hash, confirmations, required_confirmations,
                  idempotency_key, metadata, expires_at, confirmed_at, created_at
        "#,
    )
    .bind(merchant_id)
    .bind(amount)
    .bind(currency)
    .bind(&deposit_address)
    .bind(status.to_string())
    .bind(idempotency_key)
    .bind(metadata)
    .bind(expires_at)
    .fetch_one(pool)
    .await?;

    Ok(payment)
}

pub async fn find_payment_by_id(
    pool: &PgPool,
    id: Uuid,
) -> anyhow::Result<Option<Payment>> {
    let payment = sqlx::query_as::<_, Payment>(
        r#"
        SELECT id, merchant_id, amount, currency, deposit_address, status,
               tx_hash, confirmations, required_confirmations,
               idempotency_key, metadata, expires_at, confirmed_at, created_at
        FROM payments
        WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;

    Ok(payment)
}

pub async fn find_payment_by_idempotency_key(
    pool: &PgPool,
    merchant_id: Uuid,
    key: &str,
) -> anyhow::Result<Option<Payment>> {
    let payment = sqlx::query_as::<_, Payment>(
        r#"
        SELECT id, merchant_id, amount, currency, deposit_address, status,
               tx_hash, confirmations, required_confirmations,
               idempotency_key, metadata, expires_at, confirmed_at, created_at
        FROM payments
        WHERE merchant_id = $1 AND idempotency_key = $2
        "#,
    )
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
    let mut query = r#"
        SELECT id, merchant_id, amount, currency, deposit_address, status,
               tx_hash, confirmations, required_confirmations,
               idempotency_key, metadata, expires_at, confirmed_at, created_at
        FROM payments
        WHERE merchant_id = $1
    "#.to_string();

    let mut bind_count = 1;
    if status_filter.is_some() {
        query.push_str(&format!(" AND status = ${}", bind_count + 1));
        bind_count += 1;
    }

    query.push_str(&format!(" ORDER BY created_at DESC LIMIT ${} OFFSET ${}", bind_count + 1, bind_count + 2));

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
    let payments = sqlx::query_as::<_, Payment>(
        r#"
        SELECT id, merchant_id, amount, currency, deposit_address, status,
               tx_hash, confirmations, required_confirmations,
               idempotency_key, metadata, expires_at, confirmed_at, created_at
        FROM payments
        WHERE status = 'pending' AND expires_at > NOW()
        ORDER BY created_at ASC
        "#,
    )
    .fetch_all(pool)
    .await?;

    Ok(payments)
}

pub async fn find_detected_payments(pool: &PgPool) -> anyhow::Result<Vec<Payment>> {
    let payments = sqlx::query_as::<_, Payment>(
        r#"
        SELECT id, merchant_id, amount, currency, deposit_address, status,
               tx_hash, confirmations, required_confirmations,
               idempotency_key, metadata, expires_at, confirmed_at, created_at
        FROM payments
        WHERE status IN ('detected', 'confirming') AND tx_hash IS NOT NULL
        ORDER BY created_at ASC
        "#,
    )
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
    sqlx::query(
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

    Ok(())
}
