//! Choghadiya-based recommendations for Vedic clock

use noesis_vedic_api::panchang::{ChoghadiyaTimings, ActivityCategory};

use crate::models::ActivityRecommendation;

pub fn recommendations_from_choghadiya(timings: &ChoghadiyaTimings, current_time: &str) -> Vec<ActivityRecommendation> {
    let mut recommendations = Vec::new();

    if let Some(current) = timings.get_current(current_time) {
        for activity in &current.suitable_for {
            recommendations.push(ActivityRecommendation {
                activity: activity_label(*activity),
                quality: current.nature.as_str().to_string(),
                reason: format!("{} Choghadiya ({})", current.name.as_str(), current.ruler),
            });
        }

        for activity in &current.avoid {
            recommendations.push(ActivityRecommendation {
                activity: activity_label(*activity),
                quality: "avoid".to_string(),
                reason: format!("Avoid during {} Choghadiya", current.name.as_str()),
            });
        }
    }

    recommendations
}

fn activity_label(activity: ActivityCategory) -> String {
    match activity {
        ActivityCategory::StartingNew => "Starting new ventures".to_string(),
        ActivityCategory::Business => "Business".to_string(),
        ActivityCategory::Education => "Education".to_string(),
        ActivityCategory::Travel => "Travel".to_string(),
        ActivityCategory::Religious => "Religious activities".to_string(),
        ActivityCategory::Medical => "Medical".to_string(),
        ActivityCategory::Legal => "Legal matters".to_string(),
        ActivityCategory::Construction => "Construction".to_string(),
        ActivityCategory::Marriage => "Marriage".to_string(),
        ActivityCategory::Purchasing => "Purchasing".to_string(),
        ActivityCategory::Any => "General".to_string(),
    }
}
