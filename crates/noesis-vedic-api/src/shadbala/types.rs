//! Shadbala types
//!
//! FAPI-067: Define Shadbala types

use serde::{Deserialize, Serialize};

/// The six sources of planetary strength (Shadbala)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ShadbalaComponent {
    /// Sthana Bala - Positional strength
    SthanaBala,
    /// Dig Bala - Directional strength  
    DigBala,
    /// Kala Bala - Temporal strength
    KalaBala,
    /// Chesta Bala - Motional strength
    ChestaBala,
    /// Naisargika Bala - Natural strength
    NaisargikaBala,
    /// Drik Bala - Aspectual strength
    DrikBala,
}

impl std::fmt::Display for ShadbalaComponent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ShadbalaComponent::SthanaBala => write!(f, "Sthana Bala"),
            ShadbalaComponent::DigBala => write!(f, "Dig Bala"),
            ShadbalaComponent::KalaBala => write!(f, "Kala Bala"),
            ShadbalaComponent::ChestaBala => write!(f, "Chesta Bala"),
            ShadbalaComponent::NaisargikaBala => write!(f, "Naisargika Bala"),
            ShadbalaComponent::DrikBala => write!(f, "Drik Bala"),
        }
    }
}

/// Shadbala value for a single component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShadbalaValue {
    /// Component type
    pub component: ShadbalaComponent,
    /// Value in Rupas
    pub rupas: f64,
    /// Value in Shashtiamsas (60ths)
    pub shashtiamsas: f64,
}

/// Complete Shadbala for a planet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanetShadbala {
    /// Planet name
    pub planet: String,
    /// Individual component values
    pub components: Vec<ShadbalaValue>,
    /// Total Shadbala in Rupas
    pub total_rupas: f64,
    /// Total in Shashtiamsas
    pub total_shashtiamsas: f64,
    /// Required minimum (varies by planet)
    pub required_minimum: f64,
    /// Ratio (actual/required)
    pub strength_ratio: f64,
    /// Is the planet strong? (ratio >= 1.0)
    pub is_strong: bool,
}

impl PlanetShadbala {
    /// Get a specific component value
    pub fn get_component(&self, component: ShadbalaComponent) -> Option<&ShadbalaValue> {
        self.components.iter().find(|c| c.component == component)
    }
    
    /// Calculate strength percentage
    pub fn strength_percentage(&self) -> f64 {
        self.strength_ratio * 100.0
    }
}

/// Complete Shadbala analysis for all planets
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShadbalaAnalysis {
    /// Shadbala for each planet
    pub planets: Vec<PlanetShadbala>,
    /// Strongest planet
    pub strongest_planet: String,
    /// Weakest planet
    pub weakest_planet: String,
    /// Overall chart strength
    pub chart_strength: ChartStrength,
}

/// Overall chart strength assessment
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChartStrength {
    VeryStrong,
    Strong,
    Average,
    Weak,
    VeryWeak,
}

impl ChartStrength {
    pub fn from_average_ratio(ratio: f64) -> Self {
        if ratio >= 1.5 {
            ChartStrength::VeryStrong
        } else if ratio >= 1.2 {
            ChartStrength::Strong
        } else if ratio >= 0.9 {
            ChartStrength::Average
        } else if ratio >= 0.6 {
            ChartStrength::Weak
        } else {
            ChartStrength::VeryWeak
        }
    }
}

impl std::fmt::Display for ChartStrength {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChartStrength::VeryStrong => write!(f, "Very Strong"),
            ChartStrength::Strong => write!(f, "Strong"),
            ChartStrength::Average => write!(f, "Average"),
            ChartStrength::Weak => write!(f, "Weak"),
            ChartStrength::VeryWeak => write!(f, "Very Weak"),
        }
    }
}

/// Required minimum Shadbala for each planet (in Rupas)
pub fn required_shadbala(planet: &str) -> f64 {
    match planet.to_lowercase().as_str() {
        "sun" => 390.0,
        "moon" => 360.0,
        "mars" => 300.0,
        "mercury" => 420.0,
        "jupiter" => 390.0,
        "venus" => 330.0,
        "saturn" => 300.0,
        _ => 300.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shadbala_component_display() {
        assert_eq!(ShadbalaComponent::SthanaBala.to_string(), "Sthana Bala");
    }

    #[test]
    fn test_chart_strength_from_ratio() {
        assert_eq!(ChartStrength::from_average_ratio(1.6), ChartStrength::VeryStrong);
        assert_eq!(ChartStrength::from_average_ratio(1.0), ChartStrength::Average);
        assert_eq!(ChartStrength::from_average_ratio(0.5), ChartStrength::VeryWeak);
    }

    #[test]
    fn test_required_shadbala() {
        assert_eq!(required_shadbala("Sun"), 390.0);
        assert_eq!(required_shadbala("Mercury"), 420.0);
    }
}
