//! L3 Cache -- Precomputed results persisted to disk
//!
//! Stores JSON values as files keyed by MD5 hash.  A memory-backed HashMap
//! provides fast repeated lookups; disk is the authoritative store.

use crate::CacheKey;
use noesis_core::EngineError;
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;

// -----------------------------------------------------------------------
// Stats
// -----------------------------------------------------------------------

#[derive(Debug, Clone, Default)]
pub(crate) struct L3CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub loads: u64,
    pub saves: u64,
    pub total_requests: u64,
}

// -----------------------------------------------------------------------
// L3CacheInfo
// -----------------------------------------------------------------------

/// Diagnostic snapshot of L3 cache state.
#[derive(Debug, Clone)]
pub struct L3CacheInfo {
    pub memory_entries: usize,
    pub disk_entries: usize,
    pub disk_size_bytes: u64,
    pub enabled: bool,
}

// -----------------------------------------------------------------------
// L3Cache
// -----------------------------------------------------------------------

/// L3 Cache -- disk-backed precomputed results.
pub struct L3Cache {
    enabled: bool,
    cache_dir: String,
    memory_cache: Arc<RwLock<HashMap<CacheKey, Value>>>,
    stats: Arc<RwLock<L3CacheStats>>,
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

    // -- public API --------------------------------------------------------

    /// Retrieve a cached value (memory first, then disk).
    pub async fn get(&self, key: &CacheKey) -> Result<Option<Value>, EngineError> {
        if !self.enabled {
            return Ok(None);
        }

        {
            let mut stats = self.stats.write().await;
            stats.total_requests += 1;
        }

        // Memory
        if let Some(value) = self.memory_cache.read().await.get(key).cloned() {
            let mut stats = self.stats.write().await;
            stats.hits += 1;
            return Ok(Some(value));
        }

        // Disk
        if let Some(value) = self.load_from_disk(key).await? {
            self.memory_cache.write().await.insert(key.clone(), value.clone());
            let mut stats = self.stats.write().await;
            stats.hits += 1;
            stats.loads += 1;
            return Ok(Some(value));
        }

        let mut stats = self.stats.write().await;
        stats.misses += 1;
        Ok(None)
    }

    /// Persist a value to disk and cache in memory.
    pub async fn store(&self, key: &CacheKey, value: &Value) -> Result<(), EngineError> {
        if !self.enabled {
            return Ok(());
        }

        self.memory_cache.write().await.insert(key.clone(), value.clone());
        self.save_to_disk(key, value).await?;

        let mut stats = self.stats.write().await;
        stats.saves += 1;
        Ok(())
    }

    /// Remove from both memory and disk.
    pub async fn invalidate(&self, key: &CacheKey) -> Result<(), EngineError> {
        if !self.enabled {
            return Ok(());
        }
        self.memory_cache.write().await.remove(key);
        self.remove_from_disk(key).await?;
        Ok(())
    }

    /// Wipe everything.
    pub async fn clear(&self) -> Result<(), EngineError> {
        if !self.enabled {
            return Ok(());
        }
        self.memory_cache.write().await.clear();
        self.clear_disk_cache().await?;
        Ok(())
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub fn get_cache_dir(&self) -> &str {
        &self.cache_dir
    }

    pub fn set_cache_dir(&mut self, cache_dir: String) {
        self.cache_dir = cache_dir;
    }

    #[allow(dead_code)]
    pub(crate) async fn get_stats(&self) -> L3CacheStats {
        self.stats.read().await.clone()
    }

    /// Preload hook (no-op until engines wire up specific datasets).
    pub async fn preload_common_calculations(&self) -> Result<(), EngineError> {
        if !self.enabled {
            return Ok(());
        }
        tracing::info!("L3 cache preloading completed");
        Ok(())
    }

    /// Optimisation hook (no-op placeholder).
    pub async fn optimize_disk_cache(&self) -> Result<(), EngineError> {
        if !self.enabled {
            return Ok(());
        }
        tracing::info!("L3 cache optimization completed");
        Ok(())
    }

    /// Diagnostic snapshot.
    pub async fn get_cache_info(&self) -> Result<L3CacheInfo, EngineError> {
        let memory_entries = self.memory_cache.read().await.len();

        let disk_entries = if Path::new(&self.cache_dir).exists() {
            fs::read_dir(&self.cache_dir).map(|e| e.count()).unwrap_or(0)
        } else {
            0
        };

        let disk_size = if Path::new(&self.cache_dir).exists() {
            Self::calculate_directory_size(&self.cache_dir).unwrap_or(0)
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

    // -- internal ----------------------------------------------------------

    async fn load_from_disk(&self, key: &CacheKey) -> Result<Option<Value>, EngineError> {
        let file_path = self.cache_file_path(key);

        if !Path::new(&file_path).exists() {
            return Ok(None);
        }

        match fs::read_to_string(&file_path) {
            Ok(content) => match serde_json::from_str::<Value>(&content) {
                Ok(value) => Ok(Some(value)),
                Err(e) => {
                    tracing::warn!("Corrupted L3 cache file, removing: {}", e);
                    let _ = fs::remove_file(&file_path);
                    Ok(None)
                }
            },
            Err(e) => {
                tracing::warn!("Failed to read L3 cache file: {}", e);
                Ok(None)
            }
        }
    }

    async fn save_to_disk(&self, key: &CacheKey, value: &Value) -> Result<(), EngineError> {
        fs::create_dir_all(&self.cache_dir)
            .map_err(|e| EngineError::CacheError(format!("Failed to create cache directory: {}", e)))?;

        let file_path = self.cache_file_path(key);
        let content = serde_json::to_string_pretty(value)
            .map_err(|e| EngineError::CacheError(format!("Serialization failed: {}", e)))?;

        fs::write(&file_path, content)
            .map_err(|e| EngineError::CacheError(format!("Failed to write cache file: {}", e)))?;

        Ok(())
    }

    async fn remove_from_disk(&self, key: &CacheKey) -> Result<(), EngineError> {
        let file_path = self.cache_file_path(key);
        if Path::new(&file_path).exists() {
            fs::remove_file(&file_path)
                .map_err(|e| EngineError::CacheError(format!("Failed to remove cache file: {}", e)))?;
        }
        Ok(())
    }

    async fn clear_disk_cache(&self) -> Result<(), EngineError> {
        if Path::new(&self.cache_dir).exists() {
            fs::remove_dir_all(&self.cache_dir)
                .map_err(|e| EngineError::CacheError(format!("Failed to clear cache dir: {}", e)))?;
        }
        Ok(())
    }

    fn cache_file_path(&self, key: &CacheKey) -> String {
        format!("{}/{}.json", self.cache_dir, key.hash)
    }

    fn calculate_directory_size(path: &str) -> Result<u64, EngineError> {
        let mut total = 0u64;
        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_file() {
                    if let Ok(meta) = fs::metadata(&p) {
                        total += meta.len();
                    }
                } else if p.is_dir() {
                    total += Self::calculate_directory_size(p.to_str().unwrap_or(""))?;
                }
            }
        }
        Ok(total)
    }
}
