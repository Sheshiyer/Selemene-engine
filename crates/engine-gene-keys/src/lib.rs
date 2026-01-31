//! Gene Keys Consciousness Engine
//!
//! Shadow-Gift-Siddhi transformation framework.
//! Maps HD gate activations to Gene Keys and calculates the 4 Core Activation Sequences.

pub mod models;
pub mod mapping;
pub mod wisdom;
pub mod frequency;
pub mod transformation;
pub mod witness;
pub mod engine;

pub use models::{
    ActivationSequence, ActivationSource, GeneKey, GeneKeyActivation, GeneKeysChart,
    GeneKeysData, GeneKeysInfo,
};
pub use mapping::{
    map_hd_to_gene_keys,
    calculate_activation_sequences,
    find_activation_by_planet,
    extract_sun_earth_gates,
};
pub use wisdom::{gene_keys, get_gene_key};
pub use frequency::{
    Frequency, FrequencyAssessment, RecognitionPrompts, assess_frequencies,
};
pub use transformation::{
    TransformationPathway, generate_transformation_pathways, generate_complete_pathways,
};
pub use witness::generate_witness_prompt;
pub use engine::GeneKeysEngine;

pub use noesis_core::{ConsciousnessEngine, EngineError, EngineInput, EngineOutput};
