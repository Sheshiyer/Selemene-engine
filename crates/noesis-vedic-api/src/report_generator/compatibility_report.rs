//! Compatibility report generation

use super::types::{CompatibilityPair, GeneratedReport, ReportSectionContent};
use super::ReportConfig;
use chrono::Utc;

/// Compatibility score categories
#[derive(Debug, Clone, Copy)]
pub struct CompatibilityScores {
    pub varna: u8,     // 1 point
    pub vashya: u8,    // 2 points
    pub tara: u8,      // 3 points
    pub yoni: u8,      // 4 points
    pub graha_maitri: u8, // 5 points
    pub gana: u8,      // 6 points
    pub bhakoot: u8,   // 7 points
    pub nadi: u8,      // 8 points
}

impl CompatibilityScores {
    pub fn total(&self) -> u8 {
        self.varna + self.vashya + self.tara + self.yoni + 
        self.graha_maitri + self.gana + self.bhakoot + self.nadi
    }
    
    pub fn max_possible() -> u8 {
        36 // Maximum Ashtakoot score
    }
    
    pub fn percentage(&self) -> f32 {
        (self.total() as f32 / Self::max_possible() as f32) * 100.0
    }
}

/// Generate compatibility report
pub fn generate_compatibility_report(
    pair: &CompatibilityPair,
    config: &ReportConfig,
) -> GeneratedReport {
    let sections = vec![
        ReportSectionContent {
            title: "Partner Details".to_string(),
            content: format!(
                "Person 1: {}\nBirth: {}\n\nPerson 2: {}\nBirth: {}",
                pair.person1.name,
                pair.person1.datetime,
                pair.person2.name,
                pair.person2.datetime
            ),
            key_points: vec![],
            chart_data: None,
        },
        ReportSectionContent {
            title: "Ashtakoot (Eight-fold) Matching".to_string(),
            content: "Analysis of compatibility based on Moon sign matching.".to_string(),
            key_points: vec![
                "Varna (1 point): Spiritual compatibility".to_string(),
                "Vashya (2 points): Mutual attraction".to_string(),
                "Tara (3 points): Birth star compatibility".to_string(),
                "Yoni (4 points): Physical compatibility".to_string(),
                "Graha Maitri (5 points): Mental compatibility".to_string(),
                "Gana (6 points): Temperament matching".to_string(),
                "Bhakoot (7 points): Family welfare".to_string(),
                "Nadi (8 points): Health compatibility".to_string(),
            ],
            chart_data: None,
        },
        ReportSectionContent {
            title: "Dosha Analysis".to_string(),
            content: "Checking for Manglik (Mars) dosha and other compatibility factors.".to_string(),
            key_points: vec![
                "Manglik dosha must be matched between partners".to_string(),
                "Nadi dosha can be cancelled under certain conditions".to_string(),
            ],
            chart_data: None,
        },
        ReportSectionContent {
            title: "Recommendations".to_string(),
            content: "Suggestions for harmonious relationship.".to_string(),
            key_points: vec![
                "Respect each other's differences".to_string(),
                "Communicate openly about expectations".to_string(),
                "Perform recommended remedies if needed".to_string(),
            ],
            chart_data: None,
        },
    ];
    
    GeneratedReport {
        title: format!("Compatibility Report: {} & {}", pair.person1.name, pair.person2.name),
        subject_name: format!("{} & {}", pair.person1.name, pair.person2.name),
        birth_datetime: pair.person1.datetime,
        generated_at: Utc::now().naive_utc(),
        sections,
        summary: "This compatibility analysis provides insights into the relationship potential. \
                  A score above 18 is generally considered favorable for marriage.".to_string(),
    }
}

/// Interpret compatibility score
pub fn interpret_compatibility(score: u8) -> &'static str {
    match score {
        0..=17 => "Not recommended. Significant differences may cause challenges.",
        18..=24 => "Average compatibility. Relationship will require effort.",
        25..=30 => "Good compatibility. Strong foundation for partnership.",
        31..=36 => "Excellent compatibility. Highly favorable match.",
        _ => "Invalid score",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compatibility_scores() {
        let scores = CompatibilityScores {
            varna: 1,
            vashya: 2,
            tara: 3,
            yoni: 4,
            graha_maitri: 5,
            gana: 6,
            bhakoot: 7,
            nadi: 8,
        };
        
        assert_eq!(scores.total(), 36);
        assert_eq!(scores.percentage(), 100.0);
    }

    #[test]
    fn test_interpret_compatibility() {
        assert!(interpret_compatibility(15).contains("Not recommended"));
        assert!(interpret_compatibility(25).contains("Good"));
        assert!(interpret_compatibility(35).contains("Excellent"));
    }
}
