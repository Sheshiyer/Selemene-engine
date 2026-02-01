use chrono::{TimeZone, Utc, Duration};
use engine_human_design::design_time::calculate_design_time;

fn main() {
    // Shesh birth: Aug 13, 1991, 08:01 UTC
    let birth_time = Utc.with_ymd_and_hms(1991, 8, 13, 8, 1, 0).unwrap();
    
    // Expected design date from profile: May 13, 1991, 08:28 UTC
    let expected_design = Utc.with_ymd_and_hms(1991, 5, 13, 8, 28, 0).unwrap();
    
    println!("Birth time: {}", birth_time);
    println!("Expected design time (from profile): {}", expected_design);
    println!("Days between: {}", (birth_time - expected_design).num_days());
    
    // Calculate using our engine
    match calculate_design_time(birth_time, Some("")) {
        Ok(calculated_design) => {
            println!("\nCalculated design time: {}", calculated_design);
            println!("Difference from expected: {} hours", 
                (calculated_design - expected_design).num_hours());
        }
        Err(e) => {
            println!("\nError calculating design time: {:?}", e);
        }
    }
}
