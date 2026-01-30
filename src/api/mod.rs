pub mod routes;
pub mod middleware;
pub mod handlers;
pub mod ghati_handlers;
pub mod ghati_panchanga_handlers;

use axum::{
    Router,
    http::Method,
};
use tower_http::cors::{CorsLayer, Any};
use crate::SelemeneEngine;
use std::sync::Arc;

/// Create the main API router
pub fn create_api_router(engine: Arc<SelemeneEngine>) -> Router {
    // CORS configuration
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_origin(Any)
        .allow_headers(Any);

    // Create router with routes and state
    Router::<Arc<SelemeneEngine>>::new()
        .nest("/api/v1", routes::create_v1_routes())
        .route("/health", axum::routing::get(handlers::health_check))
        .route("/metrics", axum::routing::get(handlers::metrics))
        .route("/status", axum::routing::get(handlers::status))
        .layer(cors)
        // TODO: Fix middleware compatibility issues
        // .layer(axum::middleware::from_fn(middleware::logging_middleware))
        // .layer(axum::middleware::from_fn(middleware::auth_middleware))
        // .layer(axum::middleware::from_fn(middleware::rate_limit_middleware))
        .with_state(engine)
}

/// API configuration
#[derive(Debug, Clone)]
pub struct ApiConfig {
    pub host: String,
    pub port: u16,
    pub workers: usize,
    pub max_request_size: usize,
    pub timeout_seconds: u64,
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 8080,
            workers: 4,
            max_request_size: 10 * 1024 * 1024, // 10MB
            timeout_seconds: 30,
        }
    }
}
