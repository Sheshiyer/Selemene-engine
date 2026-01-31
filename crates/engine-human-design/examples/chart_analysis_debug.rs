use engine_human_design::{generate_hd_chart};
use chrono::{TimeZone, Utc};

fn main() {
    let birth_time = Utc.with_ymd_and_hms(1990, 5, 15, 14, 30, 0).unwrap();
    
    match generate_hd_chart(birth_time, "") {
        Ok(chart) => {
            println!("\n=== PERSONALITY ACTIVATIONS ===");
            for act in &chart.personality_activations {
                println!("{:?}: Gate {} Line {}", act.planet, act.gate, act.line);
            }
            
            println!("\n=== DESIGN ACTIVATIONS ===");
            for act in &chart.design_activations {
                println!("{:?}: Gate {} Line {}", act.planet, act.gate, act.line);
            }
            
            println!("\n=== ALL ACTIVATED GATES ===");
            let mut all_gates: Vec<u8> = chart.personality_activations.iter()
                .chain(chart.design_activations.iter())
                .map(|a| a.gate)
                .collect();
            all_gates.sort();
            all_gates.dedup();
            println!("{:?}", all_gates);
            
            println!("\n=== CHANNELS ===");
            println!("Active channels: {}", chart.channels.len());
            for channel in &chart.channels {
                println!("  {}-{}: {}", channel.gate1, channel.gate2, channel.name);
            }
            
            println!("\n=== CENTERS ===");
            for (center, state) in &chart.centers {
                println!("{:?}: {} (gates: {:?})", center, 
                    if state.defined { "DEFINED" } else { "UNDEFINED" },
                    state.gates);
            }
            
            println!("\n=== SUMMARY ===");
            println!("Type: {:?}", chart.hd_type);
            println!("Authority: {:?}", chart.authority);
            println!("Profile: {}/{}", chart.profile.conscious_line, chart.profile.unconscious_line);
            println!("Definition: {:?}", chart.definition);
        }
        Err(e) => eprintln!("Error: {}", e),
    }
}
