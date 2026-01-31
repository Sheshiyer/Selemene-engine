//! W1-S6-11: Consciousness-oriented witness prompts for Vimshottari periods
//!
//! Generates contextual prompts that adapt to consciousness level and integrate
//! current period qualities with upcoming transitions.

use crate::calculator::enrich_period_with_qualities;
use crate::models::{CurrentPeriod, UpcomingTransition, VedicPlanet};
use crate::wisdom_data::PLANETARY_PERIOD_QUALITIES;

/// Generate consciousness-adapted witness prompt
///
/// # Arguments
/// * `current_period` - Currently active 3-level period
/// * `upcoming_transitions` - Next N transitions (from calculate_upcoming_transitions)
/// * `consciousness_level` - User's consciousness level (0-6)
///
/// # Consciousness Levels
/// - 0-2: Beginner (concrete timing and life areas)
/// - 3-4: Intermediate (opportunities/challenges awareness)
/// - 5-6: Advanced (witnessing karmic patterns)
///
/// # Returns
/// Formatted prompt string adapted to consciousness level
pub fn generate_witness_prompt(
    current_period: &CurrentPeriod,
    upcoming_transitions: &[UpcomingTransition],
    consciousness_level: u8,
) -> String {
    let enrichment = enrich_period_with_qualities(
        &current_period.mahadasha.planet,
        &current_period.antardasha.planet,
        &current_period.pratyantardasha.planet,
    );
    
    match consciousness_level {
        0..=2 => generate_beginner_prompt(current_period, &enrichment, upcoming_transitions),
        3..=4 => generate_intermediate_prompt(current_period, &enrichment, upcoming_transitions),
        5..=6 => generate_advanced_prompt(current_period, &enrichment, upcoming_transitions),
        _ => generate_intermediate_prompt(current_period, &enrichment, upcoming_transitions),
    }
}

/// Beginner prompt: Focus on concrete timing and life areas
fn generate_beginner_prompt(
    current_period: &CurrentPeriod,
    enrichment: &crate::models::PeriodEnrichment,
    upcoming: &[UpcomingTransition],
) -> String {
    let next_transition = upcoming.first();
    
    let transition_text = if let Some(t) = next_transition {
        let next_qualities = PLANETARY_PERIOD_QUALITIES.get(&t.to_planet)
            .expect("Next planet qualities not found");
        format!(
            " In {} days, you'll transition to {}'s period, bringing a shift toward {}.",
            t.days_until,
            t.to_planet.as_str(),
            next_qualities.themes.first().unwrap_or(&"new themes".to_string())
        )
    } else {
        String::new()
    };
    
    format!(
        "You are currently in {}'s Mahadasha, {}'s Antardasha, and {}'s Pratyantardasha (until {}). This {} period emphasizes {}. Life areas to focus on: {}.{}",
        current_period.mahadasha.planet.as_str(),
        current_period.antardasha.planet.as_str(),
        current_period.pratyantardasha.planet.as_str(),
        current_period.pratyantardasha.end.format("%B %d, %Y"),
        current_period.pratyantardasha.planet.as_str(),
        enrichment.pratyantardasha_themes.join(", "),
        enrichment.life_areas.join(", "),
        transition_text
    )
}

/// Intermediate prompt: Add opportunities/challenges awareness
fn generate_intermediate_prompt(
    current_period: &CurrentPeriod,
    enrichment: &crate::models::PeriodEnrichment,
    _upcoming: &[UpcomingTransition],
) -> String {
    format!(
        "What is this {}-{}-{} period revealing about your journey? Notice how {} (Mahadasha) provides the backdrop, {} (Antardasha) colors the experience, and {} (Pratyantardasha) brings immediate focus to {}. Opportunities present: {}. Challenges to navigate: {}. How are you meeting these themes in your life right now?",
        current_period.mahadasha.planet.as_str(),
        current_period.antardasha.planet.as_str(),
        current_period.pratyantardasha.planet.as_str(),
        current_period.mahadasha.planet.as_str(),
        current_period.antardasha.planet.as_str(),
        current_period.pratyantardasha.planet.as_str(),
        enrichment.pratyantardasha_themes.join(", "),
        enrichment.opportunities.join(", "),
        enrichment.challenges.join(", "),
    )
}

/// Advanced prompt: Consciousness-level awareness (witnessing karmic patterns)
fn generate_advanced_prompt(
    current_period: &CurrentPeriod,
    enrichment: &crate::models::PeriodEnrichment,
    upcoming: &[UpcomingTransition],
) -> String {
    let next_transition_text = if let Some(t) = upcoming.first() {
        format!(
            " How does the approaching transition to {} (in {} days) invite you to prepare your consciousness?",
            t.to_planet.as_str(),
            t.days_until
        )
    } else {
        String::new()
    };
    
    format!(
        "You are in the conscious field of {}'s influence, nested within {}'s container, illuminated by {}'s immediate presence. What karmic patterns are ripening? What is seeking release through {}? Beyond the themes of {} and {}, what wants to be witnessed in pure awareness?{}",
        current_period.pratyantardasha.planet.as_str(),
        current_period.antardasha.planet.as_str(),
        current_period.mahadasha.planet.as_str(),
        enrichment.challenges.join(" and "),
        enrichment.opportunities.join(", "),
        enrichment.challenges.join(", "),
        next_transition_text
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};
    use crate::models::{CurrentMahadasha, CurrentAntardasha, CurrentPratyantardasha, TransitionLevel};

    fn create_test_current_period() -> CurrentPeriod {
        let start = Utc.with_ymd_and_hms(2020, 1, 1, 0, 0, 0).unwrap();
        let end = Utc.with_ymd_and_hms(2026, 5, 31, 0, 0, 0).unwrap();
        
        CurrentPeriod {
            mahadasha: CurrentMahadasha {
                planet: VedicPlanet::Venus,
                start,
                end,
                years: 20.0,
            },
            antardasha: CurrentAntardasha {
                planet: VedicPlanet::Mercury,
                start,
                end,
                years: 2.833,
            },
            pratyantardasha: CurrentPratyantardasha {
                planet: VedicPlanet::Jupiter,
                start,
                end,
                days: 104.0,
            },
            current_time: Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap(),
        }
    }

    #[test]
    fn test_witness_prompt_references_planets() {
        let current = create_test_current_period();
        let upcoming = vec![];
        
        let prompt = generate_witness_prompt(&current, &upcoming, 3);
        
        // Must reference all 3 current planets
        assert!(prompt.contains("Venus"));
        assert!(prompt.contains("Mercury"));
        assert!(prompt.contains("Jupiter"));
    }
    
    #[test]
    fn test_consciousness_level_adaptation() {
        let current = create_test_current_period();
        let upcoming = vec![];
        
        let prompt_1 = generate_witness_prompt(&current, &upcoming, 1);
        let prompt_3 = generate_witness_prompt(&current, &upcoming, 3);
        let prompt_6 = generate_witness_prompt(&current, &upcoming, 6);
        
        // All prompts should be different
        assert_ne!(prompt_1, prompt_3);
        assert_ne!(prompt_3, prompt_6);
        
        // Beginner should mention dates
        assert!(prompt_1.contains("until") || prompt_1.contains("May"));
        
        // Advanced should mention "consciousness" or "awareness"
        assert!(prompt_6.contains("consciousness") || prompt_6.contains("awareness"));
    }
    
    #[test]
    fn test_upcoming_transition_integration() {
        let current = create_test_current_period();
        let transition_date = Utc.with_ymd_and_hms(2025, 5, 1, 0, 0, 0).unwrap();
        
        let upcoming = vec![
            UpcomingTransition {
                transition_type: TransitionLevel::Pratyantardasha,
                from_planet: VedicPlanet::Jupiter,
                to_planet: VedicPlanet::Saturn,
                transition_date,
                days_until: 120,
            }
        ];
        
        let prompt = generate_witness_prompt(&current, &upcoming, 2);
        
        // Should mention upcoming transition
        assert!(prompt.contains("Saturn"));
        assert!(prompt.contains("120 days"));
    }
    
    #[test]
    fn test_beginner_prompt_contains_concrete_details() {
        let current = create_test_current_period();
        let upcoming = vec![];
        
        let prompt = generate_witness_prompt(&current, &upcoming, 1);
        
        // Should contain concrete timing
        assert!(prompt.contains("Mahadasha") && prompt.contains("Antardasha") && prompt.contains("Pratyantardasha"));
        
        // Should contain life areas
        assert!(prompt.contains("Life areas") || prompt.contains("focus on"));
    }
    
    #[test]
    fn test_intermediate_prompt_asks_questions() {
        let current = create_test_current_period();
        let upcoming = vec![];
        
        let prompt = generate_witness_prompt(&current, &upcoming, 3);
        
        // Should be inquiry-based (contains question marks)
        assert!(prompt.contains("?"));
        
        // Should mention opportunities and challenges
        assert!(prompt.contains("Opportunities") || prompt.contains("Challenges"));
    }
    
    #[test]
    fn test_advanced_prompt_consciousness_language() {
        let current = create_test_current_period();
        let upcoming = vec![];
        
        let prompt = generate_witness_prompt(&current, &upcoming, 6);
        
        // Should use consciousness/spiritual language
        assert!(
            prompt.contains("consciousness") 
            || prompt.contains("awareness") 
            || prompt.contains("karmic")
        );
        
        // Should be inquiry-based
        assert!(prompt.contains("?"));
    }
    
    #[test]
    fn test_enrichment_all_planets_have_data() {
        // Test that all 9 planets have enrichment data
        let planets = vec![
            VedicPlanet::Sun, VedicPlanet::Moon, VedicPlanet::Mars,
            VedicPlanet::Mercury, VedicPlanet::Jupiter, VedicPlanet::Venus,
            VedicPlanet::Saturn, VedicPlanet::Rahu, VedicPlanet::Ketu,
        ];
        
        for planet in planets {
            let enrichment = enrich_period_with_qualities(&planet, &planet, &planet);
            
            // Should have themes
            assert!(!enrichment.mahadasha_themes.is_empty());
            assert!(!enrichment.antardasha_themes.is_empty());
            assert!(!enrichment.pratyantardasha_themes.is_empty());
            
            // Should have description
            assert!(!enrichment.combined_description.is_empty());
            
            // Should have life areas
            assert!(!enrichment.life_areas.is_empty());
            
            // Should have opportunities
            assert!(!enrichment.opportunities.is_empty());
            
            // Should have challenges
            assert!(!enrichment.challenges.is_empty());
        }
    }
}
