//! Result payload for the Financial Biosensor engine.
//!
//! Every number in this module is subject to a three-lock gate: correctness
//! (units and range are stated), structural consequence (removing the number
//! changes the serialized output), and provenance (the [`ProvenanceBlock`]
//! records the engine, the version, and the exact fields each number came
//! from). The gate is encoded here as data rather than asserted in prose, so
//! that a reader of the payload can check it without reading this crate.

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

// ---------------------------------------------------------------------------
// House-model declaration
// ---------------------------------------------------------------------------

/// Stable identifier for the composite formula.
///
/// Bump this on ANY change to the weights, the normalizations, the thresholds,
/// or the contributor set. It is recorded in every payload so that two numbers
/// produced by two versions of this crate are never silently compared.
pub const FORMULA_VERSION: &str = "financial-biosensor/composite@1";

/// The kind of claim this engine makes. Not empirical, not a forecast.
pub const CLAIM_MODE: &str = "house-model";

pub const HOUSE_MODEL_STATEMENT: &str = concat!(
    "The Daily Decision Index is a house model. It is a declared weighted composite over ",
    "five engine outputs and one optional biometric sample. The weights are declared by ",
    "this house; they are not fitted to data. No claim is made that this number forecasts ",
    "any financial or other outcome. Its only warrant is that it reports, in one place, ",
    "what the named sources register at the named time. Removing any contributor changes ",
    "the number, and the provenance block records exactly which."
);

pub const AUTHORSHIP_STATEMENT: &str = concat!(
    "This engine does not decide. It reflects. The decision, and its consequences, ",
    "remain with the practitioner."
);

/// Declared claim type, carried in every payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Declaration {
    /// Always `"house-model"`, so a consumer can branch on claim type.
    pub claim_mode: String,
    /// Always `"not-prediction"`.
    pub excluded_claim: String,
    pub statement: String,
    pub authorship: String,
    pub formula_version: String,
}

impl Default for Declaration {
    fn default() -> Self {
        Self {
            claim_mode: CLAIM_MODE.to_string(),
            excluded_claim: "not-prediction".to_string(),
            statement: HOUSE_MODEL_STATEMENT.to_string(),
            authorship: AUTHORSHIP_STATEMENT.to_string(),
            formula_version: FORMULA_VERSION.to_string(),
        }
    }
}

// ---------------------------------------------------------------------------
// Contributors
// ---------------------------------------------------------------------------

/// The six contributors to the composite, in declared-weight order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContributorId {
    HeartRateVariability,
    Chronofield,
    ActivePlanetaryWeather,
    ThreeWaveCycle,
    EnergeticAuthority,
    GiftShadowSpectrum,
}

impl ContributorId {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::HeartRateVariability => "heart_rate_variability",
            Self::Chronofield => "chronofield",
            Self::ActivePlanetaryWeather => "active_planetary_weather",
            Self::ThreeWaveCycle => "three_wave_cycle",
            Self::EnergeticAuthority => "energetic_authority",
            Self::GiftShadowSpectrum => "gift_shadow_spectrum",
        }
    }

    /// Declared house weight. Sums to exactly 1.0 across all six.
    ///
    /// The biometric leg carries the largest single share because it is the
    /// only contributor sampled in the present; every other input is a
    /// deterministic function of birth data and clock time and therefore
    /// cannot contradict itself.
    pub fn declared_weight(self) -> f64 {
        match self {
            Self::HeartRateVariability => 0.25,
            Self::Chronofield => 0.20,
            Self::ActivePlanetaryWeather => 0.20,
            Self::ThreeWaveCycle => 0.15,
            Self::EnergeticAuthority => 0.12,
            Self::GiftShadowSpectrum => 0.08,
        }
    }

    /// Which leg of the Kha-Ba-La triad this contributor carries.
    pub fn leg(self) -> &'static str {
        match self {
            Self::HeartRateVariability => "Ba",
            Self::EnergeticAuthority => "La",
            _ => "Kha",
        }
    }

    pub const ALL: [ContributorId; 6] = [
        Self::HeartRateVariability,
        Self::Chronofield,
        Self::ActivePlanetaryWeather,
        Self::ThreeWaveCycle,
        Self::EnergeticAuthority,
        Self::GiftShadowSpectrum,
    ];
}

/// Whether a contributor could be read, and if not, why not.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum ContributorStatus {
    /// Read successfully; the contributor carries a normalized value.
    Present,
    /// Could not be read. `reason` is machine-stable; `detail` is readable and
    /// names the field or call site responsible where that is known.
    Absent { reason: String, detail: String },
}

impl ContributorStatus {
    pub fn absent(reason: &str, detail: impl Into<String>) -> Self {
        Self::Absent {
            reason: reason.to_string(),
            detail: detail.into(),
        }
    }

    pub fn is_present(&self) -> bool {
        matches!(self, Self::Present)
    }
}

/// One contributor, as read. This is the internal working form; the
/// serialized form is [`ContributorProvenance`].
#[derive(Debug, Clone)]
pub struct Contributor {
    pub id: ContributorId,
    /// Source engine id, or `"practitioner-declared"` for option-supplied input.
    pub engine_id: String,
    /// The source engine's reported version, or the declared input's source.
    pub engine_version: String,
    /// RFC 6901 pointers into the source result, or option keys, actually read.
    pub fields_consumed: Vec<String>,
    /// The exact normalization applied, written out.
    pub normalization: String,
    /// The raw values read, verbatim, before normalization.
    pub raw: Value,
    /// Normalized to `[0.0, 1.0]`. `None` exactly when `status` is `Absent`.
    pub normalized: Option<f64>,
    pub status: ContributorStatus,
    /// What this source registered, in words. No instruction, no forecast.
    pub observation: String,
}

impl Contributor {
    /// An absent contributor with no reading.
    pub fn absent(
        id: ContributorId,
        engine_id: &str,
        reason: &str,
        detail: impl Into<String>,
    ) -> Self {
        let detail = detail.into();
        Self {
            id,
            engine_id: engine_id.to_string(),
            engine_version: String::new(),
            fields_consumed: Vec::new(),
            normalization: "not applied — contributor absent".to_string(),
            raw: Value::Null,
            normalized: None,
            observation: format!("Not consulted: {}", detail),
            status: ContributorStatus::absent(reason, detail),
        }
    }
}

// ---------------------------------------------------------------------------
// Provenance
// ---------------------------------------------------------------------------

/// Serialized provenance for a single contributor.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContributorProvenance {
    pub contributor: String,
    pub leg: String,
    pub engine_id: String,
    pub engine_version: String,
    pub fields_consumed: Vec<String>,
    pub normalization: String,
    pub raw: Value,
    pub normalized: Option<f64>,
    pub status: ContributorStatus,
    /// Weight this contributor would carry if all six were present.
    pub weight_declared: f64,
    /// Weight actually applied after renormalization. `0.0` when absent.
    pub weight_effective: f64,
}

/// The three-lock gate, as data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvenanceBlock {
    pub formula_version: String,
    /// `CARGO_PKG_VERSION` of this crate.
    pub engine_version: String,
    pub computed_at: DateTime<Utc>,
    /// One entry per contributor, present or absent. Always length 6.
    pub contributors: Vec<ContributorProvenance>,
    /// Sum of `weight_declared` over present contributors, in `[0.0, 1.0]`.
    pub coverage: f64,
    /// Declared multiplier applied to confidence when the biometric leg is
    /// unsampled: `1.0` when present, `0.8` when absent.
    pub ba_factor: f64,
}

// ---------------------------------------------------------------------------
// Output 1 — Daily Decision Index
// ---------------------------------------------------------------------------

/// How far the sources read together. Deliberately not go / wait / avoid.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Convergence {
    Divergent,
    Partial,
    Convergent,
}

/// Whether enough of the sources could be read for a composite to mean
/// anything more than a restatement of one of them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Sufficiency {
    /// `coverage >= 0.40`.
    Sufficient,
    /// `coverage < 0.40`. The value is withheld rather than published thin.
    Insufficient,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContributorContribution {
    pub contributor: String,
    pub normalized: f64,
    pub weight_effective: f64,
    /// `normalized * weight_effective` — the additive share of the index.
    pub share: f64,
    pub observation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyDecisionIndex {
    pub date: NaiveDate,
    /// Range `[0.0, 1.0]`, 3 decimal places. `None` when `sufficiency` is
    /// `Insufficient`.
    pub value: Option<f64>,
    pub convergence: Option<Convergence>,
    pub sufficiency: Sufficiency,
    pub coverage: f64,
    pub contributions: Vec<ContributorContribution>,
}

// ---------------------------------------------------------------------------
// Output 2 — Weekly Risk Landscape
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransitionMarker {
    /// `"Mahadasha"` | `"Antardasha"` | `"Pratyantardasha"`.
    pub level: String,
    pub from_planet: String,
    pub to_planet: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskDay {
    pub date: NaiveDate,
    /// Recomputed for this date from the day-varying contributors only.
    pub value: Option<f64>,
    pub convergence: Option<Convergence>,
    pub three_wave: Option<f64>,
    pub days_to_transition: Option<i64>,
    /// This date appears in the biorhythm engine's `critical_days`.
    pub critical_cycle_day: bool,
    pub transition_on_date: Option<TransitionMarker>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeeklyRiskLandscape {
    pub start_date: NaiveDate,
    /// Exactly 7 entries, `start_date ..= start_date + 6 days`.
    pub days: Vec<RiskDay>,
    /// Contributors held at their day-zero value across the window, because
    /// no source engine emits a forward series for them.
    pub held_constant: Vec<String>,
    /// True for the whole window when the transits engine reports Sade Sati
    /// active. Carried as context, never folded into the index.
    pub saturn_pressure: bool,
}

// ---------------------------------------------------------------------------
// Output 3 — Monthly Alignment Calendar
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalendarDay {
    pub date: NaiveDate,
    pub three_wave: Option<f64>,
    pub days_to_transition: Option<i64>,
    pub transition_on_date: Option<TransitionMarker>,
    pub critical_cycle_day: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonthlyAlignmentCalendar {
    pub year: i32,
    pub month: u32,
    /// One entry per civil day of the month containing `current_time`.
    pub days: Vec<CalendarDay>,
    pub transition_dates: Vec<NaiveDate>,
    pub critical_dates: Vec<NaiveDate>,
}

// ---------------------------------------------------------------------------
// Output 4 — Decision Ownership Reflection
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceReading {
    pub contributor: String,
    pub engine_id: String,
    pub observation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionOwnershipReflection {
    /// Free text supplied through `options.decision_context`.
    pub decision_context: Option<String>,
    /// One line per contributor that could be read, and nothing more.
    pub source_readings: Vec<SourceReading>,
    /// Named absences, so the practitioner sees what was not consulted.
    pub unconsulted: Vec<String>,
    /// Reported by the transits engine; context, not a verdict.
    pub period_quality: Option<String>,
    /// Fixed statement returning authorship.
    pub authorship: String,
    /// Self-inquiry lines. Every entry ends with a question mark.
    pub authorship_checks: Vec<String>,
}

// ---------------------------------------------------------------------------
// Top-level result
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinancialBiosensorResult {
    pub declaration: Declaration,
    pub daily_decision_index: DailyDecisionIndex,
    pub weekly_risk_landscape: WeeklyRiskLandscape,
    pub monthly_alignment_calendar: MonthlyAlignmentCalendar,
    pub decision_ownership_reflection: DecisionOwnershipReflection,
    pub provenance: ProvenanceBlock,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn declared_weights_sum_to_one() {
        let total: f64 = ContributorId::ALL.iter().map(|c| c.declared_weight()).sum();
        assert!((total - 1.0).abs() < 1e-9, "weights summed to {}", total);
    }

    #[test]
    fn contributor_slugs_are_unique_and_stable() {
        let mut slugs: Vec<&str> = ContributorId::ALL.iter().map(|c| c.as_str()).collect();
        slugs.sort_unstable();
        let before = slugs.len();
        slugs.dedup();
        assert_eq!(before, slugs.len(), "contributor slugs must be unique");
    }

    #[test]
    fn triad_legs_are_all_represented() {
        let legs: Vec<&str> = ContributorId::ALL.iter().map(|c| c.leg()).collect();
        for leg in ["Kha", "Ba", "La"] {
            assert!(legs.contains(&leg), "no contributor carries {}", leg);
        }
    }

    #[test]
    fn declaration_defaults_name_the_claim_mode() {
        let d = Declaration::default();
        assert_eq!(d.claim_mode, "house-model");
        assert_eq!(d.excluded_claim, "not-prediction");
        assert_eq!(d.formula_version, FORMULA_VERSION);
    }
}
