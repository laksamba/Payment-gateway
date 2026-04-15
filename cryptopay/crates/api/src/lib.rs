pub mod routes;
pub mod middleware;

pub use cryptopay_core::errors::AppError;
pub use middleware::auth::AuthenticatedMerchant;

use std::sync::Arc;

use axum::Router;
use sha2::{Digest, Sha256};

#[derive(Clone)]
pub struct Config {
    pub database_url: String,
    pub trongrid_api_key: String,
    pub server_port: u16,
}

impl Config {
    pub fn from_env() -> Self {
        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let trongrid_api_key = std::env::var("TRONGRID_API_KEY").expect("TRONGRID_API_KEY must be set");
        let server_port = std::env::var("SERVER_PORT")
            .unwrap_or_else(|_| "8080".to_string())
            .parse()
            .expect("SERVER_PORT must be a valid u16");

        Self {
            database_url,
            trongrid_api_key,
            server_port,
        }
    }
}

#[derive(Clone)]
pub struct AppState {
    pub db: sqlx::PgPool,
    pub config: Config,
}

pub fn app() -> Router<Arc<AppState>> {
    Router::new()
        .merge(routes::auth::routes())
        .merge(routes::merchants::routes())
        .merge(routes::payments::routes())
        .merge(routes::webhooks::routes())
}

pub fn hash_api_key(api_key: &str) -> String {
    hex::encode(Sha256::digest(api_key.as_bytes()))
}
