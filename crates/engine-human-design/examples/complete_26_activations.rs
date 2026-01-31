//! Complete example: Generate HD chart with all 26 planetary activations
//!
//! Demonstrates the full activation calculation for Personality (birth time)
//! and Design (88 days before birth).

use chrono::{Utc, TimeZone};
use engine_human_design::generate_hd_chart;

fn main() {
    println!("=== Human Design: Complete 26 Planetary Activations ===\n");
    
    // Example birth time: May 15, 1990 at 2:30 PM UTC
    let birth_time = Utc.with_ymd_and_hms(1990, 5, 15, 14, 30, 0).unwrap();
    
    println!("Birth Time: {}", birth_time);
    println!("Calculating all 26 activations (13 Personality + 13 Design)...\n");
    
    match generate_hd_chart(birth_time, "") {
        Ok(chart) => {
            println!("✅ Chart generated successfully!\n");
            
            // Display Personality activations (at birth)
            println!("━━━ PERSONALITY ACTIVATIONS (Birth Time) ━━━");
            for act in &chart.personality_activations {
                println!(
                    "  {:12} → Gate {:<2}.{} ({:6.2}°)",
                    format!("{:?}", act.planet),
                    act.gate,
                    act.line,
                    act.longitude
                );
            }
            
            println!();
            
            // Display Design activations (88 days before birth)
            println!("━━━ DESIGN ACTIVATIONS (88° Solar Arc Before Birth) ━━━");
            for act in &chart.design_activations {
                println!(
                    "  {:12} → Gate {:<2}.{} ({:6.2}°)",
                    format!("{:?}", act.planet),
                    act.gate,
                    act.line,
                    act.longitude
                );
            }
            
            println!("\n━━━ SUMMARY ━━━");
            println!("Total activations: {}", 
                chart.personality_activations.len() + chart.design_activations.len());
            println!("Personality: {} planets", chart.personality_activations.len());
            println!("Design: {} planets", chart.design_activations.len());
            
            // Verify Sun/Earth opposition in both
            let pers_sun = &chart.personality_activations[0];
            let pers_earth = &chart.personality_activations[1];
            let pers_diff = (pers_earth.longitude - pers_sun.longitude + 360.0) % 360.0;
            
            let des_sun = &chart.design_activations[0];
            let des_earth = &chart.design_activations[1];
            let des_diff = (des_earth.longitude - des_sun.longitude + 360.0) % 360.0;
            
            println!("\n━━━ VERIFICATION ━━━");
            println!("Personality Sun/Earth opposition: {:.4}° (expect 180.0°)", pers_diff);
            println!("Design Sun/Earth opposition: {:.4}° (expect 180.0°)", des_diff);
            
            if (pers_diff - 180.0).abs() < 0.01 && (des_diff - 180.0).abs() < 0.01 {
                println!("✅ All oppositions verified!");
            }
        }
        Err(e) => {
            eprintln!("❌ Error generating chart: {}", e);
            eprintln!("Note: Swiss Ephemeris data files may not be available");
        }
    }
}
