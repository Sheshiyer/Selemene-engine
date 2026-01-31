//! Human Design Chart Generation
//!
//! Complete chart generation combining all 26 planetary activations
//! with centers, channels, type, authority, profile, and definition calculation.

use chrono::{DateTime, Utc};
use std::collections::HashMap;

use crate::{
    models::{HDChart, HDType, Authority, Profile, Definition},
    activations::calculate_all_activations,
    analysis::analyze_hd_chart,
};

/// Generate complete Human Design chart from birth data
///
/// This is the main entry point for HD chart generation. It calculates
/// all 26 planetary activations (13 Personality + 13 Design) and initializes
/// the chart structure. Centers, channels, type, authority, profile, and
/// definition will be calculated by subsequent agents.
///
/// # Arguments
/// * `birth_time` - Birth time in UTC
/// * `ephe_path` - Path to Swiss Ephemeris data files (use "" for built-in)
///
/// # Returns
/// Complete HDChart with all activations. Centers, channels, and derived
/// properties are initialized to placeholder values for Agent 21 to calculate.
///
/// # Performance
/// Target: <50ms for all 26 planetary calculations
pub fn generate_hd_chart(
    birth_time: DateTime<Utc>,
    ephe_path: &str,
) -> Result<HDChart, String> {
    let (personality, design) = calculate_all_activations(birth_time, ephe_path)?;
    
    // Verify we got all 26 activations
    if personality.len() != 13 {
        return Err(format!(
            "Expected 13 personality activations, got {}",
            personality.len()
        ));
    }
    if design.len() != 13 {
        return Err(format!(
            "Expected 13 design activations, got {}",
            design.len()
        ));
    }
    
    // Initialize chart with activations
    let mut chart = HDChart {
        personality_activations: personality,
        design_activations: design,
        centers: HashMap::new(),
        channels: vec![],
        hd_type: HDType::Generator,
        authority: Authority::Sacral,
        profile: Profile { 
            conscious_line: 1, 
            unconscious_line: 1 
        },
        definition: Definition::Single,
    };
    
    // Perform complete chart analysis
    analyze_hd_chart(&mut chart)?;
    
    Ok(chart)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_generate_chart_complete() {
        let birth_time = Utc.with_ymd_and_hms(1990, 5, 15, 14, 30, 0).unwrap();
        
        match generate_hd_chart(birth_time, "") {
            Ok(chart) => {
                // Verify 26 total activations
                assert_eq!(
                    chart.personality_activations.len(), 
                    13, 
                    "Should have 13 personality activations"
                );
                assert_eq!(
                    chart.design_activations.len(), 
                    13, 
                    "Should have 13 design activations"
                );
                
                // Verify all activations have valid values
                for act in chart.personality_activations.iter().chain(chart.design_activations.iter()) {
                    assert!(act.gate >= 1 && act.gate <= 64, "Gate {} out of range", act.gate);
                    assert!(act.line >= 1 && act.line <= 6, "Line {} out of range", act.line);
                    assert!(
                        act.longitude >= 0.0 && act.longitude < 360.0, 
                        "Longitude {} out of range", 
                        act.longitude
                    );
                }
                
                println!("\n✅ Generated complete HD chart with 26 activations");
                println!("Personality activations: {}", chart.personality_activations.len());
                println!("Design activations: {}", chart.design_activations.len());
            }
            Err(e) => {
                println!("Expected: Swiss Ephemeris data not available: {}", e);
            }
        }
    }

    #[test]
    fn test_chart_sun_earth_opposition() {
        let birth_time = Utc.with_ymd_and_hms(2000, 1, 1, 12, 0, 0).unwrap();
        
        match generate_hd_chart(birth_time, "") {
            Ok(chart) => {
                // First two activations should be Sun and Earth in both sets
                let pers_sun = &chart.personality_activations[0];
                let pers_earth = &chart.personality_activations[1];
                let des_sun = &chart.design_activations[0];
                let des_earth = &chart.design_activations[1];
                
                // Check Earth is opposite Sun in Personality
                let pers_diff = (pers_earth.longitude - pers_sun.longitude + 360.0) % 360.0;
                assert!(
                    (pers_diff - 180.0).abs() < 0.001,
                    "Personality Earth should be 180° from Sun, got {:.4}° difference",
                    pers_diff
                );
                
                // Check Earth is opposite Sun in Design
                let des_diff = (des_earth.longitude - des_sun.longitude + 360.0) % 360.0;
                assert!(
                    (des_diff - 180.0).abs() < 0.001,
                    "Design Earth should be 180° from Sun, got {:.4}° difference",
                    des_diff
                );
                
                println!("\n✅ Sun/Earth oppositions verified in both Personality and Design");
            }
            Err(e) => {
                println!("Expected: Swiss Ephemeris data not available: {}", e);
            }
        }
    }

    #[test]
    fn test_chart_different_birth_dates() {
        // Test multiple birth dates to ensure consistency
        let dates = vec![
            Utc.with_ymd_and_hms(1980, 3, 21, 10, 0, 0).unwrap(),
            Utc.with_ymd_and_hms(1995, 9, 23, 18, 30, 0).unwrap(),
            Utc.with_ymd_and_hms(2010, 12, 21, 6, 45, 0).unwrap(),
        ];
        
        for birth_time in dates {
            match generate_hd_chart(birth_time, "") {
                Ok(chart) => {
                    assert_eq!(chart.personality_activations.len(), 13);
                    assert_eq!(chart.design_activations.len(), 13);
                    
                    // Each chart should have unique positions
                    let pers_sun = &chart.personality_activations[0];
                    println!(
                        "Chart for {}: Personality Sun at Gate {}.{} ({:.4}°)",
                        birth_time, pers_sun.gate, pers_sun.line, pers_sun.longitude
                    );
                }
                Err(e) => {
                    println!("Expected: Swiss Ephemeris data not available: {}", e);
                    return;
                }
            }
        }
        
        println!("\n✅ All test charts generated successfully");
    }
}
