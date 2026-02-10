//! Biofield Consciousness Engine
//!
//! Analyzes biofield energy patterns from PIP (Polycontrast Interference Photography)
//! devices. Currently returns mock data - full implementation requires hardware integration.
//!
//! # Overview
//!
//! The biofield engine provides:
//! - Biofield metrics (fractal dimension, entropy, coherence, symmetry)
//! - Chakra energy readings for all 7 primary chakras
//! - Composite vitality index
//! - Somatic awareness witness prompts
//!
//! # Mock Data Notice
//!
//! This is a **stub implementation** that returns simulated data. Full biofield
//! analysis requires PIP hardware integration which will be added in future releases.
//!
//! # Usage
//!
//! ```rust,ignore
//! use engine_biofield::BiofieldEngine;
//! use noesis_core::{ConsciousnessEngine, EngineInput};
//!
//! let engine = BiofieldEngine::new();
//! let input = EngineInput { /* ... */ };
//! let output = engine.calculate(input).await?;
//! ```

pub mod engine;
pub mod mock;
pub mod models;
pub mod wisdom;
pub mod witness;

pub use engine::BiofieldEngine;
pub use mock::{generate_metrics_for_user, generate_mock_metrics};
pub use models::{BiofieldAnalysis, BiofieldMetrics, Chakra, ChakraReading};
pub use wisdom::{
    get_chakra_wisdom, get_metric_interpretation, ChakraWisdom, MetricInterpretation,
};
pub use witness::{generate_witness_prompt, generate_witness_prompts};

pub use noesis_core::{ConsciousnessEngine, EngineError, EngineInput, EngineOutput};
