//! Cross-engine smoke validation: run the Numerology engine against every
//! humdes fixture and report successes/failures.
//!
//! Numerology needs only `birth_data.name` and `birth_data.date` (no
//! lat/lng/time), so this loop does NOT gate on `has_coords` — every fixture
//! is exercised.
//!
//! Run only the smoke tests (fast, run on every `cargo test`):
//!     cargo test --package engine-numerology --test humdes_smoke_tests
//!
//! Run the long-running per-fixture report:
//!     cargo test --package engine-numerology --test humdes_smoke_tests \
//!         -- --ignored --nocapture

use engine_numerology::NumerologyEngine;
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
    #[allow(dead_code)]
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
fn humdes_first_3_inputs_have_name_and_date() {
    let idx = match load_index() {
        Some(i) => i,
        None => {
            eprintln!("(skipped — no _index.json)");
            return;
        }
    };
    assert!(idx.total_persons > 0, "no persons in index");
    let n = idx.entries.len().min(3);
    for entry in &idx.entries[..n] {
        let input = load_input(&entry.input);
        let bd = input
            .birth_data
            .as_ref()
            .unwrap_or_else(|| panic!("missing birth_data in {}", entry.input));
        // Numerology needs name + date.
        assert!(
            bd.name.as_ref().map(|n| !n.is_empty()).unwrap_or(false),
            "missing/empty name in {}",
            entry.input
        );
        assert!(!bd.date.is_empty(), "empty date in {}", entry.input);
    }
}

// --- Full smoke report (long-running, ignored by default) ----------------

#[tokio::test]
#[ignore = "long-running: runs Numerology engine on every humdes fixture (~89 persons)"]
async fn humdes_numerology_smoke_all_fixtures() {
    let idx = load_index().expect("humdes fixtures missing — run humdes_to_selemene.py first");
    println!(
        "\n========== HUMDES Numerology smoke ==========\
         \n  source persons : {}\
         \n  fixtures path  : {}\n",
        idx.total_persons,
        fixtures_root().display()
    );

    let engine = NumerologyEngine::new();
    let mut ok = 0_usize;
    let mut fail = 0_usize;
    let mut skipped_no_name = 0_usize;
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

        let input = load_input(&entry.input);
        // Numerology requires a non-empty name; skip (counted) if missing
        // so the report distinguishes "fixture-side gap" from "engine-side bug".
        let has_name = input
            .birth_data
            .as_ref()
            .and_then(|bd| bd.name.as_ref())
            .map(|n| !n.trim().is_empty())
            .unwrap_or(false);
        if !has_name {
            skipped_no_name += 1;
            continue;
        }

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

    println!("--- numerology humdes smoke ---");
    println!("  ok               : {}", ok);
    println!("  fail             : {}", fail);
    println!("  skipped (no name): {}", skipped_no_name);
    if ok > 0 {
        println!("  avg calc ms      : {:.4}", total_ms / ok as f64);
    }
    if !failures.is_empty() {
        println!("  first failures   :");
        for f in &failures {
            println!("{}", f);
        }
    }
    println!("==============================================\n");

    // Smoke contract: at least one fixture ran successfully.
    assert!(
        ok > 0 || skipped_no_name == idx.entries.len(),
        "numerology produced zero ok results across {} named fixtures",
        idx.entries.len() - skipped_no_name
    );
}
