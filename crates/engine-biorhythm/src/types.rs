//! Public result types for the Biorhythm engine.

use serde::{Deserialize, Serialize};

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

/// Biorhythm compatibility between two individuals.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompatibilityResult {
    pub physical_compatibility: f64,
    pub emotional_compatibility: f64,
    pub intellectual_compatibility: f64,
    pub overall_compatibility: f64,
}
