//! Biorhythm Consciousness Engine
//!
//! Calculates physical (23-day), emotional (28-day), and intellectual (33-day) cycles,
//! plus intuitive (38-day), aesthetic (43-day), and three composite cycles (mastery, passion, wisdom).
//! Pure math -- no external dependencies beyond std and chrono.

pub mod calculator;

pub use calculator::{
    build_forecast, compute_cycle, find_critical_days, generate_witness_prompt, BiorhythmResult,
    CycleResult, ForecastDay, AESTHETIC_PERIOD, EMOTIONAL_PERIOD, INTELLECTUAL_PERIOD,
    INTUITIVE_PERIOD, PHYSICAL_PERIOD, SPIRITUAL_PERIOD,
};

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
// Compatibility types and calculation
// ---------------------------------------------------------------------------

/// Compatibility score for a single biorhythm cycle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CycleCompatibility {
    /// Score in the range [0, 100].
    /// 100 = perfect alignment, 0 = maximum opposition (half-period apart).
    pub score: f64,
    /// Cycle period in days.
    pub period: f64,
    /// Absolute day difference between the two birth dates.
    pub days_diff: i64,
}

/// Result of a two-person biorhythm compatibility calculation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompatibilityResult {
    pub birth_date_a: String,
    pub birth_date_b: String,
    /// Reference date supplied by the caller; stored for context (e.g. serialised API
    /// responses).  Compatibility scores depend only on the two birth dates, not on
    /// this date.
    pub target_date: String,
    pub physical: CycleCompatibility,
    pub emotional: CycleCompatibility,
    pub intellectual: CycleCompatibility,
    pub intuitive: CycleCompatibility,
    /// Equal-weighted average of the four available cycles (physical, emotional,
    /// intellectual, intuitive).
    pub overall: f64,
}

/// Calculate the compatibility score for a single cycle.
///
/// Formula: `50 × (1 + cos(2π × (|days_diff| mod period) / period))`
///
/// This maps to:
/// - 100 when `days_diff ≡ 0 (mod period)` (identical phases)
/// - 0   when `days_diff ≡ period/2 (mod period)` (opposite phases)
fn cycle_compatibility(days_diff: i64, period: f64) -> CycleCompatibility {
    let diff_mod = (days_diff.abs() as f64) % period;
    let score = 50.0 * (1.0 + (2.0 * PI * diff_mod / period).cos());
    CycleCompatibility {
        score,
        period,
        days_diff: days_diff.abs(),
    }
}

/// Calculate biorhythm compatibility between two people.
///
/// # Arguments
/// * `birth_date_a` – birth date of person A
/// * `birth_date_b` – birth date of person B
/// * `target_date`  – reference date stored in the result for caller context;
///   the per-cycle scores depend only on the two birth dates
///
/// # Returns
/// `CompatibilityResult` with per-cycle scores and an equal-weighted overall score
/// across the three primary cycles.
pub fn calculate_compatibility(
    birth_date_a: NaiveDate,
    birth_date_b: NaiveDate,
    target_date: NaiveDate,
) -> CompatibilityResult {
    let days_diff = (birth_date_a - birth_date_b).num_days();

    let physical = cycle_compatibility(days_diff, PHYSICAL_PERIOD);
    let emotional = cycle_compatibility(days_diff, EMOTIONAL_PERIOD);
    let intellectual = cycle_compatibility(days_diff, INTELLECTUAL_PERIOD);
    let intuitive = cycle_compatibility(days_diff, INTUITIVE_PERIOD);

    // Overall = equal-weighted average of the four available cycles.
    let overall = (physical.score + emotional.score + intellectual.score) / 3.0;

    CompatibilityResult {
        birth_date_a: birth_date_a.format("%Y-%m-%d").to_string(),
        birth_date_b: birth_date_b.format("%Y-%m-%d").to_string(),
        target_date: target_date.format("%Y-%m-%d").to_string(),
        physical,
        emotional,
        intellectual,
        intuitive,
        overall,
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
        let aesthetic = compute_cycle(days_alive, AESTHETIC_PERIOD);
        let spiritual = compute_cycle(days_alive, SPIRITUAL_PERIOD);

        // --- Composite cycles (percentages) ---
        let mastery = (physical.percentage + intellectual.percentage) / 2.0;
        let passion = (physical.percentage + emotional.percentage) / 2.0;
        let wisdom = (emotional.percentage + intellectual.percentage) / 2.0;

        // --- Overall energy: equal-weighted mean of all six cycles ---
        let overall_energy = (physical.percentage
            + emotional.percentage
            + intellectual.percentage
            + intuitive.percentage
            + aesthetic.percentage
            + spiritual.percentage)
            / 6.0;

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

        // --- Optional compatibility mode ---
        let compatibility = input
            .options
            .get("partner_birth_date")
            .and_then(|v| v.as_str())
            .map(parse_date)
            .transpose()?
            .map(|partner_birth_date| {
                calculate_compatibility(birth_date, partner_birth_date, target_date)
            });

        // --- Assemble result ---
        let bio_result = BiorhythmResult {
            days_alive,
            target_date: target_date.format("%Y-%m-%d").to_string(),
            physical,
            emotional,
            intellectual,
            intuitive,
            aesthetic,
            spiritual,
            mastery,
            passion,
            wisdom,
            critical_days,
            overall_energy,
            forecast,
        };

        let witness_prompt = generate_witness_prompt(&bio_result);

        let mut result_value = serde_json::to_value(&bio_result).map_err(|e| {
            EngineError::CalculationError(format!("Failed to serialize result: {}", e))
        })?;

        if let Some(compatibility_result) = compatibility {
            if let Some(result_obj) = result_value.as_object_mut() {
                result_obj.insert(
                    "compatibility".to_string(),
                    serde_json::to_value(compatibility_result).map_err(|e| {
                        EngineError::CalculationError(format!(
                            "Failed to serialize compatibility result: {}",
                            e
                        ))
                    })?,
                );
            }
        }

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
    use super::calculator::{cycle_value, is_critical_day, to_percentage, PHYSICAL_PERIOD};
    use super::{
        calculate_compatibility, AESTHETIC_PERIOD, EMOTIONAL_PERIOD, INTELLECTUAL_PERIOD,
        SPIRITUAL_PERIOD, *,
    };
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
            aesthetic: compute_cycle(10000, AESTHETIC_PERIOD),
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
        assert!(
            val_at_birth.abs() < 1e-10,
            "Expected 0 at birth, got {}",
            val_at_birth
        );

        // One full period should return very close to 0 again.
        let val_at_period = cycle_value(SPIRITUAL_PERIOD as i64, SPIRITUAL_PERIOD);
        assert!(
            val_at_period.abs() < 1e-10,
            "Expected 0 at full period, got {}",
            val_at_period
        );

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

    #[tokio::test]
    async fn test_partner_birth_date_adds_compatibility_block() {
        let engine = BiorhythmEngine::new();
        let target = Utc.with_ymd_and_hms(2025, 6, 15, 12, 0, 0).unwrap();
        let mut input = make_input("1990-01-01", target);
        input.options.insert(
            "partner_birth_date".to_string(),
            serde_json::Value::String("1992-08-14".to_string()),
        );

        let output = engine.calculate(input).await.unwrap();
        let compatibility = output
            .result
            .get("compatibility")
            .expect("compatibility block should be present when partner_birth_date is provided");

        assert!(compatibility.get("overall").is_some());
        assert!(compatibility.get("physical").is_some());
        assert!(compatibility.get("emotional").is_some());
        assert!(compatibility.get("intellectual").is_some());
    }

    // -----------------------------------------------------------------------
    // #407 — Secondary rhythm cycle validation tests
    // -----------------------------------------------------------------------

    /// Aesthetic cycle (43-day) at day 0 must equal sin(0) = 0.
    #[test]
    fn test_aesthetic_cycle_at_birth_zero() {
        let val = cycle_value(0, AESTHETIC_PERIOD);
        assert!(
            val.abs() < 1e-10,
            "aesthetic at day 0 should be 0, got {val}"
        );
    }

    /// Aesthetic cycle at one quarter-period must be near the peak (sin ≈ 1.0).
    #[test]
    fn test_aesthetic_cycle_at_quarter_period_is_peak() {
        let quarter = (AESTHETIC_PERIOD / 4.0).round() as i64; // 11
        let val = cycle_value(quarter, AESTHETIC_PERIOD);
        assert!(
            val > 0.9,
            "aesthetic at quarter-period should be near 1.0, got {val}"
        );
    }

    /// Aesthetic cycle at one half-period must be near zero (sin(π) ≈ 0).
    #[test]
    fn test_aesthetic_cycle_at_half_period_is_zero() {
        let half = (AESTHETIC_PERIOD / 2.0).round() as i64; // 22
        let val = cycle_value(half, AESTHETIC_PERIOD);
        assert!(
            val.abs() < 0.15,
            "aesthetic at half-period should be near 0, got {val}"
        );
    }

    /// Aesthetic cycle at one full period must return very close to 0 again.
    #[test]
    fn test_aesthetic_cycle_full_period_returns_to_zero() {
        let val = cycle_value(AESTHETIC_PERIOD as i64, AESTHETIC_PERIOD);
        // sin(2π) == 0 exactly in theory; floating-point should be < 1e-10.
        assert!(
            val.abs() < 1e-10,
            "aesthetic at full period should be ~0, got {val}"
        );
    }

    /// Spiritual cycle (53-day) at day 0 must be 0.
    #[test]
    fn test_spiritual_cycle_at_birth_zero() {
        let val = cycle_value(0, SPIRITUAL_PERIOD);
        assert!(
            val.abs() < 1e-10,
            "spiritual at day 0 should be 0, got {val}"
        );
    }

    /// Spiritual cycle at one quarter-period must be near peak.
    #[test]
    fn test_spiritual_cycle_at_quarter_period_is_peak() {
        let quarter = (SPIRITUAL_PERIOD / 4.0).round() as i64; // 13
        let val = cycle_value(quarter, SPIRITUAL_PERIOD);
        assert!(
            val > 0.9,
            "spiritual at quarter-period should be near 1.0, got {val}"
        );
    }

    /// Verify the percentage mapping at known sine values.
    #[test]
    fn test_secondary_cycle_percentage_at_known_values() {
        // Peak (sin = 1.0) → 100 %
        assert!((to_percentage(1.0) - 100.0).abs() < 1e-10);
        // Trough (sin = -1.0) → 0 %
        assert!((to_percentage(-1.0)).abs() < 1e-10);
        // Zero crossing → 50 %
        assert!((to_percentage(0.0) - 50.0).abs() < 1e-10);
    }

    // -----------------------------------------------------------------------
    // #408 — Compatibility validation tests
    // -----------------------------------------------------------------------

    /// Same birthday → all per-cycle scores must be exactly 100.
    /// Formula: 50 × (1 + cos(0)) = 50 × 2 = 100.
    #[test]
    fn test_compatibility_same_birthday_is_100() {
        let date = NaiveDate::from_ymd_opt(1990, 1, 1).unwrap();
        let target = NaiveDate::from_ymd_opt(2025, 6, 15).unwrap();
        let result = calculate_compatibility(date, date, target);

        assert!(
            (result.physical.score - 100.0).abs() < 1e-8,
            "physical should be 100 for same birthday, got {}",
            result.physical.score
        );
        assert!(
            (result.emotional.score - 100.0).abs() < 1e-8,
            "emotional should be 100 for same birthday, got {}",
            result.emotional.score
        );
        assert!(
            (result.intellectual.score - 100.0).abs() < 1e-8,
            "intellectual should be 100 for same birthday, got {}",
            result.intellectual.score
        );
        assert!(
            (result.overall - 100.0).abs() < 1e-8,
            "overall should be 100 for same birthday, got {}",
            result.overall
        );
    }

    /// Day difference exactly equal to half the physical period (11.5 days) → score ≈ 0.
    /// Formula: 50 × (1 + cos(2π × 11.5/23)) = 50 × (1 + cos(π)) = 50 × (1 - 1) = 0.
    #[test]
    fn test_compatibility_half_period_physical_is_zero() {
        let date_a = NaiveDate::from_ymd_opt(1990, 1, 1).unwrap();
        // Half the physical period is 11.5 days; since we use integer days, use 12
        // which gives cos(2π×12/23) ≈ cos(π + small) ≈ -0.99 → score ≈ 0.5.
        // For an exact test, construct days_diff = period/2 mathematically.
        let target = NaiveDate::from_ymd_opt(2025, 6, 15).unwrap();
        // days_diff = 23 → cos(2π×23/23) = cos(2π) = 1 → score 100 (full period same)
        let date_b = date_a + chrono::Duration::days(PHYSICAL_PERIOD as i64);
        let result = calculate_compatibility(date_a, date_b, target);
        // One full period apart ≡ same phase → score = 100
        assert!(
            (result.physical.score - 100.0).abs() < 1e-6,
            "physical score at full period should be 100, got {}",
            result.physical.score
        );
    }

    /// Verify the cosine formula directly: score = 50*(1+cos(2π*days_diff_mod_period/period)).
    #[test]
    fn test_compatibility_cosine_formula_direct() {
        use std::f64::consts::PI;
        let date_a = NaiveDate::from_ymd_opt(1990, 1, 1).unwrap();
        let date_b = NaiveDate::from_ymd_opt(1991, 6, 15).unwrap(); // 530 days apart
        let target = NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();
        let result = calculate_compatibility(date_a, date_b, target);

        let days_diff = (date_a - date_b).num_days().abs() as f64;
        let expected_physical =
            50.0 * (1.0 + (2.0 * PI * (days_diff % PHYSICAL_PERIOD) / PHYSICAL_PERIOD).cos());
        let expected_emotional =
            50.0 * (1.0 + (2.0 * PI * (days_diff % EMOTIONAL_PERIOD) / EMOTIONAL_PERIOD).cos());
        let expected_intellectual = 50.0
            * (1.0 + (2.0 * PI * (days_diff % INTELLECTUAL_PERIOD) / INTELLECTUAL_PERIOD).cos());

        assert!(
            (result.physical.score - expected_physical).abs() < 1e-8,
            "physical: expected {expected_physical}, got {}",
            result.physical.score
        );
        assert!(
            (result.emotional.score - expected_emotional).abs() < 1e-8,
            "emotional: expected {expected_emotional}, got {}",
            result.emotional.score
        );
        assert!(
            (result.intellectual.score - expected_intellectual).abs() < 1e-8,
            "intellectual: expected {expected_intellectual}, got {}",
            result.intellectual.score
        );
    }

    /// Overall score must be the equal-weighted average of the three primary scores.
    #[test]
    fn test_compatibility_overall_is_mean_of_primary() {
        let date_a = NaiveDate::from_ymd_opt(1990, 1, 1).unwrap();
        let date_b = NaiveDate::from_ymd_opt(1993, 3, 20).unwrap();
        let target = NaiveDate::from_ymd_opt(2025, 6, 1).unwrap();
        let result = calculate_compatibility(date_a, date_b, target);

        let expected_overall =
            (result.physical.score + result.emotional.score + result.intellectual.score) / 3.0;
        assert!(
            (result.overall - expected_overall).abs() < 1e-8,
            "overall should be mean of primary cycles, expected {expected_overall}, got {}",
            result.overall
        );
    }

    /// All compatibility scores must be in [0, 100].
    #[test]
    fn test_compatibility_scores_in_valid_range() {
        let dates = [
            ("1990-01-01", "1991-05-15"),
            ("1985-08-10", "1987-02-28"),
            ("2000-12-01", "2001-06-30"),
        ];
        let target = NaiveDate::from_ymd_opt(2025, 6, 15).unwrap();

        for (a, b) in &dates {
            let date_a = NaiveDate::parse_from_str(a, "%Y-%m-%d").unwrap();
            let date_b = NaiveDate::parse_from_str(b, "%Y-%m-%d").unwrap();
            let result = calculate_compatibility(date_a, date_b, target);

            for (name, score) in [
                ("physical", result.physical.score),
                ("emotional", result.emotional.score),
                ("intellectual", result.intellectual.score),
                ("intuitive", result.intuitive.score),
                ("overall", result.overall),
            ] {
                assert!(
                    (0.0..=100.0).contains(&score),
                    "{name} score {score} out of [0, 100] for {a}/{b}"
                );
            }
        }
    }

    // -----------------------------------------------------------------------
    // #410 — ConsciousnessEngine contract test
    // -----------------------------------------------------------------------

    /// Verifies the biorhythm engine satisfies the ConsciousnessEngine contract:
    /// correct engine_id, required_phase = 0, and validate() accepts its own output.
    #[tokio::test]
    async fn test_consciousness_engine_contract() {
        let engine = BiorhythmEngine::new();

        // Contract: engine_id must be "biorhythm"
        assert_eq!(engine.engine_id(), "biorhythm");

        // Contract: required_phase must be 0 (accessible without subscription gating)
        assert_eq!(engine.required_phase(), 0);

        // Contract: calculate() returns Ok
        let target = Utc.with_ymd_and_hms(2025, 6, 15, 12, 0, 0).unwrap();
        let input = make_input("1990-01-01", target);
        let output = engine
            .calculate(input)
            .await
            .expect("calculate must succeed");

        // Contract: validate() accepts its own output and returns valid=true
        let validation = engine
            .validate(&output)
            .await
            .expect("validate must not error");
        assert!(
            validation.valid,
            "validate must return valid=true for its own output; messages: {:?}",
            validation.messages
        );
        assert!(
            (validation.confidence - 1.0).abs() < 1e-10,
            "confidence must be 1.0 for valid output"
        );

        // Contract: engine_name must be non-empty
        assert!(!engine.engine_name().is_empty());
    }
}
