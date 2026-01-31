//! Example demonstrating Vimshottari Dasha calculation
//!
//! Shows birth nakshatra calculation, balance determination, and Mahadasha generation

use chrono::{TimeZone, Utc};
use engine_vimshottari::{
    calculate_birth_nakshatra, calculate_dasha_balance, calculate_mahadashas,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Vimshottari Dasha Calculator Demo ===\n");

    // Example birth data
    let birth_time = Utc.with_ymd_and_hms(1985, 6, 15, 14, 30, 0).unwrap();
    let ephe_path = ""; // Use built-in Swiss Ephemeris data

    println!("Birth Time: {}", birth_time);
    println!("Calculating birth nakshatra from Moon position...\n");

    // Step 1: Calculate birth nakshatra
    match calculate_birth_nakshatra(birth_time, ephe_path) {
        Ok(nakshatra) => {
            println!("✅ Birth Nakshatra: #{} - {}", nakshatra.number, nakshatra.name);
            println!("   Ruling Planet: {:?}", nakshatra.ruling_planet);
            println!("   Degree Range: {:.3}° - {:.3}°", nakshatra.start_degree, nakshatra.end_degree);
            println!("   Deity: {}", nakshatra.deity);
            println!("   Symbol: {}", nakshatra.symbol);
            println!("   Description: {}\n", nakshatra.description);

            // For this demo, let's use a known Moon longitude for testing
            // In real calculation, this comes from Swiss Ephemeris
            let demo_moon_longitude = 125.0; // Example: 125° (in Magha nakshatra)
            
            println!("Demo Moon Longitude: {:.3}°", demo_moon_longitude);

            // Step 2: Calculate dasha balance
            let balance = calculate_dasha_balance(demo_moon_longitude, &nakshatra);
            println!("✅ Dasha Balance (first period remaining): {:.3} years\n", balance);

            // Step 3: Generate all 9 Mahadasha periods
            let starting_planet = nakshatra.ruling_planet;
            let mahadashas = calculate_mahadashas(birth_time, starting_planet, balance);

            println!("✅ Generated {} Mahadasha Periods (120-year cycle):\n", mahadashas.len());
            
            let mut total_years = 0.0;
            for (i, dasha) in mahadashas.iter().enumerate() {
                println!("{}. {:?} Mahadasha", i + 1, dasha.planet);
                println!("   Duration: {:.3} years", dasha.duration_years);
                println!("   Start: {}", dasha.start_date.format("%Y-%m-%d"));
                println!("   End: {}", dasha.end_date.format("%Y-%m-%d"));
                
                if i == 0 {
                    println!("   (Partial period - balance only)");
                }
                println!();
                
                total_years += dasha.duration_years;
            }

            println!("Total Cycle Duration: {:.1} years\n", total_years);
            
            // Show the sequence of planets
            println!("Planetary Sequence:");
            for (i, dasha) in mahadashas.iter().enumerate() {
                print!("{:?}", dasha.planet);
                if i < mahadashas.len() - 1 {
                    print!(" → ");
                }
            }
            println!("\n");

            println!("✅ All calculations complete!");
        }
        Err(e) => {
            eprintln!("❌ Error calculating nakshatra: {}", e);
            eprintln!("\nNote: Make sure Swiss Ephemeris data files are available.");
            eprintln!("The calculation requires Moon position which needs ephemeris data.");
        }
    }

    Ok(())
}
