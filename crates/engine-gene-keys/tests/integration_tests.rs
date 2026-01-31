//! Integration Tests for Gene Keys Engine
//!
//! 14 integration tests covering:
//! - Activation sequence calculation (4 tests)
//! - Frequency assessment accuracy (3 tests)
//! - Witness prompt generation (3 tests)
//! - Transformation pathways (2 tests)
//! - HD integration mode (2 tests)

use engine_gene_keys::{
    GeneKeysEngine, ConsciousnessEngine, EngineInput,
    ActivationSequence, GeneKeysChart, GeneKeyActivation, ActivationSource,
    assess_frequencies, Frequency,
    generate_transformation_pathways, generate_complete_pathways,
    generate_witness_prompt,
    get_gene_key,
};
use chrono::Utc;
use serde_json::{json, Value};
use std::collections::HashMap;

// ============================================================================
// Helper functions
// ============================================================================

fn create_engine_input(ps: u8, pe: u8, ds: u8, de: u8) -> EngineInput {
    let mut options = HashMap::new();
    options.insert("hd_gates".to_string(), json!({
        "personality_sun": ps,
        "personality_earth": pe,
        "design_sun": ds,
        "design_earth": de
    }));

    EngineInput {
        birth_data: None,
        current_time: Utc::now(),
        location: None,
        precision: noesis_core::Precision::Standard,
        options,
    }
}

fn create_engine_input_with_level(ps: u8, pe: u8, ds: u8, de: u8, level: u8) -> EngineInput {
    let mut options = HashMap::new();
    options.insert("hd_gates".to_string(), json!({
        "personality_sun": ps,
        "personality_earth": pe,
        "design_sun": ds,
        "design_earth": de
    }));
    options.insert("consciousness_level".to_string(), json!(level));

    EngineInput {
        birth_data: None,
        current_time: Utc::now(),
        location: None,
        precision: noesis_core::Precision::Standard,
        options,
    }
}

fn create_test_chart(ps: u8, pe: u8, ds: u8, de: u8) -> GeneKeysChart {
    GeneKeysChart {
        activation_sequence: ActivationSequence::from_activations(ps, pe, ds, de),
        active_keys: vec![
            GeneKeyActivation {
                key_number: ps,
                line: 3,
                source: ActivationSource::PersonalitySun,
                gene_key_data: get_gene_key(ps).cloned(),
            },
            GeneKeyActivation {
                key_number: pe,
                line: 3,
                source: ActivationSource::PersonalityEarth,
                gene_key_data: get_gene_key(pe).cloned(),
            },
            GeneKeyActivation {
                key_number: ds,
                line: 3,
                source: ActivationSource::DesignSun,
                gene_key_data: get_gene_key(ds).cloned(),
            },
            GeneKeyActivation {
                key_number: de,
                line: 3,
                source: ActivationSource::DesignEarth,
                gene_key_data: get_gene_key(de).cloned(),
            },
        ],
    }
}

// ============================================================================
// 1. Activation Sequence Calculation (4 tests)
// ============================================================================

#[tokio::test]
async fn test_activation_sequence_lifes_work() {
    let engine = GeneKeysEngine::new();
    let input = create_engine_input(17, 18, 45, 26);
    let output = engine.calculate(input).await.expect("Calculation should succeed");

    let seq = output.result.get("activation_sequence").unwrap();
    let lw = seq.get("lifes_work").unwrap();
    assert_eq!(lw[0].as_u64().unwrap(), 17, "Life's Work first key = Personality Sun");
    assert_eq!(lw[1].as_u64().unwrap(), 18, "Life's Work second key = Personality Earth");
}

#[tokio::test]
async fn test_activation_sequence_evolution() {
    let engine = GeneKeysEngine::new();
    let input = create_engine_input(17, 18, 45, 26);
    let output = engine.calculate(input).await.expect("Calculation should succeed");

    let seq = output.result.get("activation_sequence").unwrap();
    let ev = seq.get("evolution").unwrap();
    assert_eq!(ev[0].as_u64().unwrap(), 45, "Evolution first key = Design Sun");
    assert_eq!(ev[1].as_u64().unwrap(), 26, "Evolution second key = Design Earth");
}

#[tokio::test]
async fn test_activation_sequence_radiance() {
    let engine = GeneKeysEngine::new();
    let input = create_engine_input(1, 2, 13, 7);
    let output = engine.calculate(input).await.expect("Calculation should succeed");

    let seq = output.result.get("activation_sequence").unwrap();
    let rad = seq.get("radiance").unwrap();
    assert_eq!(rad[0].as_u64().unwrap(), 1, "Radiance first key = Personality Sun");
    assert_eq!(rad[1].as_u64().unwrap(), 13, "Radiance second key = Design Sun");
}

#[tokio::test]
async fn test_activation_sequence_purpose() {
    let engine = GeneKeysEngine::new();
    let input = create_engine_input(1, 2, 13, 7);
    let output = engine.calculate(input).await.expect("Calculation should succeed");

    let seq = output.result.get("activation_sequence").unwrap();
    let pur = seq.get("purpose").unwrap();
    assert_eq!(pur[0].as_u64().unwrap(), 2, "Purpose first key = Personality Earth");
    assert_eq!(pur[1].as_u64().unwrap(), 7, "Purpose second key = Design Earth");
}

// ============================================================================
// 2. Frequency Assessment Accuracy (3 tests)
// ============================================================================

#[tokio::test]
async fn test_frequency_assessment_shadow_level() {
    let chart = create_test_chart(17, 18, 45, 26);
    let assessments = assess_frequencies(&chart, Some(1));

    assert!(!assessments.is_empty(), "Should have assessments");
    for assessment in &assessments {
        assert_eq!(
            assessment.suggested_frequency,
            Some(Frequency::Shadow),
            "Level 1 should suggest Shadow frequency for Gene Key {}",
            assessment.gene_key
        );
        assert!(!assessment.shadow.is_empty(), "Shadow name should be present");
        assert!(!assessment.shadow_description.is_empty(), "Shadow description should preserve archetypal depth");
    }
}

#[tokio::test]
async fn test_frequency_assessment_gift_level() {
    let chart = create_test_chart(1, 2, 13, 7);
    let assessments = assess_frequencies(&chart, Some(3));

    assert!(!assessments.is_empty(), "Should have assessments");
    for assessment in &assessments {
        assert_eq!(
            assessment.suggested_frequency,
            Some(Frequency::Gift),
            "Level 3 should suggest Gift frequency for Gene Key {}",
            assessment.gene_key
        );
        assert!(!assessment.gift.is_empty(), "Gift name should be present");
        assert!(!assessment.gift_description.is_empty(), "Gift description should preserve archetypal depth");
    }
}

#[tokio::test]
async fn test_frequency_assessment_siddhi_level() {
    let chart = create_test_chart(64, 63, 1, 2);
    let assessments = assess_frequencies(&chart, Some(5));

    assert!(!assessments.is_empty(), "Should have assessments");
    for assessment in &assessments {
        assert_eq!(
            assessment.suggested_frequency,
            Some(Frequency::Siddhi),
            "Level 5 should suggest Siddhi frequency for Gene Key {}",
            assessment.gene_key
        );
        assert!(!assessment.siddhi.is_empty(), "Siddhi name should be present");
        assert!(!assessment.siddhi_description.is_empty(), "Siddhi description should preserve archetypal depth");
        // Verify recognition prompts exist at all 3 levels
        assert!(!assessment.recognition_prompts.shadow.is_empty());
        assert!(!assessment.recognition_prompts.gift.is_empty());
        assert!(!assessment.recognition_prompts.siddhi.is_empty());
    }
}

// ============================================================================
// 3. Witness Prompt Generation (3 tests)
// ============================================================================

#[tokio::test]
async fn test_witness_prompt_shadow_consciousness_level_1() {
    let engine = GeneKeysEngine::new();
    let input = create_engine_input_with_level(36, 6, 55, 49, 1);
    let output = engine.calculate(input).await.expect("Calculation should succeed");

    assert!(!output.witness_prompt.is_empty(), "Witness prompt should not be empty");
    assert!(output.witness_prompt.contains('?'), "Should be inquiry format");
    assert!(
        output.witness_prompt.to_lowercase().contains("unconscious") ||
        output.witness_prompt.to_lowercase().contains("shadow") ||
        output.witness_prompt.to_lowercase().contains("pattern"),
        "Shadow-level prompt should reference unconscious patterns: {}",
        output.witness_prompt
    );
}

#[tokio::test]
async fn test_witness_prompt_gift_consciousness_level_3() {
    let engine = GeneKeysEngine::new();
    let input = create_engine_input_with_level(5, 35, 14, 8, 3);
    let output = engine.calculate(input).await.expect("Calculation should succeed");

    assert!(!output.witness_prompt.is_empty(), "Witness prompt should not be empty");
    assert!(output.witness_prompt.contains('?'), "Should be inquiry format");
    assert_eq!(output.consciousness_level, 3, "Should reflect requested consciousness level");
}

#[tokio::test]
async fn test_witness_prompt_siddhi_consciousness_level_5() {
    let engine = GeneKeysEngine::new();
    let input = create_engine_input_with_level(48, 21, 57, 51, 5);
    let output = engine.calculate(input).await.expect("Calculation should succeed");

    assert!(!output.witness_prompt.is_empty(), "Witness prompt should not be empty");
    assert!(output.witness_prompt.contains('?'), "Should be inquiry format");
    assert!(
        output.witness_prompt.to_lowercase().contains("transcendent") ||
        output.witness_prompt.to_lowercase().contains("beyond") ||
        output.witness_prompt.to_lowercase().contains("siddhi"),
        "Siddhi-level prompt should reference transcendence: {}",
        output.witness_prompt
    );
    assert_eq!(output.consciousness_level, 5);
}

// ============================================================================
// 4. Transformation Pathways (2 tests)
// ============================================================================

#[tokio::test]
async fn test_transformation_pathway_shadow_to_gift() {
    let chart = create_test_chart(17, 18, 45, 26);
    let assessments = assess_frequencies(&chart, Some(2)); // Shadow level
    let pathways = generate_transformation_pathways(&assessments);

    assert!(!pathways.is_empty(), "Should generate pathways");
    for pathway in &pathways {
        assert_eq!(pathway.current_frequency, Frequency::Shadow);
        assert_eq!(pathway.next_frequency, Frequency::Gift);
        assert!(pathway.core_inquiry.contains('?'), "Core inquiry should be a question");
        assert!(!pathway.contemplations.is_empty(), "Should have contemplations");
        assert!(!pathway.witnessing_practices.is_empty(), "Should have witnessing practices");
        assert!(pathway.shadow_to_gift_inquiry.is_some(), "Should have shadow->gift inquiry");

        // Verify non-prescriptive language
        for practice in &pathway.witnessing_practices {
            assert!(
                practice.contains("might") || practice.contains("could"),
                "Witnessing practices should be invitational: {}",
                practice
            );
        }
    }
}

#[tokio::test]
async fn test_transformation_pathway_complete_journey() {
    let chart = create_test_chart(1, 2, 13, 7);
    let assessments = assess_frequencies(&chart, None);
    let pathways = generate_complete_pathways(&assessments);

    // Each Gene Key should have both Shadow->Gift and Gift->Siddhi
    let shadow_count = pathways.iter()
        .filter(|p| p.current_frequency == Frequency::Shadow)
        .count();
    let gift_count = pathways.iter()
        .filter(|p| p.current_frequency == Frequency::Gift)
        .count();

    assert!(shadow_count > 0, "Should have Shadow->Gift pathways");
    assert!(gift_count > 0, "Should have Gift->Siddhi pathways");
    assert_eq!(shadow_count, gift_count, "Should have equal number of both transition types");
}

// ============================================================================
// 5. HD Integration Mode (2 tests)
// ============================================================================

#[tokio::test]
async fn test_hd_integration_gates_mode() {
    // Mode 2: Direct gates (no HD engine needed)
    let engine = GeneKeysEngine::new();
    let input = create_engine_input(41, 31, 53, 42);
    let output = engine.calculate(input).await.expect("Gates mode should succeed");

    assert_eq!(output.engine_id, "gene-keys");
    assert_eq!(output.metadata.backend, "hd-gates");
    assert!(!output.witness_prompt.is_empty());

    // Verify the output passes validation
    let validation = engine.validate(&output).await.expect("Validation should succeed");
    assert!(validation.valid, "Output should be valid: {:?}", validation.messages);
}

#[tokio::test]
async fn test_hd_integration_missing_hd_engine() {
    // Mode 1: birth_data requires HD engine - should fail without one
    let engine = GeneKeysEngine::new(); // No HD engine attached

    let input = EngineInput {
        birth_data: Some(noesis_core::BirthData {
            name: Some("Test".to_string()),
            date: "1990-06-15".to_string(),
            time: Some("14:30".to_string()),
            latitude: 40.7128,
            longitude: -74.0060,
            timezone: "America/New_York".to_string(),
        }),
        current_time: Utc::now(),
        location: None,
        precision: noesis_core::Precision::Standard,
        options: HashMap::new(),
    };

    let result = engine.calculate(input).await;
    assert!(result.is_err(), "Should fail without HD engine for birth_data mode");
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("HD engine not available"),
        "Error should mention HD engine: {}",
        err_msg
    );
}
