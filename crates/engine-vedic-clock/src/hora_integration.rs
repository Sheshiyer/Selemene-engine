//! Hora-based recommendations for Vedic clock

use noesis_vedic_api::panchang::{HoraTimings, Planet};

use crate::models::ActivityRecommendation;
use crate::vedic_time::{hora_is_favorable, HoraPeriod};

pub fn recommendations_from_hora(
    horas: &HoraTimings,
    current_time: &str,
) -> Vec<ActivityRecommendation> {
    match horas.get_current_hora(current_time) {
        Some(current) => ruler_recommendations(
            current.ruler,
            current.is_favorable,
            &format!("{} Hora ({})", current.ruler.as_str(), current.quality),
        ),
        None => Vec::new(),
    }
}

/// Recommendations for a hora computed from local sunrise and sunset.
pub fn recommendations_for_hora_period(hora: &HoraPeriod) -> Vec<ActivityRecommendation> {
    ruler_recommendations(
        hora.ruler,
        hora_is_favorable(hora.ruler),
        &format!("{} Hora", hora.ruler.as_str()),
    )
}

fn ruler_recommendations(
    ruler: Planet,
    is_favorable: bool,
    reason: &str,
) -> Vec<ActivityRecommendation> {
    let quality = if is_favorable { "favorable" } else { "neutral" };

    let mut recommendations: Vec<ActivityRecommendation> = ruler
        .suitable_activities()
        .into_iter()
        .map(|activity| ActivityRecommendation {
            activity: activity.to_string(),
            quality: quality.to_string(),
            reason: reason.to_string(),
        })
        .collect();

    recommendations.extend(ruler.activities_to_avoid().into_iter().map(|avoid| {
        ActivityRecommendation {
            activity: avoid.to_string(),
            quality: "avoid".to_string(),
            reason: format!("Avoid during {} Hora", ruler.as_str()),
        }
    }));

    recommendations
}
