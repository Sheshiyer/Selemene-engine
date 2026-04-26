//! Vimshottari Dasha Consciousness Engine
//!
//! 120-year planetary period timeline based on Moon's nakshatra at birth.
//! 3-level hierarchy: Mahadasha → Antardasha → Pratyantardasha.

pub use noesis_core::{ConsciousnessEngine, EngineError, EngineInput, EngineOutput};

// Core modules
pub mod calculator;
pub mod engine;
pub mod models;
pub mod wisdom;
pub mod wisdom_data;
pub mod witness;

pub use engine::VimshottariEngine;

// Re-exports
pub use calculator::{
    calculate_antardashas, calculate_birth_nakshatra, calculate_complete_timeline,
    calculate_dasha_balance, calculate_mahadashas, calculate_pratyantardashas,
    enrich_period_with_qualities, get_nakshatra, get_nakshatra_from_longitude,
};
pub use models::*;
pub use wisdom_data::*;
pub use witness::generate_witness_prompt;
