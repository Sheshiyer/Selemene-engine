//! Human Design Consciousness Engine
//!
//! 88° solar arc calculation, 64 gates, 9 centers, 36 channels, 5 types.
//! Requires Swiss Ephemeris for astronomical precision.

pub use noesis_core::{ConsciousnessEngine, EngineError, EngineInput, EngineOutput};

pub mod ephemeris;
pub mod gate_sequence;
pub mod design_time;
pub mod activations;
pub mod chart;
pub mod analysis;
pub mod witness;
pub mod engine;

// Re-export ephemeris calculator for convenience
pub use ephemeris::{EphemerisCalculator, HDPlanet, PlanetPosition};

// Re-export key functions for convenience
pub use gate_sequence::{longitude_to_gate, longitude_to_line, longitude_to_gate_and_line};
pub use design_time::{calculate_design_time, initialize_ephemeris, DesignTimeError};
pub use activations::{
    calculate_personality_sun_earth,
    calculate_design_sun_earth,
    calculate_sun_earth_activations,
    calculate_personality_activations,
    calculate_design_activations,
    calculate_all_activations,
};
pub use chart::generate_hd_chart;
pub use analysis::{
    analyze_centers,
    analyze_channels,
    determine_type,
    determine_authority,
    calculate_profile,
    determine_definition,
    analyze_hd_chart,
};

pub mod models;
pub mod wisdom;
pub mod wisdom_data;

pub use models::*;
pub use wisdom_data::{
    GATES, CENTERS, CHANNELS, TYPES, AUTHORITIES, PROFILES, LINES,
    DEFINITIONS, CIRCUITRY, INCARNATION_CROSSES, VARIABLES, PLANETARY_ACTIVATIONS,
    init_wisdom,
};
pub use witness::generate_witness_prompt;
pub use engine::HumanDesignEngine;
