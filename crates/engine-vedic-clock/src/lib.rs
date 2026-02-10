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

pub mod calculator;
pub mod choghadiya_integration;
pub mod dosha;
pub mod engine;
pub mod hora_integration;
pub mod integration;
pub mod models;
pub mod organ_clock;
pub mod panchang_integration;
pub mod panchanga_qualities;
pub mod recommendations;
pub mod wisdom;
pub mod witness;

// Re-export main types
pub use calculator::{get_current_organ, get_local_hour, minutes_until_next_transition};
pub use choghadiya_integration::recommendations_from_choghadiya;
pub use dosha::{
    calculate_dosha_organ_harmony, dosha_times, get_dosha_for_hour, get_organ_dosha_affinity,
};
pub use engine::VedicClockEngine;
pub use hora_integration::recommendations_from_hora;
pub use integration::{
    get_activity_favorability, get_temporal_recommendation, synthesize_organ_dosha,
};
pub use models::{
    Activity, ActivityRecommendation, Dosha, DoshaTime, Element, Organ, OrganWindow,
    TemporalRecommendation, TimeWindow, UpcomingTransition, VedicClockResult,
};
pub use organ_clock::get_temporal_recommendation_with_api;
pub use panchang_integration::recommendation_from_complete_panchang;
pub use panchanga_qualities::{
    get_combined_quality, get_tithi_quality, PanchangaQuality, QualityRating,
};
pub use recommendations::{get_best_time, get_optimal_timing, is_favorable_now};
pub use wisdom::{get_opposing_organ, get_organ_element, get_organ_for_hour, organ_clock};
pub use witness::generate_witness_prompt;

// Re-export core traits
pub use noesis_core::{ConsciousnessEngine, EngineError, EngineInput, EngineOutput};
