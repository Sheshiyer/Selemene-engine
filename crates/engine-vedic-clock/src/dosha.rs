//! Dosha-Organ correspondence and Ayurvedic time cycles
//!
//! Maps the three Ayurvedic doshas (Vata, Pitta, Kapha) to TCM organ times
//! and provides dosha-based time period calculations.

use crate::models::{Dosha, DoshaTime, Organ};

/// Get the Ayurvedic dosha time periods
///
/// Doshas cycle twice in 24 hours:
/// - Kapha: 6-10 AM and 6-10 PM
/// - Pitta: 10 AM-2 PM and 10 PM-2 AM  
/// - Vata: 2-6 AM and 2-6 PM
pub fn dosha_times() -> Vec<DoshaTime> {
    vec![
        // Vata morning: 2-6 AM
        DoshaTime {
            dosha: Dosha::Vata,
            start_hour: 2,
            end_hour: 6,
            qualities: vec![
                "Movement".to_string(),
                "Creativity".to_string(),
                "Lightness".to_string(),
                "Variability".to_string(),
                "Spiritual sensitivity".to_string(),
            ],
        },
        // Kapha morning: 6-10 AM
        DoshaTime {
            dosha: Dosha::Kapha,
            start_hour: 6,
            end_hour: 10,
            qualities: vec![
                "Stability".to_string(),
                "Groundedness".to_string(),
                "Calmness".to_string(),
                "Heaviness".to_string(),
                "Nurturing".to_string(),
            ],
        },
        // Pitta midday: 10 AM-2 PM
        DoshaTime {
            dosha: Dosha::Pitta,
            start_hour: 10,
            end_hour: 14,
            qualities: vec![
                "Transformation".to_string(),
                "Digestion".to_string(),
                "Mental sharpness".to_string(),
                "Drive".to_string(),
                "Heat".to_string(),
            ],
        },
        // Vata afternoon: 2-6 PM
        DoshaTime {
            dosha: Dosha::Vata,
            start_hour: 14,
            end_hour: 18,
            qualities: vec![
                "Movement".to_string(),
                "Communication".to_string(),
                "Flexibility".to_string(),
                "Completion energy".to_string(),
            ],
        },
        // Kapha evening: 6-10 PM
        DoshaTime {
            dosha: Dosha::Kapha,
            start_hour: 18,
            end_hour: 22,
            qualities: vec![
                "Winding down".to_string(),
                "Heaviness".to_string(),
                "Relaxation".to_string(),
                "Connection".to_string(),
            ],
        },
        // Pitta night: 10 PM-2 AM
        DoshaTime {
            dosha: Dosha::Pitta,
            start_hour: 22,
            end_hour: 2,
            qualities: vec![
                "Internal cleansing".to_string(),
                "Metabolic processing".to_string(),
                "Dream activity".to_string(),
                "Cellular repair".to_string(),
            ],
        },
    ]
}

/// Get the dosha for a specific hour
pub fn get_dosha_for_hour(hour: u8) -> DoshaTime {
    match hour {
        2..=5 => dosha_times()[0].clone(),   // Vata AM
        6..=9 => dosha_times()[1].clone(),   // Kapha AM
        10..=13 => dosha_times()[2].clone(), // Pitta midday
        14..=17 => dosha_times()[3].clone(), // Vata PM
        18..=21 => dosha_times()[4].clone(), // Kapha PM
        22..=23 | 0..=1 => dosha_times()[5].clone(), // Pitta night
        _ => dosha_times()[0].clone(), // Default
    }
}

/// Map TCM organs to their associated dosha affinity
///
/// This creates a bridge between TCM and Ayurveda:
/// - Vata organs: those dealing with movement, elimination, fluids
/// - Pitta organs: those dealing with transformation, digestion, heat
/// - Kapha organs: those dealing with structure, nourishment, protection
pub fn get_organ_dosha_affinity(organ: &Organ) -> Dosha {
    match organ {
        // Vata-dominant organs (movement, elimination, nervous system)
        Organ::LargeIntestine => Dosha::Vata,
        Organ::Bladder => Dosha::Vata,
        Organ::Kidney => Dosha::Vata,
        
        // Pitta-dominant organs (transformation, metabolism, heat)
        Organ::Stomach => Dosha::Pitta,
        Organ::SmallIntestine => Dosha::Pitta,
        Organ::Heart => Dosha::Pitta,
        Organ::Liver => Dosha::Pitta,
        Organ::Gallbladder => Dosha::Pitta,
        
        // Kapha-dominant organs (structure, protection, fluids)
        Organ::Lung => Dosha::Kapha,
        Organ::Spleen => Dosha::Kapha,
        Organ::Pericardium => Dosha::Kapha,
        Organ::TripleWarmer => Dosha::Kapha,
    }
}

/// Get recommendations for balancing a dosha during its peak time
pub fn get_dosha_balancing_tips(dosha: &Dosha) -> Vec<&'static str> {
    match dosha {
        Dosha::Vata => vec![
            "Stay warm and grounded",
            "Follow regular routines",
            "Eat warm, nourishing foods",
            "Practice gentle, slow movements",
            "Use calming, grounding practices",
            "Avoid excessive stimulation",
        ],
        Dosha::Pitta => vec![
            "Stay cool and calm",
            "Avoid overheating",
            "Eat cooling foods",
            "Practice moderation",
            "Take breaks from intense activity",
            "Cultivate patience and compassion",
        ],
        Dosha::Kapha => vec![
            "Stay active and moving",
            "Rise early (before Kapha time)",
            "Eat lighter, warming foods",
            "Seek stimulation and variety",
            "Avoid excessive sleep or rest",
            "Embrace change and new experiences",
        ],
    }
}

/// Determine dosha harmony based on organ and dosha alignment
///
/// Returns a score (0.0-1.0) indicating how harmonious the current
/// organ time is with the current dosha time
pub fn calculate_dosha_organ_harmony(organ: &Organ, dosha: &Dosha) -> f64 {
    let organ_affinity = get_organ_dosha_affinity(organ);
    
    if organ_affinity == *dosha {
        // Perfect alignment - organ's dosha matches current dosha
        1.0
    } else {
        // Check for complementary relationships
        match (organ_affinity, dosha) {
            // Vata and Kapha balance each other (opposite qualities)
            (Dosha::Vata, Dosha::Kapha) | (Dosha::Kapha, Dosha::Vata) => 0.5,
            // Pitta mediates between Vata and Kapha
            (Dosha::Pitta, _) | (_, Dosha::Pitta) => 0.7,
            _ => 0.5,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dosha_times_complete() {
        let times = dosha_times();
        assert_eq!(times.len(), 6); // 3 doshas × 2 cycles
    }

    #[test]
    fn test_dosha_times_cover_24_hours() {
        // Each dosha period should be 4 hours
        for period in dosha_times() {
            let duration = if period.end_hour < period.start_hour {
                (24 - period.start_hour) + period.end_hour
            } else {
                period.end_hour - period.start_hour
            };
            assert_eq!(duration, 4, "Dosha period should be 4 hours");
        }
    }

    #[test]
    fn test_get_dosha_for_hour() {
        assert_eq!(get_dosha_for_hour(3).dosha, Dosha::Vata);
        assert_eq!(get_dosha_for_hour(7).dosha, Dosha::Kapha);
        assert_eq!(get_dosha_for_hour(12).dosha, Dosha::Pitta);
        assert_eq!(get_dosha_for_hour(15).dosha, Dosha::Vata);
        assert_eq!(get_dosha_for_hour(20).dosha, Dosha::Kapha);
        assert_eq!(get_dosha_for_hour(23).dosha, Dosha::Pitta);
        assert_eq!(get_dosha_for_hour(1).dosha, Dosha::Pitta);
    }

    #[test]
    fn test_organ_dosha_affinity() {
        assert_eq!(get_organ_dosha_affinity(&Organ::Lung), Dosha::Kapha);
        assert_eq!(get_organ_dosha_affinity(&Organ::Stomach), Dosha::Pitta);
        assert_eq!(get_organ_dosha_affinity(&Organ::Bladder), Dosha::Vata);
    }

    #[test]
    fn test_dosha_harmony_perfect_match() {
        // Lung (Kapha) during Kapha time
        let harmony = calculate_dosha_organ_harmony(&Organ::Lung, &Dosha::Kapha);
        assert!((harmony - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_dosha_harmony_mismatch() {
        // Stomach (Pitta) during Vata time
        let harmony = calculate_dosha_organ_harmony(&Organ::Stomach, &Dosha::Vata);
        assert!(harmony > 0.0 && harmony < 1.0);
    }

    #[test]
    fn test_balancing_tips_not_empty() {
        for dosha in [Dosha::Vata, Dosha::Pitta, Dosha::Kapha] {
            let tips = get_dosha_balancing_tips(&dosha);
            assert!(!tips.is_empty(), "{:?} should have balancing tips", dosha);
        }
    }
}
