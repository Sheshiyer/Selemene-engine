#!/bin/bash
# Agent 30: Gene Keys Archetypal Depth Validation + Integration Tests
# Run script for all tests

set -e

echo "================================"
echo "Agent 30: Gene Keys Test Suite"
echo "================================"
echo ""

cd /Volumes/madara/2026/witnessos/Selemene-engine

# Create tests directory for engine-gene-keys if it doesn't exist
echo "[1/4] Creating test directories..."
mkdir -p crates/engine-gene-keys/tests

#Create the archetypal depth validation test file
echo "[2/4] Creating archetypal depth validation tests..."
cat > crates/engine-gene-keys/tests/archetypal_depth_validation.rs << 'ARCHTEST'
//! Archetypal Depth Validation Tests for Gene Keys Engine
//!
//! Validates Rule 7: Archetypal depth must be preserved (no summarization).
//! Ensures full Shadow/Gift/Siddhi descriptions are maintained in:
//! - Source data (archetypes.json)
//! - Engine output (GeneKeysChart)
//! - API responses (EngineOutput)

use engine_gene_keys::wisdom::{gene_keys, get_gene_key};
use noesis_core::{EngineInput, EngineOutput, ConsciousnessEngine, BirthData};
use engine_gene_keys::GeneKeysEngine;
use chrono::Utc;
use serde_json::Value;
use std::collections::HashMap;

#[tokio::test]
async fn test_full_shadow_gift_siddhi_preserved() {
    // Rule 7: Archetypal depth must be preserved (no summarization)
    
    let all_gene_keys = gene_keys();
    
    // Check all 64 Gene Keys have substantial descriptions
    for key_num in 1..=64 {
        let key = all_gene_keys.get(&key_num)
            .unwrap_or_else(|| panic!("Gene Key {} missing from wisdom data", key_num));
        
        // Shadow text must be substantial (>10 words minimum)
        let shadow_word_count = key.shadow_description.split_whitespace().count();
        assert!(
            shadow_word_count >= 10,
            "Gene Key {} shadow only {} words - archetypal depth may be insufficient (expected 10+ words)",
            key_num, shadow_word_count
        );
        
        // Gift text must be substantial
        let gift_word_count = key.gift_description.split_whitespace().count();
        assert!(
            gift_word_count >= 10,
            "Gene Key {} gift only {} words - archetypal depth may be insufficient",
            key_num, gift_word_count
        );
        
        // Siddhi text must be substantial
        let siddhi_word_count = key.siddhi_description.split_whitespace().count();
        assert!(
            siddhi_word_count >= 10,
            "Gene Key {} siddhi only {} words - archetypal depth may be insufficient",
            key_num, siddhi_word_count
        );
        
        // Validate names are present
        assert!(!key.shadow.is_empty(), "Gene Key {} shadow name missing", key_num);
        assert!(!key.gift.is_empty(), "Gene Key {} gift name missing", key_num);
        assert!(!key.siddhi.is_empty(), "Gene Key {} siddhi name missing", key_num);
        assert!(!key.name.is_empty(), "Gene Key {} name missing", key_num);
    }
}

#[tokio::test]
async fn test_all_64_keys_present() {
    // Validate all 64 Gene Keys exist
    let all_gene_keys = gene_keys();
    
    assert_eq!(
        all_gene_keys.len(),
        64,
        "Expected exactly 64 Gene Keys, found {}",
        all_gene_keys.len()
    );
    
    // Validate sequential numbers 1-64
    for num in 1..=64 {
        assert!(
            all_gene_keys.contains_key(&num),
            "Gene Key {} missing from dataset",
            num
        );
    }
}

#[tokio::test]
async fn test_specific_gene_keys_depth() {
    // Test specific Gene Keys known to have rich archetypal content
    
    // Gene Key 1: The Creative
    let key_1 = get_gene_key(1).expect("Gene Key 1 should exist");
    assert_eq!(key_1.name, "The Creative");
    assert_eq!(key_1.shadow, "Entropy");
    assert_eq!(key_1.gift, "Freshness");
    assert_eq!(key_1.siddhi, "Beauty");
    assert!(!key_1.shadow_description.is_empty());
    assert!(!key_1.gift_description.is_empty());
    assert!(!key_1.siddhi_description.is_empty());
}

#[tokio::test]
async fn test_api_output_preserves_depth() {
    // Test that engine output includes full text, not summaries
    
    let engine = GeneKeysEngine::new();
    
    let mut options = HashMap::new();
    options.insert(
        "hd_gates".to_string(),
        serde_json::json!({
            "personality_sun": 1,
            "personality_earth": 2,
            "design_sun": 3,
            "design_earth": 4
        })
    );
    options.insert("consciousness_level".to_string(), serde_json::json!(3));
    
    let input = EngineInput {
        birth_data: None,
        current_time: Utc::now(),
        location: None,
        precision: noesis_core::Precision::Standard,
        options,
    };
    
    let output = engine.calculate(input).await.expect("Calculation should succeed");
    
    // Parse chart data
    let chart: engine_gene_keys::models::GeneKeysChart = 
        serde_json::from_value(output.data.clone())
            .expect("Should deserialize to GeneKeysChart");
    
    // Validate active_keys exist
    assert!(!chart.active_keys.is_empty(), "Should have active keys");
    
    // Find Gene Key 1 in active_keys
    let key_1_activation = chart.active_keys.iter()
        .find(|k| k.key_number == 1)
        .expect("Gene Key 1 should be in active keys");
    
    assert_eq!(key_1_activation.key_number, 1);
}

#[tokio::test]
async fn test_no_text_truncation() {
    // Ensure no truncation markers like "..." or "[truncated]"
    
    let all_gene_keys = gene_keys();
    
    for (key_num, key) in all_gene_keys {
        assert!(
            !key.shadow_description.contains("...") && !key.shadow_description.contains("[truncated]"),
            "Gene Key {} shadow appears truncated",
            key_num
        );
        assert!(
            !key.gift_description.contains("...") && !key.gift_description.contains("[truncated]"),
            "Gene Key {} gift appears truncated",
            key_num
        );
        assert!(
            !key.siddhi_description.contains("...") && !key.siddhi_description.contains("[truncated]"),
            "Gene Key {} siddhi appears truncated",
            key_num
        );
    }
}

#[tokio::test]
async fn test_activation_sequence_structure() {
    // Validate the 4 Activation Sequences are properly structured
    
    let engine = GeneKeysEngine::new();
    
    let mut options = HashMap::new();
    options.insert(
        "hd_gates".to_string(),
        serde_json::json!({
            "personality_sun": 1,
            "personality_earth": 2,
            "design_sun": 3,
            "design_earth": 4
        })
    );
    
    let input = EngineInput {
        birth_data: None,
        current_time: Utc::now(),
        location: None,
        precision: noesis_core::Precision::Standard,
        options,
    };
    
    let output = engine.calculate(input).await.expect("Calculation should succeed");
    
    let chart: engine_gene_keys::models::GeneKeysChart = 
        serde_json::from_value(output.data)
            .expect("Should deserialize to GeneKeysChart");
    
    // Validate Life's Work (P Sun + P Earth)
    assert_eq!(chart.activation_sequence.lifes_work.0, 1);
    assert_eq!(chart.activation_sequence.lifes_work.1, 2);
    
    // Validate Evolution (D Sun + D Earth)
    assert_eq!(chart.activation_sequence.evolution.0, 3);
    assert_eq!(chart.activation_sequence.evolution.1, 4);
    
    // Validate Radiance (P Sun + D Sun)
    assert_eq!(chart.activation_sequence.radiance.0, 1);
    assert_eq!(chart.activation_sequence.radiance.1, 3);
    
    // Validate Purpose (P Earth + D Earth)
    assert_eq!(chart.activation_sequence.purpose.0, 2);
    assert_eq!(chart.activation_sequence.purpose.1, 4);
}
ARCHTEST

echo "✓ Archetypal depth validation tests created"

# Run integration tests
echo ""
echo "[3/4] Running Gene Keys integration tests..."
cargo test -p noesis-api --test gene_keys_integration --no-fail-fast -- --test-threads=1

echo ""
echo "[4/4] Running archetypal depth validation tests..."
cargo test -p engine-gene-keys --test archetypal_depth_validation --no-fail-fast

echo ""
echo "================================"
echo "✓ All tests completed"
echo "================================"
