use chrono::{TimeZone, Utc};
use engine_biofield::BiofieldEngine;
use engine_biorhythm::BiorhythmEngine;
use engine_face_reading::FaceReadingEngine;
use engine_gene_keys::GeneKeysEngine;
use engine_human_design::HumanDesignEngine;
use engine_nadabrahman::NadaBrahmanEngine;
use engine_numerology::NumerologyEngine;
use engine_panchanga::PanchangaEngine;
use engine_transits::TransitsEngine;
use engine_vedic_clock::VedicClockEngine;
use engine_vimshottari::VimshottariEngine;
use noesis_core::{BirthData, ConsciousnessEngine, EngineInput, Precision};
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;

fn canonical_input() -> EngineInput {
    let mut options = HashMap::new();
    options.insert("question".to_string(), json!("baseline trait conformance"));

    EngineInput {
        birth_data: Some(BirthData {
            name: Some("Shesh".to_string()),
            date: "1991-08-13".to_string(),
            time: Some("13:31".to_string()),
            latitude: 12.9340,
            longitude: 77.6214,
            timezone: "Asia/Kolkata".to_string(),
        }),
        current_time: Utc.with_ymd_and_hms(2026, 3, 10, 6, 30, 0).unwrap(),
        location: None,
        precision: Precision::Standard,
        options,
    }
}

fn assert_engine_conformance(
    engine: &dyn ConsciousnessEngine,
    expected_id: &str,
    expected_phase: u8,
) {
    assert_eq!(engine.engine_id(), expected_id);
    assert!(!engine.engine_name().trim().is_empty());
    assert_eq!(engine.required_phase(), expected_phase);
    assert!(engine.required_phase() <= 3);

    let input = canonical_input();
    let key1 = engine.cache_key(&input);
    let key2 = engine.cache_key(&input);

    assert!(!key1.trim().is_empty());
    assert_eq!(key1, key2);
}

#[test]
fn trait_conformance_biofield() {
    let engine = BiofieldEngine::new();
    assert_engine_conformance(&engine, "biofield", 1);
}

#[test]
fn trait_conformance_biorhythm() {
    let engine = BiorhythmEngine::new();
    assert_engine_conformance(&engine, "biorhythm", 0);
}

#[test]
fn trait_conformance_face_reading() {
    let engine = FaceReadingEngine::new();
    assert_engine_conformance(&engine, "face-reading", 1);
}

#[test]
fn trait_conformance_gene_keys() {
    let hd_engine = Arc::new(HumanDesignEngine::new());
    let engine = GeneKeysEngine::with_hd_engine(hd_engine);
    assert_engine_conformance(&engine, "gene-keys", 2);
}

#[test]
fn trait_conformance_human_design() {
    let engine = HumanDesignEngine::new();
    assert_engine_conformance(&engine, "human-design", 1);
}

#[test]
fn trait_conformance_nadabrahman() {
    let engine = NadaBrahmanEngine::new();
    assert_engine_conformance(&engine, "nadabrahman", 0);
}

#[test]
fn trait_conformance_numerology() {
    let engine = NumerologyEngine::new();
    assert_engine_conformance(&engine, "numerology", 0);
}

#[test]
fn trait_conformance_panchanga() {
    let engine = PanchangaEngine::new();
    assert_engine_conformance(&engine, "panchanga", 0);
}

#[test]
fn trait_conformance_transits() {
    let engine = TransitsEngine::new();
    assert_engine_conformance(&engine, "transits", 0);
}

#[test]
fn trait_conformance_vedic_clock() {
    let engine = VedicClockEngine::new();
    assert_engine_conformance(&engine, "vedic-clock", 0);
}

#[test]
fn trait_conformance_vimshottari() {
    let hd_engine = Arc::new(HumanDesignEngine::new());
    let engine = VimshottariEngine::with_hd_engine(hd_engine);
    assert_engine_conformance(&engine, "vimshottari", 2);
}
