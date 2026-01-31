//! Human Design Chart Analysis
//!
//! Implements complete HD chart analysis logic: center definition, channel activation,
//! Type/Authority/Profile determination, and Definition classification.

use std::collections::{HashMap, HashSet};
use crate::models::{
    HDChart, Activation, Center, CenterState, Channel, HDType, Authority, Profile, Definition, Planet,
};
use crate::wisdom_data::CHANNELS;

/// Analyze center definitions based on activated channels
///
/// A center is defined if at least one channel connecting it has both gates activated.
/// Returns a HashMap of all 9 centers with their definition state and active gates.
pub fn analyze_centers(activations: &[Activation]) -> HashMap<Center, CenterState> {
    let all_activations: Vec<&Activation> = activations.iter().collect();
    let activated_gates: HashSet<u8> = all_activations.iter().map(|a| a.gate).collect();
    
    // Track which gates activate each center based on channel connections
    let mut center_gates: HashMap<Center, HashSet<u8>> = HashMap::new();
    
    // Check all 36 channels
    for channel in CHANNELS.values() {
        if channel.gates.len() == 2 {
            let gate1 = channel.gates[0];
            let gate2 = channel.gates[1];
            
            // Channel is active if both gates are activated
            if activated_gates.contains(&gate1) && activated_gates.contains(&gate2) {
                // Add these gates to their respective centers
                for center_name in &channel.centers {
                    if let Some(center) = center_from_string(center_name) {
                        center_gates.entry(center)
                            .or_insert_with(HashSet::new)
                            .insert(gate1);
                        center_gates.entry(center)
                            .or_insert_with(HashSet::new)
                            .insert(gate2);
                    }
                }
            }
        }
    }
    
    // Create CenterState for all 9 centers
    let all_centers = [
        Center::Head,
        Center::Ajna,
        Center::Throat,
        Center::G,
        Center::Heart,
        Center::Spleen,
        Center::SolarPlexus,
        Center::Sacral,
        Center::Root,
    ];
    
    let mut result = HashMap::new();
    for center in &all_centers {
        let gates = center_gates.get(center)
            .map(|s| s.iter().copied().collect::<Vec<u8>>())
            .unwrap_or_default();
        
        result.insert(*center, CenterState {
            defined: !gates.is_empty(),
            gates,
        });
    }
    
    result
}

/// Analyze which channels are active
///
/// A channel is active if both of its gates are activated (from any activation source).
/// Returns a list of active channels with their metadata.
pub fn analyze_channels(activations: &[Activation]) -> Vec<Channel> {
    let activated_gates: HashSet<u8> = activations.iter().map(|a| a.gate).collect();
    
    let mut active_channels = Vec::new();
    
    for (_key, channel_wisdom) in CHANNELS.iter() {
        if channel_wisdom.gates.len() == 2 {
            let gate1 = channel_wisdom.gates[0];
            let gate2 = channel_wisdom.gates[1];
            
            if activated_gates.contains(&gate1) && activated_gates.contains(&gate2) {
                active_channels.push(Channel {
                    gate1,
                    gate2,
                    name: channel_wisdom.name.clone(),
                    circuitry: channel_wisdom.circuitry.clone(),
                });
            }
        }
    }
    
    active_channels
}

/// Determine HD Type based on center definitions
///
/// Type determination follows this logic:
/// - Reflector: All centers undefined
/// - Generator: Sacral defined, NOT connected to throat
/// - Manifesting Generator: Sacral defined AND connected to throat
/// - Manifestor: Throat connected to motor (Heart/Solar Plexus/Root) WITHOUT Sacral defined
/// - Projector: Sacral undefined, at least one other center defined
pub fn determine_type(centers: &HashMap<Center, CenterState>, channels: &[Channel]) -> HDType {
    let sacral_defined = centers.get(&Center::Sacral)
        .map(|c| c.defined)
        .unwrap_or(false);
    
    let throat_defined = centers.get(&Center::Throat)
        .map(|c| c.defined)
        .unwrap_or(false);
    
    // Check if any centers are defined
    let any_defined = centers.values().any(|c| c.defined);
    
    // Reflector: No centers defined
    if !any_defined {
        return HDType::Reflector;
    }
    
    // Check if Sacral is connected to Throat via a channel
    let sacral_to_throat = is_sacral_connected_to_throat(channels);
    
    // Check if Throat is connected to a motor center (excluding Sacral)
    let throat_to_motor = is_throat_connected_to_motor(channels);
    
    if sacral_defined {
        if sacral_to_throat {
            HDType::ManifestingGenerator
        } else {
            HDType::Generator
        }
    } else if throat_defined && throat_to_motor {
        HDType::Manifestor
    } else {
        HDType::Projector
    }
}

/// Determine Authority using hierarchical logic
///
/// Authority hierarchy (first match wins):
/// 1. Emotional: Solar Plexus defined
/// 2. Sacral: Sacral defined (if no Emotional)
/// 3. Splenic: Spleen defined (if no Emotional/Sacral)
/// 4. Heart (Ego): Heart defined (if no Emotional/Sacral/Splenic)
/// 5. GCenter (Self-Projected): G-Center defined, connected to Throat (if no others)
/// 6. Mental: Head/Ajna defined but no other awareness centers
/// 7. Lunar: No centers defined (Reflector only)
pub fn determine_authority(centers: &HashMap<Center, CenterState>, channels: &[Channel]) -> Authority {
    let solar_plexus_defined = centers.get(&Center::SolarPlexus).map(|c| c.defined).unwrap_or(false);
    let sacral_defined = centers.get(&Center::Sacral).map(|c| c.defined).unwrap_or(false);
    let spleen_defined = centers.get(&Center::Spleen).map(|c| c.defined).unwrap_or(false);
    let heart_defined = centers.get(&Center::Heart).map(|c| c.defined).unwrap_or(false);
    let g_defined = centers.get(&Center::G).map(|c| c.defined).unwrap_or(false);
    let throat_defined = centers.get(&Center::Throat).map(|c| c.defined).unwrap_or(false);
    let any_defined = centers.values().any(|c| c.defined);
    
    // 1. Emotional Authority (highest priority)
    if solar_plexus_defined {
        return Authority::Emotional;
    }
    
    // 2. Sacral Authority
    if sacral_defined {
        return Authority::Sacral;
    }
    
    // 3. Splenic Authority
    if spleen_defined {
        return Authority::Splenic;
    }
    
    // 4. Heart/Ego Authority
    if heart_defined {
        return Authority::Heart;
    }
    
    // 5. G-Center/Self-Projected Authority (needs connection to Throat)
    if g_defined && throat_defined && is_g_connected_to_throat(channels) {
        return Authority::GCenter;
    }
    
    // 6. Mental Authority (Head or Ajna defined but no awareness centers)
    let head_defined = centers.get(&Center::Head).map(|c| c.defined).unwrap_or(false);
    let ajna_defined = centers.get(&Center::Ajna).map(|c| c.defined).unwrap_or(false);
    
    if (head_defined || ajna_defined) && !solar_plexus_defined && !sacral_defined && !spleen_defined {
        return Authority::Mental;
    }
    
    // 7. Lunar Authority (no centers defined - Reflectors)
    if !any_defined {
        return Authority::Lunar;
    }
    
    // Default fallback (should rarely hit this)
    Authority::Mental
}

/// Calculate Profile from Personality Sun and Design Sun line numbers
///
/// Profile = Personality Sun line / Design Sun line
/// Example: Personality Sun Line 6 + Design Sun Line 2 = Profile 6/2
pub fn calculate_profile(personality: &[Activation], design: &[Activation]) -> Profile {
    // Find Personality Sun
    let conscious_line = personality
        .iter()
        .find(|a| a.planet == Planet::Sun)
        .map(|a| a.line)
        .unwrap_or(1);
    
    // Find Design Sun
    let unconscious_line = design
        .iter()
        .find(|a| a.planet == Planet::Sun)
        .map(|a| a.line)
        .unwrap_or(1);
    
    Profile {
        conscious_line,
        unconscious_line,
    }
}

/// Determine Definition type using graph traversal
///
/// Definition types:
/// - NoDefinition: No centers defined (Reflector)
/// - Single: All defined centers connected in one continuous group
/// - Split: Two separate groups of connected centers
/// - TripleSplit: Three separate groups
/// - QuadrupleSplit: Four separate groups
pub fn determine_definition(centers: &HashMap<Center, CenterState>, channels: &[Channel]) -> Definition {
    // Get all defined centers
    let defined_centers: HashSet<Center> = centers
        .iter()
        .filter(|(_, state)| state.defined)
        .map(|(center, _)| *center)
        .collect();
    
    if defined_centers.is_empty() {
        return Definition::NoDefinition;
    }
    
    // Build adjacency graph of connected centers via channels
    let mut adjacency: HashMap<Center, HashSet<Center>> = HashMap::new();
    
    for channel in channels {
        // Get centers connected by this channel
        if let Some(channel_wisdom) = CHANNELS.values().find(|c| {
            c.gates.len() == 2 && 
            ((c.gates[0] == channel.gate1 && c.gates[1] == channel.gate2) ||
             (c.gates[0] == channel.gate2 && c.gates[1] == channel.gate1))
        }) {
            if channel_wisdom.centers.len() == 2 {
                if let (Some(c1), Some(c2)) = (
                    center_from_string(&channel_wisdom.centers[0]),
                    center_from_string(&channel_wisdom.centers[1]),
                ) {
                    if defined_centers.contains(&c1) && defined_centers.contains(&c2) {
                        adjacency.entry(c1).or_insert_with(HashSet::new).insert(c2);
                        adjacency.entry(c2).or_insert_with(HashSet::new).insert(c1);
                    }
                }
            }
        }
    }
    
    // Find connected components using DFS
    let mut visited = HashSet::new();
    let mut component_count = 0;
    
    for center in &defined_centers {
        if !visited.contains(center) {
            component_count += 1;
            dfs(*center, &adjacency, &mut visited);
        }
    }
    
    match component_count {
        1 => Definition::Single,
        2 => Definition::Split,
        3 => Definition::TripleSplit,
        4 => Definition::QuadrupleSplit,
        _ => Definition::Single, // Fallback
    }
}

/// Master function to perform all chart analysis
///
/// Fills in centers, channels, type, authority, profile, and definition in the chart.
pub fn analyze_hd_chart(chart: &mut HDChart) -> Result<(), String> {
    // Combine all activations for analysis
    let all_activations: Vec<Activation> = chart.personality_activations
        .iter()
        .chain(chart.design_activations.iter())
        .cloned()
        .collect();
    
    // 1. Analyze centers
    chart.centers = analyze_centers(&all_activations);
    
    // 2. Analyze channels
    chart.channels = analyze_channels(&all_activations);
    
    // 3. Determine Type
    chart.hd_type = determine_type(&chart.centers, &chart.channels);
    
    // 4. Determine Authority
    chart.authority = determine_authority(&chart.centers, &chart.channels);
    
    // 5. Calculate Profile
    chart.profile = calculate_profile(&chart.personality_activations, &chart.design_activations);
    
    // 6. Determine Definition
    chart.definition = determine_definition(&chart.centers, &chart.channels);
    
    Ok(())
}

// Helper functions

fn center_from_string(name: &str) -> Option<Center> {
    match name {
        "Head" => Some(Center::Head),
        "Ajna" => Some(Center::Ajna),
        "Throat" => Some(Center::Throat),
        "G" => Some(Center::G),
        "Heart" | "Ego" => Some(Center::Heart),
        "Spleen" => Some(Center::Spleen),
        "SolarPlexus" | "Solar Plexus" | "Emotional" => Some(Center::SolarPlexus),
        "Sacral" => Some(Center::Sacral),
        "Root" => Some(Center::Root),
        _ => None,
    }
}

fn is_sacral_connected_to_throat(channels: &[Channel]) -> bool {
    // Gates connecting Sacral to Throat
    let sacral_throat_channels = [
        (5, 15),   // Channel 5-15
        (14, 2),   // Channel 2-14
        (29, 46),  // Channel 29-46
        (59, 6),   // Channel 6-59
        (34, 20),  // Channel 20-34 (via Throat)
        (34, 10),  // Channel 10-34 (via G to Throat)
        (34, 57),  // Channel 34-57
    ];
    
    for channel in channels {
        for (g1, g2) in &sacral_throat_channels {
            if (channel.gate1 == *g1 && channel.gate2 == *g2) ||
               (channel.gate1 == *g2 && channel.gate2 == *g1) {
                return true;
            }
        }
    }
    
    false
}

fn is_throat_connected_to_motor(channels: &[Channel]) -> bool {
    // Motor centers: Heart (21, 40, 51, 26), Solar Plexus (6, 37, 22, 36, 49, 55), Root (60, 52, 53, 54, 38, 39, 58, 41)
    let motor_gates = [
        21, 40, 51, 26, // Heart
        6, 37, 22, 36, 49, 55, // Solar Plexus
        60, 52, 53, 54, 38, 39, 58, 41, // Root
    ];
    
    let throat_gates = [62, 23, 56, 35, 12, 45, 33, 8, 31, 7, 1, 13, 16, 20];
    
    for channel in channels {
        let has_motor = motor_gates.contains(&channel.gate1) || motor_gates.contains(&channel.gate2);
        let has_throat = throat_gates.contains(&channel.gate1) || throat_gates.contains(&channel.gate2);
        
        if has_motor && has_throat {
            return true;
        }
    }
    
    false
}

fn is_g_connected_to_throat(channels: &[Channel]) -> bool {
    // G center gates: 1, 13, 25, 46, 2, 15, 10, 7
    // Throat gates: 62, 23, 56, 35, 12, 45, 33, 8, 31, 7, 1, 13, 16, 20
    let g_gates = [1, 13, 25, 46, 2, 15, 10, 7];
    let throat_gates = [62, 23, 56, 35, 12, 45, 33, 8, 31, 7, 1, 13, 16, 20];
    
    for channel in channels {
        let has_g = g_gates.contains(&channel.gate1) || g_gates.contains(&channel.gate2);
        let has_throat = throat_gates.contains(&channel.gate1) || throat_gates.contains(&channel.gate2);
        
        if has_g && has_throat {
            return true;
        }
    }
    
    false
}

fn dfs(center: Center, adjacency: &HashMap<Center, HashSet<Center>>, visited: &mut HashSet<Center>) {
    if visited.contains(&center) {
        return;
    }
    
    visited.insert(center);
    
    if let Some(neighbors) = adjacency.get(&center) {
        for neighbor in neighbors {
            dfs(*neighbor, adjacency, visited);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Planet;
    
    #[test]
    fn test_analyze_centers_undefined() {
        let activations = vec![
            Activation { planet: Planet::Sun, gate: 1, line: 1, longitude: 0.0 },
        ];
        
        let centers = analyze_centers(&activations);
        
        // With only one gate, no channels can be complete, so all centers should be undefined
        assert_eq!(centers.len(), 9);
        for (_, state) in centers.iter() {
            assert!(!state.defined, "Centers should be undefined with incomplete channels");
        }
    }
    
    #[test]
    fn test_analyze_channels_active() {
        // Channel 1-8 (G to Throat)
        let activations = vec![
            Activation { planet: Planet::Sun, gate: 1, line: 1, longitude: 0.0 },
            Activation { planet: Planet::Earth, gate: 8, line: 1, longitude: 180.0 },
        ];
        
        let channels = analyze_channels(&activations);
        
        // Should have at least one active channel
        assert!(!channels.is_empty(), "Should detect active channel 1-8");
        
        let has_1_8 = channels.iter().any(|c| {
            (c.gate1 == 1 && c.gate2 == 8) || (c.gate1 == 8 && c.gate2 == 1)
        });
        assert!(has_1_8, "Should find channel 1-8");
    }
    
    #[test]
    fn test_determine_type_reflector() {
        let centers = HashMap::new();
        let channels = vec![];
        
        let hd_type = determine_type(&centers, &channels);
        assert_eq!(hd_type, HDType::Reflector);
    }
    
    #[test]
    fn test_determine_authority_lunar() {
        let centers = HashMap::new();
        let channels = vec![];
        
        let authority = determine_authority(&centers, &channels);
        assert_eq!(authority, Authority::Lunar);
    }
    
    #[test]
    fn test_calculate_profile() {
        let personality = vec![
            Activation { planet: Planet::Sun, gate: 1, line: 6, longitude: 0.0 },
        ];
        let design = vec![
            Activation { planet: Planet::Sun, gate: 2, line: 2, longitude: 180.0 },
        ];
        
        let profile = calculate_profile(&personality, &design);
        assert_eq!(profile.conscious_line, 6);
        assert_eq!(profile.unconscious_line, 2);
    }
    
    #[test]
    fn test_determine_definition_no_definition() {
        let centers = HashMap::new();
        let channels = vec![];
        
        let definition = determine_definition(&centers, &channels);
        assert_eq!(definition, Definition::NoDefinition);
    }
}
