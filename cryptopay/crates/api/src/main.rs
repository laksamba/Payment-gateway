use std::sync::Arc;

use anyhow::Result;
use axum::{
    routing::get,
    Router,
};
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

use cryptopay_api::{AppState, Config};

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

    // Initialize Tron client for blockchain monitoring
    let tron_client = cryptopay_monitor::tron::TronClient::new(state.config.trongrid_api_key.clone());
    
    // Log that scanner is being initialized
    let api_key_display = if state.config.trongrid_api_key.len() > 8 {
        format!("{}...{}", &state.config.trongrid_api_key[..8], &state.config.trongrid_api_key[state.config.trongrid_api_key.len()-4..])
    } else {
        "***".to_string()
    };
    tracing::info!("Initializing scanner with TronGrid API key: {}", api_key_display);

    let app = Router::new()
        .route("/health", get(health_check))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .merge(cryptopay_api::routes::auth::routes())
        .merge(cryptopay_api::routes::merchants::routes())
        .merge(cryptopay_api::routes::payments::routes())
        .merge(cryptopay_api::routes::webhooks::routes())
        .with_state(state.clone());

    // Start background task to expire old payments
    let db_clone = state.db.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
            match cryptopay_db::payments::expire_old_payments(&db_clone).await {
                Ok(count) => {
                    if count > 0 {
                        tracing::info!("Expired {} old payments", count);
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to expire old payments: {}", e);
                }
            }
        }
    });

    // Start blockchain scanner
    let scanner_db = state.db.clone();
    let scanner_tron = tron_client.clone();
    tokio::spawn(async move {
        if let Err(e) = cryptopay_monitor::scanner::run_scanner(scanner_db, scanner_tron).await {
            tracing::error!("Scanner failed: {}", e);
        }
    });

    // Start webhook retry processor
    let webhook_db = state.db.clone();
    let webhook_delivery = cryptopay_webhook::delivery::WebhookDelivery::new();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
            if let Err(e) = cryptopay_webhook::delivery::WebhookDelivery::process_pending_webhooks(
                &webhook_db,
                &webhook_delivery,
            ).await {
                tracing::error!("Webhook processor error: {}", e);
            }
        }
    });

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
