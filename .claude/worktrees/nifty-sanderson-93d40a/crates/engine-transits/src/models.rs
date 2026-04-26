//! Transit analysis data models
//!
//! Defines planetary positions, aspects, and transit analysis results.

use serde::{Deserialize, Serialize};

/// Planets used in transit analysis
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TransitPlanet {
    Sun,
    Moon,
    Mercury,
    Venus,
    Mars,
    Jupiter,
    Saturn,
    Uranus,
    Neptune,
    Pluto,
    /// Rahu (North Node)
    Rahu,
    /// Ketu (South Node)
    Ketu,
}

impl TransitPlanet {
    /// All planets for transit calculation
    pub fn all() -> &'static [TransitPlanet] {
        &[
            Self::Sun,
            Self::Moon,
            Self::Mercury,
            Self::Venus,
            Self::Mars,
            Self::Jupiter,
            Self::Saturn,
            Self::Uranus,
            Self::Neptune,
            Self::Pluto,
            Self::Rahu,
            Self::Ketu,
        ]
    }

    /// Display name
    pub fn name(&self) -> &'static str {
        match self {
            Self::Sun => "Sun",
            Self::Moon => "Moon",
            Self::Mercury => "Mercury",
            Self::Venus => "Venus",
            Self::Mars => "Mars",
            Self::Jupiter => "Jupiter",
            Self::Saturn => "Saturn",
            Self::Uranus => "Uranus",
            Self::Neptune => "Neptune",
            Self::Pluto => "Pluto",
            Self::Rahu => "Rahu",
            Self::Ketu => "Ketu",
        }
    }
}

impl std::fmt::Display for TransitPlanet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Zodiac signs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ZodiacSign {
    Aries,
    Taurus,
    Gemini,
    Cancer,
    Leo,
    Virgo,
    Libra,
    Scorpio,
    Sagittarius,
    Capricorn,
    Aquarius,
    Pisces,
}

impl ZodiacSign {
    /// Get sign from ecliptic longitude (0-360)
    pub fn from_longitude(longitude: f64) -> Self {
        let normalized = ((longitude % 360.0) + 360.0) % 360.0;
        match (normalized / 30.0) as u8 {
            0 => Self::Aries,
            1 => Self::Taurus,
            2 => Self::Gemini,
            3 => Self::Cancer,
            4 => Self::Leo,
            5 => Self::Virgo,
            6 => Self::Libra,
            7 => Self::Scorpio,
            8 => Self::Sagittarius,
            9 => Self::Capricorn,
            10 => Self::Aquarius,
            _ => Self::Pisces,
        }
    }

    /// Get degree within sign (0-30) from ecliptic longitude
    pub fn degree_in_sign(longitude: f64) -> f64 {
        let normalized = ((longitude % 360.0) + 360.0) % 360.0;
        normalized % 30.0
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Aries => "Aries",
            Self::Taurus => "Taurus",
            Self::Gemini => "Gemini",
            Self::Cancer => "Cancer",
            Self::Leo => "Leo",
            Self::Virgo => "Virgo",
            Self::Libra => "Libra",
            Self::Scorpio => "Scorpio",
            Self::Sagittarius => "Sagittarius",
            Self::Capricorn => "Capricorn",
            Self::Aquarius => "Aquarius",
            Self::Pisces => "Pisces",
        }
    }

    /// Sign index (0 = Aries, 11 = Pisces)
    pub fn index(&self) -> u8 {
        match self {
            Self::Aries => 0,
            Self::Taurus => 1,
            Self::Gemini => 2,
            Self::Cancer => 3,
            Self::Leo => 4,
            Self::Virgo => 5,
            Self::Libra => 6,
            Self::Scorpio => 7,
            Self::Sagittarius => 8,
            Self::Capricorn => 9,
            Self::Aquarius => 10,
            Self::Pisces => 11,
        }
    }
}

impl std::fmt::Display for ZodiacSign {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Position of a planet at a specific time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanetaryPosition {
    pub planet: TransitPlanet,
    /// Ecliptic longitude (0-360)
    pub longitude: f64,
    /// Ecliptic latitude
    pub latitude: f64,
    /// Speed in degrees per day (negative = retrograde)
    pub speed: f64,
    /// Zodiac sign
    pub sign: ZodiacSign,
    /// Degree within sign (0-30)
    pub degree_in_sign: f64,
    /// Whether the planet is retrograde
    pub is_retrograde: bool,
}

/// Types of aspects
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AspectType {
    /// 0 degrees
    Conjunction,
    /// 180 degrees
    Opposition,
    /// 120 degrees
    Trine,
    /// 90 degrees
    Square,
    /// 60 degrees
    Sextile,
}

impl AspectType {
    /// Target angle for this aspect
    pub fn angle(&self) -> f64 {
        match self {
            Self::Conjunction => 0.0,
            Self::Opposition => 180.0,
            Self::Trine => 120.0,
            Self::Square => 90.0,
            Self::Sextile => 60.0,
        }
    }

    /// Default orb allowance for this aspect
    pub fn default_orb(&self) -> f64 {
        match self {
            Self::Conjunction => 8.0,
            Self::Opposition => 8.0,
            Self::Trine => 6.0,
            Self::Square => 6.0,
            Self::Sextile => 4.0,
        }
    }

    /// Nature of the aspect
    pub fn nature(&self) -> AspectNature {
        match self {
            Self::Conjunction => AspectNature::Neutral,
            Self::Opposition => AspectNature::Challenging,
            Self::Trine => AspectNature::Harmonious,
            Self::Square => AspectNature::Challenging,
            Self::Sextile => AspectNature::Harmonious,
        }
    }

    /// All aspect types to check
    pub fn all() -> &'static [AspectType] {
        &[
            Self::Conjunction,
            Self::Opposition,
            Self::Trine,
            Self::Square,
            Self::Sextile,
        ]
    }
}

impl std::fmt::Display for AspectType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Conjunction => write!(f, "Conjunction"),
            Self::Opposition => write!(f, "Opposition"),
            Self::Trine => write!(f, "Trine"),
            Self::Square => write!(f, "Square"),
            Self::Sextile => write!(f, "Sextile"),
        }
    }
}

/// Nature of an aspect
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AspectNature {
    Harmonious,
    Challenging,
    Neutral,
}

/// An aspect between a transiting planet and a natal planet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransitAspect {
    pub transiting_planet: TransitPlanet,
    pub natal_planet: TransitPlanet,
    pub aspect_type: AspectType,
    /// Actual orb in degrees
    pub orb: f64,
    /// Whether the aspect is applying (getting closer) or separating
    pub is_applying: bool,
    pub nature: AspectNature,
}

/// Sade Sati phases (Saturn's 7.5-year transit over Moon)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SadeSatiPhase {
    /// Saturn in 12th from Moon (rising phase)
    Rising,
    /// Saturn conjunct Moon sign (peak phase)
    Peak,
    /// Saturn in 2nd from Moon (setting phase)
    Setting,
}

impl std::fmt::Display for SadeSatiPhase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Rising => write!(f, "Rising (12th from Moon)"),
            Self::Peak => write!(f, "Peak (conjunct Moon)"),
            Self::Setting => write!(f, "Setting (2nd from Moon)"),
        }
    }
}

/// Sade Sati status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SadeSatiStatus {
    pub is_active: bool,
    pub phase: Option<SadeSatiPhase>,
    pub saturn_sign: ZodiacSign,
    pub moon_sign: ZodiacSign,
}

/// Overall quality of the transit period
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PeriodQuality {
    HighlyFavorable,
    Favorable,
    Mixed,
    Challenging,
    Difficult,
}

impl std::fmt::Display for PeriodQuality {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::HighlyFavorable => write!(f, "Highly Favorable"),
            Self::Favorable => write!(f, "Favorable"),
            Self::Mixed => write!(f, "Mixed"),
            Self::Challenging => write!(f, "Challenging"),
            Self::Difficult => write!(f, "Difficult"),
        }
    }
}

/// Complete transit analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransitAnalysisResult {
    /// Natal planetary positions
    pub natal_positions: Vec<PlanetaryPosition>,
    /// Current transit positions
    pub transit_positions: Vec<PlanetaryPosition>,
    /// Active aspects between transit and natal planets
    pub aspects: Vec<TransitAspect>,
    /// Sade Sati status
    pub sade_sati: SadeSatiStatus,
    /// Overall period quality
    pub period_quality: PeriodQuality,
    /// Retrograde planets in transit
    pub retrograde_planets: Vec<TransitPlanet>,
}
