use std::collections::HashMap;

use chrono::{TimeZone, Utc};
use engine_human_design::{calculate_design_time, HumanDesignEngine};
use noesis_core::{BirthData, ConsciousnessEngine, EngineInput, Precision};

#[derive(Clone)]
struct ActivationRef {
    gate: u64,
    line: u64,
}

struct ChartRef {
    label: &'static str,
    date: &'static str,
    time: &'static str,
    timezone: &'static str,
    latitude: f64,
    longitude: f64,
    hd_type: &'static str,
    authority: &'static str,
    profile: &'static str,
    definition: &'static str,
    cross: (&'static str, &'static str),
    personality: HashMap<&'static str, ActivationRef>,
    design: HashMap<&'static str, ActivationRef>,
}

fn activation(gate: u64, line: u64) -> ActivationRef {
    ActivationRef { gate, line }
}

fn shesh_reference() -> ChartRef {
    ChartRef {
        label: "1991-08-13 13:31 Asia/Kolkata",
        date: "1991-08-13",
        time: "13:31",
        timezone: "Asia/Kolkata",
        latitude: 12.9716,
        longitude: 77.5946,
        hd_type: "Generator",
        authority: "Sacral",
        profile: "2/4",
        definition: "Split",
        cross: ("4/49", "23/43"),
        personality: HashMap::from([
            ("sun", activation(4, 2)),
            ("earth", activation(49, 2)),
            ("northnode", activation(54, 4)),
            ("southnode", activation(53, 4)),
            ("moon", activation(46, 6)),
            ("mercury", activation(59, 5)),
            ("venus", activation(59, 5)),
            ("mars", activation(47, 1)),
            ("jupiter", activation(4, 5)),
            ("saturn", activation(41, 1)),
            ("uranus", activation(38, 1)),
            ("neptune", activation(38, 6)),
            ("pluto", activation(1, 5)),
        ]),
        design: HashMap::from([
            ("sun", activation(23, 4)),
            ("earth", activation(43, 4)),
            ("northnode", activation(54, 6)),
            ("southnode", activation(53, 6)),
            ("moon", activation(24, 4)),
            ("mercury", activation(42, 6)),
            ("venus", activation(52, 1)),
            ("mars", activation(62, 2)),
            ("jupiter", activation(31, 5)),
            ("saturn", activation(41, 6)),
            ("uranus", activation(38, 5)),
            ("neptune", activation(54, 2)),
            ("pluto", activation(43, 1)),
        ]),
    }
}

fn second_reference() -> ChartRef {
    ChartRef {
        label: "1987-10-15 17:35 Asia/Kolkata",
        date: "1987-10-15",
        time: "17:35",
        timezone: "Asia/Kolkata",
        latitude: 12.9716,
        longitude: 77.5946,
        hd_type: "Projector",
        authority: "Splenic",
        profile: "1/4",
        definition: "Split",
        cross: ("32/42", "62/61"),
        personality: HashMap::from([
            ("sun", activation(32, 1)),
            ("earth", activation(42, 1)),
            ("northnode", activation(25, 5)),
            ("southnode", activation(46, 5)),
            ("moon", activation(56, 4)),
            ("mercury", activation(44, 6)),
            ("venus", activation(28, 5)),
            ("mars", activation(18, 1)),
            ("jupiter", activation(42, 5)),
            ("saturn", activation(26, 1)),
            ("uranus", activation(11, 1)),
            ("neptune", activation(58, 2)),
            ("pluto", activation(44, 2)),
        ]),
        design: HashMap::from([
            ("sun", activation(62, 4)),
            ("earth", activation(61, 4)),
            ("northnode", activation(17, 2)),
            ("southnode", activation(18, 2)),
            ("moon", activation(17, 6)),
            ("mercury", activation(52, 4)),
            ("venus", activation(39, 5)),
            ("mars", activation(31, 5)),
            ("jupiter", activation(3, 2)),
            ("saturn", activation(5, 5)),
            ("uranus", activation(11, 2)),
            ("neptune", activation(58, 3)),
            ("pluto", activation(28, 6)),
        ]),
    }
}

fn third_reference() -> ChartRef {
    ChartRef {
        label: "1954-07-12 12:00 Australia/Sydney",
        date: "1954-07-12",
        time: "12:00",
        timezone: "Australia/Sydney",
        latitude: -33.8688,
        longitude: 151.2093,
        hd_type: "Generator",
        authority: "Sacral",
        profile: "5/1",
        definition: "Single",
        cross: ("53/54", "42/32"),
        personality: HashMap::from([
            ("sun", activation(53, 5)),
            ("earth", activation(54, 5)),
            ("northnode", activation(38, 6)),
            ("southnode", activation(39, 6)),
            ("moon", activation(34, 4)),
            ("mercury", activation(39, 1)),
            ("venus", activation(29, 5)),
            ("mars", activation(11, 6)),
            ("jupiter", activation(39, 2)),
            ("saturn", activation(28, 1)),
            ("uranus", activation(62, 3)),
            ("neptune", activation(32, 3)),
            ("pluto", activation(4, 5)),
        ]),
        design: HashMap::from([
            ("sun", activation(42, 1)),
            ("earth", activation(32, 1)),
            ("northnode", activation(54, 5)),
            ("southnode", activation(53, 5)),
            ("moon", activation(33, 1)),
            ("mercury", activation(36, 5)),
            ("venus", activation(24, 2)),
            ("mars", activation(10, 2)),
            ("jupiter", activation(45, 6)),
            ("saturn", activation(28, 6)),
            ("uranus", activation(53, 5)),
            ("neptune", activation(32, 5)),
            ("pluto", activation(4, 5)),
        ]),
    }
}

fn fourth_reference() -> ChartRef {
    ChartRef {
        label: "1990-07-25 07:30 Asia/Kolkata",
        date: "1990-07-25",
        time: "07:30",
        timezone: "Asia/Kolkata",
        latitude: 13.0827,
        longitude: 80.2707,
        hd_type: "Projector",
        authority: "Splenic",
        profile: "6/3",
        definition: "Split",
        cross: ("56/60", "27/28"),
        personality: HashMap::from([
            ("sun", activation(56, 6)),
            ("earth", activation(60, 6)),
            ("northnode", activation(41, 6)),
            ("southnode", activation(31, 6)),
            ("moon", activation(40, 5)),
            ("mercury", activation(4, 5)),
            ("venus", activation(52, 3)),
            ("mars", activation(24, 1)),
            ("jupiter", activation(62, 5)),
            ("saturn", activation(61, 1)),
            ("uranus", activation(58, 3)),
            ("neptune", activation(38, 4)),
            ("pluto", activation(1, 2)),
        ]),
        design: HashMap::from([
            ("sun", activation(27, 3)),
            ("earth", activation(28, 3)),
            ("northnode", activation(19, 6)),
            ("southnode", activation(33, 6)),
            ("moon", activation(42, 2)),
            ("mercury", activation(2, 5)),
            ("venus", activation(22, 3)),
            ("mars", activation(55, 3)),
            ("jupiter", activation(52, 3)),
            ("saturn", activation(61, 5)),
            ("uranus", activation(38, 1)),
            ("neptune", activation(38, 6)),
            ("pluto", activation(1, 4)),
        ]),
    }
}

fn fifth_reference() -> ChartRef {
    ChartRef {
        label: "1999-11-01 04:55 Asia/Kolkata",
        date: "1999-11-01",
        time: "04:55",
        timezone: "Asia/Kolkata",
        latitude: 15.3647,
        longitude: 75.1240,
        hd_type: "Generator",
        authority: "Sacral",
        profile: "1/3",
        definition: "Split",
        cross: ("44/24", "33/19"),
        personality: HashMap::from([
            ("sun", activation(44, 1)),
            ("earth", activation(24, 1)),
            ("northnode", activation(33, 1)),
            ("southnode", activation(19, 1)),
            ("moon", activation(7, 1)),
            ("mercury", activation(34, 1)),
            ("venus", activation(47, 5)),
            ("mars", activation(38, 2)),
            ("jupiter", activation(3, 3)),
            ("saturn", activation(2, 1)),
            ("uranus", activation(19, 6)),
            ("neptune", activation(60, 6)),
            ("pluto", activation(9, 4)),
        ]),
        design: HashMap::from([
            ("sun", activation(33, 3)),
            ("earth", activation(19, 3)),
            ("northnode", activation(33, 6)),
            ("southnode", activation(19, 6)),
            ("moon", activation(51, 1)),
            ("mercury", activation(56, 3)),
            ("venus", activation(59, 6)),
            ("mars", activation(44, 5)),
            ("jupiter", activation(27, 3)),
            ("saturn", activation(2, 4)),
            ("uranus", activation(13, 2)),
            ("neptune", activation(41, 1)),
            ("pluto", activation(9, 3)),
        ]),
    }
}

fn sixth_reference() -> ChartRef {
    ChartRef {
        label: "1997-01-22 15:00 Europe/Moscow",
        date: "1997-01-22",
        time: "15:00",
        timezone: "Europe/Moscow",
        latitude: 59.9311,
        longitude: 30.3609,
        hd_type: "Generator",
        authority: "Sacral",
        profile: "1/3",
        definition: "Single",
        cross: ("41/31", "28/27"),
        personality: HashMap::from([
            ("sun", activation(41, 1)),
            ("earth", activation(31, 1)),
            ("northnode", activation(46, 3)),
            ("southnode", activation(25, 3)),
            ("moon", activation(53, 6)),
            ("mercury", activation(58, 5)),
            ("venus", activation(54, 1)),
            ("mars", activation(18, 1)),
            ("jupiter", activation(60, 5)),
            ("saturn", activation(25, 5)),
            ("uranus", activation(41, 3)),
            ("neptune", activation(60, 2)),
            ("pluto", activation(34, 6)),
        ]),
        design: HashMap::from([
            ("sun", activation(28, 3)),
            ("earth", activation(27, 3)),
            ("northnode", activation(18, 5)),
            ("southnode", activation(17, 5)),
            ("moon", activation(2, 6)),
            ("mercury", activation(50, 5)),
            ("venus", activation(6, 6)),
            ("mars", activation(29, 5)),
            ("jupiter", activation(38, 3)),
            ("saturn", activation(25, 4)),
            ("uranus", activation(60, 5)),
            ("neptune", activation(61, 5)),
            ("pluto", activation(34, 2)),
        ]),
    }
}

fn extract_activation(
    chart: &serde_json::Value,
    bucket: &str,
    planet: &str,
) -> Option<(u64, u64)> {
    let obj = chart.get(bucket)?.get(planet)?;
    Some((obj.get("gate")?.as_u64()?, obj.get("line")?.as_u64()?))
}

async fn compare_reference(reference: &ChartRef) -> Result<(), String> {
    let engine = HumanDesignEngine::new();
    let input = EngineInput {
        birth_data: Some(BirthData {
            name: Some(reference.label.to_string()),
            date: reference.date.to_string(),
            time: Some(reference.time.to_string()),
            latitude: reference.latitude,
            longitude: reference.longitude,
            timezone: reference.timezone.to_string(),
        }),
        current_time: chrono::Utc::now(),
        location: None,
        precision: Precision::Standard,
        options: HashMap::new(),
    };

    let output = engine
        .calculate(input)
        .await
        .map_err(|e| format!("engine calculation failed: {}", e))?;

    println!("\n=== {} ===", reference.label);

    let tz: chrono_tz::Tz = reference
        .timezone
        .parse()
        .map_err(|e| format!("invalid timezone: {}", e))?;
    let naive_dt = chrono::NaiveDate::parse_from_str(reference.date, "%Y-%m-%d")
        .map_err(|e| format!("invalid date: {}", e))?
        .and_time(
            chrono::NaiveTime::parse_from_str(reference.time, "%H:%M")
                .map_err(|e| format!("invalid time: {}", e))?,
        );
    let birth_utc = tz
        .from_local_datetime(&naive_dt)
        .single()
        .ok_or_else(|| "ambiguous local birth time".to_string())?
        .with_timezone(&Utc);
    let design_utc = calculate_design_time(birth_utc, Some(""))
        .map_err(|e| format!("design time calculation failed: {}", e))?;
    let design_local = design_utc.with_timezone(&tz);

    println!("Birth UTC: {}", birth_utc);
    println!("Design UTC: {}", design_utc);
    println!("Design local: {}", design_local);

    let actual_type = output.result["hd_type"].as_str().unwrap_or("<missing>");
    let actual_authority = output.result["authority"].as_str().unwrap_or("<missing>");
    let actual_profile = output.result["profile"].as_str().unwrap_or("<missing>");
    let actual_definition = output.result["definition"].as_str().unwrap_or("<missing>");
    println!(
        "Defined centers: {}",
        output.result["defined_centers"]
            .as_array()
            .map(|xs| xs
                .iter()
                .filter_map(|v| v.as_str())
                .collect::<Vec<_>>()
                .join(", "))
            .unwrap_or_else(|| "<missing>".to_string())
    );
    println!(
        "Active channels: {}",
        output.result["active_channels"]
            .as_array()
            .map(|xs| xs
                .iter()
                .filter_map(|v| v.as_str())
                .collect::<Vec<_>>()
                .join(", "))
            .unwrap_or_else(|| "<missing>".to_string())
    );

    let personality_sun = extract_activation(&output.result, "personality_activations", "sun")
        .ok_or_else(|| "missing personality sun".to_string())?;
    let personality_earth =
        extract_activation(&output.result, "personality_activations", "earth")
            .ok_or_else(|| "missing personality earth".to_string())?;
    let design_sun = extract_activation(&output.result, "design_activations", "sun")
        .ok_or_else(|| "missing design sun".to_string())?;
    let design_earth = extract_activation(&output.result, "design_activations", "earth")
        .ok_or_else(|| "missing design earth".to_string())?;

    let actual_cross_left = format!("{}/{}", personality_sun.0, personality_earth.0);
    let actual_cross_right = format!("{}/{}", design_sun.0, design_earth.0);

    println!(
        "Type: {} [{}]",
        actual_type,
        if actual_type == reference.hd_type {
            "match"
        } else {
            "mismatch"
        }
    );
    println!(
        "Authority: {} [{}]",
        actual_authority,
        if actual_authority == reference.authority {
            "match"
        } else {
            "mismatch"
        }
    );
    println!(
        "Profile: {} [{}]",
        actual_profile,
        if actual_profile == reference.profile {
            "match"
        } else {
            "mismatch"
        }
    );
    println!(
        "Definition: {} [{}]",
        actual_definition,
        if actual_definition == reference.definition {
            "match"
        } else {
            "mismatch"
        }
    );
    println!(
        "Incarnation cross: {}/{} [{}]",
        actual_cross_left,
        actual_cross_right,
        if actual_cross_left == reference.cross.0 && actual_cross_right == reference.cross.1 {
            "match"
        } else {
            "mismatch"
        }
    );

    let mut p_matches = 0usize;
    for (planet, expected) in &reference.personality {
        let actual = extract_activation(&output.result, "personality_activations", planet);
        match actual {
            Some((gate, line)) if gate == expected.gate && line == expected.line => p_matches += 1,
            _ => {}
        }
    }

    let mut d_matches = 0usize;
    for (planet, expected) in &reference.design {
        let actual = extract_activation(&output.result, "design_activations", planet);
        match actual {
            Some((gate, line)) if gate == expected.gate && line == expected.line => d_matches += 1,
            _ => {}
        }
    }

    println!("Personality activations matched: {}/13", p_matches);
    println!("Design activations matched: {}/13", d_matches);

    println!("\nPersonality mismatches:");
    for (planet, expected) in &reference.personality {
        let actual = extract_activation(&output.result, "personality_activations", planet);
        if let Some((gate, line)) = actual {
            if gate != expected.gate || line != expected.line {
                println!(
                    "  {} expected {}.{} got {}.{}",
                    planet, expected.gate, expected.line, gate, line
                );
            }
        }
    }

    println!("Design mismatches:");
    for (planet, expected) in &reference.design {
        let actual = extract_activation(&output.result, "design_activations", planet);
        if let Some((gate, line)) = actual {
            if gate != expected.gate || line != expected.line {
                println!(
                    "  {} expected {}.{} got {}.{}",
                    planet, expected.gate, expected.line, gate, line
                );
            }
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), String> {
    compare_reference(&shesh_reference()).await?;
    compare_reference(&second_reference()).await?;
    compare_reference(&third_reference()).await?;
    compare_reference(&fourth_reference()).await?;
    compare_reference(&fifth_reference()).await?;
    compare_reference(&sixth_reference()).await?;
    Ok(())
}
