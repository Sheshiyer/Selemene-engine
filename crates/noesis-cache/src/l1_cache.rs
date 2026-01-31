//! L1 Cache -- In-memory cache with LRU eviction
//!
//! Stores `serde_json::Value` payloads in a `DashMap` for lock-free concurrent
//! reads with size-bounded eviction.

use crate::{CacheKey, CachedResult};
use dashmap::DashMap;
use noesis_core::EngineError;
use serde_json::Value;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Per-layer statistics.
#[derive(Debug, Clone, Default)]
pub(crate) struct L1CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub evictions: u64,
    pub total_requests: u64,
}

/// L1 Cache -- fast, in-memory, size-bounded.
pub struct L1Cache {
    cache: Arc<DashMap<CacheKey, CachedResult>>,
    max_size_bytes: usize,
    current_size_bytes: Arc<RwLock<usize>>,
    stats: Arc<RwLock<L1CacheStats>>,
}

impl L1Cache {
    pub fn new(max_size_mb: usize) -> Self {
        let max_size_bytes = max_size_mb * 1024 * 1024;
        Self {
            cache: Arc::new(DashMap::new()),
            max_size_bytes,
            current_size_bytes: Arc::new(RwLock::new(0)),
            stats: Arc::new(RwLock::new(L1CacheStats::default())),
        }
    }

    /// Retrieve a value from L1, updating access metadata on hit.
    pub async fn get(&self, key: &CacheKey) -> Result<Option<Value>, EngineError> {
        {
            let mut stats = self.stats.write().await;
            stats.total_requests += 1;
        }

        if let Some(entry) = self.cache.get(key) {
            let mut cached = entry.clone();
            cached.accessed_at = Instant::now();
            cached.access_count += 1;
            let value = cached.value.clone();
            // Update the entry in-place
            drop(entry);
            self.cache.insert(key.clone(), cached);

            let mut stats = self.stats.write().await;
            stats.hits += 1;
            Ok(Some(value))
        } else {
            let mut stats = self.stats.write().await;
            stats.misses += 1;
            Ok(None)
        }
    }

    /// Store a JSON value in L1, evicting LRU entries if necessary.
    pub async fn store(&self, key: &CacheKey, value: &Value) -> Result<(), EngineError> {
        let estimated_size = Self::estimate_value_size(value);

        if estimated_size > self.max_size_bytes {
            return Err(EngineError::CacheError(
                "Value too large for L1 cache".to_string(),
            ));
        }

        self.ensure_space(estimated_size).await?;

        let cached = CachedResult {
            value: value.clone(),
            created_at: Instant::now(),
            accessed_at: Instant::now(),
            access_count: 1,
        };

        self.cache.insert(key.clone(), cached);

        let mut current_size = self.current_size_bytes.write().await;
        *current_size += estimated_size;
        Ok(())
    }

    /// Remove a single entry.
    pub async fn invalidate(&self, key: &CacheKey) -> Result<(), EngineError> {
        if let Some((_, cached)) = self.cache.remove(key) {
            let estimated_size = Self::estimate_value_size(&cached.value);
            let mut current_size = self.current_size_bytes.write().await;
            *current_size = current_size.saturating_sub(estimated_size);
        }
        Ok(())
    }

    /// Drop all entries.
    pub async fn clear(&self) -> Result<(), EngineError> {
        self.cache.clear();
        let mut current_size = self.current_size_bytes.write().await;
        *current_size = 0;
        Ok(())
    }

    /// Current size snapshot.
    pub async fn get_current_size(&self) -> usize {
        *self.current_size_bytes.read().await
    }

    /// Configured maximum size.
    pub fn get_max_size(&self) -> usize {
        self.max_size_bytes
    }

    /// Number of entries.
    pub fn get_entry_count(&self) -> usize {
        self.cache.len()
    }

    #[allow(dead_code)]
    pub(crate) async fn get_stats(&self) -> L1CacheStats {
        self.stats.read().await.clone()
    }

    /// Evict expired entries older than `max_age`.
    pub async fn cleanup_expired(&self, max_age: Duration) -> Result<usize, EngineError> {
        let now = Instant::now();
        let mut to_remove = Vec::new();

        for entry in self.cache.iter() {
            if now.duration_since(entry.value().created_at) > max_age {
                to_remove.push(entry.key().clone());
            }
        }

        let mut cleaned = 0;
        for key in to_remove {
            if let Some((_, cached)) = self.cache.remove(&key) {
                let sz = Self::estimate_value_size(&cached.value);
                let mut current_size = self.current_size_bytes.write().await;
                *current_size = current_size.saturating_sub(sz);
                cleaned += 1;
            }
        }
        Ok(cleaned)
    }

    // -----------------------------------------------------------------------
    // Internal helpers
    // -----------------------------------------------------------------------

    /// Make room by evicting LRU entries until `required_bytes` fits.
    async fn ensure_space(&self, required_bytes: usize) -> Result<(), EngineError> {
        {
            let current_size = self.current_size_bytes.read().await;
            if *current_size + required_bytes <= self.max_size_bytes {
                return Ok(());
            }
        }

        // Collect, sort by LRU, evict.
        let mut entries: Vec<_> = self
            .cache
            .iter()
            .map(|e| (e.key().clone(), e.value().clone()))
            .collect();

        entries.sort_by(|a, b| {
            a.1.accessed_at
                .cmp(&b.1.accessed_at)
                .then(a.1.access_count.cmp(&b.1.access_count))
        });

        let mut evicted_bytes = 0usize;
        for (key, cached) in entries {
            if evicted_bytes >= required_bytes {
                break;
            }
            let sz = Self::estimate_value_size(&cached.value);
            self.cache.remove(&key);
            evicted_bytes += sz;

            let mut stats = self.stats.write().await;
            stats.evictions += 1;
        }

        let mut current_size = self.current_size_bytes.write().await;
        *current_size = current_size.saturating_sub(evicted_bytes);
        Ok(())
    }

    /// Rough byte-size estimate of a JSON value (serialised length).
    fn estimate_value_size(value: &Value) -> usize {
        // Fast approximation: serialise to string and measure length.
        // In production this could be replaced with a cheaper heuristic.
        serde_json::to_string(value).map(|s| s.len()).unwrap_or(256)
    }
}
