//! Workflow Caching — TTL-based caching for workflow results
//!
//! Provides caching strategies appropriate for different workflow types:
//! - Natal workflows: 24h TTL (birth data rarely changes)
//! - Temporal workflows: 1h TTL (time-sensitive calculations)
//! - Archetypal workflows: 15min TTL (question-specific)
//! - Full spectrum: 1h TTL

use noesis_core::{EngineError, WorkflowResult};
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, info};

/// TTL settings for different workflow types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkflowTtl {
    /// Natal/birth-based workflows - 24 hours
    Natal,
    /// Temporal/time-based workflows - 1 hour
    Temporal,
    /// Archetypal/question-based workflows - 15 minutes
    Archetypal,
    /// Full spectrum - 1 hour
    FullSpectrum,
    /// Custom TTL
    Custom(Duration),
}

impl WorkflowTtl {
    /// Get the duration for this TTL type
    pub fn duration(&self) -> Duration {
        match self {
            Self::Natal => Duration::from_secs(24 * 60 * 60),      // 24 hours
            Self::Temporal => Duration::from_secs(60 * 60),        // 1 hour
            Self::Archetypal => Duration::from_secs(15 * 60),      // 15 minutes
            Self::FullSpectrum => Duration::from_secs(60 * 60),    // 1 hour
            Self::Custom(d) => *d,
        }
    }

    /// Get TTL for a workflow by ID
    pub fn for_workflow(workflow_id: &str) -> Self {
        match workflow_id {
            "birth-blueprint" => Self::Natal,
            "daily-practice" => Self::Temporal,
            "decision-support" => Self::Archetypal,
            "self-inquiry" => Self::Natal,
            "creative-expression" => Self::Archetypal,
            "full-spectrum" => Self::FullSpectrum,
            _ => Self::Temporal, // Default
        }
    }
}

/// Cache key for workflow results
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WorkflowCacheKey {
    /// Workflow identifier
    pub workflow_id: String,
    /// Hash of relevant input parameters
    pub input_hash: u64,
    /// Concatenated engine versions (for cache invalidation on upgrades)
    pub engine_versions: String,
}

impl WorkflowCacheKey {
    /// Create a new cache key
    pub fn new(workflow_id: &str, input_hash: u64, engine_versions: &str) -> Self {
        Self {
            workflow_id: workflow_id.to_string(),
            input_hash,
            engine_versions: engine_versions.to_string(),
        }
    }

    /// Create a cache key from input data
    pub fn from_input(
        workflow_id: &str,
        birth_date: Option<&str>,
        latitude: Option<f64>,
        longitude: Option<f64>,
        question: Option<&str>,
        engine_versions: &str,
    ) -> Self {
        let mut hasher = DefaultHasher::new();

        // Hash relevant input fields
        workflow_id.hash(&mut hasher);
        if let Some(date) = birth_date {
            date.hash(&mut hasher);
        }
        if let Some(lat) = latitude {
            lat.to_bits().hash(&mut hasher);
        }
        if let Some(lng) = longitude {
            lng.to_bits().hash(&mut hasher);
        }
        if let Some(q) = question {
            q.hash(&mut hasher);
        }

        let input_hash = hasher.finish();

        Self::new(workflow_id, input_hash, engine_versions)
    }

    /// Generate a string key for storage
    pub fn to_string_key(&self) -> String {
        format!(
            "workflow:{}:{}:{}",
            self.workflow_id,
            self.input_hash,
            self.engine_versions
        )
    }
}

/// Cached workflow entry with metadata
#[derive(Debug, Clone)]
struct CachedWorkflow {
    /// The cached result
    result: WorkflowResult,
    /// When this entry was created
    created_at: Instant,
    /// TTL for this entry
    ttl: Duration,
    /// Access count for statistics
    access_count: u64,
}

impl CachedWorkflow {
    fn new(result: WorkflowResult, ttl: Duration) -> Self {
        Self {
            result,
            created_at: Instant::now(),
            ttl,
            access_count: 0,
        }
    }

    fn is_expired(&self) -> bool {
        self.created_at.elapsed() > self.ttl
    }
}

/// Statistics for cache performance
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WorkflowCacheStats {
    pub hits: u64,
    pub misses: u64,
    pub invalidations: u64,
    pub entries: usize,
}

impl WorkflowCacheStats {
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            self.hits as f64 / total as f64
        }
    }
}

/// Workflow-level cache manager
pub struct WorkflowCache {
    /// In-memory cache storage
    cache: Arc<RwLock<HashMap<String, CachedWorkflow>>>,
    /// Engine to workflow mapping for invalidation
    engine_workflows: Arc<RwLock<HashMap<String, Vec<String>>>>,
    /// Statistics
    stats: Arc<RwLock<WorkflowCacheStats>>,
    /// Maximum cache entries
    max_entries: usize,
}

impl Default for WorkflowCache {
    fn default() -> Self {
        Self::new(1000)
    }
}

impl WorkflowCache {
    /// Create a new workflow cache
    pub fn new(max_entries: usize) -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            engine_workflows: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(WorkflowCacheStats::default())),
            max_entries,
        }
    }

    /// Get a cached workflow result
    pub async fn get(&self, key: &WorkflowCacheKey) -> Option<WorkflowResult> {
        let string_key = key.to_string_key();

        let mut cache = self.cache.write().await;
        let mut stats = self.stats.write().await;

        if let Some(entry) = cache.get_mut(&string_key) {
            if entry.is_expired() {
                debug!(key = %string_key, "Cache entry expired");
                cache.remove(&string_key);
                stats.misses += 1;
                return None;
            }

            entry.access_count += 1;
            stats.hits += 1;
            debug!(key = %string_key, "Cache hit");
            return Some(entry.result.clone());
        }

        stats.misses += 1;
        debug!(key = %string_key, "Cache miss");
        None
    }

    /// Store a workflow result in the cache
    pub async fn set(&self, key: WorkflowCacheKey, result: WorkflowResult, ttl: Duration) {
        let string_key = key.to_string_key();

        let mut cache = self.cache.write().await;

        // Evict oldest entries if at capacity
        if cache.len() >= self.max_entries {
            self.evict_oldest(&mut cache);
        }

        cache.insert(string_key.clone(), CachedWorkflow::new(result, ttl));

        // Track which engines are used in this workflow
        let mut engine_map = self.engine_workflows.write().await;
        for engine_id in &key.workflow_id.split('-').collect::<Vec<_>>() {
            engine_map
                .entry(engine_id.to_string())
                .or_default()
                .push(string_key.clone());
        }

        info!(key = %string_key, ttl_secs = ttl.as_secs(), "Cached workflow result");
    }

    /// Store with automatic TTL based on workflow type
    pub async fn set_auto_ttl(&self, key: WorkflowCacheKey, result: WorkflowResult) {
        let ttl = WorkflowTtl::for_workflow(&key.workflow_id);
        self.set(key, result, ttl.duration()).await;
    }

    /// Invalidate all entries for a specific workflow
    pub async fn invalidate_workflow(&self, workflow_id: &str) {
        let mut cache = self.cache.write().await;
        let mut stats = self.stats.write().await;

        let keys_to_remove: Vec<String> = cache
            .keys()
            .filter(|k| k.starts_with(&format!("workflow:{}:", workflow_id)))
            .cloned()
            .collect();

        for key in &keys_to_remove {
            cache.remove(key);
            stats.invalidations += 1;
        }

        info!(
            workflow_id = workflow_id,
            invalidated = keys_to_remove.len(),
            "Invalidated workflow cache entries"
        );
    }

    /// Invalidate all workflows using a specific engine
    pub async fn invalidate_engine(&self, engine_id: &str) {
        let engine_map = self.engine_workflows.read().await;
        let workflow_keys = engine_map.get(engine_id).cloned().unwrap_or_default();
        drop(engine_map);

        if workflow_keys.is_empty() {
            return;
        }

        let mut cache = self.cache.write().await;
        let mut stats = self.stats.write().await;

        for key in &workflow_keys {
            if cache.remove(key).is_some() {
                stats.invalidations += 1;
            }
        }

        info!(
            engine_id = engine_id,
            invalidated = workflow_keys.len(),
            "Invalidated cache entries for engine"
        );
    }

    /// Clear all cached entries
    pub async fn clear(&self) {
        let mut cache = self.cache.write().await;
        let mut engine_map = self.engine_workflows.write().await;
        let mut stats = self.stats.write().await;

        let count = cache.len();
        cache.clear();
        engine_map.clear();
        stats.invalidations += count as u64;

        info!(cleared = count, "Cleared workflow cache");
    }

    /// Get cache statistics
    pub async fn get_stats(&self) -> WorkflowCacheStats {
        let cache = self.cache.read().await;
        let mut stats = self.stats.read().await.clone();
        stats.entries = cache.len();
        stats
    }

    /// Remove expired entries
    pub async fn cleanup_expired(&self) {
        let mut cache = self.cache.write().await;
        let initial_count = cache.len();

        cache.retain(|_, entry| !entry.is_expired());

        let removed = initial_count - cache.len();
        if removed > 0 {
            info!(removed = removed, "Cleaned up expired cache entries");
        }
    }

    /// Evict oldest entries to make room
    fn evict_oldest(&self, cache: &mut HashMap<String, CachedWorkflow>) {
        // Find entries to evict (oldest 10%)
        let evict_count = (self.max_entries / 10).max(1);

        let mut entries: Vec<_> = cache
            .iter()
            .map(|(k, v)| (k.clone(), v.created_at))
            .collect();

        entries.sort_by_key(|(_, created)| *created);

        for (key, _) in entries.into_iter().take(evict_count) {
            cache.remove(&key);
        }
    }

    /// Health check for readiness probe
    pub async fn health_check(&self) -> Result<bool, EngineError> {
        // Simple check that the cache is accessible
        let _cache = self.cache.read().await;
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use std::collections::HashMap;

    fn mock_result(workflow_id: &str) -> WorkflowResult {
        WorkflowResult {
            workflow_id: workflow_id.to_string(),
            engine_outputs: HashMap::new(),
            synthesis: None,
            total_time_ms: 100.0,
            timestamp: Utc::now(),
        }
    }

    #[tokio::test]
    async fn test_cache_set_and_get() {
        let cache = WorkflowCache::new(100);
        let key = WorkflowCacheKey::new("test-workflow", 12345, "v1");
        let result = mock_result("test-workflow");

        cache.set(key.clone(), result.clone(), Duration::from_secs(60)).await;

        let cached = cache.get(&key).await;
        assert!(cached.is_some());
        assert_eq!(cached.unwrap().workflow_id, "test-workflow");
    }

    #[tokio::test]
    async fn test_cache_expiration() {
        let cache = WorkflowCache::new(100);
        let key = WorkflowCacheKey::new("test-workflow", 12345, "v1");
        let result = mock_result("test-workflow");

        // Set with very short TTL
        cache.set(key.clone(), result, Duration::from_millis(1)).await;

        // Wait for expiration
        tokio::time::sleep(Duration::from_millis(10)).await;

        let cached = cache.get(&key).await;
        assert!(cached.is_none());
    }

    #[tokio::test]
    async fn test_workflow_invalidation() {
        let cache = WorkflowCache::new(100);

        let key1 = WorkflowCacheKey::new("birth-blueprint", 111, "v1");
        let key2 = WorkflowCacheKey::new("birth-blueprint", 222, "v1");
        let key3 = WorkflowCacheKey::new("daily-practice", 333, "v1");

        cache.set(key1.clone(), mock_result("birth-blueprint"), Duration::from_secs(60)).await;
        cache.set(key2.clone(), mock_result("birth-blueprint"), Duration::from_secs(60)).await;
        cache.set(key3.clone(), mock_result("daily-practice"), Duration::from_secs(60)).await;

        cache.invalidate_workflow("birth-blueprint").await;

        assert!(cache.get(&key1).await.is_none());
        assert!(cache.get(&key2).await.is_none());
        assert!(cache.get(&key3).await.is_some());
    }

    #[tokio::test]
    async fn test_cache_stats() {
        let cache = WorkflowCache::new(100);
        let key = WorkflowCacheKey::new("test-workflow", 12345, "v1");

        // Miss
        cache.get(&key).await;

        // Set and hit
        cache.set(key.clone(), mock_result("test-workflow"), Duration::from_secs(60)).await;
        cache.get(&key).await;
        cache.get(&key).await;

        let stats = cache.get_stats().await;
        assert_eq!(stats.hits, 2);
        assert_eq!(stats.misses, 1);
        assert!((stats.hit_rate() - 0.666).abs() < 0.01);
    }

    #[test]
    fn test_cache_key_from_input() {
        let key1 = WorkflowCacheKey::from_input(
            "birth-blueprint",
            Some("1990-01-01"),
            Some(12.97),
            Some(77.59),
            None,
            "v1",
        );

        let key2 = WorkflowCacheKey::from_input(
            "birth-blueprint",
            Some("1990-01-01"),
            Some(12.97),
            Some(77.59),
            None,
            "v1",
        );

        // Same inputs should produce same hash
        assert_eq!(key1.input_hash, key2.input_hash);

        // Different question should produce different hash
        let key3 = WorkflowCacheKey::from_input(
            "birth-blueprint",
            Some("1990-01-01"),
            Some(12.97),
            Some(77.59),
            Some("What should I do?"),
            "v1",
        );

        assert_ne!(key1.input_hash, key3.input_hash);
    }

    #[test]
    fn test_workflow_ttl() {
        assert_eq!(WorkflowTtl::Natal.duration(), Duration::from_secs(86400));
        assert_eq!(WorkflowTtl::Temporal.duration(), Duration::from_secs(3600));
        assert_eq!(WorkflowTtl::Archetypal.duration(), Duration::from_secs(900));

        assert_eq!(WorkflowTtl::for_workflow("birth-blueprint"), WorkflowTtl::Natal);
        assert_eq!(WorkflowTtl::for_workflow("daily-practice"), WorkflowTtl::Temporal);
        assert_eq!(WorkflowTtl::for_workflow("decision-support"), WorkflowTtl::Archetypal);
    }
}
