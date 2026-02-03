//! Hora-based recommendations for Vedic clock

use noesis_vedic_api::panchang::HoraTimings;

use crate::models::ActivityRecommendation;

pub fn recommendations_from_hora(horas: &HoraTimings, current_time: &str) -> Vec<ActivityRecommendation> {
    let mut recommendations = Vec::new();

    if let Some(current) = horas.get_current_hora(current_time) {
        let quality = if current.is_favorable { "favorable" } else { "neutral" };

        for activity in current.ruler.suitable_activities() {
            recommendations.push(ActivityRecommendation {
                activity: activity.to_string(),
                quality: quality.to_string(),
                reason: format!("{} Hora ({})", current.ruler.as_str(), current.quality),
            });
        }

        for avoid in current.ruler.activities_to_avoid() {
            recommendations.push(ActivityRecommendation {
                activity: avoid.to_string(),
                quality: "avoid".to_string(),
                reason: format!("Avoid during {} Hora", current.ruler.as_str()),
            });
        }
    }

    recommendations
}
