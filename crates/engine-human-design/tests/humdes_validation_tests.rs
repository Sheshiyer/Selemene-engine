//! Validation tests against humdes.com captured data.
//!
//! Reads fixtures under `tests/fixtures/humdes/` (produced by the
//! humdes-extractor's `humdes_to_selemene.py` Phase-1 normaliser) and runs
//! the HD engine against each. Compares engine output to humdes ground truth
//! on the cardinal fields and emits a per-field accuracy report.
//!
//! Fields validated (ground truth from humdes):
//!   - type (Generator | ManifestingGenerator | Projector | Manifestor | Reflector)
//!   - profile (e.g. "3/5")
//!   - authority (Sacral | Emotional | Splenic | Heart | GCenter | Mental | Lunar)
//!   - personality Sun gate
//!   - personality Earth gate
//!   - design Sun gate
//!   - design Earth gate
//!   - incarnation cross gates (4 gates: P-sun, P-earth, D-sun, D-earth)
//!
//! Run only this suite:
//!     cargo test --package engine-human-design --test humdes_validation_tests \
//!         -- --ignored --nocapture
//!
//! Or just the smoke tests (no --ignored):
//!     cargo test --package engine-human-design --test humdes_validation_tests

use noesis_core::{ConsciousnessEngine, EngineInput};
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};

use engine_human_design::HumanDesignEngine;

// --- Fixture types -------------------------------------------------------

#[derive(Debug, Deserialize)]
struct HumdesIndex {
    #[allow(dead_code)]
    source: String,
    total_persons: usize,
    entries: Vec<HumdesEntry>,
}

#[derive(Debug, Deserialize, Clone)]
struct HumdesEntry {
    #[serde(rename = "type")]
    reading_type: String,
    reading_hash: String,
    #[allow(dead_code)]
    reading_name: String,
    person_index: u32,
    #[allow(dead_code)]
    person_name: Option<String>,
    input: String,
    expected: String,
    #[allow(dead_code)]
    metadata: String,
    has_coords: bool,
    #[allow(dead_code)]
    hd_type: Option<String>,
    #[allow(dead_code)]
    authority: Option<String>,
    #[allow(dead_code)]
    profile: Option<String>,
}

#[derive(Debug, Deserialize)]
struct HumdesExpectedFile {
    #[allow(dead_code)]
    name: Option<String>,
    expected: HumdesExpectedFields,
}

#[derive(Debug, Deserialize)]
struct HumdesExpectedFields {
    #[serde(rename = "type")]
    type_: Option<String>,
    profile: Option<HumdesProfile>,
    authority: Option<String>,
    personality_sun: HumdesGate,
    personality_earth: HumdesGate,
    design_sun: HumdesGate,
    design_earth: HumdesGate,
    #[allow(dead_code)]
    variables: Vec<String>,
    incarnation_cross: Option<HumdesCross>,
}

#[derive(Debug, Deserialize)]
struct HumdesProfile {
    conscious_line: u8,
    unconscious_line: u8,
    #[allow(dead_code)]
    text: String,
}

#[derive(Debug, Deserialize)]
struct HumdesGate {
    gate: Option<u8>,
}

#[derive(Debug, Deserialize)]
struct HumdesCross {
    #[allow(dead_code)]
    name: String,
    gates: Vec<Option<u8>>,
}

// --- Paths ---------------------------------------------------------------

/// Resolve `<workspace_root>/tests/fixtures/humdes`.
/// CARGO_MANIFEST_DIR points to the engine crate, so we go up two parents.
fn fixtures_root() -> PathBuf {
    let crate_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    // crate_dir = .../Selemene-engine/crates/engine-human-design
    crate_dir
        .parent()
        .and_then(|p| p.parent())
        .map(|p| p.join("tests/fixtures/humdes"))
        .expect("locate workspace root from crate dir")
}

fn load_index() -> Option<HumdesIndex> {
    let path = fixtures_root().join("_index.json");
    if !path.exists() {
        return None;
    }
    let s = fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("read {}: {}", path.display(), e));
    Some(serde_json::from_str(&s).expect("parse _index.json"))
}

fn load_input(rel: &str) -> EngineInput {
    let path = fixtures_root().join(rel);
    let s = fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("read {}: {}", path.display(), e));
    serde_json::from_str(&s)
        .unwrap_or_else(|e| panic!("parse {}: {}", path.display(), e))
}

fn load_expected(rel: &str) -> HumdesExpectedFile {
    let path = fixtures_root().join(rel);
    let s = fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("read {}: {}", path.display(), e));
    serde_json::from_str(&s)
        .unwrap_or_else(|e| panic!("parse {}: {}", path.display(), e))
}

// --- Diff record ---------------------------------------------------------

#[derive(Debug, Default, Clone)]
struct FieldResult {
    checked: usize,
    matched: usize,
    mismatched: Vec<String>, // (entry_id, expected, got)
    skipped: usize,          // no ground truth in fixture
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
                } else {
                    // Still count, just don't store every example
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

// --- Smoke tests (run on every `cargo test`) ----------------------------

#[test]
fn humdes_fixtures_directory_present() {
    let root = fixtures_root();
    assert!(
        root.exists(),
        "humdes fixtures missing at {} — run \
         `python humdes_to_selemene.py` in humdes-extractor first.",
        root.display()
    );
    assert!(root.join("_index.json").exists(), "_index.json missing");
}

#[test]
fn humdes_first_5_inputs_deserialize_cleanly() {
    let idx = match load_index() {
        Some(i) => i,
        None => {
            eprintln!("(skipped — no _index.json)");
            return;
        }
    };
    assert!(idx.total_persons > 0, "no persons in index");
    let n = idx.entries.len().min(5);
    for entry in &idx.entries[..n] {
        let input = load_input(&entry.input);
        assert!(
            input.birth_data.is_some(),
            "missing birth_data in {}",
            entry.input
        );
        let bd = input.birth_data.unwrap();
        bd.validate().unwrap_or_else(|e| {
            panic!("invalid birth_data in {}: {}", entry.input, e)
        });
    }
}

// --- Full validation report (long-running, ignored by default) -----------

#[tokio::test]
#[ignore = "long-running: runs HD engine on every humdes fixture (~89 persons)"]
async fn humdes_engine_full_validation_report() {
    let idx = load_index().expect(
        "humdes fixtures missing — run humdes_to_selemene.py first",
    );
    println!(
        "\n========== HUMDES validation report ==========\
         \n  source persons : {}\
         \n  fixtures path  : {}\n",
        idx.total_persons,
        fixtures_root().display()
    );

    let engine = HumanDesignEngine::new();

    let mut type_res = FieldResult::default();
    let mut prof_res = FieldResult::default();
    let mut auth_res = FieldResult::default();
    let mut psun_res = FieldResult::default();
    let mut pearth_res = FieldResult::default();
    let mut dsun_res = FieldResult::default();
    let mut dearth_res = FieldResult::default();
    let mut cross_res = FieldResult::default();
    let mut engine_failures: Vec<String> = vec![];
    let mut skipped_no_coords = 0;

    for (i, entry) in idx.entries.iter().enumerate() {
        let id = format!(
            "{:3}/{:3} {} {}/{}",
            i + 1,
            idx.entries.len(),
            entry.reading_type,
            &entry.reading_hash[..10],
            entry.person_index
        );

        if !entry.has_coords {
            skipped_no_coords += 1;
            continue;
        }

        let input = load_input(&entry.input);
        let expected = load_expected(&entry.expected);

        let result = match engine.calculate(input).await {
            Ok(out) => out.result,
            Err(e) => {
                engine_failures.push(format!("{}  {:?}", id, e));
                continue;
            }
        };

        let got_type = result
            .get("hd_type")
            .and_then(|v| v.as_str())
            .unwrap_or("?")
            .to_string();
        let got_profile = result
            .get("profile")
            .and_then(|v| v.as_str())
            .unwrap_or("?")
            .to_string();
        let got_authority = result
            .get("authority")
            .and_then(|v| v.as_str())
            .unwrap_or("?")
            .to_string();

        let got_psun = result
            .get("personality_activations")
            .and_then(|v| v.get("sun"))
            .and_then(|v| v.get("gate"))
            .and_then(|v| v.as_u64())
            .map(|v| v as u8);
        let got_pearth = result
            .get("personality_activations")
            .and_then(|v| v.get("earth"))
            .and_then(|v| v.get("gate"))
            .and_then(|v| v.as_u64())
            .map(|v| v as u8);
        let got_dsun = result
            .get("design_activations")
            .and_then(|v| v.get("sun"))
            .and_then(|v| v.get("gate"))
            .and_then(|v| v.as_u64())
            .map(|v| v as u8);
        let got_dearth = result
            .get("design_activations")
            .and_then(|v| v.get("earth"))
            .and_then(|v| v.get("gate"))
            .and_then(|v| v.as_u64())
            .map(|v| v as u8);

        // Compare scalars
        type_res.record(
            &id,
            expected.expected.type_.as_deref(),
            &got_type,
        );

        let exp_profile_text = expected
            .expected
            .profile
            .as_ref()
            .map(|p| format!("{}/{}", p.conscious_line, p.unconscious_line));
        prof_res.record(&id, exp_profile_text.as_deref(), &got_profile);

        auth_res.record(
            &id,
            expected.expected.authority.as_deref(),
            &got_authority,
        );

        let exp_psun_s = expected.expected.personality_sun.gate.map(|g| g.to_string());
        let got_psun_s = got_psun.map(|g| g.to_string()).unwrap_or_else(|| "?".into());
        psun_res.record(&id, exp_psun_s.as_deref(), &got_psun_s);

        let exp_pearth_s = expected.expected.personality_earth.gate.map(|g| g.to_string());
        let got_pearth_s = got_pearth.map(|g| g.to_string()).unwrap_or_else(|| "?".into());
        pearth_res.record(&id, exp_pearth_s.as_deref(), &got_pearth_s);

        let exp_dsun_s = expected.expected.design_sun.gate.map(|g| g.to_string());
        let got_dsun_s = got_dsun.map(|g| g.to_string()).unwrap_or_else(|| "?".into());
        dsun_res.record(&id, exp_dsun_s.as_deref(), &got_dsun_s);

        let exp_dearth_s = expected.expected.design_earth.gate.map(|g| g.to_string());
        let got_dearth_s = got_dearth.map(|g| g.to_string()).unwrap_or_else(|| "?".into());
        dearth_res.record(&id, exp_dearth_s.as_deref(), &got_dearth_s);

        // Cross: compare 4 ordered gates
        if let Some(cross) = expected.expected.incarnation_cross.as_ref() {
            let exp_cross_s = cross
                .gates
                .iter()
                .map(|g| g.map(|n| n.to_string()).unwrap_or("?".into()))
                .collect::<Vec<_>>()
                .join(",");
            let got_cross_s = format!(
                "{},{},{},{}",
                got_psun.map(|n| n.to_string()).unwrap_or("?".into()),
                got_pearth.map(|n| n.to_string()).unwrap_or("?".into()),
                got_dsun.map(|n| n.to_string()).unwrap_or("?".into()),
                got_dearth.map(|n| n.to_string()).unwrap_or("?".into()),
            );
            cross_res.record(&id, Some(&exp_cross_s), &got_cross_s);
        } else {
            cross_res.skipped += 1;
        }
    }

    // -------- print report --------
    fn print_field(label: &str, r: &FieldResult) {
        println!(
            "  {:<18}  matched {:>3}/{:>3}  ({:>5.1}%)  skipped={:>2}",
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

    println!("\n--- Per-field results ---");
    print_field("type",            &type_res);
    print_field("profile",         &prof_res);
    print_field("authority",       &auth_res);
    print_field("personality_sun", &psun_res);
    print_field("personality_earth", &pearth_res);
    print_field("design_sun",      &dsun_res);
    print_field("design_earth",    &dearth_res);
    print_field("incarnation_cross", &cross_res);

    println!(
        "\n--- Run stats ---\
         \n  fixtures      : {}\
         \n  skipped (no coords) : {}\
         \n  engine failures     : {}",
        idx.entries.len(),
        skipped_no_coords,
        engine_failures.len(),
    );
    if !engine_failures.is_empty() {
        println!("  first failures:");
        for f in engine_failures.iter().take(5) {
            println!("    {}", f);
        }
    }
    println!("==============================================\n");

    // Don't fail the test — this is a diagnostic report. The smoke tests
    // catch regressions; this report quantifies engine vs ground-truth drift.
}

// Helper: ensure paths look right even if fixtures directory moves later.
#[test]
fn fixtures_root_path_is_workspace_relative() {
    let root = fixtures_root();
    let s = root.to_string_lossy();
    assert!(
        s.ends_with("/tests/fixtures/humdes")
            || s.ends_with("\\tests\\fixtures\\humdes"),
        "unexpected fixtures path: {}",
        s
    );
    let _: &Path = root.as_path();
}
