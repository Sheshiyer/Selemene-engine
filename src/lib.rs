//! Selemene Engine - High-performance astronomical calculation engine
//! 
//! This library provides a hybrid backend system combining Swiss Ephemeris
//! reliability with native VSOP87/ELP-2000 calculation engines for
//! Panchanga and Vedic astrology calculations.

pub mod api;
pub mod engines;
pub mod cache;
pub mod config;
pub mod models;
pub mod utils;
pub mod metrics;
pub mod auth;

use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};

/// Main engine configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineConfig {
    pub calculation: CalculationConfig,
    pub cache: CacheConfig,
    pub engines: EngineBackendConfig,
    pub server: ServerConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalculationConfig {
    pub default_backend: BackendRoutingStrategy,
    pub cross_validation_rate: f64,
    pub max_concurrent: usize,
    pub timeout_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    pub redis_url: String,
    pub size_mb: usize,
    pub ttl_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineBackendConfig {
    pub swiss_ephemeris: SwissEphemerisConfig,
    pub native_solar: NativeEngineConfig,
    pub native_lunar: NativeEngineConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwissEphemerisConfig {
    pub enabled: bool,
    pub data_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NativeEngineConfig {
    pub enabled: bool,
    pub precision: PrecisionLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub workers: usize,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum BackendRoutingStrategy {
    AlwaysNative,
    AlwaysSwiss,
    Intelligent,
    Validated,
    PerformanceOptimized,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum PrecisionLevel {
    Standard,
    High,
    Extreme,
}

/// Main Selemene Engine struct
pub struct SelemeneEngine {
    pub config: Arc<RwLock<EngineConfig>>,
    // Will be implemented in subsequent modules
}

impl SelemeneEngine {
    pub fn new(config: EngineConfig) -> Self {
        Self {
            config: Arc::new(RwLock::new(config)),
        }
    }
    
    pub async fn get_config(&self) -> EngineConfig {
        self.config.read().await.clone()
    }
}

/// Re-export main types for convenience
pub use engines::*;
pub use models::*;
pub use cache::*;
pub use config::*;
