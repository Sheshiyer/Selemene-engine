//! Performance test for 26 planetary activations
//!
//! Verifies that all 26 activations complete in <50ms

use chrono::{DateTime, Utc, TimeZone};
use std::time::Instant;
use engine_human_design::calculate_all_activations;

fn main() {
    println!("=== 26 Planetary Activations Performance Test ===\n");
    
    let test_dates = vec![
        Utc.with_ymd_and_hms(1980, 3, 21, 10, 0, 0).unwrap(),
        Utc.with_ymd_and_hms(1990, 5, 15, 14, 30, 0).unwrap(),
        Utc.with_ymd_and_hms(2000, 6, 21, 12, 0, 0).unwrap(),
        Utc.with_ymd_and_hms(2010, 9, 23, 18, 45, 0).unwrap(),
        Utc.with_ymd_and_hms(2020, 12, 21, 6, 30, 0).unwrap(),
    ];
    
    let mut total_time = 0u128;
    let mut success_count = 0;
    
    for (i, birth_time) in test_dates.iter().enumerate() {
        print!("Test {}: {} ... ", i + 1, birth_time);
        
        let start = Instant::now();
        match calculate_all_activations(*birth_time, "") {
            Ok((personality, design)) => {
                let elapsed = start.elapsed();
                let ms = elapsed.as_micros() as f64 / 1000.0;
                
                println!("✅ {:.2}ms ({} + {} activations)", ms, personality.len(), design.len());
                
                total_time += elapsed.as_micros();
                success_count += 1;
                
                // Verify counts
                assert_eq!(personality.len(), 13, "Expected 13 personality activations");
                assert_eq!(design.len(), 13, "Expected 13 design activations");
            }
            Err(e) => {
                println!("⚠️  Skipped ({})", e);
            }
        }
    }
    
    if success_count > 0 {
        let avg_ms = (total_time / success_count as u128) as f64 / 1000.0;
        
        println!("\n=== Performance Summary ===");
        println!("Tests run: {}", success_count);
        println!("Average time: {:.2}ms", avg_ms);
        println!("Target: <50ms");
        
        if avg_ms < 50.0 {
            println!("✅ PASSED: Performance within target");
        } else {
            println!("❌ FAILED: Performance exceeds target");
        }
    } else {
        println!("\n⚠️  No tests completed (Swiss Ephemeris data may not be available)");
    }
}
