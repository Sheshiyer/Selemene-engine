use chrono::{Utc, TimeZone};
use engine_human_design::{generate_hd_chart, HDType, Authority};
use std::collections::{HashSet, HashMap};

fn main() {
    let mut found_types: HashMap<String, Vec<String>> = HashMap::new();
    let mut found_authorities: HashSet<String> = HashSet::new();
    let mut found_profiles: HashSet<String> = HashSet::new();
    
    // Try many different dates/times
    for year in [1970, 1975, 1980, 1985, 1990, 1995, 2000, 2005].iter() {
        for month in 1..=12 {
            for day in [1, 5, 10, 15, 20, 25].iter() {
                for hour in [0, 3, 6, 9, 12, 15, 18, 21].iter() {
                    for minute in [0, 15, 30, 45].iter() {
                        if let Some(birth_time) = Utc.with_ymd_and_hms(*year, month, *day, *hour, *minute, 0).single() {
                            if let Ok(chart) = generate_hd_chart(birth_time, "") {
                                let type_str = format!("{:?}", chart.hd_type);
                                let auth_str = format!("{:?}", chart.authority);
                                let profile_str = format!("{}/{}", chart.profile.conscious_line, chart.profile.unconscious_line);
                                let channels_count = chart.channels.len();
                                let defined_centers: Vec<_> = chart.centers.iter()
                                    .filter(|(_, info)| info.defined)
                                    .map(|(center, _)| format!("{:?}", center))
                                    .collect();
                                
                                // Only include charts with some definition
                                if channels_count > 0 {
                                    let key = format!("{}-{}-{}", type_str, auth_str, profile_str);
                                    let entry = format!("{} {} {} {:02}:{:02} | Type={} Auth={} Prof={} Channels={} DefCenters={}",
                                        year, month, day, hour, minute,
                                        type_str, auth_str, profile_str, channels_count, defined_centers.len()
                                    );
                                    
                                    found_types.entry(key.clone()).or_insert_with(Vec::new).push(entry);
                                    found_authorities.insert(auth_str);
                                    found_profiles.insert(profile_str);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    eprintln!("Found {} unique Type-Authority-Profile combinations", found_types.len());
    eprintln!("Found {} unique Authorities: {:?}", found_authorities.len(), found_authorities);
    eprintln!("Found {} unique Profiles: {:?}", found_profiles.len(), found_profiles);
    
    // Print first example of each type-authority-profile combo
    let mut sorted_keys: Vec<_> = found_types.keys().collect();
    sorted_keys.sort();
    
    for key in sorted_keys {
        if let Some(examples) = found_types.get(key) {
            if let Some(first) = examples.first() {
                println!("{}", first);
            }
        }
    }
}
