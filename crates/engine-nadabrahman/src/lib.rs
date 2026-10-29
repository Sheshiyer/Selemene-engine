//! NadaBrahman Consciousness Engine
//!
//! Raga-based sound therapy engine for the Selemene consciousness platform.
//! Recommends Melakarta ragas based on time of day (prahar system), Ayurvedic
//! dosha constitution, mood/rasa, and chakra-frequency mappings.
//!
//! # Data Sources
//!
//! All 72 Melakarta ragas and their therapeutic correlations are loaded from
//! JSON data files compiled into the binary at build time:
//!
//! - `data/nadabrahman/melakarta_ragas.json` — 72 ragas with swara patterns
//! - `data/nadabrahman/time_raga_mappings.json` — 8 prahars mapped to ragas
//! - `data/nadabrahman/chakra_frequencies.json` — 7 chakras with Solfeggio frequencies
//!
//! # Usage
//!
//! ```rust,ignore
//! use engine_nadabrahman::NadaBrahmanEngine;
//! use noesis_core::{ConsciousnessEngine, EngineInput};
//!
//! let engine = NadaBrahmanEngine::new();
//! let input = EngineInput { /* ... */ };
//! let output = engine.calculate(input).await?;
//! ```
//!
//! # Options
//!
//! The engine accepts these options in `EngineInput::options`:
//!
//! - `dosha` (string): Ayurvedic constitution — "vata", "pitta", or "kapha"
//! - `rasa` / `mood` (string): Emotional quality — one of the 9 rasas
//! - `chakra` (string): Target chakra — "root", "sacral", "solar_plexus", "heart", "throat", "third_eye", "crown"
//! - `consciousness_level` (u8): User's consciousness level 0-5
//! - `timezone` (string): IANA zone (e.g. "Asia/Kolkata") used to read the local
//!   hour of `current_time` for the prahar; takes precedence over `timezone_offset`
//! - `timezone_offset` (integer): minutes east of UTC (e.g. 330, -300), as in vedic-clock
//!
//! With neither, the prahar is read from the UTC hour. The result's `time_basis`
//! reports `{timezone, offset_minutes, local_hour, source}` where `source` is
//! `"options"` or `"utc_default"`.

pub mod data;
pub mod engine;
pub mod models;
pub mod witness;

pub use engine::NadaBrahmanEngine;
pub use models::{
    ChakraFrequency, NadaBrahmanAnalysis, Prahar, PraharRecommendation, Raga, RagaRecommendation,
};
pub use witness::{generate_witness_prompt, generate_witness_prompts};

pub use noesis_core::{ConsciousnessEngine, EngineError, EngineInput, EngineOutput};
