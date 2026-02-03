//! Eclipses Module
//!
//! FAPI-116: Solar and lunar eclipse calculations

pub mod types;
pub mod calculator;
pub mod effects;

pub use types::*;
pub use calculator::*;
pub use effects::*;

use chrono::{NaiveDate, NaiveDateTime};
use serde::{Deserialize, Serialize};

/// Type of eclipse
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EclipseType {
    SolarTotal,
    SolarPartial,
    SolarAnnular,
    SolarHybrid,
    LunarTotal,
    LunarPartial,
    LunarPenumbral,
}

impl std::fmt::Display for EclipseType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EclipseType::SolarTotal => write!(f, "Total Solar Eclipse"),
            EclipseType::SolarPartial => write!(f, "Partial Solar Eclipse"),
            EclipseType::SolarAnnular => write!(f, "Annular Solar Eclipse"),
            EclipseType::SolarHybrid => write!(f, "Hybrid Solar Eclipse"),
            EclipseType::LunarTotal => write!(f, "Total Lunar Eclipse"),
            EclipseType::LunarPartial => write!(f, "Partial Lunar Eclipse"),
            EclipseType::LunarPenumbral => write!(f, "Penumbral Lunar Eclipse"),
        }
    }
}

/// Eclipse event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EclipseEvent {
    /// Type of eclipse
    pub eclipse_type: EclipseType,
    /// Date of eclipse
    pub date: NaiveDate,
    /// Maximum eclipse time (UTC)
    pub maximum_time: NaiveDateTime,
    /// Duration of totality/annularity (if applicable)
    pub totality_duration: Option<String>,
    /// Zodiac sign where eclipse occurs
    pub zodiac_sign: String,
    /// Nakshatra where eclipse occurs
    pub nakshatra: String,
    /// Visibility regions
    pub visibility: Vec<String>,
    /// Magnitude
    pub magnitude: f64,
    /// Saros cycle
    pub saros: u16,
    /// Vedic interpretation
    pub vedic_effects: VedicEclipseEffects,
}

/// Vedic interpretation of eclipse
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VedicEclipseEffects {
    /// Affected zodiac signs
    pub affected_signs: Vec<String>,
    /// Affected nakshatras
    pub affected_nakshatras: Vec<String>,
    /// General effects
    pub general_effects: String,
    /// Recommended practices
    pub recommendations: Vec<String>,
    /// Things to avoid
    pub avoid: Vec<String>,
    /// Sutak (inauspicious period) timing
    pub sutak_starts: Option<NaiveDateTime>,
    pub sutak_ends: Option<NaiveDateTime>,
}

/// Eclipse visibility for a location
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalEclipseVisibility {
    pub is_visible: bool,
    pub visibility_type: String,  // Total, Partial, Not visible
    pub start_time: Option<NaiveDateTime>,
    pub maximum_time: Option<NaiveDateTime>,
    pub end_time: Option<NaiveDateTime>,
    pub obscuration: Option<f64>,  // Percentage for partial
}
