//! The reflection surface: what each source registered, what was not
//! consulted, and the questions that hand the decision back.
//!
//! Nothing here instructs. Each line reports a reading or names an absence.

use serde_json::Value;
use std::collections::HashMap;

use crate::models::{
    Contributor, ContributorStatus, DecisionOwnershipReflection, SourceReading,
    AUTHORSHIP_STATEMENT,
};

pub fn decision_ownership_reflection(
    contributors: &[Contributor],
    options: &HashMap<String, Value>,
    period_quality: Option<String>,
) -> DecisionOwnershipReflection {
    let decision_context = options
        .get("decision_context")
        .and_then(Value::as_str)
        .map(str::to_string);

    let mut source_readings = Vec::new();
    let mut unconsulted = Vec::new();

    for contributor in contributors {
        match &contributor.status {
            ContributorStatus::Present => source_readings.push(SourceReading {
                contributor: contributor.id.as_str().to_string(),
                engine_id: contributor.engine_id.clone(),
                observation: contributor.observation.clone(),
            }),
            ContributorStatus::Absent { detail, .. } => {
                unconsulted.push(format!("{} — {}", contributor.id.as_str(), detail));
            }
        }
    }

    DecisionOwnershipReflection {
        decision_context,
        source_readings,
        unconsulted,
        period_quality,
        authorship: AUTHORSHIP_STATEMENT.to_string(),
        authorship_checks: authorship_checks(contributors),
    }
}

/// Self-inquiry lines, chosen by which legs of the triad were readable.
///
/// The absence of a leg is stated rather than absorbed: if the body was not
/// sampled, the practitioner is asked about the body directly.
fn authorship_checks(contributors: &[Contributor]) -> Vec<String> {
    let present = |leg: &str| {
        contributors
            .iter()
            .any(|c| c.id.leg() == leg && c.status.is_present())
    };

    let mut checks = vec![
        "Before this reading, which way were you already leaning?".to_string(),
        "What would it cost you to say that out loud, to someone who would remember?".to_string(),
    ];

    if !present("Ba") {
        checks.push(
            "The body was not sampled for this reading. What is it reporting that no engine saw?"
                .to_string(),
        );
    }
    if !present("La") {
        checks.push(
            "No deliberation was declared. How long have you actually been sitting with this?"
                .to_string(),
        );
    }
    if present("Ba") && present("La") {
        checks.push(
            "All three legs read together here. Which of them are you most tempted to discount?"
                .to_string(),
        );
    }

    checks.push("If every engine here went quiet tomorrow, what would you decide?".to_string());
    checks
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::ContributorId;
    use serde_json::json;

    fn contributor(id: ContributorId, present: bool) -> Contributor {
        Contributor {
            id,
            engine_id: "test".to_string(),
            engine_version: "0".to_string(),
            fields_consumed: vec!["/test".to_string()],
            normalization: "test".to_string(),
            raw: Value::Null,
            normalized: if present { Some(0.5) } else { None },
            status: if present {
                ContributorStatus::Present
            } else {
                ContributorStatus::absent("test", "nothing was supplied")
            },
            observation: "a reading".to_string(),
        }
    }

    #[test]
    fn every_check_is_a_question() {
        let set: Vec<Contributor> = ContributorId::ALL
            .iter()
            .map(|id| contributor(*id, true))
            .collect();
        let r = decision_ownership_reflection(&set, &HashMap::new(), None);
        assert!(!r.authorship_checks.is_empty());
        for check in &r.authorship_checks {
            assert!(check.ends_with('?'), "not a question: {}", check);
        }
    }

    #[test]
    fn an_unsampled_body_is_named_rather_than_absorbed() {
        let set: Vec<Contributor> = ContributorId::ALL
            .iter()
            .map(|id| contributor(*id, *id != ContributorId::HeartRateVariability))
            .collect();
        let r = decision_ownership_reflection(&set, &HashMap::new(), None);
        assert!(r
            .authorship_checks
            .iter()
            .any(|c| c.contains("body was not sampled")));
        assert!(r
            .unconsulted
            .iter()
            .any(|u| u.starts_with("heart_rate_variability")));
    }

    #[test]
    fn present_sources_are_reported_and_absences_are_listed_separately() {
        let set = vec![
            contributor(ContributorId::ThreeWaveCycle, true),
            contributor(ContributorId::HeartRateVariability, false),
        ];
        let r = decision_ownership_reflection(&set, &HashMap::new(), Some("Mixed".to_string()));
        assert_eq!(r.source_readings.len(), 1);
        assert_eq!(r.unconsulted.len(), 1);
        assert_eq!(r.period_quality.as_deref(), Some("Mixed"));
        assert_eq!(r.authorship, AUTHORSHIP_STATEMENT);
    }

    #[test]
    fn decision_context_is_carried_through_verbatim() {
        let opts: HashMap<String, Value> = [(
            "decision_context".to_string(),
            json!("whether to take the contract"),
        )]
        .into_iter()
        .collect();
        let set = vec![contributor(ContributorId::ThreeWaveCycle, true)];
        let r = decision_ownership_reflection(&set, &opts, None);
        assert_eq!(
            r.decision_context.as_deref(),
            Some("whether to take the contract")
        );
    }
}
