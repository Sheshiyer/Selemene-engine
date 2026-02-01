//! Multi-engine workflow implementations for Noesis
//!
//! Each workflow combines multiple consciousness engines to provide
//! multi-perspective insights on a particular domain of inquiry.
//!
//! # Workflows
//!
//! - **Birth Blueprint**: Core identity mapping (numerology, human-design, vimshottari)
//! - **Daily Practice**: Temporal optimization (panchanga, vedic-clock, biorhythm)
//! - **Decision Support**: Multi-perspective guidance (tarot, i-ching, human-design)
//! - **Self-Inquiry**: Shadow work synthesis (gene-keys, enneagram)
//! - **Creative Expression**: Generative guidance (sigil-forge, sacred-geometry)
//! - **Full Spectrum**: All-engine integration

pub mod cache;
pub mod full_spectrum;
pub mod synthesis;
pub mod models;
pub mod registry;
pub mod executor;
pub mod birth_blueprint;
pub mod daily_practice;
pub mod decision_support;
pub mod self_inquiry;
pub mod creative_expression;
pub mod witness;

// Re-export primary types
pub use cache::{WorkflowCache, WorkflowCacheKey, WorkflowTtl};
pub use full_spectrum::{EngineCategory, FullSpectrumConfig, FullSpectrumResult, FullSpectrumWorkflow};
pub use models::{
    Theme, Alignment as ExtAlignment, Tension as ExtTension, 
    WitnessPrompt, InquiryType, TemporalWindow, WorkflowOutput,
};
pub use registry::WorkflowRegistry;
pub use executor::WorkflowExecutor;

// Re-export workflow implementations
pub use decision_support::DecisionSupportWorkflow;
pub use self_inquiry::SelfInquiryWorkflow;
pub use creative_expression::CreativeExpressionWorkflow;

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

/// Result of synthesizing multiple engine outputs (legacy format)
/// 
/// Note: New workflows should use `models::SynthesisResult` which has
/// stronger typed themes, alignments, and tensions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegacySynthesisResult {
    /// Type of synthesis performed
    pub synthesis_type: SynthesisType,
    /// Key themes identified across engines
    pub themes: Vec<String>,
    /// Alignments found between engine outputs
    pub alignments: Vec<Alignment>,
    /// Tensions or contrasts between perspectives
    pub tensions: Vec<Tension>,
    /// Non-prescriptive witness prompts for self-inquiry
    pub witness_prompts: Vec<String>,
    /// Summary narrative (non-interpretive)
    pub summary: String,
    /// Engine-specific extracted data
    pub extracted_data: HashMap<String, Value>,
}

// Re-export models::SynthesisResult as the primary type
pub use models::SynthesisResult;

/// An alignment between two or more engine outputs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alignment {
    /// Engines involved in this alignment
    pub engines: Vec<String>,
    /// Description of how they align
    pub description: String,
    /// Specific elements that align
    pub elements: Vec<String>,
}

/// A tension or contrast between perspectives
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tension {
    /// Engines involved in this tension
    pub engines: Vec<String>,
    /// Description of the multiple perspectives
    pub description: String,
    /// Framing as inquiry rather than contradiction
    pub inquiry_framing: String,
}

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
