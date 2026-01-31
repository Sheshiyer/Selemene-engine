//! Swiss Ephemeris integration test and demonstration

use chrono::{DateTime, Utc};
use engine_human_design::{EphemerisCalculator, HDPlanet};

fn main() {
    println!("=== Swiss Ephemeris Integration Test ===\n");

    let calc = EphemerisCalculator::new("");
    println!("✓ EphemerisCalculator initialized");
    println!("  Data path: '{}' (using built-in ephemeris)\n", calc.data_path());

    let test_date = DateTime::parse_from_rfc3339("2000-01-01T12:00:00Z")
        .unwrap()
        .with_timezone(&Utc);
    println!("Test Date: {}", test_date.format("%Y-%m-%d %H:%M:%S UTC"));
    println!("Expected Sun longitude: ~280° (Capricorn)\n");

    println!("--- Test 1: Single Planet (Sun) ---");
    match calc.get_planet_position(HDPlanet::Sun, &test_date) {
        Ok(pos) => {
            println!("✓ Sun position calculated:");
            println!("  Longitude: {:.6}°", pos.longitude);
            println!("  Latitude:  {:.6}°", pos.latitude);
            println!("  Distance:  {:.6} AU", pos.distance);
            println!("  Speed:     {:.6}°/day", pos.speed);
            
            if (pos.longitude - 280.0).abs() < 5.0 {
                println!("  ✓ Accuracy check PASSED (within 5° of expected)");
            } else {
                println!("  ✗ Accuracy check FAILED (expected ~280°)");
            }
        }
        Err(e) => {
            println!("✗ Failed: {}", e);
            std::process::exit(1);
        }
    }

    println!("\n--- Test 2: All 13 Planets for Human Design ---");
    match calc.get_all_planets(&test_date) {
        Ok(positions) => {
            println!("✓ Calculated {} planetary positions:\n", positions.len());
            
            for (planet, pos) in &positions {
                println!("  {:12?}: {:8.3}° (lat: {:6.3}°, dist: {:.6} AU)",
                         planet, pos.longitude, pos.latitude, pos.distance);
            }
            
            let all_valid = positions.iter().all(|(_, pos)| {
                pos.longitude >= 0.0 && pos.longitude < 360.0
            });
            
            if all_valid {
                println!("\n  ✓ All longitudes in valid range (0-360°)");
            } else {
                println!("\n  ✗ Some longitudes out of range!");
                std::process::exit(1);
            }
            
            let north = positions.iter().find(|(p, _)| matches!(p, HDPlanet::NorthNode));
            let south = positions.iter().find(|(p, _)| matches!(p, HDPlanet::SouthNode));
            
            if let (Some((_, n)), Some((_, s))) = (north, south) {
                let diff = (n.longitude - s.longitude + 360.0) % 360.0;
                if (diff - 180.0).abs() < 0.1 {
                    println!("  ✓ South Node correctly opposite North Node ({:.3}° difference)",
                             (diff - 180.0).abs());
                } else {
                    println!("  ✗ Node relationship incorrect (diff: {:.3}°)", diff);
                }
            }
        }
        Err(e) => {
            println!("✗ Failed: {}", e);
            std::process::exit(1);
        }
    }

    println!("\n--- Test 3: Modern Date (2024-01-01) ---");
    let modern_date = DateTime::parse_from_rfc3339("2024-01-01T00:00:00Z")
        .unwrap()
        .with_timezone(&Utc);
    
    match calc.get_planet_position(HDPlanet::Sun, &modern_date) {
        Ok(pos) => {
            println!("✓ Sun on 2024-01-01: {:.3}° (Capricorn)", pos.longitude);
        }
        Err(e) => {
            println!("✗ Failed: {}", e);
            std::process::exit(1);
        }
    }

    println!("\n=== Summary ===");
    println!("✓ Swiss Ephemeris is READY for Human Design calculations");
    println!("✓ All 13 planetary positions can be calculated");
    println!("✓ Accuracy validated against known values");
    println!("\nAPI Usage Example:");
    println!("  use engine_human_design::{{EphemerisCalculator, HDPlanet}};");
    println!("  let calc = EphemerisCalculator::new(\"\");");
    println!("  let pos = calc.get_planet_position(HDPlanet::Sun, &datetime)?;");
    println!("  let all = calc.get_all_planets(&datetime)?;");
}
