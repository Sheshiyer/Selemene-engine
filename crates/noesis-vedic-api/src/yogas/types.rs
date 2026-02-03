//! Yoga detection types
//!
//! FAPI-063: Define Yoga detection types

use serde::{Deserialize, Serialize};

/// Yoga categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum YogaCategory {
    /// Raj Yogas - Power, authority, success
    RajYoga,
    /// Dhana Yogas - Wealth
    DhanaYoga,
    /// Pancha Mahapurusha - Great person yogas
    MahapurushaYoga,
    /// Arishta Yogas - Afflictions
    ArishtaYoga,
    /// Nabhasa Yogas - Planetary patterns
    NabhasaYoga,
    /// Chandra Yogas - Moon-based
    ChandraYoga,
    /// Surya Yogas - Sun-based
    SuryaYoga,
    /// Other beneficial yogas
    ShubhaYoga,
    /// Other afflicting yogas
    PapaYoga,
}

impl std::fmt::Display for YogaCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            YogaCategory::RajYoga => write!(f, "Raj Yoga"),
            YogaCategory::DhanaYoga => write!(f, "Dhana Yoga"),
            YogaCategory::MahapurushaYoga => write!(f, "Mahapurusha Yoga"),
            YogaCategory::ArishtaYoga => write!(f, "Arishta Yoga"),
            YogaCategory::NabhasaYoga => write!(f, "Nabhasa Yoga"),
            YogaCategory::ChandraYoga => write!(f, "Chandra Yoga"),
            YogaCategory::SuryaYoga => write!(f, "Surya Yoga"),
            YogaCategory::ShubhaYoga => write!(f, "Shubha Yoga"),
            YogaCategory::PapaYoga => write!(f, "Papa Yoga"),
        }
    }
}

/// Yoga strength indicator
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum YogaStrength {
    /// Full strength, all conditions met
    Full,
    /// Partial, some conditions met
    Partial,
    /// Weak, minimal conditions
    Weak,
    /// Cancelled by other factors
    Cancelled,
}

/// A detected yoga in a chart
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedYoga {
    /// Name of the yoga
    pub name: String,
    /// Category
    pub category: YogaCategory,
    /// Strength of the yoga
    pub strength: YogaStrength,
    /// Planets involved
    pub planets_involved: Vec<String>,
    /// Houses involved
    pub houses_involved: Vec<u8>,
    /// Brief description
    pub description: String,
    /// Expected results
    pub results: String,
    /// Period when yoga gives results (typically dasha periods)
    pub activation_periods: Vec<String>,
}

/// Collection of all yogas in a chart
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YogaAnalysis {
    /// All detected yogas
    pub yogas: Vec<DetectedYoga>,
    /// Raj Yogas specifically
    pub raj_yogas: Vec<DetectedYoga>,
    /// Dhana Yogas specifically  
    pub dhana_yogas: Vec<DetectedYoga>,
    /// Mahapurusha Yogas
    pub mahapurusha_yogas: Vec<DetectedYoga>,
    /// Negative yogas
    pub arishta_yogas: Vec<DetectedYoga>,
    /// Overall yoga score
    pub total_yoga_score: f64,
    /// Summary interpretation
    pub summary: String,
}

impl YogaAnalysis {
    /// Create empty analysis
    pub fn empty() -> Self {
        Self {
            yogas: vec![],
            raj_yogas: vec![],
            dhana_yogas: vec![],
            mahapurusha_yogas: vec![],
            arishta_yogas: vec![],
            total_yoga_score: 0.0,
            summary: String::new(),
        }
    }

    /// Add a yoga to the analysis
    pub fn add_yoga(&mut self, yoga: DetectedYoga) {
        match yoga.category {
            YogaCategory::RajYoga => self.raj_yogas.push(yoga.clone()),
            YogaCategory::DhanaYoga => self.dhana_yogas.push(yoga.clone()),
            YogaCategory::MahapurushaYoga => self.mahapurusha_yogas.push(yoga.clone()),
            YogaCategory::ArishtaYoga => self.arishta_yogas.push(yoga.clone()),
            _ => {}
        }
        self.yogas.push(yoga);
    }

    /// Calculate total yoga score
    pub fn calculate_score(&mut self) {
        let mut score = 0.0;
        
        for yoga in &self.yogas {
            let base = match yoga.category {
                YogaCategory::RajYoga => 10.0,
                YogaCategory::DhanaYoga => 8.0,
                YogaCategory::MahapurushaYoga => 12.0,
                YogaCategory::ShubhaYoga => 5.0,
                YogaCategory::ArishtaYoga | YogaCategory::PapaYoga => -5.0,
                _ => 3.0,
            };
            
            let multiplier = match yoga.strength {
                YogaStrength::Full => 1.0,
                YogaStrength::Partial => 0.5,
                YogaStrength::Weak => 0.25,
                YogaStrength::Cancelled => 0.0,
            };
            
            score += base * multiplier;
        }
        
        self.total_yoga_score = score;
    }

    /// Generate summary
    pub fn generate_summary(&mut self) {
        let raj_count = self.raj_yogas.len();
        let dhana_count = self.dhana_yogas.len();
        let mahapurusha_count = self.mahapurusha_yogas.len();
        
        let mut parts = vec![];
        
        if raj_count > 0 {
            parts.push(format!("{} Raj Yoga(s) indicating success and authority", raj_count));
        }
        if dhana_count > 0 {
            parts.push(format!("{} Dhana Yoga(s) indicating wealth potential", dhana_count));
        }
        if mahapurusha_count > 0 {
            parts.push(format!("{} Mahapurusha Yoga(s) indicating special qualities", mahapurusha_count));
        }
        
        self.summary = if parts.is_empty() {
            "No major yogas detected.".to_string()
        } else {
            parts.join(". ") + "."
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_yoga_category_display() {
        assert_eq!(YogaCategory::RajYoga.to_string(), "Raj Yoga");
        assert_eq!(YogaCategory::DhanaYoga.to_string(), "Dhana Yoga");
    }

    #[test]
    fn test_yoga_analysis_add() {
        let mut analysis = YogaAnalysis::empty();
        
        let yoga = DetectedYoga {
            name: "Gaja Kesari".to_string(),
            category: YogaCategory::RajYoga,
            strength: YogaStrength::Full,
            planets_involved: vec!["Moon".to_string(), "Jupiter".to_string()],
            houses_involved: vec![1, 4],
            description: "Moon and Jupiter in kendras".to_string(),
            results: "Fame and recognition".to_string(),
            activation_periods: vec!["Moon dasha".to_string()],
        };
        
        analysis.add_yoga(yoga);
        
        assert_eq!(analysis.yogas.len(), 1);
        assert_eq!(analysis.raj_yogas.len(), 1);
    }

    #[test]
    fn test_yoga_score_calculation() {
        let mut analysis = YogaAnalysis::empty();
        
        analysis.add_yoga(DetectedYoga {
            name: "Test Raj".to_string(),
            category: YogaCategory::RajYoga,
            strength: YogaStrength::Full,
            planets_involved: vec![],
            houses_involved: vec![],
            description: String::new(),
            results: String::new(),
            activation_periods: vec![],
        });
        
        analysis.calculate_score();
        assert_eq!(analysis.total_yoga_score, 10.0);
    }
}
