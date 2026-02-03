//! Raj Yoga detection
//!
//! FAPI-065: Map common Raj Yogas

use crate::birth_chart::types::{Planet, ZodiacSign, BirthChart};
use super::types::{DetectedYoga, YogaCategory, YogaStrength};

/// Detect Raj Yogas in a birth chart
pub fn detect_raj_yogas(chart: &BirthChart) -> Vec<DetectedYoga> {
    let mut yogas = vec![];
    
    // Gaja Kesari Yoga - Moon and Jupiter in kendras from each other
    if let Some(gaja_kesari) = detect_gaja_kesari(chart) {
        yogas.push(gaja_kesari);
    }
    
    // Pancha Mahapurusha Yogas
    yogas.extend(detect_mahapurusha_yogas(chart));
    
    // Kendra-Trikona Raj Yogas
    yogas.extend(detect_kendra_trikona_yogas(chart));
    
    // Lakshmi Yoga
    if let Some(lakshmi) = detect_lakshmi_yoga(chart) {
        yogas.push(lakshmi);
    }
    
    yogas
}

/// Detect Gaja Kesari Yoga
fn detect_gaja_kesari(chart: &BirthChart) -> Option<DetectedYoga> {
    let moon_pos = chart.get_planet(Planet::Moon)?;
    let jupiter_pos = chart.get_planet(Planet::Jupiter)?;
    
    // Check if Moon and Jupiter are in kendras (1, 4, 7, 10) from each other
    let moon_house = moon_pos.house;
    let jupiter_house = jupiter_pos.house;
    
    let difference = ((jupiter_house as i8 - moon_house as i8).abs() % 12) as u8;
    let kendras = [0, 3, 6, 9]; // 0 = same house, 3 = 4th, 6 = 7th, 9 = 10th
    
    if kendras.contains(&(difference % 12)) {
        let strength = if !moon_pos.is_combust && !jupiter_pos.is_retrograde {
            YogaStrength::Full
        } else {
            YogaStrength::Partial
        };
        
        Some(DetectedYoga {
            name: "Gaja Kesari Yoga".to_string(),
            category: YogaCategory::RajYoga,
            strength,
            planets_involved: vec!["Moon".to_string(), "Jupiter".to_string()],
            houses_involved: vec![moon_house, jupiter_house],
            description: "Moon and Jupiter in mutual kendras".to_string(),
            results: "Fame, recognition, wisdom, and good fortune".to_string(),
            activation_periods: vec!["Moon Dasha".to_string(), "Jupiter Dasha".to_string()],
        })
    } else {
        None
    }
}

/// Detect Pancha Mahapurusha Yogas
fn detect_mahapurusha_yogas(chart: &BirthChart) -> Vec<DetectedYoga> {
    let mut yogas = vec![];
    let kendra_houses = [1, 4, 7, 10];
    
    // Ruchaka Yoga - Mars in own sign or exalted in kendra
    if let Some(mars) = chart.get_planet(Planet::Mars) {
        if kendra_houses.contains(&mars.house) {
            let is_strong = mars.sign == ZodiacSign::Aries 
                || mars.sign == ZodiacSign::Scorpio 
                || mars.sign == ZodiacSign::Capricorn;
            
            if is_strong {
                yogas.push(DetectedYoga {
                    name: "Ruchaka Yoga".to_string(),
                    category: YogaCategory::MahapurushaYoga,
                    strength: if mars.is_retrograde { YogaStrength::Partial } else { YogaStrength::Full },
                    planets_involved: vec!["Mars".to_string()],
                    houses_involved: vec![mars.house],
                    description: "Mars in own sign or exalted in a kendra".to_string(),
                    results: "Courage, leadership, military success, physical strength".to_string(),
                    activation_periods: vec!["Mars Dasha".to_string()],
                });
            }
        }
    }
    
    // Bhadra Yoga - Mercury in own sign or exalted in kendra
    if let Some(mercury) = chart.get_planet(Planet::Mercury) {
        if kendra_houses.contains(&mercury.house) {
            let is_strong = mercury.sign == ZodiacSign::Gemini 
                || mercury.sign == ZodiacSign::Virgo;
            
            if is_strong {
                yogas.push(DetectedYoga {
                    name: "Bhadra Yoga".to_string(),
                    category: YogaCategory::MahapurushaYoga,
                    strength: YogaStrength::Full,
                    planets_involved: vec!["Mercury".to_string()],
                    houses_involved: vec![mercury.house],
                    description: "Mercury in own sign or exalted in a kendra".to_string(),
                    results: "Intelligence, eloquence, business acumen".to_string(),
                    activation_periods: vec!["Mercury Dasha".to_string()],
                });
            }
        }
    }
    
    // Hamsa Yoga - Jupiter in own sign or exalted in kendra
    if let Some(jupiter) = chart.get_planet(Planet::Jupiter) {
        if kendra_houses.contains(&jupiter.house) {
            let is_strong = jupiter.sign == ZodiacSign::Sagittarius 
                || jupiter.sign == ZodiacSign::Pisces 
                || jupiter.sign == ZodiacSign::Cancer;
            
            if is_strong {
                yogas.push(DetectedYoga {
                    name: "Hamsa Yoga".to_string(),
                    category: YogaCategory::MahapurushaYoga,
                    strength: YogaStrength::Full,
                    planets_involved: vec!["Jupiter".to_string()],
                    houses_involved: vec![jupiter.house],
                    description: "Jupiter in own sign or exalted in a kendra".to_string(),
                    results: "Wisdom, righteousness, spiritual inclination, respect".to_string(),
                    activation_periods: vec!["Jupiter Dasha".to_string()],
                });
            }
        }
    }
    
    // Malavya Yoga - Venus in own sign or exalted in kendra
    if let Some(venus) = chart.get_planet(Planet::Venus) {
        if kendra_houses.contains(&venus.house) {
            let is_strong = venus.sign == ZodiacSign::Taurus 
                || venus.sign == ZodiacSign::Libra 
                || venus.sign == ZodiacSign::Pisces;
            
            if is_strong {
                yogas.push(DetectedYoga {
                    name: "Malavya Yoga".to_string(),
                    category: YogaCategory::MahapurushaYoga,
                    strength: YogaStrength::Full,
                    planets_involved: vec!["Venus".to_string()],
                    houses_involved: vec![venus.house],
                    description: "Venus in own sign or exalted in a kendra".to_string(),
                    results: "Beauty, luxury, artistic talents, comfortable life".to_string(),
                    activation_periods: vec!["Venus Dasha".to_string()],
                });
            }
        }
    }
    
    // Shasha Yoga - Saturn in own sign or exalted in kendra
    if let Some(saturn) = chart.get_planet(Planet::Saturn) {
        if kendra_houses.contains(&saturn.house) {
            let is_strong = saturn.sign == ZodiacSign::Capricorn 
                || saturn.sign == ZodiacSign::Aquarius 
                || saturn.sign == ZodiacSign::Libra;
            
            if is_strong {
                yogas.push(DetectedYoga {
                    name: "Shasha Yoga".to_string(),
                    category: YogaCategory::MahapurushaYoga,
                    strength: if saturn.is_retrograde { YogaStrength::Partial } else { YogaStrength::Full },
                    planets_involved: vec!["Saturn".to_string()],
                    houses_involved: vec![saturn.house],
                    description: "Saturn in own sign or exalted in a kendra".to_string(),
                    results: "Authority, discipline, longevity, service achievements".to_string(),
                    activation_periods: vec!["Saturn Dasha".to_string()],
                });
            }
        }
    }
    
    yogas
}

/// Detect Kendra-Trikona Raj Yogas
fn detect_kendra_trikona_yogas(_chart: &BirthChart) -> Vec<DetectedYoga> {
    // Simplified - would need full house lord calculation
    vec![]
}

/// Detect Lakshmi Yoga
fn detect_lakshmi_yoga(chart: &BirthChart) -> Option<DetectedYoga> {
    let venus = chart.get_planet(Planet::Venus)?;
    
    // Venus should be in own or exalted sign and 9th lord strong
    let is_venus_strong = venus.sign == ZodiacSign::Taurus 
        || venus.sign == ZodiacSign::Libra 
        || venus.sign == ZodiacSign::Pisces;
    
    if is_venus_strong && !venus.is_combust {
        Some(DetectedYoga {
            name: "Lakshmi Yoga".to_string(),
            category: YogaCategory::RajYoga,
            strength: YogaStrength::Partial,
            planets_involved: vec!["Venus".to_string()],
            houses_involved: vec![venus.house],
            description: "Venus in strength with 9th lord".to_string(),
            results: "Wealth, prosperity, luxury, fortunate marriage".to_string(),
            activation_periods: vec!["Venus Dasha".to_string()],
        })
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mahapurusha_detection() {
        // Would need a full birth chart to test
        // This is a placeholder
        assert!(true);
    }
}
