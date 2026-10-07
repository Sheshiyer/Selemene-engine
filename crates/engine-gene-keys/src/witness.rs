//! Source-bound Gene Keys witness inquiries.
//!
//! Service configuration chooses a reflection lens, not a measured user state.
//! Canonical spheres bind to supplied activations; absent lines stay unavailable.

use crate::models::{CanonicalActivationSphere, GeneKeysChart};

fn describe_sphere(name: &str, sphere: &Option<CanonicalActivationSphere>) -> String {
    match sphere {
        Some(activation) => {
            let line = activation
                .line
                .map(|line| format!("line {line}"))
                .unwrap_or_else(|| "line unavailable (gate-only input)".to_string());
            format!(
                "{name}: Gene Key {} {line} ({:?})",
                activation.key_number, activation.source
            )
        }
        None => format!("{name}: source activation unavailable"),
    }
}

/// Generate an inquiry from canonical sphere coordinates.
/// `consciousness_level` is a configured lens, never an assessment of the user.
/// This preserves the existing public function signature.
pub fn generate_witness_prompt(chart: &GeneKeysChart, consciousness_level: u8) -> String {
    let (lens, inquiry) = match consciousness_level {
        0..=2 => ("Shadow recognition", "Which unconscious patterns, if any, do you recognize when reflecting on these symbolic coordinates?"),
        5..=6 => ("Siddhi contemplation", "What transcendent or beyond-personal perspective, if any, feels useful to contemplate with these symbolic coordinates?"),
        _ => ("Gift exploration", "Which gifts or authentic actions, if any, are useful to contemplate alongside these symbolic coordinates?"),
    };
    let spheres = chart.activation_spheres();
    format!(
        "Reflection lens: {lens} (configured service setting; not a measurement of user consciousness or frequency). {}. {}. {}. {}. Framework records may be incomplete; these coordinates do not establish a complete wisdom reading. {inquiry}",
        describe_sphere("Life's Work", &spheres.lifes_work),
        describe_sphere("Evolution", &spheres.evolution),
        describe_sphere("Radiance", &spheres.radiance),
        describe_sphere("Purpose", &spheres.purpose),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{ActivationSequence, ActivationSource, GeneKeyActivation};

    fn create_test_chart() -> GeneKeysChart {
        GeneKeysChart {
            activation_sequence: ActivationSequence {
                lifes_work: (17, 18),
                evolution: (45, 26),
                radiance: (17, 45),
                purpose: (18, 26),
            },
            active_keys: vec![
                GeneKeyActivation {
                    key_number: 17,
                    line: 1,
                    source: ActivationSource::PersonalitySun,
                    gene_key_data: None,
                },
                GeneKeyActivation {
                    key_number: 18,
                    line: 2,
                    source: ActivationSource::PersonalityEarth,
                    gene_key_data: None,
                },
                GeneKeyActivation {
                    key_number: 45,
                    line: 4,
                    source: ActivationSource::DesignSun,
                    gene_key_data: None,
                },
                GeneKeyActivation {
                    key_number: 26,
                    line: 6,
                    source: ActivationSource::DesignEarth,
                    gene_key_data: None,
                },
            ],
        }
    }

    #[test]
    fn test_shadow_prompt_level_0() {
        let chart = create_test_chart();
        let prompt = generate_witness_prompt(&chart, 0);

        assert!(!prompt.is_empty(), "Prompt should not be empty");
        assert!(
            prompt.contains("unconscious"),
            "Shadow prompt should mention 'unconscious'"
        );
        assert!(
            prompt.contains("17") || prompt.contains("18"),
            "Should reference Gene Key numbers"
        );
    }

    #[test]
    fn test_shadow_prompt_level_2() {
        let chart = create_test_chart();
        let prompt = generate_witness_prompt(&chart, 2);

        assert!(!prompt.is_empty(), "Prompt should not be empty");
        assert!(prompt.contains("?"), "Should be inquiry format (question)");
    }

    #[test]
    fn test_gift_prompt_level_3() {
        let chart = create_test_chart();
        let prompt = generate_witness_prompt(&chart, 3);

        assert!(!prompt.is_empty(), "Prompt should not be empty");
        assert!(prompt.contains("?"), "Should be inquiry format (question)");
        assert!(
            prompt.contains("17") || prompt.contains("45"),
            "Should reference Radiance keys"
        );
    }

    #[test]
    fn test_gift_prompt_level_4() {
        let chart = create_test_chart();
        let prompt = generate_witness_prompt(&chart, 4);

        assert!(!prompt.is_empty(), "Prompt should not be empty");
        assert!(
            prompt.contains("authentic") || prompt.contains("gift"),
            "Gift prompt should mention gifts or authenticity"
        );
    }

    #[test]
    fn test_siddhi_prompt_level_5() {
        let chart = create_test_chart();
        let prompt = generate_witness_prompt(&chart, 5);

        assert!(!prompt.is_empty(), "Prompt should not be empty");
        assert!(
            prompt.contains("transcendent") || prompt.contains("beyond"),
            "Siddhi prompt should be transcendent"
        );
        assert!(
            prompt.contains("18") || prompt.contains("26"),
            "Should reference Purpose keys"
        );
    }

    #[test]
    fn test_siddhi_prompt_level_6() {
        let chart = create_test_chart();
        let prompt = generate_witness_prompt(&chart, 6);

        assert!(!prompt.is_empty(), "Prompt should not be empty");
        assert!(prompt.contains("?"), "Should be inquiry format (question)");
    }

    #[test]
    fn test_default_to_gift_for_invalid_level() {
        let chart = create_test_chart();
        let prompt_invalid = generate_witness_prompt(&chart, 10);
        let prompt_gift = generate_witness_prompt(&chart, 3);

        // Both should be non-empty and inquiry format
        assert!(!prompt_invalid.is_empty());
        assert!(!prompt_gift.is_empty());
        assert!(prompt_invalid.contains("?"));
    }

    #[test]
    fn test_all_prompts_reference_gene_keys() {
        let chart = create_test_chart();

        for level in 0..=6 {
            let prompt = generate_witness_prompt(&chart, level);

            // Should reference at least one Gene Key number
            let has_key_reference = prompt.contains("17")
                || prompt.contains("18")
                || prompt.contains("45")
                || prompt.contains("26");

            assert!(
                has_key_reference,
                "Level {} prompt should reference Gene Key numbers: {}",
                level, prompt
            );
        }
    }

    #[test]
    fn test_all_prompts_are_questions() {
        let chart = create_test_chart();

        for level in 0..=6 {
            let prompt = generate_witness_prompt(&chart, level);
            assert!(
                prompt.contains("?"),
                "Level {} prompt should be inquiry format (contain '?'): {}",
                level,
                prompt
            );
        }
    }

    #[test]
    fn canonical_witness_uses_each_source_binding() {
        let prompt = generate_witness_prompt(&create_test_chart(), 3);
        for expected in [
            "Life's Work: Gene Key 17 line 1 (PersonalitySun)",
            "Evolution: Gene Key 18 line 2 (PersonalityEarth)",
            "Radiance: Gene Key 45 line 4 (DesignSun)",
            "Purpose: Gene Key 26 line 6 (DesignEarth)",
        ] {
            assert!(prompt.contains(expected), "missing {expected}: {prompt}");
        }
        assert!(prompt.contains("not a measurement of user consciousness or frequency"));
        assert!(!prompt.contains("User consciousness level measured"));
    }

    #[test]
    fn gate_only_witness_does_not_invent_line() {
        let mut chart = create_test_chart();
        for activation in &mut chart.active_keys {
            activation.line = 0;
        }
        let prompt = generate_witness_prompt(&chart, 3);
        assert_eq!(prompt.matches("line unavailable").count(), 4);
        assert!(!prompt.contains("line 0"));
        assert!(!prompt.contains("line 3"));
    }

    #[test]
    fn legacy_pairs_alone_do_not_supply_canonical_spheres() {
        let mut chart = create_test_chart();
        chart.active_keys.clear();
        let prompt = generate_witness_prompt(&chart, 3);
        assert_eq!(prompt.matches("source activation unavailable").count(), 4);
        assert!(!prompt.contains("Gene Key 17"));
    }
}
