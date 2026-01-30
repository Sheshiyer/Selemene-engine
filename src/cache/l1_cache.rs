use crate::cache::{CacheKey, CachedResult};
use crate::models::{PanchangaResult, EngineError};
use dashmap::DashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// L1 Cache - In-memory cache with LRU eviction
pub struct L1Cache {
    cache: Arc<DashMap<CacheKey, CachedResult>>,
    max_size_bytes: usize,
    current_size_bytes: Arc<RwLock<usize>>,
    stats: Arc<RwLock<L1CacheStats>>,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct L1CacheStats {
    hits: u64,
    misses: u64,
    evictions: u64,
    total_requests: u64,
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

    /// Get cached result from L1 cache
    pub async fn get(&self, key: &CacheKey) -> Result<Option<PanchangaResult>, EngineError> {
        let mut stats = self.stats.write().await;
        stats.total_requests += 1;
        drop(stats);

        if let Some(entry) = self.cache.get(key) {
            // Update access statistics
            let mut cached_result = entry.clone();
            cached_result.accessed_at = Instant::now();
            cached_result.access_count += 1;
            
            // Update the cache entry
            self.cache.insert(key.clone(), cached_result.clone());
            
            let mut stats = self.stats.write().await;
            stats.hits += 1;
            
            Ok(Some(cached_result.result))
        } else {
            let mut stats = self.stats.write().await;
            stats.misses += 1;
            Ok(None)
        }
    }

    /// Store result in L1 cache
    pub async fn store(
        &self,
        key: &CacheKey,
        result: &PanchangaResult,
    ) -> Result<(), EngineError> {
        let estimated_size = self.estimate_result_size(result);
        
        // Check if we need to evict entries to make space
        if estimated_size > self.max_size_bytes {
            return Err(EngineError::CacheError(
                "Result too large for L1 cache".to_string()
            ));
        }
        
        // Ensure we have enough space
        self.ensure_space(estimated_size).await?;
        
        // Create cached result
        let cached_result = CachedResult {
            result: result.clone(),
            created_at: Instant::now(),
            accessed_at: Instant::now(),
            access_count: 1,
        };
        
        // Store in cache
        self.cache.insert(key.clone(), cached_result);
        
        // Update size tracking
        let mut current_size = self.current_size_bytes.write().await;
        *current_size += estimated_size;
        
        Ok(())
    }

    /// Invalidate cache entry
    pub async fn invalidate(&self, key: &CacheKey) -> Result<(), EngineError> {
        if let Some((_, cached_result)) = self.cache.remove(key) {
            let estimated_size = self.estimate_result_size(&cached_result.result);
            let mut current_size = self.current_size_bytes.write().await;
            *current_size = current_size.saturating_sub(estimated_size);
        }
        Ok(())
    }

    /// Clear all entries
    pub async fn clear(&self) -> Result<(), EngineError> {
        self.cache.clear();
        let mut current_size = self.current_size_bytes.write().await;
        *current_size = 0;
        Ok(())
    }

    /// Get cache statistics
    #[allow(dead_code)]
    pub(crate) async fn get_stats(&self) -> L1CacheStats {
        self.stats.read().await.clone()
    }

    /// Get current cache size in bytes
    pub async fn get_current_size(&self) -> usize {
        *self.current_size_bytes.read().await
    }

    /// Get maximum cache size in bytes
    pub fn get_max_size(&self) -> usize {
        self.max_size_bytes
    }

    /// Get cache entry count
    pub fn get_entry_count(&self) -> usize {
        self.cache.len()
    }

    /// Ensure enough space is available by evicting least recently used entries
    async fn ensure_space(&self, required_bytes: usize) -> Result<(), EngineError> {
        let current_size = self.current_size_bytes.read().await;
        
        if *current_size + required_bytes <= self.max_size_bytes {
            return Ok(());
        }
        
        drop(current_size);
        
        // Need to evict entries
        let mut evicted_bytes = 0;
        let target_eviction = required_bytes;
        
        // Sort entries by access time and count (LRU)
        let mut entries: Vec<_> = self.cache
            .iter()
            .map(|entry| {
                let key = entry.key().clone();
                let value = entry.value().clone();
                (key, value)
            })
            .collect();
        
        // Sort by access time (oldest first) and access count (least accessed first)
        entries.sort_by(|a, b| {
            a.1.accessed_at.cmp(&b.1.accessed_at)
                .then(a.1.access_count.cmp(&b.1.access_count))
        });
        
        // Evict entries until we have enough space
        for (key, cached_result) in entries {
            if evicted_bytes >= target_eviction {
                break;
            }
            
            let entry_size = self.estimate_result_size(&cached_result.result);
            self.cache.remove(&key);
            evicted_bytes += entry_size;
            
            let mut stats = self.stats.write().await;
            stats.evictions += 1;
        }
        
        // Update size tracking
        let mut current_size = self.current_size_bytes.write().await;
        *current_size = current_size.saturating_sub(evicted_bytes);
        
        Ok(())
    }

    /// Estimate the size of a result in bytes
    fn estimate_result_size(&self, result: &PanchangaResult) -> usize {
        // Rough estimation based on data structure
        let base_size = std::mem::size_of::<PanchangaResult>();
        let string_size = result.date.len() + result.backend.len();
        let option_size = 8; // Size of Option<f64>
        
        base_size + string_size + (option_size * 5) // 5 optional fields
    }

    /// Clean up expired entries (optional, for TTL-based eviction)
    pub async fn cleanup_expired(&self, max_age: Duration) -> Result<usize, EngineError> {
        let now = Instant::now();
        let mut cleaned_count = 0;
        
        let mut to_remove = Vec::new();
        
        for entry in self.cache.iter() {
            if now.duration_since(entry.value().created_at) > max_age {
                to_remove.push(entry.key().clone());
            }
        }
        
        for key in to_remove {
            if let Some((_, cached_result)) = self.cache.remove(&key) {
                let estimated_size = self.estimate_result_size(&cached_result.result);
                let mut current_size = self.current_size_bytes.write().await;
                *current_size = current_size.saturating_sub(estimated_size);
                cleaned_count += 1;
            }
        }
        
        Ok(cleaned_count)
    }
}
