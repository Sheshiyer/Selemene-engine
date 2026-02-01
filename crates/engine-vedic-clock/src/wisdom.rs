//! TCM Organ Clock wisdom data
//!
//! Contains the complete 12-organ clock mapping with associated
//! elements, emotions, and recommended activities for each 2-hour window.

use crate::models::{Element, Organ, OrganWindow};

/// Get the complete organ clock data for all 12 organs
pub fn organ_clock() -> Vec<OrganWindow> {
    vec![
        // 3-5 AM: Lung (Metal)
        OrganWindow {
            organ: Organ::Lung,
            element: Element::Metal,
            start_hour: 3,
            end_hour: 5,
            peak_energy: "Qi flows through lungs, oxygenating body for new day".to_string(),
            associated_emotion: "Grief (imbalanced) / Inspiration (balanced)".to_string(),
            recommended_activities: vec![
                "Deep breathing exercises".to_string(),
                "Meditation".to_string(),
                "Gentle pranayama".to_string(),
                "Quiet contemplation".to_string(),
                "Natural awakening".to_string(),
            ],
        },
        // 5-7 AM: Large Intestine (Metal)
        OrganWindow {
            organ: Organ::LargeIntestine,
            element: Element::Metal,
            start_hour: 5,
            end_hour: 7,
            peak_energy: "Time of release and elimination".to_string(),
            associated_emotion: "Letting go (balanced) / Holding on (imbalanced)".to_string(),
            recommended_activities: vec![
                "Bowel movement".to_string(),
                "Drinking warm water".to_string(),
                "Light stretching".to_string(),
                "Morning hygiene routine".to_string(),
                "Release of what no longer serves".to_string(),
            ],
        },
        // 7-9 AM: Stomach (Earth)
        OrganWindow {
            organ: Organ::Stomach,
            element: Element::Earth,
            start_hour: 7,
            end_hour: 9,
            peak_energy: "Optimal digestion and nourishment".to_string(),
            associated_emotion: "Trust (balanced) / Worry (imbalanced)".to_string(),
            recommended_activities: vec![
                "Eating largest meal".to_string(),
                "Mindful eating".to_string(),
                "Nourishing breakfast".to_string(),
                "Grounding activities".to_string(),
                "Setting intentions".to_string(),
            ],
        },
        // 9-11 AM: Spleen (Earth)
        OrganWindow {
            organ: Organ::Spleen,
            element: Element::Earth,
            start_hour: 9,
            end_hour: 11,
            peak_energy: "Mental clarity and concentration peak".to_string(),
            associated_emotion: "Thoughtfulness (balanced) / Overthinking (imbalanced)".to_string(),
            recommended_activities: vec![
                "Intellectual work".to_string(),
                "Studying".to_string(),
                "Problem-solving".to_string(),
                "Important meetings".to_string(),
                "Strategic planning".to_string(),
            ],
        },
        // 11 AM-1 PM: Heart (Fire)
        OrganWindow {
            organ: Organ::Heart,
            element: Element::Fire,
            start_hour: 11,
            end_hour: 13,
            peak_energy: "Joy and connection flourish".to_string(),
            associated_emotion: "Joy (balanced) / Anxiety (imbalanced)".to_string(),
            recommended_activities: vec![
                "Social connection".to_string(),
                "Sharing lunch with others".to_string(),
                "Heart-centered activities".to_string(),
                "Expressing love".to_string(),
                "Light-hearted conversation".to_string(),
            ],
        },
        // 1-3 PM: Small Intestine (Fire)
        OrganWindow {
            organ: Organ::SmallIntestine,
            element: Element::Fire,
            start_hour: 13,
            end_hour: 15,
            peak_energy: "Sorting, assimilating, and discernment".to_string(),
            associated_emotion: "Discernment (balanced) / Confusion (imbalanced)".to_string(),
            recommended_activities: vec![
                "Organizing tasks".to_string(),
                "Email and correspondence".to_string(),
                "Sorting through information".to_string(),
                "Decision-making".to_string(),
                "Assimilating morning's learning".to_string(),
            ],
        },
        // 3-5 PM: Bladder (Water)
        OrganWindow {
            organ: Organ::Bladder,
            element: Element::Water,
            start_hour: 15,
            end_hour: 17,
            peak_energy: "Stored energy available for work".to_string(),
            associated_emotion: "Flow (balanced) / Fear (imbalanced)".to_string(),
            recommended_activities: vec![
                "Focused work".to_string(),
                "Physical exercise".to_string(),
                "Completing projects".to_string(),
                "Hydration".to_string(),
                "Afternoon productivity".to_string(),
            ],
        },
        // 5-7 PM: Kidney (Water)
        OrganWindow {
            organ: Organ::Kidney,
            element: Element::Water,
            start_hour: 17,
            end_hour: 19,
            peak_energy: "Restoration and willpower".to_string(),
            associated_emotion: "Willpower (balanced) / Fear (imbalanced)".to_string(),
            recommended_activities: vec![
                "Winding down work".to_string(),
                "Light evening meal".to_string(),
                "Restorative yoga".to_string(),
                "Reflecting on the day".to_string(),
                "Replenishing energy".to_string(),
            ],
        },
        // 7-9 PM: Pericardium (Fire)
        OrganWindow {
            organ: Organ::Pericardium,
            element: Element::Fire,
            start_hour: 19,
            end_hour: 21,
            peak_energy: "Protection and emotional intimacy".to_string(),
            associated_emotion: "Openness (balanced) / Guardedness (imbalanced)".to_string(),
            recommended_activities: vec![
                "Quality time with loved ones".to_string(),
                "Intimacy".to_string(),
                "Self-care rituals".to_string(),
                "Relaxation".to_string(),
                "Emotional connection".to_string(),
            ],
        },
        // 9-11 PM: Triple Warmer (Fire)
        OrganWindow {
            organ: Organ::TripleWarmer,
            element: Element::Fire,
            start_hour: 21,
            end_hour: 23,
            peak_energy: "Balancing all body systems for sleep".to_string(),
            associated_emotion: "Harmony (balanced) / Overwhelm (imbalanced)".to_string(),
            recommended_activities: vec![
                "Preparing for sleep".to_string(),
                "Gentle relaxation".to_string(),
                "Avoiding screens".to_string(),
                "Light reading".to_string(),
                "Evening meditation".to_string(),
            ],
        },
        // 11 PM-1 AM: Gallbladder (Wood)
        OrganWindow {
            organ: Organ::Gallbladder,
            element: Element::Wood,
            start_hour: 23,
            end_hour: 1,
            peak_energy: "Decision and courage crystallize in sleep".to_string(),
            associated_emotion: "Decisiveness (balanced) / Indecision (imbalanced)".to_string(),
            recommended_activities: vec![
                "Deep sleep".to_string(),
                "Processing decisions".to_string(),
                "Dream incubation".to_string(),
                "Being asleep ideally".to_string(),
            ],
        },
        // 1-3 AM: Liver (Wood)
        OrganWindow {
            organ: Organ::Liver,
            element: Element::Wood,
            start_hour: 1,
            end_hour: 3,
            peak_energy: "Blood cleansing and planning in dreams".to_string(),
            associated_emotion: "Vision (balanced) / Anger (imbalanced)".to_string(),
            recommended_activities: vec![
                "Deep restorative sleep".to_string(),
                "Dreaming".to_string(),
                "Body detoxification".to_string(),
                "Subconscious planning".to_string(),
            ],
        },
    ]
}

/// Get the organ window for a specific hour
pub fn get_organ_for_hour(hour: u8) -> OrganWindow {
    let clock = organ_clock();
    
    // Handle the special wrap-around case for Gallbladder (23-1)
    match hour {
        3..=4 => clock[0].clone(),   // Lung
        5..=6 => clock[1].clone(),   // Large Intestine
        7..=8 => clock[2].clone(),   // Stomach
        9..=10 => clock[3].clone(),  // Spleen
        11..=12 => clock[4].clone(), // Heart
        13..=14 => clock[5].clone(), // Small Intestine
        15..=16 => clock[6].clone(), // Bladder
        17..=18 => clock[7].clone(), // Kidney
        19..=20 => clock[8].clone(), // Pericardium
        21..=22 => clock[9].clone(), // Triple Warmer
        23 | 0 => clock[10].clone(), // Gallbladder
        1..=2 => clock[11].clone(),  // Liver
        _ => clock[0].clone(),       // Default to Lung (shouldn't happen with u8)
    }
}

/// Get organ-element correspondence
pub fn get_organ_element(organ: &Organ) -> Element {
    match organ {
        Organ::Lung | Organ::LargeIntestine => Element::Metal,
        Organ::Stomach | Organ::Spleen => Element::Earth,
        Organ::Heart | Organ::SmallIntestine | Organ::Pericardium | Organ::TripleWarmer => Element::Fire,
        Organ::Bladder | Organ::Kidney => Element::Water,
        Organ::Gallbladder | Organ::Liver => Element::Wood,
    }
}

/// Get the opposing organ (12 hours apart in the cycle)
pub fn get_opposing_organ(organ: &Organ) -> Organ {
    match organ {
        Organ::Lung => Organ::Bladder,
        Organ::LargeIntestine => Organ::Kidney,
        Organ::Stomach => Organ::Pericardium,
        Organ::Spleen => Organ::TripleWarmer,
        Organ::Heart => Organ::Gallbladder,
        Organ::SmallIntestine => Organ::Liver,
        Organ::Bladder => Organ::Lung,
        Organ::Kidney => Organ::LargeIntestine,
        Organ::Pericardium => Organ::Stomach,
        Organ::TripleWarmer => Organ::Spleen,
        Organ::Gallbladder => Organ::Heart,
        Organ::Liver => Organ::SmallIntestine,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_organ_clock_complete() {
        let clock = organ_clock();
        assert_eq!(clock.len(), 12);
    }

    #[test]
    fn test_organ_clock_covers_24_hours() {
        let clock = organ_clock();
        
        // Each window should be 2 hours
        for window in &clock {
            let duration = if window.end_hour < window.start_hour {
                // Wraps around midnight
                (24 - window.start_hour) + window.end_hour
            } else {
                window.end_hour - window.start_hour
            };
            assert_eq!(duration, 2, "Window for {:?} should be 2 hours", window.organ);
        }
    }

    #[test]
    fn test_get_organ_for_hour() {
        assert_eq!(get_organ_for_hour(3).organ, Organ::Lung);
        assert_eq!(get_organ_for_hour(4).organ, Organ::Lung);
        assert_eq!(get_organ_for_hour(7).organ, Organ::Stomach);
        assert_eq!(get_organ_for_hour(12).organ, Organ::Heart);
        assert_eq!(get_organ_for_hour(15).organ, Organ::Bladder);
        assert_eq!(get_organ_for_hour(23).organ, Organ::Gallbladder);
        assert_eq!(get_organ_for_hour(0).organ, Organ::Gallbladder);
        assert_eq!(get_organ_for_hour(1).organ, Organ::Liver);
    }

    #[test]
    fn test_organ_element_mapping() {
        assert_eq!(get_organ_element(&Organ::Lung), Element::Metal);
        assert_eq!(get_organ_element(&Organ::Heart), Element::Fire);
        assert_eq!(get_organ_element(&Organ::Stomach), Element::Earth);
        assert_eq!(get_organ_element(&Organ::Kidney), Element::Water);
        assert_eq!(get_organ_element(&Organ::Liver), Element::Wood);
    }

    #[test]
    fn test_opposing_organs() {
        // 12 hours apart
        assert_eq!(get_opposing_organ(&Organ::Lung), Organ::Bladder);
        assert_eq!(get_opposing_organ(&Organ::Bladder), Organ::Lung);
        assert_eq!(get_opposing_organ(&Organ::Heart), Organ::Gallbladder);
    }

    #[test]
    fn test_each_organ_has_activities() {
        let clock = organ_clock();
        for window in &clock {
            assert!(!window.recommended_activities.is_empty(),
                "{:?} should have recommended activities", window.organ);
        }
    }
}
