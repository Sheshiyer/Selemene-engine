//! Prometheus-compatible metrics for Vedic API observability (FAPI-099)
//!
//! Provides comprehensive instrumentation for:
//! - API call counts by endpoint
//! - Cache hit/miss ratios
//! - Fallback trigger counts
//! - Response time histograms
//! - Error counts by type
//!
//! # Architecture
//!
//! `NoesisMetrics` is a thread-safe, lock-free metrics collector using atomics.
//! It exposes data in Prometheus text exposition format via `export_prometheus()`.
//!
//! # Usage
//!
//! ```rust,no_run
//! use noesis_vedic_api::metrics::NoesisMetrics;
//!
//! async fn example() {
//!     let metrics = NoesisMetrics::new();
//!     metrics.record_api_call("panchang");
//!     metrics.record_cache_hit("panchang");
//!     metrics.record_response_time("panchang", std::time::Duration::from_millis(120));
//!
//!     let output = metrics.export_prometheus().await;
//!     assert!(output.contains("noesis_api_calls_total"));
//! }
//! ```

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

/// Histogram bucket boundaries in milliseconds for response time tracking.
/// Covers the range from fast cache hits (1ms) to slow API calls (30s+).
const HISTOGRAM_BUCKETS_MS: &[u64] = &[1, 5, 10, 25, 50, 100, 250, 500, 1000, 2500, 5000, 10000, 30000];

/// All known endpoint names for consistent labeling.
pub const ENDPOINTS: &[&str] = &[
    "panchang",
    "complete_panchang",
    "vimshottari_dasha",
    "birth_chart",
    "navamsa_chart",
    "muhurtas",
    "hora_timings",
    "choghadiya",
];

/// All known error types matching VedicApiError variants.
pub const ERROR_TYPES: &[&str] = &[
    "configuration",
    "network",
    "api",
    "rate_limit",
    "invalid_input",
    "parse",
    "circuit_breaker_open",
    "cache",
    "fallback_failed",
];

/// Thread-safe atomic counter.
#[derive(Debug)]
struct Counter {
    value: AtomicU64,
}

impl Counter {
    fn new() -> Self {
        Self {
            value: AtomicU64::new(0),
        }
    }

    fn increment(&self) {
        self.value.fetch_add(1, Ordering::Relaxed);
    }

    fn get(&self) -> u64 {
        self.value.load(Ordering::Relaxed)
    }

    fn reset(&self) {
        self.value.store(0, Ordering::Relaxed);
    }
}

impl Clone for Counter {
    fn clone(&self) -> Self {
        Self {
            value: AtomicU64::new(self.get()),
        }
    }
}

/// Histogram using fixed bucket boundaries for response time distribution.
#[derive(Debug)]
struct Histogram {
    /// Count of observations in each bucket (cumulative).
    buckets: Vec<AtomicU64>,
    /// Sum of all observed values in microseconds.
    sum_us: AtomicU64,
    /// Total count of observations.
    count: AtomicU64,
}

impl Histogram {
    fn new() -> Self {
        let buckets = HISTOGRAM_BUCKETS_MS.iter().map(|_| AtomicU64::new(0)).collect();
        Self {
            buckets,
            sum_us: AtomicU64::new(0),
            count: AtomicU64::new(0),
        }
    }

    fn observe(&self, duration: Duration) {
        let ms = duration.as_millis() as u64;
        let us = duration.as_micros() as u64;

        self.sum_us.fetch_add(us, Ordering::Relaxed);
        self.count.fetch_add(1, Ordering::Relaxed);

        for (i, boundary) in HISTOGRAM_BUCKETS_MS.iter().enumerate() {
            if ms <= *boundary {
                self.buckets[i].fetch_add(1, Ordering::Relaxed);
            }
        }
    }

    fn count(&self) -> u64 {
        self.count.load(Ordering::Relaxed)
    }

    fn sum_seconds(&self) -> f64 {
        self.sum_us.load(Ordering::Relaxed) as f64 / 1_000_000.0
    }

    fn bucket_counts(&self) -> Vec<(u64, u64)> {
        HISTOGRAM_BUCKETS_MS
            .iter()
            .zip(self.buckets.iter())
            .map(|(boundary, count)| (*boundary, count.load(Ordering::Relaxed)))
            .collect()
    }
}

impl Clone for Histogram {
    fn clone(&self) -> Self {
        let buckets = self.buckets.iter()
            .map(|b| AtomicU64::new(b.load(Ordering::Relaxed)))
            .collect();
        Self {
            buckets,
            sum_us: AtomicU64::new(self.sum_us.load(Ordering::Relaxed)),
            count: AtomicU64::new(self.count.load(Ordering::Relaxed)),
        }
    }
}

/// Comprehensive metrics collector for Noesis Vedic API.
///
/// All operations are lock-free using atomic counters. The only lock is
/// on the endpoint/error maps for dynamic label insertion, guarded by RwLock
/// so reads (the hot path during export) never block writes.
#[derive(Debug, Clone)]
pub struct NoesisMetrics {
    /// API call counts keyed by endpoint name.
    api_calls: Arc<RwLock<HashMap<String, Counter>>>,
    /// Cache hit counts keyed by endpoint name.
    cache_hits: Arc<RwLock<HashMap<String, Counter>>>,
    /// Cache miss counts keyed by endpoint name.
    cache_misses: Arc<RwLock<HashMap<String, Counter>>>,
    /// Fallback trigger counts keyed by endpoint name.
    fallback_triggers: Arc<RwLock<HashMap<String, Counter>>>,
    /// Response time histograms keyed by endpoint name.
    response_times: Arc<RwLock<HashMap<String, Histogram>>>,
    /// Error counts keyed by error type.
    error_counts: Arc<RwLock<HashMap<String, Counter>>>,
    /// Total successful API responses.
    total_success: Counter,
    /// Total failed API responses.
    total_errors: Counter,
}

impl NoesisMetrics {
    /// Create a new metrics collector with pre-initialized endpoint counters.
    pub fn new() -> Self {
        let metrics = Self {
            api_calls: Arc::new(RwLock::new(HashMap::new())),
            cache_hits: Arc::new(RwLock::new(HashMap::new())),
            cache_misses: Arc::new(RwLock::new(HashMap::new())),
            fallback_triggers: Arc::new(RwLock::new(HashMap::new())),
            response_times: Arc::new(RwLock::new(HashMap::new())),
            error_counts: Arc::new(RwLock::new(HashMap::new())),
            total_success: Counter::new(),
            total_errors: Counter::new(),
        };
        metrics
    }

    // ==================== Recording Methods ====================

    /// Record an API call to the given endpoint.
    pub async fn record_api_call(&self, endpoint: &str) {
        let mut map = self.api_calls.write().await;
        map.entry(endpoint.to_string())
            .or_insert_with(Counter::new)
            .increment();
    }

    /// Record a cache hit for the given endpoint.
    pub async fn record_cache_hit(&self, endpoint: &str) {
        let mut map = self.cache_hits.write().await;
        map.entry(endpoint.to_string())
            .or_insert_with(Counter::new)
            .increment();
    }

    /// Record a cache miss for the given endpoint.
    pub async fn record_cache_miss(&self, endpoint: &str) {
        let mut map = self.cache_misses.write().await;
        map.entry(endpoint.to_string())
            .or_insert_with(Counter::new)
            .increment();
    }

    /// Record a fallback trigger for the given endpoint.
    pub async fn record_fallback_trigger(&self, endpoint: &str) {
        let mut map = self.fallback_triggers.write().await;
        map.entry(endpoint.to_string())
            .or_insert_with(Counter::new)
            .increment();
    }

    /// Record the response time for the given endpoint.
    pub async fn record_response_time(&self, endpoint: &str, duration: Duration) {
        let mut map = self.response_times.write().await;
        map.entry(endpoint.to_string())
            .or_insert_with(Histogram::new)
            .observe(duration);
    }

    /// Record an error by type (matches VedicApiError variant names).
    pub async fn record_error(&self, error_type: &str) {
        self.total_errors.increment();
        let mut map = self.error_counts.write().await;
        map.entry(error_type.to_string())
            .or_insert_with(Counter::new)
            .increment();
    }

    /// Record a successful API response.
    pub fn record_success(&self) {
        self.total_success.increment();
    }

    // ==================== Query Methods ====================

    /// Get total API calls for a specific endpoint.
    pub async fn get_api_calls(&self, endpoint: &str) -> u64 {
        let map = self.api_calls.read().await;
        map.get(endpoint).map(|c| c.get()).unwrap_or(0)
    }

    /// Get cache hit count for a specific endpoint.
    pub async fn get_cache_hits(&self, endpoint: &str) -> u64 {
        let map = self.cache_hits.read().await;
        map.get(endpoint).map(|c| c.get()).unwrap_or(0)
    }

    /// Get cache miss count for a specific endpoint.
    pub async fn get_cache_misses(&self, endpoint: &str) -> u64 {
        let map = self.cache_misses.read().await;
        map.get(endpoint).map(|c| c.get()).unwrap_or(0)
    }

    /// Get fallback trigger count for a specific endpoint.
    pub async fn get_fallback_triggers(&self, endpoint: &str) -> u64 {
        let map = self.fallback_triggers.read().await;
        map.get(endpoint).map(|c| c.get()).unwrap_or(0)
    }

    /// Get error count for a specific error type.
    pub async fn get_error_count(&self, error_type: &str) -> u64 {
        let map = self.error_counts.read().await;
        map.get(error_type).map(|c| c.get()).unwrap_or(0)
    }

    /// Get total successful responses.
    pub fn get_total_success(&self) -> u64 {
        self.total_success.get()
    }

    /// Get total error responses.
    pub fn get_total_errors(&self) -> u64 {
        self.total_errors.get()
    }

    /// Calculate cache hit ratio for a specific endpoint (0.0 to 1.0).
    pub async fn cache_hit_ratio(&self, endpoint: &str) -> f64 {
        let hits = self.get_cache_hits(endpoint).await;
        let misses = self.get_cache_misses(endpoint).await;
        let total = hits + misses;
        if total == 0 {
            0.0
        } else {
            hits as f64 / total as f64
        }
    }

    /// Calculate overall cache hit ratio across all endpoints (0.0 to 1.0).
    pub async fn overall_cache_hit_ratio(&self) -> f64 {
        let hits_map = self.cache_hits.read().await;
        let misses_map = self.cache_misses.read().await;

        let total_hits: u64 = hits_map.values().map(|c| c.get()).sum();
        let total_misses: u64 = misses_map.values().map(|c| c.get()).sum();
        let total = total_hits + total_misses;

        if total == 0 {
            0.0
        } else {
            total_hits as f64 / total as f64
        }
    }

    // ==================== Export Methods ====================

    /// Export all metrics in Prometheus text exposition format.
    ///
    /// This produces output compatible with Prometheus scrapers and can be
    /// served directly from an HTTP endpoint.
    pub async fn export_prometheus(&self) -> String {
        let mut output = String::with_capacity(4096);

        // API call counts
        output.push_str("# HELP noesis_api_calls_total Total API calls by endpoint\n");
        output.push_str("# TYPE noesis_api_calls_total counter\n");
        {
            let map = self.api_calls.read().await;
            let mut entries: Vec<_> = map.iter().collect();
            entries.sort_by_key(|(k, _)| (*k).clone());
            for (endpoint, counter) in entries {
                output.push_str(&format!(
                    "noesis_api_calls_total{{endpoint=\"{}\"}} {}\n",
                    endpoint,
                    counter.get()
                ));
            }
        }

        // Cache hits
        output.push_str("# HELP noesis_cache_hits_total Cache hits by endpoint\n");
        output.push_str("# TYPE noesis_cache_hits_total counter\n");
        {
            let map = self.cache_hits.read().await;
            let mut entries: Vec<_> = map.iter().collect();
            entries.sort_by_key(|(k, _)| (*k).clone());
            for (endpoint, counter) in entries {
                output.push_str(&format!(
                    "noesis_cache_hits_total{{endpoint=\"{}\"}} {}\n",
                    endpoint,
                    counter.get()
                ));
            }
        }

        // Cache misses
        output.push_str("# HELP noesis_cache_misses_total Cache misses by endpoint\n");
        output.push_str("# TYPE noesis_cache_misses_total counter\n");
        {
            let map = self.cache_misses.read().await;
            let mut entries: Vec<_> = map.iter().collect();
            entries.sort_by_key(|(k, _)| (*k).clone());
            for (endpoint, counter) in entries {
                output.push_str(&format!(
                    "noesis_cache_misses_total{{endpoint=\"{}\"}} {}\n",
                    endpoint,
                    counter.get()
                ));
            }
        }

        // Fallback triggers
        output.push_str("# HELP noesis_fallback_triggers_total Fallback triggers by endpoint\n");
        output.push_str("# TYPE noesis_fallback_triggers_total counter\n");
        {
            let map = self.fallback_triggers.read().await;
            let mut entries: Vec<_> = map.iter().collect();
            entries.sort_by_key(|(k, _)| (*k).clone());
            for (endpoint, counter) in entries {
                output.push_str(&format!(
                    "noesis_fallback_triggers_total{{endpoint=\"{}\"}} {}\n",
                    endpoint,
                    counter.get()
                ));
            }
        }

        // Error counts
        output.push_str("# HELP noesis_errors_total Errors by type\n");
        output.push_str("# TYPE noesis_errors_total counter\n");
        {
            let map = self.error_counts.read().await;
            let mut entries: Vec<_> = map.iter().collect();
            entries.sort_by_key(|(k, _)| (*k).clone());
            for (error_type, counter) in entries {
                output.push_str(&format!(
                    "noesis_errors_total{{error_type=\"{}\"}} {}\n",
                    error_type,
                    counter.get()
                ));
            }
        }

        // Total success/error
        output.push_str("# HELP noesis_responses_total Total responses by status\n");
        output.push_str("# TYPE noesis_responses_total counter\n");
        output.push_str(&format!(
            "noesis_responses_total{{status=\"success\"}} {}\n",
            self.total_success.get()
        ));
        output.push_str(&format!(
            "noesis_responses_total{{status=\"error\"}} {}\n",
            self.total_errors.get()
        ));

        // Response time histograms
        output.push_str("# HELP noesis_response_time_seconds Response time distribution\n");
        output.push_str("# TYPE noesis_response_time_seconds histogram\n");
        {
            let map = self.response_times.read().await;
            let mut entries: Vec<_> = map.iter().collect();
            entries.sort_by_key(|(k, _)| (*k).clone());
            for (endpoint, histogram) in entries {
                for (boundary_ms, count) in histogram.bucket_counts() {
                    let boundary_sec = boundary_ms as f64 / 1000.0;
                    output.push_str(&format!(
                        "noesis_response_time_seconds_bucket{{endpoint=\"{}\",le=\"{:.3}\"}} {}\n",
                        endpoint, boundary_sec, count
                    ));
                }
                output.push_str(&format!(
                    "noesis_response_time_seconds_bucket{{endpoint=\"{}\",le=\"+Inf\"}} {}\n",
                    endpoint,
                    histogram.count()
                ));
                output.push_str(&format!(
                    "noesis_response_time_seconds_sum{{endpoint=\"{}\"}} {:.6}\n",
                    endpoint,
                    histogram.sum_seconds()
                ));
                output.push_str(&format!(
                    "noesis_response_time_seconds_count{{endpoint=\"{}\"}} {}\n",
                    endpoint,
                    histogram.count()
                ));
            }
        }

        output
    }

    /// Export a compact JSON summary suitable for logging or health checks.
    pub async fn export_json_summary(&self) -> serde_json::Value {
        let api_calls_map = self.api_calls.read().await;
        let cache_hits_map = self.cache_hits.read().await;
        let cache_misses_map = self.cache_misses.read().await;
        let fallback_map = self.fallback_triggers.read().await;
        let error_map = self.error_counts.read().await;

        let api_calls: HashMap<String, u64> = api_calls_map
            .iter()
            .map(|(k, v)| (k.clone(), v.get()))
            .collect();

        let cache_hits: HashMap<String, u64> = cache_hits_map
            .iter()
            .map(|(k, v)| (k.clone(), v.get()))
            .collect();

        let cache_misses: HashMap<String, u64> = cache_misses_map
            .iter()
            .map(|(k, v)| (k.clone(), v.get()))
            .collect();

        let fallbacks: HashMap<String, u64> = fallback_map
            .iter()
            .map(|(k, v)| (k.clone(), v.get()))
            .collect();

        let errors: HashMap<String, u64> = error_map
            .iter()
            .map(|(k, v)| (k.clone(), v.get()))
            .collect();

        serde_json::json!({
            "api_calls": api_calls,
            "cache_hits": cache_hits,
            "cache_misses": cache_misses,
            "cache_hit_ratio": self.overall_cache_hit_ratio().await,
            "fallback_triggers": fallbacks,
            "errors": errors,
            "total_success": self.total_success.get(),
            "total_errors": self.total_errors.get(),
        })
    }

    /// Reset all metrics (useful for testing or periodic resets).
    pub async fn reset(&self) {
        self.api_calls.write().await.clear();
        self.cache_hits.write().await.clear();
        self.cache_misses.write().await.clear();
        self.fallback_triggers.write().await.clear();
        self.response_times.write().await.clear();
        self.error_counts.write().await.clear();
        self.total_success.reset();
        self.total_errors.reset();
    }
}

impl Default for NoesisMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Classify a VedicApiError into its error type string for metrics recording.
pub fn error_type_label(err: &crate::error::VedicApiError) -> &'static str {
    use crate::error::VedicApiError;
    match err {
        VedicApiError::Configuration { .. } => "configuration",
        VedicApiError::Network { .. } => "network",
        VedicApiError::Api { .. } => "api",
        VedicApiError::RateLimit { .. } => "rate_limit",
        VedicApiError::InvalidInput { .. } => "invalid_input",
        VedicApiError::Parse { .. } => "parse",
        VedicApiError::CircuitBreakerOpen => "circuit_breaker_open",
        VedicApiError::Cache { .. } => "cache",
        VedicApiError::FallbackFailed { .. } => "fallback_failed",
        VedicApiError::ParseError(_) => "parse_error",
        VedicApiError::NetworkError(_) => "network_error",
        VedicApiError::Timeout(_) => "timeout",
        VedicApiError::RateLimited { .. } => "rate_limited",
        VedicApiError::ServiceUnavailable(_) => "service_unavailable",
    }
}

/// Helper to time an async operation and record its duration.
///
/// # Example
///
/// ```ignore
/// let result = timed_operation(&metrics, "panchang", async {
///     client.get_panchang(...).await
/// }).await;
/// ```
pub async fn timed_operation<F, T>(
    metrics: &NoesisMetrics,
    endpoint: &str,
    operation: F,
) -> crate::error::Result<T>
where
    F: std::future::Future<Output = crate::error::Result<T>>,
{
    let start = std::time::Instant::now();
    metrics.record_api_call(endpoint).await;

    let result = operation.await;
    let duration = start.elapsed();

    metrics.record_response_time(endpoint, duration).await;

    match &result {
        Ok(_) => metrics.record_success(),
        Err(err) => {
            let label = error_type_label(err);
            metrics.record_error(label).await;
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_api_call_counting() {
        let metrics = NoesisMetrics::new();

        metrics.record_api_call("panchang").await;
        metrics.record_api_call("panchang").await;
        metrics.record_api_call("birth_chart").await;

        assert_eq!(metrics.get_api_calls("panchang").await, 2);
        assert_eq!(metrics.get_api_calls("birth_chart").await, 1);
        assert_eq!(metrics.get_api_calls("nonexistent").await, 0);
    }

    #[tokio::test]
    async fn test_cache_hit_miss_tracking() {
        let metrics = NoesisMetrics::new();

        metrics.record_cache_hit("panchang").await;
        metrics.record_cache_hit("panchang").await;
        metrics.record_cache_hit("panchang").await;
        metrics.record_cache_miss("panchang").await;

        assert_eq!(metrics.get_cache_hits("panchang").await, 3);
        assert_eq!(metrics.get_cache_misses("panchang").await, 1);
    }

    #[tokio::test]
    async fn test_cache_hit_ratio() {
        let metrics = NoesisMetrics::new();

        // 3 hits, 1 miss = 75% hit ratio
        metrics.record_cache_hit("panchang").await;
        metrics.record_cache_hit("panchang").await;
        metrics.record_cache_hit("panchang").await;
        metrics.record_cache_miss("panchang").await;

        let ratio = metrics.cache_hit_ratio("panchang").await;
        assert!((ratio - 0.75).abs() < f64::EPSILON);

        // No data = 0.0
        let empty_ratio = metrics.cache_hit_ratio("empty").await;
        assert!((empty_ratio - 0.0).abs() < f64::EPSILON);
    }

    #[tokio::test]
    async fn test_overall_cache_hit_ratio() {
        let metrics = NoesisMetrics::new();

        // panchang: 3 hits, 1 miss
        metrics.record_cache_hit("panchang").await;
        metrics.record_cache_hit("panchang").await;
        metrics.record_cache_hit("panchang").await;
        metrics.record_cache_miss("panchang").await;

        // birth_chart: 1 hit, 1 miss
        metrics.record_cache_hit("birth_chart").await;
        metrics.record_cache_miss("birth_chart").await;

        // Total: 4 hits / 6 total = 0.666...
        let ratio = metrics.overall_cache_hit_ratio().await;
        assert!((ratio - 4.0 / 6.0).abs() < 0.001);
    }

    #[tokio::test]
    async fn test_fallback_trigger_counting() {
        let metrics = NoesisMetrics::new();

        metrics.record_fallback_trigger("panchang").await;
        metrics.record_fallback_trigger("panchang").await;
        metrics.record_fallback_trigger("birth_chart").await;

        assert_eq!(metrics.get_fallback_triggers("panchang").await, 2);
        assert_eq!(metrics.get_fallback_triggers("birth_chart").await, 1);
    }

    #[tokio::test]
    async fn test_error_counting_by_type() {
        let metrics = NoesisMetrics::new();

        metrics.record_error("network").await;
        metrics.record_error("network").await;
        metrics.record_error("rate_limit").await;

        assert_eq!(metrics.get_error_count("network").await, 2);
        assert_eq!(metrics.get_error_count("rate_limit").await, 1);
        assert_eq!(metrics.get_total_errors(), 3);
    }

    #[tokio::test]
    async fn test_response_time_histogram() {
        let metrics = NoesisMetrics::new();

        metrics.record_response_time("panchang", Duration::from_millis(5)).await;
        metrics.record_response_time("panchang", Duration::from_millis(50)).await;
        metrics.record_response_time("panchang", Duration::from_millis(500)).await;

        // Verify the Prometheus export contains histogram data
        let output = metrics.export_prometheus().await;
        assert!(output.contains("noesis_response_time_seconds_bucket{endpoint=\"panchang\""));
        assert!(output.contains("noesis_response_time_seconds_count{endpoint=\"panchang\"} 3"));
    }

    #[tokio::test]
    async fn test_success_tracking() {
        let metrics = NoesisMetrics::new();

        metrics.record_success();
        metrics.record_success();

        assert_eq!(metrics.get_total_success(), 2);
    }

    #[tokio::test]
    async fn test_prometheus_export_format() {
        let metrics = NoesisMetrics::new();

        metrics.record_api_call("panchang").await;
        metrics.record_cache_hit("panchang").await;
        metrics.record_cache_miss("birth_chart").await;
        metrics.record_fallback_trigger("panchang").await;
        metrics.record_error("network").await;
        metrics.record_success();
        metrics.record_response_time("panchang", Duration::from_millis(100)).await;

        let output = metrics.export_prometheus().await;

        // Verify structure
        assert!(output.contains("# HELP noesis_api_calls_total"));
        assert!(output.contains("# TYPE noesis_api_calls_total counter"));
        assert!(output.contains("noesis_api_calls_total{endpoint=\"panchang\"} 1"));

        assert!(output.contains("# HELP noesis_cache_hits_total"));
        assert!(output.contains("noesis_cache_hits_total{endpoint=\"panchang\"} 1"));

        assert!(output.contains("# HELP noesis_cache_misses_total"));
        assert!(output.contains("noesis_cache_misses_total{endpoint=\"birth_chart\"} 1"));

        assert!(output.contains("# HELP noesis_fallback_triggers_total"));
        assert!(output.contains("noesis_fallback_triggers_total{endpoint=\"panchang\"} 1"));

        assert!(output.contains("# HELP noesis_errors_total"));
        assert!(output.contains("noesis_errors_total{error_type=\"network\"} 1"));

        assert!(output.contains("noesis_responses_total{status=\"success\"} 1"));
        assert!(output.contains("noesis_responses_total{status=\"error\"} 1"));

        assert!(output.contains("# TYPE noesis_response_time_seconds histogram"));
        assert!(output.contains("noesis_response_time_seconds_bucket{endpoint=\"panchang\",le=\"+Inf\"} 1"));
    }

    #[tokio::test]
    async fn test_json_summary_export() {
        let metrics = NoesisMetrics::new();

        metrics.record_api_call("panchang").await;
        metrics.record_cache_hit("panchang").await;
        metrics.record_cache_miss("panchang").await;
        metrics.record_error("network").await;
        metrics.record_success();

        let json = metrics.export_json_summary().await;

        assert_eq!(json["api_calls"]["panchang"], 1);
        assert_eq!(json["cache_hits"]["panchang"], 1);
        assert_eq!(json["cache_misses"]["panchang"], 1);
        assert_eq!(json["total_success"], 1);
        assert_eq!(json["total_errors"], 1);
        assert!(json["cache_hit_ratio"].as_f64().unwrap() > 0.0);
    }

    #[tokio::test]
    async fn test_reset_clears_all_metrics() {
        let metrics = NoesisMetrics::new();

        metrics.record_api_call("panchang").await;
        metrics.record_cache_hit("panchang").await;
        metrics.record_error("network").await;
        metrics.record_success();

        metrics.reset().await;

        assert_eq!(metrics.get_api_calls("panchang").await, 0);
        assert_eq!(metrics.get_cache_hits("panchang").await, 0);
        assert_eq!(metrics.get_error_count("network").await, 0);
        assert_eq!(metrics.get_total_success(), 0);
        assert_eq!(metrics.get_total_errors(), 0);
    }

    #[test]
    fn test_error_type_label_mapping() {
        use crate::error::VedicApiError;

        assert_eq!(
            error_type_label(&VedicApiError::Network { message: "test".into() }),
            "network"
        );
        assert_eq!(
            error_type_label(&VedicApiError::RateLimit { retry_after: None }),
            "rate_limit"
        );
        assert_eq!(
            error_type_label(&VedicApiError::CircuitBreakerOpen),
            "circuit_breaker_open"
        );
        assert_eq!(
            error_type_label(&VedicApiError::Configuration {
                field: "k".into(),
                message: "v".into()
            }),
            "configuration"
        );
        assert_eq!(
            error_type_label(&VedicApiError::Parse { message: "x".into() }),
            "parse"
        );
    }

    #[tokio::test]
    async fn test_histogram_bucket_distribution() {
        let metrics = NoesisMetrics::new();

        // Record values that fall into different buckets
        metrics.record_response_time("test", Duration::from_millis(1)).await;   // <= 1ms bucket
        metrics.record_response_time("test", Duration::from_millis(50)).await;  // <= 50ms bucket
        metrics.record_response_time("test", Duration::from_millis(5000)).await; // <= 5000ms bucket

        let output = metrics.export_prometheus().await;

        // The 1ms observation should be in the 1ms bucket
        assert!(output.contains("noesis_response_time_seconds_bucket{endpoint=\"test\",le=\"0.001\"} 1"));
        // The 50ms observation should be in the 50ms bucket (plus the 1ms one)
        assert!(output.contains("noesis_response_time_seconds_bucket{endpoint=\"test\",le=\"0.050\"} 2"));
        // The 5000ms observation should be in the 5000ms bucket (plus the previous ones)
        assert!(output.contains("noesis_response_time_seconds_bucket{endpoint=\"test\",le=\"5.000\"} 3"));
        // +Inf should contain all 3
        assert!(output.contains("noesis_response_time_seconds_bucket{endpoint=\"test\",le=\"+Inf\"} 3"));
    }

    #[tokio::test]
    async fn test_concurrent_metric_recording() {
        let metrics = Arc::new(NoesisMetrics::new());
        let mut handles = Vec::new();

        for _ in 0..100 {
            let m = metrics.clone();
            handles.push(tokio::spawn(async move {
                m.record_api_call("panchang").await;
                m.record_cache_hit("panchang").await;
            }));
        }

        for handle in handles {
            handle.await.unwrap();
        }

        assert_eq!(metrics.get_api_calls("panchang").await, 100);
        assert_eq!(metrics.get_cache_hits("panchang").await, 100);
    }
}
