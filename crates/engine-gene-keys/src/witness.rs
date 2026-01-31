//! Consciousness-level adaptive witness prompt generation for Gene Keys
//!
//! Prompts adapt to consciousness levels:
//! - Level 0-2: Shadow recognition (witnessing unconscious patterns)
//! - Level 3-4: Gift emergence (conscious expression awareness)
//! - Level 5-6: Siddhi contemplation (transcendent awareness)

use crate::models::{GeneKeysChart, ActivationSequence};
use crate::wisdom::get_gene_key;

/// Generate a consciousness-level adaptive witness prompt
///
/// # Arguments
/// * `chart` - Complete Gene Keys chart with activations
/// * `consciousness_level` - Current consciousness development level (0-6)
///
/// # Returns
/// An inquiry-format question that references specific Gene Keys
pub fn generate_witness_prompt(chart: &GeneKeysChart, consciousness_level: u8) -> String {
    match consciousness_level {
        0..=2 => generate_shadow_prompt(chart),
        3..=4 => generate_gift_prompt(chart),
        5..=6 => generate_siddhi_prompt(chart),
        _ => generate_gift_prompt(chart), // Default to Gift for out-of-range
    }
}

/// Generate Shadow-level prompt (Level 0-2)
///
/// Focuses on witnessing unconscious patterns from active Gene Keys.
/// Emphasizes non-judgmental observation of reactive states.
fn generate_shadow_prompt(chart: &GeneKeysChart) -> String {
    let seq = &chart.activation_sequence;
    
    // Get Life's Work keys (conscious purpose)
    let lifes_work_sun = get_gene_key(seq.lifes_work.0);
    let lifes_work_earth = get_gene_key(seq.lifes_work.1);
    
    // Get Evolution keys (unconscious growth)
    let evolution_sun = get_gene_key(seq.evolution.0);
    
    if let (Some(lw_sun), Some(lw_earth), Some(ev_sun)) = (lifes_work_sun, lifes_work_earth, evolution_sun) {
        format!(
            "What unconscious patterns drive your sense of purpose? How do the shadows of {} ({}) and {} ({}) shape what you believe you must do? When {} ({}) operates unconsciously, what recurring patterns do you notice in your growth journey?",
            lw_sun.shadow,
            seq.lifes_work.0,
            lw_earth.shadow,
            seq.lifes_work.1,
            ev_sun.shadow,
            seq.evolution.0
        )
    } else {
        // Fallback if keys not found
        format!(
            "What unconscious patterns drive your sense of purpose? How do Gene Keys {} and {} shape what you believe you must do?",
            seq.lifes_work.0,
            seq.lifes_work.1
        )
    }
}

/// Generate Gift-level prompt (Level 3-4)
///
/// Focuses on conscious expression from the 4 Activation Sequences.
/// Explores how gifts dance together in awareness.
fn generate_gift_prompt(chart: &GeneKeysChart) -> String {
    let seq = &chart.activation_sequence;
    
    // Get Radiance keys (core identity/magnetism)
    let radiance_pers = get_gene_key(seq.radiance.0);
    let radiance_design = get_gene_key(seq.radiance.1);
    
    // Get Purpose keys (higher calling)
    let purpose_pers = get_gene_key(seq.purpose.0);
    let purpose_design = get_gene_key(seq.purpose.1);
    
    if let (Some(rad_p), Some(rad_d), Some(pur_p), Some(pur_d)) = 
        (radiance_pers, radiance_design, purpose_pers, purpose_design) {
        format!(
            "How do your conscious gifts {} ({}) and {} ({}) create your core magnetism? When you're most authentic, what happens in the interplay between {} ({}) and {} ({}) as your higher calling reveals itself? What invitation lives in the space between {} becoming {} and {} becoming {}?",
            rad_p.gift,
            seq.radiance.0,
            rad_d.gift,
            seq.radiance.1,
            pur_p.gift,
            seq.purpose.0,
            pur_d.gift,
            seq.purpose.1,
            rad_p.shadow,
            rad_p.gift,
            pur_p.shadow,
            pur_p.gift
        )
    } else {
        // Fallback if keys not found
        format!(
            "How do Gene Keys {} and {} shape your core identity? What happens when you witness the gifts of {} and {} emerging in your life?",
            seq.radiance.0,
            seq.radiance.1,
            seq.purpose.0,
            seq.purpose.1
        )
    }
}

/// Generate Siddhi-level prompt (Level 5-6)
///
/// Focuses on transcendent awareness beyond personal purpose.
/// Invites recognition of the divine expressing through form.
fn generate_siddhi_prompt(chart: &GeneKeysChart) -> String {
    let seq = &chart.activation_sequence;
    
    // Get Purpose keys for highest realization
    let purpose_pers = get_gene_key(seq.purpose.0);
    let purpose_design = get_gene_key(seq.purpose.1);
    
    // Get Life's Work for transcendence integration
    let lifes_work_sun = get_gene_key(seq.lifes_work.0);
    
    if let (Some(pur_p), Some(pur_d), Some(lw_sun)) = (purpose_pers, purpose_design, lifes_work_sun) {
        format!(
            "Beyond the personal purpose of {} and {}, what transcendent awareness is seeking recognition? When the siddhis of {} (Gene Key {}) and {} (Gene Key {}) dissolve into unity, what remains? How does {} (Gene Key {}) become a doorway to the infinite expressing as the finite?",
            pur_p.gift,
            pur_d.gift,
            pur_p.siddhi,
            seq.purpose.0,
            pur_d.siddhi,
            seq.purpose.1,
            lw_sun.siddhi,
            seq.lifes_work.0
        )
    } else {
        // Fallback if keys not found
        format!(
            "Beyond your personal purpose, what transcendent awareness is inviting recognition through Gene Keys {} and {}? What seeks to be realized beyond the self?",
            seq.purpose.0,
            seq.lifes_work.0
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::ActivationSequence;
    
    fn create_test_chart() -> GeneKeysChart {
        GeneKeysChart {
            activation_sequence: ActivationSequence {
                lifes_work: (17, 18),
                evolution: (45, 26),
                radiance: (17, 45),
                purpose: (18, 26),
            },
            active_keys: vec![],
        }
    }
    
    #[test]
    fn test_shadow_prompt_level_0() {
        let chart = create_test_chart();
        let prompt = generate_witness_prompt(&chart, 0);
        
        assert!(!prompt.is_empty(), "Prompt should not be empty");
        assert!(prompt.contains("unconscious"), "Shadow prompt should mention 'unconscious'");
        assert!(prompt.contains("17") || prompt.contains("18"), "Should reference Gene Key numbers");
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
        assert!(prompt.contains("17") || prompt.contains("45"), "Should reference Radiance keys");
    }
    
    #[test]
    fn test_gift_prompt_level_4() {
        let chart = create_test_chart();
        let prompt = generate_witness_prompt(&chart, 4);
        
        assert!(!prompt.is_empty(), "Prompt should not be empty");
        assert!(prompt.contains("authentic") || prompt.contains("gift"), "Gift prompt should mention gifts or authenticity");
    }
    
    #[test]
    fn test_siddhi_prompt_level_5() {
        let chart = create_test_chart();
        let prompt = generate_witness_prompt(&chart, 5);
        
        assert!(!prompt.is_empty(), "Prompt should not be empty");
        assert!(prompt.contains("transcendent") || prompt.contains("beyond"), "Siddhi prompt should be transcendent");
        assert!(prompt.contains("18") || prompt.contains("26"), "Should reference Purpose keys");
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
            let has_key_reference = 
                prompt.contains("17") ||
                prompt.contains("18") ||
                prompt.contains("45") ||
                prompt.contains("26");
            
            assert!(has_key_reference, 
                "Level {} prompt should reference Gene Key numbers: {}", 
                level, prompt);
        }
    }
    
    #[test]
    fn test_all_prompts_are_questions() {
        let chart = create_test_chart();
        
        for level in 0..=6 {
            let prompt = generate_witness_prompt(&chart, level);
            assert!(prompt.contains("?"), 
                "Level {} prompt should be inquiry format (contain '?'): {}", 
                level, prompt);
        }
    }
}
