//! Vedic Vimshottari dasha validation harness (PR4 — Phase 1).
//!
//! Compares `engine-vimshottari` output against the on-disk JHora-verified
//! reference data in
//! `crates/noesis-vedic-api/tests/fixtures/reference_data/`.
//!
//! Ground-truth sources:
//!   - `shesh_chart_reference.json`  (1 chart, dasha section)
//!   - `dasha_reference.json`        (3 reference charts × mahadasha timeline)
//!
//! Fields validated (per `GROUND_TRUTH.md` §B diff strategy):
//!   - moon_nakshatra name        — string exact (whitespace normalised)
//!   - mahadasha planet sequence  — first N planets compared in order
//!   - mahadasha start_date       — parse YYYY-MM-DD, abs diff in days, ±15
//!   - birth dasha planet         — first mahadasha planet (string exact)
//!   - birth dasha balance years  — first mahadasha duration_years, ±0.1 yr
//!
//! Run smoke only (fast):
//!     cargo test --package engine-vimshottari --test vedic_validation_tests
//!
//! Run full report (longer, prints accuracy table):
//!     cargo test --package engine-vimshottari --test vedic_validation_tests \
//!         -- --ignored --nocapture

use chrono::NaiveDate;
use engine_vimshottari::VimshottariEngine;
use noesis_core::{BirthData, ConsciousnessEngine, EngineInput, Precision};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

// --- Reference fixture types --------------------------------------------

#[derive(Debug, Deserialize)]
struct SheshReference {
    birth_data: ShshBirth,
    dasha: SheshDasha,
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
    timezone: f64,
}

#[derive(Debug, Deserialize)]
struct SheshDasha {
    moon_nakshatra: String,
    #[allow(dead_code)]
    moon_longitude: f64,
    #[allow(dead_code)]
    nakshatra_ruler: String,
    birth_dasha_balance: BirthDashaBalance,
    mahadashas: Vec<RefMahadasha>,
}

#[derive(Debug, Deserialize)]
struct BirthDashaBalance {
    planet: String,
    years_remaining: f64,
    #[allow(dead_code)]
    total_period_years: f64,
}

#[derive(Debug, Deserialize)]
struct RefMahadasha {
    planet: String,
    start_date: String,
    #[allow(dead_code)]
    end_date: String,
    duration_years: f64,
    #[allow(dead_code)]
    is_birth_dasha: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct DashaReferenceIndex {
    reference_charts: Vec<ReferenceChart>,
}

#[derive(Debug, Deserialize)]
struct ReferenceChart {
    id: String,
    birth_data: ShshBirth,
    moon_data: MoonData,
    expected_mahadashas: Vec<RefMahadasha>,
}

#[derive(Debug, Deserialize)]
struct MoonData {
    nakshatra: String,
    #[allow(dead_code)]
    nakshatra_number: u8,
    #[allow(dead_code)]
    pada: u8,
}

// --- Paths ---------------------------------------------------------------

/// Resolve `<workspace_root>/crates/noesis-vedic-api/tests/fixtures/reference_data`.
/// Same two-parent climb used by the panchanga harness.
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

fn load_dasha_reference() -> Option<DashaReferenceIndex> {
    let path = reference_data_root().join("dasha_reference.json");
    if !path.exists() {
        return None;
    }
    let s = fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {}", path.display(), e));
    Some(serde_json::from_str(&s).expect("parse dasha_reference.json"))
}

// --- Per-field accuracy harness ------------------------------------------

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

    fn record_float(&mut self, label: &str, expected: Option<f64>, got: f64, tol: f64) {
        match expected {
            None => self.skipped += 1,
            Some(e) => {
                self.checked += 1;
                if (e - got).abs() <= tol {
                    self.matched += 1;
                } else if self.mismatched.len() < 10 {
                    self.mismatched.push(format!(
                        "  {}  expected={:.3}  got={:.3}  diff={:.3}",
                        label,
                        e,
                        got,
                        (e - got).abs()
                    ));
                }
            }
        }
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
        "  {:<28}  matched {:>3}/{:>3}  ({:>5.1}%)  skipped={:>2}",
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

// --- Helpers -------------------------------------------------------------

fn normalize_nakshatra_name(name: &str) -> String {
    name.chars()
        .filter(|c| !c.is_whitespace())
        .flat_map(char::to_lowercase)
        .collect()
}

fn numeric_tz_to_iso(offset_hours: f64) -> String {
    let sign = if offset_hours < 0.0 { '-' } else { '+' };
    let abs = offset_hours.abs();
    let hours = abs.trunc() as u8;
    let minutes = ((abs - abs.trunc()) * 60.0).round() as u8;
    format!("{}{:02}:{:02}", sign, hours, minutes)
}

fn build_input(date: &str, time: &str, loc: &RefLocation) -> EngineInput {
    let tz_str = numeric_tz_to_iso(loc.timezone);
    // current_time fixed to the birth date so first mahadasha is "current"
    // and downstream period-finding doesn't depend on wall-clock at test time.
    let current = NaiveDate::parse_from_str(date, "%Y-%m-%d")
        .ok()
        .and_then(|d| d.and_hms_opt(12, 0, 0))
        .map(|ndt| chrono::Utc.from_utc_datetime(&ndt))
        .unwrap_or_else(chrono::Utc::now);
    EngineInput {
        birth_data: Some(BirthData {
            name: None,
            date: date.to_string(),
            time: Some(time.to_string()),
            latitude: loc.latitude,
            longitude: loc.longitude,
            timezone: tz_str,
        }),
        current_time: current,
        location: None,
        precision: Precision::default(),
        options: HashMap::new(),
    }
}

fn date_diff_days(a: &str, b: &str) -> Option<i64> {
    let da = NaiveDate::parse_from_str(a, "%Y-%m-%d").ok()?;
    let db = NaiveDate::parse_from_str(b, "%Y-%m-%d").ok()?;
    Some((da - db).num_days().abs())
}

/// Extract YYYY-MM-DD from an RFC3339 datetime string. Engine serialises
/// mahadasha start/end as RFC3339; refs are bare dates. We compare on the
/// date portion only.
fn rfc3339_to_date(s: &str) -> Option<String> {
    s.split('T').next().map(|d| d.to_string())
}

// We need `TimeZone` for `from_utc_datetime`. Import via re-export from chrono.
use chrono::TimeZone;

// --- Smoke tests ---------------------------------------------------------

#[test]
fn shesh_reference_dasha_present() {
    let r = load_shesh_reference().expect("shesh reference missing");
    assert_eq!(r.dasha.moon_nakshatra, "Revati");
    assert_eq!(r.dasha.birth_dasha_balance.planet, "Mercury");
    assert!(!r.dasha.mahadashas.is_empty());
}

#[test]
fn dasha_reference_three_charts_present() {
    let idx = load_dasha_reference().expect("dasha reference missing");
    assert_eq!(idx.reference_charts.len(), 3);
    assert_eq!(idx.reference_charts[0].id, "chart_1_swami_vivekananda");
}

#[test]
fn tz_conversion_handles_ist_and_negatives() {
    assert_eq!(numeric_tz_to_iso(5.5), "+05:30");
    assert_eq!(numeric_tz_to_iso(-5.0), "-05:00");
}

#[test]
fn rfc3339_to_date_strips_time() {
    assert_eq!(
        rfc3339_to_date("1990-07-15T09:00:00+00:00").as_deref(),
        Some("1990-07-15")
    );
}

#[test]
fn nakshatra_normaliser_collapses_whitespace() {
    assert_eq!(
        normalize_nakshatra_name("UttaraAshadha"),
        normalize_nakshatra_name("Uttara Ashadha")
    );
}

// --- Full validation report (long-running, ignored by default) -----------

#[tokio::test]
#[ignore = "long-running: validates Vimshottari vs JHora-verified ground truth (4 reference charts)"]
async fn vedic_vimshottari_full_validation_report() {
    let engine = VimshottariEngine::new();
    let shesh = load_shesh_reference().expect("shesh reference missing");
    let dasha = load_dasha_reference().expect("dasha reference missing");

    println!(
        "\n========== VEDIC VIMSHOTTARI validation report ==========\
         \n  reference data : {}\
         \n  ground truth   : shesh + 3 dasha_reference charts = 4 reference points\n",
        reference_data_root().display()
    );

    let mut nak_res = FieldResult::default();
    let mut birth_planet_res = FieldResult::default();
    let mut birth_balance_res = FieldResult::default();
    let mut sequence_res = FieldResult::default();
    let mut start_date_res = FieldResult::default();

    let mut engine_failures: Vec<String> = vec![];

    // Build a uniform list of {id, birth_data, expected_nakshatra, expected_mahadashas}
    struct Chart<'a> {
        id: &'a str,
        date: &'a str,
        time: &'a str,
        loc: &'a RefLocation,
        expected_nak: &'a str,
        expected_mds: &'a [RefMahadasha],
        expected_balance_planet: Option<&'a str>,
        expected_balance_years: Option<f64>,
    }
    let mut charts: Vec<Chart> = Vec::with_capacity(4);

    // Shesh first
    charts.push(Chart {
        id: "shesh_1990_bangalore",
        date: &shesh.birth_data.date,
        time: &shesh.birth_data.time,
        loc: &shesh.birth_data.location,
        expected_nak: &shesh.dasha.moon_nakshatra,
        expected_mds: &shesh.dasha.mahadashas,
        expected_balance_planet: Some(&shesh.dasha.birth_dasha_balance.planet),
        expected_balance_years: Some(shesh.dasha.birth_dasha_balance.years_remaining),
    });
    // The 3 dasha_reference charts
    for c in &dasha.reference_charts {
        charts.push(Chart {
            id: &c.id,
            date: &c.birth_data.date,
            time: &c.birth_data.time,
            loc: &c.birth_data.location,
            expected_nak: &c.moon_data.nakshatra,
            expected_mds: &c.expected_mahadashas,
            expected_balance_planet: c
                .expected_mahadashas
                .iter()
                .find(|m| m.is_birth_dasha == Some(true))
                .map(|m| m.planet.as_str()),
            expected_balance_years: c
                .expected_mahadashas
                .iter()
                .find(|m| m.is_birth_dasha == Some(true))
                .map(|m| m.duration_years),
        });
    }

    for chart in &charts {
        let input = build_input(chart.date, chart.time, chart.loc);
        let result = match engine.calculate(input).await {
            Ok(out) => out.result,
            Err(e) => {
                engine_failures.push(format!("{}  {:?}", chart.id, e));
                continue;
            }
        };

        // Moon nakshatra
        let got_nak_raw = result
            .get("birth_nakshatra")
            .and_then(|v| v.get("name"))
            .and_then(|v| v.as_str())
            .unwrap_or("?");
        let got_nak = normalize_nakshatra_name(got_nak_raw);
        let exp_nak = normalize_nakshatra_name(chart.expected_nak);
        nak_res.record(chart.id, Some(&exp_nak), &got_nak);

        // Mahadashas: timeline.mahadashas: array
        let got_mds = result
            .get("timeline")
            .and_then(|v| v.get("mahadashas"))
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        // Birth dasha planet = first mahadasha planet
        if let Some(first) = got_mds.first() {
            let got_planet = first
                .get("planet")
                .and_then(|v| v.as_str())
                .unwrap_or("?")
                .to_string();
            birth_planet_res.record(chart.id, chart.expected_balance_planet, &got_planet);

            // Birth dasha balance: first mahadasha duration_years
            let got_years = first
                .get("duration_years")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);
            birth_balance_res.record_float(chart.id, chart.expected_balance_years, got_years, 0.1);
        } else {
            birth_planet_res.record(chart.id, chart.expected_balance_planet, "?");
            birth_balance_res.record_float(chart.id, chart.expected_balance_years, 0.0, 0.1);
        }

        // Sequence: compare planets in order, up to len(expected)
        let exp_seq: Vec<String> = chart
            .expected_mds
            .iter()
            .map(|m| m.planet.clone())
            .collect();
        let got_seq: Vec<String> = got_mds
            .iter()
            .take(exp_seq.len())
            .map(|m| {
                m.get("planet")
                    .and_then(|v| v.as_str())
                    .unwrap_or("?")
                    .to_string()
            })
            .collect();
        let exp_seq_s = exp_seq.join(",");
        let got_seq_s = got_seq.join(",");
        sequence_res.record(chart.id, Some(&exp_seq_s), &got_seq_s);

        // Start dates: compare first 3 (or fewer) mahadasha start dates within ±15 days
        for (idx, exp_md) in chart.expected_mds.iter().take(3).enumerate() {
            let label = format!("{}/md{}", chart.id, idx + 1);
            let got_md = got_mds.get(idx);
            let got_start_raw = got_md
                .and_then(|m| m.get("start_date"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let got_start = rfc3339_to_date(got_start_raw).unwrap_or_else(|| "?".to_string());
            let diff = date_diff_days(&exp_md.start_date, &got_start);
            match diff {
                Some(d) if d <= 15 => {
                    start_date_res.checked += 1;
                    start_date_res.matched += 1;
                }
                Some(d) => {
                    start_date_res.checked += 1;
                    if start_date_res.mismatched.len() < 10 {
                        start_date_res.mismatched.push(format!(
                            "  {}  expected={}  got={}  diff={}d",
                            label, exp_md.start_date, got_start, d
                        ));
                    }
                }
                None => start_date_res.skipped += 1,
            }
        }
    }

    println!("--- Per-field results ---");
    print_field("moon_nakshatra_name", &nak_res);
    print_field("birth_dasha_planet", &birth_planet_res);
    print_field("birth_dasha_balance_years", &birth_balance_res);
    print_field("mahadasha_sequence", &sequence_res);
    print_field("mahadasha_start_date_15d", &start_date_res);

    println!(
        "\n--- Run stats ---\
         \n  reference charts : {}\
         \n  engine failures  : {}",
        charts.len(),
        engine_failures.len(),
    );
    if !engine_failures.is_empty() {
        println!("  first failures:");
        for f in engine_failures.iter().take(5) {
            println!("    {}", f);
        }
    }
    println!("=========================================================\n");
}
