//! Noesis API Server — HTTP entry point for the Tryambakam Noesis platform
//!
//! Entry point for the Noesis API server. Initializes tracing, builds the router,
//! and starts the Axum HTTP server with environment-based configuration.

use noesis_api::{build_app_state, create_router, init_tracing, init_tracing_json, ApiConfig};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    // Load configuration from environment (fail fast on missing required vars)
    let config = match ApiConfig::from_env() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Configuration error: {}", e);
            std::process::exit(1);
        }
    };

    // Validate configuration (warn on weak settings, reject invalid combos)
    if let Err(e) = config.validate() {
        eprintln!("Configuration validation failed: {}", e);
        std::process::exit(1);
    }

    // Initialize Sentry error tracking (no-op if SENTRY_DSN is unset)
    let _sentry_guard = sentry::init((
        std::env::var("SENTRY_DSN").ok(),
        sentry::ClientOptions {
            release: sentry::release_name!(),
            environment: Some(
                std::env::var("RUST_ENV")
                    .unwrap_or_else(|_| "development".into())
                    .into(),
            ),
            traces_sample_rate: 0.1,
            ..Default::default()
        },
    ));

    // Initialize structured logging based on config
    if config.log_format == "json" {
        init_tracing_json(&config.log_level);
    } else {
        init_tracing(&config.log_level);
    }

    if sentry::Hub::current()
        .client()
        .is_some_and(|c| c.is_enabled())
    {
        tracing::info!("Sentry error tracking enabled");
    } else {
        tracing::info!("Sentry error tracking disabled (no SENTRY_DSN)");
    }

    tracing::info!("Starting Noesis API server");
    tracing::info!(
        "Configuration loaded: host={}, port={}, log_format={}",
        config.host,
        config.port,
        config.log_format
    );

    if config.redis_url.is_some() {
        tracing::info!("Redis cache enabled");
    } else {
        tracing::info!("Redis cache disabled (in-memory only)");
    }

    // Build application state with orchestrator, cache, auth, metrics
    let state = build_app_state(&config).await;
    tracing::info!("Application state initialized");

    // Spawn daily background task: purge revoked API keys older than 30 days.
    if let Some(admin_repo) = state.admin_repository.clone() {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(24 * 60 * 60));
            loop {
                interval.tick().await;
                match admin_repo.purge_revoked_api_keys(30).await {
                    Ok(n) if n > 0 => tracing::info!(deleted = n, "Purged revoked API keys older than 30 days"),
                    Ok(_) => tracing::debug!("No revoked API keys to purge"),
                    Err(e) => tracing::warn!(error = %e, "Failed to purge revoked API keys"),
                }
            }
        });
        tracing::info!("API key auto-purge task scheduled (30-day retention for revoked keys)");
    }

    // Create the Axum router with all routes and middleware
    let app = create_router(state, &config);
    tracing::info!("Router configured");

    // Bind to configured address
    let addr = config.bind_address();
    tracing::info!("Binding to {}", addr);

    let listener = TcpListener::bind(&addr).await.unwrap_or_else(|e| {
        tracing::error!("Failed to bind TCP listener to {}: {}", addr, e);
        std::process::exit(1);
    });

    tracing::info!("Noesis API server listening on {}", addr);
    tracing::info!("Health check: http://{}/health", addr);
    tracing::info!("Metrics: http://{}/metrics", addr);
    tracing::info!("API v1: http://{}/api/v1/status", addr);

    // Start the server
    axum::serve(listener, app).await.expect("Server error");
}
