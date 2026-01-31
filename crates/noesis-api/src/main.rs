//! Noesis API Server — HTTP entry point for the Tryambakam Noesis platform
//!
//! Entry point for the Noesis API server. Initializes tracing, builds the router,
//! and starts the Axum HTTP server with environment-based configuration.

use noesis_api::{build_app_state, create_router, init_tracing, init_tracing_json, ApiConfig};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    // Load configuration from environment
    let config = ApiConfig::from_env();
    
    // Validate configuration
    if let Err(e) = config.validate() {
        eprintln!("Configuration validation failed: {}", e);
        std::process::exit(1);
    }
    
    // Initialize structured logging based on config
    if config.log_format == "json" {
        init_tracing_json(&config.log_level);
    } else {
        init_tracing(&config.log_level);
    }

    tracing::info!("Starting Noesis API server");
    tracing::info!("Configuration loaded: host={}, port={}, log_format={}", 
        config.host, config.port, config.log_format);
    
    if config.redis_url.is_some() {
        tracing::info!("Redis cache enabled");
    } else {
        tracing::info!("Redis cache disabled (in-memory only)");
    }

    // Build application state with orchestrator, cache, auth, metrics
    let state = build_app_state(&config);
    tracing::info!("Application state initialized");

    // Create the Axum router with all routes and middleware
    let app = create_router(state, &config);
    tracing::info!("Router configured");

    // Bind to configured address
    let addr = config.bind_address();
    tracing::info!("Binding to {}", addr);
    
    let listener = TcpListener::bind(&addr)
        .await
        .unwrap_or_else(|e| {
            tracing::error!("Failed to bind TCP listener to {}: {}", addr, e);
            std::process::exit(1);
        });

    tracing::info!("Noesis API server listening on {}", addr);
    tracing::info!("Health check: http://{}/health", addr);
    tracing::info!("Metrics: http://{}/metrics", addr);
    tracing::info!("API v1: http://{}/api/v1/status", addr);

    // Start the server
    axum::serve(listener, app)
        .await
        .expect("Server error");
}
