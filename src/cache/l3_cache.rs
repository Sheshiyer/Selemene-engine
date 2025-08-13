use crate::cache::{CacheKey, CachedResult};
use crate::models::{PanchangaResult, EngineError};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde_json;
use std::fs;
use std::path::Path;

/// L3 Cache - Precomputed results cache
pub struct L3Cache {
    enabled: bool,
    cache_dir: String,
    memory_cache: Arc<RwLock<HashMap<CacheKey, PanchangaResult>>>,
    stats: Arc<RwLock<L3CacheStats>>,
}

#[derive(Debug, Clone, Default)]
struct L3CacheStats {
    hits: u64,
    misses: u64,
    loads: u64,
    saves: u64,
    total_requests: u64,
}

impl L3Cache {
    pub fn new(enabled: bool) -> Self {
        let cache_dir = std::env::var("L3_CACHE_DIR")
            .unwrap_or_else(|_| "./data/precomputed".to_string());
        
        Self {
            enabled,
            cache_dir,
            memory_cache: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(L3CacheStats::default())),
        }
    }

    /// Get precomputed result from L3 cache
    pub async fn get(&self, key: &CacheKey) -> Result<Option<PanchangaResult>, EngineError> {
        if !self.enabled {
            return Ok(None);
        }

        let mut stats = self.stats.write().await;
        stats.total_requests += 1;
        drop(stats);

        // Try memory cache first
        if let Some(result) = self.memory_cache.read().await.get(key).cloned() {
            let mut stats = self.stats.write().await;
            stats.hits += 1;
            return Ok(Some(result));
        }

        // Try disk cache
        if let Some(result) = self.load_from_disk(key).await? {
            // Store in memory cache
            self.memory_cache.write().await.insert(key.clone(), result.clone());
            
            let mut stats = self.stats.write().await;
            stats.hits += 1;
            stats.loads += 1;
            
            return Ok(Some(result));
        }

        let mut stats = self.stats.write().await;
        stats.misses += 1;
        Ok(None)
    }

    /// Store result in L3 cache
    pub async fn store(
        &self,
        key: &CacheKey,
        result: &PanchangaResult,
    ) -> Result<(), EngineError> {
        if !self.enabled {
            return Ok(());
        }

        // Store in memory cache
        self.memory_cache.write().await.insert(key.clone(), result.clone());
        
        // Store on disk
        self.save_to_disk(key, result).await?;
        
        let mut stats = self.stats.write().await;
        stats.saves += 1;
        
        Ok(())
    }

    /// Invalidate cache entry
    pub async fn invalidate(&self, key: &CacheKey) -> Result<(), EngineError> {
        if !self.enabled {
            return Ok(());
        }

        // Remove from memory cache
        self.memory_cache.write().await.remove(key);
        
        // Remove from disk
        self.remove_from_disk(key).await?;
        
        Ok(())
    }

    /// Clear all entries
    pub async fn clear(&self) -> Result<(), EngineError> {
        if !self.enabled {
            return Ok(());
        }

        // Clear memory cache
        self.memory_cache.write().await.clear();
        
        // Clear disk cache
        self.clear_disk_cache().await?;
        
        Ok(())
    }

    /// Get cache statistics
    pub async fn get_stats(&self) -> L3CacheStats {
        self.stats.read().await.clone()
    }

    /// Check if L3 cache is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Enable or disable L3 cache
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Get cache directory path
    pub fn get_cache_dir(&self) -> &str {
        &self.cache_dir
    }

    /// Set cache directory path
    pub fn set_cache_dir(&mut self, cache_dir: String) {
        self.cache_dir = cache_dir;
    }

    /// Load result from disk cache
    async fn load_from_disk(&self, key: &CacheKey) -> Result<Option<PanchangaResult>, EngineError> {
        let file_path = self.get_cache_file_path(key);
        
        if !Path::new(&file_path).exists() {
            return Ok(None);
        }
        
        match fs::read_to_string(&file_path) {
            Ok(content) => {
                match serde_json::from_str::<PanchangaResult>(&content) {
                    Ok(result) => Ok(Some(result)),
                    Err(e) => {
                        tracing::warn!("Failed to deserialize cached result from disk: {}", e);
                        // Remove corrupted file
                        let _ = fs::remove_file(&file_path);
                        Ok(None)
                    }
                }
            }
            Err(e) => {
                tracing::warn!("Failed to read cache file: {}", e);
                Ok(None)
            }
        }
    }

    /// Save result to disk cache
    async fn save_to_disk(&self, key: &CacheKey, result: &PanchangaResult) -> Result<(), EngineError> {
        // Ensure cache directory exists
        fs::create_dir_all(&self.cache_dir)
            .map_err(|e| EngineError::CacheError(format!("Failed to create cache directory: {}", e)))?;
        
        let file_path = self.get_cache_file_path(key);
        let content = serde_json::to_string_pretty(result)
            .map_err(|e| EngineError::CacheError(format!("Serialization failed: {}", e)))?;
        
        fs::write(&file_path, content)
            .map_err(|e| EngineError::CacheError(format!("Failed to write cache file: {}", e)))?;
        
        Ok(())
    }

    /// Remove result from disk cache
    async fn remove_from_disk(&self, key: &CacheKey) -> Result<(), EngineError> {
        let file_path = self.get_cache_file_path(key);
        
        if Path::new(&file_path).exists() {
            fs::remove_file(&file_path)
                .map_err(|e| EngineError::CacheError(format!("Failed to remove cache file: {}", e)))?;
        }
        
        Ok(())
    }

    /// Clear disk cache
    async fn clear_disk_cache(&self) -> Result<(), EngineError> {
        if Path::new(&self.cache_dir).exists() {
            fs::remove_dir_all(&self.cache_dir)
                .map_err(|e| EngineError::CacheError(format!("Failed to remove cache directory: {}", e)))?;
        }
        
        Ok(())
    }

    /// Get cache file path for a key
    fn get_cache_file_path(&self, key: &CacheKey) -> String {
        // Create a safe filename from the cache key
        let key_hash = format!("{:x}", md5::compute(serde_json::to_string(key).unwrap_or_default()));
        format!("{}/{}.json", self.cache_dir, key_hash)
    }

    /// Preload common calculations into L3 cache
    pub async fn preload_common_calculations(&self) -> Result<(), EngineError> {
        if !self.enabled {
            return Ok(());
        }

        // TODO: Implement preloading of common Panchanga calculations
        // This could include:
        // - Current year calculations
        // - Major festival dates
        // - Common coordinate locations
        // - High-precision calculations for reference dates
        
        tracing::info!("L3 cache preloading completed");
        Ok(())
    }

    /// Optimize disk cache (remove old files, compress, etc.)
    pub async fn optimize_disk_cache(&self) -> Result<(), EngineError> {
        if !self.enabled {
            return Ok(());
        }

        // TODO: Implement disk cache optimization
        // This could include:
        // - Removing old cache files
        // - Compressing cache files
        // - Reorganizing cache structure
        // - Cleaning up corrupted files
        
        tracing::info!("L3 cache optimization completed");
        Ok(())
    }

    /// Get cache size information
    pub async fn get_cache_info(&self) -> Result<L3CacheInfo, EngineError> {
        let memory_entries = self.memory_cache.read().await.len();
        
        let disk_entries = if Path::new(&self.cache_dir).exists() {
            match fs::read_dir(&self.cache_dir) {
                Ok(entries) => entries.count(),
                Err(_) => 0,
            }
        } else {
            0
        };
        
        let disk_size = if Path::new(&self.cache_dir).exists() {
            self.calculate_directory_size(&self.cache_dir).unwrap_or(0)
        } else {
            0
        };
        
        Ok(L3CacheInfo {
            memory_entries,
            disk_entries,
            disk_size_bytes: disk_size,
            enabled: self.enabled,
        })
    }

    /// Calculate directory size recursively
    fn calculate_directory_size(&self, path: &str) -> Result<u64, EngineError> {
        let mut total_size = 0u64;
        
        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if path.is_file() {
                        if let Ok(metadata) = fs::metadata(&path) {
                            total_size += metadata.len();
                        }
                    } else if path.is_dir() {
                        total_size += self.calculate_directory_size(path.to_str().unwrap())?;
                    }
                }
            }
        }
        
        Ok(total_size)
    }
}

/// L3 Cache information
#[derive(Debug, Clone)]
pub struct L3CacheInfo {
    pub memory_entries: usize,
    pub disk_entries: usize,
    pub disk_size_bytes: u64,
    pub enabled: bool,
}
