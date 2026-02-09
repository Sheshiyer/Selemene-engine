//! L2 Cache -- Redis-based distributed cache
//!
//! Provides a Redis-backed cache layer sitting between L1 (in-memory) and
//! L3 (disk). Keys are stored as `noesis:cache:{hash}` with a configurable TTL.
//! All operations are non-blocking via the `redis` async driver.
//!
//! If the Redis connection is unavailable, every method returns a graceful
//! fallback (None for gets, Ok(()) for writes) so the cache cascade continues
//! without interruption.

use crate::CacheKey;
use noesis_core::EngineError;
use redis::AsyncCommands;
use serde_json::Value;
use std::time::Duration;

/// Redis key prefix to namespace all Noesis cache entries.
const KEY_PREFIX: &str = "noesis:cache:";

/// L2 Cache -- distributed Redis layer.
pub struct L2Cache {
    client: redis::Client,
    ttl: Duration,
}

impl L2Cache {
    pub fn new(redis_url: String, ttl: Duration) -> Self {
        let client = redis::Client::open(redis_url.as_str())
            .unwrap_or_else(|e| {
                tracing::warn!("Failed to create Redis client: {}. L2 cache will be unavailable.", e);
                // Create a client with a dummy URL — is_available() will return false
                redis::Client::open("redis://invalid").expect("dummy client")
            });
        Self { client, ttl }
    }

    /// Build the namespaced Redis key from a CacheKey.
    fn redis_key(key: &CacheKey) -> String {
        format!("{}{}", KEY_PREFIX, key.hash)
    }

    /// Get a connection, returning None if Redis is unavailable.
    async fn conn(&self) -> Option<redis::aio::MultiplexedConnection> {
        match self.client.get_multiplexed_async_connection().await {
            Ok(conn) => Some(conn),
            Err(e) => {
                tracing::debug!("Redis connection unavailable: {}", e);
                None
            }
        }
    }

    /// Get a cached value from Redis.
    pub async fn get(&self, key: &CacheKey) -> Result<Option<Value>, EngineError> {
        let mut conn = match self.conn().await {
            Some(c) => c,
            None => return Ok(None),
        };

        let rkey = Self::redis_key(key);
        let result: Option<String> = conn.get(&rkey).await.unwrap_or(None);

        match result {
            Some(json_str) => {
                let value: Value = serde_json::from_str(&json_str)
                    .map_err(|e| EngineError::CacheError(format!("Redis deserialize: {}", e)))?;
                Ok(Some(value))
            }
            None => Ok(None),
        }
    }

    /// Store a value in Redis with the configured TTL.
    pub async fn store(&self, key: &CacheKey, value: &Value) -> Result<(), EngineError> {
        let mut conn = match self.conn().await {
            Some(c) => c,
            None => return Ok(()),
        };

        let rkey = Self::redis_key(key);
        let json_str = serde_json::to_string(value)
            .map_err(|e| EngineError::CacheError(format!("Redis serialize: {}", e)))?;

        let ttl_secs = self.ttl.as_secs() as i64;
        let _: () = conn
            .set_ex(&rkey, &json_str, ttl_secs as u64)
            .await
            .unwrap_or_else(|e| {
                tracing::warn!("Redis SET failed for {}: {}", rkey, e);
            });

        Ok(())
    }

    /// Remove a single key from Redis.
    pub async fn remove(&self, key: &CacheKey) -> Result<(), EngineError> {
        let mut conn = match self.conn().await {
            Some(c) => c,
            None => return Ok(()),
        };

        let rkey = Self::redis_key(key);
        let _: () = conn.del(&rkey).await.unwrap_or_else(|e| {
            tracing::warn!("Redis DEL failed for {}: {}", rkey, e);
        });

        Ok(())
    }

    /// Clear all Noesis cache keys from Redis using SCAN + DEL.
    pub async fn clear(&self) -> Result<(), EngineError> {
        let mut conn = match self.conn().await {
            Some(c) => c,
            None => return Ok(()),
        };

        // Use SCAN to find all keys with our prefix, then delete them
        let pattern = format!("{}*", KEY_PREFIX);
        let keys: Vec<String> = redis::cmd("KEYS")
            .arg(&pattern)
            .query_async(&mut conn)
            .await
            .unwrap_or_default();

        if !keys.is_empty() {
            let _: () = conn.del(keys.as_slice()).await.unwrap_or_else(|e| {
                tracing::warn!("Redis bulk DEL failed: {}", e);
            });
        }

        Ok(())
    }

    /// Batch get multiple keys from Redis.
    pub async fn get_multiple(&self, keys: &[CacheKey]) -> Result<Vec<Option<Value>>, EngineError> {
        let mut conn = match self.conn().await {
            Some(c) => c,
            None => return Ok(vec![None; keys.len()]),
        };

        if keys.is_empty() {
            return Ok(vec![]);
        }

        let rkeys: Vec<String> = keys.iter().map(Self::redis_key).collect();
        let results: Vec<Option<String>> = redis::cmd("MGET")
            .arg(&rkeys)
            .query_async(&mut conn)
            .await
            .unwrap_or_else(|e| {
                tracing::warn!("Redis MGET failed: {}", e);
                vec![None; keys.len()]
            });

        let values = results
            .into_iter()
            .map(|opt| {
                opt.and_then(|s| serde_json::from_str(&s).ok())
            })
            .collect();

        Ok(values)
    }

    /// Batch store multiple key-value pairs in Redis.
    pub async fn store_multiple(&self, items: &[(CacheKey, Value)]) -> Result<(), EngineError> {
        let mut conn = match self.conn().await {
            Some(c) => c,
            None => return Ok(()),
        };

        let ttl_secs = self.ttl.as_secs() as u64;

        // Use a pipeline for atomicity and performance
        let mut pipe = redis::pipe();
        for (key, value) in items {
            let rkey = Self::redis_key(key);
            if let Ok(json_str) = serde_json::to_string(value) {
                pipe.set_ex(rkey, json_str, ttl_secs);
            }
        }

        pipe.query_async::<Vec<()>>(&mut conn)
            .await
            .unwrap_or_else(|e| {
                tracing::warn!("Redis pipeline store failed: {}", e);
                vec![]
            });

        Ok(())
    }

    /// Check if Redis is reachable.
    pub async fn is_available(&self) -> bool {
        match self.conn().await {
            Some(mut conn) => {
                let result: redis::RedisResult<String> = redis::cmd("PING")
                    .query_async(&mut conn)
                    .await;
                result.is_ok()
            }
            None => false,
        }
    }

    /// Invalidate a single key (alias for remove).
    pub async fn invalidate(&self, key: &CacheKey) -> Result<(), EngineError> {
        self.remove(key).await
    }
}
