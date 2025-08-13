use crate::cache::{CacheKey, CachedResult};
use crate::models::{PanchangaResult, EngineError};
use redis::{Client, AsyncCommands, ConnectionManager};
use serde_json;
use std::time::Duration;
use tokio::sync::OnceCell;

/// L2 Cache - Redis-based distributed cache
pub struct L2Cache {
    client: Client,
    connection_manager: OnceCell<ConnectionManager>,
    ttl: Duration,
}

impl L2Cache {
    pub fn new(redis_url: String, ttl: Duration) -> Self {
        let client = Client::open(redis_url).expect("Failed to create Redis client");
        
        Self {
            client,
            connection_manager: OnceCell::new(),
            ttl,
        }
    }

    /// Get cached result from L2 cache
    pub async fn get(&self, key: &CacheKey) -> Result<Option<PanchangaResult>, EngineError> {
        let conn = self.get_connection().await?;
        
        let key_str = self.serialize_key(key)?;
        
        match conn.get::<_, Option<Vec<u8>>>(&key_str).await {
            Ok(Some(data)) => {
                match serde_json::from_slice::<PanchangaResult>(&data) {
                    Ok(result) => Ok(Some(result)),
                    Err(e) => {
                        tracing::warn!("Failed to deserialize cached result: {}", e);
                        Ok(None)
                    }
                }
            }
            Ok(None) => Ok(None),
            Err(e) => {
                tracing::warn!("Redis get error: {}", e);
                Ok(None)
            }
        }
    }

    /// Store result in L2 cache
    pub async fn store(
        &self,
        key: &CacheKey,
        result: &PanchangaResult,
    ) -> Result<(), EngineError> {
        let conn = self.get_connection().await?;
        
        let key_str = self.serialize_key(key)?;
        let data = serde_json::to_vec(result)
            .map_err(|e| EngineError::CacheError(format!("Serialization failed: {}", e)))?;
        
        // Set with TTL
        let ttl_seconds = self.ttl.as_secs() as usize;
        
        let _: () = conn.set_ex(&key_str, data, ttl_seconds).await
            .map_err(|e| EngineError::CacheError(format!("Redis set error: {}", e)))?;
        
        Ok(())
    }

    /// Invalidate cache entry
    pub async fn invalidate(&self, key: &CacheKey) -> Result<(), EngineError> {
        let conn = self.get_connection().await?;
        let key_str = self.serialize_key(key)?;
        
        let _: () = conn.del(&key_str).await
            .map_err(|e| EngineError::CacheError(format!("Redis del error: {}", e)))?;
        
        Ok(())
    }

    /// Clear all entries (use with caution)
    pub async fn clear(&self) -> Result<(), EngineError> {
        let conn = self.get_connection().await?;
        
        let _: () = conn.flushdb().await
            .map_err(|e| EngineError::CacheError(format!("Redis flush error: {}", e)))?;
        
        Ok(())
    }

    /// Get cache statistics from Redis
    pub async fn get_stats(&self) -> Result<L2CacheStats, EngineError> {
        let conn = self.get_connection().await?;
        
        let info: String = conn.info("stats").await
            .map_err(|e| EngineError::CacheError(format!("Redis info error: {}", e)))?;
        
        let stats = self.parse_redis_info(&info);
        Ok(stats)
    }

    /// Ping Redis to check connectivity
    pub async fn ping(&self) -> Result<bool, EngineError> {
        let conn = self.get_connection().await?;
        
        match conn.ping().await {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    /// Get Redis connection
    async fn get_connection(&self) -> Result<ConnectionManager, EngineError> {
        self.connection_manager
            .get_or_try_init(|| async {
                ConnectionManager::new(self.client.clone())
                    .await
                    .map_err(|e| EngineError::CacheError(format!("Connection failed: {}", e)))
            })
            .await
            .cloned()
    }

    /// Serialize cache key to string
    fn serialize_key(&self, key: &CacheKey) -> Result<String, EngineError> {
        serde_json::to_string(key)
            .map_err(|e| EngineError::CacheError(format!("Key serialization failed: {}", e)))
    }

    /// Parse Redis INFO output for statistics
    fn parse_redis_info(&self, info: &str) -> L2CacheStats {
        let mut stats = L2CacheStats::default();
        
        for line in info.lines() {
            if let Some((key, value)) = line.split_once(':') {
                match key {
                    "keyspace_hits" => {
                        if let Ok(hits) = value.parse::<u64>() {
                            stats.keyspace_hits = hits;
                        }
                    }
                    "keyspace_misses" => {
                        if let Ok(misses) = value.parse::<u64>() {
                            stats.keyspace_misses = misses;
                        }
                    }
                    "total_commands_processed" => {
                        if let Ok(commands) = value.parse::<u64>() {
                            stats.total_commands = commands;
                        }
                    }
                    "total_connections_received" => {
                        if let Ok(connections) = value.parse::<u64>() {
                            stats.total_connections = connections;
                        }
                    }
                    "used_memory_human" => {
                        stats.used_memory = value.to_string();
                    }
                    _ => {}
                }
            }
        }
        
        stats
    }

    /// Set custom TTL for specific keys
    pub async fn set_ttl(&mut self, ttl: Duration) {
        self.ttl = ttl;
    }

    /// Get current TTL setting
    pub fn get_ttl(&self) -> Duration {
        self.ttl
    }

    /// Batch get multiple keys
    pub async fn batch_get(&self, keys: &[CacheKey]) -> Result<Vec<Option<PanchangaResult>>, EngineError> {
        let conn = self.get_connection().await?;
        
        let key_strings: Vec<String> = keys
            .iter()
            .map(|k| self.serialize_key(k))
            .collect::<Result<Vec<_>, _>>()?;
        
        let results: Vec<Option<Vec<u8>>> = conn.mget(&key_strings).await
            .map_err(|e| EngineError::CacheError(format!("Redis mget error: {}", e)))?;
        
        let mut panchanga_results = Vec::new();
        
        for result in results {
            match result {
                Some(data) => {
                    match serde_json::from_slice::<PanchangaResult>(&data) {
                        Ok(result) => panchanga_results.push(Some(result)),
                        Err(_) => panchanga_results.push(None),
                    }
                }
                None => panchanga_results.push(None),
            }
        }
        
        Ok(panchanga_results)
    }

    /// Batch store multiple key-value pairs
    pub async fn batch_store(
        &self,
        entries: &[(CacheKey, PanchangaResult)],
    ) -> Result<(), EngineError> {
        let conn = self.get_connection().await?;
        let ttl_seconds = self.ttl.as_secs() as usize;
        
        for (key, result) in entries {
            let key_str = self.serialize_key(key)?;
            let data = serde_json::to_vec(result)
                .map_err(|e| EngineError::CacheError(format!("Serialization failed: {}", e)))?;
            
            let _: () = conn.set_ex(&key_str, data, ttl_seconds).await
                .map_err(|e| EngineError::CacheError(format!("Redis set error: {}", e)))?;
        }
        
        Ok(())
    }
}

/// L2 Cache statistics from Redis
#[derive(Debug, Clone, Default)]
pub struct L2CacheStats {
    pub keyspace_hits: u64,
    pub keyspace_misses: u64,
    pub total_commands: u64,
    pub total_connections: u64,
    pub used_memory: String,
}

impl L2CacheStats {
    pub fn hit_rate(&self) -> f64 {
        let total = self.keyspace_hits + self.keyspace_misses;
        if total == 0 {
            0.0
        } else {
            self.keyspace_hits as f64 / total as f64
        }
    }
}
