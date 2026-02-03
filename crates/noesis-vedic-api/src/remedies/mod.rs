//! Remedies Module
//!
//! FAPI-119, FAPI-120: Vedic remedies and gemstone recommendations

pub mod types;
pub mod gemstones;
pub mod mantras;
pub mod donations;

pub use types::*;
pub use gemstones::*;
pub use mantras::*;
pub use donations::*;

use serde::{Deserialize, Serialize};

/// Type of remedy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RemedyType {
    Gemstone,
    Mantra,
    Donation,
    Ritual,
    Fasting,
    Yantra,
    Rudraksha,
}

/// Remedy suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemedySuggestion {
    /// Type of remedy
    pub remedy_type: RemedyType,
    /// Name
    pub name: String,
    /// For which planet
    pub planet: String,
    /// Description
    pub description: String,
    /// How to use/perform
    pub instructions: Vec<String>,
    /// Best day/time
    pub timing: String,
    /// Expected benefits
    pub benefits: Vec<String>,
    /// Precautions
    pub precautions: Vec<String>,
}

/// Comprehensive remedy package for a chart
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemedyPackage {
    /// Primary issues identified
    pub issues: Vec<String>,
    /// Gemstone recommendations
    pub gemstones: Vec<GemstoneRecommendation>,
    /// Mantra recommendations
    pub mantras: Vec<MantraRecommendation>,
    /// Donation recommendations
    pub donations: Vec<DonationRecommendation>,
    /// Other remedies
    pub other_remedies: Vec<RemedySuggestion>,
    /// Priority order
    pub priority: Vec<String>,
    /// General advice
    pub general_advice: String,
}

/// Request for remedies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemedyRequest {
    /// Weak planets in chart
    pub weak_planets: Vec<String>,
    /// Afflicted houses
    pub afflicted_houses: Vec<u8>,
    /// Active doshas
    pub doshas: Vec<String>,
    /// Current dasha lord
    pub current_dasha_lord: Option<String>,
    /// Specific concerns
    pub concerns: Vec<String>,
}
