use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Main engine configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineConfig {
    pub swiss_ephemeris_path: String,
    pub cache_size_l1: usize,
    pub cache_size_l2: usize,
    pub cache_size_l3: usize,
    pub max_concurrent_requests: usize,
    pub default_precision: u8,
    pub enable_validation: bool,
    pub enable_metrics: bool,
    pub api_rate_limit: u32,
    pub websocket_timeout: u64,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            swiss_ephemeris_path: "data/ephemeris".to_string(),
            cache_size_l1: 1000,
            cache_size_l2: 10000,
            cache_size_l3: 100000,
            max_concurrent_requests: 100,
            default_precision: 2,
            enable_validation: true,
            enable_metrics: true,
            api_rate_limit: 1000,
            websocket_timeout: 300,
        }
    }
}

/// Configuration manager
pub struct ConfigManager {
    config: Arc<RwLock<EngineConfig>>,
}

impl ConfigManager {
    pub fn new(config: EngineConfig) -> Self {
        Self {
            config: Arc::new(RwLock::new(config)),
        }
    }

    pub async fn get_config(&self) -> EngineConfig {
        self.config.read().await.clone()
    }

    pub async fn update_config(&self, new_config: EngineConfig) {
        *self.config.write().await = new_config;
    }
}
