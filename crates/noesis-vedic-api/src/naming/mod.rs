//! Naming (Namkaran) Module
//!
//! FAPI-118: Vedic naming suggestions based on birth nakshatra

pub mod types;
pub mod generator;
pub mod syllables;

pub use types::*;
pub use generator::*;
pub use syllables::*;

use serde::{Deserialize, Serialize};

/// Name suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NameSuggestion {
    /// Suggested name
    pub name: String,
    /// Starting syllable
    pub syllable: String,
    /// Meaning
    pub meaning: String,
    /// Gender
    pub gender: Gender,
    /// Origin/language
    pub origin: String,
    /// Numerology number
    pub numerology: u8,
    /// Popularity score
    pub popularity: u8,
}

/// Gender for naming
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Gender {
    Male,
    Female,
    Neutral,
}

/// Naming request based on birth details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NamingRequest {
    /// Moon nakshatra at birth
    pub moon_nakshatra: String,
    /// Nakshatra pada (1-4)
    pub nakshatra_pada: u8,
    /// Gender of child
    pub gender: Gender,
    /// Preferred origin/language
    pub preferred_origin: Option<String>,
    /// Number of suggestions
    pub count: usize,
}

/// Naming response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NamingResponse {
    /// Moon nakshatra
    pub nakshatra: String,
    /// Pada
    pub pada: u8,
    /// Auspicious syllables
    pub syllables: Vec<String>,
    /// Name suggestions
    pub suggestions: Vec<NameSuggestion>,
    /// Naming ceremony guidelines
    pub namkaran_info: NamkaranInfo,
}

/// Namkaran ceremony information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NamkaranInfo {
    /// Recommended day for ceremony
    pub recommended_day: String,
    /// Auspicious tithis
    pub auspicious_tithis: Vec<String>,
    /// Ceremony procedure
    pub procedure: Vec<String>,
    /// Things to avoid
    pub avoid: Vec<String>,
}
