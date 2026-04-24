//! Pure calculation functions for the Biorhythm engine.
//!
//! This module is self-contained: it depends only on std, chrono, and serde.
//! It owns the cycle constants, all result types, and every trigonometric helper.

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::f64::consts::PI;

// ---------------------------------------------------------------------------
// Cycle constants
// ---------------------------------------------------------------------------

pub const PHYSICAL_PERIOD: f64 = 23.0;
pub const EMOTIONAL_PERIOD: f64 = 28.0;
pub const INTELLECTUAL_PERIOD: f64 = 33.0;
pub const INTUITIVE_PERIOD: f64 = 38.0;

/// Threshold in days for declaring a zero-crossing "critical".
pub const CRITICAL_THRESHOLD: f64 = 1.0;

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
// Pure calculation helpers
// ---------------------------------------------------------------------------

/// Sine value for a given number of days alive and cycle period.
pub(crate) fn cycle_value(days_alive: i64, period: f64) -> f64 {
    (2.0 * PI * days_alive as f64 / period).sin()
}

/// Map a sine value (-1..1) to a percentage (0..100).
pub(crate) fn to_percentage(value: f64) -> f64 {
    (value + 1.0) / 2.0 * 100.0
}

/// Determine the phase label for a cycle value and its derivative direction.
pub(crate) fn phase_label(value: f64, days_alive: i64, period: f64) -> String {
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
pub(crate) fn days_until_peak(days_alive: i64, period: f64) -> i64 {
    let current_phase = (days_alive as f64 % period) / period; // 0..1
                                                               // Peak is at phase = 0.25
    let distance = if current_phase <= 0.25 {
        0.25 - current_phase
    } else {
        1.25 - current_phase
    };
    let days = (distance * period).ceil() as i64;
    if days == 0 {
        period as i64
    } else {
        days
    }
}

/// Days until the next zero crossing.
/// Zero crossings occur at phase = 0.0 and phase = 0.5.
pub(crate) fn days_until_critical(days_alive: i64, period: f64) -> i64 {
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
pub(crate) fn is_critical_day(days_alive: i64, period: f64) -> bool {
    let value = cycle_value(days_alive, period);
    // Near zero means near a crossing. Use absolute value threshold.
    // sin(x) ~ 0 when x is near n*pi, i.e. near zero crossing.
    // A threshold of 1 day means |sin(2*pi*d/p)| < sin(2*pi*1/p).
    let threshold_value = (2.0 * PI * CRITICAL_THRESHOLD / period).sin().abs();
    value.abs() < threshold_value
}

/// Compute a single CycleResult.
pub fn compute_cycle(days_alive: i64, period: f64) -> CycleResult {
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
pub fn find_critical_days(
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
pub fn build_forecast(
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
pub fn generate_witness_prompt(result: &BiorhythmResult) -> String {
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
        highest.0, highest.1, lowest.0, lowest.1,
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
