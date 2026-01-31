//! HD Gate to Gene Keys mapping and Activation Sequences

use engine_human_design::{HDChart, Activation, Planet};
use crate::models::{GeneKeyActivation, ActivationSource, ActivationSequence};

/// Map HD gate activations to Gene Keys (1:1 correspondence)
///
/// Each HD gate maps directly to its corresponding Gene Key:
/// - Gate 1 → Gene Key 1
/// - Gate 17 → Gene Key 17
/// - Gate 64 → Gene Key 64
///
/// Line and longitude are preserved from HD activation.
///
/// # Arguments
/// * `hd_chart` - Complete HD chart with 26 activations
///
/// # Returns
/// Vec of 26 GeneKeyActivations (13 Personality + 13 Design)
pub fn map_hd_to_gene_keys(hd_chart: &HDChart) -> Vec<GeneKeyActivation> {
    let mut gene_keys = Vec::with_capacity(26);
    
    // Map Personality activations (13)
    for activation in &hd_chart.personality_activations {
        gene_keys.push(GeneKeyActivation {
            key_number: activation.gate,
            line: activation.line,
            source: ActivationSource::from_planet(activation.planet, false),
            gene_key_data: None,
        });
    }
    
    // Map Design activations (13)
    for activation in &hd_chart.design_activations {
        gene_keys.push(GeneKeyActivation {
            key_number: activation.gate,
            line: activation.line,
            source: ActivationSource::from_planet(activation.planet, true),
            gene_key_data: None,
        });
    }
    
    gene_keys
}

/// Calculate the 4 Core Activation Sequences
///
/// The sequences form the heart of Gene Keys work:
/// 1. **Life's Work**: Personality Sun + Earth (your conscious purpose)
/// 2. **Evolution**: Design Sun + Earth (your unconscious growth path)
/// 3. **Radiance**: Personality Sun + Design Sun (your core identity/magnetism)
/// 4. **Purpose**: Personality Earth + Design Earth (your higher calling)
///
/// # Arguments
/// * `hd_chart` - Complete HD chart with Sun/Earth activations
///
/// # Returns
/// ActivationSequence with all 4 sequences as (Gene Key, Gene Key) tuples
pub fn calculate_activation_sequences(hd_chart: &HDChart) -> Result<ActivationSequence, String> {
    let (pers_sun, pers_earth, design_sun, design_earth) = extract_sun_earth_gates(hd_chart)?;
    
    Ok(ActivationSequence {
        lifes_work: (pers_sun, pers_earth),
        evolution: (design_sun, design_earth),
        radiance: (pers_sun, design_sun),
        purpose: (pers_earth, design_earth),
    })
}

/// Find activation by planet in activation list
///
/// # Arguments
/// * `activations` - List of HD activations
/// * `planet` - Planet to find
///
/// # Returns
/// Reference to matching activation, or None if not found
pub fn find_activation_by_planet(
    activations: &[Activation],
    planet: Planet,
) -> Option<&Activation> {
    activations.iter().find(|a| a.planet == planet)
}

/// Extract Sun and Earth gates from HD chart
///
/// # Arguments
/// * `hd_chart` - Complete HD chart
///
/// # Returns
/// Tuple of (pers_sun, pers_earth, design_sun, design_earth) gate numbers
///
/// # Errors
/// Returns error if any Sun/Earth activation is missing
pub fn extract_sun_earth_gates(hd_chart: &HDChart) -> Result<(u8, u8, u8, u8), String> {
    let pers_sun = find_activation_by_planet(&hd_chart.personality_activations, Planet::Sun)
        .ok_or("Personality Sun not found")?
        .gate;
    
    let pers_earth = find_activation_by_planet(&hd_chart.personality_activations, Planet::Earth)
        .ok_or("Personality Earth not found")?
        .gate;
    
    let design_sun = find_activation_by_planet(&hd_chart.design_activations, Planet::Sun)
        .ok_or("Design Sun not found")?
        .gate;
    
    let design_earth = find_activation_by_planet(&hd_chart.design_activations, Planet::Earth)
        .ok_or("Design Earth not found")?
        .gate;
    
    Ok((pers_sun, pers_earth, design_sun, design_earth))
}

#[cfg(test)]
mod tests {
    use super::*;
    use engine_human_design::{
        HDChart, Activation, Planet, Center, CenterState, HDType, Authority, Profile, Definition
    };
    use std::collections::HashMap;
    
    fn create_test_hd_chart(
        pers_sun: u8,
        pers_earth: u8,
        design_sun: u8,
        design_earth: u8,
    ) -> HDChart {
        let mut personality_activations = vec![
            Activation { planet: Planet::Sun, gate: pers_sun, line: 3, longitude: 120.5 },
            Activation { planet: Planet::Earth, gate: pers_earth, line: 4, longitude: 300.5 },
        ];
        
        // Add remaining 11 Personality planets
        let other_planets = [
            (Planet::Moon, 25), (Planet::NorthNode, 10), (Planet::SouthNode, 15),
            (Planet::Mercury, 8), (Planet::Venus, 12), (Planet::Mars, 31),
            (Planet::Jupiter, 45), (Planet::Saturn, 28), (Planet::Uranus, 2),
            (Planet::Neptune, 36), (Planet::Pluto, 60),
        ];
        
        for (planet, gate) in other_planets {
            personality_activations.push(Activation {
                planet,
                gate,
                line: 2,
                longitude: 180.0,
            });
        }
        
        let mut design_activations = vec![
            Activation { planet: Planet::Sun, gate: design_sun, line: 5, longitude: 45.3 },
            Activation { planet: Planet::Earth, gate: design_earth, line: 1, longitude: 225.3 },
        ];
        
        // Add remaining 11 Design planets
        for (planet, gate) in other_planets {
            design_activations.push(Activation {
                planet,
                gate: gate + 1,
                line: 3,
                longitude: 90.0,
            });
        }
        
        HDChart {
            personality_activations,
            design_activations,
            centers: HashMap::new(),
            channels: vec![],
            hd_type: HDType::Generator,
            authority: Authority::Sacral,
            profile: Profile { conscious_line: 3, unconscious_line: 5 },
            definition: Definition::Single,
        }
    }
    
    #[test]
    fn test_map_hd_to_gene_keys() {
        let hd_chart = create_test_hd_chart(17, 18, 45, 26);
        let gene_keys = map_hd_to_gene_keys(&hd_chart);
        
        // Should have 26 total activations
        assert_eq!(gene_keys.len(), 26, "Should have 26 Gene Key activations");
        
        // Check Personality Sun mapping
        let pers_sun = &gene_keys[0];
        assert_eq!(pers_sun.key_number, 17, "Personality Sun should map to Gene Key 17");
        assert_eq!(pers_sun.line, 3, "Line should be preserved");
        assert_eq!(pers_sun.source, ActivationSource::PersonalitySun);
        
        // Check Personality Earth mapping
        let pers_earth = &gene_keys[1];
        assert_eq!(pers_earth.key_number, 18, "Personality Earth should map to Gene Key 18");
        assert_eq!(pers_earth.line, 4, "Line should be preserved");
        
        // Check Design Sun mapping (activation 13)
        let design_sun = &gene_keys[13];
        assert_eq!(design_sun.key_number, 45, "Design Sun should map to Gene Key 45");
        assert_eq!(design_sun.line, 5, "Line should be preserved");
        assert_eq!(design_sun.source, ActivationSource::DesignSun);
        
        // Check Design Earth mapping (activation 14)
        let design_earth = &gene_keys[14];
        assert_eq!(design_earth.key_number, 26, "Design Earth should map to Gene Key 26");
        assert_eq!(design_earth.line, 1, "Line should be preserved");
        assert_eq!(design_earth.source, ActivationSource::DesignEarth);
    }
    
    #[test]
    fn test_calculate_activation_sequences() {
        let hd_chart = create_test_hd_chart(17, 18, 45, 26);
        let sequences = calculate_activation_sequences(&hd_chart)
            .expect("Should calculate sequences");
        
        // Life's Work: Personality Sun + Earth
        assert_eq!(sequences.lifes_work, (17, 18), 
            "Life's Work should be Personality Sun + Earth");
        
        // Evolution: Design Sun + Earth
        assert_eq!(sequences.evolution, (45, 26), 
            "Evolution should be Design Sun + Earth");
        
        // Radiance: Personality Sun + Design Sun
        assert_eq!(sequences.radiance, (17, 45), 
            "Radiance should be Personality Sun + Design Sun");
        
        // Purpose: Personality Earth + Design Earth
        assert_eq!(sequences.purpose, (18, 26), 
            "Purpose should be Personality Earth + Design Earth");
    }
    
    #[test]
    fn test_extract_sun_earth_gates() {
        let hd_chart = create_test_hd_chart(1, 2, 3, 4);
        let result = extract_sun_earth_gates(&hd_chart)
            .expect("Should extract gates");
        
        assert_eq!(result, (1, 2, 3, 4), "Should extract all 4 Sun/Earth gates");
    }
    
    #[test]
    fn test_find_activation_by_planet() {
        let activations = vec![
            Activation { planet: Planet::Sun, gate: 17, line: 3, longitude: 120.0 },
            Activation { planet: Planet::Earth, gate: 18, line: 4, longitude: 300.0 },
            Activation { planet: Planet::Moon, gate: 25, line: 2, longitude: 45.0 },
        ];
        
        let sun = find_activation_by_planet(&activations, Planet::Sun);
        assert!(sun.is_some(), "Should find Sun");
        assert_eq!(sun.unwrap().gate, 17, "Sun should be at gate 17");
        
        let earth = find_activation_by_planet(&activations, Planet::Earth);
        assert!(earth.is_some(), "Should find Earth");
        assert_eq!(earth.unwrap().gate, 18, "Earth should be at gate 18");
        
        let mars = find_activation_by_planet(&activations, Planet::Mars);
        assert!(mars.is_none(), "Should not find Mars");
    }
    
    #[test]
    fn test_different_sequences() {
        // Test with different gate combinations
        let test_cases = vec![
            (1, 2, 3, 4),
            (17, 18, 45, 26),
            (64, 63, 1, 2),
            (33, 19, 7, 13),
        ];
        
        for (ps, pe, ds, de) in test_cases {
            let chart = create_test_hd_chart(ps, pe, ds, de);
            let seq = calculate_activation_sequences(&chart)
                .expect("Should calculate sequences");
            
            assert_eq!(seq.lifes_work, (ps, pe));
            assert_eq!(seq.evolution, (ds, de));
            assert_eq!(seq.radiance, (ps, ds));
            assert_eq!(seq.purpose, (pe, de));
        }
    }
    
    #[test]
    fn test_activation_source_from_planet() {
        assert_eq!(
            ActivationSource::from_planet(Planet::Sun, false),
            ActivationSource::PersonalitySun
        );
        assert_eq!(
            ActivationSource::from_planet(Planet::Sun, true),
            ActivationSource::DesignSun
        );
        assert_eq!(
            ActivationSource::from_planet(Planet::Earth, false),
            ActivationSource::PersonalityEarth
        );
        assert_eq!(
            ActivationSource::from_planet(Planet::Earth, true),
            ActivationSource::DesignEarth
        );
        assert_eq!(
            ActivationSource::from_planet(Planet::Venus, false),
            ActivationSource::PersonalityVenus
        );
        assert_eq!(
            ActivationSource::from_planet(Planet::Venus, true),
            ActivationSource::DesignVenus
        );
    }
}
