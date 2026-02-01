//! Face Reading Consciousness Engine
//!
//! Constitutional analysis combining multiple face reading traditions:
//! - Chinese Face Reading (Mian Xiang): Five Elements analysis
//! - Ayurvedic Face Analysis: Dosha determination
//! - Western Physiognomy: Personality trait indicators
//!
//! # Current Status
//!
//! This is a **stub implementation** that returns mock analysis data.
//! Full implementation requires MediaPipe integration for facial landmark detection.
//!
//! # Usage
//!
//! ```rust,no_run
//! use engine_face_reading::FaceReadingEngine;
//! use noesis_core::{ConsciousnessEngine, EngineInput, Precision};
//! use chrono::Utc;
//! use std::collections::HashMap;
//!
//! #[tokio::main]
//! async fn main() {
//!     let engine = FaceReadingEngine::new();
//!     
//!     let mut options = HashMap::new();
//!     options.insert("seed".to_string(), serde_json::json!(42)); // For reproducibility
//!     
//!     let input = EngineInput {
//!         birth_data: None,
//!         current_time: Utc::now(),
//!         location: None,
//!         precision: Precision::Standard,
//!         options,
//!     };
//!     
//!     let output = engine.calculate(input).await.unwrap();
//!     println!("Analysis: {}", output.result);
//!     println!("Witness prompt: {}", output.witness_prompt);
//! }
//! ```

pub mod models;
pub mod wisdom;
pub mod mock;
pub mod witness;
pub mod engine;

// Re-export main types
pub use models::{
    BodyType, ConstitutionAnalysis, Dosha, Element, ElementalBalance,
    FaceAnalysis, FaceZone, HealthIndicator, PersonalityTrait,
};
pub use wisdom::{FaceZoneWisdom, ZoneIndicator, get_zone_wisdom, all_zone_wisdom};
pub use mock::generate_mock_analysis;
pub use witness::{generate_witness_prompts, generate_single_witness_prompt};
pub use engine::FaceReadingEngine;

// Re-export core types
pub use noesis_core::{ConsciousnessEngine, EngineError, EngineInput, EngineOutput};
