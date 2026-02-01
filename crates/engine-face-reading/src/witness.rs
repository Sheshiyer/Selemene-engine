//! Witness prompts for facial self-observation
//!
//! Non-prescriptive prompts that invite self-reflection without diagnosis.
//! Adapts to consciousness level and analysis results.

use crate::models::FaceAnalysis;
use crate::wisdom::get_zone_wisdom;
use rand::prelude::*;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

/// Generate witness prompts based on face analysis
///
/// Returns 3-4 non-prescriptive prompts for self-observation
pub fn generate_witness_prompts(
    analysis: &FaceAnalysis,
    consciousness_level: u8,
    seed: Option<u64>,
) -> Vec<String> {
    let mut rng = match seed {
        Some(s) => ChaCha8Rng::seed_from_u64(s),
        None => ChaCha8Rng::from_entropy(),
    };

    let mut prompts = Vec::new();

    // Add level-appropriate prompts
    match consciousness_level {
        0..=2 => prompts.extend(generate_foundational_prompts(analysis, &mut rng)),
        3..=4 => prompts.extend(generate_awareness_prompts(analysis, &mut rng)),
        5..=6 => prompts.extend(generate_integration_prompts(analysis, &mut rng)),
        _ => prompts.extend(generate_awareness_prompts(analysis, &mut rng)),
    }

    // Ensure we have 3-4 prompts
    while prompts.len() < 3 {
        prompts.push(generate_general_prompt(&mut rng));
    }
    if prompts.len() > 4 {
        prompts.truncate(4);
    }

    prompts
}

/// Generate a single witness prompt (for engine output)
pub fn generate_single_witness_prompt(
    analysis: &FaceAnalysis,
    consciousness_level: u8,
) -> String {
    let prompts = generate_witness_prompts(analysis, consciousness_level, None);
    prompts.into_iter().next().unwrap_or_else(|| {
        "What do you notice when you look at your reflection with curiosity rather than judgment?".to_string()
    })
}

/// Foundational prompts (levels 0-2) - basic self-observation
fn generate_foundational_prompts(analysis: &FaceAnalysis, rng: &mut ChaCha8Rng) -> Vec<String> {
    let mut prompts = Vec::new();

    // Zone-based observation
    if let Some(indicator) = analysis.health_indicators.first() {
        let zone_name = indicator.zone.display_name().to_lowercase();
        prompts.push(format!(
            "What do you notice when you observe your {} in the mirror? What stories might that area tell?"
        , zone_name));
    }

    // Element-based
    let element = analysis.constitution.tcm_element;
    prompts.push(format!(
        "How might your {} nature be reflected in your facial features? Take a moment to look without judgment.",
        element.display_name()
    ));

    // General observation
    let general_prompts = [
        "When you look at your face today, what is the first thing you notice?",
        "How does your face feel different today compared to yesterday?",
        "What expression does your resting face naturally fall into?",
        "If your face could speak, what might it be communicating right now?",
    ];
    prompts.push(general_prompts.choose(rng).unwrap().to_string());

    prompts
}

/// Awareness prompts (levels 3-4) - deeper inquiry
fn generate_awareness_prompts(analysis: &FaceAnalysis, rng: &mut ChaCha8Rng) -> Vec<String> {
    let mut prompts = Vec::new();

    // Constitution-based
    let dosha = analysis.constitution.primary_dosha;
    prompts.push(format!(
        "How does your {} constitution express itself through your facial features? What qualities feel most alive in you today?",
        dosha.display_name()
    ));

    // Element balance inquiry
    let dominant = analysis.elemental_balance.dominant();
    prompts.push(format!(
        "Where do you see your {} nature expressed in your face? What aspects of {} feel most present right now?",
        dominant.display_name(),
        dominant.description().to_lowercase()
    ));

    // Trait reflection
    if let Some(trait_info) = analysis.personality_indicators.first() {
        prompts.push(format!(
            "You show signs of being {}. How do you experience this quality within yourself? Does it feel authentic to who you are?",
            trait_info.trait_name.to_lowercase()
        ));
    }

    // Emotional mapping
    let emotional_prompts = [
        "How does your inner state today reflect in your facial expression?",
        "What emotions seem to have left traces on your face over time?",
        "Where in your face do you hold tension, and what might that reveal about your relationship with stress?",
        "How does your face change when you think about something that brings you joy?",
    ];
    prompts.push(emotional_prompts.choose(rng).unwrap().to_string());

    prompts
}

/// Integration prompts (levels 5-6) - transcendent inquiry
fn generate_integration_prompts(analysis: &FaceAnalysis, rng: &mut ChaCha8Rng) -> Vec<String> {
    let mut prompts = Vec::new();

    // Life journey reflection
    prompts.push(
        "What stories might your features tell about your life journey? What wisdom have the years etched into your face?".to_string()
    );

    // Element integration
    let element = analysis.constitution.tcm_element;
    let zones = crate::wisdom::zones_for_element(element);
    if let Some(zone) = zones.first() {
        if let Some(wisdom) = get_zone_wisdom(*zone) {
            prompts.push(format!(
                "The {} in you connects to {}. How does this ancient wisdom resonate with your direct experience of yourself?",
                element.display_name(),
                wisdom.emotional_connection.to_lowercase()
            ));
        }
    }

    // Transcendent observation
    let transcendent_prompts = [
        "Beyond the physical features, what essence do you see looking back at you in the mirror?",
        "How has your relationship with your own face changed over your lifetime?",
        "What would it mean to accept every line and feature as a perfect expression of your journey?",
        "When you look at your face with complete compassion, what do you discover?",
        "How might your face be a doorway to understanding your deeper nature?",
    ];
    prompts.push(transcendent_prompts.choose(rng).unwrap().to_string());

    // Body-mind connection
    prompts.push(
        "What connection do you notice between how your face feels and your current state of mind?".to_string()
    );

    prompts
}

/// Generate a general prompt as fallback
fn generate_general_prompt(rng: &mut ChaCha8Rng) -> String {
    let prompts = [
        "What do you notice when you look at your reflection with curiosity rather than judgment?",
        "How might your face be expressing what words cannot?",
        "What aspect of your inner life seems most visible in your features today?",
        "If you were meeting yourself for the first time, what would your face tell you?",
        "What does it feel like to simply observe your face without trying to change anything?",
    ];
    prompts.choose(rng).unwrap().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock::generate_mock_analysis;

    #[test]
    fn test_generate_witness_prompts_count() {
        let analysis = generate_mock_analysis(Some(42));
        
        for level in 0..=6 {
            let prompts = generate_witness_prompts(&analysis, level, Some(42));
            assert!(prompts.len() >= 3, "Level {} should have at least 3 prompts", level);
            assert!(prompts.len() <= 4, "Level {} should have at most 4 prompts", level);
        }
    }

    #[test]
    fn test_prompts_are_questions() {
        let analysis = generate_mock_analysis(Some(123));
        let prompts = generate_witness_prompts(&analysis, 3, Some(123));
        
        for prompt in &prompts {
            assert!(prompt.contains('?'), "Prompt should be a question: {}", prompt);
        }
    }

    #[test]
    fn test_single_witness_prompt() {
        let analysis = generate_mock_analysis(Some(999));
        let prompt = generate_single_witness_prompt(&analysis, 3);
        
        assert!(!prompt.is_empty());
        assert!(prompt.contains('?'));
    }

    #[test]
    fn test_level_appropriate_content() {
        let analysis = generate_mock_analysis(Some(42));
        
        // Foundational level should mention observation
        let low_prompts = generate_witness_prompts(&analysis, 1, Some(42));
        let has_basic_observation = low_prompts.iter().any(|p| 
            p.contains("notice") || p.contains("observe") || p.contains("look")
        );
        assert!(has_basic_observation, "Low level should have observation prompts");
        
        // High level should have transcendent content
        let high_prompts = generate_witness_prompts(&analysis, 6, Some(42));
        let has_transcendent = high_prompts.iter().any(|p| 
            p.contains("essence") || p.contains("journey") || 
            p.contains("wisdom") || p.contains("compassion")
        );
        assert!(has_transcendent, "High level should have transcendent prompts");
    }

    #[test]
    fn test_prompts_non_prescriptive() {
        let analysis = generate_mock_analysis(Some(42));
        let prompts = generate_witness_prompts(&analysis, 3, Some(42));
        
        for prompt in &prompts {
            // Should not contain diagnostic language
            assert!(!prompt.contains("you have"), "Prompt should not diagnose: {}", prompt);
            assert!(!prompt.contains("you should"), "Prompt should not prescribe: {}", prompt);
            assert!(!prompt.contains("you must"), "Prompt should not command: {}", prompt);
        }
    }

    #[test]
    fn test_prompts_reference_analysis() {
        let analysis = generate_mock_analysis(Some(42));
        let prompts = generate_witness_prompts(&analysis, 3, Some(42));
        
        // At least one prompt should reference specific analysis elements
        let dosha_name = analysis.constitution.primary_dosha.display_name().to_lowercase();
        let element_name = analysis.constitution.tcm_element.display_name().to_lowercase();
        
        let references_analysis = prompts.iter().any(|p| {
            let lower = p.to_lowercase();
            lower.contains(&dosha_name) || lower.contains(&element_name)
        });
        
        assert!(references_analysis, "At least one prompt should reference the analysis");
    }
}
