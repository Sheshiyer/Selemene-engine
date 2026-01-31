//! Reference Chart Validation Tests
//!
//! 8 reference charts validated against known expected activation sequences.
//! Each test verifies:
//! 1. All 4 activation sequences match expected values
//! 2. Witness prompt is non-empty
//! 3. Frequency assessments exist for all active keys
//! 4. Engine validation passes

use engine_gene_keys::{
    GeneKeysEngine, ConsciousnessEngine, EngineInput,
};
use chrono::Utc;
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
struct ReferenceChart {
    name: String,
    description: String,
    hd_gates: HdGates,
    expected: ExpectedSequences,
}

#[derive(Debug, Deserialize)]
struct HdGates {
    personality_sun: u8,
    personality_earth: u8,
    design_sun: u8,
    design_earth: u8,
}

#[derive(Debug, Deserialize)]
struct ExpectedSequences {
    lifes_work: [u8; 2],
    evolution: [u8; 2],
    radiance: [u8; 2],
    purpose: [u8; 2],
}

fn load_reference_charts() -> Vec<ReferenceChart> {
    let json_str = include_str!("reference_charts.json");
    serde_json::from_str(json_str).expect("Failed to parse reference_charts.json")
}

fn create_input_from_gates(gates: &HdGates) -> EngineInput {
    let mut options = HashMap::new();
    options.insert("hd_gates".to_string(), json!({
        "personality_sun": gates.personality_sun,
        "personality_earth": gates.personality_earth,
        "design_sun": gates.design_sun,
        "design_earth": gates.design_earth
    }));

    EngineInput {
        birth_data: None,
        current_time: Utc::now(),
        location: None,
        precision: noesis_core::Precision::Standard,
        options,
    }
}

async fn validate_reference_chart(index: usize) {
    let charts = load_reference_charts();
    let chart = &charts[index];
    let engine = GeneKeysEngine::new();
    let input = create_input_from_gates(&chart.hd_gates);

    // Calculate
    let output = engine.calculate(input).await
        .unwrap_or_else(|e| panic!("[{}] Calculation failed: {:?}", chart.name, e));

    // Verify activation sequences
    let seq = output.result.get("activation_sequence")
        .unwrap_or_else(|| panic!("[{}] Missing activation_sequence", chart.name));

    let lifes_work = seq.get("lifes_work").unwrap();
    assert_eq!(
        (lifes_work[0].as_u64().unwrap() as u8, lifes_work[1].as_u64().unwrap() as u8),
        (chart.expected.lifes_work[0], chart.expected.lifes_work[1]),
        "[{}] Life's Work mismatch", chart.name
    );

    let evolution = seq.get("evolution").unwrap();
    assert_eq!(
        (evolution[0].as_u64().unwrap() as u8, evolution[1].as_u64().unwrap() as u8),
        (chart.expected.evolution[0], chart.expected.evolution[1]),
        "[{}] Evolution mismatch", chart.name
    );

    let radiance = seq.get("radiance").unwrap();
    assert_eq!(
        (radiance[0].as_u64().unwrap() as u8, radiance[1].as_u64().unwrap() as u8),
        (chart.expected.radiance[0], chart.expected.radiance[1]),
        "[{}] Radiance mismatch", chart.name
    );

    let purpose = seq.get("purpose").unwrap();
    assert_eq!(
        (purpose[0].as_u64().unwrap() as u8, purpose[1].as_u64().unwrap() as u8),
        (chart.expected.purpose[0], chart.expected.purpose[1]),
        "[{}] Purpose mismatch", chart.name
    );

    // Verify witness prompt is non-empty
    assert!(
        !output.witness_prompt.is_empty(),
        "[{}] Witness prompt is empty", chart.name
    );

    // Verify witness prompt contains question marks (inquiry format)
    assert!(
        output.witness_prompt.contains('?'),
        "[{}] Witness prompt should be inquiry format", chart.name
    );

    // Verify frequency assessments exist
    let freq = output.result.get("frequency_assessments")
        .unwrap_or_else(|| panic!("[{}] Missing frequency_assessments", chart.name));
    let freq_array = freq.as_array()
        .unwrap_or_else(|| panic!("[{}] frequency_assessments is not an array", chart.name));
    assert!(
        !freq_array.is_empty(),
        "[{}] frequency_assessments is empty", chart.name
    );

    // Verify active_keys exist
    let active_keys = output.result.get("active_keys")
        .unwrap_or_else(|| panic!("[{}] Missing active_keys", chart.name));
    let keys_array = active_keys.as_array()
        .unwrap_or_else(|| panic!("[{}] active_keys is not an array", chart.name));
    assert_eq!(
        keys_array.len(), 4,
        "[{}] Expected 4 active keys (Sun/Earth pairs)", chart.name
    );

    // Verify engine validation passes
    let validation = engine.validate(&output).await
        .unwrap_or_else(|e| panic!("[{}] Validation call failed: {:?}", chart.name, e));
    assert!(
        validation.valid,
        "[{}] Validation failed: {:?}", chart.name, validation.messages
    );
    assert_eq!(
        validation.confidence, 1.0,
        "[{}] Expected confidence 1.0", chart.name
    );
}

#[tokio::test]
async fn test_reference_chart_1_classic_generator() {
    validate_reference_chart(0).await;
}

#[tokio::test]
async fn test_reference_chart_2_creative_visionary() {
    validate_reference_chart(1).await;
}

#[tokio::test]
async fn test_reference_chart_3_gate_boundaries() {
    validate_reference_chart(2).await;
}

#[tokio::test]
async fn test_reference_chart_4_leadership_channel() {
    validate_reference_chart(3).await;
}

#[tokio::test]
async fn test_reference_chart_5_emotional_wave() {
    validate_reference_chart(4).await;
}

#[tokio::test]
async fn test_reference_chart_6_sacral_power() {
    validate_reference_chart(5).await;
}

#[tokio::test]
async fn test_reference_chart_7_root_pressure() {
    validate_reference_chart(6).await;
}

#[tokio::test]
async fn test_reference_chart_8_splenic_awareness() {
    validate_reference_chart(7).await;
}
