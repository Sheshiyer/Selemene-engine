use chrono::{Utc, TimeZone};
use engine_human_design::{EphemerisCalculator, calculate_personality_activations};

fn main() {
    let calculator = EphemerisCalculator::new("");
    let birth_time = Utc.with_ymd_and_hms(2000, 6, 21, 12, 0, 0).unwrap();
    
    match calculate_personality_activations(&birth_time, &calculator) {
        Ok(activations) => {
            println!("Got {} activations:", activations.len());
            for (i, act) in activations.iter().enumerate() {
                println!("{}: {:?} -> Gate {}.{} at {:.4}°", i, act.planet, act.gate, act.line, act.longitude);
            }
        }
        Err(e) => {
            println!("Error: {:?}", e);
        }
    }
}
