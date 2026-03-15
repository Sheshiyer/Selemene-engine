use chrono::Utc;
use engine_human_design::{generate_hd_chart, Authority, Definition, HDType, Planet};

fn find_activation(
    activations: &[engine_human_design::models::Activation],
    planet: Planet,
) -> engine_human_design::models::Activation {
    activations
        .iter()
        .find(|a| a.planet == planet)
        .cloned()
        .expect("planet activation should exist")
}

fn assert_cross(
    chart: &engine_human_design::HDChart,
    personality: (u8, u8),
    design: (u8, u8),
) {
    let p_sun = find_activation(&chart.personality_activations, Planet::Sun);
    let p_earth = find_activation(&chart.personality_activations, Planet::Earth);
    let d_sun = find_activation(&chart.design_activations, Planet::Sun);
    let d_earth = find_activation(&chart.design_activations, Planet::Earth);

    assert_eq!((p_sun.gate, p_earth.gate), personality);
    assert_eq!((d_sun.gate, d_earth.gate), design);
}

#[test]
fn external_reference_1991_chart_matches_site() {
    let birth_time = chrono::TimeZone::with_ymd_and_hms(&Utc, 1991, 8, 13, 8, 1, 0).unwrap();
    let chart = generate_hd_chart(birth_time, "").expect("chart should generate");

    let p_sun = find_activation(&chart.personality_activations, Planet::Sun);
    let p_earth = find_activation(&chart.personality_activations, Planet::Earth);
    let d_sun = find_activation(&chart.design_activations, Planet::Sun);
    let d_earth = find_activation(&chart.design_activations, Planet::Earth);

    assert_eq!((p_sun.gate, p_sun.line), (4, 2));
    assert_eq!((p_earth.gate, p_earth.line), (49, 2));
    assert_eq!(find_activation(&chart.personality_activations, Planet::Moon).gate, 46);
    assert_eq!(
        (
            find_activation(&chart.personality_activations, Planet::NorthNode).gate,
            find_activation(&chart.personality_activations, Planet::NorthNode).line,
        ),
        (54, 4)
    );
    assert_eq!(
        (
            find_activation(&chart.personality_activations, Planet::SouthNode).gate,
            find_activation(&chart.personality_activations, Planet::SouthNode).line,
        ),
        (53, 4)
    );
    assert_eq!(find_activation(&chart.personality_activations, Planet::Mercury).gate, 59);
    assert_eq!(find_activation(&chart.personality_activations, Planet::Venus).gate, 59);
    assert_eq!(find_activation(&chart.personality_activations, Planet::Mars).gate, 47);
    assert_eq!(find_activation(&chart.personality_activations, Planet::Jupiter).gate, 4);
    assert_eq!(find_activation(&chart.personality_activations, Planet::Saturn).gate, 41);
    assert_eq!(find_activation(&chart.personality_activations, Planet::Uranus).gate, 38);
    assert_eq!(find_activation(&chart.personality_activations, Planet::Neptune).gate, 38);
    assert_eq!(find_activation(&chart.personality_activations, Planet::Pluto).gate, 1);
    assert_eq!((d_sun.gate, d_sun.line), (23, 4));
    assert_eq!((d_earth.gate, d_earth.line), (43, 4));
    assert_eq!(
        (
            find_activation(&chart.design_activations, Planet::NorthNode).gate,
            find_activation(&chart.design_activations, Planet::NorthNode).line,
        ),
        (54, 6)
    );
    assert_eq!(
        (
            find_activation(&chart.design_activations, Planet::SouthNode).gate,
            find_activation(&chart.design_activations, Planet::SouthNode).line,
        ),
        (53, 6)
    );
    assert_eq!(chart.hd_type, HDType::Generator);
    assert_eq!(chart.authority, Authority::Sacral);
    assert_eq!(chart.profile.conscious_line, 2);
    assert_eq!(chart.profile.unconscious_line, 4);
    assert_eq!(chart.definition, Definition::Split);
}

#[test]
fn external_reference_1987_structural_chart_matches_site() {
    let birth_time = chrono::TimeZone::with_ymd_and_hms(&Utc, 1987, 10, 15, 12, 5, 0).unwrap();
    let chart = generate_hd_chart(birth_time, "").expect("chart should generate");

    let p_sun = find_activation(&chart.personality_activations, Planet::Sun);
    let p_earth = find_activation(&chart.personality_activations, Planet::Earth);
    let d_sun = find_activation(&chart.design_activations, Planet::Sun);
    let d_earth = find_activation(&chart.design_activations, Planet::Earth);

    assert_eq!((p_sun.gate, p_sun.line), (32, 1));
    assert_eq!((p_earth.gate, p_earth.line), (42, 1));
    assert_eq!((d_sun.gate, d_sun.line), (62, 4));
    assert_eq!((d_earth.gate, d_earth.line), (61, 4));
    assert_eq!(
        (
            find_activation(&chart.personality_activations, Planet::NorthNode).gate,
            find_activation(&chart.personality_activations, Planet::NorthNode).line,
        ),
        (25, 5)
    );
    assert_eq!(
        (
            find_activation(&chart.personality_activations, Planet::SouthNode).gate,
            find_activation(&chart.personality_activations, Planet::SouthNode).line,
        ),
        (46, 5)
    );
    assert_eq!(
        (
            find_activation(&chart.design_activations, Planet::NorthNode).gate,
            find_activation(&chart.design_activations, Planet::NorthNode).line,
        ),
        (17, 2)
    );
    assert_eq!(
        (
            find_activation(&chart.design_activations, Planet::SouthNode).gate,
            find_activation(&chart.design_activations, Planet::SouthNode).line,
        ),
        (18, 2)
    );

    assert_eq!(chart.hd_type, HDType::Projector);
    assert_eq!(chart.authority, Authority::Splenic);
    assert_eq!(chart.profile.conscious_line, 1);
    assert_eq!(chart.profile.unconscious_line, 4);
    assert_eq!(chart.definition, Definition::Split);

    let active_channels: std::collections::BTreeSet<(u8, u8)> = chart
        .channels
        .iter()
        .map(|ch| (ch.gate1.min(ch.gate2), ch.gate1.max(ch.gate2)))
        .collect();

    assert!(active_channels.contains(&(17, 62)));
    assert!(active_channels.contains(&(18, 58)));
    assert!(active_channels.contains(&(26, 44)));
    assert!(active_channels.contains(&(11, 56)));
}

#[test]
fn external_reference_1954_structural_chart_matches_site() {
    let birth_time = chrono::TimeZone::with_ymd_and_hms(&Utc, 1954, 7, 12, 2, 0, 0).unwrap();
    let chart = generate_hd_chart(birth_time, "").expect("chart should generate");

    assert_cross(&chart, (53, 54), (42, 32));
    assert_eq!(chart.hd_type, HDType::Generator);
    assert_eq!(chart.authority, Authority::Sacral);
    assert_eq!(chart.profile.conscious_line, 5);
    assert_eq!(chart.profile.unconscious_line, 1);
    assert_eq!(chart.definition, Definition::Single);

    let active_channels: std::collections::BTreeSet<(u8, u8)> = chart
        .channels
        .iter()
        .map(|ch| (ch.gate1.min(ch.gate2), ch.gate1.max(ch.gate2)))
        .collect();

    assert!(active_channels.contains(&(10, 34)));
    assert!(active_channels.contains(&(28, 38)));
    assert!(active_channels.contains(&(32, 54)));
    assert!(active_channels.contains(&(42, 53)));
}

#[test]
fn external_reference_1990_structural_chart_matches_site() {
    let birth_time = chrono::TimeZone::with_ymd_and_hms(&Utc, 1990, 7, 25, 2, 0, 0).unwrap();
    let chart = generate_hd_chart(birth_time, "").expect("chart should generate");

    assert_cross(&chart, (56, 60), (27, 28));
    assert_eq!(chart.hd_type, HDType::Projector);
    assert_eq!(chart.authority, Authority::Splenic);
    assert_eq!(chart.profile.conscious_line, 6);
    assert_eq!(chart.profile.unconscious_line, 3);
    assert_eq!(chart.definition, Definition::Split);

    let active_channels: std::collections::BTreeSet<(u8, u8)> = chart
        .channels
        .iter()
        .map(|ch| (ch.gate1.min(ch.gate2), ch.gate1.max(ch.gate2)))
        .collect();

    assert!(active_channels.contains(&(24, 61)));
    assert!(active_channels.contains(&(28, 38)));
}

#[test]
fn external_reference_1999_structural_chart_matches_site() {
    let birth_time = chrono::TimeZone::with_ymd_and_hms(&Utc, 1999, 10, 31, 23, 25, 0).unwrap();
    let chart = generate_hd_chart(birth_time, "").expect("chart should generate");

    assert_cross(&chart, (44, 24), (33, 19));
    assert_eq!(chart.hd_type, HDType::Generator);
    assert_eq!(chart.authority, Authority::Sacral);
    assert_eq!(chart.profile.conscious_line, 1);
    assert_eq!(chart.profile.unconscious_line, 3);
    assert_eq!(chart.definition, Definition::Split);

    let active_channels: std::collections::BTreeSet<(u8, u8)> = chart
        .channels
        .iter()
        .map(|ch| (ch.gate1.min(ch.gate2), ch.gate1.max(ch.gate2)))
        .collect();

    assert!(active_channels.contains(&(3, 60)));
    assert!(active_channels.contains(&(13, 33)));
}

#[test]
fn external_reference_1997_structural_chart_matches_site() {
    let birth_time = chrono::TimeZone::with_ymd_and_hms(&Utc, 1997, 1, 22, 12, 0, 0).unwrap();
    let chart = generate_hd_chart(birth_time, "").expect("chart should generate");

    assert_cross(&chart, (41, 31), (28, 27));
    assert_eq!(chart.hd_type, HDType::Generator);
    assert_eq!(chart.authority, Authority::Sacral);
    assert_eq!(chart.profile.conscious_line, 1);
    assert_eq!(chart.profile.unconscious_line, 3);
    assert_eq!(chart.definition, Definition::Single);

    let active_channels: std::collections::BTreeSet<(u8, u8)> = chart
        .channels
        .iter()
        .map(|ch| (ch.gate1.min(ch.gate2), ch.gate1.max(ch.gate2)))
        .collect();

    assert!(active_channels.contains(&(18, 58)));
    assert!(active_channels.contains(&(27, 50)));
    assert!(active_channels.contains(&(28, 38)));
    assert!(active_channels.contains(&(29, 46)));
}
