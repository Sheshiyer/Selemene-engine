//! Mock face analysis generator
//!
//! Generates plausible face analysis results for the stub implementation.
//! Uses seeded random for reproducibility.

use crate::models::{
    BodyType, ConstitutionAnalysis, Dosha, Element, ElementalBalance,
    FaceAnalysis, FaceZone, HealthIndicator, PersonalityTrait,
};
use rand::prelude::*;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

/// Generate a mock face analysis with optional seed for reproducibility
pub fn generate_mock_analysis(seed: Option<u64>) -> FaceAnalysis {
    let mut rng = match seed {
        Some(s) => ChaCha8Rng::seed_from_u64(s),
        None => ChaCha8Rng::from_entropy(),
    };

    FaceAnalysis {
        constitution: generate_constitution(&mut rng),
        personality_indicators: generate_personality_traits(&mut rng),
        elemental_balance: generate_elemental_balance(&mut rng),
        health_indicators: generate_health_indicators(&mut rng),
        is_mock_data: true,
    }
}

/// Generate constitutional analysis
fn generate_constitution(rng: &mut ChaCha8Rng) -> ConstitutionAnalysis {
    let doshas = [Dosha::Vata, Dosha::Pitta, Dosha::Kapha];

    let primary_idx: usize = rng.gen_range(0..3);
    let primary_dosha = doshas[primary_idx];

    // 60% chance of having a secondary dosha
    let secondary_dosha = if rng.gen_bool(0.6) {
        let secondary_idx = (primary_idx + rng.gen_range(1..3)) % 3;
        Some(doshas[secondary_idx])
    } else {
        None
    };

    // Correlate element somewhat with dosha
    let tcm_element = match primary_dosha {
        Dosha::Vata => *[Element::Metal, Element::Water].choose(rng).unwrap(),
        Dosha::Pitta => *[Element::Fire, Element::Wood].choose(rng).unwrap(),
        Dosha::Kapha => *[Element::Earth, Element::Water].choose(rng).unwrap(),
    };

    // Correlate body type with dosha
    let body_type = match primary_dosha {
        Dosha::Vata => if rng.gen_bool(0.7) { BodyType::Ectomorph } else { BodyType::Mesomorph },
        Dosha::Pitta => if rng.gen_bool(0.7) { BodyType::Mesomorph } else { BodyType::Ectomorph },
        Dosha::Kapha => if rng.gen_bool(0.7) { BodyType::Endomorph } else { BodyType::Mesomorph },
    };

    ConstitutionAnalysis {
        primary_dosha,
        secondary_dosha,
        tcm_element,
        body_type,
    }
}

/// Generate 3-5 personality traits
fn generate_personality_traits(rng: &mut ChaCha8Rng) -> Vec<PersonalityTrait> {
    let all_traits = [
        ("Analytical Thinker", "high forehead", "Shows capacity for abstract thinking and long-term planning"),
        ("Strong-Willed", "prominent chin", "Indicates determination and follow-through in pursuits"),
        ("Intuitive", "wide-set eyes", "Suggests broad perspective and empathic understanding"),
        ("Detail-Oriented", "close-set eyes", "Shows focus and precision in observations"),
        ("Natural Leader", "strong cheekbones", "Indicates natural authority and presence"),
        ("Creative Visionary", "arched eyebrows", "Suggests selective, aesthetic sensibilities"),
        ("Diplomatic", "balanced features", "Shows ability to see multiple perspectives"),
        ("Passionate", "full lips", "Indicates expressiveness and sensual awareness"),
        ("Reserved", "thin lips", "Shows precision in communication and discernment"),
        ("Nurturing", "rounded face", "Suggests caring nature and emotional availability"),
        ("Determined", "strong jawline", "Indicates persistence and strong will"),
        ("Adaptable", "soft features", "Shows flexibility and openness to change"),
        ("Perceptive", "prominent eyes", "Indicates keen observation and insight"),
        ("Grounded", "strong nose", "Shows practical nature and self-confidence"),
    ];

    let count = rng.gen_range(3..=5);
    let selected: Vec<_> = all_traits.choose_multiple(rng, count).collect();

    selected
        .into_iter()
        .map(|(name, indicator, desc)| PersonalityTrait {
            trait_name: name.to_string(),
            facial_indicator: indicator.to_string(),
            description: desc.to_string(),
        })
        .collect()
}

/// Generate balanced elemental scores
fn generate_elemental_balance(rng: &mut ChaCha8Rng) -> ElementalBalance {
    // Start with base values
    let mut wood: f64 = rng.gen_range(0.1..0.4);
    let mut fire: f64 = rng.gen_range(0.1..0.4);
    let mut earth: f64 = rng.gen_range(0.1..0.4);
    let mut metal: f64 = rng.gen_range(0.1..0.4);
    let mut water: f64 = rng.gen_range(0.1..0.4);

    // Normalize to sum to 1.0
    let sum = wood + fire + earth + metal + water;
    wood /= sum;
    fire /= sum;
    earth /= sum;
    metal /= sum;
    water /= sum;

    ElementalBalance {
        wood,
        fire,
        earth,
        metal,
        water,
    }
}

/// Generate 2-3 health indicators
fn generate_health_indicators(rng: &mut ChaCha8Rng) -> Vec<HealthIndicator> {
    let all_indicators = [
        (FaceZone::Forehead, "Bladder/Small Intestine", "Area appears clear, suggesting good fluid balance"),
        (FaceZone::Forehead, "Mental processing", "Subtle lines present, may indicate active mental life"),
        (FaceZone::Eyebrows, "Liver/Gallbladder", "Good density suggesting healthy liver qi"),
        (FaceZone::Eyes, "Liver/Heart", "Brightness indicates good Shen (spirit)"),
        (FaceZone::Eyes, "Kidneys", "Under-eye area suggests adequate rest patterns"),
        (FaceZone::Cheeks, "Lungs", "Color indicates healthy lung qi"),
        (FaceZone::Cheeks, "Large Intestine", "Texture suggests good elimination patterns"),
        (FaceZone::Nose, "Heart/Spleen", "Coloration appears balanced"),
        (FaceZone::Mouth, "Spleen/Stomach", "Lip color suggests healthy digestion"),
        (FaceZone::Chin, "Kidneys/Hormones", "Area appears clear, suggesting hormonal balance"),
        (FaceZone::Ears, "Kidneys", "Earlobe quality suggests good constitutional strength"),
        (FaceZone::Jawline, "Digestive system", "Definition suggests healthy digestive fire"),
        (FaceZone::Temples, "Gallbladder", "Area appears relaxed, suggesting good qi flow"),
    ];

    let count = rng.gen_range(2..=3);
    let selected: Vec<_> = all_indicators.choose_multiple(rng, count).collect();

    selected
        .into_iter()
        .map(|(zone, organ, observation)| HealthIndicator {
            zone: *zone,
            associated_organ: organ.to_string(),
            observation: observation.to_string(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_mock_analysis() {
        let analysis = generate_mock_analysis(None);
        assert!(analysis.is_mock_data);
        assert!(!analysis.personality_indicators.is_empty());
        assert!(!analysis.health_indicators.is_empty());
    }

    #[test]
    fn test_seeded_reproducibility() {
        let analysis1 = generate_mock_analysis(Some(12345));
        let analysis2 = generate_mock_analysis(Some(12345));

        assert_eq!(
            analysis1.constitution.primary_dosha,
            analysis2.constitution.primary_dosha
        );
        assert_eq!(
            analysis1.constitution.tcm_element,
            analysis2.constitution.tcm_element
        );
    }

    #[test]
    fn test_personality_trait_count() {
        for _ in 0..10 {
            let analysis = generate_mock_analysis(None);
            let count = analysis.personality_indicators.len();
            assert!(count >= 3 && count <= 5);
        }
    }

    #[test]
    fn test_health_indicator_count() {
        for _ in 0..10 {
            let analysis = generate_mock_analysis(None);
            let count = analysis.health_indicators.len();
            assert!(count >= 2 && count <= 3);
        }
    }

    #[test]
    fn test_elemental_balance_sums_to_one() {
        let analysis = generate_mock_analysis(Some(42));
        let balance = &analysis.elemental_balance;
        let sum = balance.wood + balance.fire + balance.earth + balance.metal + balance.water;
        assert!((sum - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_different_seeds_different_results() {
        let analysis1 = generate_mock_analysis(Some(1));
        let analysis2 = generate_mock_analysis(Some(999999));

        // Very unlikely to be identical with different seeds
        let same = analysis1.constitution.primary_dosha == analysis2.constitution.primary_dosha
            && analysis1.constitution.tcm_element == analysis2.constitution.tcm_element
            && analysis1.personality_indicators.len() == analysis2.personality_indicators.len();
        
        // At least something should differ
        assert!(!same || analysis1.elemental_balance.wood != analysis2.elemental_balance.wood);
    }
}
