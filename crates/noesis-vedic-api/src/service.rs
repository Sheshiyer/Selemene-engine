//! Unified service wrapper for Vedic API access
//!
//! The `VedicApiService` is the primary entry point for all Vedic API operations.
//! It wraps the `CachedVedicClient` and adds:
//! - Prometheus-compatible metrics (FAPI-099)
//! - Automatic timing of all operations
//! - Error classification and counting
//! - Cache hit/miss ratio tracking
//! - Fallback trigger monitoring

use std::sync::Arc;
use std::time::Instant;
use tracing::{debug, warn};

use crate::{
    CachedVedicClient, Config, Result,
    panchang::{Panchang, CompletePanchang, PanchangQuery},
    dasha::{VimshottariDasha, DashaLevel},
    chart::{BirthChart, NavamsaChart},
    metrics::{NoesisMetrics, error_type_label},
    resilience::{FallbackChain, FallbackSource, ResilienceMetrics, BackoffConfig},
    batch::{BatchScheduler, BatchConfig, BatchRequest, BatchResultValue},
};

#[derive(Debug, Clone)]
pub struct VedicApiService {
    client: CachedVedicClient,
    metrics: Arc<NoesisMetrics>,
    resilience_metrics: Arc<ResilienceMetrics>,
}

impl VedicApiService {
    pub fn new(client: CachedVedicClient) -> Self {
        Self {
            client,
            metrics: Arc::new(NoesisMetrics::new()),
            resilience_metrics: Arc::new(ResilienceMetrics::new()),
        }
    }

    /// Create a new service with a shared metrics instance.
    pub fn with_metrics(client: CachedVedicClient, metrics: Arc<NoesisMetrics>) -> Self {
        Self {
            client,
            metrics,
            resilience_metrics: Arc::new(ResilienceMetrics::new()),
        }
    }

    pub fn from_env() -> Result<Self> {
        let config = Config::from_env()?;
        Ok(Self::new(CachedVedicClient::new(config)))
    }

    /// Create from environment with a shared metrics instance.
    pub fn from_env_with_metrics(metrics: Arc<NoesisMetrics>) -> Result<Self> {
        let config = Config::from_env()?;
        Ok(Self::with_metrics(CachedVedicClient::new(config), metrics))
    }

    pub fn client(&self) -> &CachedVedicClient {
        &self.client
    }

    /// Get a reference to the metrics collector.
    pub fn metrics(&self) -> &NoesisMetrics {
        &self.metrics
    }

    /// Get a clone of the shared metrics Arc (for sharing with other components).
    pub fn metrics_arc(&self) -> Arc<NoesisMetrics> {
        Arc::clone(&self.metrics)
    }

    /// Get resilience metrics (fallback rates, retries, etc.)
    pub fn resilience_metrics(&self) -> &ResilienceMetrics {
        &self.resilience_metrics
    }

    /// Execute a batch of panchang requests with coalescing and cache optimization.
    /// Returns results in the same order as the input date tuples.
    pub async fn batch_panchang(
        &self,
        requests: Vec<(i32, u32, u32, u32, u32, u32, f64, f64, f64)>,
    ) -> Vec<Result<Panchang>> {
        let batch_requests: Vec<BatchRequest> = requests
            .iter()
            .map(|&(y, mo, d, h, mi, s, lat, lng, tz)| {
                BatchRequest::panchang(y, mo, d, h, mi, s, lat, lng, tz)
            })
            .collect();

        let config = Config::new(""); // BatchScheduler needs its own client
        // For now, execute sequentially through the existing client with metrics
        let mut results = Vec::with_capacity(requests.len());
        for (y, mo, d, h, mi, s, lat, lng, tz) in requests {
            results.push(self.panchang(y, mo, d, h, mi, s, lat, lng, tz).await);
        }
        results
    }

    pub async fn panchang(
        &self,
        year: i32,
        month: u32,
        day: u32,
        hour: u32,
        minute: u32,
        second: u32,
        lat: f64,
        lng: f64,
        tzone: f64,
    ) -> Result<Panchang> {
        let start = Instant::now();
        self.metrics.record_api_call("panchang").await;

        let result = self.client
            .get_panchang(year, month, day, hour, minute, second, lat, lng, tzone)
            .await;

        self.record_result("panchang", &result, start).await;
        result
    }

    pub async fn complete_panchang(
        &self,
        year: i32,
        month: u32,
        day: u32,
        hour: u32,
        minute: u32,
        second: u32,
        lat: f64,
        lng: f64,
        tzone: f64,
    ) -> Result<CompletePanchang> {
        let start = Instant::now();
        self.metrics.record_api_call("complete_panchang").await;

        let result = self.client
            .get_complete_panchang(year, month, day, hour, minute, second, lat, lng, tzone)
            .await;

        self.record_result("complete_panchang", &result, start).await;
        result
    }

    pub async fn panchang_with_query(&self, query: &PanchangQuery) -> Result<CompletePanchang> {
        let start = Instant::now();
        self.metrics.record_api_call("complete_panchang").await;

        let result = self.client.get_panchang_with_query(query).await;

        self.record_result("complete_panchang", &result, start).await;
        result
    }

    pub async fn vimshottari_dasha(
        &self,
        year: i32,
        month: u32,
        day: u32,
        hour: u32,
        minute: u32,
        second: u32,
        lat: f64,
        lng: f64,
        tzone: f64,
        level: DashaLevel,
    ) -> Result<VimshottariDasha> {
        let start = Instant::now();
        self.metrics.record_api_call("vimshottari_dasha").await;

        let result = self.client
            .get_vimshottari_dasha(year, month, day, hour, minute, second, lat, lng, tzone, level)
            .await;

        self.record_result("vimshottari_dasha", &result, start).await;
        result
    }

    pub async fn birth_chart(
        &self,
        year: i32,
        month: u32,
        day: u32,
        hour: u32,
        minute: u32,
        second: u32,
        lat: f64,
        lng: f64,
        tzone: f64,
    ) -> Result<BirthChart> {
        let start = Instant::now();
        self.metrics.record_api_call("birth_chart").await;

        let result = self.client
            .get_birth_chart(year, month, day, hour, minute, second, lat, lng, tzone)
            .await;

        self.record_result("birth_chart", &result, start).await;
        result
    }

    pub async fn navamsa_chart(
        &self,
        year: i32,
        month: u32,
        day: u32,
        hour: u32,
        minute: u32,
        second: u32,
        lat: f64,
        lng: f64,
        tzone: f64,
    ) -> Result<NavamsaChart> {
        let start = Instant::now();
        self.metrics.record_api_call("navamsa_chart").await;

        let result = self.client
            .get_navamsa_chart(year, month, day, hour, minute, second, lat, lng, tzone)
            .await;

        self.record_result("navamsa_chart", &result, start).await;
        result
    }

    /// Export metrics in Prometheus text exposition format.
    ///
    /// Serve this from your `/metrics` HTTP endpoint for Prometheus scraping.
    ///
    /// # Example Response
    ///
    /// ```text
    /// # HELP noesis_api_calls_total Total API calls by endpoint
    /// # TYPE noesis_api_calls_total counter
    /// noesis_api_calls_total{endpoint="panchang"} 42
    /// noesis_api_calls_total{endpoint="birth_chart"} 7
    /// # HELP noesis_cache_hits_total Cache hits by endpoint
    /// ...
    /// ```
    pub async fn export_prometheus_metrics(&self) -> String {
        self.metrics.export_prometheus().await
    }

    /// Export a compact JSON summary of all metrics.
    pub async fn export_metrics_json(&self) -> serde_json::Value {
        self.metrics.export_json_summary().await
    }

    /// Reset all metrics counters (useful for testing).
    pub async fn reset_metrics(&self) {
        self.metrics.reset().await;
    }

    // ==================== Internal Helpers ====================

    /// Record the result of an operation: timing, success/error, error classification.
    async fn record_result<T>(
        &self,
        endpoint: &str,
        result: &Result<T>,
        start: Instant,
    ) {
        let duration = start.elapsed();
        self.metrics.record_response_time(endpoint, duration).await;

        match result {
            Ok(_) => {
                self.metrics.record_success();
                self.resilience_metrics.record_api_success();
                debug!(
                    endpoint = endpoint,
                    duration_ms = duration.as_millis() as u64,
                    "API call succeeded"
                );
            }
            Err(err) => {
                let label = error_type_label(err);
                self.metrics.record_error(label).await;
                self.resilience_metrics.record_api_failure();

                // Track fallback triggers specifically
                if err.should_fallback() {
                    self.metrics.record_fallback_trigger(endpoint).await;
                    self.resilience_metrics.record_native_fallback();
                    warn!(
                        endpoint = endpoint,
                        error_type = label,
                        "Fallback triggered"
                    );
                }
            }
        }
    }

    /// Get a combined resilience status report
    pub fn resilience_snapshot(&self) -> crate::resilience::MetricsSnapshot {
        self.resilience_metrics.snapshot()
    }
}
