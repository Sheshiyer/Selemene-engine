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
/// structured log aggregation systems (e.g., Loki, ELK, Datadog, CloudWatch).
///
/// Configuration:
/// - Log level from parameter (allows override via env)
/// - JSON formatter with structured fields
/// - Includes span context (trace_id, span_id), timestamps, and all structured fields
/// - Adds custom fields: user_id, engine_id, duration_ms (when present in span context)
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
                .with_span_events(fmt::format::FmtSpan::CLOSE) // Log span close events with duration
                .json()  // JSON formatter for production
                .flatten_event(true)  // Flatten span context into event
                .with_current_span(true),  // Include current span info
        )
        .init();

    tracing::info!("Tracing initialized with JSON formatter, level: {}", log_level);
}

/// Initialize tracing with JSON formatter and OpenTelemetry integration.
///
/// Use this for full observability with distributed tracing via Jaeger/OTLP.
/// Combines structured JSON logs with trace context propagation.
///
/// # Arguments
/// * `log_level` - Log level filter string
/// * `service_name` - Service name for OpenTelemetry
/// * `otlp_endpoint` - OTLP endpoint (e.g., "http://jaeger:4317")
///
/// # Example
/// ```rust,no_run
/// noesis_api::init_tracing_with_otel("info", "noesis-api", "http://localhost:4317").await?;
/// ```
#[cfg(feature = "otel")]
pub async fn init_tracing_with_otel(
    log_level: &str,
    service_name: &str,
    otlp_endpoint: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use tracing_opentelemetry::OpenTelemetryLayer;
    use opentelemetry::trace::TracerProvider;
    
    // Initialize OpenTelemetry tracer
    noesis_metrics::init_tracing(service_name, otlp_endpoint).await?;
    
    let tracer = opentelemetry::global::tracer_provider()
        .tracer(service_name.to_string());
    
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(log_level));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(OpenTelemetryLayer::new(tracer))
        .with(
            fmt::layer()
                .with_target(true)
                .with_thread_ids(false)
                .with_line_number(true)
                .with_file(false)
                .with_span_events(fmt::format::FmtSpan::CLOSE)
                .json()
                .flatten_event(true)
                .with_current_span(true),
        )
        .init();

    tracing::info!(
        "Tracing initialized with OpenTelemetry: service={}, endpoint={}",
        service_name,
        otlp_endpoint
    );
    
    Ok(())
}
