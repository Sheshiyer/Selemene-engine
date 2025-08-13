pub mod calculation_orchestrator;
pub mod hybrid_backend;
pub mod native_solar;
pub mod native_lunar;
pub mod swiss_ephemeris;
pub mod validation;

pub use calculation_orchestrator::CalculationOrchestrator;
pub use hybrid_backend::HybridBackend;
pub use native_solar::NativeSolarEngine;
pub use native_lunar::NativeLunarEngine;
pub use swiss_ephemeris::SwissEphemerisEngine;
pub use validation::ValidationEngine;

use crate::models::{PanchangaRequest, PanchangaResult, EngineError};
use crate::EngineConfig;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Main calculation orchestrator that coordinates all engines
pub struct CalculationOrchestrator {
    hybrid_backend: HybridBackend,
    config: Arc<RwLock<EngineConfig>>,
}

impl CalculationOrchestrator {
    pub fn new(config: Arc<RwLock<EngineConfig>>) -> Self {
        Self {
            hybrid_backend: HybridBackend::new(config.clone()),
            config,
        }
    }

    pub async fn calculate_panchanga(
        &self,
        request: PanchangaRequest,
    ) -> Result<PanchangaResult, EngineError> {
        // 1. Request validation and preprocessing
        let validated_request = self.validate_request(request)?;
        
        // 2. Backend selection
        let backend_choice = self.hybrid_backend.select_backend(&validated_request).await?;
        
        // 3. Calculation execution
        let calculation_result = match backend_choice {
            Backend::Native => self.calculate_with_native(&validated_request).await?,
            Backend::Swiss => self.calculate_with_swiss(&validated_request).await?,
            Backend::Validated => self.calculate_with_validation(&validated_request).await?,
        };
        
        // 4. Result post-processing
        let final_result = self.post_process_result(calculation_result)?;
        
        Ok(final_result)
    }

    fn validate_request(&self, request: PanchangaRequest) -> Result<PanchangaRequest, EngineError> {
        // TODO: Implement request validation
        Ok(request)
    }

    async fn calculate_with_native(
        &self,
        request: &PanchangaRequest,
    ) -> Result<PanchangaResult, EngineError> {
        // TODO: Implement native engine calculation
        todo!("Native engine calculation not yet implemented")
    }

    async fn calculate_with_swiss(
        &self,
        request: &PanchangaRequest,
    ) -> Result<PanchangaResult, EngineError> {
        // TODO: Implement Swiss Ephemeris calculation
        todo!("Swiss Ephemeris calculation not yet implemented")
    }

    async fn calculate_with_validation(
        &self,
        request: &PanchangaRequest,
    ) -> Result<PanchangaResult, EngineError> {
        // TODO: Implement cross-validation calculation
        todo!("Cross-validation calculation not yet implemented")
    }

    fn post_process_result(&self, result: PanchangaResult) -> Result<PanchangaResult, EngineError> {
        // TODO: Implement result post-processing
        Ok(result)
    }
}

/// Available calculation backends
#[derive(Debug, Clone, Copy)]
pub enum Backend {
    Native,
    Swiss,
    Validated,
}
