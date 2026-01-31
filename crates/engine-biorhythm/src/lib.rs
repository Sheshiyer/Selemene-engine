//! Biorhythm Consciousness Engine
//!
//! Calculates physical (23-day), emotional (28-day), and intellectual (33-day) cycles,
//! plus intuitive (38-day) and three composite cycles (mastery, passion, wisdom).
//! Pure math -- no external dependencies beyond std and chrono.

use async_trait::async_trait;
use chrono::{NaiveDate, Utc};
use noesis_core::{
    CalculationMetadata, ConsciousnessEngine, EngineError, EngineInput, EngineOutput,
    ValidationResult,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::f64::consts::PI;
use std::time::Instant;

// ---------------------------------------------------------------------------
// Cycle constants
// ---------------------------------------------------------------------------

const PHYSICAL_PERIOD: f64 = 23.0;
const EMOTIONAL_PERIOD: f64 = 28.0;
const INTELLECTUAL_PERIOD: f64 = 33.0;
const INTUITIVE_PERIOD: f64 = 38.0;

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
// Pure calculation helpers
// ---------------------------------------------------------------------------

/// Sine value for a given number of days alive and cycle period.
fn cycle_value(days_alive: i64, period: f64) -> f64 {
    (2.0 * PI * days_alive as f64 / period).sin()
}

/// Map a sine value (-1..1) to a percentage (0..100).
fn to_percentage(value: f64) -> f64 {
    (value + 1.0) / 2.0 * 100.0
}

/// Determine the phase label for a cycle value and its derivative direction.
fn phase_label(value: f64, days_alive: i64, period: f64) -> String {
    // Check for critical (near zero crossing) first.
    if is_critical_day(days_alive, period) {
        return "Critical".to_string();
    }

    let cos_val = (2.0 * PI * days_alive as f64 / period).cos();

    if value > 0.95 {
        "Peak".to_string()
    } else if value < -0.95 {
        "Low".to_string()
    } else if cos_val > 0.0 {
        "Rising".to_string()
    } else {
        "Falling".to_string()
    }
}

/// Days until the next positive peak (sin = 1).
/// Peak occurs when days_alive / period = 0.25 + n for integer n.
fn days_until_peak(days_alive: i64, period: f64) -> i64 {
    let current_phase = (days_alive as f64 % period) / period; // 0..1
    // Peak is at phase = 0.25
    let distance = if current_phase <= 0.25 {
        0.25 - current_phase
    } else {
        1.25 - current_phase
    };
    let days = (distance * period).ceil() as i64;
    if days == 0 { period as i64 } else { days }
}

/// Days until the next zero crossing.
/// Zero crossings occur at phase = 0.0 and phase = 0.5.
fn days_until_critical(days_alive: i64, period: f64) -> i64 {
    let current_phase = (days_alive as f64 % period) / period; // 0..1
    // Zero crossings at 0.0 and 0.5
    let targets = [0.5, 1.0]; // next crossings relative to current position
    let mut min_days = i64::MAX;
    for &target in &targets {
        let distance = if current_phase < target {
            target - current_phase
        } else {
            continue;
        };
        let days = (distance * period).ceil() as i64;
        if days > 0 && days < min_days {
            min_days = days;
        }
    }
    if min_days == i64::MAX {
        // Wrap around: next zero crossing is at phase 0.0 of the next cycle
        let distance = 1.0 - current_phase;
        (distance * period).ceil() as i64
    } else {
        min_days
    }
}

/// Whether this day is within CRITICAL_THRESHOLD days of a zero crossing.
fn is_critical_day(days_alive: i64, period: f64) -> bool {
    let value = cycle_value(days_alive, period);
    // Near zero means near a crossing. Use absolute value threshold.
    // sin(x) ~ 0 when x is near n*pi, i.e. near zero crossing.
    // A threshold of 1 day means |sin(2*pi*d/p)| < sin(2*pi*1/p).
    let threshold_value = (2.0 * PI * CRITICAL_THRESHOLD / period).sin().abs();
    value.abs() < threshold_value
}

/// Compute a single CycleResult.
fn compute_cycle(days_alive: i64, period: f64) -> CycleResult {
    let value = cycle_value(days_alive, period);
    let percentage = to_percentage(value);
    let phase = phase_label(value, days_alive, period);
    let until_peak = days_until_peak(days_alive, period);
    let until_critical = days_until_critical(days_alive, period);
    let critical = is_critical_day(days_alive, period);
    let cycle_day = days_alive.rem_euclid(period as i64);

    CycleResult {
        value,
        percentage,
        phase,
        days_until_peak: until_peak,
        days_until_critical: until_critical,
        is_critical: critical,
        cycle_day,
    }
}

/// Collect upcoming critical days (dates where any primary cycle crosses zero) within a window.
fn find_critical_days(
    birth_date: NaiveDate,
    target_date: NaiveDate,
    window_days: i64,
) -> Vec<String> {
    let mut critical = Vec::new();
    let base_days = (target_date - birth_date).num_days();

    for offset in 1..=window_days {
        let d = base_days + offset;
        let any_critical = is_critical_day(d, PHYSICAL_PERIOD)
            || is_critical_day(d, EMOTIONAL_PERIOD)
            || is_critical_day(d, INTELLECTUAL_PERIOD);
        if any_critical {
            let date = target_date + chrono::Duration::days(offset);
            critical.push(date.format("%Y-%m-%d").to_string());
        }
    }

    critical
}

/// Build the optional forecast.
fn build_forecast(
    birth_date: NaiveDate,
    target_date: NaiveDate,
    forecast_days: i64,
) -> Vec<ForecastDay> {
    let base_days = (target_date - birth_date).num_days();
    (1..=forecast_days)
        .map(|offset| {
            let d = base_days + offset;
            let phys = to_percentage(cycle_value(d, PHYSICAL_PERIOD));
            let emot = to_percentage(cycle_value(d, EMOTIONAL_PERIOD));
            let inte = to_percentage(cycle_value(d, INTELLECTUAL_PERIOD));
            let intu = to_percentage(cycle_value(d, INTUITIVE_PERIOD));
            let date = target_date + chrono::Duration::days(offset);
            ForecastDay {
                date: date.format("%Y-%m-%d").to_string(),
                days_alive: d,
                physical: phys,
                emotional: emot,
                intellectual: inte,
                intuitive: intu,
                overall_energy: (phys + emot + inte) / 3.0,
            }
        })
        .collect()
}

/// Generate a reflective witness prompt from the current cycle state.
fn generate_witness_prompt(result: &BiorhythmResult) -> String {
    let phys_pct = result.physical.percentage;
    let emot_pct = result.emotional.percentage;
    let inte_pct = result.intellectual.percentage;

    // Find the highest and lowest primary cycles.
    let cycles = [
        ("physical", phys_pct),
        ("emotional", emot_pct),
        ("intellectual", inte_pct),
    ];
    let highest = cycles
        .iter()
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
        .unwrap();
    let lowest = cycles
        .iter()
        .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
        .unwrap();

    let any_critical = result.physical.is_critical
        || result.emotional.is_critical
        || result.intellectual.is_critical;

    let base = format!(
        "Your {} cycle is at {:.0}% while {} is at {:.0}%.",
        highest.0,
        highest.1,
        lowest.0,
        lowest.1,
    );

    let reflection = if any_critical {
        " Today holds a critical crossing — a threshold moment. \
         What old pattern is completing, and what new rhythm wants to begin?"
    } else if (highest.1 - lowest.1).abs() > 50.0 {
        " Notice: how does this contrast show up in your day? \
         Are you the energy, or the one who observes it?"
    } else if result.overall_energy > 70.0 {
        " With high overall energy, the temptation is to do more. \
         What would it mean to be fully present instead of merely productive?"
    } else if result.overall_energy < 30.0 {
        " Low energy is not a problem to solve — it is a season. \
         What does stillness want to teach you today?"
    } else {
        " In this balanced moment, awareness itself becomes the practice. \
         Can you notice the rhythm without trying to change it?"
    };

    format!("{}{}", base, reflection)
}

// ---------------------------------------------------------------------------
// Parse helpers
// ---------------------------------------------------------------------------

/// Parse a YYYY-MM-DD date string into NaiveDate.
fn parse_date(date_str: &str) -> Result<NaiveDate, EngineError> {
    NaiveDate::parse_from_str(date_str, "%Y-%m-%d").map_err(|e| {
        EngineError::CalculationError(format!("Invalid date '{}': {}", date_str, e))
    })
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
        ] {
            if cycle.value < -1.0 || cycle.value > 1.0 {
                valid = false;
                messages.push(format!("{} value {} out of [-1, 1] range", name, cycle.value));
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
            if val < 0.0 || val > 100.0 {
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

        let target_date = input.current_time.date_naive().format("%Y-%m-%d").to_string();
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
    use chrono::{DateTime, TimeZone, Utc};
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
    fn test_witness_prompt_not_empty() {
        let result = BiorhythmResult {
            days_alive: 10000,
            target_date: "2025-06-15".to_string(),
            physical: compute_cycle(10000, PHYSICAL_PERIOD),
            emotional: compute_cycle(10000, EMOTIONAL_PERIOD),
            intellectual: compute_cycle(10000, INTELLECTUAL_PERIOD),
            intuitive: compute_cycle(10000, INTUITIVE_PERIOD),
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
}
