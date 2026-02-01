//! Face Reading data models
//!
//! Structures representing face analysis results combining:
//! - Chinese Face Reading (Mian Xiang)
//! - Ayurvedic Face Analysis  
//! - Western Physiognomy

use serde::{Deserialize, Serialize};

/// Complete face analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaceAnalysis {
    /// Constitutional typing from facial features
    pub constitution: ConstitutionAnalysis,
    /// Personality traits derived from facial structure
    pub personality_indicators: Vec<PersonalityTrait>,
    /// Five elements balance (TCM)
    pub elemental_balance: ElementalBalance,
    /// Health indicators from facial zones
    pub health_indicators: Vec<HealthIndicator>,
    /// Always true for stub implementation
    pub is_mock_data: bool,
}

/// Constitutional analysis combining Ayurveda and TCM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstitutionAnalysis {
    /// Primary Ayurvedic dosha
    pub primary_dosha: Dosha,
    /// Secondary dosha if applicable
    pub secondary_dosha: Option<Dosha>,
    /// Primary TCM element
    pub tcm_element: Element,
    /// Western body type classification
    pub body_type: BodyType,
}

/// Personality trait derived from facial feature
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonalityTrait {
    /// Name of the trait
    pub trait_name: String,
    /// Facial feature indicating this trait (e.g., "high forehead", "wide-set eyes")
    pub facial_indicator: String,
    /// Description of how this trait manifests
    pub description: String,
}

/// Five elements balance from TCM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementalBalance {
    /// Wood element (liver, growth, creativity)
    pub wood: f64,
    /// Fire element (heart, joy, communication)
    pub fire: f64,
    /// Earth element (spleen, stability, nurturing)
    pub earth: f64,
    /// Metal element (lungs, precision, boundaries)
    pub metal: f64,
    /// Water element (kidneys, wisdom, adaptability)
    pub water: f64,
}

impl ElementalBalance {
    /// Create a new balanced elemental profile
    pub fn balanced() -> Self {
        Self {
            wood: 0.2,
            fire: 0.2,
            earth: 0.2,
            metal: 0.2,
            water: 0.2,
        }
    }

    /// Get the dominant element
    pub fn dominant(&self) -> Element {
        let elements = [
            (self.wood, Element::Wood),
            (self.fire, Element::Fire),
            (self.earth, Element::Earth),
            (self.metal, Element::Metal),
            (self.water, Element::Water),
        ];
        elements
            .into_iter()
            .max_by(|a, b| a.0.partial_cmp(&b.0).unwrap())
            .map(|(_, e)| e)
            .unwrap_or(Element::Earth)
    }

    /// Normalize values to sum to 1.0
    pub fn normalize(&mut self) {
        let sum = self.wood + self.fire + self.earth + self.metal + self.water;
        if sum > 0.0 {
            self.wood /= sum;
            self.fire /= sum;
            self.earth /= sum;
            self.metal /= sum;
            self.water /= sum;
        }
    }
}

/// Health indicator from facial zone observation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthIndicator {
    /// Face zone of observation
    pub zone: FaceZone,
    /// Associated organ system
    pub associated_organ: String,
    /// What was observed
    pub observation: String,
}

/// Face zones used in traditional face reading
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FaceZone {
    Forehead,
    Eyebrows,
    Eyes,
    Nose,
    Cheeks,
    Mouth,
    Chin,
    Ears,
    Jawline,
    Temples,
}

impl FaceZone {
    /// Get all face zones
    pub fn all() -> &'static [FaceZone] {
        &[
            FaceZone::Forehead,
            FaceZone::Eyebrows,
            FaceZone::Eyes,
            FaceZone::Nose,
            FaceZone::Cheeks,
            FaceZone::Mouth,
            FaceZone::Chin,
            FaceZone::Ears,
            FaceZone::Jawline,
            FaceZone::Temples,
        ]
    }

    /// Get display name
    pub fn display_name(&self) -> &'static str {
        match self {
            FaceZone::Forehead => "Forehead",
            FaceZone::Eyebrows => "Eyebrows",
            FaceZone::Eyes => "Eyes",
            FaceZone::Nose => "Nose",
            FaceZone::Cheeks => "Cheeks",
            FaceZone::Mouth => "Mouth",
            FaceZone::Chin => "Chin",
            FaceZone::Ears => "Ears",
            FaceZone::Jawline => "Jawline",
            FaceZone::Temples => "Temples",
        }
    }
}

/// Ayurvedic doshas
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Dosha {
    Vata,
    Pitta,
    Kapha,
}

impl Dosha {
    /// Get display name
    pub fn display_name(&self) -> &'static str {
        match self {
            Dosha::Vata => "Vata",
            Dosha::Pitta => "Pitta",
            Dosha::Kapha => "Kapha",
        }
    }

    /// Get brief description
    pub fn description(&self) -> &'static str {
        match self {
            Dosha::Vata => "Air and space: creative, quick-thinking, adaptable",
            Dosha::Pitta => "Fire and water: driven, focused, transformative",
            Dosha::Kapha => "Earth and water: stable, nurturing, enduring",
        }
    }
}

/// TCM Five Elements
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Element {
    Wood,
    Fire,
    Earth,
    Metal,
    Water,
}

impl Element {
    /// Get display name
    pub fn display_name(&self) -> &'static str {
        match self {
            Element::Wood => "Wood",
            Element::Fire => "Fire",
            Element::Earth => "Earth",
            Element::Metal => "Metal",
            Element::Water => "Water",
        }
    }

    /// Get brief description
    pub fn description(&self) -> &'static str {
        match self {
            Element::Wood => "Growth, creativity, vision, flexibility",
            Element::Fire => "Joy, communication, passion, warmth",
            Element::Earth => "Stability, nurturing, grounding, balance",
            Element::Metal => "Precision, boundaries, refinement, letting go",
            Element::Water => "Wisdom, adaptability, depth, flow",
        }
    }

    /// Get associated organ (TCM)
    pub fn associated_organ(&self) -> &'static str {
        match self {
            Element::Wood => "Liver/Gallbladder",
            Element::Fire => "Heart/Small Intestine",
            Element::Earth => "Spleen/Stomach",
            Element::Metal => "Lungs/Large Intestine",
            Element::Water => "Kidneys/Bladder",
        }
    }
}

/// Western body type classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BodyType {
    Ectomorph,
    Mesomorph,
    Endomorph,
}

impl BodyType {
    /// Get display name
    pub fn display_name(&self) -> &'static str {
        match self {
            BodyType::Ectomorph => "Ectomorph",
            BodyType::Mesomorph => "Mesomorph",
            BodyType::Endomorph => "Endomorph",
        }
    }

    /// Get brief description
    pub fn description(&self) -> &'static str {
        match self {
            BodyType::Ectomorph => "Lean, long-limbed, fine-featured",
            BodyType::Mesomorph => "Athletic, medium build, defined features",
            BodyType::Endomorph => "Soft, rounded features, solid build",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_elemental_balance_dominant() {
        let balance = ElementalBalance {
            wood: 0.3,
            fire: 0.2,
            earth: 0.15,
            metal: 0.15,
            water: 0.2,
        };
        assert_eq!(balance.dominant(), Element::Wood);
    }

    #[test]
    fn test_elemental_balance_normalize() {
        let mut balance = ElementalBalance {
            wood: 2.0,
            fire: 2.0,
            earth: 2.0,
            metal: 2.0,
            water: 2.0,
        };
        balance.normalize();
        assert!((balance.wood - 0.2).abs() < 0.001);
        assert!((balance.fire - 0.2).abs() < 0.001);
    }

    #[test]
    fn test_face_zone_all() {
        assert_eq!(FaceZone::all().len(), 10);
    }

    #[test]
    fn test_dosha_descriptions() {
        assert!(!Dosha::Vata.description().is_empty());
        assert!(!Dosha::Pitta.description().is_empty());
        assert!(!Dosha::Kapha.description().is_empty());
    }

    #[test]
    fn test_element_organs() {
        assert!(Element::Wood.associated_organ().contains("Liver"));
        assert!(Element::Water.associated_organ().contains("Kidney"));
    }
}
