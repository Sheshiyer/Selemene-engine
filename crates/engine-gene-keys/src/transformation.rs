//! Transformation Pathway Guidance
//!
//! Shadow→Gift→Siddhi transformation through inquiry-based contemplation.
//! NON-PRESCRIPTIVE: Uses questions and witnessing prompts, not instructions.

use crate::frequency::{Frequency, FrequencyAssessment};
use crate::models::GeneKey;
use crate::wisdom::get_gene_key;
use serde::{Deserialize, Serialize};

/// Transformation pathway for a single Gene Key
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformationPathway {
    /// Gene Key number (1-64)
    pub gene_key: u8,
    
    /// Gene Key name
    pub name: String,
    
    /// Current frequency (user-identified or suggested)
    pub current_frequency: Frequency,
    
    /// Next frequency in transformation arc
    pub next_frequency: Frequency,
    
    /// Core inquiry for this transformation
    pub core_inquiry: String,
    
    /// Contemplation prompts (inquiry-based, not prescriptive)
    pub contemplations: Vec<String>,
    
    /// Witnessing practices (invitations, not commands)
    pub witnessing_practices: Vec<String>,
    
    /// Shadow→Gift transition inquiry
    pub shadow_to_gift_inquiry: Option<String>,
    
    /// Gift→Siddhi transition inquiry
    pub gift_to_siddhi_inquiry: Option<String>,
}

/// Generate transformation pathways from frequency assessments
///
/// Creates inquiry-based guidance for Shadow→Gift and Gift→Siddhi transitions.
/// Language is non-prescriptive: invites witnessing and exploration, never commands.
pub fn generate_transformation_pathways(
    assessments: &[FrequencyAssessment],
) -> Vec<TransformationPathway> {
    let mut pathways = Vec::new();
    
    for assessment in assessments {
        // Use suggested frequency or default to Shadow
        let current_freq = assessment.suggested_frequency.unwrap_or(Frequency::Shadow);
        
        if let Some(gene_key) = get_gene_key(assessment.gene_key) {
            // Generate pathways for each possible transition
            match current_freq {
                Frequency::Shadow => {
                    pathways.push(create_shadow_to_gift_pathway(gene_key));
                }
                Frequency::Gift => {
                    pathways.push(create_gift_to_siddhi_pathway(gene_key));
                }
                Frequency::Siddhi => {
                    // At Siddhi, the pathway is embodiment and integration
                    pathways.push(create_siddhi_integration_pathway(gene_key));
                }
            }
        }
    }
    
    pathways
}

/// Create Shadow→Gift transformation pathway
fn create_shadow_to_gift_pathway(gene_key: &GeneKey) -> TransformationPathway {
    let shadow = &gene_key.shadow;
    let gift = &gene_key.gift;
    
    let core_inquiry = format!(
        "What happens when you witness {} without trying to change it? What space opens?",
        shadow
    );
    
    let contemplations = vec![
        format!("Notice when {} arises. Can you feel it in your body before your mind names it?", shadow),
        format!("What is {} protecting? What fear lies beneath the pattern?", shadow),
        format!("When the {} pattern softens, what quality naturally emerges?", shadow),
        "Can you stay present with discomfort without reacting or fixing?".to_string(),
        format!("What would {} look like if it weren't a problem to solve?", shadow),
    ];
    
    let witnessing_practices = vec![
        format!("You might notice {} arising in daily life, pausing to feel it without judgment.", shadow),
        format!("You might explore the sensation of {} in your body, staying curious rather than reactive.", shadow),
        "You might sit with the pattern, asking 'What are you trying to tell me?'".to_string(),
        format!("You might observe when {} transforms naturally into {}.", shadow, gift),
    ];
    
    let shadow_to_gift_inquiry = Some(format!(
        "What happens when you notice the rigidity of {} without trying to change it? \
        Can you see the larger pattern this {} is protecting? \
        What dissolves when {} is no longer the enemy?",
        shadow, shadow, shadow
    ));
    
    TransformationPathway {
        gene_key: gene_key.number,
        name: gene_key.name.clone(),
        current_frequency: Frequency::Shadow,
        next_frequency: Frequency::Gift,
        core_inquiry,
        contemplations,
        witnessing_practices,
        shadow_to_gift_inquiry,
        gift_to_siddhi_inquiry: None,
    }
}

/// Create Gift→Siddhi transformation pathway
fn create_gift_to_siddhi_pathway(gene_key: &GeneKey) -> TransformationPathway {
    let gift = &gene_key.gift;
    let siddhi = &gene_key.siddhi;
    
    let core_inquiry = format!(
        "When {} is flowing, who is experiencing it? What remains when the 'doer' falls away?",
        gift
    );
    
    let contemplations = vec![
        format!("Notice when {} is active. Is there effort? Is there a 'you' doing it?", gift),
        format!("What happens when {} operates without personal identity attached?", gift),
        format!("Can you feel the space where {} dissolves into {}?", gift, siddhi),
        "What remains when this gift serves without anyone serving?".to_string(),
        format!("Is {} a personal achievement or a universal expression?", gift),
    ];
    
    let witnessing_practices = vec![
        format!("You might notice when {} flows effortlessly, observing if there's a 'you' claiming it.", gift),
        format!("You might explore the edge where {} becomes impersonal, no longer 'yours'.", gift),
        "You might rest in the paradox: serving without a server, expressing without an expresser.".to_string(),
        format!("You might allow {} to operate through you rather than as you.", gift),
    ];
    
    let gift_to_siddhi_inquiry = Some(format!(
        "When you can express {} fully, what dissolves? \
        What space opens when {} releases its personal vantage point? \
        Can you sense {} as a field rather than a quality you possess?",
        gift, gift, siddhi
    ));
    
    TransformationPathway {
        gene_key: gene_key.number,
        name: gene_key.name.clone(),
        current_frequency: Frequency::Gift,
        next_frequency: Frequency::Siddhi,
        core_inquiry,
        contemplations,
        witnessing_practices,
        shadow_to_gift_inquiry: None,
        gift_to_siddhi_inquiry,
    }
}

/// Create Siddhi integration pathway
fn create_siddhi_integration_pathway(gene_key: &GeneKey) -> TransformationPathway {
    let siddhi = &gene_key.siddhi;
    
    let core_inquiry = format!(
        "When {} is present, what serves? How does the impersonal express through the personal?",
        siddhi
    );
    
    let contemplations = vec![
        format!("Notice when {} is simply present, without arrival or achievement.", siddhi),
        format!("Can {} be ordinary? What happens when the sacred becomes everyday?", siddhi),
        "Is there integration still needed, or does the question itself dissolve?".to_string(),
        format!("What does {} serve in this moment, through this form?", siddhi),
    ];
    
    let witnessing_practices = vec![
        format!("You might rest in {} without needing to understand or explain it.", siddhi),
        format!("You might allow {} to move through daily life, moment by moment.", siddhi),
        "You might notice when spiritual experience becomes simple presence.".to_string(),
    ];
    
    TransformationPathway {
        gene_key: gene_key.number,
        name: gene_key.name.clone(),
        current_frequency: Frequency::Siddhi,
        next_frequency: Frequency::Siddhi, // No next - integration ongoing
        core_inquiry,
        contemplations,
        witnessing_practices,
        shadow_to_gift_inquiry: None,
        gift_to_siddhi_inquiry: None,
    }
}

/// Generate comprehensive pathways for all frequencies
/// Returns both Shadow→Gift and Gift→Siddhi for complete journey
pub fn generate_complete_pathways(
    assessments: &[FrequencyAssessment],
) -> Vec<TransformationPathway> {
    let mut pathways = Vec::new();
    
    for assessment in assessments {
        if let Some(gene_key) = get_gene_key(assessment.gene_key) {
            // Always generate both transitions for complete journey view
            pathways.push(create_shadow_to_gift_pathway(gene_key));
            pathways.push(create_gift_to_siddhi_pathway(gene_key));
        }
    }
    
    pathways
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frequency::{assess_frequencies, RecognitionPrompts};
    use crate::models::{ActivationSequence, ActivationSource, GeneKeyActivation, GeneKeysChart};
    
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
            ],
        }
    }
    
    #[test]
    fn test_generate_transformation_pathways() {
        let chart = create_test_chart();
        let assessments = assess_frequencies(&chart, Some(2)); // Shadow level
        let pathways = generate_transformation_pathways(&assessments);
        
        assert_eq!(pathways.len(), 1);
        assert_eq!(pathways[0].gene_key, 17);
        assert_eq!(pathways[0].current_frequency, Frequency::Shadow);
        assert_eq!(pathways[0].next_frequency, Frequency::Gift);
    }
    
    #[test]
    fn test_shadow_to_gift_pathway() {
        let gene_key = get_gene_key(17).unwrap(); // Opinion → Far-Sightedness
        let pathway = create_shadow_to_gift_pathway(gene_key);
        
        assert_eq!(pathway.current_frequency, Frequency::Shadow);
        assert_eq!(pathway.next_frequency, Frequency::Gift);
        assert!(pathway.shadow_to_gift_inquiry.is_some());
        assert!(pathway.gift_to_siddhi_inquiry.is_none());
        assert!(!pathway.contemplations.is_empty());
        assert!(!pathway.witnessing_practices.is_empty());
    }
    
    #[test]
    fn test_gift_to_siddhi_pathway() {
        let gene_key = get_gene_key(17).unwrap();
        let pathway = create_gift_to_siddhi_pathway(gene_key);
        
        assert_eq!(pathway.current_frequency, Frequency::Gift);
        assert_eq!(pathway.next_frequency, Frequency::Siddhi);
        assert!(pathway.shadow_to_gift_inquiry.is_none());
        assert!(pathway.gift_to_siddhi_inquiry.is_some());
    }
    
    #[test]
    fn test_non_prescriptive_language() {
        let gene_key = get_gene_key(17).unwrap();
        let pathway = create_shadow_to_gift_pathway(gene_key);
        
        // Verify inquiry format (questions, not commands)
        assert!(pathway.core_inquiry.contains('?'));
        
        // Verify NO prescriptive language
        let inquiry = pathway.shadow_to_gift_inquiry.unwrap();
        assert!(!inquiry.contains("You must"));
        assert!(!inquiry.contains("You should"));
        assert!(!inquiry.contains("Do this"));
        
        // Verify contemplations are invitations
        for contemplation in &pathway.contemplations {
            let lower = contemplation.to_lowercase();
            assert!(
                contemplation.contains('?') || lower.contains("notice") || lower.contains("can you"),
                "Contemplation should be inquiry or witnessing: {}",
                contemplation
            );
        }
        
        // Verify witnessing practices use "might" (invitational)
        for practice in &pathway.witnessing_practices {
            assert!(
                practice.contains("might") || practice.contains("could"),
                "Practice should be invitational: {}",
                practice
            );
        }
    }
    
    #[test]
    fn test_siddhi_integration_pathway() {
        let gene_key = get_gene_key(1).unwrap();
        let pathway = create_siddhi_integration_pathway(gene_key);
        
        assert_eq!(pathway.current_frequency, Frequency::Siddhi);
        assert_eq!(pathway.next_frequency, Frequency::Siddhi); // No next
        assert!(pathway.shadow_to_gift_inquiry.is_none());
        assert!(pathway.gift_to_siddhi_inquiry.is_none());
    }
    
    #[test]
    fn test_complete_pathways_all_transitions() {
        let chart = create_test_chart();
        let assessments = assess_frequencies(&chart, None);
        let pathways = generate_complete_pathways(&assessments);
        
        // Should generate both Shadow→Gift and Gift→Siddhi
        assert_eq!(pathways.len(), 2); // 1 key * 2 transitions
        
        // Verify both transitions present
        let shadow_pathway = pathways.iter()
            .find(|p| p.current_frequency == Frequency::Shadow)
            .expect("Shadow→Gift pathway should exist");
        let gift_pathway = pathways.iter()
            .find(|p| p.current_frequency == Frequency::Gift)
            .expect("Gift→Siddhi pathway should exist");
        
        assert_eq!(shadow_pathway.next_frequency, Frequency::Gift);
        assert_eq!(gift_pathway.next_frequency, Frequency::Siddhi);
    }
    
    #[test]
    fn test_inquiry_contains_questions() {
        let gene_key = get_gene_key(1).unwrap();
        let shadow_pathway = create_shadow_to_gift_pathway(gene_key);
        let gift_pathway = create_gift_to_siddhi_pathway(gene_key);
        
        // All inquiries should contain questions
        assert!(shadow_pathway.core_inquiry.contains('?'));
        assert!(gift_pathway.core_inquiry.contains('?'));
        
        if let Some(inquiry) = &shadow_pathway.shadow_to_gift_inquiry {
            assert!(inquiry.contains('?'));
        }
        
        if let Some(inquiry) = &gift_pathway.gift_to_siddhi_inquiry {
            assert!(inquiry.contains('?'));
        }
    }
}
