//! Human Design Chart Analysis
//!
//! Implements complete HD chart analysis logic: center definition, channel activation,
//! Type/Authority/Profile determination, and Definition classification.

use crate::models::{
    Activation, Authority, Center, CenterState, Channel, Definition, HDChart, HDType, Planet,
    Profile,
};
use std::collections::{HashMap, HashSet};

struct ChannelSpec {
    gate1: u8,
    gate2: u8,
    centers: [Center; 2],
    circuitry: &'static str,
}

const CANONICAL_CHANNEL_SPECS: &[ChannelSpec] = &[
    ChannelSpec { gate1: 64, gate2: 47, centers: [Center::Head, Center::Ajna], circuitry: "Collective" },
    ChannelSpec { gate1: 61, gate2: 24, centers: [Center::Head, Center::Ajna], circuitry: "Individual" },
    ChannelSpec { gate1: 63, gate2: 4, centers: [Center::Head, Center::Ajna], circuitry: "Collective" },
    ChannelSpec { gate1: 17, gate2: 62, centers: [Center::Ajna, Center::Throat], circuitry: "Collective" },
    ChannelSpec { gate1: 43, gate2: 23, centers: [Center::Ajna, Center::Throat], circuitry: "Individual" },
    ChannelSpec { gate1: 11, gate2: 56, centers: [Center::Ajna, Center::Throat], circuitry: "Collective" },
    ChannelSpec { gate1: 48, gate2: 16, centers: [Center::Spleen, Center::Throat], circuitry: "Collective" },
    ChannelSpec { gate1: 57, gate2: 20, centers: [Center::Spleen, Center::Throat], circuitry: "Individual" },
    ChannelSpec { gate1: 20, gate2: 10, centers: [Center::Throat, Center::G], circuitry: "Individual" },
    ChannelSpec { gate1: 31, gate2: 7, centers: [Center::Throat, Center::G], circuitry: "Collective" },
    ChannelSpec { gate1: 8, gate2: 1, centers: [Center::Throat, Center::G], circuitry: "Individual" },
    ChannelSpec { gate1: 33, gate2: 13, centers: [Center::Throat, Center::G], circuitry: "Collective" },
    ChannelSpec { gate1: 45, gate2: 21, centers: [Center::Throat, Center::Heart], circuitry: "Tribal" },
    ChannelSpec { gate1: 12, gate2: 22, centers: [Center::Throat, Center::SolarPlexus], circuitry: "Individual" },
    ChannelSpec { gate1: 35, gate2: 36, centers: [Center::Throat, Center::SolarPlexus], circuitry: "Collective" },
    ChannelSpec { gate1: 34, gate2: 20, centers: [Center::Sacral, Center::Throat], circuitry: "Individual" },
    ChannelSpec { gate1: 2, gate2: 14, centers: [Center::G, Center::Sacral], circuitry: "Individual" },
    ChannelSpec { gate1: 29, gate2: 46, centers: [Center::Sacral, Center::G], circuitry: "Collective" },
    ChannelSpec { gate1: 5, gate2: 15, centers: [Center::Sacral, Center::G], circuitry: "Collective" },
    ChannelSpec { gate1: 10, gate2: 34, centers: [Center::G, Center::Sacral], circuitry: "Individual" },
    ChannelSpec { gate1: 10, gate2: 57, centers: [Center::G, Center::Spleen], circuitry: "Individual" },
    ChannelSpec { gate1: 25, gate2: 51, centers: [Center::G, Center::Heart], circuitry: "Individual" },
    ChannelSpec { gate1: 26, gate2: 44, centers: [Center::Heart, Center::Spleen], circuitry: "Tribal" },
    ChannelSpec { gate1: 27, gate2: 50, centers: [Center::Sacral, Center::Spleen], circuitry: "Tribal" },
    ChannelSpec { gate1: 28, gate2: 38, centers: [Center::Spleen, Center::Root], circuitry: "Individual" },
    ChannelSpec { gate1: 32, gate2: 54, centers: [Center::Spleen, Center::Root], circuitry: "Tribal" },
    ChannelSpec { gate1: 18, gate2: 58, centers: [Center::Spleen, Center::Root], circuitry: "Collective" },
    ChannelSpec { gate1: 59, gate2: 6, centers: [Center::Sacral, Center::SolarPlexus], circuitry: "Tribal" },
    ChannelSpec { gate1: 37, gate2: 40, centers: [Center::SolarPlexus, Center::Heart], circuitry: "Tribal" },
    ChannelSpec { gate1: 19, gate2: 49, centers: [Center::Root, Center::SolarPlexus], circuitry: "Tribal" },
    ChannelSpec { gate1: 39, gate2: 55, centers: [Center::Root, Center::SolarPlexus], circuitry: "Individual" },
    ChannelSpec { gate1: 41, gate2: 30, centers: [Center::Root, Center::SolarPlexus], circuitry: "Collective" },
    ChannelSpec { gate1: 3, gate2: 60, centers: [Center::Sacral, Center::Root], circuitry: "Individual" },
    ChannelSpec { gate1: 42, gate2: 53, centers: [Center::Sacral, Center::Root], circuitry: "Collective" },
    ChannelSpec { gate1: 9, gate2: 52, centers: [Center::Sacral, Center::Root], circuitry: "Collective" },
    ChannelSpec { gate1: 34, gate2: 57, centers: [Center::Sacral, Center::Spleen], circuitry: "Individual" },
];

/// Analyze center definitions based on activated channels
///
/// A center is defined if at least one channel connecting it has both gates activated.
/// Returns a HashMap of all 9 centers with their definition state and active gates.
pub fn analyze_centers(activations: &[Activation]) -> HashMap<Center, CenterState> {
    let all_activations: Vec<&Activation> = activations.iter().collect();
    let activated_gates: HashSet<u8> = all_activations.iter().map(|a| a.gate).collect();

    // Track which gates activate each center based on channel connections
    let mut center_gates: HashMap<Center, HashSet<u8>> = HashMap::new();

    // Check all 36 canonical channels.
    for channel in CANONICAL_CHANNEL_SPECS {
        if activated_gates.contains(&channel.gate1) && activated_gates.contains(&channel.gate2) {
            center_gates
                .entry(channel.centers[0])
                .or_default()
                .insert(channel.gate1);
            center_gates
                .entry(channel.centers[1])
                .or_default()
                .insert(channel.gate2);
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
        let gates = center_gates
            .get(center)
            .map(|s| s.iter().copied().collect::<Vec<u8>>())
            .unwrap_or_default();

        result.insert(
            *center,
            CenterState {
                defined: !gates.is_empty(),
                gates,
            },
        );
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

    for channel_spec in CANONICAL_CHANNEL_SPECS {
        if activated_gates.contains(&channel_spec.gate1) && activated_gates.contains(&channel_spec.gate2) {
            active_channels.push(Channel {
                gate1: channel_spec.gate1,
                gate2: channel_spec.gate2,
                name: format!("Channel {}-{}", channel_spec.gate1, channel_spec.gate2),
                circuitry: channel_spec.circuitry.to_string(),
            });
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
    let sacral_defined = centers
        .get(&Center::Sacral)
        .map(|c| c.defined)
        .unwrap_or(false);

    let throat_defined = centers
        .get(&Center::Throat)
        .map(|c| c.defined)
        .unwrap_or(false);

    // Check if any centers are defined
    let any_defined = centers.values().any(|c| c.defined);

    // Reflector: No centers defined
    if !any_defined {
        return HDType::Reflector;
    }

    // Check if Throat is connected to a motor center (excluding Sacral)
    let throat_to_motor = is_throat_connected_to_motor(channels);

    if sacral_defined {
        if throat_to_motor {
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
pub fn determine_authority(
    centers: &HashMap<Center, CenterState>,
    channels: &[Channel],
) -> Authority {
    let solar_plexus_defined = centers
        .get(&Center::SolarPlexus)
        .map(|c| c.defined)
        .unwrap_or(false);
    let sacral_defined = centers
        .get(&Center::Sacral)
        .map(|c| c.defined)
        .unwrap_or(false);
    let spleen_defined = centers
        .get(&Center::Spleen)
        .map(|c| c.defined)
        .unwrap_or(false);
    let heart_defined = centers
        .get(&Center::Heart)
        .map(|c| c.defined)
        .unwrap_or(false);
    let g_defined = centers.get(&Center::G).map(|c| c.defined).unwrap_or(false);
    let throat_defined = centers
        .get(&Center::Throat)
        .map(|c| c.defined)
        .unwrap_or(false);
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
    let head_defined = centers
        .get(&Center::Head)
        .map(|c| c.defined)
        .unwrap_or(false);
    let ajna_defined = centers
        .get(&Center::Ajna)
        .map(|c| c.defined)
        .unwrap_or(false);

    if (head_defined || ajna_defined) && !solar_plexus_defined && !sacral_defined && !spleen_defined
    {
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
pub fn determine_definition(
    centers: &HashMap<Center, CenterState>,
    channels: &[Channel],
) -> Definition {
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
        if let Some(channel_spec) = CANONICAL_CHANNEL_SPECS.iter().find(|spec| {
            (spec.gate1 == channel.gate1 && spec.gate2 == channel.gate2)
                || (spec.gate1 == channel.gate2 && spec.gate2 == channel.gate1)
        }) {
            let c1 = channel_spec.centers[0];
            let c2 = channel_spec.centers[1];
            if defined_centers.contains(&c1) && defined_centers.contains(&c2) {
                adjacency.entry(c1).or_default().insert(c2);
                adjacency.entry(c2).or_default().insert(c1);
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
    let all_activations: Vec<Activation> = chart
        .personality_activations
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

fn is_throat_connected_to_motor(channels: &[Channel]) -> bool {
    let motor_centers = [
        Center::Heart,
        Center::SolarPlexus,
        Center::Root,
        Center::Sacral,
    ];

    channels.iter().any(|channel| {
        CANONICAL_CHANNEL_SPECS.iter().any(|spec| {
            let same_channel = (spec.gate1 == channel.gate1 && spec.gate2 == channel.gate2)
                || (spec.gate1 == channel.gate2 && spec.gate2 == channel.gate1);
            if !same_channel {
                return false;
            }

            let has_throat = spec.centers.contains(&Center::Throat);
            let has_motor = spec.centers.iter().any(|center| motor_centers.contains(center));
            has_throat && has_motor
        })
    })
}

fn is_g_connected_to_throat(channels: &[Channel]) -> bool {
    // G center gates: 1, 13, 25, 46, 2, 15, 10, 7
    // Throat gates: 62, 23, 56, 35, 12, 45, 33, 8, 31, 7, 1, 13, 16, 20
    let g_gates = [1, 13, 25, 46, 2, 15, 10, 7];
    let throat_gates = [62, 23, 56, 35, 12, 45, 33, 8, 31, 7, 1, 13, 16, 20];

    for channel in channels {
        let has_g = g_gates.contains(&channel.gate1) || g_gates.contains(&channel.gate2);
        let has_throat =
            throat_gates.contains(&channel.gate1) || throat_gates.contains(&channel.gate2);

        if has_g && has_throat {
            return true;
        }
    }

    false
}

fn dfs(
    center: Center,
    adjacency: &HashMap<Center, HashSet<Center>>,
    visited: &mut HashSet<Center>,
) {
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
        let activations = vec![Activation {
            planet: Planet::Sun,
            gate: 1,
            line: 1,
            longitude: 0.0,
        }];

        let centers = analyze_centers(&activations);

        // With only one gate, no channels can be complete, so all centers should be undefined
        assert_eq!(centers.len(), 9);
        for (_, state) in centers.iter() {
            assert!(
                !state.defined,
                "Centers should be undefined with incomplete channels"
            );
        }
    }

    #[test]
    fn test_analyze_channels_active() {
        // Channel 1-8 (G to Throat)
        let activations = vec![
            Activation {
                planet: Planet::Sun,
                gate: 1,
                line: 1,
                longitude: 0.0,
            },
            Activation {
                planet: Planet::Earth,
                gate: 8,
                line: 1,
                longitude: 180.0,
            },
        ];

        let channels = analyze_channels(&activations);

        // Should have at least one active channel
        assert!(!channels.is_empty(), "Should detect active channel 1-8");

        let has_1_8 = channels
            .iter()
            .any(|c| (c.gate1 == 1 && c.gate2 == 8) || (c.gate1 == 8 && c.gate2 == 1));
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
    fn test_determine_type_generator_without_motor_to_throat() {
        let centers = HashMap::from([
            (
                Center::Sacral,
                CenterState {
                    defined: true,
                    gates: vec![34, 42],
                },
            ),
            (
                Center::G,
                CenterState {
                    defined: true,
                    gates: vec![10],
                },
            ),
            (
                Center::Spleen,
                CenterState {
                    defined: true,
                    gates: vec![28, 32],
                },
            ),
            (
                Center::Root,
                CenterState {
                    defined: true,
                    gates: vec![38, 53, 54],
                },
            ),
        ]);
        let channels = vec![
            Channel {
                gate1: 10,
                gate2: 34,
                name: "Channel 10-34".to_string(),
                circuitry: "Individual".to_string(),
            },
            Channel {
                gate1: 28,
                gate2: 38,
                name: "Channel 28-38".to_string(),
                circuitry: "Individual".to_string(),
            },
            Channel {
                gate1: 32,
                gate2: 54,
                name: "Channel 32-54".to_string(),
                circuitry: "Tribal".to_string(),
            },
            Channel {
                gate1: 42,
                gate2: 53,
                name: "Channel 42-53".to_string(),
                circuitry: "Collective".to_string(),
            },
        ];

        let hd_type = determine_type(&centers, &channels);
        assert_eq!(hd_type, HDType::Generator);
    }

    #[test]
    fn test_determine_type_manifesting_generator_with_motor_to_throat() {
        let centers = HashMap::from([
            (
                Center::Sacral,
                CenterState {
                    defined: true,
                    gates: vec![34],
                },
            ),
            (
                Center::Throat,
                CenterState {
                    defined: true,
                    gates: vec![20],
                },
            ),
        ]);
        let channels = vec![Channel {
            gate1: 34,
            gate2: 20,
            name: "Channel 34-20".to_string(),
            circuitry: "Individual".to_string(),
        }];

        let hd_type = determine_type(&centers, &channels);
        assert_eq!(hd_type, HDType::ManifestingGenerator);
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
        let personality = vec![Activation {
            planet: Planet::Sun,
            gate: 1,
            line: 6,
            longitude: 0.0,
        }];
        let design = vec![Activation {
            planet: Planet::Sun,
            gate: 2,
            line: 2,
            longitude: 180.0,
        }];

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
