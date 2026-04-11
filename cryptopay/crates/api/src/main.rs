use std::sync::Arc;

use anyhow::Result;
use axum::{
    routing::get,
    Router,
};
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

use cryptopay_api::{AppState, Config, AppError};

mod routes;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    dotenvy::dotenv().ok();

    let config = Config::from_env();
    tracing::info!("Connecting to database");

    let db = cryptopay_db::connect(&config.database_url).await?;

    let state = Arc::new(AppState {
        db,
        config,
    });

    let app = Router::new()
        .route("/health", get(health_check))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .merge(routes::merchants::routes())
        .merge(routes::payments::routes())
        .with_state(state.clone());

    let port = state.config.server_port;
    let addr = format!("0.0.0.0:{}", port);
    tracing::info!("Starting CryptoPay API server on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({
        "status": "ok",
        "version": "0.1.0"
    }))
}