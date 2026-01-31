//! L2 Cache -- Redis-based distributed cache (currently stubbed)
//!
//! Redis integration is temporarily disabled. All methods return `Ok(None)` or
//! `Ok(())` so the rest of the cache stack works without a running Redis instance.

use crate::CacheKey;
use noesis_core::EngineError;
use serde_json::Value;
use std::time::Duration;

/// L2 Cache -- distributed Redis layer.
pub struct L2Cache {
    #[allow(dead_code)]
    redis_url: String,
    #[allow(dead_code)]
    ttl: Duration,
}

impl L2Cache {
    pub fn new(redis_url: String, ttl: Duration) -> Self {
        Self { redis_url, ttl }
    }

    /// Get a cached value from Redis (temporarily disabled).
    pub async fn get(&self, _key: &CacheKey) -> Result<Option<Value>, EngineError> {
        // TODO: Re-enable Redis cache
        Ok(None)
    }

    /// Store a value in Redis (temporarily disabled).
    pub async fn store(&self, _key: &CacheKey, _value: &Value) -> Result<(), EngineError> {
        // TODO: Re-enable Redis cache
        Ok(())
    }

    /// Remove a single key from Redis (temporarily disabled).
    pub async fn remove(&self, _key: &CacheKey) -> Result<(), EngineError> {
        // TODO: Re-enable Redis cache
        Ok(())
    }

    /// Clear all keys managed by this cache (temporarily disabled).
    pub async fn clear(&self) -> Result<(), EngineError> {
        // TODO: Re-enable Redis cache
        Ok(())
    }

    /// Batch get (temporarily disabled).
    pub async fn get_multiple(&self, keys: &[CacheKey]) -> Result<Vec<Option<Value>>, EngineError> {
        // TODO: Re-enable Redis cache
        Ok(vec![None; keys.len()])
    }

    /// Batch store (temporarily disabled).
    pub async fn store_multiple(&self, _items: &[(CacheKey, Value)]) -> Result<(), EngineError> {
        // TODO: Re-enable Redis cache
        Ok(())
    }

    /// Connectivity check (temporarily disabled).
    pub async fn is_available(&self) -> bool {
        // TODO: Re-enable Redis cache
        false
    }

    /// Invalidate a single key (temporarily disabled).
    pub async fn invalidate(&self, _key: &CacheKey) -> Result<(), EngineError> {
        // TODO: Re-enable Redis cache
        Ok(())
    }
}
