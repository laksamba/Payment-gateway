use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

pub async fn insert_event(
    pool: &PgPool,
    payment_id: Uuid,
    event_type: &str,
    payload: Value,
) -> anyhow::Result<()> {
    sqlx::query(
        r#"
        INSERT INTO events (payment_id, event_type, payload)
        VALUES ($1, $2, $3)
        "#,
    )
    .bind(payment_id)
    .bind(event_type)
    .bind(payload)
    .execute(pool)
    .await?;

    Ok(())
}
