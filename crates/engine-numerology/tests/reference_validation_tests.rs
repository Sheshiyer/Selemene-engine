//! MVP-19: Numerology Reference Validation Tests
//!
//! Verifies life path, expression, soul urge, and personality calculations
//! against known reference data. Uses well-known birthdates where numerology
//! values are established in the literature.
//!
//! Life Path calculation method:
//!   1. Reduce year to single/master digit
//!   2. Reduce month to single/master digit
//!   3. Reduce day to single/master digit
//!   4. Sum the three reduced values, reduce again to single/master
//!
//! Master numbers (11, 22, 33) are preserved during reduction.

use engine_numerology::{ConsciousnessEngine, EngineInput, NumerologyEngine};
use chrono::Utc;
use noesis_core::Precision;
use serde_json::Value;
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Test Helpers
// ---------------------------------------------------------------------------

fn make_input(name: &str, date: &str) -> EngineInput {
    EngineInput {
        birth_data: Some(noesis_core::BirthData {
            name: Some(name.to_string()),
            date: date.to_string(),
            time: None,
            latitude: 0.0,
            longitude: 0.0,
            timezone: "UTC".into(),
        }),
        current_time: Utc::now(),
        location: None,
        precision: Precision::Standard,
        options: HashMap::new(),
    }
}

async fn compute_life_path(name: &str, date: &str) -> u32 {
    let engine = NumerologyEngine::new();
    let output = engine.calculate(make_input(name, date)).await
        .unwrap_or_else(|e| panic!("Calculation failed for {} ({}): {:?}", name, date, e));
    output.result["life_path"]["value"].as_u64()
        .unwrap_or_else(|| panic!("Missing life_path.value for {}", name)) as u32
}

async fn compute_all(name: &str, date: &str) -> Value {
    let engine = NumerologyEngine::new();
    let output = engine.calculate(make_input(name, date)).await
        .unwrap_or_else(|e| panic!("Calculation failed for {} ({}): {:?}", name, date, e));
    output.result
}

// ---------------------------------------------------------------------------
// Reference Data
//
// Life Path is calculated: reduce(year) + reduce(month) + reduce(day) -> reduce(sum)
//
// Example: 1990-01-15
//   Year: 1+9+9+0 = 19 -> 1+9 = 10 -> 1+0 = 1
//   Month: 01 -> 1
//   Day: 15 -> 1+5 = 6
//   Sum: 1+1+6 = 8
// ---------------------------------------------------------------------------

struct ReferenceChart {
    name: &'static str,
    date: &'static str,
    expected_life_path: u32,
    description: &'static str,
}

fn reference_charts() -> Vec<ReferenceChart> {
    vec![
        ReferenceChart {
            name: "Test Person A",
            date: "1990-01-15",
            expected_life_path: 8,
            // Year: 1+9+9+0=19->1+0=1, Month: 1, Day: 1+5=6, Sum: 1+1+6=8
            description: "1990-01-15: year=1, month=1, day=6, sum=8",
        },
        ReferenceChart {
            name: "Test Person B",
            date: "1985-11-22",
            expected_life_path: 11,
            // Year: 1+9+8+5=23->5, Month: 11 (master, keep), Day: 22 (master, keep)
            // But LP reduces each component: year=5, month=11->2, day=22->4
            // Actually: year 1+9+8+5=23->5, month 1+1=2, day 2+2=4, sum=5+2+4=11 (master!)
            description: "1985-11-22: year=5, month=2, day=4, sum=11 (master)",
        },
        ReferenceChart {
            name: "Test Person C",
            date: "2000-03-07",
            expected_life_path: 3,
            // Year: 2+0+0+0=2, Month: 3, Day: 7, Sum: 2+3+7=12->3
            description: "2000-03-07: year=2, month=3, day=7, sum=12->3",
        },
        ReferenceChart {
            name: "Test Person D",
            date: "1978-06-29",
            expected_life_path: 6,
            // Year: 1+9+7+8=25->7, Month: 6, Day: 2+9=11 (master->2 for LP sum)
            // year=7, month=6, day=11->2, sum=7+6+2=15->6
            description: "1978-06-29: year=7, month=6, day=2, sum=15->6",
        },
        ReferenceChart {
            name: "Test Person E",
            date: "1991-09-14",
            expected_life_path: 7,
            // Year: 1+9+9+1=20->2, Month: 9, Day: 1+4=5, Sum: 2+9+5=16->7
            description: "1991-09-14 (Shesh): year=2, month=9, day=5, sum=16->7",
        },
        ReferenceChart {
            name: "Test Person F",
            date: "1995-12-25",
            expected_life_path: 7,
            // Year: 1+9+9+5=24->6, Month: 1+2=3, Day: 2+5=7, Sum: 6+3+7=16->7
            description: "1995-12-25: year=6, month=3, day=7, sum=16->7",
        },
        ReferenceChart {
            name: "Test Person G",
            date: "1970-10-05",
            expected_life_path: 5,
            // Year: 1+9+7+0=17->8, Month: 1+0=1, Day: 5, Sum: 8+1+5=14->5
            description: "1970-10-05: year=8, month=1, day=5, sum=14->5",
        },
        ReferenceChart {
            name: "Test Person H",
            date: "2001-01-01",
            expected_life_path: 5,
            // Year: 2+0+0+1=3, Month: 1, Day: 1, Sum: 3+1+1=5
            description: "2001-01-01: year=3, month=1, day=1, sum=5",
        },
        ReferenceChart {
            name: "Test Person I",
            date: "1999-09-09",
            // year: reduce_to_core(1999) -> 28 -> 10 -> 1
            // month: reduce_to_core(9) -> 9
            // day: reduce_to_core(9) -> 9
            // raw_sum = 1+9+9 = 19
            // reduce_to_core(19) -> 10 -> 1
            expected_life_path: 1,
            description: "1999-09-09: year=1, month=9, day=9, sum=19->10->1",
        },
        ReferenceChart {
            name: "Test Person J Master 22",
            date: "1980-04-04",
            expected_life_path: 8,
            // Year: 1+9+8+0=18->9, Month: 4, Day: 4, Sum: 9+4+4=17->8
            description: "1980-04-04: year=9, month=4, day=4, sum=17->8",
        },
    ]
}

// ---------------------------------------------------------------------------
// Life Path Validation Tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_life_path_all_reference_charts() {
    let charts = reference_charts();
    let mut pass_count = 0;
    let mut fail_count = 0;

    for chart in &charts {
        let actual = compute_life_path(chart.name, chart.date).await;
        if actual == chart.expected_life_path {
            pass_count += 1;
        } else {
            fail_count += 1;
            eprintln!(
                "  [FAIL] {}: expected LP={}, got LP={} ({})",
                chart.name, chart.expected_life_path, actual, chart.description
            );
        }
    }

    assert_eq!(
        fail_count, 0,
        "{} of {} life path calculations failed",
        fail_count,
        charts.len()
    );
    eprintln!("  [PASS] All {} life path calculations correct", pass_count);
}

// ---------------------------------------------------------------------------
// Expression Number (Name-based) Tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_expression_number_john_doe() {
    let result = compute_all("John Doe", "1990-01-15").await;
    let expr = result["expression"]["value"].as_u64().unwrap() as u32;

    // "John Doe": J(1)+O(6)+H(8)+N(5)+D(4)+O(6)+E(5) = 35 -> 8
    assert_eq!(
        expr, 8,
        "John Doe expression should be 8, got {}",
        expr
    );
}

#[tokio::test]
async fn test_expression_number_alice_smith() {
    let result = compute_all("Alice Smith", "2000-01-01").await;
    let expr = result["expression"]["value"].as_u64().unwrap() as u32;

    // "Alice Smith": A(1)+L(3)+I(9)+C(3)+E(5)+S(1)+M(4)+I(9)+T(2)+H(8) = 45 -> 9
    assert_eq!(
        expr, 9,
        "Alice Smith expression should be 9, got {}",
        expr
    );
}

// ---------------------------------------------------------------------------
// Soul Urge (Vowels) Tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_soul_urge_john_doe() {
    let result = compute_all("John Doe", "1990-01-15").await;
    let su = result["soul_urge"]["value"].as_u64().unwrap() as u32;

    // "John Doe" vowels: O(6) + O(6) + E(5) = 17 -> 8
    assert_eq!(
        su, 8,
        "John Doe soul urge should be 8, got {}",
        su
    );
}

// ---------------------------------------------------------------------------
// Personality (Consonants) Tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_personality_john_doe() {
    let result = compute_all("John Doe", "1990-01-15").await;
    let pers = result["personality"]["value"].as_u64().unwrap() as u32;

    // "John Doe" consonants: J(1)+H(8)+N(5)+D(4) = 18 -> 9
    assert_eq!(
        pers, 9,
        "John Doe personality should be 9, got {}",
        pers
    );
}

// ---------------------------------------------------------------------------
// Master Number Preservation Tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_master_number_11_life_path() {
    // 1985-11-22: year=5, month=2, day=4, sum=11 (master preserved)
    let lp = compute_life_path("Master Test", "1985-11-22").await;
    assert_eq!(
        lp, 11,
        "1985-11-22 should produce master number 11 life path, got {}",
        lp
    );
}

#[tokio::test]
async fn test_non_master_numbers_reduce_correctly() {
    // Verify numbers that look like masters but aren't in the right context still reduce
    // 2000-03-07: year=2, month=3, day=7, sum=12->3
    let lp = compute_life_path("Reduce Test", "2000-03-07").await;
    assert_eq!(lp, 3, "2000-03-07 should reduce to LP 3, got {}", lp);
}

// ---------------------------------------------------------------------------
// Engine Validation Roundtrip
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_engine_validation_passes_for_all_references() {
    let engine = NumerologyEngine::new();

    for chart in &reference_charts() {
        let input = make_input(chart.name, chart.date);
        let output = engine.calculate(input).await
            .unwrap_or_else(|e| panic!("Calculation failed for {}: {:?}", chart.name, e));

        let validation = engine.validate(&output).await
            .unwrap_or_else(|e| panic!("Validation call failed for {}: {:?}", chart.name, e));

        assert!(
            validation.valid,
            "[{}] Validation failed: {:?}",
            chart.name, validation.messages
        );
        assert_eq!(
            validation.confidence, 1.0,
            "[{}] Expected confidence 1.0, got {}",
            chart.name, validation.confidence
        );
    }
}

// ---------------------------------------------------------------------------
// Output Structure Validation
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_output_has_all_required_fields() {
    let engine = NumerologyEngine::new();
    let output = engine
        .calculate(make_input("Test User", "1990-06-15"))
        .await
        .unwrap();

    assert_eq!(output.engine_id, "numerology");
    assert!(!output.witness_prompt.is_empty(), "Witness prompt must be non-empty");
    assert_eq!(output.consciousness_level, 0);
    assert_eq!(output.metadata.backend, "native-rust");

    // Verify all 6 core numbers are present
    let result = &output.result;
    for field in &["life_path", "expression", "soul_urge", "personality", "birthday", "chaldean_name"] {
        assert!(
            result.get(field).is_some(),
            "Missing field '{}' in output",
            field
        );
        let num = &result[field];
        assert!(num["value"].is_number(), "'{}' should have numeric value", field);
        assert!(num["is_master"].is_boolean(), "'{}' should have is_master flag", field);
        assert!(num["reduction_chain"].is_array(), "'{}' should have reduction_chain", field);
        assert!(num["meaning"].is_string(), "'{}' should have meaning string", field);
    }
}

// ---------------------------------------------------------------------------
// Birthday Number Tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_birthday_number_day_15() {
    let result = compute_all("Test", "1990-01-15").await;
    let birthday = result["birthday"]["value"].as_u64().unwrap() as u32;
    // Day 15: 1+5 = 6
    assert_eq!(birthday, 6, "Birthday number for day 15 should be 6, got {}", birthday);
}

#[tokio::test]
async fn test_birthday_number_day_22_master() {
    let result = compute_all("Test", "1990-01-22").await;
    let birthday = result["birthday"]["value"].as_u64().unwrap() as u32;
    // Day 22 is a master number
    assert_eq!(birthday, 22, "Birthday number for day 22 should be master 22, got {}", birthday);

    let is_master = result["birthday"]["is_master"].as_bool().unwrap();
    assert!(is_master, "Day 22 birthday should be flagged as master number");
}

// ---------------------------------------------------------------------------
// Chaldean Name Number Tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_chaldean_name_john() {
    let result = compute_all("John", "1990-01-01").await;
    let chaldean = result["chaldean_name"]["value"].as_u64().unwrap() as u32;
    // "John" Chaldean: J(1)+O(7)+H(5)+N(5) = 18 -> 9
    assert_eq!(chaldean, 9, "John Chaldean name should be 9, got {}", chaldean);
}
