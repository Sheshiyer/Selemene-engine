use crate::config::EngineConfig;
use crate::models::{PanchangaRequest, EngineError};
use super::{Backend, BackendRoutingStrategy};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Hybrid backend system that intelligently routes calculations
pub struct HybridBackend {
    config: Arc<RwLock<EngineConfig>>,
    routing_strategy: BackendRoutingStrategy,
}

impl HybridBackend {
    pub fn new(config: Arc<RwLock<EngineConfig>>) -> Self {
        let routing_strategy = BackendRoutingStrategy::AlwaysNative; // Default routing strategy

        Self {
            config,
            routing_strategy,
        }
    }

    /// Select the appropriate backend for a calculation request
    pub async fn select_backend(
        &self,
        request: &PanchangaRequest,
    ) -> Result<Backend, EngineError> {
        match self.routing_strategy {
            BackendRoutingStrategy::AlwaysNative => Ok(Backend::Native),
            BackendRoutingStrategy::AlwaysSwiss => Ok(Backend::Swiss),
            BackendRoutingStrategy::Intelligent => self.select_intelligently(request).await,
            BackendRoutingStrategy::Validated => Ok(Backend::Validated),
            BackendRoutingStrategy::PerformanceOptimized => self.select_for_performance(request).await,
        }
    }

    /// Intelligent backend selection based on request characteristics
    async fn select_intelligently(
        &self,
        _request: &PanchangaRequest,
    ) -> Result<Backend, EngineError> {
        // TODO: Implement intelligent selection logic
        // For now, default to native for performance
        Ok(Backend::Native)
    }

    /// Performance-optimized backend selection
    async fn select_for_performance(
        &self,
        _request: &PanchangaRequest,
    ) -> Result<Backend, EngineError> {
        // TODO: Implement performance-based selection
        // Consider factors like:
        // - Request complexity
        // - Current system load
        // - Historical performance data
        Ok(Backend::Native)
    }

    /// Update routing strategy
    pub async fn update_routing_strategy(&mut self, strategy: BackendRoutingStrategy) {
        self.routing_strategy = strategy;
        
        // Update configuration
        {
            let _config = self.config.write().await;
            // Note: config doesn't have calculation field, so we just update our internal state
        }
    }

    /// Get current routing strategy
    pub fn get_routing_strategy(&self) -> BackendRoutingStrategy {
        self.routing_strategy
    }
}
