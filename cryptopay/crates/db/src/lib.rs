pub mod payments;
pub mod merchants;
pub mod events;

pub async fn connect(database_url: &str) -> anyhow::Result<sqlx::PgPool> {
    let pool = sqlx::PgPool::connect(database_url).await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    Ok(pool)
}