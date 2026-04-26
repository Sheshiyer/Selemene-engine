//! Witness prompt generation for NadaBrahman sound awareness
//!
//! Generates non-prescriptive listening and sound awareness prompts
//! based on raga recommendations. Prompts invite self-inquiry about
//! one's relationship to sound and inner vibration.

use crate::models::NadaBrahmanAnalysis;

/// Generate a witness prompt from the analysis
pub fn generate_witness_prompt(analysis: &NadaBrahmanAnalysis) -> String {
    let prompts = generate_witness_prompts(analysis);
    prompts.join(" ")
}

/// Generate multiple witness prompts based on the analysis
pub fn generate_witness_prompts(analysis: &NadaBrahmanAnalysis) -> Vec<String> {
    let mut prompts = Vec::new();

    // Opening: sound awareness based on time of day
    prompts.push(generate_time_prompt(
        &analysis.time_recommendation.prahar_name,
    ));

    // Raga-specific listening prompt
    if let Some(primary) = analysis.recommendations.first() {
        prompts.push(generate_raga_prompt(&primary.raga_name));
    }

    // Dosha-specific somatic prompt
    if let Some(dosha) = &analysis.dosha_recommendation {
        prompts.push(generate_dosha_prompt(dosha));
    }

    // Closing integrative prompt
    prompts.push(generate_integrative_prompt());

    prompts
}

/// Generate a time-of-day aware sound prompt
fn generate_time_prompt(prahar_name: &str) -> String {
    if prahar_name.contains("Morning") || prahar_name.contains("Pratah") {
        "As the day begins, what sounds surround you right now? \
         Without naming them, can you feel their texture and quality?"
            .to_string()
    } else if prahar_name.contains("Midday") || prahar_name.contains("Madhyahna") {
        "In the fullness of midday, what is the dominant vibration you feel? \
         Where in your body does the day's energy resonate most?"
            .to_string()
    } else if prahar_name.contains("Evening") || prahar_name.contains("Sayahna") {
        "As the day transitions to night, what sounds are fading? \
         What new sounds are emerging? What does this transition feel like inside?"
            .to_string()
    } else if prahar_name.contains("Night")
        || prahar_name.contains("Pradosha")
        || prahar_name.contains("Nisha")
    {
        "In the stillness of night, what is the quietest sound you can hear? \
         What happens when you listen beyond that?"
            .to_string()
    } else if prahar_name.contains("Pre-dawn") || prahar_name.contains("Brahma") {
        "In this sacred hour before dawn, what does silence sound like? \
         Can you hear the vibration beneath all sound?"
            .to_string()
    } else {
        "What sounds are present in your environment right now? \
         Without judging or naming them, simply notice their presence."
            .to_string()
    }
}

/// Generate a prompt based on the recommended raga
fn generate_raga_prompt(raga_name: &str) -> String {
    format!(
        "The raga {} carries a particular emotional quality. \
         If you were to hum a single note right now, what pitch would your body choose? \
         What feeling arises with that tone?",
        raga_name
    )
}

/// Generate a dosha-aware somatic prompt
fn generate_dosha_prompt(dosha: &str) -> String {
    let dosha_lower = dosha.to_lowercase();
    if dosha_lower.contains("vata") {
        "Notice the quality of movement in your body — the breath, the pulse, \
         any restlessness. What would a steady, grounding tone feel like in your belly?"
            .to_string()
    } else if dosha_lower.contains("pitta") {
        "Where do you feel heat or intensity in your body right now? \
         Imagine a cool, sweet melody washing over that area. What shifts?"
            .to_string()
    } else if dosha_lower.contains("kapha") {
        "Where does your body feel heavy or still? \
         If a bright, rhythmic sound could move through that area, \
         what would begin to awaken?"
            .to_string()
    } else {
        "What does your body's own rhythm sound like right now? \
         Is it fast or slow, loud or quiet, steady or changing?"
            .to_string()
    }
}

/// Generate a closing integrative prompt
fn generate_integrative_prompt() -> String {
    "If your entire being — body, breath, and awareness — were a musical instrument, \
     what is it playing right now? Listen without changing the melody."
        .to_string()
}

/// Template prompts for sound/nada awareness
pub const NADA_AWARENESS_TEMPLATES: &[&str] = &[
    "What is the sound of your own silence?",
    "Can you hear the space between sounds?",
    "What note is your heart humming right now?",
    "If you could hear your breath as music, what instrument would it be?",
    "Where in your body does sound resonate most deeply?",
    "What is the quality of the vibration you feel right now?",
    "If your spine were a string, what note is it tuned to?",
    "What would it feel like to be completely still inside?",
    "Can you hear the nada — the subtle inner sound?",
    "What does your body want to express through sound?",
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{ChakraFrequency, PraharRecommendation, RagaRecommendation};

    fn create_test_analysis() -> NadaBrahmanAnalysis {
        NadaBrahmanAnalysis {
            time_recommendation: PraharRecommendation {
                prahar_name: "Pratah (Morning)".to_string(),
                prahar_number: 1,
                time_range: "06:00-09:00".to_string(),
                primary_raga: RagaRecommendation {
                    raga_number: 15,
                    raga_name: "Mayamalavagowla".to_string(),
                    reason: "Morning raga".to_string(),
                    score: 1.0,
                },
                secondary_ragas: vec![],
                dosha_dominance: "kapha".to_string(),
                energy_quality: "ascending".to_string(),
            },
            recommendations: vec![RagaRecommendation {
                raga_number: 15,
                raga_name: "Mayamalavagowla".to_string(),
                reason: "Morning raga".to_string(),
                score: 1.0,
            }],
            chakra_frequency: Some(ChakraFrequency {
                chakra_name: "heart".to_string(),
                solfeggio_hz: 639.0,
                binaural_target_hz: 10.5,
            }),
            dosha_recommendation: Some("vata".to_string()),
            rasa_mapping: None,
        }
    }

    #[test]
    fn test_generate_witness_prompts() {
        let analysis = create_test_analysis();
        let prompts = generate_witness_prompts(&analysis);

        assert!(prompts.len() >= 3, "Should generate at least 3 prompts");
        for prompt in &prompts {
            assert!(!prompt.is_empty(), "Prompts should not be empty");
            assert!(prompt.contains('?'), "Prompts should be questions");
        }
    }

    #[test]
    fn test_generate_witness_prompt_single() {
        let analysis = create_test_analysis();
        let prompt = generate_witness_prompt(&analysis);

        assert!(!prompt.is_empty());
        assert!(prompt.contains('?'));
        assert!(prompt.contains("Mayamalavagowla"));
    }

    #[test]
    fn test_prompts_are_non_prescriptive() {
        let analysis = create_test_analysis();
        let prompts = generate_witness_prompts(&analysis);

        let prescriptive_words = ["should", "must", "need to", "have to"];
        for prompt in &prompts {
            for word in prescriptive_words {
                assert!(
                    !prompt.to_lowercase().contains(word),
                    "Prompt should not contain prescriptive word '{}': {}",
                    word,
                    prompt
                );
            }
        }
    }

    #[test]
    fn test_time_prompts_vary() {
        let morning = generate_time_prompt("Pratah (Morning)");
        let evening = generate_time_prompt("Sayahna (Evening)");
        let night = generate_time_prompt("Nisha (Midnight)");

        assert_ne!(morning, evening);
        assert_ne!(evening, night);
    }

    #[test]
    fn test_dosha_prompts_vary() {
        let vata = generate_dosha_prompt("vata");
        let pitta = generate_dosha_prompt("pitta");
        let kapha = generate_dosha_prompt("kapha");

        assert_ne!(vata, pitta);
        assert_ne!(pitta, kapha);
        assert!(vata.contains("grounding"));
        assert!(pitta.contains("cool"));
        assert!(kapha.contains("bright"));
    }

    #[test]
    fn test_template_prompts() {
        assert!(!NADA_AWARENESS_TEMPLATES.is_empty());
        for template in NADA_AWARENESS_TEMPLATES {
            assert!(template.contains('?'), "Templates should be questions");
        }
    }
}
