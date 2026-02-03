//! Transit calculation types
//!
//! FAPI-073: Define transit calculation types

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

/// A transit event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransitEvent {
    /// Planet making the transit
    pub transiting_planet: String,
    /// Sign being transited
    pub sign: String,
    /// Degree in sign
    pub degree: f64,
    /// Start date of this transit
    pub start_date: NaiveDate,
    /// End date (when planet leaves sign)
    pub end_date: Option<NaiveDate>,
    /// Whether planet is retrograde
    pub is_retrograde: bool,
    /// Aspects to natal planets
    pub aspects: Vec<TransitAspect>,
}

/// An aspect formed during transit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransitAspect {
    /// Natal planet being aspected
    pub natal_planet: String,
    /// Natal position
    pub natal_sign: String,
    /// Aspect type (conjunction, opposition, etc.)
    pub aspect_type: AspectType,
    /// Whether aspect is applying or separating
    pub is_applying: bool,
    /// Orb in degrees
    pub orb: f64,
    /// Nature of the aspect (benefic, malefic, neutral)
    pub nature: AspectNature,
}

/// Types of aspects
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AspectType {
    /// Same sign (0°)
    Conjunction,
    /// Opposite sign (180°)
    Opposition,
    /// 4th/10th house (90°)
    Square,
    /// 5th/9th house (120°)
    Trine,
    /// 3rd/11th house (60°)
    Sextile,
    /// 7th house aspect (Vedic full aspect)
    SeventhAspect,
    /// Special Mars aspects (4th, 7th, 8th)
    MarsSpecial,
    /// Special Jupiter aspects (5th, 7th, 9th)
    JupiterSpecial,
    /// Special Saturn aspects (3rd, 7th, 10th)
    SaturnSpecial,
}

impl std::fmt::Display for AspectType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AspectType::Conjunction => write!(f, "Conjunction"),
            AspectType::Opposition => write!(f, "Opposition"),
            AspectType::Square => write!(f, "Square"),
            AspectType::Trine => write!(f, "Trine"),
            AspectType::Sextile => write!(f, "Sextile"),
            AspectType::SeventhAspect => write!(f, "7th Aspect"),
            AspectType::MarsSpecial => write!(f, "Mars Special"),
            AspectType::JupiterSpecial => write!(f, "Jupiter Special"),
            AspectType::SaturnSpecial => write!(f, "Saturn Special"),
        }
    }
}

/// Nature of an aspect
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AspectNature {
    Benefic,
    Malefic,
    Neutral,
    Mixed,
}

/// Transit period analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransitAnalysis {
    /// Date of analysis
    pub analysis_date: NaiveDate,
    /// Current transits
    pub current_transits: Vec<TransitEvent>,
    /// Significant transit aspects
    pub significant_aspects: Vec<TransitAspect>,
    /// Sade Sati status
    pub sade_sati_status: Option<SadeSatiStatus>,
    /// Jupiter transit status
    pub jupiter_transit: Option<JupiterTransitStatus>,
    /// Overall period quality
    pub period_quality: PeriodQuality,
    /// Key dates in coming months
    pub upcoming_dates: Vec<SignificantDate>,
}

/// Sade Sati (Saturn's 7.5 year transit over Moon)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SadeSatiStatus {
    /// Is person currently in Sade Sati
    pub is_active: bool,
    /// Phase (rising, peak, setting)
    pub phase: Option<SadeSatiPhase>,
    /// Start date
    pub start_date: Option<NaiveDate>,
    /// End date
    pub end_date: Option<NaiveDate>,
    /// Saturn's current sign
    pub saturn_sign: String,
    /// Natal Moon sign
    pub moon_sign: String,
}

/// Sade Sati phases
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SadeSatiPhase {
    /// Saturn in 12th from Moon (rising phase)
    Rising,
    /// Saturn in same sign as Moon (peak phase)
    Peak,
    /// Saturn in 2nd from Moon (setting phase)
    Setting,
}

/// Jupiter transit status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JupiterTransitStatus {
    /// Jupiter's current sign
    pub current_sign: String,
    /// Relationship to ascendant
    pub from_ascendant: u8,
    /// Relationship to Moon
    pub from_moon: u8,
    /// Quality of transit
    pub quality: TransitQuality,
    /// Areas of life affected
    pub affected_areas: Vec<String>,
}

/// Transit quality
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransitQuality {
    Excellent,
    Good,
    Neutral,
    Challenging,
    Difficult,
}

/// Overall period quality
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PeriodQuality {
    HighlyFavorable,
    Favorable,
    Mixed,
    Challenging,
    Difficult,
}

/// A significant upcoming date
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignificantDate {
    pub date: NaiveDate,
    pub event: String,
    pub planets_involved: Vec<String>,
    pub nature: AspectNature,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aspect_type_display() {
        assert_eq!(AspectType::Conjunction.to_string(), "Conjunction");
        assert_eq!(AspectType::Trine.to_string(), "Trine");
    }

    #[test]
    fn test_sade_sati_phases() {
        let status = SadeSatiStatus {
            is_active: true,
            phase: Some(SadeSatiPhase::Peak),
            start_date: None,
            end_date: None,
            saturn_sign: "Aquarius".to_string(),
            moon_sign: "Aquarius".to_string(),
        };
        
        assert!(status.is_active);
        assert_eq!(status.phase, Some(SadeSatiPhase::Peak));
    }
}
