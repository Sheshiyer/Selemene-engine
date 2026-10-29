//! Choghadiya-based recommendations for Vedic clock

use noesis_vedic_api::panchang::{
    get_activities_to_avoid, get_suitable_activities, ActivityCategory, ChoghadiyaName,
    ChoghadiyaNature, ChoghadiyaTimings,
};

use crate::models::ActivityRecommendation;
use crate::vedic_time::ChoghadiyaPeriod;

pub fn recommendations_from_choghadiya(
    timings: &ChoghadiyaTimings,
    current_time: &str,
) -> Vec<ActivityRecommendation> {
    match timings.get_current(current_time) {
        Some(current) => period_recommendations(
            current.name,
            current.nature,
            &current.ruler,
            &current.suitable_for,
            &current.avoid,
        ),
        None => Vec::new(),
    }
}

/// Recommendations for a choghadiya computed from local sunrise and sunset.
pub fn recommendations_for_choghadiya_period(
    choghadiya: &ChoghadiyaPeriod,
) -> Vec<ActivityRecommendation> {
    period_recommendations(
        choghadiya.name,
        choghadiya.nature,
        choghadiya.ruler.as_str(),
        &get_suitable_activities(choghadiya.name),
        &get_activities_to_avoid(choghadiya.name),
    )
}

fn period_recommendations(
    name: ChoghadiyaName,
    nature: ChoghadiyaNature,
    ruler: &str,
    suitable_for: &[ActivityCategory],
    avoid: &[ActivityCategory],
) -> Vec<ActivityRecommendation> {
    let mut recommendations: Vec<ActivityRecommendation> = suitable_for
        .iter()
        .map(|activity| ActivityRecommendation {
            activity: activity_label(*activity),
            quality: nature.as_str().to_string(),
            reason: format!("{} Choghadiya ({})", name.as_str(), ruler),
        })
        .collect();

    recommendations.extend(avoid.iter().map(|activity| ActivityRecommendation {
        activity: activity_label(*activity),
        quality: "avoid".to_string(),
        reason: format!("Avoid during {} Choghadiya", name.as_str()),
    }));

    recommendations
}

pub(crate) fn activity_label(activity: ActivityCategory) -> String {
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
