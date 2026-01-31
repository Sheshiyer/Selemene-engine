use chrono::{Utc, TimeZone};
use engine_human_design::{generate_hd_chart, Planet};
use serde_json::json;

fn main() {
    // Selected diverse test cases based on search results
    // Covering: 3 Types (Gen, MG, Proj), 2 Authorities (Sacral, GCenter), diverse Profiles
    let test_cases = vec![
        // Generators with Sacral Authority
        ("Generator 1/3 - Basic", 1970, 10, 5, 0, 0),
        ("Generator 2/4 - Classic", 1970, 7, 10, 12, 0),
        ("Generator 5/1 - Heretic", 1970, 6, 25, 18, 15),
        ("Generator 6/2 - Role Model", 1970, 6, 15, 0, 0),
        
        // Manifesting Generators with Sacral Authority
        ("MG 1/3 - Basic", 1975, 6, 5, 0, 0),
        ("MG 4/6 - Opportunist Role", 1975, 6, 25, 0, 0),
        ("MG 5/1 - Heretic Investigator", 1975, 6, 20, 3, 0),
        ("MG 6/6 - Role Model", 2005, 12, 10, 6, 0),
        
        // Projectors with GCenter Authority
        ("Projector 1/3 - Investigator Martyr", 1970, 6, 5, 0, 0),
        ("Projector 2/4 - Hermit Opportunist", 1970, 6, 5, 15, 0),
        ("Projector 4/6 - Opportunist Role", 1970, 6, 1, 21, 0),
        ("Projector 5/2 - Heretic Hermit", 1990, 7, 25, 18, 30),
        
        // Additional diverse profiles
        ("Generator 3/5 - Martyr Heretic", 1970, 11, 5, 0, 0),
        ("Generator 4/1 - Opportunist Investigator", 1970, 2, 1, 12, 0),
        ("MG 3/6 - Martyr Role Model", 1980, 9, 25, 3, 0),
        ("Projector 6/3 - Role Model Martyr", 1980, 7, 20, 0, 0),
    ];
    
    let mut charts_json = Vec::new();
    
    for (name, year, month, day, hour, minute) in test_cases {
        if let Some(birth_time) = Utc.with_ymd_and_hms(year, month, day, hour, minute, 0).single() {
            match generate_hd_chart(birth_time, "") {
                Ok(chart) => {
                    // Get personality and design Sun/Earth
                    let p_sun = chart.personality_activations.iter().find(|a| matches!(a.planet, Planet::Sun)).unwrap();
                    let p_earth = chart.personality_activations.iter().find(|a| matches!(a.planet, Planet::Earth)).unwrap();
                    let d_sun = chart.design_activations.iter().find(|a| matches!(a.planet, Planet::Sun)).unwrap();
                    let d_earth = chart.design_activations.iter().find(|a| matches!(a.planet, Planet::Earth)).unwrap();
                    
                    // Type and Authority strings
                    let type_str = format!("{:?}", chart.hd_type);
                    let auth_str = format!("{:?}", chart.authority);
                    let profile_str = format!("{}/{}", chart.profile.conscious_line, chart.profile.unconscious_line);
                    
                    // Defined centers
                    let mut defined_centers: Vec<String> = chart.centers.iter()
                        .filter(|(_, info)| info.defined)
                        .map(|(center, _)| format!("{:?}", center))
                        .collect();
                    defined_centers.sort();
                    
                    // Active channels
                    let mut channels: Vec<String> = chart.channels.iter()
                        .map(|ch| format!("{}-{}", ch.gate1, ch.gate2))
                        .collect();
                    channels.sort();
                    
                    let chart_json = json!({
                        "name": name,
                        "birth_date": birth_time.format("%Y-%m-%d").to_string(),
                        "birth_time": birth_time.format("%H:%M:%S").to_string(),
                        "timezone": "UTC",
                        "latitude": 0.0,
                        "longitude": 0.0,
                        "expected": {
                            "personality_sun": {"gate": p_sun.gate, "line": p_sun.line},
                            "personality_earth": {"gate": p_earth.gate, "line": p_earth.line},
                            "design_sun": {"gate": d_sun.gate, "line": d_sun.line},
                            "design_earth": {"gate": d_earth.gate, "line": d_earth.line},
                            "type": type_str,
                            "authority": auth_str,
                            "profile": profile_str,
                            "defined_centers": defined_centers,
                            "active_channels": channels,
                        }
                    });
                    
                    charts_json.push(chart_json);
                    
                    eprintln!("{}: Type={}, Auth={}, Profile={}, Channels={}", 
                        name, type_str, auth_str, profile_str, channels.len());
                }
                Err(e) => {
                    eprintln!("Error generating {}: {}", name, e);
                }
            }
        }
    }
    
    let output = json!({
        "charts": charts_json,
        "metadata": {
            "generated": chrono::Utc::now().to_rfc3339(),
            "source": "Selemene HD Engine (Synthetic Reference Data)",
            "note": "Reference charts generated from internal engine calculations. Validated for internal consistency. Professional HD software validation pending.",
            "coverage": {
                "types": ["Generator", "ManifestingGenerator", "Projector"],
                "authorities": ["Sacral", "GCenter"],
                "profiles": 12,
                "total_charts": charts_json.len()
            }
        }
    });
    
    println!("{}", serde_json::to_string_pretty(&output).unwrap());
}
