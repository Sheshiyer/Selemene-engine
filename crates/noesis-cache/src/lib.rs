//! Noesis Cache -- Multi-layer caching system (L1 in-memory, L2 Redis, L3 disk)
//!
//! Engine-agnostic cache that stores `serde_json::Value` payloads.
//! Engines provide their own cache keys via the `ConsciousnessEngine::cache_key` trait method;
//! this crate hashes those keys and manages L1/L2/L3 storage transparently.

pub mod l1_cache;
pub mod l2_cache;
pub mod l3_cache;

use noesis_core::EngineError;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

use l1_cache::L1Cache;
use l2_cache::L2Cache;
use l3_cache::L3Cache;

// ---------------------------------------------------------------------------
// CacheKey
// ---------------------------------------------------------------------------

/// A deterministic cache key derived from an engine-provided string.
///
/// The raw key is hashed with MD5 to produce a fixed-length hex string that is
/// safe for use as a file name and Redis key.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct CacheKey {
    /// The original key string provided by the engine.
    pub raw: String,
    /// MD5 hex digest of `raw`.
    pub hash: String,
}

impl CacheKey {
    /// Build a `CacheKey` from an arbitrary string (typically produced by
    /// `ConsciousnessEngine::cache_key`).
    pub fn new(raw: impl Into<String>) -> Self {
        let raw = raw.into();
        let hash = format!("{:x}", md5::compute(&raw));
        Self { raw, hash }
    }
}

// ---------------------------------------------------------------------------
// CachedResult
// ---------------------------------------------------------------------------

/// A cached calculation result with access-tracking metadata.
#[derive(Debug, Clone)]
pub struct CachedResult {
    /// The engine output serialised as a JSON value.
    pub value: Value,
    pub created_at: std::time::Instant,
    pub accessed_at: std::time::Instant,
    pub access_count: u64,
}

impl CachedResult {
    pub fn new(value: Value) -> Self {
        let now = std::time::Instant::now();
        Self {
            value,
            created_at: now,
            accessed_at: now,
            access_count: 0,
        }
    }
}

// ---------------------------------------------------------------------------
// CacheStats
// ---------------------------------------------------------------------------

/// Aggregate statistics across all cache layers.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CacheStats {
    pub l1_hits: u64,
    pub l2_hits: u64,
    pub l3_hits: u64,
    pub cache_misses: u64,
    pub total_requests: u64,
}

impl CacheStats {
    pub fn hit_rate(&self) -> f64 {
        if self.total_requests == 0 {
            0.0
        } else {
            (self.l1_hits + self.l2_hits + self.l3_hits) as f64 / self.total_requests as f64
        }
    }
}

// ---------------------------------------------------------------------------
// CacheManager
// ---------------------------------------------------------------------------

/// Multi-layer cache manager (L1 in-memory, L2 Redis, L3 disk).
pub struct CacheManager {
    l1_cache: Arc<L1Cache>,
    l2_cache: Arc<L2Cache>,
    l3_cache: Arc<L3Cache>,
    stats: Arc<RwLock<CacheStats>>,
}

impl CacheManager {
    pub fn new(
        redis_url: String,
        l1_size_mb: usize,
        l2_ttl: Duration,
        l3_enabled: bool,
    ) -> Self {
        Self {
            l1_cache: Arc::new(L1Cache::new(l1_size_mb)),
            l2_cache: Arc::new(L2Cache::new(redis_url, l2_ttl)),
            l3_cache: Arc::new(L3Cache::new(l3_enabled)),
            stats: Arc::new(RwLock::new(CacheStats::default())),
        }
    }

    /// Retrieve a cached value, checking L1 -> L2 -> L3 in order.
    pub async fn get(&self, key: &CacheKey) -> Result<Option<Value>, EngineError> {
        {
            let mut stats = self.stats.write().await;
            stats.total_requests += 1;
        }

        // L1 -- in-memory
        if let Some(value) = self.l1_cache.get(key).await? {
            let mut stats = self.stats.write().await;
            stats.l1_hits += 1;
            return Ok(Some(value));
        }

        // L2 -- Redis
        if let Some(value) = self.l2_cache.get(key).await? {
            // Populate L1
            self.l1_cache.store(key, &value).await?;
            let mut stats = self.stats.write().await;
            stats.l2_hits += 1;
            return Ok(Some(value));
        }

        // L3 -- disk
        if let Some(value) = self.l3_cache.get(key).await? {
            // Populate higher layers
            self.l1_cache.store(key, &value).await?;
            self.l2_cache.store(key, &value).await?;
            let mut stats = self.stats.write().await;
            stats.l3_hits += 1;
            return Ok(Some(value));
        }

        let mut stats = self.stats.write().await;
        stats.cache_misses += 1;
        Ok(None)
    }

    /// Store a value in L1 and L2.
    pub async fn store(&self, key: &CacheKey, value: &Value) -> Result<(), EngineError> {
        self.l1_cache.store(key, value).await?;
        self.l2_cache.store(key, value).await?;
        Ok(())
    }

    /// Store a value in L3 (precomputed / persistent).
    pub async fn store_precomputed(&self, key: &CacheKey, value: &Value) -> Result<(), EngineError> {
        self.l3_cache.store(key, value).await?;
        Ok(())
    }

    /// Invalidate a key across all layers.
    pub async fn invalidate(&self, key: &CacheKey) -> Result<(), EngineError> {
        self.l1_cache.invalidate(key).await?;
        self.l2_cache.invalidate(key).await?;
        self.l3_cache.invalidate(key).await?;
        Ok(())
    }

    /// Clear every layer.
    pub async fn clear_all(&self) -> Result<(), EngineError> {
        self.l1_cache.clear().await?;
        self.l2_cache.clear().await?;
        self.l3_cache.clear().await?;
        Ok(())
    }

    /// Snapshot of aggregate statistics.
    pub async fn get_stats(&self) -> CacheStats {
        self.stats.read().await.clone()
    }

    /// Reset aggregate statistics to zero.
    pub async fn reset_stats(&self) {
        let mut stats = self.stats.write().await;
        *stats = CacheStats::default();
    }

    /// Preload common calculations into the cache (engine-specific hook).
    pub async fn preload_common_calculations(&self) -> Result<(), EngineError> {
        // Hook for engines to preload frequently-requested results.
        Ok(())
    }

    /// Tune cache parameters based on usage patterns.
    pub async fn optimize(&self) -> Result<(), EngineError> {
        // Hook for LRU tuning, TTL adjustment, memory compaction.
        Ok(())
    }
}
