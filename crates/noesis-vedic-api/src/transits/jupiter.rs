//! Jupiter transit analysis
//!
//! FAPI-077: Identify Jupiter transit blessings

use super::types::{JupiterTransitStatus, TransitQuality};
use crate::birth_chart::types::ZodiacSign;

/// Analyze Jupiter transit
pub fn analyze_jupiter_transit(
    jupiter_sign: ZodiacSign,
    ascendant_sign: ZodiacSign,
    moon_sign: ZodiacSign,
) -> JupiterTransitStatus {
    let jupiter_num = jupiter_sign.number();
    let asc_num = ascendant_sign.number();
    let moon_num = moon_sign.number();
    
    let from_asc = ((jupiter_num as i8 - asc_num as i8).rem_euclid(12) + 1) as u8;
    let from_moon = ((jupiter_num as i8 - moon_num as i8).rem_euclid(12) + 1) as u8;
    
    let quality = determine_jupiter_quality(from_asc, from_moon);
    let affected_areas = get_affected_areas(from_asc);
    
    JupiterTransitStatus {
        current_sign: jupiter_sign.to_string(),
        from_ascendant: from_asc,
        from_moon: from_moon,
        quality,
        affected_areas,
    }
}

/// Determine quality of Jupiter transit
fn determine_jupiter_quality(from_asc: u8, from_moon: u8) -> TransitQuality {
    // Favorable houses from Ascendant: 2, 5, 7, 9, 11
    // Favorable houses from Moon: 2, 5, 7, 9, 11
    let favorable_houses = [2, 5, 7, 9, 11];
    
    let asc_favorable = favorable_houses.contains(&from_asc);
    let moon_favorable = favorable_houses.contains(&from_moon);
    
    match (asc_favorable, moon_favorable) {
        (true, true) => TransitQuality::Excellent,
        (true, false) | (false, true) => TransitQuality::Good,
        (false, false) => {
            // Check for challenging houses
            let challenging = [6, 8, 12];
            if challenging.contains(&from_asc) && challenging.contains(&from_moon) {
                TransitQuality::Difficult
            } else if challenging.contains(&from_asc) || challenging.contains(&from_moon) {
                TransitQuality::Challenging
            } else {
                TransitQuality::Neutral
            }
        }
    }
}

/// Get areas of life affected by Jupiter transit
fn get_affected_areas(from_ascendant: u8) -> Vec<String> {
    match from_ascendant {
        1 => vec![
            "Self, personality, health".to_string(),
            "New beginnings, confidence".to_string(),
            "Overall life expansion".to_string(),
        ],
        2 => vec![
            "Family, finances, speech".to_string(),
            "Wealth accumulation".to_string(),
            "Food and pleasures".to_string(),
        ],
        3 => vec![
            "Siblings, communication".to_string(),
            "Short travels, courage".to_string(),
            "Skills and hobbies".to_string(),
        ],
        4 => vec![
            "Home, mother, property".to_string(),
            "Domestic happiness".to_string(),
            "Vehicles, education".to_string(),
        ],
        5 => vec![
            "Children, creativity".to_string(),
            "Romance, speculation".to_string(),
            "Intelligence, education".to_string(),
        ],
        6 => vec![
            "Health challenges may arise".to_string(),
            "Work environment changes".to_string(),
            "Victory over enemies".to_string(),
        ],
        7 => vec![
            "Marriage, partnerships".to_string(),
            "Business relations".to_string(),
            "Public dealings".to_string(),
        ],
        8 => vec![
            "Transformations".to_string(),
            "Inheritance matters".to_string(),
            "Research, occult interests".to_string(),
        ],
        9 => vec![
            "Luck, fortune, dharma".to_string(),
            "Long travels".to_string(),
            "Higher education, guru".to_string(),
        ],
        10 => vec![
            "Career advancement".to_string(),
            "Professional recognition".to_string(),
            "Status and authority".to_string(),
        ],
        11 => vec![
            "Gains, income increase".to_string(),
            "Fulfillment of desires".to_string(),
            "Social networks expand".to_string(),
        ],
        12 => vec![
            "Spiritual growth".to_string(),
            "Foreign connections".to_string(),
            "Expenses may increase".to_string(),
        ],
        _ => vec!["General blessings".to_string()],
    }
}

/// Get Jupiter transit predictions
pub fn jupiter_transit_predictions(status: &JupiterTransitStatus) -> Vec<String> {
    let mut predictions = vec![];
    
    match status.quality {
        TransitQuality::Excellent => {
            predictions.push("Highly favorable period for growth and expansion".to_string());
            predictions.push("Good time for important decisions and new ventures".to_string());
        }
        TransitQuality::Good => {
            predictions.push("Generally positive period".to_string());
            predictions.push("Benefits in indicated areas of life".to_string());
        }
        TransitQuality::Neutral => {
            predictions.push("Mixed results expected".to_string());
            predictions.push("Focus on consolidation rather than expansion".to_string());
        }
        TransitQuality::Challenging => {
            predictions.push("Some obstacles may arise".to_string());
            predictions.push("Exercise caution in major decisions".to_string());
        }
        TransitQuality::Difficult => {
            predictions.push("Exercise patience during this transit".to_string());
            predictions.push("Good for introspection and spiritual growth".to_string());
        }
    }
    
    predictions
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jupiter_transit_analysis() {
        let status = analyze_jupiter_transit(
            ZodiacSign::Aries,
            ZodiacSign::Leo, // Jupiter in 9th from Leo - excellent
            ZodiacSign::Scorpio,
        );
        
        assert_eq!(status.from_ascendant, 9);
        assert!(!status.affected_areas.is_empty());
    }

    #[test]
    fn test_jupiter_quality() {
        // Jupiter in 9th from ascendant and 5th from Moon - excellent
        let quality = determine_jupiter_quality(9, 5);
        assert_eq!(quality, TransitQuality::Excellent);
        
        // Jupiter in 6th from both - difficult
        let quality = determine_jupiter_quality(6, 6);
        assert_eq!(quality, TransitQuality::Difficult);
    }
}
