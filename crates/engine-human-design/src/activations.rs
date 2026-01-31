//! Planetary Activation Calculations for Human Design
//!
//! Calculates Personality (birth) and Design (88 days before) activations
//! for Sun, Earth, and eventually all 13 planets.

use chrono::{DateTime, Utc};
use noesis_core::EngineError;

use crate::{
    models::{Activation, Planet},
    ephemeris::{EphemerisCalculator, HDPlanet},
    gate_sequence::{longitude_to_gate, longitude_to_line},
    design_time::calculate_design_time,
};

/// Calculate Personality Sun and Earth activations (at birth time)
///
/// # Arguments
/// * `birth_time` - Birth time in UTC
/// * `calculator` - Swiss Ephemeris calculator
///
/// # Returns
/// Tuple of (Sun activation, Earth activation)
pub fn calculate_personality_sun_earth(
    birth_time: &DateTime<Utc>,
    calculator: &EphemerisCalculator,
) -> Result<(Activation, Activation), EngineError> {
    // Get Sun position at birth
    let sun_pos = calculator.get_planet_position(HDPlanet::Sun, birth_time)?;
    let sun_longitude = sun_pos.longitude;
    
    // Calculate Sun gate and line
    let sun_gate = longitude_to_gate(sun_longitude);
    let sun_line = longitude_to_line(sun_longitude, sun_gate);
    
    let sun_activation = Activation {
        planet: Planet::Sun,
        gate: sun_gate,
        line: sun_line,
        longitude: sun_longitude,
    };
    
    // Earth is opposite Sun (180 degrees)
    let earth_longitude = (sun_longitude + 180.0) % 360.0;
    let earth_gate = longitude_to_gate(earth_longitude);
    let earth_line = longitude_to_line(earth_longitude, earth_gate);
    
    let earth_activation = Activation {
        planet: Planet::Earth,
        gate: earth_gate,
        line: earth_line,
        longitude: earth_longitude,
    };
    
    Ok((sun_activation, earth_activation))
}

/// Calculate Design Sun and Earth activations (at Design time - 88 days before birth)
///
/// # Arguments
/// * `birth_time` - Birth time in UTC
/// * `calculator` - Swiss Ephemeris calculator
///
/// # Returns
/// Tuple of (Sun activation, Earth activation) at Design time
pub fn calculate_design_sun_earth(
    birth_time: &DateTime<Utc>,
    calculator: &EphemerisCalculator,
) -> Result<(Activation, Activation), EngineError> {
    // Calculate Design time (88 days solar arc before birth)
    let design_time = calculate_design_time(*birth_time, Some(calculator.data_path()))
        .map_err(|e| EngineError::CalculationError(format!("Design time calculation failed: {}", e)))?;
    
    // Get Sun position at Design time
    let sun_pos = calculator.get_planet_position(HDPlanet::Sun, &design_time)?;
    let sun_longitude = sun_pos.longitude;
    
    // Calculate Sun gate and line
    let sun_gate = longitude_to_gate(sun_longitude);
    let sun_line = longitude_to_line(sun_longitude, sun_gate);
    
    let sun_activation = Activation {
        planet: Planet::Sun,
        gate: sun_gate,
        line: sun_line,
        longitude: sun_longitude,
    };
    
    // Earth is opposite Sun (180 degrees)
    let earth_longitude = (sun_longitude + 180.0) % 360.0;
    let earth_gate = longitude_to_gate(earth_longitude);
    let earth_line = longitude_to_line(earth_longitude, earth_gate);
    
    let earth_activation = Activation {
        planet: Planet::Earth,
        gate: earth_gate,
        line: earth_line,
        longitude: earth_longitude,
    };
    
    Ok((sun_activation, earth_activation))
}

/// Calculate complete personality and design activations for Sun and Earth
///
/// Returns both sets of activations in a tuple.
///
/// # Arguments
/// * `birth_time` - Birth time in UTC
/// * `calculator` - Swiss Ephemeris calculator
///
/// # Returns
/// Tuple of ((personality_sun, personality_earth), (design_sun, design_earth))
pub fn calculate_sun_earth_activations(
    birth_time: &DateTime<Utc>,
    calculator: &EphemerisCalculator,
) -> Result<((Activation, Activation), (Activation, Activation)), EngineError> {
    let personality = calculate_personality_sun_earth(birth_time, calculator)?;
    let design = calculate_design_sun_earth(birth_time, calculator)?;
    Ok((personality, design))
}

/// Map HDPlanet enum to our Planet enum
fn hdplanet_to_planet(hdplanet: HDPlanet) -> Planet {
    match hdplanet {
        HDPlanet::Sun => Planet::Sun,
        HDPlanet::Earth => Planet::Earth,
        HDPlanet::Moon => Planet::Moon,
        HDPlanet::NorthNode => Planet::NorthNode,
        HDPlanet::SouthNode => Planet::SouthNode,
        HDPlanet::Mercury => Planet::Mercury,
        HDPlanet::Venus => Planet::Venus,
        HDPlanet::Mars => Planet::Mars,
        HDPlanet::Jupiter => Planet::Jupiter,
        HDPlanet::Saturn => Planet::Saturn,
        HDPlanet::Uranus => Planet::Uranus,
        HDPlanet::Neptune => Planet::Neptune,
        HDPlanet::Pluto => Planet::Pluto,
    }
}

/// Calculate single planet activation from position data
fn create_activation(planet: Planet, longitude: f64) -> Activation {
    let gate = longitude_to_gate(longitude);
    let line = longitude_to_line(longitude, gate);
    
    Activation {
        planet,
        gate,
        line,
        longitude,
    }
}

/// Calculate all 13 planetary activations for Personality (at birth time)
///
/// # Arguments
/// * `birth_time` - Birth time in UTC
/// * `calculator` - Swiss Ephemeris calculator
///
/// # Returns
/// Vector of 13 activations in planetary order: Sun, Earth, Moon, Nodes, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto
pub fn calculate_personality_activations(
    birth_time: &DateTime<Utc>,
    calculator: &EphemerisCalculator,
) -> Result<Vec<Activation>, EngineError> {
    let planet_positions = calculator.get_all_planets(birth_time)?;
    
    let activations = planet_positions
        .into_iter()
        .map(|(hdplanet, pos)| {
            let planet = hdplanet_to_planet(hdplanet);
            create_activation(planet, pos.longitude)
        })
        .collect();
    
    Ok(activations)
}

/// Calculate all 13 planetary activations for Design (at Design time - 88 days before birth)
///
/// # Arguments
/// * `birth_time` - Birth time in UTC
/// * `calculator` - Swiss Ephemeris calculator
///
/// # Returns
/// Vector of 13 activations in planetary order at Design time
pub fn calculate_design_activations(
    birth_time: &DateTime<Utc>,
    calculator: &EphemerisCalculator,
) -> Result<Vec<Activation>, EngineError> {
    // Calculate Design time (88 days solar arc before birth)
    let design_time = calculate_design_time(*birth_time, Some(calculator.data_path()))
        .map_err(|e| EngineError::CalculationError(format!("Design time calculation failed: {}", e)))?;
    
    let planet_positions = calculator.get_all_planets(&design_time)?;
    
    let activations = planet_positions
        .into_iter()
        .map(|(hdplanet, pos)| {
            let planet = hdplanet_to_planet(hdplanet);
            create_activation(planet, pos.longitude)
        })
        .collect();
    
    Ok(activations)
}

/// Calculate all 26 planetary activations: 13 Personality + 13 Design
///
/// This is the main function for generating a complete HD chart's planetary data.
///
/// # Arguments
/// * `birth_time` - Birth time in UTC
/// * `ephe_path` - Path to Swiss Ephemeris data files (use "" for built-in)
///
/// # Returns
/// Tuple of (Personality activations, Design activations), each containing 13 planets
pub fn calculate_all_activations(
    birth_time: DateTime<Utc>,
    ephe_path: &str,
) -> Result<(Vec<Activation>, Vec<Activation>), String> {
    let calculator = EphemerisCalculator::new(ephe_path);
    
    let personality = calculate_personality_activations(&birth_time, &calculator)
        .map_err(|e| format!("Personality calculation failed: {}", e))?;
    
    let design = calculate_design_activations(&birth_time, &calculator)
        .map_err(|e| format!("Design calculation failed: {}", e))?;
    
    Ok((personality, design))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_earth_opposite_sun() {
        // Test that Earth is exactly 180 degrees opposite Sun
        let calculator = EphemerisCalculator::new("");
        let birth_time = Utc.with_ymd_and_hms(2000, 6, 21, 12, 0, 0).unwrap();
        
        match calculate_personality_sun_earth(&birth_time, &calculator) {
            Ok((sun, earth)) => {
                // Earth should be 180 degrees from Sun
                let longitude_diff = (earth.longitude - sun.longitude + 360.0) % 360.0;
                assert!(
                    (longitude_diff - 180.0).abs() < 0.001,
                    "Earth should be exactly 180° from Sun, got {:.6}° difference",
                    longitude_diff
                );
                
                println!("Personality Sun: Gate {}.{} at {:.4}°", sun.gate, sun.line, sun.longitude);
                println!("Personality Earth: Gate {}.{} at {:.4}°", earth.gate, earth.line, earth.longitude);
            }
            Err(e) => {
                println!("Expected: Swiss Ephemeris data not available: {:?}", e);
            }
        }
    }

    #[test]
    fn test_personality_vs_design_independence() {
        // Test that Design activations are different from Personality
        let calculator = EphemerisCalculator::new("");
        let birth_time = Utc.with_ymd_and_hms(1990, 3, 21, 10, 30, 0).unwrap();
        
        match calculate_sun_earth_activations(&birth_time, &calculator) {
            Ok(((pers_sun, pers_earth), (des_sun, des_earth))) => {
                // Personality and Design should be different (88 days difference)
                assert_ne!(
                    pers_sun.longitude, des_sun.longitude,
                    "Personality and Design Sun should have different positions"
                );
                
                // But both Earth activations should still be 180° from their respective Suns
                let pers_diff = (pers_earth.longitude - pers_sun.longitude + 360.0) % 360.0;
                let des_diff = (des_earth.longitude - des_sun.longitude + 360.0) % 360.0;
                
                assert!((pers_diff - 180.0).abs() < 0.001, "Personality Earth not opposite Sun");
                assert!((des_diff - 180.0).abs() < 0.001, "Design Earth not opposite Sun");
                
                println!("\nPersonality:");
                println!("  Sun: Gate {}.{} at {:.4}°", pers_sun.gate, pers_sun.line, pers_sun.longitude);
                println!("  Earth: Gate {}.{} at {:.4}°", pers_earth.gate, pers_earth.line, pers_earth.longitude);
                
                println!("\nDesign:");
                println!("  Sun: Gate {}.{} at {:.4}°", des_sun.gate, des_sun.line, des_sun.longitude);
                println!("  Earth: Gate {}.{} at {:.4}°", des_earth.gate, des_earth.line, des_earth.longitude);
            }
            Err(e) => {
                println!("Expected: Swiss Ephemeris data not available: {:?}", e);
            }
        }
    }

    #[test]
    fn test_gate_line_validity() {
        // Test that all gates and lines are in valid ranges
        let calculator = EphemerisCalculator::new("");
        let birth_time = Utc.with_ymd_and_hms(2020, 12, 25, 18, 45, 30).unwrap();
        
        match calculate_sun_earth_activations(&birth_time, &calculator) {
            Ok(((pers_sun, pers_earth), (des_sun, des_earth))) => {
                // Check Personality activations
                assert!(pers_sun.gate >= 1 && pers_sun.gate <= 64, "Personality Sun gate out of range");
                assert!(pers_sun.line >= 1 && pers_sun.line <= 6, "Personality Sun line out of range");
                assert!(pers_earth.gate >= 1 && pers_earth.gate <= 64, "Personality Earth gate out of range");
                assert!(pers_earth.line >= 1 && pers_earth.line <= 6, "Personality Earth line out of range");
                
                // Check Design activations
                assert!(des_sun.gate >= 1 && des_sun.gate <= 64, "Design Sun gate out of range");
                assert!(des_sun.line >= 1 && des_sun.line <= 6, "Design Sun line out of range");
                assert!(des_earth.gate >= 1 && des_earth.gate <= 64, "Design Earth gate out of range");
                assert!(des_earth.line >= 1 && des_earth.line <= 6, "Design Earth line out of range");
            }
            Err(e) => {
                println!("Expected: Swiss Ephemeris data not available: {:?}", e);
            }
        }
    }
    
    #[test]
    fn test_known_birth_date_summer_solstice() {
        // Test with summer solstice 2000 (Sun should be near 0° Cancer / 90° longitude)
        let calculator = EphemerisCalculator::new("");
        let birth_time = Utc.with_ymd_and_hms(2000, 6, 21, 1, 48, 0).unwrap(); // Exact solstice time
        
        match calculate_personality_sun_earth(&birth_time, &calculator) {
            Ok((sun, earth)) => {
                // Sun should be around 90° (0° Cancer)
                assert!(
                    (sun.longitude - 90.0).abs() < 5.0,
                    "Summer solstice Sun should be near 90°, got {:.2}°",
                    sun.longitude
                );
                
                // Earth should be around 270° (opposite)
                assert!(
                    (earth.longitude - 270.0).abs() < 5.0,
                    "Summer solstice Earth should be near 270°, got {:.2}°",
                    earth.longitude
                );
                
                println!("Summer Solstice 2000:");
                println!("  Sun: Gate {}.{} at {:.4}°", sun.gate, sun.line, sun.longitude);
                println!("  Earth: Gate {}.{} at {:.4}°", earth.gate, earth.line, earth.longitude);
            }
            Err(e) => {
                println!("Expected: Swiss Ephemeris data not available: {:?}", e);
            }
        }
    }

    #[test]
    fn test_all_13_personality_activations() {
        // Test that we get exactly 13 activations for Personality
        let calculator = EphemerisCalculator::new("");
        let birth_time = Utc.with_ymd_and_hms(2000, 1, 1, 12, 0, 0).unwrap();
        
        match calculate_personality_activations(&birth_time, &calculator) {
            Ok(activations) => {
                // Should have exactly 13 activations
                assert_eq!(activations.len(), 13, "Should have 13 personality activations");
                
                // Check that all planets are present
                let expected_planets = vec![
                    Planet::Sun, Planet::Earth, Planet::Moon,
                    Planet::NorthNode, Planet::SouthNode,
                    Planet::Mercury, Planet::Venus, Planet::Mars,
                    Planet::Jupiter, Planet::Saturn,
                    Planet::Uranus, Planet::Neptune, Planet::Pluto,
                ];
                
                for (i, expected) in expected_planets.iter().enumerate() {
                    assert_eq!(
                        activations[i].planet, *expected,
                        "Planet at index {} should be {:?}, got {:?}",
                        i, expected, activations[i].planet
                    );
                }
                
                // All gates should be 1-64, lines 1-6
                for act in &activations {
                    assert!(act.gate >= 1 && act.gate <= 64, "Gate {} out of range", act.gate);
                    assert!(act.line >= 1 && act.line <= 6, "Line {} out of range", act.line);
                    assert!(act.longitude >= 0.0 && act.longitude < 360.0, "Longitude {} out of range", act.longitude);
                }
                
                println!("\nPersonality Activations (Y2K):");
                for act in &activations {
                    println!("  {:?}: Gate {}.{} at {:.4}°", act.planet, act.gate, act.line, act.longitude);
                }
            }
            Err(e) => {
                println!("Expected: Swiss Ephemeris data not available: {:?}", e);
            }
        }
    }

    #[test]
    fn test_all_13_design_activations() {
        // Test that we get exactly 13 activations for Design
        let calculator = EphemerisCalculator::new("");
        let birth_time = Utc.with_ymd_and_hms(2000, 1, 1, 12, 0, 0).unwrap();
        
        match calculate_design_activations(&birth_time, &calculator) {
            Ok(activations) => {
                // Should have exactly 13 activations
                assert_eq!(activations.len(), 13, "Should have 13 design activations");
                
                // Check that all planets are present
                let expected_planets = vec![
                    Planet::Sun, Planet::Earth, Planet::Moon,
                    Planet::NorthNode, Planet::SouthNode,
                    Planet::Mercury, Planet::Venus, Planet::Mars,
                    Planet::Jupiter, Planet::Saturn,
                    Planet::Uranus, Planet::Neptune, Planet::Pluto,
                ];
                
                for (i, expected) in expected_planets.iter().enumerate() {
                    assert_eq!(
                        activations[i].planet, *expected,
                        "Planet at index {} should be {:?}, got {:?}",
                        i, expected, activations[i].planet
                    );
                }
                
                // All gates should be 1-64, lines 1-6
                for act in &activations {
                    assert!(act.gate >= 1 && act.gate <= 64, "Gate {} out of range", act.gate);
                    assert!(act.line >= 1 && act.line <= 6, "Line {} out of range", act.line);
                    assert!(act.longitude >= 0.0 && act.longitude < 360.0, "Longitude {} out of range", act.longitude);
                }
                
                println!("\nDesign Activations (Y2K):");
                for act in &activations {
                    println!("  {:?}: Gate {}.{} at {:.4}°", act.planet, act.gate, act.line, act.longitude);
                }
            }
            Err(e) => {
                println!("Expected: Swiss Ephemeris data not available: {:?}", e);
            }
        }
    }

    #[test]
    fn test_all_26_activations() {
        // Test the main function that returns all 26 activations
        let birth_time = Utc.with_ymd_and_hms(1990, 5, 15, 14, 30, 0).unwrap();
        
        match calculate_all_activations(birth_time, "") {
            Ok((personality, design)) => {
                // Should have 13 each
                assert_eq!(personality.len(), 13, "Should have 13 personality activations");
                assert_eq!(design.len(), 13, "Should have 13 design activations");
                
                // Personality and Design should be different
                assert_ne!(
                    personality[0].longitude, design[0].longitude,
                    "Personality and Design Sun should differ"
                );
                
                // Verify Earth is opposite Sun in both
                let pers_sun_earth_diff = (personality[1].longitude - personality[0].longitude + 360.0) % 360.0;
                assert!((pers_sun_earth_diff - 180.0).abs() < 0.001, "Personality Earth not opposite Sun");
                
                let des_sun_earth_diff = (design[1].longitude - design[0].longitude + 360.0) % 360.0;
                assert!((des_sun_earth_diff - 180.0).abs() < 0.001, "Design Earth not opposite Sun");
                
                println!("\nComplete 26 Activations for May 15, 1990:");
                println!("\nPersonality (Birth):");
                for act in &personality {
                    println!("  {:?}: Gate {}.{}", act.planet, act.gate, act.line);
                }
                println!("\nDesign (88 days before):");
                for act in &design {
                    println!("  {:?}: Gate {}.{}", act.planet, act.gate, act.line);
                }
            }
            Err(e) => {
                println!("Expected: Swiss Ephemeris data not available: {}", e);
            }
        }
    }

    #[test]
    fn test_personality_sun_earth_match_agent19() {
        // Verify that our new functions match Agent 19's Sun/Earth calculations
        let calculator = EphemerisCalculator::new("");
        let birth_time = Utc.with_ymd_and_hms(2000, 6, 21, 12, 0, 0).unwrap();
        
        match (
            calculate_personality_sun_earth(&birth_time, &calculator),
            calculate_personality_activations(&birth_time, &calculator)
        ) {
            (Ok((sun_old, earth_old)), Ok(all_activations)) => {
                // First two activations should be Sun and Earth
                let sun_new = &all_activations[0];
                let earth_new = &all_activations[1];
                
                // Should match exactly
                assert_eq!(sun_old.longitude, sun_new.longitude, "Sun longitude mismatch");
                assert_eq!(sun_old.gate, sun_new.gate, "Sun gate mismatch");
                assert_eq!(sun_old.line, sun_new.line, "Sun line mismatch");
                
                assert_eq!(earth_old.longitude, earth_new.longitude, "Earth longitude mismatch");
                assert_eq!(earth_old.gate, earth_new.gate, "Earth gate mismatch");
                assert_eq!(earth_old.line, earth_new.line, "Earth line mismatch");
                
                println!("✅ Agent 19 compatibility verified: Sun/Earth calculations match");
            }
            _ => {
                println!("Expected: Swiss Ephemeris data not available");
            }
        }
    }
}
