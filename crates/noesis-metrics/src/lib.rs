//! Noesis Metrics -- Prometheus metrics collection
//!
//! Migrated from the original Selemene Engine metrics system.
//! All metric names use the `noesis_` prefix. Includes per-engine
//! calculation counters and duration histograms keyed by `engine_id`.

use prometheus::{
    Counter, Gauge, Histogram, HistogramOpts, HistogramVec, IntCounterVec, IntGauge, IntGaugeVec,
    Opts, Registry,
};
use std::sync::Arc;

use lazy_static::lazy_static;

lazy_static! {
    pub static ref REGISTRY: Registry = Registry::new();
}

// ---------------------------------------------------------------------------
// OpenTelemetry Tracing Configuration
// ---------------------------------------------------------------------------

/// Initialize OpenTelemetry tracing with OTLP exporter (Jaeger compatible).
///
/// Call this once at application startup to enable distributed tracing.
/// Traces will be exported to the configured OTLP endpoint (default: Jaeger).
///
/// # Arguments
/// * `service_name` - Name of the service for trace identification
/// * `otlp_endpoint` - OTLP gRPC endpoint (e.g., "http://jaeger:4317")
///
/// # Example
/// ```rust,no_run
/// noesis_metrics::init_otel_tracing("noesis-api", "http://localhost:4317").await?;
/// ```
pub async fn init_otel_tracing(
    service_name: &str,
    otlp_endpoint: &str,
) -> Result<opentelemetry_sdk::trace::Tracer, Box<dyn std::error::Error + Send + Sync>> {
    use opentelemetry_otlp::WithExportConfig;
    use opentelemetry_sdk::{runtime, trace as sdktrace, Resource};
    use opentelemetry::KeyValue;
    
    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint(otlp_endpoint),
        )
        .with_trace_config(
            sdktrace::Config::default()
                .with_resource(Resource::new(vec![
                    KeyValue::new("service.name", service_name.to_string()),
                    KeyValue::new("service.version", env!("CARGO_PKG_VERSION")),
                ]))
                .with_sampler(sdktrace::Sampler::AlwaysOn),
        )
        .install_batch(runtime::Tokio)?;
    
    tracing::info!(
        "OpenTelemetry tracing initialized: service={}, endpoint={}",
        service_name,
        otlp_endpoint
    );
    
    Ok(tracer)
}

/// Shutdown OpenTelemetry tracing gracefully.
/// Call this before application exit to ensure all spans are exported.
pub fn shutdown_tracing() {
    opentelemetry::global::shutdown_tracer_provider();
    tracing::info!("OpenTelemetry tracing shut down");
}

// ---------------------------------------------------------------------------
// NoesisMetrics
// ---------------------------------------------------------------------------

/// Top-level metrics struct for the Noesis platform.
///
/// Carries both the original (renamed) Selemene metrics and the new
/// per-engine counters/histograms.
pub struct NoesisMetrics {
    // -- Request metrics -----------------------------------------------------
    pub requests_total: Counter,
    pub request_duration: Histogram,
    pub active_connections: Gauge,

    // -- Calculation metrics (aggregate) -------------------------------------
    pub calculations_total: Counter,
    pub calculation_duration: Histogram,
    pub calculation_errors: Counter,

    // -- Backend usage metrics -----------------------------------------------
    pub swiss_ephemeris_usage: Counter,
    pub native_engine_usage: Counter,
    pub cache_hits: Counter,
    pub cache_misses: Counter,

    // -- Accuracy metrics ----------------------------------------------------
    pub validation_differences: Histogram,
    pub precision_achieved: Histogram,

    // -- System metrics ------------------------------------------------------
    pub memory_usage_bytes: Gauge,
    pub cpu_usage_percent: Gauge,
    pub uptime_seconds: Gauge,

    // -- Per-engine metrics (NEW) --------------------------------------------
    /// Total calculations broken down by `engine_id` label.
    pub engine_calculations_total: IntCounterVec,
    /// Calculation duration broken down by `engine_id` label.
    pub engine_calculation_duration: HistogramVec,
    /// Total calculations broken down by `engine_id` and `status` labels.
    pub engine_calculation_status_total: IntCounterVec,
    /// Total calculation errors broken down by `engine_id` and `error_type` labels.
    pub engine_calculation_errors_total: IntCounterVec,

    // -- Per-layer cache metrics -----------------------------------------------
    /// Cumulative cache hits broken down by `layer` label (l1, l2, l3).
    pub cache_layer_hits: IntGaugeVec,
    /// Cumulative cache misses across all layers.
    pub cache_layer_misses: IntGauge,
    /// Total cache lookup requests.
    pub cache_layer_requests: IntGauge,
    /// Computed cache hit rate (0.0–1.0).
    pub cache_hit_rate: Gauge,
}

impl NoesisMetrics {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // -- Request metrics -------------------------------------------------
        let requests_total = Counter::new(
            "noesis_requests_total",
            "Total number of requests",
        )?;

        let request_duration = Histogram::with_opts(HistogramOpts::new(
            "noesis_request_duration_seconds",
            "Request duration in seconds",
        ))?;

        let active_connections = Gauge::new(
            "noesis_active_connections",
            "Number of active connections",
        )?;

        // -- Calculation metrics (aggregate) ---------------------------------
        let calculations_total = Counter::new(
            "noesis_calculations_total",
            "Total number of calculations",
        )?;

        let calculation_duration = Histogram::with_opts(HistogramOpts::new(
            "noesis_calculation_duration_seconds",
            "Calculation duration in seconds",
        ))?;

        let calculation_errors = Counter::new(
            "noesis_calculation_errors_total",
            "Total number of calculation errors",
        )?;

        // -- Backend usage metrics -------------------------------------------
        let swiss_ephemeris_usage = Counter::new(
            "noesis_swiss_ephemeris_usage_total",
            "Total usage of Swiss Ephemeris backend",
        )?;

        let native_engine_usage = Counter::new(
            "noesis_native_engine_usage_total",
            "Total usage of native engines",
        )?;

        let cache_hits = Counter::new(
            "noesis_cache_hits_total",
            "Total cache hits",
        )?;

        let cache_misses = Counter::new(
            "noesis_cache_misses_total",
            "Total cache misses",
        )?;

        // -- Accuracy metrics ------------------------------------------------
        let validation_differences = Histogram::with_opts(HistogramOpts::new(
            "noesis_validation_differences_arcseconds",
            "Differences between backend calculations in arcseconds",
        ))?;

        let precision_achieved = Histogram::with_opts(HistogramOpts::new(
            "noesis_precision_achieved_arcseconds",
            "Precision achieved in calculations in arcseconds",
        ))?;

        // -- System metrics --------------------------------------------------
        let memory_usage_bytes = Gauge::new(
            "noesis_memory_usage_bytes",
            "Memory usage in bytes",
        )?;

        let cpu_usage_percent = Gauge::new(
            "noesis_cpu_usage_percent",
            "CPU usage percentage",
        )?;

        let uptime_seconds = Gauge::new(
            "noesis_uptime_seconds",
            "Uptime in seconds",
        )?;

        // -- Per-engine metrics (NEW) ----------------------------------------
        let engine_calculations_total = IntCounterVec::new(
            Opts::new(
                "noesis_engine_calculations_total",
                "Total calculations per engine",
            ),
            &["engine_id"],
        )?;

        let engine_calculation_duration = HistogramVec::new(
            HistogramOpts::new(
                "noesis_engine_calculation_duration_seconds",
                "Calculation duration per engine in seconds",
            ),
            &["engine_id"],
        )?;

        let engine_calculation_status_total = IntCounterVec::new(
            Opts::new(
                "noesis_engine_calculation_status_total",
                "Total calculations per engine by status",
            ),
            &["engine_id", "status"],
        )?;

        let engine_calculation_errors_total = IntCounterVec::new(
            Opts::new(
                "noesis_engine_calculation_errors_total",
                "Total calculation errors per engine by error type",
            ),
            &["engine_id", "error_type"],
        )?;

        // -- Per-layer cache metrics -----------------------------------------
        let cache_layer_hits = IntGaugeVec::new(
            Opts::new(
                "noesis_cache_layer_hits_total",
                "Cumulative cache hits per layer",
            ),
            &["layer"],
        )?;

        let cache_layer_misses = IntGauge::new(
            "noesis_cache_layer_misses_total",
            "Cumulative cache misses across all layers",
        )?;

        let cache_layer_requests = IntGauge::new(
            "noesis_cache_layer_requests_total",
            "Total cache lookup requests",
        )?;

        let cache_hit_rate = Gauge::new(
            "noesis_cache_hit_rate",
            "Cache hit rate (0.0-1.0)",
        )?;

        // -- Register everything with the Prometheus registry ----------------
        REGISTRY.register(Box::new(requests_total.clone()))?;
        REGISTRY.register(Box::new(request_duration.clone()))?;
        REGISTRY.register(Box::new(active_connections.clone()))?;
        REGISTRY.register(Box::new(calculations_total.clone()))?;
        REGISTRY.register(Box::new(calculation_duration.clone()))?;
        REGISTRY.register(Box::new(calculation_errors.clone()))?;
        REGISTRY.register(Box::new(swiss_ephemeris_usage.clone()))?;
        REGISTRY.register(Box::new(native_engine_usage.clone()))?;
        REGISTRY.register(Box::new(cache_hits.clone()))?;
        REGISTRY.register(Box::new(cache_misses.clone()))?;
        REGISTRY.register(Box::new(validation_differences.clone()))?;
        REGISTRY.register(Box::new(precision_achieved.clone()))?;
        REGISTRY.register(Box::new(memory_usage_bytes.clone()))?;
        REGISTRY.register(Box::new(cpu_usage_percent.clone()))?;
        REGISTRY.register(Box::new(uptime_seconds.clone()))?;
        REGISTRY.register(Box::new(engine_calculations_total.clone()))?;
        REGISTRY.register(Box::new(engine_calculation_duration.clone()))?;
        REGISTRY.register(Box::new(engine_calculation_status_total.clone()))?;
        REGISTRY.register(Box::new(engine_calculation_errors_total.clone()))?;
        REGISTRY.register(Box::new(cache_layer_hits.clone()))?;
        REGISTRY.register(Box::new(cache_layer_misses.clone()))?;
        REGISTRY.register(Box::new(cache_layer_requests.clone()))?;
        REGISTRY.register(Box::new(cache_hit_rate.clone()))?;

        Ok(Self {
            requests_total,
            request_duration,
            active_connections,
            calculations_total,
            calculation_duration,
            calculation_errors,
            swiss_ephemeris_usage,
            native_engine_usage,
            cache_hits,
            cache_misses,
            validation_differences,
            precision_achieved,
            memory_usage_bytes,
            cpu_usage_percent,
            uptime_seconds,
            engine_calculations_total,
            engine_calculation_duration,
            engine_calculation_status_total,
            engine_calculation_errors_total,
            cache_layer_hits,
            cache_layer_misses,
            cache_layer_requests,
            cache_hit_rate,
        })
    }

    // -- Convenience recorders -----------------------------------------------

    /// Record an inbound request with its duration.
    pub fn record_request(&self, duration: f64) {
        self.requests_total.inc();
        self.request_duration.observe(duration);
    }

    /// Record an aggregate calculation (backend + duration + precision).
    pub fn record_calculation(&self, backend: &str, duration: f64, precision: f64) {
        self.calculations_total.inc();
        self.calculation_duration.observe(duration);
        self.precision_achieved.observe(precision);

        match backend {
            "swiss" => self.swiss_ephemeris_usage.inc(),
            "native" => self.native_engine_usage.inc(),
            _ => {}
        }
    }

    /// Record a per-engine calculation (counter + duration histogram).
    pub fn record_engine_calculation(&self, engine_id: &str, duration: f64) {
        self.engine_calculations_total
            .with_label_values(&[engine_id])
            .inc();
        self.engine_calculation_duration
            .with_label_values(&[engine_id])
            .observe(duration);
    }

    /// Record a per-engine calculation with status.
    pub fn record_engine_calculation_with_status(&self, engine_id: &str, status: &str, duration: f64) {
        self.engine_calculations_total
            .with_label_values(&[engine_id])
            .inc();
        self.engine_calculation_duration
            .with_label_values(&[engine_id])
            .observe(duration);
        self.engine_calculation_status_total
            .with_label_values(&[engine_id, status])
            .inc();
    }

    /// Record a per-engine calculation error.
    pub fn record_engine_calculation_error(&self, engine_id: &str, error_type: &str) {
        self.engine_calculation_errors_total
            .with_label_values(&[engine_id, error_type])
            .inc();
    }

    /// Record a calculation error.
    pub fn record_calculation_error(&self) {
        self.calculation_errors.inc();
    }

    /// Record a cache hit.
    pub fn record_cache_hit(&self) {
        self.cache_hits.inc();
    }

    /// Record a cache miss.
    pub fn record_cache_miss(&self) {
        self.cache_misses.inc();
    }

    /// Record a validation difference in arcseconds.
    pub fn record_validation_difference(&self, difference_arcseconds: f64) {
        self.validation_differences.observe(difference_arcseconds);
    }

    /// Update system-level gauges.
    pub fn update_system_metrics(
        &self,
        memory_bytes: f64,
        cpu_percent: f64,
        uptime_secs: f64,
    ) {
        self.memory_usage_bytes.set(memory_bytes);
        self.cpu_usage_percent.set(cpu_percent);
        self.uptime_seconds.set(uptime_secs);
    }

    /// Update the active connections gauge.
    pub fn update_active_connections(&self, count: f64) {
        self.active_connections.set(count);
    }

    /// Update per-layer cache statistics from a `CacheStats` snapshot.
    ///
    /// Call this before encoding metrics (e.g. in the `/metrics` handler) to
    /// ensure Prometheus scrapes see fresh values.
    pub fn update_cache_layer_stats(
        &self,
        l1_hits: u64,
        l2_hits: u64,
        l3_hits: u64,
        misses: u64,
        total_requests: u64,
    ) {
        self.cache_layer_hits.with_label_values(&["l1"]).set(l1_hits as i64);
        self.cache_layer_hits.with_label_values(&["l2"]).set(l2_hits as i64);
        self.cache_layer_hits.with_label_values(&["l3"]).set(l3_hits as i64);
        self.cache_layer_misses.set(misses as i64);
        self.cache_layer_requests.set(total_requests as i64);
        if total_requests > 0 {
            let rate = (l1_hits + l2_hits + l3_hits) as f64 / total_requests as f64;
            self.cache_hit_rate.set(rate);
        }
    }

    /// Encode all registered metrics in Prometheus text exposition format.
    pub fn get_metrics_text(&self) -> Result<String, Box<dyn std::error::Error>> {
        use prometheus::Encoder;
        let mut buffer = Vec::new();
        let encoder = prometheus::TextEncoder::new();
        encoder.encode(&REGISTRY.gather(), &mut buffer)?;
        Ok(String::from_utf8(buffer)?)
    }
}

// ---------------------------------------------------------------------------
// MetricsCollector -- background system-metrics gatherer
// ---------------------------------------------------------------------------

/// Periodically gathers system metrics (memory, CPU, uptime) and pushes
/// them into [`NoesisMetrics`].
pub struct MetricsCollector {
    metrics: Arc<NoesisMetrics>,
    start_time: std::time::Instant,
}

impl MetricsCollector {
    pub fn new(metrics: Arc<NoesisMetrics>) -> Self {
        Self {
            metrics,
            start_time: std::time::Instant::now(),
        }
    }

    /// Collect and update system metrics once.
    pub async fn collect_system_metrics(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut sys = sysinfo::System::new();
        sys.refresh_memory();
        sys.refresh_cpu_usage();
        // Brief sleep to let CPU sampling stabilize
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        sys.refresh_cpu_usage();

        let memory_usage = sys.used_memory() as f64;
        let cpu_usage = sys.global_cpu_usage() as f64;
        let uptime = self.start_time.elapsed().as_secs_f64();

        self.metrics
            .update_system_metrics(memory_usage, cpu_usage, uptime);

        Ok(())
    }

    /// Spawn a background task that collects system metrics every 30 seconds.
    pub async fn start_collection_loop(&self) {
        let metrics = self.metrics.clone();
        let start_time = self.start_time;

        tokio::spawn(async move {
            let mut interval =
                tokio::time::interval(std::time::Duration::from_secs(30));
            let mut sys = sysinfo::System::new();

            loop {
                interval.tick().await;

                if let Err(e) =
                    Self::collect_system_metrics_loop(&mut sys, metrics.clone(), start_time).await
                {
                    tracing::warn!("Failed to collect system metrics: {}", e);
                }
            }
        });
    }

    async fn collect_system_metrics_loop(
        sys: &mut sysinfo::System,
        metrics: Arc<NoesisMetrics>,
        start_time: std::time::Instant,
    ) -> Result<(), Box<dyn std::error::Error>> {
        sys.refresh_memory();
        sys.refresh_cpu_usage();

        let memory_usage = sys.used_memory() as f64;
        let cpu_usage = sys.global_cpu_usage() as f64;
        let uptime = start_time.elapsed().as_secs_f64();

        metrics.update_system_metrics(memory_usage, cpu_usage, uptime);

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn noesis_metrics_creates_successfully() {
        let metrics = NoesisMetrics::new().expect("metrics should initialise");
        metrics.record_request(0.042);
        metrics.record_calculation("swiss", 0.015, 0.001);
        metrics.record_engine_calculation("solar_v1", 0.012);
        metrics.record_engine_calculation("lunar_v1", 0.018);
        metrics.record_cache_hit();
        metrics.record_cache_miss();
        metrics.record_calculation_error();
        metrics.record_validation_difference(0.5);
        metrics.update_system_metrics(1024.0, 12.5, 3600.0);
        metrics.update_active_connections(5.0);

        let text = metrics
            .get_metrics_text()
            .expect("should encode metrics text");
        assert!(text.contains("noesis_requests_total"));
        assert!(text.contains("noesis_engine_calculations_total"));
    }
}
