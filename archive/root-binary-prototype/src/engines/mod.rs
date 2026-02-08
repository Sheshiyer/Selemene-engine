pub mod calculation_orchestrator;
pub mod hybrid_backend;
pub mod native_solar;
pub mod native_lunar;
pub mod swiss_ephemeris;
pub mod validation;
pub mod panchanga_calculator;

pub use calculation_orchestrator::CalculationOrchestrator;
pub use hybrid_backend::HybridBackend;
pub use native_solar::NativeSolarEngine;
pub use native_lunar::NativeLunarEngine;
pub use swiss_ephemeris::SwissEphemerisEngine;
pub use validation::ValidationEngine;
pub use panchanga_calculator::PanchangaCalculator;

/// Backend routing strategies
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendRoutingStrategy {
    AlwaysNative,
    AlwaysSwiss,
    Intelligent,
    Validated,
    PerformanceOptimized,
}

/// Available backends
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Backend {
    Native,
    Swiss,
    Validated,
}


