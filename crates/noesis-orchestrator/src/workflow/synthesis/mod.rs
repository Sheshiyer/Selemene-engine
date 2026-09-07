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

pub mod birth_blueprint;
pub mod creative_expression;
pub mod daily_practice;
pub mod decision_support;
pub mod full_spectrum;
pub mod self_inquiry;

pub use birth_blueprint::BirthBlueprintSynthesizer;
pub use creative_expression::CreativeExpressionSynthesis;
pub use daily_practice::DailyPracticeSynthesizer;
pub use decision_support::DecisionSupportSynthesis;
pub use full_spectrum::{CrossEngineTheme, FullSpectrumSynthesizer, ThemeCategory};
pub use self_inquiry::SelfInquirySynthesis;

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

/// Dispatch the producer-backed synthesis modules by canonical workflow ID.
/// Full Spectrum intentionally returns `None` until its aggregate contract has
/// a lossless adapter; callers must surface that as `unsupported`.
pub fn synthesize_supported(
    workflow_id: &str,
    results: &HashMap<String, EngineOutput>,
    input: &EngineInput,
) -> Option<ExtSynthesisResult> {
    let synthesis = match workflow_id {
        "birth-blueprint" => BirthBlueprintSynthesizer::synthesize(results, input),
        "daily-practice" => DailyPracticeSynthesizer::synthesize(results, input),
        "decision-support" => DecisionSupportSynthesis::synthesize(results, input),
        "self-inquiry" => SelfInquirySynthesis::synthesize(results, input),
        "creative-expression" => CreativeExpressionSynthesis::synthesize(results, input),
        _ => return None,
    };
    (!synthesis.summary.trim().is_empty()).then_some(synthesis)
}
