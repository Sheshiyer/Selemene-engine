//! Numerology type definitions.
//!
//! All public input/output structs for the numerology engine live here.
//! `lib.rs` re-exports everything via `pub use types::*`.

use serde::{Deserialize, Serialize};

/// A single numerology number with its reduction chain and meaning.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NumerologyNumber {
    pub value: u32,
    pub is_master: bool,
    pub reduction_chain: Vec<u32>,
    pub meaning: String,
}

/// The complete set of numerology numbers calculated for a person.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NumerologyResult {
    pub life_path: NumerologyNumber,
    pub expression: NumerologyNumber,
    pub soul_urge: NumerologyNumber,
    pub personality: NumerologyNumber,
    pub birthday: NumerologyNumber,
    pub chaldean_name: NumerologyNumber,
}
