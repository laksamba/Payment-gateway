use sqlx::PgPool;

use cryptopay_core::Merchant;

pub async fn create_merchant(
    pool: &PgPool,
    name: &str,
    email: &str,
    api_key_hash: &str,
    webhook_secret: &str,
    webhook_url: Option<&str>,
) -> anyhow::Result<Merchant> {
    let merchant = sqlx::query_as::<_, Merchant>(
        r#"
        INSERT INTO merchants (name, email, api_key_hash, webhook_secret, webhook_url)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id, name, email, api_key_hash, webhook_url, webhook_secret, fee_percent, is_active, created_at
        "#,
    )
    .bind(name)
    .bind(email)
    .bind(api_key_hash)
    .bind(webhook_secret)
    .bind(webhook_url)
    .fetch_one(pool)
    .await?;

    Ok(merchant)
}

pub async fn update_merchant_webhook_url(
    pool: &PgPool,
    id: uuid::Uuid,
    webhook_url: Option<&str>,
) -> anyhow::Result<Merchant> {
    let merchant = sqlx::query_as::<_, Merchant>(
        r#"
        UPDATE merchants
        SET webhook_url = $1
        WHERE id = $2
        RETURNING id, name, email, api_key_hash, webhook_url, webhook_secret, fee_percent, is_active, created_at
        "#,
    )
    .bind(webhook_url)
    .bind(id)
    .fetch_one(pool)
    .await?;

    Ok(merchant)
}

pub async fn update_merchant_api_key_hash(
    pool: &PgPool,
    id: uuid::Uuid,
    api_key_hash: &str,
) -> anyhow::Result<Merchant> {
    let merchant = sqlx::query_as::<_, Merchant>(
        r#"
        UPDATE merchants
        SET api_key_hash = $1
        WHERE id = $2
        RETURNING id, name, email, api_key_hash, webhook_url, webhook_secret, fee_percent, is_active, created_at
        "#,
    )
    .bind(api_key_hash)
    .bind(id)
    .fetch_one(pool)
    .await?;

    Ok(merchant)
}

pub async fn update_merchant_webhook_secret(
    pool: &PgPool,
    id: uuid::Uuid,
    webhook_secret: &str,
) -> anyhow::Result<Merchant> {
    let merchant = sqlx::query_as::<_, Merchant>(
        r#"
        UPDATE merchants
        SET webhook_secret = $1
        WHERE id = $2
        RETURNING id, name, email, api_key_hash, webhook_url, webhook_secret, fee_percent, is_active, created_at
        "#,
    )
    .bind(webhook_secret)
    .bind(id)
    .fetch_one(pool)
    .await?;

    Ok(merchant)
}

pub async fn find_merchant_by_api_key_hash(
    pool: &PgPool,
    hash: &str,
) -> anyhow::Result<Option<Merchant>> {
    let merchant = sqlx::query_as::<_, Merchant>(
        r#"
        SELECT id, name, email, api_key_hash, webhook_url, webhook_secret, fee_percent, is_active, created_at
        FROM merchants
        WHERE api_key_hash = $1
        "#,
    )
    .bind(hash)
    .fetch_optional(pool)
    .await?;

    Ok(merchant)
}

pub async fn find_merchant_by_email(
    pool: &PgPool,
    email: &str,
) -> anyhow::Result<Option<Merchant>> {
    let merchant = sqlx::query_as::<_, Merchant>(
        r#"
        SELECT id, name, email, api_key_hash, webhook_url, webhook_secret, fee_percent, is_active, created_at
        FROM merchants
        WHERE email = $1
        "#,
    )
    .bind(email)
    .fetch_optional(pool)
    .await?;

    Ok(merchant)
}

pub async fn find_merchant_by_id(
    pool: &PgPool,
    id: uuid::Uuid,
) -> anyhow::Result<Option<Merchant>> {
    let merchant = sqlx::query_as::<_, Merchant>(
        r#"
        SELECT id, name, email, api_key_hash, webhook_url, webhook_secret, fee_percent, is_active, created_at
        FROM merchants
        WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;

    Ok(merchant)
}
