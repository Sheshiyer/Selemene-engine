//! Witness prompt generation.
//!
//! The prompt is a question, never an instruction, and it is built from how
//! much of the reading could actually be taken. A thin reading asks a
//! different question than a full one.

use crate::composite::Composite;
use crate::models::{Contributor, Sufficiency};

/// A single self-inquiry question sized to the reading that produced it.
pub fn generate_witness_prompt(composite: &Composite, contributors: &[Contributor]) -> String {
    let read = contributors
        .iter()
        .filter(|c| c.status.is_present())
        .count();
    let total = contributors.len();

    if composite.sufficiency == Sufficiency::Insufficient {
        return format!(
            "Only {} of {} sources could be read, so no index is offered here — what were you \
             hoping the number would settle for you?",
            read, total
        );
    }

    match composite.value {
        Some(value) => format!(
            "{} of {} sources were read and they compose to {:.3}; before you saw that, which way \
             were you already leaning, and what would it cost you to say so out loud?",
            read, total, value
        ),
        None => format!(
            "{} of {} sources were read and no index was produced — what would you decide if none \
             of them ever reported again?",
            read, total
        ),
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::composite::compose;
    use crate::models::{ContributorId, ContributorStatus};
    use serde_json::Value;

    fn contributor(id: ContributorId, normalized: Option<f64>) -> Contributor {
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

    fn assert_contract(prompt: &str) {
        assert!(prompt.ends_with('?'), "must be a question: {}", prompt);
        assert!(prompt.len() >= 24, "too short: {}", prompt);
        let lowered = prompt.to_ascii_lowercase();
        assert!(!lowered.contains("you should"), "instructs: {}", prompt);
        assert!(!lowered.contains("you must"), "instructs: {}", prompt);
    }

    #[test]
    fn a_full_reading_produces_a_conforming_question() {
        let set: Vec<Contributor> = ContributorId::ALL
            .iter()
            .map(|id| contributor(*id, Some(0.5)))
            .collect();
        let composite = compose(&set).unwrap();
        assert_contract(&generate_witness_prompt(&composite, &set));
    }

    #[test]
    fn a_thin_reading_produces_a_conforming_question() {
        let mut set: Vec<Contributor> = ContributorId::ALL
            .iter()
            .map(|id| contributor(*id, None))
            .collect();
        set[3] = contributor(ContributorId::ThreeWaveCycle, Some(0.9));
        let composite = compose(&set).unwrap();
        assert_eq!(composite.sufficiency, Sufficiency::Insufficient);
        let prompt = generate_witness_prompt(&composite, &set);
        assert_contract(&prompt);
        assert!(prompt.contains("no index is offered"));
    }
}
