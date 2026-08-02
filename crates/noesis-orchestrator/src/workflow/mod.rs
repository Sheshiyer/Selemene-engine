//! Multi-engine workflow implementations for Noesis
//!
//! Each workflow combines multiple consciousness engines to provide
//! multi-perspective insights on a particular domain of inquiry.
//!
//! # Workflows
//!
//! - **Birth Blueprint**: Core identity mapping (numerology, human-design, vimshottari)
//! - **Daily Practice**: Temporal calibration (panchanga, vedic-clock, biorhythm)
//! - **Decision Support**: Multi-perspective guidance (tarot, i-ching, human-design)
//! - **Self-Inquiry**: Shadow work synthesis (gene-keys, enneagram)
//! - **Creative Expression**: Generative guidance (sigil-forge, sacred-geometry)
//! - **Full Spectrum**: All-engine integration

pub mod birth_blueprint;
pub mod cache;
pub mod creative_expression;
pub mod daily_practice;
pub mod decision_support;
pub mod executor;
pub mod full_spectrum;
pub mod models;
pub mod registry;
pub mod self_inquiry;
pub mod synthesis;
pub mod witness;

// Re-export primary types
pub use cache::{WorkflowCache, WorkflowCacheKey, WorkflowTtl};
pub use executor::WorkflowExecutor;
pub use full_spectrum::{
    EngineCategory, FullSpectrumConfig, FullSpectrumResult, FullSpectrumWorkflow,
};
pub use models::{
    Alignment as ExtAlignment, InquiryType, TemporalWindow, Tension as ExtTension, Theme,
    WitnessPrompt, WorkflowOutput,
};
pub use registry::WorkflowRegistry;

// Re-export workflow implementations
pub use creative_expression::CreativeExpressionWorkflow;
pub use decision_support::DecisionSupportWorkflow;
pub use self_inquiry::SelfInquiryWorkflow;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Types of synthesis approaches available for workflows
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SynthesisType {
    /// Birth blueprint: Numerology + Human Design + Vimshottari synthesis
    BirthBlueprint,
    /// Daily practice: Panchanga + Vedic Clock + Biorhythm synthesis
    DailyPractice,
    /// Decision support: Tarot + I-Ching + HD Authority alignment
    DecisionSupport,
    /// Self-inquiry: Gene Keys shadows + Enneagram patterns
    SelfInquiry,
    /// Creative expression: Sigil + Sacred Geometry combination
    CreativeExpression,
    /// Full spectrum: All engines integration
    FullSpectrum,
    /// No synthesis (raw engine outputs only)
    None,
}

// Re-export models::SynthesisResult as the primary synthesis type
pub use models::SynthesisResult;

/// Extended workflow definition with synthesis support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtendedWorkflowDefinition {
    pub id: String,
    pub name: String,
    pub description: String,
    pub engine_ids: Vec<String>,
    pub synthesis_type: SynthesisType,
    pub required_phase: u8,
    pub default_options: HashMap<String, Value>,
}

impl ExtendedWorkflowDefinition {
    /// Convert to the base WorkflowDefinition type
    pub fn to_base(&self) -> noesis_core::WorkflowDefinition {
        noesis_core::WorkflowDefinition {
            id: self.id.clone(),
            name: self.name.clone(),
            description: self.description.clone(),
            engine_ids: self.engine_ids.clone(),
        }
    }
}
