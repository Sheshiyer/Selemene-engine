use engine_human_design::{EphemerisCalculator, HDPlanet};
use chrono::{TimeZone, Utc};

fn main() {
    let calc = EphemerisCalculator::new("");
    let dt = Utc.with_ymd_and_hms(2000, 6, 21, 12, 0, 0).unwrap();
    
    println!("Testing date: {}", dt);
    
    match calc.get_planet_position(HDPlanet::Sun, &dt) {
        Ok(pos) => {
            println!("Sun longitude: {:.6}°", pos.longitude);
            let earth_lon = (pos.longitude + 180.0) % 360.0;
            println!("Earth longitude: {:.6}°", earth_lon);
            let diff = (earth_lon - pos.longitude + 360.0) % 360.0;
            println!("Difference: {:.6}°", diff);
        }
        Err(e) => {
            println!("Error: {}", e);
        }
    }
}
