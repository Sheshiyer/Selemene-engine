//! Snapshot tests for expected output fixtures
//!
//! Validates the 20 golden output fixtures (5 users × 4 engines) in
//! `tests/fixtures/expected_outputs/`. Each test loads a fixture file,
//! deserializes it, and verifies structural integrity plus engine-specific
//! field constraints.
//!
//! Run with: cargo test --test fixture_snapshot_tests

use serde_json::Value;
use std::path::PathBuf;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Absolute path to the workspace `tests/fixtures/expected_outputs/` directory.
fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("tests/fixtures/expected_outputs")
}

/// Load and parse a fixture JSON file.
fn load_fixture(engine: &str, user: &str) -> Value {
    let path = fixtures_dir().join(engine).join(format!("{}.json", user));
    let content = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("Failed to read fixture {}/{}.json: {}", engine, user, e));
    serde_json::from_str(&content)
        .unwrap_or_else(|e| panic!("Failed to parse fixture {}/{}.json: {}", engine, user, e))
}

/// Assert that a fixture contains all required top-level EngineOutput fields.
fn assert_engine_output_structure(fixture: &Value, expected_engine_id: &str) {
    assert_eq!(
        fixture["engine_id"]
            .as_str()
            .expect("engine_id must be a string"),
        expected_engine_id,
        "engine_id mismatch"
    );
    assert!(
        fixture["result"].is_object(),
        "result must be a JSON object"
    );
    assert!(
        fixture["witness_prompt"].is_string()
            && !fixture["witness_prompt"].as_str().unwrap().is_empty(),
        "witness_prompt must be a non-empty string"
    );
    assert!(
        fixture["consciousness_level"].is_number(),
        "consciousness_level must be a number"
    );

    let meta = &fixture["metadata"];
    assert!(meta.is_object(), "metadata must be an object");
    assert!(
        meta["calculation_time_ms"].is_number(),
        "metadata.calculation_time_ms must be a number"
    );
    assert!(
        meta["backend"].is_string(),
        "metadata.backend must be a string"
    );
    assert!(
        meta["precision_achieved"].is_string(),
        "metadata.precision_achieved must be a string"
    );
    assert!(
        meta["cached"].is_boolean(),
        "metadata.cached must be a boolean"
    );
    assert!(
        meta["timestamp"].is_string(),
        "metadata.timestamp must be a string"
    );
}

const REFERENCE_USERS: &[&str] = &[
    "user_nyc_1990",
    "user_london_1985",
    "user_tokyo_1995",
    "user_sydney_1988",
    "user_mumbai_1992",
];

// ---------------------------------------------------------------------------
// Biorhythm fixtures (5 users)
// ---------------------------------------------------------------------------

fn assert_biorhythm_fixture(fixture: &Value) {
    assert_engine_output_structure(fixture, "biorhythm");

    let result = &fixture["result"];

    let days_alive = result["days_alive"]
        .as_i64()
        .expect("days_alive must be an integer");
    assert!(days_alive > 0, "days_alive must be positive");

    assert!(
        result["target_date"].is_string(),
        "target_date must be a string"
    );

    for cycle in &["physical", "emotional", "intellectual", "intuitive"] {
        let c = &result[cycle];
        assert!(c.is_object(), "{} must be an object", cycle);

        let value = c["value"]
            .as_f64()
            .unwrap_or_else(|| panic!("{}.value must be f64", cycle));
        assert!(
            (-1.0..=1.0).contains(&value),
            "{}.value {} out of [-1, 1]",
            cycle,
            value
        );

        let pct = c["percentage"]
            .as_f64()
            .unwrap_or_else(|| panic!("{}.percentage must be f64", cycle));
        assert!(
            (0.0..=100.0).contains(&pct),
            "{}.percentage {} out of [0, 100]",
            cycle,
            pct
        );

        let phase = c["phase"]
            .as_str()
            .unwrap_or_else(|| panic!("{}.phase must be string", cycle));
        assert!(
            ["Rising", "Falling", "Peak", "Low", "Critical"].contains(&phase),
            "{}.phase '{}' is not a valid phase",
            cycle,
            phase
        );

        assert!(
            c["days_until_peak"].is_number(),
            "{}.days_until_peak must be a number",
            cycle
        );
        assert!(
            c["days_until_critical"].is_number(),
            "{}.days_until_critical must be a number",
            cycle
        );
        assert!(
            c["is_critical"].is_boolean(),
            "{}.is_critical must be a boolean",
            cycle
        );
        assert!(
            c["cycle_day"].is_number(),
            "{}.cycle_day must be a number",
            cycle
        );
    }

    for composite in &["mastery", "passion", "wisdom", "overall_energy"] {
        let v = result[composite]
            .as_f64()
            .unwrap_or_else(|| panic!("{} must be f64", composite));
        assert!(
            (0.0..=100.0).contains(&v),
            "{} = {} out of [0, 100]",
            composite,
            v
        );
    }

    assert!(
        result["critical_days"].is_array(),
        "critical_days must be an array"
    );
}

#[test]
fn test_biorhythm_fixture_user_nyc_1990() {
    let f = load_fixture("biorhythm", "user_nyc_1990");
    assert_biorhythm_fixture(&f);
    let result = &f["result"];
    assert_eq!(result["days_alive"].as_i64().unwrap(), 13135);
    assert_eq!(result["target_date"].as_str().unwrap(), "2026-01-01");
    assert!(
        (result["physical"]["value"].as_f64().unwrap() - 0.5195839500353275).abs() < 1e-9,
        "physical value mismatch"
    );
    assert_eq!(
        result["intellectual"]["phase"].as_str().unwrap(),
        "Critical"
    );
}

#[test]
fn test_biorhythm_fixture_user_london_1985() {
    let f = load_fixture("biorhythm", "user_london_1985");
    assert_biorhythm_fixture(&f);
    let result = &f["result"];
    assert_eq!(result["days_alive"].as_i64().unwrap(), 14805);
    assert_eq!(result["emotional"]["phase"].as_str().unwrap(), "Low");
}

#[test]
fn test_biorhythm_fixture_user_tokyo_1995() {
    let f = load_fixture("biorhythm", "user_tokyo_1995");
    assert_biorhythm_fixture(&f);
    let result = &f["result"];
    assert_eq!(result["days_alive"].as_i64().unwrap(), 10987);
}

#[test]
fn test_biorhythm_fixture_user_sydney_1988() {
    let f = load_fixture("biorhythm", "user_sydney_1988");
    assert_biorhythm_fixture(&f);
    let result = &f["result"];
    assert_eq!(result["days_alive"].as_i64().unwrap(), 13800);
    assert_eq!(result["physical"]["phase"].as_str().unwrap(), "Critical");
    assert_eq!(result["intellectual"]["phase"].as_str().unwrap(), "Rising");
}

#[test]
fn test_biorhythm_fixture_user_mumbai_1992() {
    let f = load_fixture("biorhythm", "user_mumbai_1992");
    assert_biorhythm_fixture(&f);
    let result = &f["result"];
    assert_eq!(result["days_alive"].as_i64().unwrap(), 12228);
    assert_eq!(result["emotional"]["phase"].as_str().unwrap(), "Low");
}

/// Validate all 5 biorhythm fixtures in one sweep.
#[test]
fn test_all_biorhythm_fixtures_valid() {
    for user in REFERENCE_USERS {
        let f = load_fixture("biorhythm", user);
        assert_biorhythm_fixture(&f);
    }
}

// ---------------------------------------------------------------------------
// Vimshottari fixtures (5 users)
// ---------------------------------------------------------------------------

const VALID_PLANETS: &[&str] = &[
    "Sun", "Moon", "Mars", "Rahu", "Jupiter", "Saturn", "Mercury", "Ketu", "Venus",
];

const VALID_NAKSHATRAS: &[&str] = &[
    "Ashwini",
    "Bharani",
    "Krittika",
    "Rohini",
    "Mrigashira",
    "Ardra",
    "Punarvasu",
    "Pushya",
    "Ashlesha",
    "Magha",
    "Purva Phalguni",
    "Uttara Phalguni",
    "Hasta",
    "Chitra",
    "Swati",
    "Vishakha",
    "Anuradha",
    "Jyeshtha",
    "Mula",
    "Purva Ashadha",
    "Uttara Ashadha",
    "Shravana",
    "Dhanishta",
    "Shatabhisha",
    "Purva Bhadrapada",
    "Uttara Bhadrapada",
    "Revati",
];

fn assert_vimshottari_fixture(fixture: &Value) {
    assert_engine_output_structure(fixture, "vimshottari");

    let result = &fixture["result"];

    // birth_nakshatra
    let nak = &result["birth_nakshatra"];
    assert!(nak.is_object(), "birth_nakshatra must be an object");
    let nak_name = nak["name"]
        .as_str()
        .expect("birth_nakshatra.name must be string");
    assert!(
        VALID_NAKSHATRAS.contains(&nak_name),
        "birth_nakshatra.name '{}' is not a valid nakshatra",
        nak_name
    );
    let nak_num = nak["number"]
        .as_u64()
        .expect("birth_nakshatra.number must be u64");
    assert!(
        (1..=27).contains(&nak_num),
        "birth_nakshatra.number {} out of range [1, 27]",
        nak_num
    );
    let moon_lon = nak["moon_longitude"]
        .as_f64()
        .expect("birth_nakshatra.moon_longitude must be f64");
    assert!(
        (0.0..360.0).contains(&moon_lon),
        "moon_longitude {} out of [0, 360)",
        moon_lon
    );

    // timeline
    let timeline = &result["timeline"];
    assert!(timeline.is_object(), "timeline must be an object");
    let total_years = timeline["total_years"]
        .as_u64()
        .expect("timeline.total_years must be u64");
    assert_eq!(total_years, 120, "timeline.total_years must be 120");

    let mahadashas = timeline["mahadashas"]
        .as_array()
        .expect("timeline.mahadashas must be an array");
    assert_eq!(mahadashas.len(), 9, "must have exactly 9 mahadashas");

    for maha in mahadashas {
        let planet = maha["planet"]
            .as_str()
            .expect("mahadasha.planet must be string");
        assert!(
            VALID_PLANETS.contains(&planet),
            "mahadasha.planet '{}' is not valid",
            planet
        );
        assert!(
            maha["start_date"].is_string(),
            "mahadasha.start_date must be string"
        );
        assert!(
            maha["end_date"].is_string(),
            "mahadasha.end_date must be string"
        );
        let dur = maha["duration_years"]
            .as_f64()
            .expect("mahadasha.duration_years must be number");
        assert!(dur > 0.0, "mahadasha.duration_years must be positive");
    }

    // current_period
    let cp = &result["current_period"];
    assert!(cp.is_object(), "current_period must be an object");
    let cp_planet = cp["mahadasha"]["planet"]
        .as_str()
        .expect("current_period.mahadasha.planet must be string");
    assert!(
        VALID_PLANETS.contains(&cp_planet),
        "current_period mahadasha planet '{}' is not valid",
        cp_planet
    );
}

#[test]
fn test_vimshottari_fixture_user_nyc_1990() {
    let f = load_fixture("vimshottari", "user_nyc_1990");
    assert_vimshottari_fixture(&f);
    let nak_name = f["result"]["birth_nakshatra"]["name"].as_str().unwrap();
    let moon_lon = f["result"]["birth_nakshatra"]["moon_longitude"]
        .as_f64()
        .unwrap();
    assert!(
        (moon_lon - 45.0).abs() < 1e-9,
        "moon_longitude should be 45.0"
    );
    assert_eq!(nak_name, "Rohini");
}

#[test]
fn test_vimshottari_fixture_user_london_1985() {
    let f = load_fixture("vimshottari", "user_london_1985");
    assert_vimshottari_fixture(&f);
    assert_eq!(
        f["result"]["birth_nakshatra"]["name"].as_str().unwrap(),
        "Vishakha"
    );
}

#[test]
fn test_vimshottari_fixture_user_tokyo_1995() {
    let f = load_fixture("vimshottari", "user_tokyo_1995");
    assert_vimshottari_fixture(&f);
    assert_eq!(
        f["result"]["birth_nakshatra"]["name"].as_str().unwrap(),
        "Pushya"
    );
}

#[test]
fn test_vimshottari_fixture_user_sydney_1988() {
    let f = load_fixture("vimshottari", "user_sydney_1988");
    assert_vimshottari_fixture(&f);
    assert_eq!(
        f["result"]["birth_nakshatra"]["name"].as_str().unwrap(),
        "Shravana"
    );
}

#[test]
fn test_vimshottari_fixture_user_mumbai_1992() {
    let f = load_fixture("vimshottari", "user_mumbai_1992");
    assert_vimshottari_fixture(&f);
    assert_eq!(
        f["result"]["birth_nakshatra"]["name"].as_str().unwrap(),
        "Purva Phalguni"
    );
}

/// Validate all 5 vimshottari fixtures in one sweep.
#[test]
fn test_all_vimshottari_fixtures_valid() {
    for user in REFERENCE_USERS {
        let f = load_fixture("vimshottari", user);
        assert_vimshottari_fixture(&f);
    }
}

// ---------------------------------------------------------------------------
// Vedic-Clock fixtures (5 users)
// ---------------------------------------------------------------------------

const VALID_ORGANS: &[&str] = &[
    "Lung",
    "LargeIntestine",
    "Stomach",
    "Spleen",
    "Heart",
    "SmallIntestine",
    "Bladder",
    "Kidney",
    "Pericardium",
    "TripleWarmer",
    "Gallbladder",
    "Liver",
];

const VALID_DOSHAS: &[&str] = &["Vata", "Pitta", "Kapha"];

const VALID_ELEMENTS: &[&str] = &["Wood", "Fire", "Earth", "Metal", "Water"];

fn assert_vedic_clock_fixture(fixture: &Value) {
    assert_engine_output_structure(fixture, "vedic-clock");

    let result = &fixture["result"];

    // current_organ
    let organ = &result["current_organ"];
    assert!(organ.is_object(), "current_organ must be an object");
    let organ_name = organ["organ"]
        .as_str()
        .expect("current_organ.organ must be string");
    assert!(
        VALID_ORGANS.contains(&organ_name),
        "current_organ.organ '{}' is not valid",
        organ_name
    );
    let element = organ["element"]
        .as_str()
        .expect("current_organ.element must be string");
    assert!(
        VALID_ELEMENTS.contains(&element),
        "current_organ.element '{}' is not valid",
        element
    );
    assert!(
        organ["time_window"].is_string(),
        "current_organ.time_window must be string"
    );
    assert!(
        organ["peak_energy"].is_string(),
        "current_organ.peak_energy must be string"
    );
    assert!(
        organ["recommended_activities"].is_array(),
        "current_organ.recommended_activities must be array"
    );

    // current_dosha
    let dosha = &result["current_dosha"];
    assert!(dosha.is_object(), "current_dosha must be an object");
    let dosha_name = dosha["dosha"]
        .as_str()
        .expect("current_dosha.dosha must be string");
    assert!(
        VALID_DOSHAS.contains(&dosha_name),
        "current_dosha.dosha '{}' is not valid",
        dosha_name
    );
    assert!(
        dosha["qualities"].is_array(),
        "current_dosha.qualities must be array"
    );

    // recommendation
    let rec = &result["recommendation"];
    assert!(rec.is_object(), "recommendation must be an object");

    // timezone
    let tz = &result["timezone"];
    assert!(tz.is_object(), "timezone must be an object");
    assert!(
        tz["offset_minutes"].is_number(),
        "timezone.offset_minutes must be a number"
    );
    let local_hour = tz["local_hour"]
        .as_u64()
        .expect("timezone.local_hour must be u64");
    assert!(
        local_hour < 24,
        "timezone.local_hour must be 0-23, got {}",
        local_hour
    );
}

#[test]
fn test_vedic_clock_fixture_user_nyc_1990() {
    let f = load_fixture("vedic-clock", "user_nyc_1990");
    assert_vedic_clock_fixture(&f);
    assert_eq!(
        f["result"]["current_organ"]["organ"].as_str().unwrap(),
        "Spleen"
    );
    assert_eq!(
        f["result"]["current_dosha"]["dosha"].as_str().unwrap(),
        "Kapha"
    );
    assert_eq!(f["result"]["timezone"]["local_hour"].as_u64().unwrap(), 9);
}

#[test]
fn test_vedic_clock_fixture_user_london_1985() {
    let f = load_fixture("vedic-clock", "user_london_1985");
    assert_vedic_clock_fixture(&f);
    assert_eq!(
        f["result"]["current_organ"]["organ"].as_str().unwrap(),
        "SmallIntestine"
    );
    assert_eq!(
        f["result"]["current_dosha"]["dosha"].as_str().unwrap(),
        "Vata"
    );
    assert_eq!(f["result"]["timezone"]["local_hour"].as_u64().unwrap(), 14);
}

#[test]
fn test_vedic_clock_fixture_user_tokyo_1995() {
    let f = load_fixture("vedic-clock", "user_tokyo_1995");
    assert_vedic_clock_fixture(&f);
    assert_eq!(
        f["result"]["current_organ"]["organ"].as_str().unwrap(),
        "Gallbladder"
    );
    assert_eq!(
        f["result"]["current_dosha"]["dosha"].as_str().unwrap(),
        "Pitta"
    );
    assert_eq!(f["result"]["timezone"]["local_hour"].as_u64().unwrap(), 23);
}

#[test]
fn test_vedic_clock_fixture_user_sydney_1988() {
    let f = load_fixture("vedic-clock", "user_sydney_1988");
    assert_vedic_clock_fixture(&f);
    assert_eq!(
        f["result"]["current_organ"]["organ"].as_str().unwrap(),
        "Liver"
    );
    assert_eq!(
        f["result"]["current_dosha"]["dosha"].as_str().unwrap(),
        "Pitta"
    );
    assert_eq!(f["result"]["timezone"]["local_hour"].as_u64().unwrap(), 1);
}

#[test]
fn test_vedic_clock_fixture_user_mumbai_1992() {
    let f = load_fixture("vedic-clock", "user_mumbai_1992");
    assert_vedic_clock_fixture(&f);
    assert_eq!(
        f["result"]["current_organ"]["organ"].as_str().unwrap(),
        "Pericardium"
    );
    assert_eq!(
        f["result"]["current_dosha"]["dosha"].as_str().unwrap(),
        "Kapha"
    );
    assert_eq!(f["result"]["timezone"]["local_hour"].as_u64().unwrap(), 19);
}

/// Validate all 5 vedic-clock fixtures in one sweep.
#[test]
fn test_all_vedic_clock_fixtures_valid() {
    for user in REFERENCE_USERS {
        let f = load_fixture("vedic-clock", user);
        assert_vedic_clock_fixture(&f);
    }
}

// ---------------------------------------------------------------------------
// Biofield fixtures (5 users)
// ---------------------------------------------------------------------------

const VALID_CHAKRAS: &[&str] = &[
    "Root",
    "Sacral",
    "SolarPlexus",
    "Heart",
    "Throat",
    "ThirdEye",
    "Crown",
];

fn assert_biofield_fixture(fixture: &Value) {
    assert_engine_output_structure(fixture, "biofield");

    let result = &fixture["result"];

    // metrics
    let metrics = &result["metrics"];
    assert!(metrics.is_object(), "metrics must be an object");

    let fd = metrics["fractal_dimension"]
        .as_f64()
        .expect("metrics.fractal_dimension must be f64");
    assert!(
        (1.0..=2.0).contains(&fd),
        "fractal_dimension {} out of [1.0, 2.0]",
        fd
    );

    for field in &["entropy", "coherence", "symmetry", "vitality_index"] {
        let v = metrics[field]
            .as_f64()
            .unwrap_or_else(|| panic!("metrics.{} must be f64", field));
        assert!(
            (0.0..=1.0).contains(&v),
            "metrics.{} = {} out of [0.0, 1.0]",
            field,
            v
        );
    }

    // chakra_readings
    let chakras = result["chakra_readings"]
        .as_array()
        .expect("chakra_readings must be an array");
    assert_eq!(chakras.len(), 7, "must have exactly 7 chakra readings");

    let mut seen_chakras = std::collections::HashSet::new();
    for reading in chakras {
        let chakra = reading["chakra"].as_str().expect("chakra must be string");
        assert!(
            VALID_CHAKRAS.contains(&chakra),
            "chakra '{}' is not valid",
            chakra
        );
        assert!(
            seen_chakras.insert(chakra),
            "duplicate chakra reading for '{}'",
            chakra
        );

        let activity = reading["activity_level"]
            .as_f64()
            .expect("activity_level must be f64");
        assert!(
            (0.0..=1.0).contains(&activity),
            "activity_level {} out of [0, 1]",
            activity
        );

        let balance = reading["balance"].as_f64().expect("balance must be f64");
        assert!(
            (-1.0..=1.0).contains(&balance),
            "balance {} out of [-1, 1]",
            balance
        );

        assert!(
            reading["color_intensity"].is_string(),
            "color_intensity must be a string"
        );
    }

    assert!(
        result["interpretation"].is_string(),
        "interpretation must be a string"
    );
    assert!(
        result["areas_of_attention"].is_array(),
        "areas_of_attention must be an array"
    );
    assert!(
        result["is_mock_data"]
            .as_bool()
            .expect("is_mock_data must be boolean"),
        "is_mock_data must be true for these fixtures"
    );
}

#[test]
fn test_biofield_fixture_user_nyc_1990() {
    let f = load_fixture("biofield", "user_nyc_1990");
    assert_biofield_fixture(&f);
    let metrics = &f["result"]["metrics"];
    let fd = metrics["fractal_dimension"].as_f64().unwrap();
    let vi = metrics["vitality_index"].as_f64().unwrap();
    assert!((1.0..=2.0).contains(&fd));
    assert!((0.0..=1.0).contains(&vi));
}

#[test]
fn test_biofield_fixture_user_london_1985() {
    let f = load_fixture("biofield", "user_london_1985");
    assert_biofield_fixture(&f);
}

#[test]
fn test_biofield_fixture_user_tokyo_1995() {
    let f = load_fixture("biofield", "user_tokyo_1995");
    assert_biofield_fixture(&f);
}

#[test]
fn test_biofield_fixture_user_sydney_1988() {
    let f = load_fixture("biofield", "user_sydney_1988");
    assert_biofield_fixture(&f);
}

#[test]
fn test_biofield_fixture_user_mumbai_1992() {
    let f = load_fixture("biofield", "user_mumbai_1992");
    assert_biofield_fixture(&f);
}

/// Validate all 5 biofield fixtures in one sweep.
#[test]
fn test_all_biofield_fixtures_valid() {
    for user in REFERENCE_USERS {
        let f = load_fixture("biofield", user);
        assert_biofield_fixture(&f);
    }
}

// ---------------------------------------------------------------------------
// Cross-engine meta-test: all 20 fixture files exist and are valid JSON
// ---------------------------------------------------------------------------

#[test]
fn test_all_20_fixture_files_exist_and_parse() {
    let engines = &["biorhythm", "vimshottari", "vedic-clock", "biofield"];
    let mut count = 0;
    for engine in engines {
        for user in REFERENCE_USERS {
            let fixture = load_fixture(engine, user); // panics if missing or invalid
            assert!(
                fixture["engine_id"].is_string(),
                "{}/{}: missing engine_id",
                engine,
                user
            );
            count += 1;
        }
    }
    assert_eq!(count, 20, "Expected exactly 20 fixture files");
}
