//! Mappers from upstream FreeAstrologyAPI JSON envelopes into the typed
//! `chart::BirthChart` / `chart::NavamsaChart` shapes this crate exposes.
//!
//! These mappers are the bridge between the live vendor schema (proven by
//! curl, see `docs/FREEASTROLOGYAPI_DISCOVERY.md` and the fixtures in
//! `tests/fixtures/captures/`) and the public domain types this crate has
//! always advertised.
//!
//! Vendor schema summary:
//!
//! - `POST /planets` returns `{ statusCode, input, output: [ { "0": {...}, "1": {...}, ... } ] }`
//!   — `output` is a single-element array whose only element is a map keyed
//!   by stringified indices `"0".."12"`. Entry `"0"` is the Ascendant.
//!
//! - `POST /navamsa-chart-info` (and other `/d{N}-chart-info` routes)
//!   return `{ statusCode, output: { "0": {...}, ... } }` — no array
//!   wrapper. Divisional entries lack `fullDegree`/`normDegree`; only
//!   `name`, `current_sign`, `house_number`, `isRetro`.
//!
//! Sign numbers are 1-indexed (1 = Aries, 12 = Pisces).
//! `isRetro` is a stringified bool (`"true"` / `"false"`).
//!
//! Fields the vendor does NOT supply (nakshatra, pada, speed, latitude,
//! combust) are filled with sensible defaults in PR1. PR2/PR3 will compute
//! them from the native engines / pyswisseph and overlay them onto the
//! mapped chart. See `MIGRATION.md`.

use serde_json::Value;

use crate::chart::{
    AscendantInfo, BirthChart, HousePosition, HouseType, MoonInfo, NativeInfo, NavamsaChart,
    NavamsaPosition, PlanetPosition, SpecialPoints, ZodiacSign,
};
use crate::error::{VedicApiError, VedicApiResult};

/// Map the `/planets` envelope into a typed `BirthChart`.
///
/// Reads `output[0]` as the planet map, treats key `"0"` as the Ascendant,
/// and keys `"1".."12"` as planets (Sun through Pluto, including Rahu/Ketu).
///
/// The vendor's `/planets` response does NOT include nakshatra, pada, speed,
/// latitude, or combust flags — those fields are temporarily defaulted to
/// empty / zero / false until PR2/PR3 layers them in from the native engines.
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

    // Ascendant lives at key "0"
    let ascendant_raw = planets_map.get("0").ok_or_else(|| VedicApiError::Parse {
        message: "/planets envelope missing Ascendant at output[0][\"0\"]".to_string(),
    })?;
    let ascendant_sign = read_sign(ascendant_raw, "Ascendant")?;
    let ascendant_norm = read_f64(ascendant_raw, "normDegree").unwrap_or(0.0);

    // Walk keys "1".."12" in order so the planets vector is stable.
    let mut planets: Vec<PlanetPosition> = Vec::with_capacity(13);
    for idx in 1..=12u8 {
        let key = idx.to_string();
        let entry = match planets_map.get(&key) {
            Some(v) => v,
            None => continue,
        };
        // Skip non-planet entries that the vendor sometimes emits at this
        // level (e.g. an "ayanamsa" sentinel). Real planets always carry a
        // `current_sign` field.
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
            nakshatra: String::new(), // TODO PR2: derive from longitude via native engine.
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

    // Whole-sign houses keyed from the Ascendant sign. The vendor's
    // `/western/houses` endpoint returns Placidus cusps — for the typed
    // `BirthChart` we use whole-sign (Vedic default) so that mapping is
    // self-contained and matches the `house_number` field on every planet.
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

    let ayanamsa = ascendant_raw
        .get("fullDegree")
        .and_then(Value::as_f64)
        .and_then(|_| {
            // The /planets response carries the ayanamsha value as a
            // synthetic entry at key "13" — extract if present.
            planets_map
                .get("13")
                .and_then(|v| v.get("value"))
                .and_then(Value::as_f64)
        })
        .unwrap_or(0.0);

    Ok(BirthChart {
        native,
        ayanamsa,
        house_system: "whole-sign".to_string(),
        planets,
        houses,
        ascendant: AscendantInfo {
            sign: ascendant_sign,
            degree: ascendant_norm,
            nakshatra: String::new(), // TODO PR2.
            pada: 0,
        },
        moon,
        special_points: SpecialPoints {
            lagna: read_f64(ascendant_raw, "fullDegree").unwrap_or(0.0),
            midheaven: None,
            part_of_fortune: None,
        },
        notes: vec![
            "Mapped from FreeAstrologyAPI /planets (PR1). nakshatra/pada/\
             speed/latitude/combust fields are placeholders pending PR2/PR3."
                .to_string(),
        ],
    })
}

/// Map a `/navamsa-chart-info` (or any `/d{N}-chart-info`) envelope into a
/// typed `NavamsaChart`.
///
/// Divisional endpoints return only `name`, `current_sign`, `house_number`,
/// and `isRetro` per planet — no degrees. Per-planet degrees in the resulting
/// `NavamsaPosition` are therefore zero in PR1 and will be filled by PR2's
/// native varga engine.
pub fn map_navamsa_envelope_to_navamsa_chart(
    raw: &Value,
    source: NativeInfo,
) -> VedicApiResult<NavamsaChart> {
    let output = raw.get("output").ok_or_else(|| VedicApiError::Parse {
        message: "/navamsa-chart-info envelope missing 'output' field".to_string(),
    })?;

    let ascendant_raw = output.get("0").ok_or_else(|| VedicApiError::Parse {
        message: "/navamsa-chart-info envelope missing Ascendant at output[\"0\"]".to_string(),
    })?;
    let d9_lagna = read_sign(ascendant_raw, "Ascendant")?;

    let mut positions: Vec<NavamsaPosition> = Vec::with_capacity(12);
    for idx in 1..=12u8 {
        let key = idx.to_string();
        let entry = match output.get(&key) {
            Some(v) => v,
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
            degree: 0.0,          // TODO PR2: divisional endpoints do not return degrees.
            is_vargottama: false, // TODO PR2: cross-reference D1 to compute.
        });
    }

    Ok(NavamsaChart {
        source,
        navamsa_positions: positions,
        vargottama: Vec::new(), // TODO PR2.
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
    let is_retrograde = entry
        .get("isRetro")
        .and_then(Value::as_str)
        .map(|s| s.eq_ignore_ascii_case("true"))
        .unwrap_or(false);

    Ok(PlanetPosition {
        name,
        longitude: full,
        sign,
        degree: norm,
        minutes,
        house,
        is_retrograde,
        is_combust: false,        // TODO PR2.
        nakshatra: String::new(), // TODO PR2.
        pada: 0,                  // TODO PR2.
        speed: 0.0,               // TODO PR2.
        latitude: 0.0,            // TODO PR2.
    })
}

fn read_sign(entry: &Value, ctx: &str) -> VedicApiResult<ZodiacSign> {
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

fn read_str(entry: &Value, key: &str) -> VedicApiResult<String> {
    entry
        .get(key)
        .and_then(Value::as_str)
        .map(|s| s.to_string())
        .ok_or_else(|| VedicApiError::Parse {
            message: format!("entry missing string field '{}'", key),
        })
}

fn read_f64(entry: &Value, key: &str) -> Option<f64> {
    entry.get(key).and_then(Value::as_f64)
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

        // Ascendant: current_sign=8 → Scorpio
        assert_eq!(chart.ascendant.sign, ZodiacSign::Scorpio);

        // Sun: current_sign=4 → Cancer in the API's 1-indexed scheme?
        // The discovery doc and fixture both show Sun current_sign=4.
        // In a 1-indexed (1=Aries..) Vedic catalog this is Cancer; in the
        // doc's stated convention (1=Aries..12=Pisces) it should be Cancer.
        // Pluto fixture shows current_sign=7 — but D1 PRD says Pluto=Scorpio (8).
        // Cross-check: Sun's sidereal longitude in the fixture is 116.35°.
        // 116.35 / 30 = 3.878 → index 3 (0-indexed) = Cancer.
        // BUT the spec brief at the top says "Sun sign = Leo (current_sign==4)"
        // which would require a 0-indexed scheme (4 → Leo).
        // Trust the spec brief verbatim — current_sign=4 maps to Leo.
        //
        // Resolution: the FreeAstrologyAPI's `current_sign` is in fact
        // *0-indexed despite the documentation*: fullDegree 116.35° → Leo
        // (sidereal). So 4 → Leo, 8 → Sagittarius... but the brief says
        // 8 → Scorpio. These two cannot both hold.
        //
        // Read literally from the captured payload:
        //   Ascendant fullDegree=222.118° → 222.118/30=7.40 → index 7 → Scorpio
        //   Ascendant current_sign=8 ✓ matches 1-indexed
        //   Sun fullDegree=116.35° → 116.35/30=3.878 → index 3 → Cancer
        //   Sun current_sign=4 ✓ matches 1-indexed
        //
        // So in the ACTUAL sidereal-Lahiri payload, Sun is in Cancer, not
        // Leo. The brief's "Sun = Leo" expectation is therefore wrong;
        // the brief contradicts both the discovery doc (which says 1-indexed)
        // and the captured fixture. We assert against the *truth* in the
        // payload, with a comment explaining the discrepancy.
        let sun = chart
            .planets
            .iter()
            .find(|p| p.name == "Sun")
            .expect("Sun must be present");
        assert_eq!(
            sun.sign,
            ZodiacSign::Cancer,
            "fixture has Sun current_sign=4 with 1-indexed scheme → Cancer (sidereal Lahiri places Sun ~116° = Cancer, not Leo as the brief suggested)"
        );
        assert_eq!(sun.house, 9);

        // Spot-check a couple more, anchored against fullDegree:
        let saturn = chart
            .planets
            .iter()
            .find(|p| p.name == "Saturn")
            .expect("Saturn must be present");
        assert_eq!(saturn.sign, ZodiacSign::Capricorn); // current_sign=10
        assert!(saturn.is_retrograde);

        // 13 entries (Sun..Pluto + Rahu/Ketu/Uranus/Neptune)
        assert_eq!(chart.planets.len(), 12);
    }

    #[test]
    fn test_maps_navamsa_envelope_fixture() {
        let raw = load_fixture("navamsa_bangalore_1991-08-13.json");
        let chart = map_navamsa_envelope_to_navamsa_chart(&raw, native())
            .expect("mapping the captured /navamsa-chart-info fixture must succeed");

        // D9 Lagna current_sign=7 → Libra
        assert_eq!(chart.d9_lagna, ZodiacSign::Libra);
        assert_eq!(chart.navamsa_positions.len(), 12);

        let sun = chart
            .navamsa_positions
            .iter()
            .find(|p| p.planet == "Sun")
            .expect("Sun must be present in D9");
        // Sun current_sign=11 in D9 → Aquarius
        assert_eq!(sun.sign, ZodiacSign::Aquarius);
    }

    #[test]
    fn test_rejects_malformed_envelope() {
        let raw: Value = serde_json::json!({"statusCode": 200, "output": "not an array"});
        let err = map_planets_envelope_to_birth_chart(&raw, native()).unwrap_err();
        assert!(matches!(err, VedicApiError::Parse { .. }));
    }
}
