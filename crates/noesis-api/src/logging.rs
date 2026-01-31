//! Logging and tracing configuration for Noesis API
//!
//! Initializes structured logging with tracing-subscriber.
//! Logs include span context (trace_id, span_id), timestamps, and module paths.

use tracing_subscriber::{
    fmt,
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter,
};

/// Initialize the tracing subscriber with structured logging.
///
/// Configuration:
/// - Log level from parameter (allows override via env)
/// - Pretty formatter for development (human-readable)
/// - Includes span context (trace_id, span_id) in all logs
/// - Includes target (module path) and timestamp
///
/// Call this once at application startup before creating the router.
///
/// # Arguments
/// * `log_level` - Log level filter string (e.g., "info,noesis_api=debug")
///
/// # Example
/// ```rust,no_run
/// noesis_api::init_tracing("info,noesis_api=debug");
/// ```
pub fn init_tracing(log_level: &str) {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(log_level));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(
            fmt::layer()
                .with_target(true)       // Include module path (e.g., noesis_api::middleware)
                .with_thread_ids(false)  // Exclude thread IDs for cleaner output
                .with_line_number(true)  // Include line numbers for debugging
                .with_file(false)        // Exclude file paths to reduce noise
                .pretty(),               // Pretty formatter for development
        )
        .init();

    tracing::info!("Tracing initialized with level: {}", log_level);
}

/// Initialize tracing with JSON formatter (for production).
///
/// Use this in production environments where logs are ingested by
/// structured log aggregation systems (e.g., ELK, Datadog, CloudWatch).
///
/// Configuration:
/// - Log level from parameter (allows override via env)
/// - JSON formatter with structured fields
/// - Includes span context, timestamps, and all structured fields
///
/// # Arguments
/// * `log_level` - Log level filter string (e.g., "info,noesis_api=debug")
///
/// # Example
/// ```rust,no_run
/// noesis_api::init_tracing_json("info");
/// ```
pub fn init_tracing_json(log_level: &str) {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(log_level));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(
            fmt::layer()
                .with_target(true)
                .with_thread_ids(false)
                .with_line_number(true)
                .with_file(false)
                .json(),  // JSON formatter for production
        )
        .init();

    tracing::info!("Tracing initialized with JSON formatter, level: {}", log_level);
}
