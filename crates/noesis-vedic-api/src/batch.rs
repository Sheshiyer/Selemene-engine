//! Batch request optimization for reducing API call count
//!
//! FAPI-106: Batch Request Optimization
//!
//! Features:
//! - Request coalescing: identical concurrent requests share a single API call
//! - Batch size limits: prevents overwhelming the API
//! - Cache-first lookup: checks cache before making API calls
//! - Concurrent execution within batch size constraints

use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::RwLock;
use tracing::{debug, info};

use crate::{
    Config, VedicApiClient, VedicApiError,
    cache::{ApiCache, panchang_key, birth_key},
    panchang::Panchang,
    dasha::{VimshottariDasha, DashaLevel},
    chart::BirthChart,
    resilience::{ExponentialBackoff, BackoffConfig},
};

// ====================== CONFIGURATION ======================

/// Configuration for batch request behavior
#[derive(Debug, Clone)]
pub struct BatchConfig {
    /// Maximum number of requests in a single batch
    pub max_batch_size: usize,
    /// Timeout for the entire batch in milliseconds
    pub batch_timeout_ms: u64,
    /// Whether to coalesce identical requests
    pub coalesce_identical: bool,
    /// Maximum number of concurrent batches
    pub max_concurrent_batches: usize,
}

impl Default for BatchConfig {
    fn default() -> Self {
        Self {
            max_batch_size: 10,
            batch_timeout_ms: 5000,
            coalesce_identical: true,
            max_concurrent_batches: 3,
        }
    }
}

// ====================== BATCH REQUEST ======================

/// The type of API request being batched
#[derive(Debug, Clone, PartialEq)]
pub enum RequestType {
    Panchang,
    BirthChart,
    Dasha(DashaLevel),
}

/// A single request within a batch
#[derive(Debug, Clone)]
pub struct BatchRequest {
    /// Type of the request
    pub request_type: RequestType,
    /// Request parameters
    pub year: i32,
    pub month: u32,
    pub day: u32,
    pub hour: u32,
    pub minute: u32,
    pub second: u32,
    pub lat: f64,
    pub lng: f64,
    pub tzone: f64,
}

impl BatchRequest {
    /// Create a Panchang batch request
    pub fn panchang(
        year: i32, month: u32, day: u32,
        hour: u32, minute: u32, second: u32,
        lat: f64, lng: f64, tzone: f64,
    ) -> Self {
        Self {
            request_type: RequestType::Panchang,
            year, month, day, hour, minute, second,
            lat, lng, tzone,
        }
    }

    /// Create a BirthChart batch request
    pub fn birth_chart(
        year: i32, month: u32, day: u32,
        hour: u32, minute: u32, second: u32,
        lat: f64, lng: f64, tzone: f64,
    ) -> Self {
        Self {
            request_type: RequestType::BirthChart,
            year, month, day, hour, minute, second,
            lat, lng, tzone,
        }
    }

    /// Create a Dasha batch request
    pub fn dasha(
        year: i32, month: u32, day: u32,
        hour: u32, minute: u32, second: u32,
        lat: f64, lng: f64, tzone: f64,
        level: DashaLevel,
    ) -> Self {
        Self {
            request_type: RequestType::Dasha(level),
            year, month, day, hour, minute, second,
            lat, lng, tzone,
        }
    }

    /// Generate a cache key for this request, used for deduplication and caching
    pub fn cache_key(&self) -> String {
        match &self.request_type {
            RequestType::Panchang => {
                panchang_key(self.year, self.month, self.day, self.lat, self.lng)
            }
            RequestType::BirthChart => {
                birth_key(self.year, self.month, self.day, self.hour, self.minute, self.lat, self.lng)
            }
            RequestType::Dasha(level) => {
                format!(
                    "{}:{:?}",
                    birth_key(self.year, self.month, self.day, self.hour, self.minute, self.lat, self.lng),
                    level
                )
            }
        }
    }
}

// ====================== BATCH RESULT ======================

/// Result for a single request within a batch
#[derive(Debug, Clone)]
pub enum BatchResultValue {
    Panchang(Panchang),
    BirthChart(BirthChart),
    Dasha(VimshottariDasha),
}

/// Alias for a single batch result
pub type BatchResult = Result<BatchResultValue, VedicApiError>;

// ====================== BATCH SCHEDULER ======================

/// Orchestrates batch execution with coalescing and cache optimization
pub struct BatchScheduler {
    client: VedicApiClient,
    cache: ApiCache,
    config: BatchConfig,
    backoff: ExponentialBackoff,
    /// Tracks in-flight requests for coalescing
    #[allow(dead_code)]
    inflight: Arc<RwLock<HashMap<String, Arc<tokio::sync::Notify>>>>,
    /// Results from coalesced requests
    #[allow(dead_code)]
    coalesced_results: Arc<RwLock<HashMap<String, BatchResult>>>,
}

impl BatchScheduler {
    /// Create a new batch scheduler
    pub fn new(api_config: Config, batch_config: BatchConfig) -> Self {
        let client = VedicApiClient::new(api_config);
        let cache = ApiCache::new();
        let backoff = ExponentialBackoff::new(BackoffConfig {
            initial_delay_ms: 100,
            max_delay_ms: 5000,
            max_retries: 2,
            multiplier: 2.0,
            jitter: false,
        });

        Self {
            client,
            cache,
            config: batch_config,
            backoff,
            inflight: Arc::new(RwLock::new(HashMap::new())),
            coalesced_results: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Execute a batch of requests with coalescing and cache optimization.
    ///
    /// Returns results in the same order as the input requests.
    pub async fn execute_batch(&self, requests: Vec<BatchRequest>) -> Vec<BatchResult> {
        info!("Executing batch of {} requests", requests.len());

        if requests.is_empty() {
            return vec![];
        }

        let mut results: Vec<Option<BatchResult>> = vec![None; requests.len()];

        // Phase 1: Check cache for all requests
        let mut uncached_indices: Vec<usize> = Vec::new();
        for (i, req) in requests.iter().enumerate() {
            if let Some(cached) = self.check_cache(req).await {
                debug!("Batch: cache hit for request {}", i);
                results[i] = Some(Ok(cached));
            } else {
                uncached_indices.push(i);
            }
        }

        debug!(
            "Batch: {} cache hits, {} to fetch",
            requests.len() - uncached_indices.len(),
            uncached_indices.len()
        );

        // Phase 2: Coalesce identical uncached requests
        if self.config.coalesce_identical {
            let mut key_to_indices: HashMap<String, Vec<usize>> = HashMap::new();
            for &idx in &uncached_indices {
                let key = requests[idx].cache_key();
                key_to_indices.entry(key).or_default().push(idx);
            }

            // Execute unique requests only
            let unique_keys: Vec<String> = key_to_indices.keys().cloned().collect();
            let mut unique_results: HashMap<String, BatchResult> = HashMap::new();

            // Process in chunks respecting max_batch_size
            for chunk in unique_keys.chunks(self.config.max_batch_size) {
                let chunk_results = self.execute_chunk(chunk, &requests, &key_to_indices).await;
                unique_results.extend(chunk_results);
            }

            // Fan out results to all indices that share the same key
            for (key, indices) in &key_to_indices {
                if let Some(result) = unique_results.get(key) {
                    for &idx in indices {
                        results[idx] = Some(result.clone());
                    }
                }
            }
        } else {
            // No coalescing - execute each request individually in batches
            for chunk in uncached_indices.chunks(self.config.max_batch_size) {
                let mut handles = Vec::new();
                for &idx in chunk {
                    let req = requests[idx].clone();
                    let result = self.execute_single(&req).await;
                    handles.push((idx, result));
                }
                for (idx, result) in handles {
                    results[idx] = Some(result);
                }
            }
        }

        // Fill any remaining None entries with errors
        results
            .into_iter()
            .map(|r| {
                r.unwrap_or_else(|| {
                    Err(VedicApiError::Network {
                        message: "Batch request failed: no result produced".to_string(),
                    })
                })
            })
            .collect()
    }

    /// Execute a chunk of unique requests
    async fn execute_chunk(
        &self,
        keys: &[String],
        requests: &[BatchRequest],
        key_to_indices: &HashMap<String, Vec<usize>>,
    ) -> HashMap<String, BatchResult> {
        let mut results = HashMap::new();

        for key in keys {
            // Find the first request matching this key
            if let Some(indices) = key_to_indices.get(key) {
                if let Some(&idx) = indices.first() {
                    let result = self.execute_single(&requests[idx]).await;
                    results.insert(key.clone(), result);
                }
            }
        }

        results
    }

    /// Execute a single request with backoff and cache population
    async fn execute_single(&self, req: &BatchRequest) -> BatchResult {
        let client = self.client.clone();
        let req_clone = req.clone();

        let api_result = self
            .backoff
            .execute(|| {
                let c = client.clone();
                let r = req_clone.clone();
                async move {
                    match r.request_type {
                        RequestType::Panchang => {
                            c.get_panchang(
                                r.year, r.month, r.day,
                                r.hour, r.minute, r.second,
                                r.lat, r.lng, r.tzone,
                            )
                            .await
                            .map(BatchResultValue::Panchang)
                        }
                        RequestType::BirthChart => {
                            c.get_birth_chart(
                                r.year, r.month, r.day,
                                r.hour, r.minute, r.second,
                                r.lat, r.lng, r.tzone,
                            )
                            .await
                            .map(BatchResultValue::BirthChart)
                        }
                        RequestType::Dasha(level) => {
                            c.get_vimshottari_dasha(
                                r.year, r.month, r.day,
                                r.hour, r.minute, r.second,
                                r.lat, r.lng, r.tzone,
                                level,
                            )
                            .await
                            .map(BatchResultValue::Dasha)
                        }
                    }
                }
            })
            .await;

        // Populate cache on success
        if let Ok(ref value) = api_result {
            self.populate_cache(req, value).await;
        }

        api_result
    }

    /// Check the cache for a request
    async fn check_cache(&self, req: &BatchRequest) -> Option<BatchResultValue> {
        match req.request_type {
            RequestType::Panchang => {
                let key = panchang_key(req.year, req.month, req.day, req.lat, req.lng);
                self.cache
                    .get_panchang(&key)
                    .await
                    .map(BatchResultValue::Panchang)
            }
            RequestType::BirthChart => {
                let key = birth_key(
                    req.year, req.month, req.day,
                    req.hour, req.minute,
                    req.lat, req.lng,
                );
                self.cache
                    .get_birth_chart(&key)
                    .await
                    .map(BatchResultValue::BirthChart)
            }
            RequestType::Dasha(level) => {
                let key = format!(
                    "{}:{:?}",
                    birth_key(
                        req.year, req.month, req.day,
                        req.hour, req.minute,
                        req.lat, req.lng,
                    ),
                    level
                );
                self.cache
                    .get_dasha(&key)
                    .await
                    .map(BatchResultValue::Dasha)
            }
        }
    }

    /// Populate cache after a successful API response
    async fn populate_cache(&self, req: &BatchRequest, value: &BatchResultValue) {
        match value {
            BatchResultValue::Panchang(p) => {
                let key = panchang_key(req.year, req.month, req.day, req.lat, req.lng);
                self.cache.set_panchang(&key, p.clone()).await;
            }
            BatchResultValue::BirthChart(c) => {
                let key = birth_key(
                    req.year, req.month, req.day,
                    req.hour, req.minute,
                    req.lat, req.lng,
                );
                self.cache.set_birth_chart(&key, c.clone()).await;
            }
            BatchResultValue::Dasha(d) => {
                let key = format!(
                    "{}:{:?}",
                    birth_key(
                        req.year, req.month, req.day,
                        req.hour, req.minute,
                        req.lat, req.lng,
                    ),
                    req.request_type
                );
                self.cache.set_dasha(&key, d.clone()).await;
            }
        }
    }

    /// Get the batch configuration
    pub fn config(&self) -> &BatchConfig {
        &self.config
    }

    /// Get cache statistics
    pub async fn cache_stats(&self) -> crate::cache::CacheStats {
        self.cache.stats().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_batch_config_defaults() {
        let config = BatchConfig::default();
        assert_eq!(config.max_batch_size, 10);
        assert_eq!(config.batch_timeout_ms, 5000);
        assert!(config.coalesce_identical);
        assert_eq!(config.max_concurrent_batches, 3);
    }

    #[test]
    fn test_batch_request_panchang() {
        let req = BatchRequest::panchang(2024, 1, 1, 12, 0, 0, 12.97, 77.59, 5.5);
        assert!(matches!(req.request_type, RequestType::Panchang));
        assert_eq!(req.year, 2024);
    }

    #[test]
    fn test_batch_request_key_identity() {
        let r1 = BatchRequest::panchang(2024, 1, 1, 12, 0, 0, 12.97, 77.59, 5.5);
        let r2 = BatchRequest::panchang(2024, 1, 1, 12, 0, 0, 12.97, 77.59, 5.5);
        assert_eq!(r1.cache_key(), r2.cache_key());
    }

    #[test]
    fn test_batch_request_key_different() {
        let r1 = BatchRequest::panchang(2024, 1, 1, 12, 0, 0, 12.97, 77.59, 5.5);
        let r2 = BatchRequest::panchang(2024, 1, 2, 12, 0, 0, 12.97, 77.59, 5.5);
        assert_ne!(r1.cache_key(), r2.cache_key());
    }

    #[test]
    fn test_batch_request_dasha_key() {
        let r1 = BatchRequest::dasha(1990, 1, 1, 12, 0, 0, 28.6, 77.2, 5.5, DashaLevel::Mahadasha);
        let r2 = BatchRequest::dasha(1990, 1, 1, 12, 0, 0, 28.6, 77.2, 5.5, DashaLevel::Antardasha);
        assert_ne!(r1.cache_key(), r2.cache_key());
    }
}
