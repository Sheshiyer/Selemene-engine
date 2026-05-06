//! Witness prompt generation for biofield awareness
//!
//! Generates non-prescriptive somatic awareness prompts based on biofield analysis.
//! Includes the Witness Dyad: Aletheios (left pillar — truth/integration) and
//! Pichet (right pillar — vitality/action). Prompts invite self-inquiry without
//! diagnosing or prescribing.

use crate::models::{BiofieldAnalysis, BiofieldMetrics, Chakra, ChakraReading};

/// Structured Witness Dyad output — Aletheios and Pichet perspectives
pub struct WitnessDyad {
    pub aletheios: String,
    pub pichet: String,
    pub synthesis: String,
}

/// Generate the Witness Dyad from biofield analysis.
///
/// - Aletheios (left pillar): truth-revealing, contemplative, integrative.
///   Simplifies when overwhelmed; invites rest and unconcealment.
/// - Pichet (right pillar): vitalizing, action-oriented.
///   Energizes when stagnant; invites embodiment and forward movement.
/// - Synthesis: a unifying question that holds both pillars.
pub fn generate_witness_dyad(analysis: &BiofieldAnalysis) -> WitnessDyad {
    let m = &analysis.metrics;

    let aletheios = generate_aletheios(m);
    let pichet = generate_pichet(m);
    let synthesis = generate_synthesis(m);

    WitnessDyad {
        aletheios,
        pichet,
        synthesis,
    }
}

/// Aletheios — the Left Pillar. Truth-revealing, contemplative, integrative.
/// Speaks from stillness. Simplifies when overwhelmed.
fn generate_aletheios(m: &BiofieldMetrics) -> String {
    // High coherence → naming the quality of alignment
    if m.coherence > 0.72 {
        return format!(
            "Something has settled in your field — coherence at {:.0}%. \
             What is the name of this quality? Not what caused it, not what to do with it — \
             simply: what is it? Let that answer be simple.",
            m.coherence * 100.0
        );
    }

    // Low vitality → rest and witness (not fix)
    if m.vitality_index < 0.38 {
        return format!(
            "Vitality is quiet right now — {:.0}% of its peak. \
             Aletheios does not ask you to restore it. \
             What does this quietness actually feel like, beneath the story of what it means?",
            m.vitality_index * 100.0
        );
    }

    // Low entropy → stillness / stagnation
    if m.entropy < 0.35 {
        return "The field is still. Stillness is not the same as absence. \
                What is present in this stillness that you have not yet named?"
            .to_string();
    }

    // Asymmetry → truth on the neglected side
    if m.symmetry < 0.48 {
        return "One side of your field is quieter than the other. \
                That quieter side is not broken — it is waiting. \
                What has it been waiting to say?"
            .to_string();
    }

    // Low coherence → fragmentation without judgment
    if m.coherence < 0.42 {
        return "The field is distributed — scattered across many frequencies. \
                Aletheios asks: what is the one true thing underneath all of this fragmentation? \
                Not a solution. A fact."
            .to_string();
    }

    // Default — generic contemplative truth inquiry
    "What does your body already know that your mind has not yet accepted? \
     Sit with that. No interpretation needed."
        .to_string()
}

/// Pichet — the Right Pillar. Vitalizing, action-oriented, forward-moving.
/// Speaks from aliveness. Energizes when stagnant.
fn generate_pichet(m: &BiofieldMetrics) -> String {
    // High vitality → channel the aliveness
    if m.vitality_index > 0.72 {
        let highest_chakra = highest_active_chakra_name(m);
        return format!(
            "Your vitality is at {:.0}% — the field is active. \
             The {} center is leading. What wants to move through you right now? \
             Do not think about it. Let it move.",
            m.vitality_index * 100.0,
            highest_chakra
        );
    }

    // High entropy → dynamic pattern, ride it
    if m.entropy > 0.65 {
        return format!(
            "The field is dynamic — entropy at {:.0}%. Pichet sees this as aliveness, not chaos. \
             What one action, even tiny, would honor the energy that is moving right now?",
            m.entropy * 100.0
        );
    }

    // Low vitality → Pichet vitalizes — introduces movement
    if m.vitality_index < 0.42 {
        return "Pichet does not ask you to force energy that isn't there. \
                But: what small, physical act — a breath, a sound, a shift in posture — \
                would be the most honest response to this moment?"
            .to_string();
    }

    // Low coherence → one point of action
    if m.coherence < 0.45 {
        return "When the field is scattered, Pichet simplifies the question: \
                of everything you are holding right now, what is the one thing \
                that — if you let it go — would allow everything else to settle?"
            .to_string();
    }

    // High symmetry + coherence → integration in motion
    if m.symmetry > 0.7 && m.coherence > 0.6 {
        return format!(
            "Symmetry at {:.0}%, coherence at {:.0}%. \
             The field is balanced and awake. Pichet asks: what are you building with this? \
             Aliveness wants direction.",
            m.symmetry * 100.0,
            m.coherence * 100.0
        );
    }

    // Default — vitalizing action prompt
    "Pichet asks one question: what is your body asking to do right now, \
     not what it 'should' do? Follow that impulse for ten seconds. \
     Notice what shifts."
        .to_string()
}

/// Synthesis — the unifying question that holds both Aletheios and Pichet.
fn generate_synthesis(m: &BiofieldMetrics) -> String {
    // High coherence + high vitality → full integration moment
    if m.coherence > 0.65 && m.vitality_index > 0.65 {
        return "Both pillars are lit. The field is coherent and alive. \
                What is ready to be born from this convergence?"
            .to_string();
    }

    // Low coherence + low vitality → gentle holding
    if m.coherence < 0.4 && m.vitality_index < 0.4 {
        return "The field is asking for rest and truth at once. \
                The synthesis question: what would full permission feel like, right now?"
            .to_string();
    }

    // High coherence, low vitality → integration without forcing
    if m.coherence > 0.6 && m.vitality_index < 0.45 {
        return "Something is integrated but not yet animated. \
                The synthesis: what does this knowing want to become in the world?"
            .to_string();
    }

    // Low coherence, high vitality → grounding the energy
    if m.coherence < 0.45 && m.vitality_index > 0.6 {
        return "Energy is present but the field is scattered. \
                The synthesis: what container would let this aliveness go deeper, rather than wider?"
            .to_string();
    }

    // Default synthesis
    "Between unconcealment and aliveness — what is the quality of this moment, \
     exactly as it is?"
        .to_string()
}

/// Returns the name of the most active chakra center from biofield metrics.
fn highest_active_chakra_name(m: &BiofieldMetrics) -> &'static str {
    m.chakra_readings
        .iter()
        .max_by(|a, b| a.activity_level.partial_cmp(&b.activity_level).unwrap())
        .map(|r| r.chakra.name())
        .unwrap_or("heart")
}

/// Generate witness prompts based on biofield analysis
///
/// Returns 3-4 non-prescriptive prompts for somatic awareness
pub fn generate_witness_prompts(analysis: &BiofieldAnalysis) -> Vec<String> {
    let mut prompts = Vec::new();

    // Always include a general body awareness prompt
    prompts.push(generate_general_awareness_prompt());

    // Add prompt based on overall metrics
    prompts.push(generate_metrics_prompt(&analysis.metrics));

    // Add chakra-specific prompt if there's notable activity
    if let Some(chakra_prompt) = generate_chakra_prompt(&analysis.metrics.chakra_readings) {
        prompts.push(chakra_prompt);
    }

    // Add a closing integrative prompt
    prompts.push(generate_integrative_prompt(&analysis.metrics));

    prompts
}

/// Generate a single witness prompt string for engine output
pub fn generate_witness_prompt(analysis: &BiofieldAnalysis) -> String {
    let prompts = generate_witness_prompts(analysis);
    prompts.join(" ")
}

/// General body awareness prompt
fn generate_general_awareness_prompt() -> String {
    "What sensations do you notice in your body right now? \
    Without trying to change anything, simply observe what is present."
        .to_string()
}

/// Generate prompt based on overall biofield metrics
fn generate_metrics_prompt(metrics: &BiofieldMetrics) -> String {
    if metrics.coherence < 0.4 {
        "Where in your body do you feel scattered or fragmented? \
         What would it be like to simply witness this without needing to fix it?"
            .to_string()
    } else if metrics.coherence > 0.7 {
        "Notice the sense of alignment in your body. \
         Where do you feel the most centered? What quality does this centeredness have?"
            .to_string()
    } else if metrics.symmetry < 0.5 {
        "Bring attention to both sides of your body. \
         Do you notice any differences between left and right? \
         What might each side be expressing?"
            .to_string()
    } else if metrics.entropy < 0.35 {
        "Where do you feel energy moving or flowing in your body? \
         Where does it feel still or stagnant? What is the quality of each?"
            .to_string()
    } else {
        "What does your breath tell you about your current state? \
         Without controlling it, notice its rhythm, depth, and quality."
            .to_string()
    }
}

/// Generate prompt based on chakra readings
fn generate_chakra_prompt(readings: &[ChakraReading]) -> Option<String> {
    // Find the chakra with lowest activity
    let lowest = readings
        .iter()
        .min_by(|a, b| a.activity_level.partial_cmp(&b.activity_level).unwrap())?;

    // Find the chakra with highest activity
    let highest = readings
        .iter()
        .max_by(|a, b| a.activity_level.partial_cmp(&b.activity_level).unwrap())?;

    // Find most imbalanced chakra
    let most_imbalanced = readings
        .iter()
        .max_by(|a, b| a.balance.abs().partial_cmp(&b.balance.abs()).unwrap())?;

    // Generate prompt based on patterns
    if lowest.activity_level < 0.4 {
        Some(generate_low_activity_prompt(lowest.chakra))
    } else if highest.activity_level > 0.8 {
        Some(generate_high_activity_prompt(highest.chakra))
    } else if most_imbalanced.balance.abs() > 0.4 {
        Some(generate_balance_prompt(most_imbalanced))
    } else {
        // Return a general chakra awareness prompt
        Some(
            "If you bring attention to your spine, from its base to your crown, \
              what do you notice along this central axis? \
              Where does your attention naturally rest?"
                .to_string(),
        )
    }
}

/// Generate prompt for low activity chakra
fn generate_low_activity_prompt(chakra: Chakra) -> String {
    let location = match chakra {
        Chakra::Root => "the base of your spine and pelvic floor",
        Chakra::Sacral => "your lower belly, below your navel",
        Chakra::SolarPlexus => "your solar plexus, the area around your stomach",
        Chakra::Heart => "the center of your chest",
        Chakra::Throat => "your throat and neck",
        Chakra::ThirdEye => "the space between your eyebrows",
        Chakra::Crown => "the top of your head",
    };

    format!(
        "Bring gentle attention to {}. \
         What sensations are present there? \
         If this area could speak, what might it want to express?",
        location
    )
}

/// Generate prompt for high activity chakra
fn generate_high_activity_prompt(chakra: Chakra) -> String {
    let location = match chakra {
        Chakra::Root => "the base of your spine",
        Chakra::Sacral => "your lower abdomen",
        Chakra::SolarPlexus => "your solar plexus area",
        Chakra::Heart => "your heart center",
        Chakra::Throat => "your throat",
        Chakra::ThirdEye => "the area between your eyebrows",
        Chakra::Crown => "the crown of your head",
    };

    format!(
        "Notice the energy you feel in {}. \
         What is the quality of this aliveness? \
         How does it want to move or express itself?",
        location
    )
}

/// Generate prompt for imbalanced chakra
fn generate_balance_prompt(reading: &ChakraReading) -> String {
    let chakra_name = reading.chakra.name();
    let side = if reading.balance > 0.0 {
        "right"
    } else {
        "left"
    };
    let other_side = if reading.balance > 0.0 {
        "left"
    } else {
        "right"
    };

    format!(
        "In the {} area, notice if there's any difference between {} and {}. \
         What quality does each side carry? \
         What might be inviting more balance?",
        chakra_name, side, other_side
    )
}

/// Generate integrative closing prompt
fn generate_integrative_prompt(metrics: &BiofieldMetrics) -> String {
    if metrics.vitality_index > 0.7 {
        "As you hold awareness of your whole body, \
         what feels most alive? What is your body celebrating right now?"
            .to_string()
    } else if metrics.vitality_index < 0.4 {
        "What does your body need most right now? \
         Not what you think it should need, but what it is actually asking for. \
         Listen without agenda."
            .to_string()
    } else {
        "Taking in your body as a whole, \
         what is the overall felt sense? \
         If you could describe it in one word or image, what would it be?"
            .to_string()
    }
}

/// Template prompts that can be used regardless of analysis
pub const SOMATIC_AWARENESS_TEMPLATES: &[&str] = &[
    "What sensations do you notice in your body right now?",
    "Where do you feel energy moving or stuck?",
    "What does your breath tell you about your current state?",
    "If your body could speak, what would it say?",
    "Where do you feel most alive right now?",
    "What part of your body is calling for attention?",
    "How does the space around your body feel?",
    "What is the quality of stillness in your body?",
    "Where do you hold tension, and what is it guarding?",
    "What would full permission feel like in your body?",
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock::generate_mock_metrics;
    use crate::models::BiofieldAnalysis;

    fn create_test_analysis() -> BiofieldAnalysis {
        BiofieldAnalysis {
            metrics: generate_mock_metrics(Some(42)),
            interpretation: "Test interpretation".to_string(),
            areas_of_attention: vec!["Test area".to_string()],
            is_mock_data: true,
        }
    }

    #[test]
    fn test_generate_witness_prompts() {
        let analysis = create_test_analysis();
        let prompts = generate_witness_prompts(&analysis);

        assert!(prompts.len() >= 3, "Should generate at least 3 prompts");
        assert!(prompts.len() <= 4, "Should generate at most 4 prompts");

        for prompt in &prompts {
            assert!(!prompt.is_empty(), "Prompts should not be empty");
            assert!(prompt.contains("?"), "Prompts should be questions");
        }
    }

    #[test]
    fn test_generate_witness_prompt_single() {
        let analysis = create_test_analysis();
        let prompt = generate_witness_prompt(&analysis);

        assert!(!prompt.is_empty());
        assert!(prompt.contains("?"));
    }

    #[test]
    fn test_prompts_are_non_prescriptive() {
        let analysis = create_test_analysis();
        let prompts = generate_witness_prompts(&analysis);

        for prompt in &prompts {
            // Should not contain prescriptive language
            let prescriptive_words = ["should", "must", "need to", "have to", "try to"];
            for word in prescriptive_words {
                assert!(
                    !prompt.to_lowercase().contains(word) || prompt.contains("what it should"),
                    "Prompt should not contain prescriptive word '{}': {}",
                    word,
                    prompt
                );
            }
        }
    }

    #[test]
    fn test_prompts_vary_with_metrics() {
        // High coherence metrics
        let mut high_coherence = generate_mock_metrics(Some(42));
        high_coherence.coherence = 0.85;
        let analysis1 = BiofieldAnalysis {
            metrics: high_coherence,
            interpretation: String::new(),
            areas_of_attention: vec![],
            is_mock_data: true,
        };

        // Low coherence metrics
        let mut low_coherence = generate_mock_metrics(Some(42));
        low_coherence.coherence = 0.25;
        let analysis2 = BiofieldAnalysis {
            metrics: low_coherence,
            interpretation: String::new(),
            areas_of_attention: vec![],
            is_mock_data: true,
        };

        let prompts1 = generate_witness_prompts(&analysis1);
        let prompts2 = generate_witness_prompts(&analysis2);

        // Should have different metric-based prompts
        assert_ne!(
            prompts1[1], prompts2[1],
            "Different metrics should yield different prompts"
        );
    }

    #[test]
    fn test_chakra_prompts_reference_body_areas() {
        let body_terms = ["spine", "chest", "throat", "belly", "head", "brow"];

        // Generate prompts for different chakra configurations
        let mut found_body_reference = false;
        for seed in 0..10 {
            let analysis = BiofieldAnalysis {
                metrics: generate_mock_metrics(Some(seed)),
                interpretation: String::new(),
                areas_of_attention: vec![],
                is_mock_data: true,
            };
            let prompt = generate_witness_prompt(&analysis);

            for term in body_terms {
                if prompt.to_lowercase().contains(term) {
                    found_body_reference = true;
                    break;
                }
            }
        }

        assert!(
            found_body_reference,
            "Some prompts should reference body areas"
        );
    }

    #[test]
    fn test_template_prompts() {
        assert!(!SOMATIC_AWARENESS_TEMPLATES.is_empty());

        for template in SOMATIC_AWARENESS_TEMPLATES {
            assert!(template.contains("?"), "Templates should be questions");
        }
    }
}
