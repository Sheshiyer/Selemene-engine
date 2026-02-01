//! VedicClock-TCM Consciousness Engine
//!
//! Synthesizes two ancient time-keeping systems:
//! 1. **TCM Organ Clock**: 12 organs, each active for 2-hour windows in 24-hour cycle
//! 2. **Vedic Panchanga**: Tithi, Nakshatra, Yoga, Karana - temporal qualities
//!
//! The engine provides optimal timing recommendations based on current time + location.
//!
//! # Features
//! - TCM 12-organ clock with associated elements and emotions
//! - Ayurvedic dosha time periods (Vata, Pitta, Kapha)
//! - Dosha-organ correspondence mapping
//! - Activity-based timing recommendations
//! - Optional Panchanga quality overlays
//! - Non-prescriptive witness prompts for self-observation
//!
//! # Usage
//! ```rust,ignore
//! use engine_vedic_clock::VedicClockEngine;
//! use noesis_core::{ConsciousnessEngine, EngineInput};
//!
//! let engine = VedicClockEngine::new();
//! let input = EngineInput { /* ... */ };
//! let output = engine.calculate(input).await?;
//! ```

pub mod models;
pub mod wisdom;
pub mod calculator;
pub mod dosha;
pub mod panchanga_qualities;
pub mod integration;
pub mod recommendations;
pub mod witness;
pub mod engine;

// Re-export main types
pub use models::{
    Activity, ActivityRecommendation, Dosha, DoshaTime, Element, Organ, OrganWindow,
    TemporalRecommendation, TimeWindow, UpcomingTransition, VedicClockResult,
};
pub use wisdom::{get_organ_for_hour, organ_clock, get_organ_element, get_opposing_organ};
pub use calculator::{get_current_organ, get_local_hour, minutes_until_next_transition};
pub use dosha::{dosha_times, get_dosha_for_hour, get_organ_dosha_affinity, calculate_dosha_organ_harmony};
pub use panchanga_qualities::{get_combined_quality, get_tithi_quality, PanchangaQuality, QualityRating};
pub use integration::{get_temporal_recommendation, get_activity_favorability, synthesize_organ_dosha};
pub use recommendations::{get_optimal_timing, get_best_time, is_favorable_now};
pub use witness::generate_witness_prompt;
pub use engine::VedicClockEngine;

// Re-export core traits
pub use noesis_core::{ConsciousnessEngine, EngineError, EngineInput, EngineOutput};
