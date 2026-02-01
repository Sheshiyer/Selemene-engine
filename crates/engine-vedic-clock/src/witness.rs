//! Witness prompts for the VedicClock-TCM engine
//!
//! Non-prescriptive, inquiry-based prompts that invite self-observation
//! of energy patterns throughout the day.

use crate::models::{Organ, Dosha};
use rand::prelude::IndexedRandom;

/// Generate a witness prompt based on current organ and dosha
///
/// # Arguments
/// * `organ` - Currently active organ
/// * `dosha` - Currently dominant dosha
/// * `consciousness_level` - User's consciousness level (0-6)
///
/// # Returns
/// A non-prescriptive inquiry prompt
pub fn generate_witness_prompt(organ: &Organ, dosha: &Dosha, consciousness_level: u8) -> String {
    match consciousness_level {
        0..=2 => generate_awareness_prompt(organ, dosha),
        3..=4 => generate_observation_prompt(organ, dosha),
        5..=6 => generate_integration_prompt(organ, dosha),
        _ => generate_observation_prompt(organ, dosha),
    }
}

/// Level 0-2: Basic awareness prompts
/// Focus on noticing bodily sensations and energy patterns
fn generate_awareness_prompt(organ: &Organ, dosha: &Dosha) -> String {
    let prompts = get_awareness_prompts(organ);
    let dosha_prompts = get_dosha_awareness_prompts(dosha);
    
    // Select one from each category
    let organ_prompt = prompts.choose(&mut rand::rng())
        .copied()
        .unwrap_or("What do you notice about your energy right now?");
    
    let dosha_prompt = dosha_prompts.choose(&mut rand::rng())
        .copied()
        .unwrap_or("");
    
    if dosha_prompt.is_empty() {
        organ_prompt.to_string()
    } else {
        format!("{} {}", organ_prompt, dosha_prompt)
    }
}

/// Level 3-4: Deeper observation prompts
/// Explore the relationship between time, body, and activity
fn generate_observation_prompt(organ: &Organ, dosha: &Dosha) -> String {
    let prompts = get_observation_prompts(organ);
    let dosha_prompts = get_dosha_observation_prompts(dosha);
    
    let organ_prompt = prompts.choose(&mut rand::rng())
        .copied()
        .unwrap_or("What patterns do you notice in your energy at this time of day?");
    
    let dosha_prompt = dosha_prompts.choose(&mut rand::rng())
        .copied()
        .unwrap_or("");
    
    if dosha_prompt.is_empty() {
        organ_prompt.to_string()
    } else {
        format!("{} {}", organ_prompt, dosha_prompt)
    }
}

/// Level 5-6: Integration prompts
/// Transcendent awareness of the interconnection of all cycles
fn generate_integration_prompt(organ: &Organ, dosha: &Dosha) -> String {
    let prompts = get_integration_prompts(organ);
    let dosha_prompts = get_dosha_integration_prompts(dosha);
    
    let organ_prompt = prompts.choose(&mut rand::rng())
        .copied()
        .unwrap_or("What remains constant as the cycles of time move through you?");
    
    let dosha_prompt = dosha_prompts.choose(&mut rand::rng())
        .copied()
        .unwrap_or("");
    
    if dosha_prompt.is_empty() {
        organ_prompt.to_string()
    } else {
        format!("{} {}", organ_prompt, dosha_prompt)
    }
}

// Organ-specific awareness prompts (Level 0-2)
fn get_awareness_prompts(organ: &Organ) -> Vec<&'static str> {
    match organ {
        Organ::Lung => vec![
            "What do you notice about your breath right now?",
            "How does your chest feel in this moment?",
            "What happens when you take a deep, conscious breath?",
        ],
        Organ::LargeIntestine => vec![
            "What are you ready to release today?",
            "How does your body feel about letting go?",
            "What do you notice about your sense of elimination and release?",
        ],
        Organ::Stomach => vec![
            "What do you notice about your appetite right now?",
            "How does your stomach respond to the thought of nourishment?",
            "What feels nourishing to you in this moment?",
        ],
        Organ::Spleen => vec![
            "How clear does your mind feel right now?",
            "What do you notice about your ability to concentrate?",
            "Where does your attention naturally want to go?",
        ],
        Organ::Heart => vec![
            "What do you notice in your heart space right now?",
            "How open or closed does your chest feel?",
            "What brings you a sense of joy in this moment?",
        ],
        Organ::SmallIntestine => vec![
            "How easily can you discern what's important right now?",
            "What do you notice about your ability to sort through information?",
            "What deserves your attention and what doesn't?",
        ],
        Organ::Bladder => vec![
            "How is your energy level right now?",
            "What do you notice about your capacity for sustained effort?",
            "Where in your body do you feel stored energy?",
        ],
        Organ::Kidney => vec![
            "What do you notice about your willpower right now?",
            "How does your body respond to the idea of rest?",
            "What fears, if any, are present in your awareness?",
        ],
        Organ::Pericardium => vec![
            "How protected or vulnerable do you feel emotionally?",
            "What do you notice about your boundaries right now?",
            "How open are you to intimate connection?",
        ],
        Organ::TripleWarmer => vec![
            "How balanced do you feel across body, mind, and spirit?",
            "What do you notice about your readiness for rest?",
            "How harmonious does your internal state feel?",
        ],
        Organ::Gallbladder => vec![
            "What decisions are wanting to be made?",
            "How decisive or indecisive do you feel?",
            "What do you notice about your courage right now?",
        ],
        Organ::Liver => vec![
            "What plans or visions are arising in your awareness?",
            "How do you feel about the direction of your life?",
            "What creative impulses are present?",
        ],
    }
}

// Organ-specific observation prompts (Level 3-4)
fn get_observation_prompts(organ: &Organ) -> Vec<&'static str> {
    match organ {
        Organ::Lung => vec![
            "How does your breath pattern relate to your emotional state?",
            "What connection do you observe between inspiration and your breathing?",
            "How does grief or letting go show up in your breathing?",
        ],
        Organ::LargeIntestine => vec![
            "What patterns do you notice around holding on versus releasing?",
            "How does your body's elimination reflect your mental releasing?",
            "What relationship exists between physical and emotional release for you?",
        ],
        Organ::Stomach => vec![
            "How do your eating patterns reflect your need for security?",
            "What do you observe about digestion and trust in your life?",
            "How does worry affect your stomach and appetite?",
        ],
        Organ::Spleen => vec![
            "What patterns exist between overthinking and your energy?",
            "How does your mental clarity connect to your sense of groundedness?",
            "What do you observe about studying or concentration at this time?",
        ],
        Organ::Heart => vec![
            "How does your sense of joy affect your physical heart?",
            "What patterns exist between connection and your heart energy?",
            "How does anxiety manifest in your heart space?",
        ],
        Organ::SmallIntestine => vec![
            "How do you sort truth from falsehood in your life?",
            "What patterns exist between discernment and peace of mind?",
            "How does confusion affect your physical body?",
        ],
        Organ::Bladder => vec![
            "What relationship do you observe between fear and your energy reserves?",
            "How do your work patterns reflect your bladder energy?",
            "What happens to your body when you push through fatigue?",
        ],
        Organ::Kidney => vec![
            "How does fear manifest in your body and life choices?",
            "What patterns exist between willpower and restoration?",
            "How do you replenish your deepest energy reserves?",
        ],
        Organ::Pericardium => vec![
            "How do your emotional boundaries affect your relationships?",
            "What patterns exist between protection and openness for you?",
            "How does emotional intimacy affect your physical heart?",
        ],
        Organ::TripleWarmer => vec![
            "What patterns of harmony or disharmony do you notice across your systems?",
            "How does your sleep preparation affect your overall balance?",
            "What happens when you try to maintain harmony under stress?",
        ],
        Organ::Gallbladder => vec![
            "How do unresolved decisions affect your sleep and dreams?",
            "What patterns exist between indecision and frustration?",
            "How does courage relate to your sense of direction in life?",
        ],
        Organ::Liver => vec![
            "How does suppressed anger affect your vision and planning?",
            "What patterns exist between creative blockage and frustration?",
            "How do your dreams reflect your waking life direction?",
        ],
    }
}

// Organ-specific integration prompts (Level 5-6)
fn get_integration_prompts(organ: &Organ) -> Vec<&'static str> {
    match organ {
        Organ::Lung => vec![
            "What breathes you when you are not breathing yourself?",
            "Where does the boundary between self and air dissolve?",
            "What inspiration exists beyond personal inspiration?",
        ],
        Organ::LargeIntestine => vec![
            "What releases when there is no one holding on?",
            "How does universal flow move through individual form?",
            "What remains when all that can be released is released?",
        ],
        Organ::Stomach => vec![
            "What nourishes when there is no hunger?",
            "How does trust exist beyond circumstance?",
            "What receives when the receiver is absent?",
        ],
        Organ::Spleen => vec![
            "What thinks when thinking stops?",
            "Where does understanding arise before thought?",
            "What knows without the knower?",
        ],
        Organ::Heart => vec![
            "What loves when there is no lover?",
            "How does joy exist independent of its cause?",
            "What connects when connection transcends separation?",
        ],
        Organ::SmallIntestine => vec![
            "What discerns when preference is absent?",
            "How does truth sort itself without a sorter?",
            "What clarity exists before decision?",
        ],
        Organ::Bladder => vec![
            "What energy exists beyond personal reserves?",
            "How does universal will express through individual action?",
            "What works when the worker rests?",
        ],
        Organ::Kidney => vec![
            "What courage exists when fear is fully allowed?",
            "How does will manifest without a willer?",
            "What depths remain when depth is fully plumbed?",
        ],
        Organ::Pericardium => vec![
            "What protection exists when vulnerability is complete?",
            "How does love guard itself without walls?",
            "What intimacy exists beyond the intimate?",
        ],
        Organ::TripleWarmer => vec![
            "What harmony exists beyond balance and imbalance?",
            "How do all systems unite in awareness itself?",
            "What rests when rest and activity are one?",
        ],
        Organ::Gallbladder => vec![
            "What decides when there is no decider?",
            "How does direction emerge from directionlessness?",
            "What courage exists when fear is welcomed?",
        ],
        Organ::Liver => vec![
            "What plans when planning ceases?",
            "How does vision arise from the unseen?",
            "What creates when the creator dissolves?",
        ],
    }
}

// Dosha-specific awareness prompts
fn get_dosha_awareness_prompts(dosha: &Dosha) -> Vec<&'static str> {
    match dosha {
        Dosha::Vata => vec![
            "What movement do you notice in your mind?",
            "How grounded or scattered do you feel?",
            "",
        ],
        Dosha::Pitta => vec![
            "What heat or intensity do you notice in your body?",
            "How sharp or diffuse is your focus?",
            "",
        ],
        Dosha::Kapha => vec![
            "What heaviness or lightness do you notice?",
            "How motivated or still do you feel?",
            "",
        ],
    }
}

// Dosha-specific observation prompts
fn get_dosha_observation_prompts(dosha: &Dosha) -> Vec<&'static str> {
    match dosha {
        Dosha::Vata => vec![
            "How do the qualities of air and space express through you now?",
            "What patterns connect your mental activity to physical restlessness?",
            "",
        ],
        Dosha::Pitta => vec![
            "How does your inner fire transform experience into understanding?",
            "What patterns exist between your drive and your digestion?",
            "",
        ],
        Dosha::Kapha => vec![
            "How do earth and water qualities support or hinder you now?",
            "What patterns connect your sense of stability to stagnation?",
            "",
        ],
    }
}

// Dosha-specific integration prompts
fn get_dosha_integration_prompts(dosha: &Dosha) -> Vec<&'static str> {
    match dosha {
        Dosha::Vata => vec![
            "What moves when the mover is still?",
            "How does emptiness create form?",
            "",
        ],
        Dosha::Pitta => vec![
            "What transforms when transformation stops?",
            "How does light illuminate itself?",
            "",
        ],
        Dosha::Kapha => vec![
            "What stability exists in constant change?",
            "How does form arise from formlessness?",
            "",
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_witness_prompt_level_0() {
        let prompt = generate_witness_prompt(&Organ::Heart, &Dosha::Pitta, 0);
        assert!(!prompt.is_empty());
        assert!(prompt.contains("?"), "Prompt should be a question");
    }

    #[test]
    fn test_generate_witness_prompt_level_3() {
        let prompt = generate_witness_prompt(&Organ::Stomach, &Dosha::Kapha, 3);
        assert!(!prompt.is_empty());
        assert!(prompt.contains("?"), "Prompt should be a question");
    }

    #[test]
    fn test_generate_witness_prompt_level_5() {
        let prompt = generate_witness_prompt(&Organ::Liver, &Dosha::Vata, 5);
        assert!(!prompt.is_empty());
        assert!(prompt.contains("?"), "Prompt should be a question");
    }

    #[test]
    fn test_all_organs_have_prompts() {
        for organ in Organ::all_in_cycle_order() {
            let awareness = get_awareness_prompts(&organ);
            let observation = get_observation_prompts(&organ);
            let integration = get_integration_prompts(&organ);
            
            assert!(!awareness.is_empty(), "{:?} should have awareness prompts", organ);
            assert!(!observation.is_empty(), "{:?} should have observation prompts", organ);
            assert!(!integration.is_empty(), "{:?} should have integration prompts", organ);
        }
    }

    #[test]
    fn test_all_doshas_have_prompts() {
        for dosha in [Dosha::Vata, Dosha::Pitta, Dosha::Kapha] {
            let awareness = get_dosha_awareness_prompts(&dosha);
            let observation = get_dosha_observation_prompts(&dosha);
            let integration = get_dosha_integration_prompts(&dosha);
            
            assert!(!awareness.is_empty(), "{:?} should have awareness prompts", dosha);
            assert!(!observation.is_empty(), "{:?} should have observation prompts", dosha);
            assert!(!integration.is_empty(), "{:?} should have integration prompts", dosha);
        }
    }

    #[test]
    fn test_prompts_are_non_prescriptive() {
        // Check that prompts don't use prescriptive language
        let prescriptive_words = ["should", "must", "need to", "have to", "ought"];
        
        let prompt = generate_witness_prompt(&Organ::Heart, &Dosha::Pitta, 3);
        
        for word in prescriptive_words {
            assert!(
                !prompt.to_lowercase().contains(word),
                "Prompt should not contain prescriptive language '{}': {}",
                word,
                prompt
            );
        }
    }
}
