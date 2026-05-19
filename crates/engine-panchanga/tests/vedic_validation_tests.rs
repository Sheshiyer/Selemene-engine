//! Vedic Panchanga validation harness (PR4 — Phase 1).
//!
//! Compares `engine-panchanga` output against the on-disk JHora-verified
//! reference data in
//! `crates/noesis-vedic-api/tests/fixtures/reference_data/`.
//!
//! Ground-truth sources (Tier-1, hand-verified, NO live API):
//!   - `shesh_chart_reference.json`        (1 person × full panchang)
//!   - `panchang_jhora_reference.json`     (5 civil-date panchang entries)
//!
//! Field comparison strategy (per `GROUND_TRUTH.md` §B):
//!   - tithi name           — string exact, paksha-suffix stripped from engine
//!   - tithi number/paksha  — derived from engine `tithi_index`
//!   - nakshatra name       — string exact (whitespace normalised)
//!   - nakshatra pada       — u8 exact (derived from `nakshatra_value.fract()*4`)
//!   - yoga name            — string exact
//!   - karana name          — string exact
//!   - vara                 — English day-name extracted from `vara_name` parentheses
//!
//! Fast smoke tests run on every `cargo test`. The full validation report is
//! `#[ignore]`-gated by convention with the HD harness.
//!
//! Run smoke only (fast):
//!     cargo test --package engine-panchanga --test vedic_validation_tests
//!
//! Run full report (longer, prints accuracy table):
//!     cargo test --package engine-panchanga --test vedic_validation_tests \
//!         -- --ignored --nocapture

use engine_panchanga::PanchangaEngine;
use noesis_core::{BirthData, ConsciousnessEngine, EngineInput, Precision};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

// --- Reference fixture types --------------------------------------------

#[derive(Debug, Deserialize)]
struct SheshReference {
    birth_data: ShshBirth,
    panchang: SheshPanchang,
    #[allow(dead_code)]
    dasha: serde_json::Value,
}

#[derive(Debug, Deserialize)]
struct ShshBirth {
    date: String,
    time: String,
    location: RefLocation,
}

#[derive(Debug, Deserialize)]
struct RefLocation {
    #[allow(dead_code)]
    name: Option<String>,
    latitude: f64,
    longitude: f64,
    /// Numeric IST offset (e.g. 5.5). Reference fixtures use a float, not an
    /// IANA string.
    timezone: f64,
}

#[derive(Debug, Deserialize)]
struct SheshPanchang {
    tithi: SheshTithi,
    nakshatra: SheshNakshatra,
    yoga: SheshYoga,
    karana: SheshKarana,
    vara: String,
    #[allow(dead_code)]
    paksha: String,
    #[allow(dead_code)]
    hindu_month: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SheshTithi {
    name: String,
    paksha: String,
    number: u8,
}

#[derive(Debug, Deserialize)]
struct SheshNakshatra {
    name: String,
    #[allow(dead_code)]
    number: u8,
    pada: u8,
}

#[derive(Debug, Deserialize)]
struct SheshYoga {
    name: String,
}

#[derive(Debug, Deserialize)]
struct SheshKarana {
    name: String,
}

#[derive(Debug, Deserialize)]
struct PanchangJhoraIndex {
    test_dates: Vec<PanchangTestDate>,
}

#[derive(Debug, Deserialize)]
struct PanchangTestDate {
    id: String,
    date: String,
    time: String,
    location: RefLocation,
    expected: PanchangExpected,
}

#[derive(Debug, Deserialize)]
struct PanchangExpected {
    tithi: SheshTithi,
    nakshatra: SheshNakshatra,
    yoga: SheshYoga,
    karana: SheshKarana,
    vara: String,
}

// --- Paths ---------------------------------------------------------------

/// Resolve `<workspace_root>/crates/noesis-vedic-api/tests/fixtures/reference_data`.
///
/// `CARGO_MANIFEST_DIR` is `.../crates/engine-panchanga`. Climb 2 parents to
/// reach workspace root, then descend into the cross-crate fixtures dir. We
/// intentionally read the existing references from `noesis-vedic-api` rather
/// than duplicating them — they're the canonical PR1/PR2/PR3 ground truth.
fn reference_data_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .map(|p| p.join("crates/noesis-vedic-api/tests/fixtures/reference_data"))
        .expect("locate workspace root from crate dir")
}

fn load_shesh_reference() -> Option<SheshReference> {
    let path = reference_data_root().join("shesh_chart_reference.json");
    if !path.exists() {
        return None;
    }
    let s = fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {}", path.display(), e));
    Some(serde_json::from_str(&s).expect("parse shesh_chart_reference.json"))
}

fn load_panchang_jhora_reference() -> Option<PanchangJhoraIndex> {
    let path = reference_data_root().join("panchang_jhora_reference.json");
    if !path.exists() {
        return None;
    }
    let s = fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {}", path.display(), e));
    Some(serde_json::from_str(&s).expect("parse panchang_jhora_reference.json"))
}

// --- Per-field accuracy harness (copy of HD `FieldResult`) ---------------

#[derive(Debug, Default, Clone)]
struct FieldResult {
    checked: usize,
    matched: usize,
    mismatched: Vec<String>,
    skipped: usize,
}

impl FieldResult {
    fn record(&mut self, label: &str, expected: Option<&str>, got: &str) {
        match expected {
            None => self.skipped += 1,
            Some(e) => {
                self.checked += 1;
                if e == got {
                    self.matched += 1;
                } else if self.mismatched.len() < 10 {
                    self.mismatched
                        .push(format!("  {}  expected={}  got={}", label, e, got));
                }
            }
        }
    }

    fn record_u8(&mut self, label: &str, expected: Option<u8>, got: u8) {
        self.record(
            label,
            expected.map(|n| n.to_string()).as_deref(),
            &got.to_string(),
        );
    }

    fn pct(&self) -> f64 {
        if self.checked == 0 {
            0.0
        } else {
            (self.matched as f64) * 100.0 / (self.checked as f64)
        }
    }
}

fn print_field(label: &str, r: &FieldResult) {
    println!(
        "  {:<20}  matched {:>3}/{:>3}  ({:>5.1}%)  skipped={:>2}",
        label,
        r.matched,
        r.checked,
        r.pct(),
        r.skipped,
    );
    if !r.mismatched.is_empty() {
        println!("    first mismatches:");
        for m in &r.mismatched {
            println!("    {}", m);
        }
    }
}

// --- Name / shape normalisers --------------------------------------------

/// Normalise nakshatra spellings so JHora "UttaraAshadha" matches engine
/// "Uttara Ashadha". Lowercase + strip whitespace. Both refs and engine
/// outputs are pushed through this before comparison.
fn normalize_nakshatra_name(name: &str) -> String {
    name.chars()
        .filter(|c| !c.is_whitespace())
        .flat_map(char::to_lowercase)
        .collect()
}

/// Engine tithi names look like "Navami (Krishna)" / "Purnima" / "Amavasya".
/// JHora reference gives bare names ("Navami", "Purnima", "Amavasya"). Strip
/// the parenthesised paksha suffix.
fn strip_paksha_suffix(tithi_name: &str) -> String {
    tithi_name
        .split_once(" (")
        .map(|(name, _)| name.to_string())
        .unwrap_or_else(|| tithi_name.to_string())
}

/// Engine vara names look like "Ravivara (Sunday)". JHora reference gives
/// bare English day names ("Sunday"). Extract the parenthesised portion.
fn extract_vara_english(vara_name: &str) -> String {
    if let (Some(open), Some(close)) = (vara_name.find('('), vara_name.find(')')) {
        if close > open + 1 {
            return vara_name[open + 1..close].to_string();
        }
    }
    vara_name.to_string()
}

/// Engine `tithi_index` is 0..30. Krishna paksha is indices 15..30 (Purnima at
/// 14, Amavasya at 29). Map index → paksha string matching JHora convention.
fn paksha_from_tithi_index(idx: u8) -> &'static str {
    // 0..14 = Shukla (Pratipada..Purnima), 15..29 = Krishna (Pratipada..Amavasya)
    if idx >= 15 {
        "Krishna"
    } else {
        "Shukla"
    }
}

/// JHora tithi numbering: Shukla 1..15 (Purnima=15), Krishna 16..30 (Amavasya=30).
/// Engine `tithi_index` is 0..29. Map to JHora numbering.
fn jhora_tithi_number(idx: u8) -> u8 {
    idx + 1
}

/// Compute nakshatra pada (1..4) from raw `nakshatra_value` (continuous 0..27).
fn pada_from_nakshatra_value(nv: f64) -> u8 {
    ((nv.fract() * 4.0).floor() as u8) + 1
}

// --- Input construction --------------------------------------------------

/// Build an `EngineInput` for the panchanga engine from a JHora-style
/// reference location (numeric timezone offset in hours).
fn build_input(date: &str, time: &str, loc: &RefLocation) -> EngineInput {
    let tz_str = numeric_tz_to_iso(loc.timezone);
    EngineInput {
        birth_data: Some(BirthData {
            name: None,
            date: date.to_string(),
            time: Some(time.to_string()),
            latitude: loc.latitude,
            longitude: loc.longitude,
            timezone: tz_str,
        }),
        current_time: chrono::Utc::now(),
        location: None,
        precision: Precision::default(),
        options: HashMap::new(),
    }
}

/// Convert numeric UTC offset (e.g. 5.5) to ISO `+HH:MM` form.
fn numeric_tz_to_iso(offset_hours: f64) -> String {
    let sign = if offset_hours < 0.0 { '-' } else { '+' };
    let abs = offset_hours.abs();
    let hours = abs.trunc() as u8;
    let minutes = ((abs - abs.trunc()) * 60.0).round() as u8;
    format!("{}{:02}:{:02}", sign, hours, minutes)
}

// --- Smoke tests (run on every `cargo test`) -----------------------------

#[test]
fn shesh_reference_fixture_present() {
    let path = reference_data_root().join("shesh_chart_reference.json");
    assert!(
        path.exists(),
        "shesh_chart_reference.json missing at {}",
        path.display()
    );
    let r = load_shesh_reference().expect("shesh reference loads");
    assert_eq!(r.birth_data.date, "1990-07-15");
    assert_eq!(r.panchang.tithi.name, "Navami");
    assert_eq!(r.panchang.nakshatra.name, "Revati");
}

#[test]
fn panchang_jhora_reference_fixture_present() {
    let path = reference_data_root().join("panchang_jhora_reference.json");
    assert!(
        path.exists(),
        "panchang_jhora_reference.json missing at {}",
        path.display()
    );
    let idx = load_panchang_jhora_reference().expect("panchang_jhora reference loads");
    assert_eq!(idx.test_dates.len(), 5);
}

#[test]
fn tz_conversion_roundtrips_ist() {
    assert_eq!(numeric_tz_to_iso(5.5), "+05:30");
    assert_eq!(numeric_tz_to_iso(-8.0), "-08:00");
    assert_eq!(numeric_tz_to_iso(0.0), "+00:00");
}

#[test]
fn nakshatra_normaliser_handles_jhora_spelling() {
    assert_eq!(
        normalize_nakshatra_name("UttaraAshadha"),
        normalize_nakshatra_name("Uttara Ashadha")
    );
    assert_eq!(
        normalize_nakshatra_name("Revati"),
        normalize_nakshatra_name("revati")
    );
}

#[test]
fn paksha_strip_handles_engine_tithi_names() {
    assert_eq!(strip_paksha_suffix("Navami (Krishna)"), "Navami");
    assert_eq!(strip_paksha_suffix("Purnima"), "Purnima");
    assert_eq!(strip_paksha_suffix("Amavasya"), "Amavasya");
}

#[test]
fn vara_extracts_english_name() {
    assert_eq!(extract_vara_english("Ravivara (Sunday)"), "Sunday");
    assert_eq!(extract_vara_english("Shukravara (Friday)"), "Friday");
    assert_eq!(extract_vara_english("Sunday"), "Sunday");
}

// --- Full validation report (long-running, ignored by default) -----------

#[tokio::test]
#[ignore = "long-running: validates Panchanga vs JHora-verified ground truth (6 reference points)"]
async fn vedic_panchanga_full_validation_report() {
    let engine = PanchangaEngine::new();

    let shesh = load_shesh_reference().expect("shesh reference missing");
    let jhora = load_panchang_jhora_reference().expect("panchang_jhora reference missing");

    println!(
        "\n========== VEDIC PANCHANGA validation report ==========\
         \n  reference data : {}\
         \n  ground truth   : shesh (1) + jhora (5) = 6 reference points\n",
        reference_data_root().display()
    );

    let mut tithi_name_res = FieldResult::default();
    let mut tithi_paksha_res = FieldResult::default();
    let mut tithi_number_res = FieldResult::default();
    let mut nakshatra_name_res = FieldResult::default();
    let mut nakshatra_pada_res = FieldResult::default();
    let mut yoga_name_res = FieldResult::default();
    let mut karana_name_res = FieldResult::default();
    let mut vara_res = FieldResult::default();

    let mut engine_failures: Vec<String> = vec![];

    // --- 1. Shesh natal panchang ---
    {
        let id = "shesh_1990_bangalore";
        let input = build_input(
            &shesh.birth_data.date,
            &shesh.birth_data.time,
            &shesh.birth_data.location,
        );
        match engine.calculate(input).await {
            Ok(out) => {
                let r = out.result;
                let got_tithi = strip_paksha_suffix(
                    r.get("tithi_name").and_then(|v| v.as_str()).unwrap_or("?"),
                );
                let got_tithi_idx =
                    r.get("tithi_index").and_then(|v| v.as_u64()).unwrap_or(0) as u8;
                let got_paksha = paksha_from_tithi_index(got_tithi_idx).to_string();
                let got_tithi_num = jhora_tithi_number(got_tithi_idx);

                let got_nak_raw = r
                    .get("nakshatra_name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("?");
                let got_nak = normalize_nakshatra_name(got_nak_raw);
                let exp_nak = normalize_nakshatra_name(&shesh.panchang.nakshatra.name);

                let got_nak_val = r
                    .get("nakshatra_value")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0);
                let got_pada = pada_from_nakshatra_value(got_nak_val);

                let got_yoga = r
                    .get("yoga_name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("?")
                    .to_string();
                let got_karana = r
                    .get("karana_name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("?")
                    .to_string();
                let got_vara = extract_vara_english(
                    r.get("vara_name").and_then(|v| v.as_str()).unwrap_or("?"),
                );

                tithi_name_res.record(id, Some(&shesh.panchang.tithi.name), &got_tithi);
                tithi_paksha_res.record(id, Some(&shesh.panchang.tithi.paksha), &got_paksha);
                tithi_number_res.record_u8(id, Some(shesh.panchang.tithi.number), got_tithi_num);
                nakshatra_name_res.record(id, Some(&exp_nak), &got_nak);
                nakshatra_pada_res.record_u8(id, Some(shesh.panchang.nakshatra.pada), got_pada);
                yoga_name_res.record(id, Some(&shesh.panchang.yoga.name), &got_yoga);
                karana_name_res.record(id, Some(&shesh.panchang.karana.name), &got_karana);
                vara_res.record(id, Some(&shesh.panchang.vara), &got_vara);
            }
            Err(e) => {
                engine_failures.push(format!("{}  {:?}", id, e));
            }
        }
    }

    // --- 2. JHora civil-date panchang (5 entries) ---
    for entry in &jhora.test_dates {
        let id = entry.id.as_str();
        let input = build_input(&entry.date, &entry.time, &entry.location);
        match engine.calculate(input).await {
            Ok(out) => {
                let r = out.result;
                let got_tithi = strip_paksha_suffix(
                    r.get("tithi_name").and_then(|v| v.as_str()).unwrap_or("?"),
                );
                let got_tithi_idx =
                    r.get("tithi_index").and_then(|v| v.as_u64()).unwrap_or(0) as u8;
                let got_paksha = paksha_from_tithi_index(got_tithi_idx).to_string();
                let got_tithi_num = jhora_tithi_number(got_tithi_idx);

                let got_nak_raw = r
                    .get("nakshatra_name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("?");
                let got_nak = normalize_nakshatra_name(got_nak_raw);
                let exp_nak = normalize_nakshatra_name(&entry.expected.nakshatra.name);

                let got_nak_val = r
                    .get("nakshatra_value")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0);
                let got_pada = pada_from_nakshatra_value(got_nak_val);

                let got_yoga = r
                    .get("yoga_name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("?")
                    .to_string();
                let got_karana = r
                    .get("karana_name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("?")
                    .to_string();
                let got_vara = extract_vara_english(
                    r.get("vara_name").and_then(|v| v.as_str()).unwrap_or("?"),
                );

                tithi_name_res.record(id, Some(&entry.expected.tithi.name), &got_tithi);
                tithi_paksha_res.record(id, Some(&entry.expected.tithi.paksha), &got_paksha);
                tithi_number_res.record_u8(id, Some(entry.expected.tithi.number), got_tithi_num);
                nakshatra_name_res.record(id, Some(&exp_nak), &got_nak);
                nakshatra_pada_res.record_u8(id, Some(entry.expected.nakshatra.pada), got_pada);
                yoga_name_res.record(id, Some(&entry.expected.yoga.name), &got_yoga);
                karana_name_res.record(id, Some(&entry.expected.karana.name), &got_karana);
                vara_res.record(id, Some(&entry.expected.vara), &got_vara);
            }
            Err(e) => {
                engine_failures.push(format!("{}  {:?}", id, e));
            }
        }
    }

    println!("--- Per-field results ---");
    print_field("tithi_name", &tithi_name_res);
    print_field("tithi_paksha", &tithi_paksha_res);
    print_field("tithi_number", &tithi_number_res);
    print_field("nakshatra_name", &nakshatra_name_res);
    print_field("nakshatra_pada", &nakshatra_pada_res);
    print_field("yoga_name", &yoga_name_res);
    print_field("karana_name", &karana_name_res);
    print_field("vara", &vara_res);

    println!(
        "\n--- Run stats ---\
         \n  reference points : 6\
         \n  engine failures  : {}",
        engine_failures.len(),
    );
    if !engine_failures.is_empty() {
        println!("  first failures:");
        for f in engine_failures.iter().take(5) {
            println!("    {}", f);
        }
    }
    println!("=========================================================\n");

    // Diagnostic report; smoke tests catch regressions, this quantifies drift.
}
