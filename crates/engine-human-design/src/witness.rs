//! Witness prompt generation for Human Design consciousness
//!
//! Generates inquiry-based self-observation prompts tailored to:
//! - User's consciousness level (1-2 basic, 3-4 intermediate, 5+ advanced)
//! - HD Type and Strategy
//! - Authority
//! - Profile
//! - Definition patterns

use crate::{HDChart, HDType, Authority, Profile, Definition};

/// Generate a consciousness-oriented witness prompt based on HD chart and level.
///
/// # Arguments
/// * `chart` - The complete Human Design chart
/// * `consciousness_level` - User's current consciousness phase (0-5)
///
/// # Returns
/// An inquiry-based question designed to cultivate self-awareness
pub fn generate_witness_prompt(chart: &HDChart, consciousness_level: u8) -> String {
    match consciousness_level {
        0..=2 => generate_basic_witness(chart),
        3..=4 => generate_intermediate_witness(chart),
        _ => generate_advanced_witness(chart),
    }
}

/// Level 1-2: Basic body awareness (Type/Strategy focus)
fn generate_basic_witness(chart: &HDChart) -> String {
    match chart.hd_type {
        HDType::Generator => {
            "What does it feel like in your body when you wait to respond to life's invitations rather than initiating?"
        }
        HDType::ManifestingGenerator => {
            "How do you experience the pull to respond quickly and then skip steps once momentum builds?"
        }
        HDType::Projector => {
            "How do you experience recognition when it arrives naturally, without effort or seeking?"
        }
        HDType::Manifestor => {
            "Where do you notice the urge to initiate before informing others of what's coming?"
        }
        HDType::Reflector => {
            "What happens in your awareness when you give yourself time to sense the full lunar cycle before deciding?"
        }
    }.to_string()
}

/// Level 3-4: Intermediate awareness (Profile + Authority dynamics)
fn generate_intermediate_witness(chart: &HDChart) -> String {
    // Profile-based prompts
    let profile_key = format!("{}/{}", chart.profile.conscious_line, chart.profile.unconscious_line);
    
    let profile_prompt = match profile_key.as_str() {
        "1/3" => "How do you experience the dance between deep investigation and experiential learning through trial and error?",
        "1/4" => "Where do you notice your need for solid foundation meeting your natural gift for sharing within networks?",
        "2/4" => "What happens when your natural talent calls you out from solitude into the realm of connection and networking?",
        "2/5" => "How do you experience being called out to solve problems when you'd rather remain in your natural state?",
        "3/5" => "Where do you see yourself experimenting until breakthrough, then being projected upon to provide solutions?",
        "3/6" => "How do you navigate the transition from experimental engagement to observational wisdom?",
        "4/6" => "What happens when your network-building nature meets your need for experimentation and eventual observation?",
        "4/1" => "How do you bridge your gift for connection with your need for investigative depth and security?",
        "5/1" => "Where do you notice the tension between being projected upon for solutions and your need for solid foundation?",
        "5/2" => "How do you experience being called to solve problems when your natural state is to be called out from within?",
        "6/2" => "What does it feel like to be on the roof observing life while also being called down to share natural gifts?",
        "6/3" => "How do you dance between objective observation and the pull to experiment directly with life?",
        _ => "How do you experience the interplay between your conscious and unconscious life themes?",
    };
    
    profile_prompt.to_string()
}

/// Level 5+: Advanced awareness (Authority + Definition patterns)
fn generate_advanced_witness(chart: &HDChart) -> String {
    // Authority-based deep inquiries
    let authority_prompt = match chart.authority {
        Authority::Sacral => {
            "Where do you notice the sacral response arising in the present moment, distinct from mental narrative?"
        }
        Authority::Emotional => {
            "How do you experience yourself riding the emotional wave before making decisions, without forcing clarity?"
        }
        Authority::Splenic => {
            "What is it like to trust the instantaneous knowing that arises and vanishes in a single moment?"
        }
        Authority::Heart => {
            "Where do you feel the alignment between what you will commit to and your authentic power?"
        }
        Authority::GCenter => {
            "How do you distinguish between true direction arising from your G center versus mental constructs?"
        }
        Authority::Mental => {
            "What happens when you verbalize your thoughts in different environments before arriving at clarity?"
        }
        Authority::Lunar => {
            "How do you experience the full lunar cycle revealing consistent truth beyond transient impressions?"
        }
    };
    
    // Add definition awareness if applicable
    let definition_layer = match chart.definition {
        Definition::Split => " And how do you notice the bridging energy when others enter your field, connecting what feels separate?",
        Definition::TripleSplit => " And what is your experience of needing multiple bridges to feel a sense of wholeness?",
        Definition::QuadrupleSplit => " And how do you witness the complexity of multiple separate islands within your design seeking connection?",
        Definition::NoDefinition => " And what is it like to be completely open, sampling and reflecting the energy around you?",
        Definition::Single => "",
    };
    
    format!("{}{}", authority_prompt, definition_layer)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use crate::{Center, CenterState};
    
    fn create_test_chart(hd_type: HDType, authority: Authority, profile: Profile, definition: Definition) -> HDChart {
        HDChart {
            personality_activations: vec![],
            design_activations: vec![],
            centers: HashMap::new(),
            channels: vec![],
            hd_type,
            authority,
            profile,
            definition,
        }
    }
    
    #[test]
    fn test_basic_generator_prompt() {
        let chart = create_test_chart(
            HDType::Generator,
            Authority::Sacral,
            Profile { conscious_line: 1, unconscious_line: 3 },
            Definition::Single,
        );
        
        let prompt = generate_witness_prompt(&chart, 1);
        assert!(prompt.contains("wait to respond"));
        assert!(!prompt.is_empty());
    }
    
    #[test]
    fn test_intermediate_profile_prompt() {
        let chart = create_test_chart(
            HDType::Projector,
            Authority::Splenic,
            Profile { conscious_line: 6, unconscious_line: 2 },
            Definition::Single,
        );
        
        let prompt = generate_witness_prompt(&chart, 3);
        assert!(prompt.contains("roof observing") || prompt.contains("called down"));
        assert!(!prompt.is_empty());
    }
    
    #[test]
    fn test_advanced_authority_prompt() {
        let chart = create_test_chart(
            HDType::Generator,
            Authority::Emotional,
            Profile { conscious_line: 3, unconscious_line: 5 },
            Definition::Split,
        );
        
        let prompt = generate_witness_prompt(&chart, 5);
        assert!(prompt.contains("wave") || prompt.contains("emotional"));
        assert!(prompt.contains("bridging"));
        assert!(!prompt.is_empty());
    }
    
    #[test]
    fn test_all_prompts_non_empty() {
        let types = [HDType::Generator, HDType::ManifestingGenerator, HDType::Projector, HDType::Manifestor, HDType::Reflector];
        let authorities = [Authority::Sacral, Authority::Emotional, Authority::Splenic, Authority::Heart, Authority::GCenter, Authority::Mental, Authority::Lunar];
        
        for hd_type in &types {
            for authority in &authorities {
                let chart = create_test_chart(
                    *hd_type,
                    *authority,
                    Profile { conscious_line: 1, unconscious_line: 3 },
                    Definition::Single,
                );
                
                for level in 0..=5 {
                    let prompt = generate_witness_prompt(&chart, level);
                    assert!(!prompt.is_empty(), "Empty prompt for {:?}, {:?}, level {}", hd_type, authority, level);
                    assert!(prompt.ends_with('?'), "Prompt should be a question: {}", prompt);
                }
            }
        }
    }
}
