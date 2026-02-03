//! Shadbala calculator
//!
//! FAPI-069: Calculate total Shadbala and Rupas

use crate::birth_chart::types::{Planet, ZodiacSign, BirthChart};
use super::types::{ShadbalaComponent, ShadbalaValue, PlanetShadbala, required_shadbala};

/// Calculate Sthana Bala (Positional Strength)
pub fn calculate_sthana_bala(planet: Planet, sign: ZodiacSign, degree: f64) -> f64 {
    let mut bala = 0.0;
    
    // Uchcha Bala (Exaltation strength)
    bala += calculate_uchcha_bala(planet, sign, degree);
    
    // Saptavargaja Bala (Divisional chart strength) - simplified
    bala += calculate_saptavargaja_bala(planet, sign);
    
    // Ojhayugma Bala (Odd-even sign strength)
    bala += calculate_ojhayugma_bala(planet, sign);
    
    // Kendra Bala (Angular strength) - would need house info
    bala += 15.0; // Default
    
    // Drekkana Bala - simplified
    bala += 10.0;
    
    bala
}

/// Calculate Uchcha Bala (exaltation strength)
fn calculate_uchcha_bala(planet: Planet, sign: ZodiacSign, degree: f64) -> f64 {
    let longitude = (sign.number() as f64 - 1.0) * 30.0 + degree;
    
    // Exaltation points for each planet
    let (exalt_long, _debil_long) = match planet {
        Planet::Sun => (10.0, 190.0),      // Aries 10°
        Planet::Moon => (33.0, 213.0),     // Taurus 3°
        Planet::Mars => (298.0, 118.0),    // Capricorn 28°
        Planet::Mercury => (165.0, 345.0), // Virgo 15°
        Planet::Jupiter => (95.0, 275.0),  // Cancer 5°
        Planet::Venus => (357.0, 177.0),   // Pisces 27°
        Planet::Saturn => (200.0, 20.0),   // Libra 20°
        _ => return 0.0,
    };
    
    // Distance from exaltation point
    let distance = (longitude - exalt_long).abs();
    let normalized_distance = if distance > 180.0 { 360.0 - distance } else { distance };
    
    // Maximum 60 shashtiamsas when at exact exaltation
    let bala = (180.0 - normalized_distance) / 3.0;
    bala.max(0.0)
}

/// Calculate Saptavargaja Bala (simplified)
fn calculate_saptavargaja_bala(planet: Planet, sign: ZodiacSign) -> f64 {
    // Simplified - based on sign relationship
    let sign_lord = sign.ruler();
    
    if sign_lord == planet {
        45.0 // Own sign
    } else {
        match (planet, sign_lord) {
            // Natural friendships
            (Planet::Sun, Planet::Moon) | (Planet::Moon, Planet::Sun) => 30.0,
            (Planet::Sun, Planet::Mars) | (Planet::Mars, Planet::Sun) => 30.0,
            (Planet::Sun, Planet::Jupiter) | (Planet::Jupiter, Planet::Sun) => 30.0,
            (Planet::Moon, Planet::Mercury) | (Planet::Mercury, Planet::Moon) => 30.0,
            (Planet::Jupiter, Planet::Mars) | (Planet::Mars, Planet::Jupiter) => 30.0,
            (Planet::Venus, Planet::Mercury) | (Planet::Mercury, Planet::Venus) => 30.0,
            (Planet::Venus, Planet::Saturn) | (Planet::Saturn, Planet::Venus) => 30.0,
            // Natural enmities
            (Planet::Sun, Planet::Saturn) | (Planet::Saturn, Planet::Sun) => 7.5,
            (Planet::Sun, Planet::Venus) | (Planet::Venus, Planet::Sun) => 7.5,
            (Planet::Moon, Planet::Saturn) | (Planet::Saturn, Planet::Moon) => 7.5,
            _ => 15.0, // Neutral
        }
    }
}

/// Calculate Ojhayugma Bala
fn calculate_ojhayugma_bala(planet: Planet, sign: ZodiacSign) -> f64 {
    let is_odd_sign = sign.number() % 2 == 1;
    
    match planet {
        // Sun, Mars, Jupiter prefer odd signs
        Planet::Sun | Planet::Mars | Planet::Jupiter => {
            if is_odd_sign { 15.0 } else { 0.0 }
        }
        // Moon, Venus, Saturn prefer even signs
        Planet::Moon | Planet::Venus | Planet::Saturn => {
            if !is_odd_sign { 15.0 } else { 0.0 }
        }
        // Mercury is neutral
        _ => 7.5,
    }
}

/// Calculate Dig Bala (Directional Strength)
pub fn calculate_dig_bala(planet: Planet, house: u8) -> f64 {
    // Maximum Dig Bala is 60 shashtiamsas
    let preferred_house = match planet {
        Planet::Sun | Planet::Mars => 10, // 10th house (south)
        Planet::Moon | Planet::Venus => 4, // 4th house (north)
        Planet::Jupiter | Planet::Mercury => 1, // 1st house (east)
        Planet::Saturn => 7, // 7th house (west)
        _ => 1,
    };
    
    // Distance from preferred house (in houses, 0-6)
    let distance = ((house as i8 - preferred_house as i8).abs() % 12) as u8;
    let normalized = if distance > 6 { 12 - distance } else { distance };
    
    // 60 at preferred, 0 at opposite
    60.0 - (normalized as f64 * 10.0)
}

/// Calculate all Shadbala components for a planet
pub fn calculate_full_shadbala(
    planet: Planet,
    sign: ZodiacSign,
    degree: f64,
    house: u8,
    is_retrograde: bool,
) -> PlanetShadbala {
    let sthana = calculate_sthana_bala(planet, sign, degree);
    let dig = calculate_dig_bala(planet, house);
    
    // Simplified Kala Bala
    let kala = 30.0; // Would need birth time calculations
    
    // Chesta Bala (retrograde planets get bonus)
    let chesta = if is_retrograde { 60.0 } else { 30.0 };
    
    // Naisargika Bala (natural strength)
    let naisargika = match planet {
        Planet::Sun => 60.0,
        Planet::Moon => 51.43,
        Planet::Venus => 42.85,
        Planet::Jupiter => 34.28,
        Planet::Mercury => 25.71,
        Planet::Mars => 17.14,
        Planet::Saturn => 8.57,
        _ => 0.0,
    };
    
    // Drik Bala (aspectual) - simplified
    let drik = 15.0;
    
    let total = sthana + dig + kala + chesta + naisargika + drik;
    let required = required_shadbala(&planet.to_string());
    let ratio = total / required;
    
    PlanetShadbala {
        planet: planet.to_string(),
        components: vec![
            ShadbalaValue { component: ShadbalaComponent::SthanaBala, rupas: sthana, shashtiamsas: sthana },
            ShadbalaValue { component: ShadbalaComponent::DigBala, rupas: dig, shashtiamsas: dig },
            ShadbalaValue { component: ShadbalaComponent::KalaBala, rupas: kala, shashtiamsas: kala },
            ShadbalaValue { component: ShadbalaComponent::ChestaBala, rupas: chesta, shashtiamsas: chesta },
            ShadbalaValue { component: ShadbalaComponent::NaisargikaBala, rupas: naisargika, shashtiamsas: naisargika },
            ShadbalaValue { component: ShadbalaComponent::DrikBala, rupas: drik, shashtiamsas: drik },
        ],
        total_rupas: total,
        total_shashtiamsas: total,
        required_minimum: required,
        strength_ratio: ratio,
        is_strong: ratio >= 1.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dig_bala() {
        // Sun in 10th house should have high Dig Bala
        let dig = calculate_dig_bala(Planet::Sun, 10);
        assert!(dig >= 50.0);
        
        // Sun in 4th house should have low Dig Bala
        let dig = calculate_dig_bala(Planet::Sun, 4);
        assert!(dig <= 20.0);
    }

    #[test]
    fn test_uchcha_bala() {
        // Sun near exaltation in Aries
        let bala = calculate_uchcha_bala(Planet::Sun, ZodiacSign::Aries, 10.0);
        assert!(bala >= 50.0);
        
        // Sun in Libra (debilitation)
        let bala = calculate_uchcha_bala(Planet::Sun, ZodiacSign::Libra, 10.0);
        assert!(bala < 20.0);
    }

    #[test]
    fn test_full_shadbala() {
        let shadbala = calculate_full_shadbala(
            Planet::Sun,
            ZodiacSign::Aries,
            10.0,
            10,
            false,
        );
        
        assert_eq!(shadbala.planet, "Sun");
        assert!(shadbala.total_rupas > 0.0);
        assert_eq!(shadbala.components.len(), 6);
    }
}
