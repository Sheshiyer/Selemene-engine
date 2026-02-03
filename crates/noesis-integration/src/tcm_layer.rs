//! Traditional Chinese Medicine (TCM) Layering on Vedic Astrology
//!
//! This module provides TCM analysis that layers on top of Vedic astrological data,
//! creating a bridge between Eastern medical astrology and Vedic planetary influences.

use chrono::Datelike;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{BirthProfile, Result};

/// TCM Analysis combining elemental, organ, and meridian systems
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TCMAnalysis {
    /// Dominant element based on birth chart
    pub dominant_element: TCMElement,
    /// Secondary element
    pub secondary_element: TCMElement,
    /// Element balance score (0.0-1.0 for each element)
    pub element_balance: HashMap<TCMElement, f64>,
    /// Organs that need attention
    pub vulnerable_organs: Vec<TCMOrgan>,
    /// Strong organs
    pub strong_organs: Vec<TCMOrgan>,
    /// Seasonal influences
    pub seasonal_influence: SeasonalInfluence,
    /// Time-of-day recommendations
    pub optimal_times: Vec<OptimalTimeWindow>,
    /// Constitutional type
    pub constitution: ConstitutionalType,
    /// Lifestyle recommendations
    pub recommendations: Vec<TCMRecommendation>,
}

/// Five Elements (Wu Xing)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TCMElement {
    Wood,   // Growth, expansion (Jupiter)
    Fire,   // Transformation, passion (Sun, Mars)
    Earth,  // Nurturing, stability (Saturn)
    Metal,  // Structure, clarity (Venus)
    Water,  // Flow, wisdom (Moon, Mercury)
}

impl TCMElement {
    /// Get the Vedic planets associated with this element
    pub fn vedic_planets(&self) -> Vec<&'static str> {
        match self {
            TCMElement::Wood => vec!["Jupiter"],
            TCMElement::Fire => vec!["Sun", "Mars"],
            TCMElement::Earth => vec!["Saturn"],
            TCMElement::Metal => vec!["Venus"],
            TCMElement::Water => vec!["Moon", "Mercury"],
        }
    }
    
    /// Get the season associated with this element
    pub fn season(&self) -> &'static str {
        match self {
            TCMElement::Wood => "Spring",
            TCMElement::Fire => "Summer",
            TCMElement::Earth => "Late Summer",
            TCMElement::Metal => "Autumn",
            TCMElement::Water => "Winter",
        }
    }
    
    /// Get the direction
    pub fn direction(&self) -> &'static str {
        match self {
            TCMElement::Wood => "East",
            TCMElement::Fire => "South",
            TCMElement::Earth => "Center",
            TCMElement::Metal => "West",
            TCMElement::Water => "North",
        }
    }
    
    /// Get the color
    pub fn color(&self) -> &'static str {
        match self {
            TCMElement::Wood => "Green",
            TCMElement::Fire => "Red",
            TCMElement::Earth => "Yellow",
            TCMElement::Metal => "White",
            TCMElement::Water => "Black/Blue",
        }
    }
    
    /// Get the emotion
    pub fn emotion(&self) -> &'static str {
        match self {
            TCMElement::Wood => "Anger/Courage",
            TCMElement::Fire => "Joy/Excitement",
            TCMElement::Earth => "Worry/Reflection",
            TCMElement::Metal => "Grief/Release",
            TCMElement::Water => "Fear/Wisdom",
        }
    }
    
    /// Get taste
    pub fn taste(&self) -> &'static str {
        match self {
            TCMElement::Wood => "Sour",
            TCMElement::Fire => "Bitter",
            TCMElement::Earth => "Sweet",
            TCMElement::Metal => "Pungent/Spicy",
            TCMElement::Water => "Salty",
        }
    }
    
    /// Get element name as string
    pub fn as_str(&self) -> &'static str {
        match self {
            TCMElement::Wood => "Wood",
            TCMElement::Fire => "Fire",
            TCMElement::Earth => "Earth",
            TCMElement::Metal => "Metal",
            TCMElement::Water => "Water",
        }
    }
    
    /// Get generating element (mother)
    pub fn generating_element(&self) -> TCMElement {
        match self {
            TCMElement::Wood => TCMElement::Water,
            TCMElement::Fire => TCMElement::Wood,
            TCMElement::Earth => TCMElement::Fire,
            TCMElement::Metal => TCMElement::Earth,
            TCMElement::Water => TCMElement::Metal,
        }
    }
    
    /// Get controlling element (grandmother)
    pub fn controlling_element(&self) -> TCMElement {
        match self {
            TCMElement::Wood => TCMElement::Earth,
            TCMElement::Fire => TCMElement::Metal,
            TCMElement::Earth => TCMElement::Water,
            TCMElement::Metal => TCMElement::Wood,
            TCMElement::Water => TCMElement::Fire,
        }
    }
    
    /// Get generated element (child)
    pub fn generated_element(&self) -> TCMElement {
        match self {
            TCMElement::Wood => TCMElement::Fire,
            TCMElement::Fire => TCMElement::Earth,
            TCMElement::Earth => TCMElement::Metal,
            TCMElement::Metal => TCMElement::Water,
            TCMElement::Water => TCMElement::Wood,
        }
    }
}

/// TCM Organs (Zang-Fu system)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TCMOrgan {
    // Yin organs (Zang) - solid, store essence
    Heart,
    Liver,
    Spleen,
    Lungs,
    Kidneys,
    Pericardium,
    
    // Yang organs (Fu) - hollow, transform substances
    SmallIntestine,
    GallBladder,
    Stomach,
    LargeIntestine,
    Bladder,
    TripleBurner,
}

impl TCMOrgan {
    /// Get the element associated with this organ
    pub fn element(&self) -> TCMElement {
        match self {
            TCMOrgan::Liver | TCMOrgan::GallBladder => TCMElement::Wood,
            TCMOrgan::Heart | TCMOrgan::SmallIntestine | TCMOrgan::Pericardium | TCMOrgan::TripleBurner => TCMElement::Fire,
            TCMOrgan::Spleen | TCMOrgan::Stomach => TCMElement::Earth,
            TCMOrgan::Lungs | TCMOrgan::LargeIntestine => TCMElement::Metal,
            TCMOrgan::Kidneys | TCMOrgan::Bladder => TCMElement::Water,
        }
    }
    
    /// Check if this is a Yin organ
    pub fn is_yin(&self) -> bool {
        matches!(self,
            TCMOrgan::Heart |
            TCMOrgan::Liver |
            TCMOrgan::Spleen |
            TCMOrgan::Lungs |
            TCMOrgan::Kidneys |
            TCMOrgan::Pericardium
        )
    }
    
    /// Check if this is a Yang organ
    pub fn is_yang(&self) -> bool {
        !self.is_yin()
    }
    
    /// Get the meridian hours (when this organ is most active)
    pub fn peak_hours(&self) -> (u8, u8) {
        match self {
            TCMOrgan::Lungs => (3, 5),          // 3-5 AM
            TCMOrgan::LargeIntestine => (5, 7), // 5-7 AM
            TCMOrgan::Stomach => (7, 9),        // 7-9 AM
            TCMOrgan::Spleen => (9, 11),        // 9-11 AM
            TCMOrgan::Heart => (11, 13),        // 11 AM-1 PM
            TCMOrgan::SmallIntestine => (13, 15), // 1-3 PM
            TCMOrgan::Bladder => (15, 17),      // 3-5 PM
            TCMOrgan::Kidneys => (17, 19),      // 5-7 PM
            TCMOrgan::Pericardium => (19, 21),  // 7-9 PM
            TCMOrgan::TripleBurner => (21, 23), // 9-11 PM
            TCMOrgan::GallBladder => (23, 1),   // 11 PM-1 AM
            TCMOrgan::Liver => (1, 3),          // 1-3 AM
        }
    }
    
    /// Get paired organ (Yin-Yang pair)
    pub fn paired_organ(&self) -> TCMOrgan {
        match self {
            TCMOrgan::Heart => TCMOrgan::SmallIntestine,
            TCMOrgan::SmallIntestine => TCMOrgan::Heart,
            TCMOrgan::Liver => TCMOrgan::GallBladder,
            TCMOrgan::GallBladder => TCMOrgan::Liver,
            TCMOrgan::Spleen => TCMOrgan::Stomach,
            TCMOrgan::Stomach => TCMOrgan::Spleen,
            TCMOrgan::Lungs => TCMOrgan::LargeIntestine,
            TCMOrgan::LargeIntestine => TCMOrgan::Lungs,
            TCMOrgan::Kidneys => TCMOrgan::Bladder,
            TCMOrgan::Bladder => TCMOrgan::Kidneys,
            TCMOrgan::Pericardium => TCMOrgan::TripleBurner,
            TCMOrgan::TripleBurner => TCMOrgan::Pericardium,
        }
    }
}

/// Seasonal influence on constitution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeasonalInfluence {
    pub current_season: String,
    pub dominant_element: TCMElement,
    pub vulnerable_organs: Vec<TCMOrgan>,
    pub recommendations: Vec<String>,
}

/// Optimal time window for activities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimalTimeWindow {
    pub activity_type: String,
    pub start_hour: u8,
    pub end_hour: u8,
    pub associated_organ: TCMOrgan,
    pub description: String,
}

/// Constitutional type (9 types in TCM)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConstitutionalType {
    Balanced,           // Neutral
    QiDeficient,        // Earth/Metal weakness
    YangDeficient,      // Fire/Kidney yang weakness
    YinDeficient,       // Water/Kidney yin weakness
    PhlegmDampness,     // Earth/Spleen excess
    DampHeat,           // Earth/Liver damp heat
    BloodStasis,        // Heart/Liver stagnation
    QiStagnation,       // Liver Wood stagnation
    SpecialConstitution, // Unique pattern
}

/// TCM Recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TCMRecommendation {
    pub category: RecommendationCategory,
    pub description: String,
    pub priority: Priority,
    pub element: Option<TCMElement>,
}

/// Category of recommendation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecommendationCategory {
    Diet,
    Exercise,
    Sleep,
    Meditation,
    Acupressure,
    Herbal,
    Lifestyle,
    Seasonal,
}

/// Priority level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Priority {
    Low,
    Medium,
    High,
    Critical,
}

impl TCMAnalysis {
    /// Generate TCM analysis from birth profile
    pub fn from_birth_profile(profile: &BirthProfile) -> Result<Self> {
        // Parse birth date to determine seasonal influence
        let date = profile.parse_date()?;
        let month = date.month();
        
        // Determine dominant element based on birth month and other factors
        let (dominant, secondary) = Self::calculate_elements(month);
        
        // Calculate element balance
        let mut element_balance = HashMap::new();
        element_balance.insert(TCMElement::Wood, Self::element_score(month, TCMElement::Wood));
        element_balance.insert(TCMElement::Fire, Self::element_score(month, TCMElement::Fire));
        element_balance.insert(TCMElement::Earth, Self::element_score(month, TCMElement::Earth));
        element_balance.insert(TCMElement::Metal, Self::element_score(month, TCMElement::Metal));
        element_balance.insert(TCMElement::Water, Self::element_score(month, TCMElement::Water));
        
        // Determine vulnerable and strong organs
        let (vulnerable, strong) = Self::determine_organ_balance(&dominant, &element_balance);
        
        // Generate recommendations
        let recommendations = Self::generate_recommendations(&dominant, &vulnerable);
        
        Ok(TCMAnalysis {
            dominant_element: dominant,
            secondary_element: secondary,
            element_balance,
            vulnerable_organs: vulnerable.clone(),
            strong_organs: strong,
            seasonal_influence: Self::calculate_seasonal_influence(month),
            optimal_times: Self::calculate_optimal_times(&dominant),
            constitution: Self::determine_constitution(&dominant, &vulnerable),
            recommendations,
        })
    }
    
    /// Calculate dominant and secondary elements
    fn calculate_elements(birth_month: u32) -> (TCMElement, TCMElement) {
        match birth_month {
            2 | 3 | 4 => (TCMElement::Wood, TCMElement::Fire),      // Spring
            5 | 6 | 7 => (TCMElement::Fire, TCMElement::Earth),      // Summer
            8 => (TCMElement::Earth, TCMElement::Metal),             // Late summer
            9 | 10 | 11 => (TCMElement::Metal, TCMElement::Water),   // Autumn
            12 | 1 => (TCMElement::Water, TCMElement::Wood),         // Winter
            _ => (TCMElement::Earth, TCMElement::Metal),
        }
    }
    
    /// Calculate element score based on birth month
    fn element_score(birth_month: u32, element: TCMElement) -> f64 {
        let dominant = Self::calculate_elements(birth_month);
        
        if element == dominant.0 {
            0.8
        } else if element == dominant.1 {
            0.6
        } else if element == dominant.0.controlling_element() {
            0.3 // Controlling element is weaker
        } else {
            0.5
        }
    }
    
    /// Determine organ balance
    fn determine_organ_balance(
        dominant: &TCMElement,
        _balance: &HashMap<TCMElement, f64>,
    ) -> (Vec<TCMOrgan>, Vec<TCMOrgan>) {
        let mut vulnerable = Vec::new();
        let mut strong = Vec::new();
        
        // Organs of the dominant element are generally strong
        match dominant {
            TCMElement::Wood => {
                strong.push(TCMOrgan::Liver);
                strong.push(TCMOrgan::GallBladder);
                vulnerable.push(TCMOrgan::Lungs); // Controlled by Wood
            }
            TCMElement::Fire => {
                strong.push(TCMOrgan::Heart);
                strong.push(TCMOrgan::SmallIntestine);
                vulnerable.push(TCMOrgan::Kidneys); // Controlled by Fire
            }
            TCMElement::Earth => {
                strong.push(TCMOrgan::Spleen);
                strong.push(TCMOrgan::Stomach);
                vulnerable.push(TCMOrgan::Liver); // Controlled by Earth
            }
            TCMElement::Metal => {
                strong.push(TCMOrgan::Lungs);
                strong.push(TCMOrgan::LargeIntestine);
                vulnerable.push(TCMOrgan::Heart); // Controlled by Metal
            }
            TCMElement::Water => {
                strong.push(TCMOrgan::Kidneys);
                strong.push(TCMOrgan::Bladder);
                vulnerable.push(TCMOrgan::Spleen); // Controlled by Water
            }
        }
        
        (vulnerable, strong)
    }
    
    /// Calculate seasonal influence
    fn calculate_seasonal_influence(month: u32) -> SeasonalInfluence {
        let (season, element) = match month {
            3..=5 => ("Spring", TCMElement::Wood),
            6..=8 => ("Summer", TCMElement::Fire),
            9 => ("Late Summer", TCMElement::Earth),
            10..=11 => ("Autumn", TCMElement::Metal),
            12 | 1 | 2 => ("Winter", TCMElement::Water),
            _ => ("Unknown", TCMElement::Earth),
        };
        
        let vulnerable = match element {
            TCMElement::Wood => vec![TCMOrgan::Liver, TCMOrgan::GallBladder],
            TCMElement::Fire => vec![TCMOrgan::Heart, TCMOrgan::SmallIntestine],
            TCMElement::Earth => vec![TCMOrgan::Spleen, TCMOrgan::Stomach],
            TCMElement::Metal => vec![TCMOrgan::Lungs, TCMOrgan::LargeIntestine],
            TCMElement::Water => vec![TCMOrgan::Kidneys, TCMOrgan::Bladder],
        };
        
        let recommendations = vec![
            format!("Focus on {} element foods and activities", element.as_str()),
            format!("Support your {} organs", element.as_str()),
        ];
        
        SeasonalInfluence {
            current_season: season.to_string(),
            dominant_element: element,
            vulnerable_organs: vulnerable,
            recommendations,
        }
    }
    
    /// Calculate optimal times based on dominant element
    fn calculate_optimal_times(dominant: &TCMElement) -> Vec<OptimalTimeWindow> {
        let mut times = Vec::new();
        
        // Add time windows for the dominant element's organs
        match dominant {
            TCMElement::Wood => {
                times.push(OptimalTimeWindow {
                    activity_type: "Physical exercise".to_string(),
                    start_hour: 1,
                    end_hour: 3,
                    associated_organ: TCMOrgan::Liver,
                    description: "Liver time - good for planning and decision making".to_string(),
                });
            }
            TCMElement::Fire => {
                times.push(OptimalTimeWindow {
                    activity_type: "Social activities".to_string(),
                    start_hour: 11,
                    end_hour: 13,
                    associated_organ: TCMOrgan::Heart,
                    description: "Heart time - best for connection and joy".to_string(),
                });
            }
            TCMElement::Earth => {
                times.push(OptimalTimeWindow {
                    activity_type: "Digestion and nourishment".to_string(),
                    start_hour: 7,
                    end_hour: 11,
                    associated_organ: TCMOrgan::Stomach,
                    description: "Stomach and Spleen time - best for eating and studying".to_string(),
                });
            }
            TCMElement::Metal => {
                times.push(OptimalTimeWindow {
                    activity_type: "Mental work".to_string(),
                    start_hour: 3,
                    end_hour: 7,
                    associated_organ: TCMOrgan::Lungs,
                    description: "Lung time - best for breathing exercises and organization".to_string(),
                });
            }
            TCMElement::Water => {
                times.push(OptimalTimeWindow {
                    activity_type: "Rest and meditation".to_string(),
                    start_hour: 17,
                    end_hour: 19,
                    associated_organ: TCMOrgan::Kidneys,
                    description: "Kidney time - best for rest and rejuvenation".to_string(),
                });
            }
        }
        
        times
    }
    
    /// Determine constitutional type
    fn determine_constitution(
        dominant: &TCMElement,
        vulnerable: &[TCMOrgan],
    ) -> ConstitutionalType {
        // Simplified constitution determination
        match dominant {
            TCMElement::Wood => {
                if vulnerable.contains(&TCMOrgan::Liver) {
                    ConstitutionalType::QiStagnation
                } else {
                    ConstitutionalType::Balanced
                }
            }
            TCMElement::Fire => {
                if vulnerable.contains(&TCMOrgan::Heart) {
                    ConstitutionalType::YinDeficient
                } else {
                    ConstitutionalType::Balanced
                }
            }
            TCMElement::Earth => {
                if vulnerable.contains(&TCMOrgan::Spleen) {
                    ConstitutionalType::PhlegmDampness
                } else {
                    ConstitutionalType::Balanced
                }
            }
            TCMElement::Metal => {
                if vulnerable.contains(&TCMOrgan::Lungs) {
                    ConstitutionalType::QiDeficient
                } else {
                    ConstitutionalType::Balanced
                }
            }
            TCMElement::Water => {
                if vulnerable.contains(&TCMOrgan::Kidneys) {
                    ConstitutionalType::YangDeficient
                } else {
                    ConstitutionalType::Balanced
                }
            }
        }
    }
    
    /// Generate recommendations based on TCM analysis
    fn generate_recommendations(
        dominant: &TCMElement,
        vulnerable: &[TCMOrgan],
    ) -> Vec<TCMRecommendation> {
        let mut recommendations = Vec::new();
        
        // Diet recommendations
        recommendations.push(TCMRecommendation {
            category: RecommendationCategory::Diet,
            description: format!("Include more {}-flavored foods", dominant.taste()),
            priority: Priority::High,
            element: Some(*dominant),
        });
        
        // Exercise recommendations
        let exercise_rec = match dominant {
            TCMElement::Wood => "Gentle stretching and outdoor activities",
            TCMElement::Fire => "Moderate cardio and social sports",
            TCMElement::Earth => "Walking and grounding exercises",
            TCMElement::Metal => "Breathing exercises and yoga",
            TCMElement::Water => "Swimming and flowing movements",
        };
        recommendations.push(TCMRecommendation {
            category: RecommendationCategory::Exercise,
            description: exercise_rec.to_string(),
            priority: Priority::Medium,
            element: Some(*dominant),
        });
        
        // Sleep recommendations
        if vulnerable.iter().any(|o| *o == TCMOrgan::Kidneys || *o == TCMOrgan::Liver) {
            recommendations.push(TCMRecommendation {
                category: RecommendationCategory::Sleep,
                description: "Prioritize sleep before 11 PM to support Kidney and Liver restoration".to_string(),
                priority: Priority::High,
                element: Some(TCMElement::Water),
            });
        }
        
        recommendations
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_element_relationships() {
        // Wood generates Fire
        assert_eq!(TCMElement::Wood.generated_element(), TCMElement::Fire);
        
        // Fire is controlled by Water
        assert_eq!(TCMElement::Fire.controlling_element(), TCMElement::Water);
        
        // Water is generated by Metal
        assert_eq!(TCMElement::Metal.generated_element(), TCMElement::Water);
    }

    #[test]
    fn test_organ_element_association() {
        assert_eq!(TCMOrgan::Liver.element(), TCMElement::Wood);
        assert_eq!(TCMOrgan::Heart.element(), TCMElement::Fire);
        assert_eq!(TCMOrgan::Spleen.element(), TCMElement::Earth);
        assert_eq!(TCMOrgan::Lungs.element(), TCMElement::Metal);
        assert_eq!(TCMOrgan::Kidneys.element(), TCMElement::Water);
    }

    #[test]
    fn test_organ_peak_hours() {
        let (start, end) = TCMOrgan::Liver.peak_hours();
        assert_eq!(start, 1);
        assert_eq!(end, 3);
        
        let (start, end) = TCMOrgan::Heart.peak_hours();
        assert_eq!(start, 11);
        assert_eq!(end, 13);
    }

    #[test]
    fn test_tcm_analysis_from_profile() {
        let profile = BirthProfile::new(
            "1991-08-13",  // August - Summer/Fire
            "13:31",
            12.9716,
            77.5946,
            "Asia/Kolkata",
        );
        
        let analysis = TCMAnalysis::from_birth_profile(&profile).unwrap();
        
        // August should give Fire as dominant
        assert_eq!(analysis.dominant_element, TCMElement::Fire);
        assert_eq!(analysis.seasonal_influence.current_season, "Summer");
        assert!(!analysis.recommendations.is_empty());
    }
}
