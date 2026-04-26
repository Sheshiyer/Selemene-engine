//! Gene Keys Consciousness Engine
//!
//! Shadow-Gift-Siddhi transformation framework.
//! Maps HD gate activations to Gene Keys and calculates the 4 Core Activation Sequences.

pub mod engine;
pub mod frequency;
pub mod mapping;
pub mod models;
pub mod transformation;
pub mod wisdom;
pub mod witness;

pub use engine::GeneKeysEngine;
pub use frequency::{assess_frequencies, Frequency, FrequencyAssessment, RecognitionPrompts};
pub use mapping::{
    calculate_activation_sequences, extract_sun_earth_gates, find_activation_by_planet,
    map_hd_to_gene_keys,
};
pub use models::{
    ActivationSequence, ActivationSource, GeneKey, GeneKeyActivation, GeneKeysChart, GeneKeysData,
    GeneKeysInfo,
};
pub use transformation::{
    generate_complete_pathways, generate_transformation_pathways, TransformationPathway,
};
pub use wisdom::{gene_keys, get_gene_key};
pub use witness::generate_witness_prompt;

pub use noesis_core::{ConsciousnessEngine, EngineError, EngineInput, EngineOutput};
