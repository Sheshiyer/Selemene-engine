//! Synthesis modules for combining engine outputs
//!
//! Each synthesis module extracts relevant data from engine outputs
//! and finds alignments, tensions, and themes across multiple perspectives.
//!
//! # Synthesizers
//!
//! - **BirthBlueprintSynthesizer**: Cross-references natal patterns from numerology, HD, vimshottari
//! - **DailyPracticeSynthesizer**: Combines temporal recommendations from panchanga, vedic-clock, biorhythm
//! - **DecisionSupportSynthesis**: Aligns Tarot, I-Ching, and HD Authority perspectives
//! - **SelfInquirySynthesis**: Maps Gene Keys shadows to Enneagram core patterns
//! - **CreativeExpressionSynthesis**: Combines Sigil and Sacred Geometry for creative direction
//! - **FullSpectrumSynthesizer**: Integrates all engines

pub mod full_spectrum;
pub mod birth_blueprint;
pub mod daily_practice;
pub mod decision_support;
pub mod self_inquiry;
pub mod creative_expression;

pub use full_spectrum::{CrossEngineTheme, FullSpectrumSynthesizer, ThemeCategory};
pub use birth_blueprint::BirthBlueprintSynthesizer;
pub use daily_practice::DailyPracticeSynthesizer;
pub use decision_support::DecisionSupportSynthesis;
pub use self_inquiry::SelfInquirySynthesis;
pub use creative_expression::CreativeExpressionSynthesis;

use crate::workflow::models::SynthesisResult as ExtSynthesisResult;
use noesis_core::{EngineInput, EngineOutput};
use std::collections::HashMap;

/// Trait for workflow-specific synthesis logic
pub trait Synthesizer {
    /// Synthesize results from multiple engines
    fn synthesize(
        results: &HashMap<String, EngineOutput>,
        input: &EngineInput,
    ) -> ExtSynthesisResult;
}
