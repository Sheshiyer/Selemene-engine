//! Witness prompt generation for transit analysis
//!
//! Generates consciousness-level tiered questions based on transit aspects,
//! Sade Sati status, and planetary retrogrades.

use crate::models::{
    AspectNature, AspectType, PeriodQuality, SadeSatiPhase, TransitAnalysisResult, TransitPlanet,
};

/// Generate a witness prompt based on the transit analysis and consciousness level.
///
/// # Consciousness Levels
/// - 0: Basic awareness (simple observation questions)
/// - 1: Self-reflection (pattern recognition)
/// - 2: Deep inquiry (archetypal/psychological)
/// - 3: Transcendent (non-dual awareness)
pub fn generate_witness_prompt(
    analysis: &TransitAnalysisResult,
    consciousness_level: u8,
) -> String {
    let mut prompts = Vec::new();

    // Sade Sati prompt (always significant when active)
    if analysis.sade_sati.is_active {
        prompts.push(sade_sati_prompt(
            &analysis.sade_sati.phase,
            consciousness_level,
        ));
    }

    // Retrograde prompt
    if !analysis.retrograde_planets.is_empty() {
        prompts.push(retrograde_prompt(
            &analysis.retrograde_planets,
            consciousness_level,
        ));
    }

    // Aspect-based prompts (pick the most significant)
    if let Some(aspect) = analysis.aspects.first() {
        prompts.push(aspect_prompt(
            aspect.transiting_planet,
            aspect.natal_planet,
            aspect.aspect_type,
            aspect.nature,
            consciousness_level,
        ));
    }

    // Period quality prompt
    prompts.push(period_quality_prompt(
        analysis.period_quality,
        consciousness_level,
    ));

    prompts.join("\n\n")
}

fn sade_sati_prompt(phase: &Option<SadeSatiPhase>, level: u8) -> String {
    match level {
        0 => match phase {
            Some(SadeSatiPhase::Rising) => "Saturn approaches your Moon sign. What area of your life feels like it's being restructured?".to_string(),
            Some(SadeSatiPhase::Peak) => "Saturn sits upon your natal Moon. Where do you feel the weight of responsibility most strongly right now?".to_string(),
            Some(SadeSatiPhase::Setting) => "Saturn moves past your Moon sign. What lessons from recent challenges are becoming clear?".to_string(),
            None => "How does your emotional landscape feel in this moment?".to_string(),
        },
        1 => match phase {
            Some(SadeSatiPhase::Rising) => "Sade Sati's rising phase strips away what no longer serves. What identity or comfort are you being asked to release? Can you feel the difference between who you think you are and who you're becoming?".to_string(),
            Some(SadeSatiPhase::Peak) => "At the peak of Sade Sati, Saturn and Moon meet. Your emotional patterns face their deepest test. What emotional reaction keeps recurring that might be pointing to something your soul needs to integrate?".to_string(),
            Some(SadeSatiPhase::Setting) => "Sade Sati's setting phase brings consolidation. What new emotional foundation has been built from the rubble of old patterns? What strength did you not know you had?".to_string(),
            None => "What emotional patterns are you noticing that might be connected to deeper karmic themes?".to_string(),
        },
        2 => match phase {
            Some(SadeSatiPhase::Rising) => "The dissolution of Sade Sati's rising phase mirrors the alchemical nigredo. What in you is dying so that something truer can be born? Can you witness the death without becoming the grief?".to_string(),
            Some(SadeSatiPhase::Peak) => "Saturn on your Moon: the lord of time meets the seat of feeling. This is the crucible. What is the difference between the pain that transforms and the suffering that comes from resisting transformation?".to_string(),
            Some(SadeSatiPhase::Setting) => "As Sade Sati releases its grip, what do you now understand about impermanence that you couldn't have learned any other way? How has limitation revealed freedom?".to_string(),
            None => "How are your emotional responses pointing toward deeper archetypal patterns seeking expression?".to_string(),
        },
        _ => "If Saturn is the teacher and your Moon is the student, what lesson has already been learned that your mind hasn't caught up to? Can you rest in the knowing that precedes understanding?".to_string(),
    }
}

fn retrograde_prompt(planets: &[TransitPlanet], level: u8) -> String {
    let planet_names: Vec<&str> = planets.iter().map(|p| p.name()).collect();
    let names = planet_names.join(", ");

    match level {
        0 => format!("{} in retrograde — what areas feel like they need revisiting or slowing down?", names),
        1 => format!("With {} retrograde, the energy turns inward. What internal review is being called for? What did you rush past that now asks for your attention?", names),
        2 => format!("Retrograde {} invites a spiral rather than a line. What appears as going backward is actually going deeper. Where is the depth calling you that forward motion kept you from reaching?", names),
        _ => format!("{} in apparent retrograde. The cosmos mirrors: all forward motion contains return. What is the still point around which your apparent progress revolves?", names),
    }
}

fn aspect_prompt(
    transiting: TransitPlanet,
    natal: TransitPlanet,
    aspect_type: AspectType,
    nature: AspectNature,
    level: u8,
) -> String {
    let transit_name = transiting.name();
    let natal_name = natal.name();

    match (nature, level) {
        (AspectNature::Harmonious, 0) => format!("{} {} {} forms a supportive {} — what opportunities are opening up that you might overlook?", transit_name, aspect_type, natal_name, aspect_type),
        (AspectNature::Harmonious, 1) => format!("The {} between transit {} and your natal {} creates a channel of ease. But ease can breed complacency. How are you using this flow rather than just enjoying it?", aspect_type, transit_name, natal_name),
        (AspectNature::Harmonious, _) => format!("Transit {} in {} to natal {}. Grace and effort are not opposites. What would it look like to be fully receptive to support while remaining fully engaged?", transit_name, aspect_type, natal_name),
        (AspectNature::Challenging, 0) => format!("{} {} {} creates tension — where do you feel friction, and what might it be trying to show you?", transit_name, aspect_type, natal_name),
        (AspectNature::Challenging, 1) => format!("The {} between transit {} and natal {} generates creative tension. What is the growth edge here? What would you avoid if you could, that you actually need to face?", aspect_type, transit_name, natal_name),
        (AspectNature::Challenging, _) => format!("Transit {} {} natal {}. Pressure creates diamonds. What is being compressed in you right now, and can you trust that compression serves crystallization?", transit_name, aspect_type, natal_name),
        (AspectNature::Neutral, 0) => format!("{} conjunct your natal {} — what themes from both planets are merging in your experience right now?", transit_name, natal_name),
        (AspectNature::Neutral, 1) => format!("The conjunction of transit {} with natal {} blends their energies into something new. What is emerging from this fusion that neither planet could produce alone?", transit_name, natal_name),
        (AspectNature::Neutral, _) => format!("Transit {} meets natal {} in conjunction. Two become one. What is the third thing that arises when apparent opposites merge?", transit_name, natal_name),
    }
}

fn period_quality_prompt(quality: PeriodQuality, level: u8) -> String {
    match (quality, level) {
        (PeriodQuality::HighlyFavorable, 0) => "This is a highly supportive period. What intentions are you setting to make the most of this energy?".to_string(),
        (PeriodQuality::HighlyFavorable, _) => "Cosmic winds are at your back. The question isn't what you can achieve, but what is truly worth achieving. What would you create if you knew you couldn't fail?".to_string(),
        (PeriodQuality::Favorable, 0) => "Current transits support forward movement. What small step would create the most momentum?".to_string(),
        (PeriodQuality::Favorable, _) => "Favorable transits reveal what was always possible but required alignment to manifest. What has been waiting for the right moment?".to_string(),
        (PeriodQuality::Mixed, 0) => "Mixed energies today — some areas flow while others resist. Where do you feel the most clarity?".to_string(),
        (PeriodQuality::Mixed, _) => "Mixed transits mirror life's fundamental both/and nature. Can you hold the tension of opposites without collapsing into either pole?".to_string(),
        (PeriodQuality::Challenging, 0) => "Challenging transits are active. What is one thing you can do today that honors both the difficulty and your resilience?".to_string(),
        (PeriodQuality::Challenging, _) => "Challenging transits are invitations disguised as obstacles. What is the invitation hiding inside the challenge you're facing?".to_string(),
        (PeriodQuality::Difficult, 0) => "Difficult transits are in effect. Focus on what you can control and be gentle with yourself. What do you need most right now?".to_string(),
        (PeriodQuality::Difficult, _) => "Difficult transits break the shell so the seed can sprout. What is the shell? What is the seed? Can you be both the breaking and the becoming?".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{SadeSatiStatus, ZodiacSign};

    fn make_minimal_result(
        sade_sati_active: bool,
        retrogrades: Vec<TransitPlanet>,
    ) -> TransitAnalysisResult {
        TransitAnalysisResult {
            natal_positions: vec![],
            transit_positions: vec![],
            aspects: vec![],
            sade_sati: SadeSatiStatus {
                is_active: sade_sati_active,
                phase: if sade_sati_active {
                    Some(SadeSatiPhase::Peak)
                } else {
                    None
                },
                saturn_sign: ZodiacSign::Aquarius,
                moon_sign: ZodiacSign::Aquarius,
            },
            period_quality: PeriodQuality::Mixed,
            retrograde_planets: retrogrades,
        }
    }

    #[test]
    fn test_prompt_non_empty() {
        let result = make_minimal_result(false, vec![]);
        let prompt = generate_witness_prompt(&result, 0);
        assert!(!prompt.is_empty());
    }

    #[test]
    fn test_sade_sati_prompt_included() {
        let result = make_minimal_result(true, vec![]);
        let prompt = generate_witness_prompt(&result, 1);
        assert!(
            prompt.contains("Sade Sati"),
            "Should mention Sade Sati: {}",
            prompt
        );
    }

    #[test]
    fn test_retrograde_prompt_included() {
        let result =
            make_minimal_result(false, vec![TransitPlanet::Mercury, TransitPlanet::Saturn]);
        let prompt = generate_witness_prompt(&result, 0);
        assert!(
            prompt.contains("retrograde"),
            "Should mention retrograde: {}",
            prompt
        );
        assert!(prompt.contains("Mercury"));
        assert!(prompt.contains("Saturn"));
    }

    #[test]
    fn test_consciousness_levels_differ() {
        let result = make_minimal_result(true, vec![]);
        let p0 = generate_witness_prompt(&result, 0);
        let p2 = generate_witness_prompt(&result, 2);
        assert_ne!(
            p0, p2,
            "Different consciousness levels should produce different prompts"
        );
    }
}
