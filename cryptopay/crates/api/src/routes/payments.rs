use std::sync::Arc;

use axum::{routing::get, Router};

use crate::AppState;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/payments", get(list_payments))
}

async fn list_payments() -> &'static str {
    "payments"
}