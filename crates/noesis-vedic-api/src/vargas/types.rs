//! Varga (Divisional Chart) types
//!
//! FAPI-056: Define general Varga (divisional chart) types

use serde::{Deserialize, Serialize};
use super::navamsa_types::NavamsaPosition;

/// Available divisional charts
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum VargaType {
    /// D1 - Rashi (Birth Chart)
    Rashi,
    /// D2 - Hora (Wealth)
    Hora,
    /// D3 - Drekkana (Siblings)
    Drekkana,
    /// D4 - Chaturthamsa (Fortune, Property)
    Chaturthamsa,
    /// D7 - Saptamsa (Children)
    Saptamsa,
    /// D9 - Navamsa (Marriage, Dharma)
    Navamsa,
    /// D10 - Dasamsa (Career)
    Dasamsa,
    /// D12 - Dwadasamsa (Parents)
    Dwadasamsa,
    /// D16 - Shodasamsa (Vehicles, Happiness)
    Shodasamsa,
    /// D20 - Vimsamsa (Spiritual Progress)
    Vimsamsa,
    /// D24 - Chaturvimsamsa (Education)
    Chaturvimsamsa,
    /// D27 - Saptavimsamsa (Strength)
    Saptavimsamsa,
    /// D30 - Trimsamsa (Misfortunes)
    Trimsamsa,
    /// D40 - Khavedamsa (Auspicious Effects)
    Khavedamsa,
    /// D45 - Akshavedamsa (General Indications)
    Akshavedamsa,
    /// D60 - Shashtiamsa (Past Life Karma)
    Shashtiamsa,
}

impl VargaType {
    /// Get the divisor number
    pub fn divisor(&self) -> u8 {
        match self {
            VargaType::Rashi => 1,
            VargaType::Hora => 2,
            VargaType::Drekkana => 3,
            VargaType::Chaturthamsa => 4,
            VargaType::Saptamsa => 7,
            VargaType::Navamsa => 9,
            VargaType::Dasamsa => 10,
            VargaType::Dwadasamsa => 12,
            VargaType::Shodasamsa => 16,
            VargaType::Vimsamsa => 20,
            VargaType::Chaturvimsamsa => 24,
            VargaType::Saptavimsamsa => 27,
            VargaType::Trimsamsa => 30,
            VargaType::Khavedamsa => 40,
            VargaType::Akshavedamsa => 45,
            VargaType::Shashtiamsa => 60,
        }
    }

    /// Get the primary signification
    pub fn signification(&self) -> &'static str {
        match self {
            VargaType::Rashi => "Overall life, physical body",
            VargaType::Hora => "Wealth and financial matters",
            VargaType::Drekkana => "Siblings, courage, efforts",
            VargaType::Chaturthamsa => "Fortune, property, vehicles",
            VargaType::Saptamsa => "Children, progeny",
            VargaType::Navamsa => "Marriage, spouse, dharma",
            VargaType::Dasamsa => "Career, profession, status",
            VargaType::Dwadasamsa => "Parents, ancestral lineage",
            VargaType::Shodasamsa => "Vehicles, conveyances, happiness",
            VargaType::Vimsamsa => "Spiritual progress, worship",
            VargaType::Chaturvimsamsa => "Education, learning",
            VargaType::Saptavimsamsa => "Physical strength, stamina",
            VargaType::Trimsamsa => "Misfortunes, difficulties",
            VargaType::Khavedamsa => "Auspicious effects",
            VargaType::Akshavedamsa => "General indications",
            VargaType::Shashtiamsa => "Past life karma, overall fruits",
        }
    }
}

impl std::fmt::Display for VargaType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VargaType::Rashi => write!(f, "D1 (Rashi)"),
            VargaType::Hora => write!(f, "D2 (Hora)"),
            VargaType::Drekkana => write!(f, "D3 (Drekkana)"),
            VargaType::Chaturthamsa => write!(f, "D4 (Chaturthamsa)"),
            VargaType::Saptamsa => write!(f, "D7 (Saptamsa)"),
            VargaType::Navamsa => write!(f, "D9 (Navamsa)"),
            VargaType::Dasamsa => write!(f, "D10 (Dasamsa)"),
            VargaType::Dwadasamsa => write!(f, "D12 (Dwadasamsa)"),
            VargaType::Shodasamsa => write!(f, "D16 (Shodasamsa)"),
            VargaType::Vimsamsa => write!(f, "D20 (Vimsamsa)"),
            VargaType::Chaturvimsamsa => write!(f, "D24 (Chaturvimsamsa)"),
            VargaType::Saptavimsamsa => write!(f, "D27 (Saptavimsamsa)"),
            VargaType::Trimsamsa => write!(f, "D30 (Trimsamsa)"),
            VargaType::Khavedamsa => write!(f, "D40 (Khavedamsa)"),
            VargaType::Akshavedamsa => write!(f, "D45 (Akshavedamsa)"),
            VargaType::Shashtiamsa => write!(f, "D60 (Shashtiamsa)"),
        }
    }
}

/// Position in a divisional chart
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VargaPosition {
    /// The varga type
    pub varga: VargaType,
    /// Planet name
    pub planet: String,
    /// Sign in this varga
    pub sign: String,
    /// Sign number (1-12)
    pub sign_number: u8,
    /// Degree in sign (for some vargas)
    pub degree: Option<f64>,
}

/// Complete divisional chart
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VargaChart {
    /// Type of this varga
    pub varga_type: VargaType,
    /// All planet positions
    pub positions: Vec<VargaPosition>,
    /// Ascendant sign
    pub ascendant_sign: String,
    /// Ascendant sign number
    pub ascendant_sign_number: u8,
}

/// Varga Bala (Divisional Chart Strength) component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VargaBala {
    /// Planet name
    pub planet: String,
    /// Individual varga points
    pub varga_points: Vec<VargaPoint>,
    /// Total varga bala
    pub total_bala: f64,
    /// Varga vishwa (out of 20)
    pub varga_vishwa: f64,
}

/// Points for a single varga
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VargaPoint {
    pub varga: VargaType,
    pub sign: String,
    pub dignity: String,
    pub points: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_varga_divisor() {
        assert_eq!(VargaType::Rashi.divisor(), 1);
        assert_eq!(VargaType::Navamsa.divisor(), 9);
        assert_eq!(VargaType::Dasamsa.divisor(), 10);
        assert_eq!(VargaType::Shashtiamsa.divisor(), 60);
    }

    #[test]
    fn test_varga_display() {
        assert_eq!(VargaType::Navamsa.to_string(), "D9 (Navamsa)");
        assert_eq!(VargaType::Dasamsa.to_string(), "D10 (Dasamsa)");
    }
}
