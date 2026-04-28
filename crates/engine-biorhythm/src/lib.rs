//! Biorhythm Consciousness Engine
//!
//! Calculates physical (23-day), emotional (28-day), and intellectual (33-day) cycles,
//! plus intuitive (38-day) and three composite cycles (mastery, passion, wisdom).
//! Pure math -- no external dependencies beyond std and chrono.

pub mod calculator;

pub use calculator::{
    build_forecast, compute_cycle, find_critical_days, generate_witness_prompt, BiorhythmResult,
    CycleResult, ForecastDay, EMOTIONAL_PERIOD, INTELLECTUAL_PERIOD, INTUITIVE_PERIOD,
    PHYSICAL_PERIOD,
};

use async_trait::async_trait;
use chrono::Utc;
use noesis_core::{
    CalculationMetadata, ConsciousnessEngine, EngineError, EngineInput, EngineOutput,
    ValidationResult,
};
use sha2::{Digest, Sha256};
use std::time::Instant;

// ---------------------------------------------------------------------------
// Cycle constants
// ---------------------------------------------------------------------------

const PHYSICAL_PERIOD: f64 = 23.0;
const EMOTIONAL_PERIOD: f64 = 28.0;
const INTELLECTUAL_PERIOD: f64 = 33.0;
const INTUITIVE_PERIOD: f64 = 38.0;
const SPIRITUAL_PERIOD: f64 = 53.0;

/// Threshold in days for declaring a zero-crossing "critical".
const CRITICAL_THRESHOLD: f64 = 1.0;

// ---------------------------------------------------------------------------
// Result types
// ---------------------------------------------------------------------------

/// Full biorhythm calculation result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiorhythmResult {
    pub days_alive: i64,
    pub target_date: String,
    pub physical: CycleResult,
    pub emotional: CycleResult,
    pub intellectual: CycleResult,
    pub intuitive: CycleResult,
    pub spiritual: CycleResult,
    pub mastery: f64,
    pub passion: f64,
    pub wisdom: f64,
    pub critical_days: Vec<String>,
    pub overall_energy: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub forecast: Option<Vec<ForecastDay>>,
}

/// Result for a single biorhythm cycle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CycleResult {
    pub value: f64,
    pub percentage: f64,
    pub phase: String,
    pub days_until_peak: i64,
    pub days_until_critical: i64,
    pub is_critical: bool,
    pub cycle_day: i64,
}

/// One day in the optional forecast window.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForecastDay {
    pub date: String,
    pub days_alive: i64,
    pub physical: f64,
    pub emotional: f64,
    pub intellectual: f64,
    pub intuitive: f64,
    pub overall_energy: f64,
}

// ---------------------------------------------------------------------------
// Engine struct
// ---------------------------------------------------------------------------

/// Biorhythm consciousness engine.
pub struct BiorhythmEngine;

impl BiorhythmEngine {
    pub fn new() -> Self {
        Self
    }
}

impl Default for BiorhythmEngine {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Parse helpers
// ---------------------------------------------------------------------------

/// Parse a YYYY-MM-DD date string into NaiveDate.
fn parse_date(date_str: &str) -> Result<chrono::NaiveDate, EngineError> {
    chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
        .map_err(|e| EngineError::CalculationError(format!("Invalid date '{}': {}", date_str, e)))
}

// ---------------------------------------------------------------------------
// ConsciousnessEngine implementation
// ---------------------------------------------------------------------------

#[async_trait]
impl ConsciousnessEngine for BiorhythmEngine {
    fn engine_id(&self) -> &str {
        "biorhythm"
    }

    fn engine_name(&self) -> &str {
        "Biorhythm"
    }

    fn required_phase(&self) -> u8 {
        0
    }

    async fn calculate(&self, input: EngineInput) -> Result<EngineOutput, EngineError> {
        let start = Instant::now();

        // --- Extract birth date ---
        let birth_data = input.birth_data.as_ref().ok_or_else(|| {
            EngineError::CalculationError(
                "birth_data is required for biorhythm calculations".into(),
            )
        })?;

        let birth_date = parse_date(&birth_data.date)?;
        let target_date = input.current_time.date_naive();

        let days_alive = (target_date - birth_date).num_days();
        if days_alive < 0 {
            return Err(EngineError::CalculationError(
                "Target date is before birth date".into(),
            ));
        }

        // --- Primary cycles ---
        let physical = compute_cycle(days_alive, PHYSICAL_PERIOD);
        let emotional = compute_cycle(days_alive, EMOTIONAL_PERIOD);
        let intellectual = compute_cycle(days_alive, INTELLECTUAL_PERIOD);
        let intuitive = compute_cycle(days_alive, INTUITIVE_PERIOD);
        let spiritual = compute_cycle(days_alive, SPIRITUAL_PERIOD);

        // --- Composite cycles (percentages) ---
        let mastery = (physical.percentage + intellectual.percentage) / 2.0;
        let passion = (physical.percentage + emotional.percentage) / 2.0;
        let wisdom = (emotional.percentage + intellectual.percentage) / 2.0;

        // --- Overall energy ---
        let overall_energy =
            (physical.percentage + emotional.percentage + intellectual.percentage) / 3.0;

        // --- Forecast days option ---
        let forecast_days = input
            .options
            .get("forecast_days")
            .and_then(|v| v.as_i64())
            .unwrap_or(7);

        // --- Critical days in upcoming window ---
        let critical_days = find_critical_days(birth_date, target_date, forecast_days);

        // --- Optional forecast ---
        let forecast = if forecast_days > 0 {
            Some(build_forecast(birth_date, target_date, forecast_days))
        } else {
            None
        };

        // --- Assemble result ---
        let bio_result = BiorhythmResult {
            days_alive,
            target_date: target_date.format("%Y-%m-%d").to_string(),
            physical,
            emotional,
            intellectual,
            intuitive,
            spiritual,
            mastery,
            passion,
            wisdom,
            critical_days,
            overall_energy,
            forecast,
        };

        let witness_prompt = generate_witness_prompt(&bio_result);

        let result_value = serde_json::to_value(&bio_result).map_err(|e| {
            EngineError::CalculationError(format!("Failed to serialize result: {}", e))
        })?;

        let elapsed_ms = start.elapsed().as_secs_f64() * 1000.0;

        Ok(EngineOutput {
            engine_id: self.engine_id().to_string(),
            result: result_value,
            witness_prompt,
            consciousness_level: 0,
            metadata: CalculationMetadata {
                calculation_time_ms: elapsed_ms,
                backend: "native-rust".to_string(),
                precision_achieved: format!("{:?}", input.precision),
                cached: false,
                timestamp: Utc::now(),
                engine_version: env!("CARGO_PKG_VERSION").to_string(),
            },
        })
    }

    async fn validate(&self, output: &EngineOutput) -> Result<ValidationResult, EngineError> {
        let mut messages = Vec::new();
        let mut valid = true;

        // Deserialize to check structural integrity.
        let bio_result: BiorhythmResult =
            serde_json::from_value(output.result.clone()).map_err(|e| {
                EngineError::ValidationError(format!(
                    "Failed to deserialize BiorhythmResult: {}",
                    e
                ))
            })?;

        // Validate that days_alive is non-negative.
        if bio_result.days_alive < 0 {
            valid = false;
            messages.push("days_alive is negative".to_string());
        }

        // Validate primary cycle values are in [-1, 1].
        for (name, cycle) in [
            ("physical", &bio_result.physical),
            ("emotional", &bio_result.emotional),
            ("intellectual", &bio_result.intellectual),
            ("intuitive", &bio_result.intuitive),
            ("spiritual", &bio_result.spiritual),
        ] {
            if cycle.value < -1.0 || cycle.value > 1.0 {
                valid = false;
                messages.push(format!(
                    "{} value {} out of [-1, 1] range",
                    name, cycle.value
                ));
            }
            if cycle.percentage < 0.0 || cycle.percentage > 100.0 {
                valid = false;
                messages.push(format!(
                    "{} percentage {} out of [0, 100] range",
                    name, cycle.percentage
                ));
            }
        }

        // Validate composite values are in [0, 100].
        for (name, val) in [
            ("mastery", bio_result.mastery),
            ("passion", bio_result.passion),
            ("wisdom", bio_result.wisdom),
            ("overall_energy", bio_result.overall_energy),
        ] {
            if !(0.0..=100.0).contains(&val) {
                valid = false;
                messages.push(format!("{} value {} out of [0, 100] range", name, val));
            }
        }

        if valid {
            messages.push("All biorhythm values within expected ranges".to_string());
        }

        let confidence = if valid { 1.0 } else { 0.3 };

        Ok(ValidationResult {
            valid,
            confidence,
            messages,
        })
    }

    fn cache_key(&self, input: &EngineInput) -> String {
        let mut hasher = Sha256::new();
        hasher.update(self.engine_id().as_bytes());

        if let Some(ref bd) = input.birth_data {
            hasher.update(bd.date.as_bytes());
        }

        let target_date = input
            .current_time
            .date_naive()
            .format("%Y-%m-%d")
            .to_string();
        hasher.update(target_date.as_bytes());

        if let Some(forecast) = input.options.get("forecast_days") {
            hasher.update(forecast.to_string().as_bytes());
        }

        format!("{:x}", hasher.finalize())
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use super::calculator::{cycle_value, is_critical_day, to_percentage, PHYSICAL_PERIOD};
    use chrono::{DateTime, NaiveDate, TimeZone, Utc};
    use noesis_core::{BirthData, Precision};
    use std::collections::HashMap;

    fn make_input(birth_date: &str, target: DateTime<Utc>) -> EngineInput {
        EngineInput {
            birth_data: Some(BirthData {
                name: Some("Test".to_string()),
                date: birth_date.to_string(),
                time: None,
                latitude: 0.0,
                longitude: 0.0,
                timezone: "UTC".to_string(),
            }),
            current_time: target,
            location: None,
            precision: Precision::Standard,
            options: HashMap::new(),
        }
    }

    #[test]
    fn test_cycle_value_at_birth() {
        // At day 0, all sine values should be 0 (sin(0) = 0).
        let val = cycle_value(0, PHYSICAL_PERIOD);
        assert!((val).abs() < 1e-10);
    }

    #[test]
    fn test_cycle_value_at_quarter() {
        // At 1/4 of the period, sine should be 1.0 (peak).
        let quarter = (PHYSICAL_PERIOD / 4.0).round() as i64;
        let val = cycle_value(quarter, PHYSICAL_PERIOD);
        // Not exactly 1.0 due to rounding, but close.
        assert!(val > 0.9, "Expected near peak, got {}", val);
    }

    #[test]
    fn test_percentage_mapping() {
        assert!((to_percentage(1.0) - 100.0).abs() < 1e-10);
        assert!((to_percentage(-1.0) - 0.0).abs() < 1e-10);
        assert!((to_percentage(0.0) - 50.0).abs() < 1e-10);
    }

    #[test]
    fn test_is_critical_at_zero() {
        // Day 0 should be critical (sin(0) = 0, right at zero crossing).
        assert!(is_critical_day(0, PHYSICAL_PERIOD));
    }

    #[test]
    fn test_compute_cycle_basic() {
        let result = compute_cycle(100, PHYSICAL_PERIOD);
        assert!(result.value >= -1.0 && result.value <= 1.0);
        assert!(result.percentage >= 0.0 && result.percentage <= 100.0);
        assert!(result.days_until_peak > 0);
        assert!(result.days_until_critical > 0);
    }

    #[tokio::test]
    async fn test_calculate_returns_valid_output() {
        let engine = BiorhythmEngine::new();
        let target = Utc.with_ymd_and_hms(2025, 6, 15, 12, 0, 0).unwrap();
        let input = make_input("1990-01-01", target);
        let output = engine.calculate(input).await.unwrap();

        assert_eq!(output.engine_id, "biorhythm");
        assert!(!output.witness_prompt.is_empty());

        // Deserialize and check structure.
        let bio: BiorhythmResult = serde_json::from_value(output.result).unwrap();
        assert!(bio.days_alive > 0);
        assert!(bio.overall_energy >= 0.0 && bio.overall_energy <= 100.0);
    }

    #[tokio::test]
    async fn test_calculate_missing_birth_data() {
        let engine = BiorhythmEngine::new();
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
    async fn test_validate_accepts_good_output() {
        let engine = BiorhythmEngine::new();
        let target = Utc.with_ymd_and_hms(2025, 6, 15, 12, 0, 0).unwrap();
        let input = make_input("1990-01-01", target);
        let output = engine.calculate(input).await.unwrap();
        let validation = engine.validate(&output).await.unwrap();
        assert!(validation.valid);
        assert!((validation.confidence - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_cache_key_deterministic() {
        let engine = BiorhythmEngine::new();
        let target = Utc.with_ymd_and_hms(2025, 6, 15, 12, 0, 0).unwrap();
        let input = make_input("1990-01-01", target);
        let key1 = engine.cache_key(&input);
        let key2 = engine.cache_key(&input);
        assert_eq!(key1, key2);
    }

    #[test]
    fn test_cache_key_varies_by_birth() {
        let engine = BiorhythmEngine::new();
        let target = Utc.with_ymd_and_hms(2025, 6, 15, 12, 0, 0).unwrap();
        let input_a = make_input("1990-01-01", target);
        let input_b = make_input("1995-05-20", target);
        assert_ne!(engine.cache_key(&input_a), engine.cache_key(&input_b));
    }

    #[test]
    fn test_find_critical_days() {
        let birth = NaiveDate::from_ymd_opt(1990, 1, 1).unwrap();
        let target = NaiveDate::from_ymd_opt(2025, 6, 15).unwrap();
        let critical = find_critical_days(birth, target, 7);
        // Should return dates as strings and have reasonable count.
        assert!(critical.len() <= 7);
        for d in &critical {
            assert!(d.len() == 10); // YYYY-MM-DD
        }
    }

    #[test]
    fn test_forecast_has_six_cycle_fields_in_range() {
        let birth = NaiveDate::from_ymd_opt(1990, 1, 1).unwrap();
        let target = NaiveDate::from_ymd_opt(2025, 6, 15).unwrap();
        let forecast = build_forecast(birth, target, 7);
        assert_eq!(forecast.len(), 7);
        for day in &forecast {
            for (name, val) in [
                ("physical", day.physical),
                ("emotional", day.emotional),
                ("intellectual", day.intellectual),
                ("intuitive", day.intuitive),
                ("aesthetic", day.aesthetic),
                ("spiritual", day.spiritual),
                ("overall_energy", day.overall_energy),
            ] {
                assert!(
                    (0.0..=100.0).contains(&val),
                    "{} value {} out of [0, 100] on {}",
                    name,
                    val,
                    day.date
                );
            }
        }
    }

    #[test]
    fn test_witness_prompt_not_empty() {
        let result = BiorhythmResult {
            days_alive: 10000,
            target_date: "2025-06-15".to_string(),
            physical: compute_cycle(10000, PHYSICAL_PERIOD),
            emotional: compute_cycle(10000, EMOTIONAL_PERIOD),
            intellectual: compute_cycle(10000, INTELLECTUAL_PERIOD),
            intuitive: compute_cycle(10000, INTUITIVE_PERIOD),
            spiritual: compute_cycle(10000, SPIRITUAL_PERIOD),
            mastery: 50.0,
            passion: 50.0,
            wisdom: 50.0,
            critical_days: vec![],
            overall_energy: 50.0,
            forecast: None,
        };
        let prompt = generate_witness_prompt(&result);
        assert!(!prompt.is_empty());
        assert!(prompt.contains('%'));
    }

    #[test]
    fn test_spiritual_cycle_period_and_invariants() {
        // At day 0, spiritual sine value should be 0 (sin(0) = 0).
        let val_at_birth = cycle_value(0, SPIRITUAL_PERIOD);
        assert!(val_at_birth.abs() < 1e-10, "Expected 0 at birth, got {}", val_at_birth);

        // One full period should return very close to 0 again.
        let val_at_period = cycle_value(SPIRITUAL_PERIOD as i64, SPIRITUAL_PERIOD);
        assert!(val_at_period.abs() < 1e-10, "Expected 0 at full period, got {}", val_at_period);

        // compute_cycle yields value in [-1, 1] and percentage in [0, 100].
        let result = compute_cycle(100, SPIRITUAL_PERIOD);
        assert!(result.value >= -1.0 && result.value <= 1.0);
        assert!(result.percentage >= 0.0 && result.percentage <= 100.0);
        assert!(result.days_until_peak > 0);
        assert!(result.days_until_critical > 0);

        // cycle_day must be in [0, 52] inclusive (period 53 ⇒ modulo 53).
        assert!(result.cycle_day >= 0 && result.cycle_day < SPIRITUAL_PERIOD as i64);
    }

    #[tokio::test]
    async fn test_spiritual_field_in_output() {
        let engine = BiorhythmEngine::new();
        let target = Utc.with_ymd_and_hms(2025, 6, 15, 12, 0, 0).unwrap();
        let input = make_input("1990-01-01", target);
        let output = engine.calculate(input).await.unwrap();

        let bio: BiorhythmResult = serde_json::from_value(output.result).unwrap();
        assert!(bio.spiritual.value >= -1.0 && bio.spiritual.value <= 1.0);
        assert!(bio.spiritual.percentage >= 0.0 && bio.spiritual.percentage <= 100.0);
    }
}
