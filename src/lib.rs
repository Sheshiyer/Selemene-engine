//! Selemene Engine - High-performance astronomical calculation engine
//! 
//! This library provides a simple API for Panchanga and Vedic astrology calculations.

pub mod models;
pub mod engines;
pub mod api;
pub mod cache;
pub mod auth;
pub mod metrics;
pub mod time;
pub mod utils;
pub mod config;

// For now, we'll keep this minimal to get a working deployment
pub mod simple;

pub use simple::*;

use std::sync::Arc;
use crate::engines::CalculationOrchestrator;
use crate::cache::CacheManager;
use crate::config::EngineConfig;
use tokio::sync::RwLock;

/// Main Selemene Engine struct
pub struct SelemeneEngine {
    pub orchestrator: Arc<CalculationOrchestrator>,
    pub cache_manager: Arc<CacheManager>,
    pub config: Arc<RwLock<EngineConfig>>,
}

impl SelemeneEngine {
    pub fn new(
        orchestrator: Arc<CalculationOrchestrator>,
        cache_manager: Arc<CacheManager>,
        config: Arc<RwLock<EngineConfig>>,
    ) -> Self {
        Self {
            orchestrator,
            cache_manager,
            config,
        }
    }

    pub async fn calculate_panchanga(&self, request: crate::models::PanchangaRequest) -> Result<crate::models::PanchangaResult, crate::models::EngineError> {
        self.orchestrator.calculate_panchanga(request).await
    }

    pub async fn get_config(&self) -> tokio::sync::RwLockReadGuard<'_, EngineConfig> {
        self.config.read().await
    }
}
