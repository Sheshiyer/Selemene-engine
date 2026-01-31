use chrono::{DateTime, Utc, TimeZone};
use engine_human_design::EphemerisCalculator;

fn main() {
    let calculator = EphemerisCalculator::new("");
    let birth_time = Utc.with_ymd_and_hms(2000, 6, 21, 12, 0, 0).unwrap();
    
    let planets = calculator.get_all_planets(&birth_time).unwrap();
    
    for (i, (planet, pos)) in planets.iter().enumerate() {
        println!("{}: {:?} -> lon={:.4}", i, planet, pos.longitude);
    }
}
