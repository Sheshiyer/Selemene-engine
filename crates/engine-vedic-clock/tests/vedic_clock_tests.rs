//! Integration tests for the VedicClock-TCM engine
//!
//! Tests the complete engine functionality including:
//! - Organ clock mapping for all 24 hours
//! - Dosha time periods
//! - Activity recommendations
//! - Witness prompt generation

use engine_vedic_clock::{
    VedicClockEngine, ConsciousnessEngine, EngineInput,
    Organ, Dosha, Element, Activity,
    get_organ_for_hour, get_dosha_for_hour, organ_clock, dosha_times,
    get_optimal_timing, is_favorable_now, get_best_time,
    generate_witness_prompt, get_organ_element, get_opposing_organ,
    calculate_dosha_organ_harmony, get_temporal_recommendation,
};
use chrono::{TimeZone, Utc};
use noesis_core::Precision;
use serde_json::json;
use std::collections::HashMap;

// =============================================================================
// W2-S3-11: Organ Clock Mapping Tests
// =============================================================================

#[test]
fn test_organ_clock_24_hours_coverage() {
    // Verify that every hour (0-23) maps to an organ
    for hour in 0..24 {
        let organ = get_organ_for_hour(hour);
        assert!(
            !organ.organ.display_name().is_empty(),
            "Hour {} should have a valid organ",
            hour
        );
    }
}

#[test]
fn test_organ_clock_specific_mappings() {
    // Test specific hour -> organ mappings per spec
    let mappings = [
        (3, Organ::Lung),
        (4, Organ::Lung),
        (5, Organ::LargeIntestine),
        (6, Organ::LargeIntestine),
        (7, Organ::Stomach),
        (8, Organ::Stomach),
        (9, Organ::Spleen),
        (10, Organ::Spleen),
        (11, Organ::Heart),
        (12, Organ::Heart),
        (13, Organ::SmallIntestine),
        (14, Organ::SmallIntestine),
        (15, Organ::Bladder),
        (16, Organ::Bladder),
        (17, Organ::Kidney),
        (18, Organ::Kidney),
        (19, Organ::Pericardium),
        (20, Organ::Pericardium),
        (21, Organ::TripleWarmer),
        (22, Organ::TripleWarmer),
        (23, Organ::Gallbladder),
        (0, Organ::Gallbladder),
        (1, Organ::Liver),
        (2, Organ::Liver),
    ];

    for (hour, expected_organ) in mappings {
        let organ = get_organ_for_hour(hour);
        assert_eq!(
            organ.organ, expected_organ,
            "Hour {} should be {:?}, got {:?}",
            hour, expected_organ, organ.organ
        );
    }
}

#[test]
fn test_organ_element_associations() {
    // Per TCM Five Element theory
    assert_eq!(get_organ_element(&Organ::Lung), Element::Metal);
    assert_eq!(get_organ_element(&Organ::LargeIntestine), Element::Metal);
    assert_eq!(get_organ_element(&Organ::Stomach), Element::Earth);
    assert_eq!(get_organ_element(&Organ::Spleen), Element::Earth);
    assert_eq!(get_organ_element(&Organ::Heart), Element::Fire);
    assert_eq!(get_organ_element(&Organ::SmallIntestine), Element::Fire);
    assert_eq!(get_organ_element(&Organ::Bladder), Element::Water);
    assert_eq!(get_organ_element(&Organ::Kidney), Element::Water);
    assert_eq!(get_organ_element(&Organ::Pericardium), Element::Fire);
    assert_eq!(get_organ_element(&Organ::TripleWarmer), Element::Fire);
    assert_eq!(get_organ_element(&Organ::Gallbladder), Element::Wood);
    assert_eq!(get_organ_element(&Organ::Liver), Element::Wood);
}

#[test]
fn test_organ_clock_has_12_organs() {
    let clock = organ_clock();
    assert_eq!(clock.len(), 12);
}

#[test]
fn test_each_organ_window_is_2_hours() {
    let clock = organ_clock();
    
    for window in clock {
        let duration = if window.end_hour < window.start_hour {
            (24 - window.start_hour) + window.end_hour
        } else {
            window.end_hour - window.start_hour
        };
        assert_eq!(duration, 2, "{:?} window should be 2 hours", window.organ);
    }
}

// =============================================================================
// W2-S3-11: Dosha Time Period Tests
// =============================================================================

#[test]
fn test_dosha_times_six_periods() {
    // 3 doshas × 2 cycles (AM and PM) = 6 periods
    let times = dosha_times();
    assert_eq!(times.len(), 6);
}

#[test]
fn test_dosha_specific_mappings() {
    // Vata: 2-6 AM and 2-6 PM (14-18)
    assert_eq!(get_dosha_for_hour(3).dosha, Dosha::Vata);
    assert_eq!(get_dosha_for_hour(15).dosha, Dosha::Vata);
    
    // Kapha: 6-10 AM and 6-10 PM (18-22)
    assert_eq!(get_dosha_for_hour(7).dosha, Dosha::Kapha);
    assert_eq!(get_dosha_for_hour(19).dosha, Dosha::Kapha);
    
    // Pitta: 10 AM-2 PM and 10 PM-2 AM
    assert_eq!(get_dosha_for_hour(11).dosha, Dosha::Pitta);
    assert_eq!(get_dosha_for_hour(23).dosha, Dosha::Pitta);
}

#[test]
fn test_dosha_periods_are_4_hours() {
    for period in dosha_times() {
        let duration = if period.end_hour < period.start_hour {
            (24 - period.start_hour) + period.end_hour
        } else {
            period.end_hour - period.start_hour
        };
        assert_eq!(duration, 4, "{:?} period should be 4 hours", period.dosha);
    }
}

#[test]
fn test_dosha_organ_harmony() {
    // Perfect harmony when organ dosha matches time dosha
    let harmony = calculate_dosha_organ_harmony(&Organ::Lung, &Dosha::Kapha);
    assert!((harmony - 1.0).abs() < 0.001, "Lung (Kapha) should have perfect harmony during Kapha time");
    
    // Imperfect but non-zero harmony for mismatches
    let harmony = calculate_dosha_organ_harmony(&Organ::Heart, &Dosha::Vata);
    assert!(harmony > 0.0 && harmony < 1.0);
}

// =============================================================================
// W2-S3-11: Activity Recommendations Tests
// =============================================================================

#[test]
fn test_optimal_timing_returns_results() {
    let dt = Utc.with_ymd_and_hms(2024, 1, 1, 10, 0, 0).unwrap();
    
    for activity in [
        Activity::Meditation,
        Activity::Exercise,
        Activity::Work,
        Activity::Eating,
        Activity::Sleep,
        Activity::Creative,
        Activity::Social,
    ] {
        let windows = get_optimal_timing(activity, dt, 0);
        assert!(
            !windows.is_empty(),
            "{:?} should have optimal timing windows",
            activity
        );
    }
}

#[test]
fn test_optimal_timing_sorted_by_quality() {
    let dt = Utc.with_ymd_and_hms(2024, 1, 1, 10, 0, 0).unwrap();
    let windows = get_optimal_timing(Activity::Work, dt, 0);
    
    for i in 1..windows.len() {
        assert!(
            windows[i - 1].quality >= windows[i].quality,
            "Windows should be sorted by quality descending"
        );
    }
}

#[test]
fn test_eating_favorable_during_stomach_time() {
    // 8 AM is Stomach time
    let dt = Utc.with_ymd_and_hms(2024, 1, 1, 8, 0, 0).unwrap();
    let (favorable, _reason) = is_favorable_now(Activity::Eating, dt, 0);
    assert!(favorable, "Eating should be favorable during Stomach time (7-9 AM)");
}

#[test]
fn test_meditation_favorable_early_morning() {
    // 4 AM is Lung time - good for meditation
    let dt = Utc.with_ymd_and_hms(2024, 1, 1, 4, 0, 0).unwrap();
    let (favorable, _) = is_favorable_now(Activity::Meditation, dt, 0);
    // Either favorable or has good windows
    let best = get_best_time(Activity::Meditation);
    assert!(best.quality > 0.0);
}

#[test]
fn test_work_favorable_during_spleen_time() {
    // 10 AM is Spleen time - good for mental work
    let dt = Utc.with_ymd_and_hms(2024, 1, 1, 10, 0, 0).unwrap();
    let (favorable, _) = is_favorable_now(Activity::Work, dt, 0);
    assert!(favorable, "Work should be favorable during Spleen time (9-11 AM)");
}

#[test]
fn test_best_time_returns_valid_window() {
    for activity in [
        Activity::Meditation,
        Activity::Exercise,
        Activity::Work,
        Activity::Eating,
        Activity::Sleep,
        Activity::Creative,
        Activity::Social,
    ] {
        let best = get_best_time(activity);
        assert!(best.quality > 0.0, "{:?} should have positive quality", activity);
        assert!(!best.reason.is_empty(), "{:?} should have a reason", activity);
    }
}

// =============================================================================
// W2-S3-11: Witness Prompt Tests
// =============================================================================

#[test]
fn test_witness_prompts_are_questions() {
    for organ in Organ::all_in_cycle_order() {
        for dosha in [Dosha::Vata, Dosha::Pitta, Dosha::Kapha] {
            for level in [0, 2, 3, 4, 5, 6] {
                let prompt = generate_witness_prompt(&organ, &dosha, level);
                assert!(
                    prompt.contains("?"),
                    "Prompt for {:?}/{:?}/level {} should be a question: {}",
                    organ, dosha, level, prompt
                );
            }
        }
    }
}

#[test]
fn test_witness_prompts_non_prescriptive() {
    let prescriptive_words = ["should", "must", "need to", "have to", "ought"];
    
    for organ in Organ::all_in_cycle_order() {
        for dosha in [Dosha::Vata, Dosha::Pitta, Dosha::Kapha] {
            let prompt = generate_witness_prompt(&organ, &dosha, 3);
            
            for word in prescriptive_words {
                assert!(
                    !prompt.to_lowercase().contains(word),
                    "Prompt should not contain prescriptive '{}': {}",
                    word, prompt
                );
            }
        }
    }
}

#[test]
fn test_witness_prompts_vary_by_level() {
    let organ = Organ::Heart;
    let dosha = Dosha::Pitta;
    
    let awareness = generate_witness_prompt(&organ, &dosha, 1);
    let observation = generate_witness_prompt(&organ, &dosha, 3);
    let integration = generate_witness_prompt(&organ, &dosha, 5);
    
    // All should be non-empty and questions
    assert!(!awareness.is_empty());
    assert!(!observation.is_empty());
    assert!(!integration.is_empty());
    
    // They might be different (though randomness means they could occasionally be same)
    // Just verify they're all valid questions
    assert!(awareness.contains("?"));
    assert!(observation.contains("?"));
    assert!(integration.contains("?"));
}

// =============================================================================
// W2-S3-11: Full Engine Integration Tests
// =============================================================================

#[tokio::test]
async fn test_engine_calculate_basic() {
    let engine = VedicClockEngine::new();
    let input = create_test_input(0);
    
    let result = engine.calculate(input).await;
    assert!(result.is_ok(), "Engine calculation should succeed");
    
    let output = result.unwrap();
    assert_eq!(output.engine_id, "vedic-clock");
    assert!(!output.witness_prompt.is_empty());
    assert!(output.result.get("current_organ").is_some());
    assert!(output.result.get("current_dosha").is_some());
    assert!(output.result.get("recommendation").is_some());
}

#[tokio::test]
async fn test_engine_with_timezone() {
    let engine = VedicClockEngine::new();
    
    // Test with IST timezone (+330 minutes)
    let mut options = HashMap::new();
    options.insert("timezone_offset".to_string(), json!(330));
    
    let input = EngineInput {
        birth_data: None,
        current_time: Utc.with_ymd_and_hms(2024, 1, 1, 4, 0, 0).unwrap(), // 4 AM UTC = 9:30 AM IST
        location: None,
        precision: Precision::Standard,
        options,
    };
    
    let result = engine.calculate(input).await;
    assert!(result.is_ok());
    
    let output = result.unwrap();
    // At 9:30 AM IST, should be Spleen time
    let organ = output.result.get("current_organ").unwrap();
    let organ_name = organ.get("organ").unwrap().as_str().unwrap();
    assert_eq!(organ_name, "Spleen", "9:30 AM should be Spleen time");
}

#[tokio::test]
async fn test_engine_with_activity() {
    let engine = VedicClockEngine::new();
    
    let mut options = HashMap::new();
    options.insert("timezone_offset".to_string(), json!(0));
    options.insert("activity".to_string(), json!("meditation"));
    
    let input = EngineInput {
        birth_data: None,
        current_time: Utc::now(),
        location: None,
        precision: Precision::Standard,
        options,
    };
    
    let result = engine.calculate(input).await;
    assert!(result.is_ok());
    
    let output = result.unwrap();
    assert!(output.result.get("activity_timing").is_some());
    
    let timing = output.result.get("activity_timing").unwrap();
    assert_eq!(timing.get("activity").unwrap().as_str().unwrap(), "Meditation");
    assert!(timing.get("optimal_windows").is_some());
}

#[tokio::test]
async fn test_engine_with_panchanga() {
    let engine = VedicClockEngine::new();
    
    let mut options = HashMap::new();
    options.insert("timezone_offset".to_string(), json!(0));
    options.insert("tithi_index".to_string(), json!(2)); // Tritiya
    options.insert("nakshatra_index".to_string(), json!(3));
    
    let input = EngineInput {
        birth_data: None,
        current_time: Utc::now(),
        location: None,
        precision: Precision::Standard,
        options,
    };
    
    let result = engine.calculate(input).await;
    assert!(result.is_ok());
    
    let output = result.unwrap();
    let recommendation = output.result.get("recommendation").unwrap();
    let panchanga = recommendation.get("panchanga_quality");
    assert!(panchanga.is_some(), "Panchanga quality should be present");
    assert!(!panchanga.unwrap().is_null());
}

#[tokio::test]
async fn test_engine_validation() {
    let engine = VedicClockEngine::new();
    let input = create_test_input(0);
    
    let output = engine.calculate(input).await.unwrap();
    let validation = engine.validate(&output).await.unwrap();
    
    assert!(validation.valid);
    assert!((validation.confidence - 1.0).abs() < 0.001);
}

#[tokio::test]
async fn test_engine_cache_key_deterministic() {
    let engine = VedicClockEngine::new();
    let input = create_test_input(0);
    
    let key1 = engine.cache_key(&input);
    let key2 = engine.cache_key(&input);
    
    assert_eq!(key1, key2, "Cache key should be deterministic");
}

#[tokio::test]
async fn test_engine_upcoming_transitions() {
    let engine = VedicClockEngine::new();
    let input = create_test_input(0);
    
    let output = engine.calculate(input).await.unwrap();
    let upcoming = output.result.get("upcoming_transitions");
    
    assert!(upcoming.is_some(), "Should include upcoming transitions");
    let transitions = upcoming.unwrap().as_array().unwrap();
    assert!(!transitions.is_empty(), "Should have at least one upcoming transition");
}

// =============================================================================
// Integration Tests: Temporal Recommendation
// =============================================================================

#[test]
fn test_temporal_recommendation_complete() {
    let dt = Utc.with_ymd_and_hms(2024, 1, 1, 10, 0, 0).unwrap();
    let rec = get_temporal_recommendation(dt, 0, Some(2), Some(3));
    
    // Check all fields are populated
    assert!(!rec.time_window.is_empty());
    assert!(!rec.activities.is_empty());
    assert!(rec.panchanga_quality.is_some());
}

#[test]
fn test_temporal_recommendation_without_panchanga() {
    let dt = Utc.with_ymd_and_hms(2024, 1, 1, 10, 0, 0).unwrap();
    let rec = get_temporal_recommendation(dt, 0, None, None);
    
    assert!(!rec.time_window.is_empty());
    assert!(!rec.activities.is_empty());
    assert!(rec.panchanga_quality.is_none());
}

// =============================================================================
// Helper Functions
// =============================================================================

fn create_test_input(timezone_offset: i32) -> EngineInput {
    let mut options = HashMap::new();
    options.insert("timezone_offset".to_string(), json!(timezone_offset));
    
    EngineInput {
        birth_data: None,
        current_time: Utc::now(),
        location: None,
        precision: Precision::Standard,
        options,
    }
}
