//! Example: Calculate Sun and Earth activations for Personality and Design
//!
//! This example demonstrates how to calculate the fundamental Sun/Earth activations
//! for both the conscious Personality (at birth) and unconscious Design (88 days before).

use engine_human_design::{
    EphemerisCalculator,
    calculate_sun_earth_activations,
};
use chrono::{TimeZone, Utc};

fn main() {
    println!("=== Human Design Sun/Earth Activation Calculator ===\n");
    
    // Initialize Swiss Ephemeris calculator
    // Use empty string for built-in ephemeris or provide path to data files
    let calculator = EphemerisCalculator::new("");
    
    // Example birth time
    let birth_time = Utc.with_ymd_and_hms(1990, 6, 15, 14, 30, 0).unwrap();
    
    println!("Birth Time: {} UTC\n", birth_time);
    
    // Calculate all Sun/Earth activations
    match calculate_sun_earth_activations(&birth_time, &calculator) {
        Ok(((pers_sun, pers_earth), (des_sun, des_earth))) => {
            println!("PERSONALITY (Conscious - at birth time)");
            println!("========================================");
            println!("Sun   → Gate {}.{} at {:.4}° longitude", 
                pers_sun.gate, pers_sun.line, pers_sun.longitude);
            println!("Earth → Gate {}.{} at {:.4}° longitude", 
                pers_earth.gate, pers_earth.line, pers_earth.longitude);
            
            println!("\nDESIGN (Unconscious - 88 days before birth)");
            println!("============================================");
            println!("Sun   → Gate {}.{} at {:.4}° longitude", 
                des_sun.gate, des_sun.line, des_sun.longitude);
            println!("Earth → Gate {}.{} at {:.4}° longitude", 
                des_earth.gate, des_earth.line, des_earth.longitude);
            
            println!("\nVERIFICATION");
            println!("============");
            
            // Verify Earth oppositions
            let pers_diff = (pers_earth.longitude - pers_sun.longitude + 360.0) % 360.0;
            let des_diff = (des_earth.longitude - des_sun.longitude + 360.0) % 360.0;
            
            println!("Personality: Earth is {:.4}° from Sun (should be 180°)", pers_diff);
            println!("Design: Earth is {:.4}° from Sun (should be 180°)", des_diff);
            
            // Calculate Sun movement
            let sun_movement = (pers_sun.longitude - des_sun.longitude + 360.0) % 360.0;
            println!("\nSun moved {:.2}° between Design and Personality", sun_movement);
            
            println!("\n✓ Calculation successful!");
        }
        Err(e) => {
            eprintln!("Error calculating activations: {}", e);
            eprintln!("\nNote: This example requires Swiss Ephemeris data.");
            eprintln!("The library will attempt to use built-in data, but external");
            eprintln!("ephemeris files may be needed for full accuracy.");
        }
    }
    
    println!("\n=== Additional Test: Multiple Birth Dates ===\n");
    
    let test_dates = vec![
        ("Spring Equinox 2000", Utc.with_ymd_and_hms(2000, 3, 20, 7, 35, 0).unwrap()),
        ("Summer Solstice 2000", Utc.with_ymd_and_hms(2000, 6, 21, 1, 48, 0).unwrap()),
        ("Random Date 1995", Utc.with_ymd_and_hms(1995, 11, 23, 18, 15, 0).unwrap()),
    ];
    
    for (label, birth_time) in test_dates {
        println!("{}", label);
        match calculate_sun_earth_activations(&birth_time, &calculator) {
            Ok(((pers_sun, pers_earth), (des_sun, des_earth))) => {
                println!("  P: Sun {}.{} / Earth {}.{}  |  D: Sun {}.{} / Earth {}.{}", 
                    pers_sun.gate, pers_sun.line,
                    pers_earth.gate, pers_earth.line,
                    des_sun.gate, des_sun.line,
                    des_earth.gate, des_earth.line);
            }
            Err(e) => {
                println!("  Error: {}", e);
            }
        }
    }
}
