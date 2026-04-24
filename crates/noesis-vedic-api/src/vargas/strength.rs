//! Varga strength calculations
//!
//! FAPI-061: Create Varga strength calculator

use crate::birth_chart::types::{Planet, ZodiacSign, Dignity};
use super::types::{VargaType, VargaBala, VargaPoint};

/// Calculate dignity points for a planet in a varga
pub fn calculate_varga_points(planet: Planet, sign: ZodiacSign, varga: VargaType) -> f64 {
    let dignity = get_dignity_in_sign(planet, sign);
    
    // Base points for dignity
    let dignity_points = match dignity {
        Dignity::Exalted => 20.0,
        Dignity::MoolaTrikona => 18.0,
        Dignity::OwnSign => 15.0,
        Dignity::Friendly => 10.0,
        Dignity::Neutral => 5.0,
        Dignity::Enemy => 2.0,
        Dignity::Debilitated => 0.0,
    };
    
    // Weight based on varga importance
    let weight = varga_weight(varga);
    
    dignity_points * weight
}

/// Get weight/importance of a varga
fn varga_weight(varga: VargaType) -> f64 {
    match varga {
        VargaType::Rashi => 1.0,
        VargaType::Hora => 0.5,
        VargaType::Drekkana => 0.5,
        VargaType::Saptamsa => 0.5,
        VargaType::Navamsa => 1.0, // Equal to Rashi
        VargaType::Dasamsa => 0.5,
        VargaType::Dwadasamsa => 0.5,
        VargaType::Trimsamsa => 0.5,
        VargaType::Shashtiamsa => 0.5,
        _ => 0.25,
    }
}

/// Get dignity of planet in sign
fn get_dignity_in_sign(planet: Planet, sign: ZodiacSign) -> Dignity {
    // Simplified dignity calculation
    match (planet, sign) {
        // Exaltation
        (Planet::Sun, ZodiacSign::Aries) => Dignity::Exalted,
        (Planet::Moon, ZodiacSign::Taurus) => Dignity::Exalted,
        (Planet::Mars, ZodiacSign::Capricorn) => Dignity::Exalted,
        (Planet::Mercury, ZodiacSign::Virgo) => Dignity::Exalted,
        (Planet::Jupiter, ZodiacSign::Cancer) => Dignity::Exalted,
        (Planet::Venus, ZodiacSign::Pisces) => Dignity::Exalted,
        (Planet::Saturn, ZodiacSign::Libra) => Dignity::Exalted,
        
        // Debilitation
        (Planet::Sun, ZodiacSign::Libra) => Dignity::Debilitated,
        (Planet::Moon, ZodiacSign::Scorpio) => Dignity::Debilitated,
        (Planet::Mars, ZodiacSign::Cancer) => Dignity::Debilitated,
        (Planet::Mercury, ZodiacSign::Pisces) => Dignity::Debilitated,
        (Planet::Jupiter, ZodiacSign::Capricorn) => Dignity::Debilitated,
        (Planet::Venus, ZodiacSign::Virgo) => Dignity::Debilitated,
        (Planet::Saturn, ZodiacSign::Aries) => Dignity::Debilitated,
        
        // Own signs
        (Planet::Sun, ZodiacSign::Leo) => Dignity::OwnSign,
        (Planet::Moon, ZodiacSign::Cancer) => Dignity::OwnSign,
        (Planet::Mars, ZodiacSign::Aries) | (Planet::Mars, ZodiacSign::Scorpio) => Dignity::OwnSign,
        (Planet::Mercury, ZodiacSign::Gemini) => Dignity::OwnSign,
        (Planet::Jupiter, ZodiacSign::Sagittarius) | (Planet::Jupiter, ZodiacSign::Pisces) => Dignity::OwnSign,
        (Planet::Venus, ZodiacSign::Taurus) | (Planet::Venus, ZodiacSign::Libra) => Dignity::OwnSign,
        (Planet::Saturn, ZodiacSign::Capricorn) | (Planet::Saturn, ZodiacSign::Aquarius) => Dignity::OwnSign,
        
        _ => Dignity::Neutral,
    }
}

/// Calculate Shad Varga Bala (6 important vargas)
pub fn calculate_shad_varga_bala(
    positions: &[(VargaType, ZodiacSign)],
    planet: Planet,
) -> VargaBala {
    let shad_vargas = [
        VargaType::Rashi,
        VargaType::Hora,
        VargaType::Drekkana,
        VargaType::Navamsa,
        VargaType::Dwadasamsa,
        VargaType::Trimsamsa,
    ];
    
    let mut varga_points = vec![];
    let mut total_bala = 0.0;
    
    for (varga, sign) in positions {
        if shad_vargas.contains(varga) {
            let points = calculate_varga_points(planet, *sign, *varga);
            let dignity = get_dignity_in_sign(planet, *sign);
            
            varga_points.push(VargaPoint {
                varga: *varga,
                sign: sign.to_string(),
                dignity: format!("{:?}", dignity),
                points,
            });
            
            total_bala += points;
        }
    }
    
    // Varga vishwa is total out of maximum 20
    let max_possible = 20.0 * 6.0; // 6 vargas
    let varga_vishwa = (total_bala / max_possible) * 20.0;
    
    VargaBala {
        planet: planet.to_string(),
        varga_points,
        total_bala,
        varga_vishwa,
    }
}

/// Calculate Sapta Varga Bala (7 important vargas)
pub fn calculate_sapta_varga_bala(
    positions: &[(VargaType, ZodiacSign)],
    planet: Planet,
) -> VargaBala {
    let sapta_vargas = [
        VargaType::Rashi,
        VargaType::Hora,
        VargaType::Drekkana,
        VargaType::Saptamsa,
        VargaType::Navamsa,
        VargaType::Dwadasamsa,
        VargaType::Trimsamsa,
    ];
    
    let mut varga_points = vec![];
    let mut total_bala = 0.0;
    
    for (varga, sign) in positions {
        if sapta_vargas.contains(varga) {
            let points = calculate_varga_points(planet, *sign, *varga);
            let dignity = get_dignity_in_sign(planet, *sign);
            
            varga_points.push(VargaPoint {
                varga: *varga,
                sign: sign.to_string(),
                dignity: format!("{:?}", dignity),
                points,
            });
            
            total_bala += points;
        }
    }
    
    let max_possible = 20.0 * 7.0;
    let varga_vishwa = (total_bala / max_possible) * 20.0;
    
    VargaBala {
        planet: planet.to_string(),
        varga_points,
        total_bala,
        varga_vishwa,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_varga_points_exalted() {
        let points = calculate_varga_points(Planet::Sun, ZodiacSign::Aries, VargaType::Rashi);
        assert_eq!(points, 20.0);
    }

    #[test]
    fn test_varga_points_debilitated() {
        let points = calculate_varga_points(Planet::Sun, ZodiacSign::Libra, VargaType::Rashi);
        assert_eq!(points, 0.0);
    }

    #[test]
    fn test_dignity_lookup() {
        assert_eq!(get_dignity_in_sign(Planet::Jupiter, ZodiacSign::Cancer), Dignity::Exalted);
        assert_eq!(get_dignity_in_sign(Planet::Jupiter, ZodiacSign::Capricorn), Dignity::Debilitated);
    }
}
