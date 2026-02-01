//! Panchanga qualities for temporal overlays
//!
//! Provides simplified Panchanga (Tithi, Nakshatra, Yoga, Karana) qualities
//! for integration with the organ clock system.

use serde::{Deserialize, Serialize};

/// Simplified Panchanga quality assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PanchangaQuality {
    /// Overall quality rating
    pub rating: QualityRating,
    /// Brief description
    pub description: String,
    /// Favorable activities
    pub favorable_for: Vec<String>,
    /// Activities to avoid
    pub avoid: Vec<String>,
}

/// Quality rating for Panchanga elements
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QualityRating {
    Excellent,
    Good,
    Neutral,
    Challenging,
}

impl QualityRating {
    pub fn to_score(&self) -> f64 {
        match self {
            QualityRating::Excellent => 1.0,
            QualityRating::Good => 0.75,
            QualityRating::Neutral => 0.5,
            QualityRating::Challenging => 0.25,
        }
    }

    pub fn display(&self) -> &'static str {
        match self {
            QualityRating::Excellent => "Excellent",
            QualityRating::Good => "Good",
            QualityRating::Neutral => "Neutral",
            QualityRating::Challenging => "Challenging",
        }
    }
}

/// Tithi (lunar day) qualities - simplified set
pub fn tithi_qualities() -> Vec<(&'static str, PanchangaQuality)> {
    vec![
        ("Pratipada", PanchangaQuality {
            rating: QualityRating::Good,
            description: "First lunar day - new beginnings".to_string(),
            favorable_for: vec!["Starting projects".to_string(), "New ventures".to_string()],
            avoid: vec!["Major decisions".to_string()],
        }),
        ("Dwitiya", PanchangaQuality {
            rating: QualityRating::Good,
            description: "Second lunar day - foundation building".to_string(),
            favorable_for: vec!["Planning".to_string(), "Foundation work".to_string()],
            avoid: vec!["Confrontation".to_string()],
        }),
        ("Tritiya", PanchangaQuality {
            rating: QualityRating::Excellent,
            description: "Third lunar day - auspicious".to_string(),
            favorable_for: vec!["All auspicious activities".to_string(), "Travel".to_string()],
            avoid: vec![],
        }),
        ("Chaturthi", PanchangaQuality {
            rating: QualityRating::Neutral,
            description: "Fourth lunar day - mixed energy".to_string(),
            favorable_for: vec!["Spiritual practices".to_string()],
            avoid: vec!["Travel".to_string(), "Starting new things".to_string()],
        }),
        ("Panchami", PanchangaQuality {
            rating: QualityRating::Good,
            description: "Fifth lunar day - learning".to_string(),
            favorable_for: vec!["Education".to_string(), "Learning".to_string(), "Arts".to_string()],
            avoid: vec![],
        }),
        ("Shashthi", PanchangaQuality {
            rating: QualityRating::Challenging,
            description: "Sixth lunar day - caution needed".to_string(),
            favorable_for: vec!["Fasting".to_string(), "Purification".to_string()],
            avoid: vec!["Major undertakings".to_string()],
        }),
        ("Saptami", PanchangaQuality {
            rating: QualityRating::Good,
            description: "Seventh lunar day - positive energy".to_string(),
            favorable_for: vec!["Travel".to_string(), "Vehicle purchase".to_string()],
            avoid: vec![],
        }),
        ("Ashtami", PanchangaQuality {
            rating: QualityRating::Challenging,
            description: "Eighth lunar day - intense energy".to_string(),
            favorable_for: vec!["Meditation".to_string(), "Tantric practices".to_string()],
            avoid: vec!["Material activities".to_string()],
        }),
        ("Navami", PanchangaQuality {
            rating: QualityRating::Challenging,
            description: "Ninth lunar day - destructive energy".to_string(),
            favorable_for: vec!["Ending things".to_string(), "Letting go".to_string()],
            avoid: vec!["Starting new things".to_string()],
        }),
        ("Dashami", PanchangaQuality {
            rating: QualityRating::Excellent,
            description: "Tenth lunar day - very auspicious".to_string(),
            favorable_for: vec!["All activities".to_string(), "Especially religious".to_string()],
            avoid: vec![],
        }),
        ("Ekadashi", PanchangaQuality {
            rating: QualityRating::Good,
            description: "Eleventh lunar day - spiritual".to_string(),
            favorable_for: vec!["Fasting".to_string(), "Spiritual practices".to_string()],
            avoid: vec!["Heavy eating".to_string()],
        }),
        ("Dwadashi", PanchangaQuality {
            rating: QualityRating::Good,
            description: "Twelfth lunar day - completion".to_string(),
            favorable_for: vec!["Breaking fast".to_string(), "Charity".to_string()],
            avoid: vec![],
        }),
        ("Trayodashi", PanchangaQuality {
            rating: QualityRating::Good,
            description: "Thirteenth lunar day - favorable".to_string(),
            favorable_for: vec!["Friendship".to_string(), "Travel".to_string()],
            avoid: vec![],
        }),
        ("Chaturdashi", PanchangaQuality {
            rating: QualityRating::Challenging,
            description: "Fourteenth lunar day - intense".to_string(),
            favorable_for: vec!["Shiva worship".to_string(), "Endings".to_string()],
            avoid: vec!["Beginnings".to_string()],
        }),
        ("Purnima/Amavasya", PanchangaQuality {
            rating: QualityRating::Neutral,
            description: "Full/New Moon - powerful energy".to_string(),
            favorable_for: vec!["Meditation".to_string(), "Rituals".to_string()],
            avoid: vec!["Impulsive decisions".to_string()],
        }),
    ]
}

/// Get Tithi quality by index (0-14, wraps around)
pub fn get_tithi_quality(tithi_index: u8) -> PanchangaQuality {
    let tithis = tithi_qualities();
    let index = (tithi_index as usize) % tithis.len();
    tithis[index].1.clone()
}

/// Simplified Nakshatra (lunar mansion) quality categories
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NakshatraCategory {
    Fixed,      // Stability, permanence
    Movable,    // Travel, change
    Sharp,      // Force, power
    Soft,       // Gentle activities
    Dreadful,   // Destruction, endings
    Mixed,      // Various activities
}

impl NakshatraCategory {
    pub fn quality(&self) -> PanchangaQuality {
        match self {
            NakshatraCategory::Fixed => PanchangaQuality {
                rating: QualityRating::Good,
                description: "Fixed nakshatra - stability".to_string(),
                favorable_for: vec!["Foundation work".to_string(), "Permanent structures".to_string(), "Establishing routines".to_string()],
                avoid: vec!["Travel".to_string(), "Major changes".to_string()],
            },
            NakshatraCategory::Movable => PanchangaQuality {
                rating: QualityRating::Good,
                description: "Movable nakshatra - change".to_string(),
                favorable_for: vec!["Travel".to_string(), "Starting journeys".to_string(), "Vehicle use".to_string()],
                avoid: vec!["Permanent commitments".to_string()],
            },
            NakshatraCategory::Sharp => PanchangaQuality {
                rating: QualityRating::Neutral,
                description: "Sharp nakshatra - power".to_string(),
                favorable_for: vec!["Competitive activities".to_string(), "Surgery".to_string(), "Breaking habits".to_string()],
                avoid: vec!["Gentle activities".to_string(), "Romance".to_string()],
            },
            NakshatraCategory::Soft => PanchangaQuality {
                rating: QualityRating::Excellent,
                description: "Soft nakshatra - gentle".to_string(),
                favorable_for: vec!["Arts".to_string(), "Romance".to_string(), "Healing".to_string(), "Creative work".to_string()],
                avoid: vec!["Harsh activities".to_string()],
            },
            NakshatraCategory::Dreadful => PanchangaQuality {
                rating: QualityRating::Challenging,
                description: "Dreadful nakshatra - endings".to_string(),
                favorable_for: vec!["Destruction of negativity".to_string(), "Releasing".to_string()],
                avoid: vec!["Beginnings".to_string(), "Important decisions".to_string()],
            },
            NakshatraCategory::Mixed => PanchangaQuality {
                rating: QualityRating::Neutral,
                description: "Mixed nakshatra - balanced".to_string(),
                favorable_for: vec!["Daily activities".to_string(), "Routine work".to_string()],
                avoid: vec![],
            },
        }
    }
}

/// Get Nakshatra category by index (0-26)
pub fn get_nakshatra_category(nakshatra_index: u8) -> NakshatraCategory {
    // Simplified mapping - in production, use full 27 nakshatra data
    match nakshatra_index % 6 {
        0 => NakshatraCategory::Fixed,
        1 => NakshatraCategory::Movable,
        2 => NakshatraCategory::Sharp,
        3 => NakshatraCategory::Soft,
        4 => NakshatraCategory::Dreadful,
        _ => NakshatraCategory::Mixed,
    }
}

/// Combined Panchanga quality assessment
pub fn get_combined_quality(
    tithi_index: Option<u8>,
    nakshatra_index: Option<u8>,
) -> PanchangaQuality {
    let tithi_quality = tithi_index.map(get_tithi_quality);
    let nakshatra_quality = nakshatra_index.map(|n| get_nakshatra_category(n).quality());

    match (tithi_quality, nakshatra_quality) {
        (Some(t), Some(n)) => {
            // Combine the two qualities
            let avg_score = (t.rating.to_score() + n.rating.to_score()) / 2.0;
            let combined_rating = if avg_score >= 0.9 {
                QualityRating::Excellent
            } else if avg_score >= 0.6 {
                QualityRating::Good
            } else if avg_score >= 0.4 {
                QualityRating::Neutral
            } else {
                QualityRating::Challenging
            };

            let mut favorable = t.favorable_for;
            favorable.extend(n.favorable_for);
            favorable.dedup();

            let mut avoid = t.avoid;
            avoid.extend(n.avoid);
            avoid.dedup();

            PanchangaQuality {
                rating: combined_rating,
                description: format!("{} | {}", t.description, n.description),
                favorable_for: favorable,
                avoid,
            }
        }
        (Some(t), None) => t,
        (None, Some(n)) => n,
        (None, None) => PanchangaQuality {
            rating: QualityRating::Neutral,
            description: "No Panchanga data available".to_string(),
            favorable_for: vec![],
            avoid: vec![],
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tithi_qualities_complete() {
        let tithis = tithi_qualities();
        assert_eq!(tithis.len(), 15);
    }

    #[test]
    fn test_quality_rating_scores() {
        assert!((QualityRating::Excellent.to_score() - 1.0).abs() < 0.001);
        assert!((QualityRating::Good.to_score() - 0.75).abs() < 0.001);
        assert!((QualityRating::Neutral.to_score() - 0.5).abs() < 0.001);
        assert!((QualityRating::Challenging.to_score() - 0.25).abs() < 0.001);
    }

    #[test]
    fn test_get_tithi_quality() {
        let quality = get_tithi_quality(0); // Pratipada
        assert_eq!(quality.rating, QualityRating::Good);
        
        let quality = get_tithi_quality(2); // Tritiya
        assert_eq!(quality.rating, QualityRating::Excellent);
    }

    #[test]
    fn test_nakshatra_categories() {
        let soft = NakshatraCategory::Soft.quality();
        assert_eq!(soft.rating, QualityRating::Excellent);
        
        let dreadful = NakshatraCategory::Dreadful.quality();
        assert_eq!(dreadful.rating, QualityRating::Challenging);
    }

    #[test]
    fn test_combined_quality() {
        // With both values
        let combined = get_combined_quality(Some(2), Some(3)); // Tritiya + Soft
        assert!(combined.rating == QualityRating::Excellent || combined.rating == QualityRating::Good);
        
        // With only tithi
        let tithi_only = get_combined_quality(Some(0), None);
        assert_eq!(tithi_only.rating, QualityRating::Good);
        
        // With neither
        let none = get_combined_quality(None, None);
        assert_eq!(none.rating, QualityRating::Neutral);
    }
}
