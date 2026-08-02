//! The composite index: declared weights, renormalization over what could be
//! read, a sufficiency floor, and the confidence the engine reports about its
//! own number.
//!
//! Nothing here is fitted. The weights are house constants, and the whole
//! point of [`FORMULA_VERSION`](crate::models::FORMULA_VERSION) is that two
//! numbers produced under different constants are never silently compared.

use noesis_core::EngineError;
use std::collections::HashMap;

use crate::models::{
    Contributor, ContributorContribution, ContributorId, Convergence, Sufficiency,
};

/// Below this coverage the composite restates a single source, so the value is
/// withheld rather than published thin.
pub const SUFFICIENCY_FLOOR: f64 = 0.40;

/// Upper bound of the `Divergent` band.
pub const DIVERGENT_BELOW: f64 = 0.40;
/// Lower bound of the `Convergent` band.
pub const CONVERGENT_AT_OR_ABOVE: f64 = 0.60;

/// Declared confidence penalty applied when the biometric leg is unsampled.
///
/// This sits on top of the weight, because the absence of that contributor
/// removes the only present-tense measurement rather than merely a quarter of
/// the weight.
pub const BA_ABSENT_FACTOR: f64 = 0.8;

fn round3(v: f64) -> f64 {
    (v * 1000.0).round() / 1000.0
}

/// The outcome of composing whatever could be read.
#[derive(Debug, Clone)]
pub struct Composite {
    pub value: Option<f64>,
    pub convergence: Option<Convergence>,
    pub sufficiency: Sufficiency,
    pub coverage: f64,
    pub contributions: Vec<ContributorContribution>,
    /// Effective weight per contributor after renormalization; `0.0` for any
    /// contributor that could not be read.
    pub effective_weights: HashMap<ContributorId, f64>,
}

impl Composite {
    /// `coverage * ba_factor`, the confidence the engine reports about itself.
    pub fn confidence(&self, hrv_present: bool) -> f64 {
        (self.coverage * ba_factor(hrv_present)).clamp(0.0, 1.0)
    }
}

/// `1.0` when the biometric leg was sampled, [`BA_ABSENT_FACTOR`] otherwise.
pub fn ba_factor(hrv_present: bool) -> f64 {
    if hrv_present {
        1.0
    } else {
        BA_ABSENT_FACTOR
    }
}

pub fn convergence_for(value: f64) -> Convergence {
    if value < DIVERGENT_BELOW {
        Convergence::Divergent
    } else if value < CONVERGENT_AT_OR_ABOVE {
        Convergence::Partial
    } else {
        Convergence::Convergent
    }
}

/// Compose over the contributors as read.
pub fn compose(contributors: &[Contributor]) -> Result<Composite, EngineError> {
    let scores: Vec<(ContributorId, Option<f64>, String)> = contributors
        .iter()
        .map(|c| (c.id, c.normalized, c.observation.clone()))
        .collect();
    compose_scores(&scores)
}

/// Compose over the contributors with some sub-scores replaced — used to
/// recompute the index for a forward date, where only the day-varying
/// contributors move.
pub fn compose_with_overrides(
    contributors: &[Contributor],
    overrides: &HashMap<ContributorId, Option<f64>>,
) -> Result<Composite, EngineError> {
    let scores: Vec<(ContributorId, Option<f64>, String)> = contributors
        .iter()
        .map(|c| {
            let score = match overrides.get(&c.id) {
                // A contributor that was absent at day zero stays absent: an
                // override cannot conjure a source that could not be read.
                Some(v) if c.normalized.is_some() => *v,
                _ => c.normalized,
            };
            (c.id, score, c.observation.clone())
        })
        .collect();
    compose_scores(&scores)
}

fn compose_scores(
    scores: &[(ContributorId, Option<f64>, String)],
) -> Result<Composite, EngineError> {
    let coverage: f64 = scores
        .iter()
        .filter(|(_, v, _)| v.is_some())
        .map(|(id, _, _)| id.declared_weight())
        .sum();

    if coverage <= 0.0 {
        return Err(EngineError::CalculationError(
            "no contributor could be read; the composite has nothing to report".to_string(),
        ));
    }

    let mut effective_weights: HashMap<ContributorId, f64> = HashMap::new();
    let mut contributions = Vec::new();
    let mut accumulated = 0.0_f64;

    for (id, score, observation) in scores {
        match score {
            Some(normalized) => {
                let weight = id.declared_weight() / coverage;
                effective_weights.insert(*id, weight);
                accumulated += weight * normalized;
                contributions.push(ContributorContribution {
                    contributor: id.as_str().to_string(),
                    normalized: *normalized,
                    weight_effective: weight,
                    share: weight * normalized,
                    observation: observation.clone(),
                });
            }
            None => {
                effective_weights.insert(*id, 0.0);
            }
        }
    }

    let sufficient = coverage >= SUFFICIENCY_FLOOR;
    let value = if sufficient {
        Some(round3(accumulated))
    } else {
        None
    };

    Ok(Composite {
        value,
        convergence: value.map(convergence_for),
        sufficiency: if sufficient {
            Sufficiency::Sufficient
        } else {
            Sufficiency::Insufficient
        },
        coverage: round3(coverage),
        contributions,
        effective_weights,
    })
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::models::ContributorStatus;
    use serde_json::Value;

    fn c(id: ContributorId, normalized: Option<f64>) -> Contributor {
        Contributor {
            id,
            engine_id: "test".to_string(),
            engine_version: "0".to_string(),
            fields_consumed: vec!["/test".to_string()],
            normalization: "test".to_string(),
            raw: Value::Null,
            normalized,
            status: if normalized.is_some() {
                ContributorStatus::Present
            } else {
                ContributorStatus::absent("test", "test")
            },
            observation: "test".to_string(),
        }
    }

    /// The five worked degradation cases from the design.
    #[test]
    fn birth_data_only_covers_fifty_five_percent() {
        let set = vec![
            c(ContributorId::HeartRateVariability, None),
            c(ContributorId::Chronofield, Some(0.5)),
            c(ContributorId::ActivePlanetaryWeather, Some(0.5)),
            c(ContributorId::ThreeWaveCycle, Some(0.5)),
            c(ContributorId::EnergeticAuthority, None),
            c(ContributorId::GiftShadowSpectrum, None),
        ];
        let composite = compose(&set).unwrap();
        assert_eq!(composite.coverage, 0.55);
        assert!((composite.confidence(false) - 0.44).abs() < 1e-9);
        assert!(composite.value.is_some());
    }

    #[test]
    fn declaring_deliberation_raises_coverage_to_sixty_seven() {
        let set = vec![
            c(ContributorId::HeartRateVariability, None),
            c(ContributorId::Chronofield, Some(0.5)),
            c(ContributorId::ActivePlanetaryWeather, Some(0.5)),
            c(ContributorId::ThreeWaveCycle, Some(0.5)),
            c(ContributorId::EnergeticAuthority, Some(0.5)),
            c(ContributorId::GiftShadowSpectrum, None),
        ];
        let composite = compose(&set).unwrap();
        assert_eq!(composite.coverage, 0.67);
        assert!((composite.confidence(false) - 0.536).abs() < 1e-9);
    }

    #[test]
    fn sampling_the_body_raises_coverage_to_ninety_two_and_lifts_the_factor() {
        let set = vec![
            c(ContributorId::HeartRateVariability, Some(0.5)),
            c(ContributorId::Chronofield, Some(0.5)),
            c(ContributorId::ActivePlanetaryWeather, Some(0.5)),
            c(ContributorId::ThreeWaveCycle, Some(0.5)),
            c(ContributorId::EnergeticAuthority, Some(0.5)),
            c(ContributorId::GiftShadowSpectrum, None),
        ];
        let composite = compose(&set).unwrap();
        assert_eq!(composite.coverage, 0.92);
        assert!((composite.confidence(true) - 0.92).abs() < 1e-9);
    }

    #[test]
    fn all_six_present_reaches_full_coverage_and_confidence() {
        let set: Vec<Contributor> = ContributorId::ALL
            .iter()
            .map(|id| c(*id, Some(0.5)))
            .collect();
        let composite = compose(&set).unwrap();
        assert_eq!(composite.coverage, 1.0);
        assert!((composite.confidence(true) - 1.0).abs() < 1e-9);
        assert_eq!(composite.value, Some(0.5));
    }

    #[test]
    fn below_the_floor_the_value_is_withheld() {
        let set = vec![
            c(ContributorId::HeartRateVariability, None),
            c(ContributorId::Chronofield, None),
            c(ContributorId::ActivePlanetaryWeather, None),
            c(ContributorId::ThreeWaveCycle, Some(0.9)),
            c(ContributorId::EnergeticAuthority, None),
            c(ContributorId::GiftShadowSpectrum, None),
        ];
        let composite = compose(&set).unwrap();
        assert_eq!(composite.coverage, 0.15);
        assert_eq!(composite.value, None);
        assert_eq!(composite.convergence, None);
        assert_eq!(composite.sufficiency, Sufficiency::Insufficient);
        assert!((composite.confidence(false) - 0.12).abs() < 1e-9);
    }

    #[test]
    fn effective_weights_over_present_contributors_sum_to_one() {
        let set = vec![
            c(ContributorId::HeartRateVariability, None),
            c(ContributorId::Chronofield, Some(0.3)),
            c(ContributorId::ActivePlanetaryWeather, Some(0.7)),
            c(ContributorId::ThreeWaveCycle, Some(0.5)),
            c(ContributorId::EnergeticAuthority, Some(0.1)),
            c(ContributorId::GiftShadowSpectrum, None),
        ];
        let composite = compose(&set).unwrap();
        let total: f64 = composite
            .contributions
            .iter()
            .map(|c| c.weight_effective)
            .sum();
        assert!((total - 1.0).abs() < 1e-9, "weights summed to {}", total);
    }

    #[test]
    fn no_contributor_at_all_is_an_error() {
        let set: Vec<Contributor> = ContributorId::ALL.iter().map(|id| c(*id, None)).collect();
        assert!(compose(&set).is_err());
    }

    #[test]
    fn convergence_bands_are_declared_not_instructions() {
        assert_eq!(convergence_for(0.39), Convergence::Divergent);
        assert_eq!(convergence_for(0.40), Convergence::Partial);
        assert_eq!(convergence_for(0.59), Convergence::Partial);
        assert_eq!(convergence_for(0.60), Convergence::Convergent);
    }

    #[test]
    fn an_override_cannot_revive_an_absent_contributor() {
        let set = vec![
            c(ContributorId::HeartRateVariability, None),
            c(ContributorId::Chronofield, Some(0.5)),
            c(ContributorId::ActivePlanetaryWeather, Some(0.5)),
            c(ContributorId::ThreeWaveCycle, Some(0.5)),
            c(ContributorId::EnergeticAuthority, None),
            c(ContributorId::GiftShadowSpectrum, None),
        ];
        let mut overrides = HashMap::new();
        overrides.insert(ContributorId::HeartRateVariability, Some(1.0));
        let composite = compose_with_overrides(&set, &overrides).unwrap();
        assert_eq!(composite.coverage, 0.55);
    }

    #[test]
    fn an_override_moves_a_present_contributor() {
        let set = vec![
            c(ContributorId::HeartRateVariability, None),
            c(ContributorId::Chronofield, Some(0.0)),
            c(ContributorId::ActivePlanetaryWeather, Some(0.0)),
            c(ContributorId::ThreeWaveCycle, Some(0.0)),
            c(ContributorId::EnergeticAuthority, None),
            c(ContributorId::GiftShadowSpectrum, None),
        ];
        let base = compose(&set).unwrap();
        let mut overrides = HashMap::new();
        overrides.insert(ContributorId::ThreeWaveCycle, Some(1.0));
        let moved = compose_with_overrides(&set, &overrides).unwrap();
        assert_ne!(base.value, moved.value);
    }
}
