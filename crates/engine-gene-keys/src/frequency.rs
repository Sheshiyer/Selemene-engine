//! Consciousness Frequency Assessment Framework
//!
//! Gene Keys frequencies (Shadow/Gift/Siddhi) are NOT deterministic from birth data.
//! Unlike HD Type/Authority, frequencies depend on consciousness level and life experience.
//! This module provides assessment framework and recognition prompts, not prediction.

use crate::models::{GeneKey, GeneKeysChart};
use crate::wisdom::get_gene_key;
use serde::{Deserialize, Serialize};

/// Consciousness frequency level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Frequency {
    /// Shadow: reactive unconscious pattern
    Shadow,
    /// Gift: constructive conscious expression
    Gift,
    /// Siddhi: transcendent realization
    Siddhi,
}

/// Frequency assessment for a single Gene Key
/// Provides framework for self-discovery, not deterministic prediction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrequencyAssessment {
    /// Gene Key number (1-64)
    pub gene_key: u8,
    
    /// Gene Key name
    pub name: String,
    
    /// Shadow name (e.g., "Entropy", "Opinion")
    pub shadow: String,
    
    /// Gift name (e.g., "Freshness", "Far-Sightedness")
    pub gift: String,
    
    /// Siddhi name (e.g., "Beauty", "Omniscience")
    pub siddhi: String,
    
    /// Full shadow description (archetypal depth preserved)
    pub shadow_description: String,
    
    /// Full gift description (archetypal depth preserved)
    pub gift_description: String,
    
    /// Full siddhi description (archetypal depth preserved)
    pub siddhi_description: String,
    
    /// Suggested frequency based on consciousness_level (optional)
    /// User must ultimately self-identify through recognition prompts
    pub suggested_frequency: Option<Frequency>,
    
    /// Recognition prompts to help user identify current frequency
    pub recognition_prompts: RecognitionPrompts,
}

/// Recognition prompts for each frequency level
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecognitionPrompts {
    /// Shadow recognition questions
    pub shadow: Vec<String>,
    
    /// Gift recognition questions
    pub gift: Vec<String>,
    
    /// Siddhi recognition questions
    pub siddhi: Vec<String>,
}

/// Assess frequencies for all active Gene Keys in chart
///
/// # Arguments
/// * `gene_keys_chart` - Complete Gene Keys chart with activations
/// * `consciousness_level` - Optional consciousness level (0-6)
///   - Level 0-2: Mostly Shadow (unconscious patterns)
///   - Level 3-4: Gift emergence (conscious expression)
///   - Level 5-6: Siddhi awareness (transcendent)
///
/// # Returns
/// Frequency assessments with recognition prompts for self-identification
pub fn assess_frequencies(
    gene_keys_chart: &GeneKeysChart,
    consciousness_level: Option<u8>,
) -> Vec<FrequencyAssessment> {
    let mut assessments = Vec::new();
    
    for activation in &gene_keys_chart.active_keys {
        let key_number = activation.key_number;
        
        if let Some(gene_key) = get_gene_key(key_number) {
            let suggested_frequency = consciousness_level.and_then(|level| {
                suggest_frequency_from_level(level)
            });
            
            let recognition_prompts = generate_recognition_prompts(gene_key);
            
            assessments.push(FrequencyAssessment {
                gene_key: key_number,
                name: gene_key.name.clone(),
                shadow: gene_key.shadow.clone(),
                gift: gene_key.gift.clone(),
                siddhi: gene_key.siddhi.clone(),
                shadow_description: gene_key.shadow_description.clone(),
                gift_description: gene_key.gift_description.clone(),
                siddhi_description: gene_key.siddhi_description.clone(),
                suggested_frequency,
                recognition_prompts,
            });
        }
    }
    
    assessments
}

/// Suggest likely frequency based on consciousness level
/// This is a rough guide only - user must self-identify
fn suggest_frequency_from_level(level: u8) -> Option<Frequency> {
    match level {
        0..=2 => Some(Frequency::Shadow),  // Unconscious patterns dominate
        3..=4 => Some(Frequency::Gift),     // Conscious expression emerging
        5..=6 => Some(Frequency::Siddhi),   // Transcendent awareness
        _ => None,
    }
}

/// Generate recognition prompts for self-identification
fn generate_recognition_prompts(gene_key: &GeneKey) -> RecognitionPrompts {
    let shadow_prompts = generate_shadow_prompts(gene_key);
    let gift_prompts = generate_gift_prompts(gene_key);
    let siddhi_prompts = generate_siddhi_prompts(gene_key);
    
    RecognitionPrompts {
        shadow: shadow_prompts,
        gift: gift_prompts,
        siddhi: siddhi_prompts,
    }
}

/// Generate shadow recognition prompts (witnessing unconscious patterns)
fn generate_shadow_prompts(gene_key: &GeneKey) -> Vec<String> {
    let shadow_name = &gene_key.shadow;
    
    vec![
        format!("Do you notice yourself reacting unconsciously through the pattern of {}?", shadow_name),
        format!("When {} arises, can you feel the contraction or resistance in your body?", shadow_name),
        format!("Do you find yourself caught in {} without realizing it until later?", shadow_name),
        "What triggers this pattern? When does it feel most intense?".to_string(),
        "Can you witness the pattern without trying to change or fix it?".to_string(),
    ]
}

/// Generate gift recognition prompts (witnessing conscious expression)
fn generate_gift_prompts(gene_key: &GeneKey) -> Vec<String> {
    let shadow_name = &gene_key.shadow;
    let gift_name = &gene_key.gift;
    
    vec![
        format!("When do you experience {} arising naturally, without effort?", gift_name),
        format!("Can you notice the space between the {} pattern and the emergence of {}?", shadow_name, gift_name),
        format!("What happens when {} flows through you? How does it feel different from the shadow?", gift_name),
        "When this gift is active, who are you serving? What opens?".to_string(),
        "Does this quality arise more often now than in the past?".to_string(),
    ]
}

/// Generate siddhi recognition prompts (witnessing transcendent awareness)
fn generate_siddhi_prompts(gene_key: &GeneKey) -> Vec<String> {
    let gift_name = &gene_key.gift;
    let siddhi_name = &gene_key.siddhi;
    
    vec![
        format!("Have you had moments where {} dissolved into {}?", gift_name, siddhi_name),
        format!("When {} is present, is there still a 'you' experiencing it?", siddhi_name),
        format!("What remains when the personal expression of {} falls away?", gift_name),
        "Have you touched a state beyond your individual consciousness?".to_string(),
        "Can you rest in the paradox of this frequency without needing to understand it?".to_string(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{ActivationSequence, GeneKeyActivation, ActivationSource};
    
    fn create_test_chart() -> GeneKeysChart {
        GeneKeysChart {
            activation_sequence: ActivationSequence {
                lifes_work: (17, 18),
                evolution: (1, 2),
                radiance: (17, 1),
                purpose: (18, 2),
            },
            active_keys: vec![
                GeneKeyActivation {
                    key_number: 17,
                    line: 3,
                    source: ActivationSource::PersonalitySun,
                    gene_key_data: None,
                },
                GeneKeyActivation {
                    key_number: 1,
                    line: 2,
                    source: ActivationSource::DesignSun,
                    gene_key_data: None,
                },
            ],
        }
    }
    
    #[test]
    fn test_assess_without_consciousness_level() {
        let chart = create_test_chart();
        let assessments = assess_frequencies(&chart, None);
        
        assert_eq!(assessments.len(), 2);
        assert!(assessments[0].suggested_frequency.is_none());
        assert!(!assessments[0].shadow_description.is_empty());
        assert!(!assessments[0].gift_description.is_empty());
        assert!(!assessments[0].siddhi_description.is_empty());
    }
    
    #[test]
    fn test_assess_with_shadow_level() {
        let chart = create_test_chart();
        let assessments = assess_frequencies(&chart, Some(2));
        
        assert_eq!(assessments[0].suggested_frequency, Some(Frequency::Shadow));
    }
    
    #[test]
    fn test_assess_with_gift_level() {
        let chart = create_test_chart();
        let assessments = assess_frequencies(&chart, Some(4));
        
        assert_eq!(assessments[0].suggested_frequency, Some(Frequency::Gift));
    }
    
    #[test]
    fn test_assess_with_siddhi_level() {
        let chart = create_test_chart();
        let assessments = assess_frequencies(&chart, Some(6));
        
        assert_eq!(assessments[0].suggested_frequency, Some(Frequency::Siddhi));
    }
    
    #[test]
    fn test_recognition_prompts_present() {
        let chart = create_test_chart();
        let assessments = assess_frequencies(&chart, None);
        
        assert!(!assessments[0].recognition_prompts.shadow.is_empty());
        assert!(!assessments[0].recognition_prompts.gift.is_empty());
        assert!(!assessments[0].recognition_prompts.siddhi.is_empty());
    }
    
    #[test]
    fn test_shadow_frequency_range() {
        assert_eq!(suggest_frequency_from_level(0), Some(Frequency::Shadow));
        assert_eq!(suggest_frequency_from_level(1), Some(Frequency::Shadow));
        assert_eq!(suggest_frequency_from_level(2), Some(Frequency::Shadow));
    }
    
    #[test]
    fn test_gift_frequency_range() {
        assert_eq!(suggest_frequency_from_level(3), Some(Frequency::Gift));
        assert_eq!(suggest_frequency_from_level(4), Some(Frequency::Gift));
    }
    
    #[test]
    fn test_siddhi_frequency_range() {
        assert_eq!(suggest_frequency_from_level(5), Some(Frequency::Siddhi));
        assert_eq!(suggest_frequency_from_level(6), Some(Frequency::Siddhi));
    }
}
