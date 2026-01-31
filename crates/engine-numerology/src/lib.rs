//! Numerology Consciousness Engine
//!
//! Implements Pythagorean and Chaldean numerology systems.
//! Pure math -- no external dependencies beyond noesis-core.

pub use noesis_core::{ConsciousnessEngine, EngineError, EngineInput, EngineOutput};

use async_trait::async_trait;
use chrono::Utc;
use noesis_core::{CalculationMetadata, ValidationResult};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::time::Instant;

// ---------------------------------------------------------------------------
// Pythagorean letter-to-number mapping (A=1 .. I=9, J=1 .. R=9, S=1 .. Z=8)
// ---------------------------------------------------------------------------
fn pythagorean_value(ch: char) -> Option<u32> {
    match ch.to_ascii_uppercase() {
        'A' => Some(1),
        'B' => Some(2),
        'C' => Some(3),
        'D' => Some(4),
        'E' => Some(5),
        'F' => Some(6),
        'G' => Some(7),
        'H' => Some(8),
        'I' => Some(9),
        'J' => Some(1),
        'K' => Some(2),
        'L' => Some(3),
        'M' => Some(4),
        'N' => Some(5),
        'O' => Some(6),
        'P' => Some(7),
        'Q' => Some(8),
        'R' => Some(9),
        'S' => Some(1),
        'T' => Some(2),
        'U' => Some(3),
        'V' => Some(4),
        'W' => Some(5),
        'X' => Some(6),
        'Y' => Some(7),
        'Z' => Some(8),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Chaldean letter-to-number mapping
// ---------------------------------------------------------------------------
fn chaldean_value(ch: char) -> Option<u32> {
    match ch.to_ascii_uppercase() {
        'A' => Some(1),
        'B' => Some(2),
        'C' => Some(3),
        'D' => Some(4),
        'E' => Some(5),
        'F' => Some(8),
        'G' => Some(3),
        'H' => Some(5),
        'I' => Some(1),
        'J' => Some(1),
        'K' => Some(2),
        'L' => Some(3),
        'M' => Some(4),
        'N' => Some(5),
        'O' => Some(7),
        'P' => Some(8),
        'Q' => Some(1),
        'R' => Some(2),
        'S' => Some(3),
        'T' => Some(4),
        'U' => Some(6),
        'V' => Some(6),
        'W' => Some(6),
        'X' => Some(5),
        'Y' => Some(1),
        'Z' => Some(7),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Reduction helpers
// ---------------------------------------------------------------------------

/// Returns true if `n` is a master number that should not be reduced further.
fn is_master(n: u32) -> bool {
    matches!(n, 11 | 22 | 33)
}

/// Sum the individual digits of `n`.
fn digit_sum(n: u32) -> u32 {
    let mut total = 0u32;
    let mut v = n;
    while v > 0 {
        total += v % 10;
        v /= 10;
    }
    total
}

/// Reduce a number to a single digit or master number (11, 22, 33).
/// Returns the final value and the full reduction chain (including the input).
fn reduce_to_core(n: u32) -> (u32, Vec<u32>) {
    let mut chain = Vec::new();
    let mut current = n;
    chain.push(current);
    while current > 9 && !is_master(current) {
        current = digit_sum(current);
        chain.push(current);
    }
    (current, chain)
}

fn is_vowel(ch: char) -> bool {
    matches!(ch.to_ascii_uppercase(), 'A' | 'E' | 'I' | 'O' | 'U')
}

// ---------------------------------------------------------------------------
// NumerologyNumber & NumerologyResult
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NumerologyNumber {
    pub value: u32,
    pub is_master: bool,
    pub reduction_chain: Vec<u32>,
    pub meaning: String,
}

impl NumerologyNumber {
    fn from_raw(raw_sum: u32) -> Self {
        let (value, chain) = reduce_to_core(raw_sum);
        Self {
            value,
            is_master: is_master(value),
            reduction_chain: chain,
            meaning: meaning_for(value),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NumerologyResult {
    pub life_path: NumerologyNumber,
    pub expression: NumerologyNumber,
    pub soul_urge: NumerologyNumber,
    pub personality: NumerologyNumber,
    pub birthday: NumerologyNumber,
    pub chaldean_name: NumerologyNumber,
}

// ---------------------------------------------------------------------------
// Meaning lookup
// ---------------------------------------------------------------------------

fn meaning_for(n: u32) -> String {
    match n {
        1 => "Leadership, independence, pioneering".into(),
        2 => "Partnership, diplomacy, sensitivity".into(),
        3 => "Creativity, expression, joy".into(),
        4 => "Structure, discipline, foundation".into(),
        5 => "Freedom, change, adventure".into(),
        6 => "Responsibility, nurturing, harmony".into(),
        7 => "Analysis, wisdom, introspection".into(),
        8 => "Power, abundance, achievement".into(),
        9 => "Compassion, completion, universal love".into(),
        11 => "Intuition, spiritual insight, illumination (master)".into(),
        22 => "Master builder, practical visionary (master)".into(),
        33 => "Master teacher, selfless service (master)".into(),
        _ => format!("Compound vibration of {}", n),
    }
}

// ---------------------------------------------------------------------------
// Core calculations
// ---------------------------------------------------------------------------

/// Life Path: reduce year, month, day separately, then sum and reduce.
fn calculate_life_path(date: &str) -> Result<NumerologyNumber, EngineError> {
    let parts: Vec<&str> = date.split('-').collect();
    if parts.len() != 3 {
        return Err(EngineError::CalculationError(format!(
            "Invalid date format '{}', expected YYYY-MM-DD",
            date
        )));
    }

    let year: u32 = parts[0]
        .parse()
        .map_err(|_| EngineError::CalculationError("Invalid year".into()))?;
    let month: u32 = parts[1]
        .parse()
        .map_err(|_| EngineError::CalculationError("Invalid month".into()))?;
    let day: u32 = parts[2]
        .parse()
        .map_err(|_| EngineError::CalculationError("Invalid day".into()))?;

    let (year_reduced, _) = reduce_to_core(year);
    let (month_reduced, _) = reduce_to_core(month);
    let (day_reduced, _) = reduce_to_core(day);

    let raw_sum = year_reduced + month_reduced + day_reduced;
    Ok(NumerologyNumber::from_raw(raw_sum))
}

/// Expression (Destiny) Number: full name reduced via Pythagorean mapping.
fn calculate_expression(name: &str) -> NumerologyNumber {
    let raw_sum: u32 = name
        .chars()
        .filter_map(pythagorean_value)
        .sum();
    NumerologyNumber::from_raw(raw_sum)
}

/// Soul Urge: vowels only, Pythagorean mapping.
fn calculate_soul_urge(name: &str) -> NumerologyNumber {
    let raw_sum: u32 = name
        .chars()
        .filter(|c| c.is_ascii_alphabetic() && is_vowel(*c))
        .filter_map(pythagorean_value)
        .sum();
    NumerologyNumber::from_raw(raw_sum)
}

/// Personality Number: consonants only, Pythagorean mapping.
fn calculate_personality(name: &str) -> NumerologyNumber {
    let raw_sum: u32 = name
        .chars()
        .filter(|c| c.is_ascii_alphabetic() && !is_vowel(*c))
        .filter_map(pythagorean_value)
        .sum();
    NumerologyNumber::from_raw(raw_sum)
}

/// Birthday Number: just the day of birth reduced.
fn calculate_birthday(date: &str) -> Result<NumerologyNumber, EngineError> {
    let day_str = date
        .split('-')
        .nth(2)
        .ok_or_else(|| EngineError::CalculationError("Missing day in date".into()))?;
    let day: u32 = day_str
        .parse()
        .map_err(|_| EngineError::CalculationError("Invalid day".into()))?;
    Ok(NumerologyNumber::from_raw(day))
}

/// Chaldean Name Number: full name using Chaldean mapping.
fn calculate_chaldean_name(name: &str) -> NumerologyNumber {
    let raw_sum: u32 = name.chars().filter_map(chaldean_value).sum();
    NumerologyNumber::from_raw(raw_sum)
}

// ---------------------------------------------------------------------------
// Witness prompt generation
// ---------------------------------------------------------------------------

fn generate_witness_prompt(result: &NumerologyResult) -> String {
    let lp = result.life_path.value;
    let base = match lp {
        1 => "Your Life Path 1 speaks of the pioneer within. Notice: are you leading from authentic will, or from the need to be first?",
        2 => "Your Life Path 2 speaks of the bridge-builder. Notice: are you keeping peace, or discovering peace within yourself?",
        3 => "Your Life Path 3 speaks of creative expression. Notice: is the joy you share a reflection of inner joy, or a mask over silence?",
        4 => "Your Life Path 4 speaks of the architect of reality. Notice: are your foundations built from love, or from fear of chaos?",
        5 => "Your Life Path 5 speaks of the seeker of freedom. Notice: are you running toward experience, or away from stillness?",
        6 => "Your Life Path 6 speaks of the nurturer. Notice: is your care for others also care for yourself?",
        7 => "Your Life Path 7 suggests a path of inner wisdom. Notice: are you the seeker, or that which is found?",
        8 => "Your Life Path 8 speaks of mastery over the material. Notice: does your power serve the whole, or only the self?",
        9 => "Your Life Path 9 speaks of universal compassion. Notice: in your giving, have you also allowed yourself to receive?",
        11 => "Your Life Path 11 carries the master vibration of intuition. Notice: the light you channel -- is it yours to hold, or to pass through?",
        22 => "Your Life Path 22 carries the master builder vibration. Notice: the vision you build -- does it serve the world you see, or the world that is?",
        33 => "Your Life Path 33 carries the master teacher vibration. Notice: in your teaching, who is truly learning?",
        _ => "Your Life Path carries a unique vibration. Notice: what does this number reveal about the witness behind the personality?",
    };
    base.to_string()
}

// ---------------------------------------------------------------------------
// NumerologyEngine
// ---------------------------------------------------------------------------

pub struct NumerologyEngine;

impl NumerologyEngine {
    pub fn new() -> Self {
        Self
    }

    fn compute(&self, input: &EngineInput) -> Result<NumerologyResult, EngineError> {
        let birth = input
            .birth_data
            .as_ref()
            .ok_or_else(|| EngineError::CalculationError("birth_data is required for numerology".into()))?;

        let name = birth
            .name
            .as_deref()
            .ok_or_else(|| EngineError::CalculationError("name is required for numerology calculations".into()))?;

        if name.trim().is_empty() {
            return Err(EngineError::CalculationError("name must not be empty".into()));
        }

        let date = &birth.date;

        let life_path = calculate_life_path(date)?;
        let expression = calculate_expression(name);
        let soul_urge = calculate_soul_urge(name);
        let personality = calculate_personality(name);
        let birthday = calculate_birthday(date)?;
        let chaldean_name = calculate_chaldean_name(name);

        Ok(NumerologyResult {
            life_path,
            expression,
            soul_urge,
            personality,
            birthday,
            chaldean_name,
        })
    }
}

impl Default for NumerologyEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ConsciousnessEngine for NumerologyEngine {
    fn engine_id(&self) -> &str {
        "numerology"
    }

    fn engine_name(&self) -> &str {
        "Numerology"
    }

    fn required_phase(&self) -> u8 {
        0
    }

    async fn calculate(&self, input: EngineInput) -> Result<EngineOutput, EngineError> {
        let start = Instant::now();

        let result = self.compute(&input)?;
        let witness_prompt = generate_witness_prompt(&result);

        let result_json = serde_json::to_value(&result).map_err(|e| {
            EngineError::InternalError(format!("Failed to serialize NumerologyResult: {}", e))
        })?;

        let elapsed = start.elapsed().as_secs_f64() * 1000.0;

        Ok(EngineOutput {
            engine_id: self.engine_id().to_string(),
            result: result_json,
            witness_prompt,
            consciousness_level: 0,
            metadata: CalculationMetadata {
                calculation_time_ms: elapsed,
                backend: "native-rust".into(),
                precision_achieved: "exact".into(),
                cached: false,
                timestamp: Utc::now(),
            },
        })
    }

    async fn validate(&self, output: &EngineOutput) -> Result<ValidationResult, EngineError> {
        let mut messages = Vec::new();
        let mut valid = true;

        // Verify engine_id matches
        if output.engine_id != "numerology" {
            messages.push(format!(
                "engine_id mismatch: expected 'numerology', got '{}'",
                output.engine_id
            ));
            valid = false;
        }

        // Attempt to deserialize the result back into NumerologyResult
        let parsed: Result<NumerologyResult, _> = serde_json::from_value(output.result.clone());
        match parsed {
            Ok(nr) => {
                // Validate each core number is in the valid range (1-9 or master)
                let numbers = [
                    ("life_path", &nr.life_path),
                    ("expression", &nr.expression),
                    ("soul_urge", &nr.soul_urge),
                    ("personality", &nr.personality),
                    ("birthday", &nr.birthday),
                    ("chaldean_name", &nr.chaldean_name),
                ];

                for (label, num) in &numbers {
                    let v = num.value;
                    let valid_single = (1..=9).contains(&v);
                    let valid_master = is_master(v);
                    if !valid_single && !valid_master {
                        messages.push(format!(
                            "{} has invalid value {}: must be 1-9 or master (11,22,33)",
                            label, v
                        ));
                        valid = false;
                    }
                    if num.is_master != is_master(v) {
                        messages.push(format!(
                            "{}: is_master flag inconsistent with value {}",
                            label, v
                        ));
                        valid = false;
                    }
                }

                if valid {
                    messages.push("All numerology numbers are within valid ranges".into());
                }
            }
            Err(e) => {
                messages.push(format!("Failed to parse NumerologyResult: {}", e));
                valid = false;
            }
        }

        let confidence = if valid { 1.0 } else { 0.0 };

        Ok(ValidationResult {
            valid,
            confidence,
            messages,
        })
    }

    fn cache_key(&self, input: &EngineInput) -> String {
        let mut hasher = Sha256::new();
        hasher.update(b"numerology:");
        if let Some(ref birth) = input.birth_data {
            if let Some(ref name) = birth.name {
                hasher.update(name.as_bytes());
            }
            hasher.update(b"|");
            hasher.update(birth.date.as_bytes());
        }
        let hash = hasher.finalize();
        format!("numerology:{:x}", hash)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use noesis_core::Precision;
    use std::collections::HashMap;

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

    #[test]
    fn test_digit_sum() {
        assert_eq!(digit_sum(29), 11);
        assert_eq!(digit_sum(38), 11);
        assert_eq!(digit_sum(123), 6);
        assert_eq!(digit_sum(9), 9);
    }

    #[test]
    fn test_reduce_to_core_single_digit() {
        let (v, chain) = reduce_to_core(7);
        assert_eq!(v, 7);
        assert_eq!(chain, vec![7]);
    }

    #[test]
    fn test_reduce_to_core_master_11() {
        let (v, chain) = reduce_to_core(29);
        assert_eq!(v, 11);
        assert_eq!(chain, vec![29, 11]);
    }

    #[test]
    fn test_reduce_to_core_master_22() {
        let (v, chain) = reduce_to_core(22);
        assert_eq!(v, 22);
        assert_eq!(chain, vec![22]);
    }

    #[test]
    fn test_reduce_to_core_master_33() {
        let (v, chain) = reduce_to_core(33);
        assert_eq!(v, 33);
        assert_eq!(chain, vec![33]);
    }

    #[test]
    fn test_reduce_to_core_normal_reduction() {
        // 48 -> 12 -> 3
        let (v, chain) = reduce_to_core(48);
        assert_eq!(v, 3);
        assert_eq!(chain, vec![48, 12, 3]);
    }

    #[test]
    fn test_pythagorean_mapping() {
        assert_eq!(pythagorean_value('A'), Some(1));
        assert_eq!(pythagorean_value('Z'), Some(8));
        assert_eq!(pythagorean_value('J'), Some(1));
        assert_eq!(pythagorean_value('S'), Some(1));
        assert_eq!(pythagorean_value(' '), None);
    }

    #[test]
    fn test_chaldean_mapping() {
        assert_eq!(chaldean_value('A'), Some(1));
        assert_eq!(chaldean_value('F'), Some(8));
        assert_eq!(chaldean_value('O'), Some(7));
        assert_eq!(chaldean_value('Z'), Some(7));
        assert_eq!(chaldean_value(' '), None);
    }

    #[test]
    fn test_vowel_classification() {
        assert!(is_vowel('A'));
        assert!(is_vowel('e'));
        assert!(is_vowel('I'));
        assert!(is_vowel('o'));
        assert!(is_vowel('U'));
        assert!(!is_vowel('B'));
        assert!(!is_vowel('z'));
    }

    #[test]
    fn test_life_path_basic() {
        // 1990-05-15: year 1+9+9+0=19->10->1, month 0+5=5, day 1+5=6
        // sum = 1+5+6 = 12 -> 3
        let lp = calculate_life_path("1990-05-15").unwrap();
        assert_eq!(lp.value, 3);
        assert!(!lp.is_master);
    }

    #[test]
    fn test_life_path_invalid_date() {
        let result = calculate_life_path("not-a-date");
        assert!(result.is_err());
    }

    #[test]
    fn test_expression_number() {
        // "John" -> J(1) + O(6) + H(8) + N(5) = 20 -> 2
        let expr = calculate_expression("John");
        assert_eq!(expr.value, 2);
    }

    #[test]
    fn test_soul_urge() {
        // "John" vowels: O(6) -> 6
        let su = calculate_soul_urge("John");
        assert_eq!(su.value, 6);
    }

    #[test]
    fn test_personality() {
        // "John" consonants: J(1) + H(8) + N(5) = 14 -> 5
        let p = calculate_personality("John");
        assert_eq!(p.value, 5);
    }

    #[test]
    fn test_birthday() {
        // Day 15 -> 1+5 = 6
        let b = calculate_birthday("1990-05-15").unwrap();
        assert_eq!(b.value, 6);
    }

    #[test]
    fn test_chaldean_name() {
        // "John" -> J(1) + O(7) + H(5) + N(5) = 18 -> 9
        let cn = calculate_chaldean_name("John");
        assert_eq!(cn.value, 9);
    }

    #[tokio::test]
    async fn test_engine_calculate() {
        let engine = NumerologyEngine::new();
        let input = make_input("John Doe", "1990-05-15");
        let output = engine.calculate(input).await.unwrap();
        assert_eq!(output.engine_id, "numerology");
        assert!(!output.witness_prompt.is_empty());

        let result: NumerologyResult = serde_json::from_value(output.result).unwrap();
        // Verify all numbers are in valid range
        for num in [
            &result.life_path,
            &result.expression,
            &result.soul_urge,
            &result.personality,
            &result.birthday,
            &result.chaldean_name,
        ] {
            assert!(
                (1..=9).contains(&num.value) || is_master(num.value),
                "Number {} out of valid range",
                num.value
            );
        }
    }

    #[tokio::test]
    async fn test_engine_validate_valid() {
        let engine = NumerologyEngine::new();
        let input = make_input("Jane Smith", "1985-11-22");
        let output = engine.calculate(input).await.unwrap();
        let validation = engine.validate(&output).await.unwrap();
        assert!(validation.valid);
        assert_eq!(validation.confidence, 1.0);
    }

    #[tokio::test]
    async fn test_engine_missing_birth_data() {
        let engine = NumerologyEngine::new();
        let input = EngineInput {
            birth_data: None,
            current_time: Utc::now(),
            location: None,
            precision: Precision::Standard,
            options: HashMap::new(),
        };
        let result = engine.calculate(input).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_engine_missing_name() {
        let engine = NumerologyEngine::new();
        let input = EngineInput {
            birth_data: Some(noesis_core::BirthData {
                name: None,
                date: "1990-01-01".into(),
                time: None,
                latitude: 0.0,
                longitude: 0.0,
                timezone: "UTC".into(),
            }),
            current_time: Utc::now(),
            location: None,
            precision: Precision::Standard,
            options: HashMap::new(),
        };
        let result = engine.calculate(input).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_cache_key_deterministic() {
        let engine = NumerologyEngine::new();
        let input = make_input("Alice", "2000-01-01");
        let key1 = engine.cache_key(&input);
        let key2 = engine.cache_key(&input);
        assert_eq!(key1, key2);
        assert!(key1.starts_with("numerology:"));
    }

    #[test]
    fn test_cache_key_differs_for_different_inputs() {
        let engine = NumerologyEngine::new();
        let input_a = make_input("Alice", "2000-01-01");
        let input_b = make_input("Bob", "2000-01-01");
        assert_ne!(engine.cache_key(&input_a), engine.cache_key(&input_b));
    }

    #[test]
    fn test_engine_metadata() {
        let engine = NumerologyEngine::new();
        assert_eq!(engine.engine_id(), "numerology");
        assert_eq!(engine.engine_name(), "Numerology");
        assert_eq!(engine.required_phase(), 0);
    }
}
