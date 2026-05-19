//! Cross-engine smoke validation: run the Transits engine against every
//! humdes fixture and report successes/failures.
//!
//! Mirrors the `engine-panchanga` and `engine-numerology` smoke harness:
//! same 89 humdes fixtures, same gating on `has_coords`, same accounting.
//!
//! Run only the smoke tests (fast, run on every `cargo test`):
//!     cargo test --package engine-transits --test humdes_smoke_tests
//!
//! Run the long-running per-fixture report:
//!     cargo test --package engine-transits --test humdes_smoke_tests \
//!         -- --ignored --nocapture

use engine_transits::TransitsEngine;
use noesis_core::{ConsciousnessEngine, EngineInput};
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;
use std::time::Instant;

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
    person_index: u32,
    input: String,
    has_coords: bool,
}

// --- Paths ---------------------------------------------------------------

/// Resolve `<workspace_root>/tests/fixtures/humdes`.
/// CARGO_MANIFEST_DIR points to the engine crate, so we go up two parents.
fn fixtures_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
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
    let s = fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {}", path.display(), e));
    Some(serde_json::from_str(&s).expect("parse _index.json"))
}

fn load_input(rel: &str) -> EngineInput {
    let path = fixtures_root().join(rel);
    let s = fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {}", path.display(), e));
    serde_json::from_str(&s).unwrap_or_else(|e| panic!("parse {}: {}", path.display(), e))
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
fn humdes_first_3_inputs_deserialize_for_transits() {
    let idx = match load_index() {
        Some(i) => i,
        None => {
            eprintln!("(skipped — no _index.json)");
            return;
        }
    };
    assert!(idx.total_persons > 0, "no persons in index");
    let mut checked = 0;
    for entry in idx.entries.iter().filter(|e| e.has_coords).take(3) {
        let input = load_input(&entry.input);
        let bd = input
            .birth_data
            .as_ref()
            .unwrap_or_else(|| panic!("missing birth_data in {}", entry.input));
        bd.validate()
            .unwrap_or_else(|e| panic!("invalid birth_data in {}: {}", entry.input, e));
        // Transits needs date + time + tz + coords; assert all exist.
        assert!(!bd.date.is_empty(), "empty date in {}", entry.input);
        assert!(bd.time.is_some(), "missing time in {}", entry.input);
        assert!(!bd.timezone.is_empty(), "empty timezone in {}", entry.input);
        checked += 1;
    }
    assert!(checked > 0, "no coordinated fixtures found");
}

// --- Full smoke report (long-running, ignored by default) ----------------

#[tokio::test]
#[ignore = "long-running: runs Transits engine on every humdes fixture (~89 persons)"]
async fn humdes_transits_smoke_all_fixtures() {
    let idx = load_index().expect("humdes fixtures missing — run humdes_to_selemene.py first");
    println!(
        "\n========== HUMDES Transits smoke ==========\
         \n  source persons : {}\
         \n  fixtures path  : {}\n",
        idx.total_persons,
        fixtures_root().display()
    );

    let engine = TransitsEngine::new();
    let mut ok = 0_usize;
    let mut fail = 0_usize;
    let mut skipped_no_coords = 0_usize;
    let mut failures: Vec<String> = vec![];
    let mut total_ms = 0.0_f64;

    for (i, entry) in idx.entries.iter().enumerate() {
        let id = format!(
            "{:3}/{:3} {} {}",
            i + 1,
            idx.entries.len(),
            entry.reading_type,
            &entry.reading_hash[..10.min(entry.reading_hash.len())],
        );

        if !entry.has_coords {
            skipped_no_coords += 1;
            continue;
        }

        let input = load_input(&entry.input);
        let start = Instant::now();
        match engine.calculate(input).await {
            Ok(out) => {
                ok += 1;
                total_ms += out.metadata.calculation_time_ms;
            }
            Err(e) => {
                fail += 1;
                if failures.len() < 5 {
                    failures.push(format!(
                        "  {}  {}  err={:?}  wall={:.2}ms",
                        id,
                        entry.input,
                        e,
                        start.elapsed().as_secs_f64() * 1000.0
                    ));
                }
            }
        }
    }

    println!("--- transits humdes smoke ---");
    println!("  ok                : {}", ok);
    println!("  fail              : {}", fail);
    println!("  skipped (no coords): {}", skipped_no_coords);
    if ok > 0 {
        println!("  avg calc ms       : {:.3}", total_ms / ok as f64);
    }
    if !failures.is_empty() {
        println!("  first failures    :");
        for f in &failures {
            println!("{}", f);
        }
    }
    println!("==============================================\n");

    assert!(
        ok > 0 || skipped_no_coords == idx.entries.len(),
        "transits produced zero ok results across {} coord-bearing fixtures",
        idx.entries.len() - skipped_no_coords
    );
}
