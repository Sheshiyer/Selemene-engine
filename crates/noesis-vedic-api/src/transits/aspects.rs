//! Transit aspect calculations
//!
//! FAPI-075: Calculate transit aspects to natal chart

use super::types::{AspectType, AspectNature, TransitAspect};
use crate::birth_chart::types::{Planet, ZodiacSign};

/// Calculate aspect between two longitudes
pub fn calculate_aspect(transit_long: f64, natal_long: f64) -> Option<(AspectType, f64)> {
    let diff = (transit_long - natal_long).rem_euclid(360.0);
    let orb_limit = 8.0; // Standard orb for major aspects
    
    // Conjunction (0°)
    if diff < orb_limit || diff > (360.0 - orb_limit) {
        let orb = if diff < orb_limit { diff } else { 360.0 - diff };
        return Some((AspectType::Conjunction, orb));
    }
    
    // Opposition (180°)
    if (diff - 180.0).abs() < orb_limit {
        return Some((AspectType::Opposition, (diff - 180.0).abs()));
    }
    
    // Square (90° / 270°)
    if (diff - 90.0).abs() < orb_limit {
        return Some((AspectType::Square, (diff - 90.0).abs()));
    }
    if (diff - 270.0).abs() < orb_limit {
        return Some((AspectType::Square, (diff - 270.0).abs()));
    }
    
    // Trine (120° / 240°)
    if (diff - 120.0).abs() < orb_limit {
        return Some((AspectType::Trine, (diff - 120.0).abs()));
    }
    if (diff - 240.0).abs() < orb_limit {
        return Some((AspectType::Trine, (diff - 240.0).abs()));
    }
    
    // Sextile (60° / 300°)
    if (diff - 60.0).abs() < orb_limit {
        return Some((AspectType::Sextile, (diff - 60.0).abs()));
    }
    if (diff - 300.0).abs() < orb_limit {
        return Some((AspectType::Sextile, (diff - 300.0).abs()));
    }
    
    None
}

/// Check for special Vedic aspects
pub fn check_vedic_aspects(
    transiting_planet: Planet,
    transit_sign: u8,
    natal_sign: u8,
) -> Option<AspectType> {
    let diff = ((natal_sign as i8 - transit_sign as i8).rem_euclid(12) + 1) as u8;
    
    // All planets aspect 7th
    if diff == 7 {
        return Some(AspectType::SeventhAspect);
    }
    
    // Mars special aspects: 4th, 7th, 8th
    if transiting_planet == Planet::Mars && (diff == 4 || diff == 8) {
        return Some(AspectType::MarsSpecial);
    }
    
    // Jupiter special aspects: 5th, 7th, 9th
    if transiting_planet == Planet::Jupiter && (diff == 5 || diff == 9) {
        return Some(AspectType::JupiterSpecial);
    }
    
    // Saturn special aspects: 3rd, 7th, 10th
    if transiting_planet == Planet::Saturn && (diff == 3 || diff == 10) {
        return Some(AspectType::SaturnSpecial);
    }
    
    None
}

/// Determine aspect nature based on planets involved
pub fn determine_aspect_nature(
    transiting: Planet,
    natal: Planet,
    aspect_type: AspectType,
) -> AspectNature {
    let benefics = [Planet::Jupiter, Planet::Venus, Planet::Mercury, Planet::Moon];
    let malefics = [Planet::Saturn, Planet::Mars, Planet::Rahu, Planet::Ketu];
    
    let transit_benefic = benefics.contains(&transiting);
    let natal_benefic = benefics.contains(&natal);
    let transit_malefic = malefics.contains(&transiting);
    
    match aspect_type {
        AspectType::Conjunction => {
            if transit_benefic && natal_benefic {
                AspectNature::Benefic
            } else if transit_malefic {
                AspectNature::Malefic
            } else {
                AspectNature::Mixed
            }
        }
        AspectType::Trine | AspectType::Sextile => {
            if transit_benefic {
                AspectNature::Benefic
            } else {
                AspectNature::Neutral
            }
        }
        AspectType::Opposition | AspectType::Square => {
            if transit_malefic {
                AspectNature::Malefic
            } else {
                AspectNature::Neutral
            }
        }
        AspectType::JupiterSpecial => AspectNature::Benefic,
        AspectType::SaturnSpecial | AspectType::MarsSpecial => AspectNature::Mixed,
        _ => AspectNature::Neutral,
    }
}

/// Build a complete transit aspect
pub fn build_transit_aspect(
    transiting: Planet,
    transit_long: f64,
    natal: Planet,
    natal_long: f64,
    natal_sign: ZodiacSign,
) -> Option<TransitAspect> {
    let (aspect_type, orb) = calculate_aspect(transit_long, natal_long)?;
    let nature = determine_aspect_nature(transiting, natal, aspect_type);
    
    // Determine if applying (getting closer) or separating
    // Simplified: assume all are applying for now
    let is_applying = true;
    
    Some(TransitAspect {
        natal_planet: natal.to_string(),
        natal_sign: natal_sign.to_string(),
        aspect_type,
        is_applying,
        orb,
        nature,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_conjunction() {
        let (aspect, orb) = calculate_aspect(100.0, 102.0).unwrap();
        assert_eq!(aspect, AspectType::Conjunction);
        assert!(orb < 3.0);
    }

    #[test]
    fn test_calculate_opposition() {
        let (aspect, orb) = calculate_aspect(0.0, 180.0).unwrap();
        assert_eq!(aspect, AspectType::Opposition);
        assert!(orb < 1.0);
    }

    #[test]
    fn test_calculate_trine() {
        let (aspect, _orb) = calculate_aspect(0.0, 120.0).unwrap();
        assert_eq!(aspect, AspectType::Trine);
    }

    #[test]
    fn test_vedic_seventh_aspect() {
        let aspect = check_vedic_aspects(Planet::Sun, 1, 7);
        assert_eq!(aspect, Some(AspectType::SeventhAspect));
    }

    #[test]
    fn test_jupiter_special_aspect() {
        // Jupiter in sign 1, aspecting 5th (sign 5)
        let aspect = check_vedic_aspects(Planet::Jupiter, 1, 5);
        assert_eq!(aspect, Some(AspectType::JupiterSpecial));
        
        // Jupiter aspecting 9th (sign 9)
        let aspect = check_vedic_aspects(Planet::Jupiter, 1, 9);
        assert_eq!(aspect, Some(AspectType::JupiterSpecial));
    }

    #[test]
    fn test_aspect_nature() {
        // Jupiter trine should be benefic
        let nature = determine_aspect_nature(Planet::Jupiter, Planet::Sun, AspectType::Trine);
        assert_eq!(nature, AspectNature::Benefic);
        
        // Saturn square should be malefic
        let nature = determine_aspect_nature(Planet::Saturn, Planet::Moon, AspectType::Square);
        assert_eq!(nature, AspectNature::Malefic);
    }
}
