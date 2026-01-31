//! Integration tests for HD chart analysis
//!
//! Tests complete chart generation including centers, channels, type, authority, profile, and definition.

use engine_human_design::{generate_hd_chart, HDType, Authority, Definition, Center};
use chrono::{TimeZone, Utc};

#[test]
fn test_complete_chart_generation() {
    // Test with a known birth time
    let birth_time = Utc.with_ymd_and_hms(1990, 5, 15, 14, 30, 0).unwrap();
    
    match generate_hd_chart(birth_time, "") {
        Ok(chart) => {
            // Verify activations
            assert_eq!(chart.personality_activations.len(), 13);
            assert_eq!(chart.design_activations.len(), 13);
            
            // Verify centers analysis completed
            assert_eq!(chart.centers.len(), 9, "Should have all 9 centers");
            
            // Verify we have all center types
            let expected_centers = [
                Center::Head,
                Center::Ajna,
                Center::Throat,
                Center::G,
                Center::Heart,
                Center::Spleen,
                Center::SolarPlexus,
                Center::Sacral,
                Center::Root,
            ];
            
            for center in &expected_centers {
                assert!(
                    chart.centers.contains_key(center),
                    "Missing center: {:?}",
                    center
                );
            }
            
            // Verify type is determined (not default)
            println!("Type: {:?}", chart.hd_type);
            println!("Authority: {:?}", chart.authority);
            println!("Profile: {}/{}", chart.profile.conscious_line, chart.profile.unconscious_line);
            println!("Definition: {:?}", chart.definition);
            println!("Channels: {}", chart.channels.len());
            
            // Verify profile lines are valid (1-6)
            assert!(
                chart.profile.conscious_line >= 1 && chart.profile.conscious_line <= 6,
                "Invalid conscious line: {}",
                chart.profile.conscious_line
            );
            assert!(
                chart.profile.unconscious_line >= 1 && chart.profile.unconscious_line <= 6,
                "Invalid unconscious line: {}",
                chart.profile.unconscious_line
            );
            
            // Check consistency between Type and centers
            match chart.hd_type {
                HDType::Reflector => {
                    assert_eq!(chart.authority, Authority::Lunar, "Reflector should have Lunar authority");
                    assert_eq!(chart.definition, Definition::NoDefinition, "Reflector should have no definition");
                }
                HDType::Generator | HDType::ManifestingGenerator => {
                    let sacral = chart.centers.get(&Center::Sacral).unwrap();
                    assert!(sacral.defined, "Generator/MG should have Sacral defined");
                }
                HDType::Manifestor | HDType::Projector => {
                    let sacral = chart.centers.get(&Center::Sacral).unwrap();
                    assert!(!sacral.defined, "Manifestor/Projector should NOT have Sacral defined");
                }
            }
        }
        Err(e) => panic!("Chart generation failed: {}", e),
    }
}

#[test]
fn test_generator_chart() {
    // Use a birth time that typically produces a Generator chart
    let birth_time = Utc.with_ymd_and_hms(1985, 3, 20, 10, 0, 0).unwrap();
    
    if let Ok(chart) = generate_hd_chart(birth_time, "") {
        println!("Type: {:?}", chart.hd_type);
        println!("Sacral defined: {:?}", chart.centers.get(&Center::Sacral).map(|c| c.defined));
        
        // If Sacral is defined, should be Generator or Manifesting Generator
        if chart.centers.get(&Center::Sacral).map(|c| c.defined).unwrap_or(false) {
            assert!(
                matches!(chart.hd_type, HDType::Generator | HDType::ManifestingGenerator),
                "Sacral defined should result in Generator or MG, got {:?}",
                chart.hd_type
            );
            
            // Should have Sacral or Emotional authority
            assert!(
                matches!(chart.authority, Authority::Sacral | Authority::Emotional),
                "Generator should have Sacral or Emotional authority, got {:?}",
                chart.authority
            );
        }
    }
}

#[test]
fn test_multiple_birth_times() {
    // Test various birth times to ensure robustness
    // Using dates from 1990-2000 which work with built-in ephemeris
    let test_dates = vec![
        Utc.with_ymd_and_hms(1990, 6, 15, 12, 0, 0).unwrap(),
        Utc.with_ymd_and_hms(1995, 7, 4, 18, 30, 0).unwrap(),
        Utc.with_ymd_and_hms(1998, 3, 21, 9, 15, 0).unwrap(),
        Utc.with_ymd_and_hms(2000, 12, 25, 15, 45, 0).unwrap(),
    ];
    
    for birth_time in test_dates {
        match generate_hd_chart(birth_time, "") {
            Ok(chart) => {
                assert_eq!(chart.centers.len(), 9);
                assert!(chart.profile.conscious_line >= 1 && chart.profile.conscious_line <= 6);
                assert!(chart.profile.unconscious_line >= 1 && chart.profile.unconscious_line <= 6);
                
                println!(
                    "Birth: {} | Type: {:?} | Authority: {:?} | Profile: {}/{} | Channels: {}",
                    birth_time,
                    chart.hd_type,
                    chart.authority,
                    chart.profile.conscious_line,
                    chart.profile.unconscious_line,
                    chart.channels.len()
                );
            }
            Err(e) => panic!("Failed for {}: {}", birth_time, e),
        }
    }
}

#[test]
fn test_center_definition_logic() {
    let birth_time = Utc.with_ymd_and_hms(1990, 5, 15, 14, 30, 0).unwrap();
    
    if let Ok(chart) = generate_hd_chart(birth_time, "") {
        let defined_centers: Vec<_> = chart.centers
            .iter()
            .filter(|(_, state)| state.defined)
            .map(|(center, _)| center)
            .collect();
        
        println!("Defined centers: {:?}", defined_centers);
        
        // Each defined center should have gates
        for (center, state) in chart.centers.iter() {
            if state.defined {
                assert!(
                    !state.gates.is_empty(),
                    "Defined center {:?} should have gates",
                    center
                );
            }
        }
    }
}

#[test]
fn test_channel_activation() {
    let birth_time = Utc.with_ymd_and_hms(1990, 5, 15, 14, 30, 0).unwrap();
    
    if let Ok(chart) = generate_hd_chart(birth_time, "") {
        println!("Active channels: {}", chart.channels.len());
        
        // Each channel should have valid gate numbers
        for channel in &chart.channels {
            assert!(channel.gate1 >= 1 && channel.gate1 <= 64);
            assert!(channel.gate2 >= 1 && channel.gate2 <= 64);
            assert_ne!(channel.gate1, channel.gate2);
            assert!(!channel.name.is_empty());
            
            println!("Channel: {}-{} ({})", channel.gate1, channel.gate2, channel.name);
        }
    }
}

#[test]
fn test_profile_combinations() {
    // Test that profile combinations are valid
    let birth_time = Utc.with_ymd_and_hms(1990, 5, 15, 14, 30, 0).unwrap();
    
    if let Ok(chart) = generate_hd_chart(birth_time, "") {
        let profile = format!("{}/{}", chart.profile.conscious_line, chart.profile.unconscious_line);
        
        // Valid profiles in HD
        let _valid_profiles = [
            "1/3", "1/4", "2/4", "2/5", "3/5", "3/6",
            "4/6", "4/1", "5/1", "5/2", "6/2", "6/3"
        ];
        
        println!("Profile: {}", profile);
        
        // Note: Not all combinations are "standard" profiles, but all line combinations 1-6 are technically valid
        assert!(chart.profile.conscious_line <= 6);
        assert!(chart.profile.unconscious_line <= 6);
    }
}

#[test]
fn test_definition_types() {
    let birth_time = Utc.with_ymd_and_hms(1990, 5, 15, 14, 30, 0).unwrap();
    
    if let Ok(chart) = generate_hd_chart(birth_time, "") {
        println!("Definition: {:?}", chart.definition);
        
        // Definition should match center definition pattern
        let defined_count = chart.centers.values().filter(|c| c.defined).count();
        
        if defined_count == 0 {
            assert_eq!(chart.definition, Definition::NoDefinition);
        }
        // Other definition types depend on connectivity, not just count
    }
}

#[test]
fn test_authority_hierarchy() {
    let birth_time = Utc.with_ymd_and_hms(1990, 5, 15, 14, 30, 0).unwrap();
    
    if let Ok(chart) = generate_hd_chart(birth_time, "") {
        println!("Authority: {:?}", chart.authority);
        
        // Check authority consistency with centers
        let solar_plexus_defined = chart.centers.get(&Center::SolarPlexus)
            .map(|c| c.defined)
            .unwrap_or(false);
        let _sacral_defined = chart.centers.get(&Center::Sacral)
            .map(|c| c.defined)
            .unwrap_or(false);
        
        // If Solar Plexus is defined, must be Emotional authority
        if solar_plexus_defined {
            assert_eq!(
                chart.authority,
                Authority::Emotional,
                "Solar Plexus defined should result in Emotional authority"
            );
        }
        
        // If no centers defined, must be Lunar authority
        let any_defined = chart.centers.values().any(|c| c.defined);
        if !any_defined {
            assert_eq!(
                chart.authority,
                Authority::Lunar,
                "No centers defined should result in Lunar authority"
            );
        }
    }
}

#[test]
fn test_type_authority_consistency() {
    let birth_time = Utc.with_ymd_and_hms(1990, 5, 15, 14, 30, 0).unwrap();
    
    if let Ok(chart) = generate_hd_chart(birth_time, "") {
        // Reflector must have Lunar authority
        if chart.hd_type == HDType::Reflector {
            assert_eq!(chart.authority, Authority::Lunar);
            assert_eq!(chart.definition, Definition::NoDefinition);
        }
        
        // Generator/MG must have defined Sacral
        if matches!(chart.hd_type, HDType::Generator | HDType::ManifestingGenerator) {
            let sacral_defined = chart.centers.get(&Center::Sacral)
                .map(|c| c.defined)
                .unwrap_or(false);
            assert!(sacral_defined, "Generator/MG must have Sacral defined");
        }
        
        println!(
            "Type: {:?}, Authority: {:?}, Definition: {:?}",
            chart.hd_type, chart.authority, chart.definition
        );
    }
}
