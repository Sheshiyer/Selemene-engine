//! Vimshottari Dasha Consciousness Engine
//!
//! 120-year planetary period timeline based on Moon's nakshatra at birth.
//! 3-level hierarchy: Mahadasha → Antardasha → Pratyantardasha.

pub use noesis_core::{ConsciousnessEngine, EngineError, EngineInput, EngineOutput};

// Core modules
pub mod models;
pub mod calculator;
pub mod wisdom;
pub mod wisdom_data;
pub mod witness;
pub mod engine;

pub use engine::VimshottariEngine;

// Re-exports
pub use models::*;
pub use wisdom_data::*;
pub use calculator::{
    calculate_birth_nakshatra,
    calculate_dasha_balance,
    calculate_mahadashas,
    calculate_antardashas,
    calculate_pratyantardashas,
    calculate_complete_timeline,
    get_nakshatra_from_longitude,
    get_nakshatra,
    enrich_period_with_qualities,
};
pub use witness::generate_witness_prompt;
