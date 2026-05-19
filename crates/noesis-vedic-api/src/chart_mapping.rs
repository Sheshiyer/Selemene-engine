//! Mappers from upstream FreeAstrologyAPI JSON envelopes into the typed
//! `chart::BirthChart` / `chart::NavamsaChart` shapes this crate exposes.
//!
//! Vendor schema summary:
//!
//! - `POST /planets` returns `{ statusCode, input, output: [ { "0": {...}, "1": {...}, ... } ] }`.
//!   `output` is a single-element array whose only element is a map keyed by
//!   stringified indices `"0".."12"`. Entry `"0"` is the Ascendant. An extra
//!   entry at `"13"` carries the ayanamsha value.
//!
//! - `POST /navamsa-chart-info` (and other `/d{N}-chart-info` routes) return
//!   `{ statusCode, output: { "0": {...}, ... } }` — no array wrapper.
//!   Divisional entries lack `fullDegree` / `normDegree`.
//!
//! Sign numbers are 1-indexed (1 = Aries, 12 = Pisces). `isRetro` is a
//! stringified bool (`"true"` / `"false"`).

use serde_json::Value;

use crate::chart::{
    AscendantInfo, BirthChart, HousePosition, HouseType, MoonInfo, NativeInfo, NavamsaChart,
    NavamsaPosition, PlanetPosition, SpecialPoints, ZodiacSign,
};
use crate::error::{VedicApiError, VedicApiResult};

/// Stringified indices `"0".."13"`, pre-allocated so the hot mapping loops
/// don't allocate one fresh `String` per planet per call.
const PLANET_KEYS: [&str; 14] = [
    "0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "11", "12", "13",
];

/// Map the `/planets` envelope into a typed `BirthChart`.
pub fn map_planets_envelope_to_birth_chart(
    raw: &Value,
    native: NativeInfo,
) -> VedicApiResult<BirthChart> {
    let output = raw.get("output").ok_or_else(|| VedicApiError::Parse {
        message: "/planets envelope missing 'output' field".to_string(),
    })?;
    let planets_map = output
        .as_array()
        .and_then(|arr| arr.first())
        .ok_or_else(|| VedicApiError::Parse {
            message: "/planets envelope 'output' is not a non-empty array".to_string(),
        })?;

    let ascendant_raw = planets_map
        .get(PLANET_KEYS[0])
        .ok_or_else(|| VedicApiError::Parse {
            message: "/planets envelope missing Ascendant at output[0][\"0\"]".to_string(),
        })?;
    let ascendant_sign = read_sign(ascendant_raw, "Ascendant")?;
    let ascendant_norm = read_f64(ascendant_raw, "normDegree").unwrap_or(0.0);

    // Single pass over keys "1".."13": real planets get appended, the
    // synthetic "ayanamsa" entry at "13" gets harvested.
    let mut planets: Vec<PlanetPosition> = Vec::with_capacity(13);
    let mut ayanamsa = 0.0_f64;
    for idx in 1u8..=13 {
        let entry = match planets_map.get(PLANET_KEYS[idx as usize]) {
            Some(e) => e,
            None => continue,
        };
        if idx == 13 {
            ayanamsa = entry.get("value").and_then(Value::as_f64).unwrap_or(0.0);
            continue;
        }
        // Skip non-planet sentinels at this level (real planets always carry a `current_sign`).
        if entry.get("current_sign").is_none() {
            continue;
        }
        planets.push(planet_from_entry(entry)?);
    }

    let moon = planets
        .iter()
        .find(|p| p.name.eq_ignore_ascii_case("Moon"))
        .map(|p| MoonInfo {
            sign: p.sign,
            degree: p.degree,
            nakshatra: String::new(),
            pada: 0,
            rashi_lord: p.sign.ruler().to_string(),
        })
        .unwrap_or_else(|| MoonInfo {
            sign: ZodiacSign::Aries,
            degree: 0.0,
            nakshatra: String::new(),
            pada: 0,
            rashi_lord: ZodiacSign::Aries.ruler().to_string(),
        });

    // Whole-sign houses keyed from the Ascendant sign (Vedic default). The
    // vendor's `/western/houses` endpoint returns Placidus cusps with real
    // degrees — use it when you need those.
    let asc_idx = ascendant_sign.index();
    let houses: Vec<HousePosition> = (0..12u8)
        .map(|i| {
            let sign = ZodiacSign::from_index(asc_idx + i as usize);
            let house_number = i + 1;
            HousePosition {
                number: house_number,
                sign,
                cusp: (sign.index() as f64) * 30.0,
                degree: 0.0,
                house_type: classify_house(house_number),
                is_kendra: matches!(house_number, 1 | 4 | 7 | 10),
                is_panapara: matches!(house_number, 2 | 5 | 8 | 11),
                is_apoklima: matches!(house_number, 3 | 6 | 9 | 12),
            }
        })
        .collect();

    Ok(BirthChart {
        native,
        ayanamsa,
        house_system: "whole-sign".to_string(),
        planets,
        houses,
        ascendant: AscendantInfo {
            sign: ascendant_sign,
            degree: ascendant_norm,
            nakshatra: String::new(),
            pada: 0,
        },
        moon,
        special_points: SpecialPoints {
            lagna: read_f64(ascendant_raw, "fullDegree").unwrap_or(0.0),
            midheaven: None,
            part_of_fortune: None,
        },
        notes: Vec::new(),
    })
}

/// Map a `/navamsa-chart-info` (or any `/d{N}-chart-info`) envelope into a
/// typed `NavamsaChart`.
///
/// Divisional endpoints return only `name`, `current_sign`, `house_number`,
/// and `isRetro` per planet — no degrees.
pub fn map_navamsa_envelope_to_navamsa_chart(
    raw: &Value,
    source: NativeInfo,
) -> VedicApiResult<NavamsaChart> {
    let output = raw.get("output").ok_or_else(|| VedicApiError::Parse {
        message: "/navamsa-chart-info envelope missing 'output' field".to_string(),
    })?;

    let ascendant_raw = output
        .get(PLANET_KEYS[0])
        .ok_or_else(|| VedicApiError::Parse {
            message: "/navamsa-chart-info envelope missing Ascendant at output[\"0\"]".to_string(),
        })?;
    let d9_lagna = read_sign(ascendant_raw, "Ascendant")?;

    let mut positions: Vec<NavamsaPosition> = Vec::with_capacity(12);
    for idx in 1u8..=12 {
        let entry = match output.get(PLANET_KEYS[idx as usize]) {
            Some(e) => e,
            None => continue,
        };
        if entry.get("current_sign").is_none() {
            continue;
        }
        let name = read_str(entry, "name")?;
        let sign = read_sign(entry, &name)?;
        positions.push(NavamsaPosition {
            planet: name,
            sign,
            degree: 0.0,
            is_vargottama: false,
        });
    }

    Ok(NavamsaChart {
        source,
        navamsa_positions: positions,
        vargottama: Vec::new(),
        d9_lagna,
    })
}

// ---- internal helpers --------------------------------------------------

fn planet_from_entry(entry: &Value) -> VedicApiResult<PlanetPosition> {
    let name = read_str(entry, "name")?;
    let sign = read_sign(entry, &name)?;
    let full = read_f64(entry, "fullDegree").unwrap_or(0.0);
    let norm = read_f64(entry, "normDegree").unwrap_or(0.0);
    let minutes = (norm.fract() * 60.0).abs();
    let house = entry
        .get("house_number")
        .and_then(Value::as_u64)
        .unwrap_or(0) as u8;
    let is_retrograde = parse_retro(entry, &name)?;

    // TODO(PR2/PR3): nakshatra, pada, speed, latitude, combust come from
    // native engines (engine-panchanga + Swiss ephemeris); the vendor's
    // `/planets` endpoint does not supply them.
    Ok(PlanetPosition {
        name,
        longitude: full,
        sign,
        degree: norm,
        minutes,
        house,
        is_retrograde,
        is_combust: false,
        nakshatra: String::new(),
        pada: 0,
        speed: 0.0,
        latitude: 0.0,
    })
}

/// Strict parser for the vendor's stringified `isRetro` field. Accepts only
/// `"true"` / `"false"` — any other value (including a missing field) is a
/// `Parse` error rather than a silent `false`.
pub(crate) fn parse_retro(entry: &Value, ctx: &str) -> VedicApiResult<bool> {
    match entry.get("isRetro").and_then(Value::as_str) {
        Some("true") => Ok(true),
        Some("false") => Ok(false),
        Some(other) => Err(VedicApiError::Parse {
            message: format!(
                "{}: isRetro must be \"true\"|\"false\", got {:?}",
                ctx, other
            ),
        }),
        None => Err(VedicApiError::Parse {
            message: format!("{}: missing isRetro", ctx),
        }),
    }
}

pub(crate) fn read_sign(entry: &Value, ctx: &str) -> VedicApiResult<ZodiacSign> {
    let n = entry
        .get("current_sign")
        .and_then(Value::as_u64)
        .ok_or_else(|| VedicApiError::Parse {
            message: format!("entry '{}' missing 'current_sign'", ctx),
        })?;
    ZodiacSign::from_number(n as u8).ok_or_else(|| VedicApiError::Parse {
        message: format!(
            "entry '{}' has out-of-range current_sign={} (expected 1..=12)",
            ctx, n
        ),
    })
}

pub(crate) fn read_str(entry: &Value, key: &str) -> VedicApiResult<String> {
    entry
        .get(key)
        .and_then(Value::as_str)
        .map(|s| s.to_string())
        .ok_or_else(|| VedicApiError::Parse {
            message: format!("entry missing string field '{}'", key),
        })
}

pub(crate) fn read_f64(entry: &Value, key: &str) -> Option<f64> {
    entry.get(key).and_then(Value::as_f64)
}

/// Required-field variant of [`read_f64`] for callers that cannot fall back
/// to a default.
pub(crate) fn require_f64(entry: &Value, key: &str, ctx: &str) -> VedicApiResult<f64> {
    read_f64(entry, key).ok_or_else(|| VedicApiError::Parse {
        message: format!("{}: missing or non-numeric '{}'", ctx, key),
    })
}

/// Required-field variant for unsigned integers.
pub(crate) fn require_u64(entry: &Value, key: &str, ctx: &str) -> VedicApiResult<u64> {
    entry
        .get(key)
        .and_then(Value::as_u64)
        .ok_or_else(|| VedicApiError::Parse {
            message: format!("{}: missing or non-integer '{}'", ctx, key),
        })
}

fn classify_house(n: u8) -> HouseType {
    match n {
        1 | 5 | 9 => HouseType::Dharma,
        2 | 6 | 10 => HouseType::Artha,
        3 | 7 | 11 => HouseType::Kama,
        _ => HouseType::Moksha,
    }
}

// ---- tests -------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn native() -> NativeInfo {
        NativeInfo {
            birth_date: "1991-08-13".to_string(),
            birth_time: "13:31:00".to_string(),
            latitude: 12.9716,
            longitude: 77.5946,
            timezone: 5.5,
        }
    }

    fn load_fixture(name: &str) -> Value {
        let path = format!(
            "{}/tests/fixtures/captures/{}",
            env!("CARGO_MANIFEST_DIR"),
            name
        );
        let bytes = std::fs::read(&path)
            .unwrap_or_else(|e| panic!("failed to read fixture {}: {}", path, e));
        serde_json::from_slice(&bytes)
            .unwrap_or_else(|e| panic!("failed to parse fixture {}: {}", path, e))
    }

    #[test]
    fn from_number_round_trips() {
        assert_eq!(ZodiacSign::from_number(1), Some(ZodiacSign::Aries));
        assert_eq!(ZodiacSign::from_number(4), Some(ZodiacSign::Cancer));
        assert_eq!(ZodiacSign::from_number(8), Some(ZodiacSign::Scorpio));
        assert_eq!(ZodiacSign::from_number(12), Some(ZodiacSign::Pisces));
        assert_eq!(ZodiacSign::from_number(0), None);
        assert_eq!(ZodiacSign::from_number(13), None);
    }

    #[test]
    fn test_maps_planets_envelope_fixture() {
        let raw = load_fixture("planets_bangalore_1991-08-13.json");
        let chart = map_planets_envelope_to_birth_chart(&raw, native())
            .expect("mapping the captured /planets fixture must succeed");

        // current_sign is 1-indexed (1=Aries..12=Pisces).
        assert_eq!(chart.ascendant.sign, ZodiacSign::Scorpio); // current_sign=8

        let sun = chart
            .planets
            .iter()
            .find(|p| p.name == "Sun")
            .expect("Sun must be present");
        // fullDegree 116.35° → Cancer; current_sign=4 → Cancer.
        assert_eq!(sun.sign, ZodiacSign::Cancer);
        assert_eq!(sun.house, 9);

        let saturn = chart
            .planets
            .iter()
            .find(|p| p.name == "Saturn")
            .expect("Saturn must be present");
        assert_eq!(saturn.sign, ZodiacSign::Capricorn); // current_sign=10
        assert!(saturn.is_retrograde);

        assert_eq!(chart.planets.len(), 12);
    }

    #[test]
    fn test_maps_navamsa_envelope_fixture() {
        let raw = load_fixture("navamsa_bangalore_1991-08-13.json");
        let chart = map_navamsa_envelope_to_navamsa_chart(&raw, native())
            .expect("mapping the captured /navamsa-chart-info fixture must succeed");

        assert_eq!(chart.d9_lagna, ZodiacSign::Libra); // current_sign=7
        assert_eq!(chart.navamsa_positions.len(), 12);

        let sun = chart
            .navamsa_positions
            .iter()
            .find(|p| p.planet == "Sun")
            .expect("Sun must be present in D9");
        assert_eq!(sun.sign, ZodiacSign::Aquarius); // current_sign=11
    }

    #[test]
    fn test_rejects_malformed_envelope() {
        let raw: Value = serde_json::json!({"statusCode": 200, "output": "not an array"});
        let err = map_planets_envelope_to_birth_chart(&raw, native()).unwrap_err();
        assert!(matches!(err, VedicApiError::Parse { .. }));
    }

    #[test]
    fn parse_retro_rejects_garbage() {
        let entry = serde_json::json!({"isRetro": "yes"});
        let err = parse_retro(&entry, "Sun").unwrap_err();
        assert!(matches!(err, VedicApiError::Parse { .. }));

        let entry = serde_json::json!({});
        let err = parse_retro(&entry, "Sun").unwrap_err();
        assert!(matches!(err, VedicApiError::Parse { .. }));

        assert!(parse_retro(&serde_json::json!({"isRetro": "true"}), "Sun").unwrap());
        assert!(!parse_retro(&serde_json::json!({"isRetro": "false"}), "Sun").unwrap());
    }
}
