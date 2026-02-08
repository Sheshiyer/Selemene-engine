use prometheus::{Counter, Histogram, Gauge, Registry, HistogramOpts};
use std::sync::Arc;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref REGISTRY: Registry = Registry::new();
}

/// Engine metrics for monitoring and observability
pub struct EngineMetrics {
    // Request metrics
    pub requests_total: Counter,
    pub request_duration: Histogram,
    pub active_connections: Gauge,
    
    // Calculation metrics
    pub calculations_total: Counter,
    pub calculation_duration: Histogram,
    pub calculation_errors: Counter,
    
    // Backend usage metrics
    pub swiss_ephemeris_usage: Counter,
    pub native_engine_usage: Counter,
    pub cache_hits: Counter,
    pub cache_misses: Counter,
    
    // Accuracy metrics
    pub validation_differences: Histogram,
    pub precision_achieved: Histogram,
    
    // System metrics
    pub memory_usage_bytes: Gauge,
    pub cpu_usage_percent: Gauge,
    pub uptime_seconds: Gauge,
}

impl EngineMetrics {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let requests_total = Counter::new(
            "selemene_requests_total",
            "Total number of requests"
        )?;
        
        let request_duration = Histogram::with_opts(
            HistogramOpts::new("selemene_request_duration_seconds", "Request duration in seconds")
        )?;
        
        let active_connections = Gauge::new(
            "selemene_active_connections",
            "Number of active connections"
        )?;
        
        let calculations_total = Counter::new(
            "selemene_calculations_total",
            "Total number of calculations"
        )?;
        
        let calculation_duration = Histogram::with_opts(
            HistogramOpts::new("selemene_calculation_duration_seconds", "Calculation duration in seconds")
        )?;
        
        let calculation_errors = Counter::new(
            "selemene_calculation_errors_total",
            "Total number of calculation errors"
        )?;
        
        let swiss_ephemeris_usage = Counter::new(
            "selemene_swiss_ephemeris_usage_total",
            "Total usage of Swiss Ephemeris backend"
        )?;
        
        let native_engine_usage = Counter::new(
            "selemene_native_engine_usage_total",
            "Total usage of native engines"
        )?;
        
        let cache_hits = Counter::new(
            "selemene_cache_hits_total",
            "Total cache hits"
        )?;
        
        let cache_misses = Counter::new(
            "selemene_cache_misses_total",
            "Total cache misses"
        )?;
        
        let validation_differences = Histogram::with_opts(
            HistogramOpts::new("selemene_validation_differences_arcseconds", "Differences between backend calculations in arcseconds")
        )?;
        
        let precision_achieved = Histogram::with_opts(
            HistogramOpts::new("selemene_precision_achieved_arcseconds", "Precision achieved in calculations in arcseconds")
        )?;
        
        let memory_usage_bytes = Gauge::new(
            "selemene_memory_usage_bytes",
            "Memory usage in bytes"
        )?;
        
        let cpu_usage_percent = Gauge::new(
            "selemene_cpu_usage_percent",
            "CPU usage percentage"
        )?;
        
        let uptime_seconds = Gauge::new(
            "selemene_uptime_seconds",
            "Uptime in seconds"
        )?;
        
        // Register metrics with Prometheus registry
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
        })
    }

    /// Record a request
    pub fn record_request(&self, duration: f64) {
        self.requests_total.inc();
        self.request_duration.observe(duration);
    }

    /// Record a calculation
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

    /// Record calculation error
    pub fn record_calculation_error(&self) {
        self.calculation_errors.inc();
    }

    /// Record cache hit
    pub fn record_cache_hit(&self) {
        self.cache_hits.inc();
    }

    /// Record cache miss
    pub fn record_cache_miss(&self) {
        self.cache_misses.inc();
    }

    /// Record validation difference
    pub fn record_validation_difference(&self, difference_arcseconds: f64) {
        self.validation_differences.observe(difference_arcseconds);
    }

    /// Update system metrics
    pub fn update_system_metrics(&self, memory_bytes: f64, cpu_percent: f64, uptime_seconds: f64) {
        self.memory_usage_bytes.set(memory_bytes);
        self.cpu_usage_percent.set(cpu_percent);
        self.uptime_seconds.set(uptime_seconds);
    }

    /// Update active connections
    pub fn update_active_connections(&self, count: f64) {
        self.active_connections.set(count);
    }

    /// Get metrics as Prometheus text format
    pub fn get_metrics_text(&self) -> Result<String, Box<dyn std::error::Error>> {
        use prometheus::Encoder;
        let mut buffer = Vec::new();
        let encoder = prometheus::TextEncoder::new();
        encoder.encode(&REGISTRY.gather(), &mut buffer)?;
        
        Ok(String::from_utf8(buffer)?)
    }
}

/// Metrics collector for gathering system metrics
pub struct MetricsCollector {
    metrics: Arc<EngineMetrics>,
    start_time: std::time::Instant,
}

impl MetricsCollector {
    pub fn new(metrics: Arc<EngineMetrics>) -> Self {
        Self {
            metrics,
            start_time: std::time::Instant::now(),
        }
    }

    /// Collect and update system metrics
    pub async fn collect_system_metrics(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Get memory usage (simplified for now)
        let memory_usage = 0.0; // TODO: Implement actual memory monitoring
        
        // Get CPU usage (simplified)
        let cpu_usage = 0.0; // TODO: Implement actual CPU monitoring
        
        // Calculate uptime
        let uptime = self.start_time.elapsed().as_secs_f64();
        
        // Update metrics
        self.metrics.update_system_metrics(memory_usage, cpu_usage, uptime);
        
        Ok(())
    }

    /// Start metrics collection loop
    pub async fn start_collection_loop(&self) {
        let metrics = self.metrics.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(30));
            
            loop {
                interval.tick().await;
                
                // Collect system metrics
                if let Err(e) = Self::collect_system_metrics_static(metrics.clone()).await {
                    tracing::warn!("Failed to collect system metrics: {}", e);
                }
            }
        });
    }

    async fn collect_system_metrics_static(metrics: Arc<EngineMetrics>) -> Result<(), Box<dyn std::error::Error>> {
        // Get memory usage (simplified for now)
        let memory_usage = 0.0; // TODO: Implement actual memory monitoring
        
        // Get CPU usage (simplified)
        let cpu_usage = 0.0; // TODO: Implement actual CPU monitoring
        
        // Calculate uptime (simplified)
        let uptime = 0.0; // TODO: Implement actual uptime tracking
        
        // Update metrics
        metrics.update_system_metrics(memory_usage, cpu_usage, uptime);
        
        Ok(())
    }
}
