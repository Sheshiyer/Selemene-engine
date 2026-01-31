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
use tower::ServiceBuilder;
use tower_http::timeout::TimeoutLayer;
use crate::SelemeneEngine;
use std::sync::Arc;
use std::time::Duration;

/// Create the main API router
pub fn create_api_router(engine: Arc<SelemeneEngine>) -> Router {
    // Get timeout from environment or use default (30 seconds)
    let timeout_secs = std::env::var("REQUEST_TIMEOUT_SECS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(30);
    let timeout = Duration::from_secs(timeout_secs);

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
        .layer(
            ServiceBuilder::new()
                .layer(TimeoutLayer::new(timeout))
                .layer(cors)
        )
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
