use chrono::{TimeZone, Utc};
use engine_human_design::{generate_hd_chart, HDType, Authority, CenterState, Planet};

fn main() {
    // Shesh Iyer birth data:
    // August 13, 1991, 13:31 IST (UTC+5:30) = 08:01 UTC
    // Bengaluru: 12.9716°N, 77.5946°E
    
    let birth_time = Utc.with_ymd_and_hms(1991, 8, 13, 8, 1, 0).unwrap();
    
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║      HUMAN DESIGN CALCULATION - SHESH IYER                   ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!();
    println!("Birth: August 13, 1991, 13:31 IST (08:01 UTC)");
    println!("Location: Bengaluru (12.9716°N, 77.5946°E)");
    println!();
    
    match generate_hd_chart(birth_time, "") {
        Ok(chart) => {
            println!("=== CALCULATED CHART ===");
            println!();
            println!("Type: {:?}", chart.hd_type);
            println!("Authority: {:?}", chart.authority);
            println!("Profile: {}/{}", chart.profile.conscious_line, chart.profile.unconscious_line);
            println!("Definition: {:?}", chart.definition);
            println!();
            
            println!("=== DEFINED CENTERS ===");
            for (center, state) in &chart.centers {
                if state.defined {
                    println!("  ✓ {:?}", center);
                }
            }
            println!();
            
            println!("=== UNDEFINED CENTERS ===");
            for (center, state) in &chart.centers {
                if !state.defined {
                    println!("  ○ {:?}", center);
                }
            }
            println!();
            
            println!("=== ACTIVE CHANNELS ===");
            for channel in &chart.channels {
                println!("  {} - {} ({})", channel.gate1, channel.gate2, channel.name);
            }
            println!();
            
            // Get Sun/Earth from personality_activations
            let p_sun = chart.personality_activations.iter().find(|a| a.planet == Planet::Sun);
            let p_earth = chart.personality_activations.iter().find(|a| a.planet == Planet::Earth);
            let d_sun = chart.design_activations.iter().find(|a| a.planet == Planet::Sun);
            let d_earth = chart.design_activations.iter().find(|a| a.planet == Planet::Earth);
            
            println!("=== SUN/EARTH GATES (Incarnation Cross) ===");
            if let Some(sun) = p_sun {
                println!("Personality Sun: Gate {}.{}", sun.gate, sun.line);
            }
            if let Some(earth) = p_earth {
                println!("Personality Earth: Gate {}.{}", earth.gate, earth.line);
            }
            if let Some(sun) = d_sun {
                println!("Design Sun: Gate {}.{}", sun.gate, sun.line);
            }
            if let Some(earth) = d_earth {
                println!("Design Earth: Gate {}.{}", earth.gate, earth.line);
            }
            println!();
            
            if let (Some(ps), Some(pe), Some(ds), Some(de)) = (p_sun, p_earth, d_sun, d_earth) {
                println!("=== INCARNATION CROSS ===");
                println!("Cross: {}/{} | {}/{}", ps.gate, pe.gate, ds.gate, de.gate);
            }
            
            // Validation against known data
            println!();
            println!("═══════════════════════════════════════════════════════════════");
            println!("VALIDATION AGAINST KNOWN PROFILE:");
            println!("═══════════════════════════════════════════════════════════════");
            println!();
            
            // Expected values from entrodromia profile:
            // Type: Generator, Profile: 2/4, Authority: Sacral
            // Cross: Right Angle Cross of Explanation (4/49 | 23/43)
            
            let expected_type = HDType::Generator;
            let expected_authority = Authority::Sacral;
            
            let type_ok = chart.hd_type == expected_type;
            println!("Type: {:?} {} (expected: Generator)", 
                chart.hd_type, 
                if type_ok { "✓" } else { "✗" }
            );
            
            let auth_ok = chart.authority == expected_authority;
            println!("Authority: {:?} {} (expected: Sacral)", 
                chart.authority, 
                if auth_ok { "✓" } else { "✗" }
            );
            
            let profile_ok = chart.profile.conscious_line == 2 && chart.profile.unconscious_line == 4;
            println!("Profile: {}/{} {} (expected: 2/4)", 
                chart.profile.conscious_line, chart.profile.unconscious_line,
                if profile_ok { "✓" } else { "✗" }
            );
            
            // Check incarnation cross gates
            if let (Some(ps), Some(pe), Some(ds), Some(de)) = (p_sun, p_earth, d_sun, d_earth) {
                // Expected: 4/49 | 23/43
                let cross_matches = 
                    ((ps.gate == 4 && pe.gate == 49) || (ps.gate == 49 && pe.gate == 4)) &&
                    ((ds.gate == 23 && de.gate == 43) || (ds.gate == 43 && de.gate == 23));
                println!("Cross gates: {}/{} | {}/{} {} (expected: 4/49 | 23/43)", 
                    ps.gate, pe.gate, ds.gate, de.gate,
                    if cross_matches { "✓" } else { "✗" }
                );
            }
            
            println!();
            if type_ok && auth_ok && profile_ok {
                println!("✅ Core chart elements VALIDATED!");
            } else {
                println!("⚠️  Some elements differ - checking calculation accuracy...");
            }
            
            // Print all activations for debugging
            println!();
            println!("=== ALL PERSONALITY ACTIVATIONS ===");
            for act in &chart.personality_activations {
                println!("  {:?}: Gate {}.{} ({:.2}°)", act.planet, act.gate, act.line, act.longitude);
            }
            
            println!();
            println!("=== ALL DESIGN ACTIVATIONS ===");
            for act in &chart.design_activations {
                println!("  {:?}: Gate {}.{} ({:.2}°)", act.planet, act.gate, act.line, act.longitude);
            }
        }
        Err(e) => {
            println!("Error generating chart: {:?}", e);
        }
    }
}
