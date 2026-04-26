//! Human Design Consciousness Engine
//!
//! 88° solar arc calculation, 64 gates, 9 centers, 36 channels, 5 types.
//! Requires Swiss Ephemeris for astronomical precision.

pub use noesis_core::{ConsciousnessEngine, EngineError, EngineInput, EngineOutput};

pub mod activations;
pub mod analysis;
pub mod chart;
pub mod design_time;
pub mod engine;
pub mod ephemeris;
pub mod gate_sequence;
pub mod witness;

// Re-export ephemeris calculator for convenience
pub use ephemeris::{EphemerisCalculator, HDPlanet, PlanetPosition};

// Re-export key functions for convenience
pub use activations::{
    calculate_all_activations, calculate_design_activations, calculate_design_sun_earth,
    calculate_personality_activations, calculate_personality_sun_earth,
    calculate_sun_earth_activations,
};
pub use analysis::{
    analyze_centers, analyze_channels, analyze_hd_chart, calculate_profile, determine_authority,
    determine_definition, determine_type,
};
pub use chart::generate_hd_chart;
pub use design_time::{calculate_design_time, initialize_ephemeris, DesignTimeError};
pub use gate_sequence::{longitude_to_gate, longitude_to_gate_and_line, longitude_to_line};

pub mod models;
pub mod wisdom;
pub mod wisdom_data;

pub use engine::HumanDesignEngine;
pub use models::*;
pub use wisdom_data::{
    init_wisdom, AUTHORITIES, CENTERS, CHANNELS, CIRCUITRY, DEFINITIONS, GATES,
    INCARNATION_CROSSES, LINES, PLANETARY_ACTIVATIONS, PROFILES, TYPES, VARIABLES,
};
pub use witness::generate_witness_prompt;
