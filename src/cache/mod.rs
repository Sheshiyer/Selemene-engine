use crate::models::{PanchangaRequest, PanchangaResult, EngineError};
use dashmap::DashMap;
use redis::AsyncCommands;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};
use std::time::{Duration, Instant};

pub mod l1_cache;
pub mod l2_cache;
pub mod l3_cache;

use l1_cache::L1Cache;
use l2_cache::L2Cache;
use l3_cache::L3Cache;

/// Cache key for storing results
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct CacheKey {
    pub date: String,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub precision: u8,
    pub backend: String,
}

impl CacheKey {
    pub fn from_request(request: &PanchangaRequest, backend: &str) -> Self {
        Self {
            date: request.date.clone(),
            latitude: request.latitude,
            longitude: request.longitude,
            precision: request.precision.map(|p| p as u8).unwrap_or(2),
            backend: backend.to_string(),
        }
    }
}

/// Cached calculation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedResult {
    pub result: PanchangaResult,
    pub created_at: Instant,
    pub accessed_at: Instant,
    pub access_count: u64,
}

/// Cache statistics
#[derive(Debug, Clone, Default)]
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

/// Multi-layer cache manager
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

    /// Get cached result from multi-layer cache
    pub async fn get(&self, key: &CacheKey) -> Result<Option<PanchangaResult>, EngineError> {
        let mut stats = self.stats.write().await;
        stats.total_requests += 1;
        drop(stats);

        // Try L1 cache first (in-memory)
        if let Some(result) = self.l1_cache.get(key).await? {
            let mut stats = self.stats.write().await;
            stats.l1_hits += 1;
            return Ok(Some(result));
        }

        // Try L2 cache (Redis)
        if let Some(result) = self.l2_cache.get(key).await? {
            // Populate L1 cache
            self.l1_cache.store(key, &result).await?;
            
            let mut stats = self.stats.write().await;
            stats.l2_hits += 1;
            return Ok(Some(result));
        }

        // Try L3 cache (precomputed)
        if let Some(result) = self.l3_cache.get(key).await? {
            // Populate higher caches
            self.l1_cache.store(key, &result).await?;
            self.l2_cache.store(key, &result).await?;
            
            let mut stats = self.stats.write().await;
            stats.l3_hits += 1;
            return Ok(Some(result));
        }

        let mut stats = self.stats.write().await;
        stats.cache_misses += 1;
        Ok(None)
    }

    /// Store result in cache
    pub async fn store(
        &self,
        key: &CacheKey,
        result: &PanchangaResult,
    ) -> Result<(), EngineError> {
        // Store in L1 and L2 caches
        self.l1_cache.store(key, result).await?;
        self.l2_cache.store(key, result).await?;
        
        Ok(())
    }

    /// Store result in L3 cache (precomputed)
    pub async fn store_precomputed(
        &self,
        key: &CacheKey,
        result: &PanchangaResult,
    ) -> Result<(), EngineError> {
        self.l3_cache.store(key, result).await?;
        Ok(())
    }

    /// Invalidate cache entry
    pub async fn invalidate(&self, key: &CacheKey) -> Result<(), EngineError> {
        self.l1_cache.invalidate(key).await?;
        self.l2_cache.invalidate(key).await?;
        self.l3_cache.invalidate(key).await?;
        Ok(())
    }

    /// Clear all caches
    pub async fn clear_all(&self) -> Result<(), EngineError> {
        self.l1_cache.clear().await?;
        self.l2_cache.clear().await?;
        self.l3_cache.clear().await?;
        Ok(())
    }

    /// Get cache statistics
    pub async fn get_stats(&self) -> CacheStats {
        self.stats.read().await.clone()
    }

    /// Reset cache statistics
    pub async fn reset_stats(&self) {
        let mut stats = self.stats.write().await;
        *stats = CacheStats::default();
    }

    /// Preload cache with common calculations
    pub async fn preload_common_calculations(&self) -> Result<(), EngineError> {
        // TODO: Implement preloading of common Panchanga calculations
        // This could include:
        // - Current date calculations
        // - Major festival dates
        // - Common coordinate locations
        
        Ok(())
    }

    /// Optimize cache based on usage patterns
    pub async fn optimize(&self) -> Result<(), EngineError> {
        // TODO: Implement cache optimization
        // This could include:
        // - LRU eviction policy tuning
        // - TTL adjustment based on access patterns
        // - Memory usage optimization
        
        Ok(())
    }
}
