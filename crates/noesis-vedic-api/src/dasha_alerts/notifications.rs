//! Dasha notification generation

use chrono::NaiveDate;
use super::{DashaAlertConfig, DashaTransitionEvent, DashaTransitionType, TransitionSignificance};
use super::types::DashaAlert;

/// Generate alerts for upcoming transitions
pub fn generate_alerts(
    transitions: &[DashaTransitionEvent],
    config: &DashaAlertConfig,
    today: NaiveDate,
) -> Vec<DashaAlert> {
    let mut alerts = Vec::new();
    
    if !config.enabled {
        return alerts;
    }
    
    for transition in transitions {
        let alert_days = match transition.transition_type {
            DashaTransitionType::Mahadasha => &config.mahadasha_alert_days,
            DashaTransitionType::Antardasha => &config.antardasha_alert_days,
            DashaTransitionType::Pratyantardasha => {
                if config.include_pratyantardasha {
                    &config.antardasha_alert_days
                } else {
                    continue;
                }
            }
            DashaTransitionType::Sookshmadasha => continue,
        };
        
        for &days in alert_days {
            let alert_date = transition.transition_date - chrono::Duration::days(days as i64);
            
            if alert_date == today {
                let type_str = match transition.transition_type {
                    DashaTransitionType::Mahadasha => "Mahadasha",
                    DashaTransitionType::Antardasha => "Antardasha",
                    DashaTransitionType::Pratyantardasha => "Pratyantardasha",
                    DashaTransitionType::Sookshmadasha => "Sookshmadasha",
                };
                
                alerts.push(DashaAlert::new(
                    type_str,
                    &transition.from_lord,
                    &transition.to_lord,
                    transition.transition_date,
                    days,
                ));
            }
        }
    }
    
    alerts
}

/// Format notification message
pub fn format_notification(
    transition_type: DashaTransitionType,
    from_lord: &str,
    to_lord: &str,
    days_until: i64,
    significance: TransitionSignificance,
) -> String {
    let type_name = match transition_type {
        DashaTransitionType::Mahadasha => "Mahadasha",
        DashaTransitionType::Antardasha => "Antardasha",
        DashaTransitionType::Pratyantardasha => "Pratyantardasha",
        DashaTransitionType::Sookshmadasha => "Sookshmadasha",
    };
    
    let urgency = match days_until {
        0 => "TODAY!",
        1 => "Tomorrow!",
        2..=7 => "This week",
        8..=30 => "This month",
        31..=90 => "In the coming months",
        _ => "In the future",
    };
    
    let significance_note = match significance {
        TransitionSignificance::Major => "This is a MAJOR transition.",
        TransitionSignificance::Moderate => "A moderate shift in energy.",
        TransitionSignificance::Minor => "A subtle change.",
    };
    
    format!(
        "🌟 {} Transition {} 🌟\n\
         {} → {}\n\
         Days until change: {}\n\n\
         {}",
        type_name, urgency,
        from_lord, to_lord,
        days_until,
        significance_note
    )
}

/// Get recommended preparations for transition
pub fn get_transition_preparations(to_lord: &str) -> Vec<String> {
    match to_lord.to_lowercase().as_str() {
        "saturn" => vec![
            "Prepare for disciplined routine".to_string(),
            "Complete pending obligations".to_string(),
            "Strengthen health practices".to_string(),
            "Consider donating to elderly".to_string(),
        ],
        "jupiter" => vec![
            "Start learning something new".to_string(),
            "Connect with teachers or mentors".to_string(),
            "Plan for expansion".to_string(),
            "Engage in spiritual practices".to_string(),
        ],
        "rahu" => vec![
            "Ground yourself through meditation".to_string(),
            "Avoid impulsive decisions".to_string(),
            "Document important matters clearly".to_string(),
            "Stay connected to traditions".to_string(),
        ],
        "ketu" => vec![
            "Reflect on attachments".to_string(),
            "Practice detachment exercises".to_string(),
            "Pursue spiritual interests".to_string(),
            "Simplify lifestyle".to_string(),
        ],
        _ => vec![
            "Maintain regular routines".to_string(),
            "Stay adaptable to changes".to_string(),
            "Practice mindfulness".to_string(),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_notification() {
        let msg = format_notification(
            DashaTransitionType::Mahadasha,
            "Jupiter",
            "Saturn",
            7,
            TransitionSignificance::Major,
        );
        
        assert!(msg.contains("Mahadasha"));
        assert!(msg.contains("Jupiter"));
        assert!(msg.contains("Saturn"));
        assert!(msg.contains("MAJOR"));
    }

    #[test]
    fn test_preparations() {
        let preps = get_transition_preparations("Saturn");
        assert!(!preps.is_empty());
        assert!(preps.iter().any(|p| p.contains("disciplined")));
    }
}
