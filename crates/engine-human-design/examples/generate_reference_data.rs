use chrono::{Utc, TimeZone};
use engine_human_design::{generate_hd_chart, HDType, Authority, Definition, Planet};

fn main() {
    // Create diverse test cases
    let test_cases = vec![
        // Case 1: Generator
        ("Generator Test 1", Utc.with_ymd_and_hms(1985, 6, 15, 14, 30, 0).unwrap()),
        // Case 2: Projector
        ("Projector Test 1", Utc.with_ymd_and_hms(1990, 3, 21, 8, 15, 0).unwrap()),
        // Case 3: Manifestor
        ("Manifestor Test 1", Utc.with_ymd_and_hms(1978, 11, 5, 22, 45, 0).unwrap()),
        // Case 4: Reflector (rare - try multiple)
        ("Reflector Test 1", Utc.with_ymd_and_hms(1995, 2, 14, 3, 20, 0).unwrap()),
        // Case 5: Manifesting Generator
        ("MG Test 1", Utc.with_ymd_and_hms(1988, 9, 9, 18, 0, 0).unwrap()),
        // Case 6: Another Generator
        ("Generator Test 2", Utc.with_ymd_and_hms(1975, 4, 10, 6, 30, 0).unwrap()),
        // Case 7: Another Projector
        ("Projector Test 2", Utc.with_ymd_and_hms(1992, 7, 25, 16, 45, 0).unwrap()),
        // Case 8: Another Manifestor
        ("Manifestor Test 2", Utc.with_ymd_and_hms(1980, 12, 1, 10, 15, 0).unwrap()),
        // Case 9: Another MG
        ("MG Test 2", Utc.with_ymd_and_hms(2000, 1, 1, 0, 0, 0).unwrap()),
        // Case 10: Another Generator
        ("Generator Test 3", Utc.with_ymd_and_hms(1970, 8, 8, 20, 20, 0).unwrap()),
        // Case 11: Another Projector
        ("Projector Test 3", Utc.with_ymd_and_hms(1998, 5, 5, 12, 12, 0).unwrap()),
        // Case 12: Try for Reflector again
        ("Reflector Test 2", Utc.with_ymd_and_hms(1987, 10, 23, 1, 30, 0).unwrap()),
    ];

    println!("{{");
    println!("  \"charts\": [");
    
    for (i, (name, birth_time)) in test_cases.iter().enumerate() {
        match generate_hd_chart(*birth_time, "") {
            Ok(chart) => {
                let comma = if i < test_cases.len() - 1 { "," } else { "" };
                
                println!("    {{");
                println!("      \"name\": \"{}\",", name);
                println!("      \"birth_date\": \"{}\",", birth_time.format("%Y-%m-%d"));
                println!("      \"birth_time\": \"{}\",", birth_time.format("%H:%M:%S"));
                println!("      \"timezone\": \"UTC\",");
                println!("      \"latitude\": 0.0,");
                println!("      \"longitude\": 0.0,");
                println!("      \"expected\": {{");
                
                // Get personality and design Sun/Earth
                let p_sun = chart.personality_activations.iter().find(|a| matches!(a.planet, Planet::Sun)).unwrap();
                let p_earth = chart.personality_activations.iter().find(|a| matches!(a.planet, Planet::Earth)).unwrap();
                let d_sun = chart.design_activations.iter().find(|a| matches!(a.planet, Planet::Sun)).unwrap();
                let d_earth = chart.design_activations.iter().find(|a| matches!(a.planet, Planet::Earth)).unwrap();
                
                println!("        \"personality_sun\": {{\"gate\": {}, \"line\": {}}},", p_sun.gate, p_sun.line);
                println!("        \"personality_earth\": {{\"gate\": {}, \"line\": {}}},", p_earth.gate, p_earth.line);
                println!("        \"design_sun\": {{\"gate\": {}, \"line\": {}}},", d_sun.gate, d_sun.line);
                println!("        \"design_earth\": {{\"gate\": {}, \"line\": {}}},", d_earth.gate, d_earth.line);
                
                // Type and Authority
                let type_str = match chart.hd_type {
                    HDType::Generator => "Generator",
                    HDType::ManifestingGenerator => "ManifestingGenerator",
                    HDType::Projector => "Projector",
                    HDType::Manifestor => "Manifestor",
                    HDType::Reflector => "Reflector",
                };
                
                let auth_str = match chart.authority {
                    Authority::Emotional => "Emotional",
                    Authority::Sacral => "Sacral",
                    Authority::Splenic => "Splenic",
                    Authority::Heart => "Heart",
                    Authority::GCenter => "GCenter",
                    Authority::Mental => "Mental",
                    Authority::Lunar => "Lunar",
                };
                
                println!("        \"type\": \"{}\",", type_str);
                println!("        \"authority\": \"{}\",", auth_str);
                println!("        \"profile\": \"{}/{}\",", chart.profile.conscious_line, chart.profile.unconscious_line);
                
                // Defined centers
                let mut defined_centers: Vec<String> = chart.centers.iter()
                    .filter(|(_, info)| info.defined)
                    .map(|(center, _)| format!("{:?}", center))
                    .collect();
                defined_centers.sort();
                
                print!("        \"defined_centers\": [");
                for (j, center) in defined_centers.iter().enumerate() {
                    if j > 0 { print!(", "); }
                    print!("\"{}\"", center);
                }
                println!("],");
                
                // Active channels
                let mut channels: Vec<String> = chart.channels.iter()
                    .map(|ch| format!("{}-{}", ch.gate1, ch.gate2))
                    .collect();
                channels.sort();
                
                print!("        \"active_channels\": [");
                for (j, channel) in channels.iter().enumerate() {
                    if j > 0 { print!(", "); }
                    print!("\"{}\"", channel);
                }
                println!("]");
                
                println!("      }}");
                println!("    }}{}", comma);
                
                eprintln!("{}: Type={:?}, Authority={:?}, Profile={}/{}", 
                    name, chart.hd_type, chart.authority, 
                    chart.profile.conscious_line, chart.profile.unconscious_line);
            }
            Err(e) => {
                eprintln!("Error generating chart for {}: {}", name, e);
            }
        }
    }
    
    println!("  ]");
    println!("}}");
}
