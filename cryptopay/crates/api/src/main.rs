use std::sync::Arc;

use anyhow::Result;
use axum::{
    routing::get,
    Router,
};
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use sha2::{Digest, Sha256};

use cryptopay_api::{AppState, Config, AppError};

mod routes;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    dotenvy::dotenv().ok();

    let config = Config::from_env();
    tracing::info!("Connecting to database");

    let db = cryptopay_db::connect(&config.database_url).await?;

    // Create or find test merchant for development
    let test_api_key = "test_api_key_123";
    let test_webhook_secret = "test_webhook_secret_456";
    let test_api_key_hash = hex::encode(sha2::Sha256::digest(test_api_key.as_bytes()));
    
    // Try to find existing test merchant
    let merchant = match cryptopay_db::merchants::find_merchant_by_email(&db, "test@example.com").await {
        Ok(Some(merchant)) => {
            tracing::info!("Found existing test merchant with ID: {}", merchant.id);
            merchant
        },
        _ => {
            // Create new test merchant
            match cryptopay_db::merchants::create_merchant(
                &db,
                "Test Merchant",
                "test@example.com",
                &test_api_key_hash,
                test_webhook_secret,
            ).await {
                Ok(merchant) => {
                    tracing::info!("Created test merchant with ID: {}", merchant.id);
                    merchant
                },
                Err(e) => {
                    tracing::error!("Failed to create test merchant: {}", e);
                    return Err(e.into());
                }
            }
        }
    };

    let state = Arc::new(AppState {
        db,
        config,
    });

    // Initialize Tron client for blockchain monitoring
    let tron_client = cryptopay_monitor::tron::TronClient::new(state.config.trongrid_api_key.clone());

    let app = Router::new()
        .route("/health", get(health_check))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .merge(routes::merchants::routes())
        .merge(routes::payments::routes())
        .merge(routes::webhooks::routes())
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
    let webhook_secret = test_webhook_secret.to_string();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
            if let Err(e) = cryptopay_webhook::delivery::WebhookDelivery::process_pending_webhooks(
                &webhook_db,
                &webhook_delivery,
                &webhook_secret,
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