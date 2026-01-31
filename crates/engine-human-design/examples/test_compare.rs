use chrono::{Utc, TimeZone};
use engine_human_design::{EphemerisCalculator, calculate_personality_sun_earth, calculate_personality_activations};

fn main() {
    let calculator = EphemerisCalculator::new("");
    let birth_time = Utc.with_ymd_and_hms(2000, 6, 21, 12, 0, 0).unwrap();
    
    // Agent 19's original function
    match calculate_personality_sun_earth(&birth_time, &calculator) {
        Ok((sun, earth)) => {
            println!("Agent 19 (original):");
            println!("  Sun: Gate {}.{} at {:.4}°", sun.gate, sun.line, sun.longitude);
            println!("  Earth: Gate {}.{} at {:.4}°", earth.gate, earth.line, earth.longitude);
        }
        Err(e) => {
            println!("Agent 19 error: {:?}", e);
        }
    }
    
    println!();
    
    // Agent 20's new function
    match calculate_personality_activations(&birth_time, &calculator) {
        Ok(activations) => {
            println!("Agent 20 (new):");
            println!("  Sun: Gate {}.{} at {:.4}°", activations[0].gate, activations[0].line, activations[0].longitude);
            println!("  Earth: Gate {}.{} at {:.4}°", activations[1].gate, activations[1].line, activations[1].longitude);
        }
        Err(e) => {
            println!("Agent 20 error: {:?}", e);
        }
    }
}
