//! Dhana Yoga detection
//!
//! FAPI-066: Map wealth-related Dhana Yogas

use crate::birth_chart::types::{Planet, ZodiacSign, BirthChart};
use super::types::{DetectedYoga, YogaCategory, YogaStrength};

/// Detect Dhana Yogas in a birth chart
pub fn detect_dhana_yogas(chart: &BirthChart) -> Vec<DetectedYoga> {
    let mut yogas = vec![];
    
    // Check various wealth yogas
    if let Some(yoga) = detect_venus_jupiter_dhana(chart) {
        yogas.push(yoga);
    }
    
    if let Some(yoga) = detect_second_house_dhana(chart) {
        yogas.push(yoga);
    }
    
    if let Some(yoga) = detect_eleventh_house_dhana(chart) {
        yogas.push(yoga);
    }
    
    yogas
}

/// Venus-Jupiter combination for wealth
fn detect_venus_jupiter_dhana(chart: &BirthChart) -> Option<DetectedYoga> {
    let venus = chart.get_planet(Planet::Venus)?;
    let jupiter = chart.get_planet(Planet::Jupiter)?;
    
    // Check if Venus and Jupiter are in mutual kendras or trikonas
    let venus_house = venus.house;
    let jupiter_house = jupiter.house;
    
    let kendras = [1, 4, 7, 10];
    let trikonas = [1, 5, 9];
    
    let venus_in_kendra = kendras.contains(&venus_house);
    let jupiter_in_kendra = kendras.contains(&jupiter_house);
    let venus_in_trikona = trikonas.contains(&venus_house);
    let jupiter_in_trikona = trikonas.contains(&jupiter_house);
    
    if (venus_in_kendra && jupiter_in_trikona) || (venus_in_trikona && jupiter_in_kendra) {
        let strength = if !venus.is_combust && !jupiter.is_retrograde {
            YogaStrength::Full
        } else {
            YogaStrength::Partial
        };
        
        Some(DetectedYoga {
            name: "Venus-Jupiter Dhana Yoga".to_string(),
            category: YogaCategory::DhanaYoga,
            strength,
            planets_involved: vec!["Venus".to_string(), "Jupiter".to_string()],
            houses_involved: vec![venus_house, jupiter_house],
            description: "Venus and Jupiter in kendra-trikona relationship".to_string(),
            results: "Wealth through ethical means, financial prosperity".to_string(),
            activation_periods: vec!["Venus Dasha".to_string(), "Jupiter Dasha".to_string()],
        })
    } else {
        None
    }
}

/// Second house lord strength for wealth
fn detect_second_house_dhana(chart: &BirthChart) -> Option<DetectedYoga> {
    // Get 2nd house cusp and its lord
    let second_house = chart.get_house(2)?;
    let lord = second_house.lord;
    let lord_position = chart.get_planet(lord)?;
    
    // Check if 2nd lord is in a good house (1, 2, 5, 9, 10, 11)
    let good_houses = [1, 2, 5, 9, 10, 11];
    
    if good_houses.contains(&lord_position.house) {
        let strength = match lord_position.dignity {
            Some(crate::birth_chart::types::Dignity::Exalted) 
            | Some(crate::birth_chart::types::Dignity::OwnSign) => YogaStrength::Full,
            Some(crate::birth_chart::types::Dignity::Friendly) => YogaStrength::Partial,
            _ => YogaStrength::Weak,
        };
        
        Some(DetectedYoga {
            name: "Second Lord Dhana Yoga".to_string(),
            category: YogaCategory::DhanaYoga,
            strength,
            planets_involved: vec![lord.to_string()],
            houses_involved: vec![2, lord_position.house],
            description: format!("2nd lord {} well placed in house {}", lord, lord_position.house),
            results: "Accumulated wealth, good family finances".to_string(),
            activation_periods: vec![format!("{} Dasha", lord)],
        })
    } else {
        None
    }
}

/// Eleventh house for gains
fn detect_eleventh_house_dhana(chart: &BirthChart) -> Option<DetectedYoga> {
    let eleventh_house = chart.get_house(11)?;
    let lord = eleventh_house.lord;
    let lord_position = chart.get_planet(lord)?;
    
    // Planets in 11th house
    let planets_in_11 = chart.planets_in_house(11);
    
    // 11th lord in good position with benefics in 11th
    let benefics = [Planet::Jupiter, Planet::Venus, Planet::Mercury, Planet::Moon];
    let benefics_in_11: Vec<_> = planets_in_11.iter()
        .filter(|p| benefics.contains(&p.planet))
        .collect();
    
    if !benefics_in_11.is_empty() || [1, 2, 5, 9, 10, 11].contains(&lord_position.house) {
        Some(DetectedYoga {
            name: "Eleventh House Dhana Yoga".to_string(),
            category: YogaCategory::DhanaYoga,
            strength: if benefics_in_11.len() >= 2 { YogaStrength::Full } else { YogaStrength::Partial },
            planets_involved: benefics_in_11.iter().map(|p| p.planet.to_string()).collect(),
            houses_involved: vec![11],
            description: "Strong 11th house indicating gains".to_string(),
            results: "Income from multiple sources, fulfillment of desires".to_string(),
            activation_periods: vec![format!("{} Dasha", lord)],
        })
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dhana_detection_placeholder() {
        // Would need a full birth chart
        assert!(true);
    }
}
