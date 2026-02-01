//! Integration of TCM organ clock with Panchanga qualities
//!
//! Combines the organ clock system with Vedic Panchanga elements
//! to provide enhanced temporal recommendations.

use chrono::{DateTime, Utc};
use crate::models::{OrganWindow, TemporalRecommendation, ActivityRecommendation};
use crate::calculator::get_current_organ;
use crate::dosha::get_dosha_for_hour;
use crate::panchanga_qualities::{get_combined_quality, PanchangaQuality, QualityRating};

/// Generate a complete temporal recommendation for the current time
///
/// # Arguments
/// * `datetime` - UTC datetime
/// * `timezone_offset` - Offset from UTC in minutes
/// * `tithi_index` - Optional Tithi index (0-14)
/// * `nakshatra_index` - Optional Nakshatra index (0-26)
pub fn get_temporal_recommendation(
    datetime: DateTime<Utc>,
    timezone_offset: i32,
    tithi_index: Option<u8>,
    nakshatra_index: Option<u8>,
) -> TemporalRecommendation {
    let organ = get_current_organ(datetime, timezone_offset);
    let local_hour = crate::calculator::get_local_hour(datetime, timezone_offset);
    let dosha = get_dosha_for_hour(local_hour);
    
    // Get Panchanga quality if available
    let panchanga = get_combined_quality(tithi_index, nakshatra_index);
    
    // Generate activity recommendations based on organ + dosha + panchanga
    let activities = generate_activities(&organ, &panchanga);
    
    let panchanga_quality = if tithi_index.is_some() || nakshatra_index.is_some() {
        Some(format!("{}: {}", panchanga.rating.display(), panchanga.description))
    } else {
        None
    };

    TemporalRecommendation {
        time_window: organ.time_range_display(),
        organ: organ.organ,
        dosha: dosha.dosha,
        activities,
        panchanga_quality,
    }
}

/// Generate activity recommendations based on organ and panchanga
fn generate_activities(organ: &OrganWindow, panchanga: &PanchangaQuality) -> Vec<ActivityRecommendation> {
    let mut activities = Vec::new();
    
    // Add organ-based recommendations
    for activity in &organ.recommended_activities {
        let quality = match panchanga.rating {
            QualityRating::Excellent => "optimal",
            QualityRating::Good => "favorable",
            QualityRating::Neutral => "neutral",
            QualityRating::Challenging => "use caution",
        };
        
        activities.push(ActivityRecommendation {
            activity: activity.clone(),
            quality: quality.to_string(),
            reason: format!(
                "{} time ({}) - {} energy",
                organ.organ.display_name(),
                organ.element.display_name(),
                organ.peak_energy
            ),
        });
    }
    
    // Add Panchanga-favorable activities
    for favorable in &panchanga.favorable_for {
        if !activities.iter().any(|a| a.activity.to_lowercase() == favorable.to_lowercase()) {
            activities.push(ActivityRecommendation {
                activity: favorable.clone(),
                quality: "panchanga-favored".to_string(),
                reason: panchanga.description.clone(),
            });
        }
    }
    
    // Add cautions for Panchanga-unfavorable activities
    for avoid in &panchanga.avoid {
        activities.push(ActivityRecommendation {
            activity: avoid.clone(),
            quality: "avoid".to_string(),
            reason: format!("Panchanga advises caution: {}", panchanga.description),
        });
    }
    
    activities
}

/// Check if an activity is favorable at the given time
///
/// # Arguments
/// * `activity` - The activity to check
/// * `organ` - Current organ window
/// * `panchanga` - Current panchanga quality
///
/// # Returns
/// A score from 0.0 (unfavorable) to 1.0 (highly favorable)
pub fn get_activity_favorability(
    activity: &str,
    organ: &OrganWindow,
    panchanga: &PanchangaQuality,
) -> f64 {
    let activity_lower = activity.to_lowercase();
    
    // Check if activity is in organ's recommended activities
    let organ_bonus: f64 = if organ.recommended_activities
        .iter()
        .any(|a| a.to_lowercase().contains(&activity_lower) || activity_lower.contains(&a.to_lowercase()))
    {
        0.3
    } else {
        0.0
    };
    
    // Check if activity is favorable per Panchanga
    let panchanga_bonus: f64 = if panchanga.favorable_for
        .iter()
        .any(|f| f.to_lowercase().contains(&activity_lower) || activity_lower.contains(&f.to_lowercase()))
    {
        0.2
    } else {
        0.0
    };
    
    // Check if activity should be avoided
    let avoid_penalty: f64 = if panchanga.avoid
        .iter()
        .any(|a| a.to_lowercase().contains(&activity_lower) || activity_lower.contains(&a.to_lowercase()))
    {
        -0.3
    } else {
        0.0
    };
    
    // Base score from Panchanga rating
    let base_score = panchanga.rating.to_score();
    
    // Calculate final score (clamped to 0.0-1.0)
    (base_score + organ_bonus + panchanga_bonus + avoid_penalty).clamp(0.0, 1.0)
}

/// Synthesize organ clock with dosha time for enhanced recommendation
pub fn synthesize_organ_dosha(
    datetime: DateTime<Utc>,
    timezone_offset: i32,
) -> String {
    let organ = get_current_organ(datetime, timezone_offset);
    let local_hour = crate::calculator::get_local_hour(datetime, timezone_offset);
    let dosha = get_dosha_for_hour(local_hour);
    
    // Calculate harmony between organ and dosha
    let harmony = crate::dosha::calculate_dosha_organ_harmony(&organ.organ, &dosha.dosha);
    
    let harmony_desc = if harmony >= 0.8 {
        "harmonious"
    } else if harmony >= 0.5 {
        "balanced"
    } else {
        "transitional"
    };
    
    format!(
        "{} ({}) time during {} period - {} energy. {}",
        organ.organ.display_name(),
        organ.element.display_name(),
        dosha.dosha.display_name(),
        harmony_desc,
        organ.peak_energy
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_get_temporal_recommendation_basic() {
        let dt = Utc.with_ymd_and_hms(2024, 1, 1, 8, 0, 0).unwrap();
        let rec = get_temporal_recommendation(dt, 0, None, None);
        
        assert_eq!(rec.organ, crate::models::Organ::Stomach);
        assert!(!rec.activities.is_empty());
        assert!(rec.panchanga_quality.is_none());
    }

    #[test]
    fn test_get_temporal_recommendation_with_panchanga() {
        let dt = Utc.with_ymd_and_hms(2024, 1, 1, 8, 0, 0).unwrap();
        let rec = get_temporal_recommendation(dt, 0, Some(2), Some(3)); // Tritiya + Soft
        
        assert!(rec.panchanga_quality.is_some());
        let quality = rec.panchanga_quality.unwrap();
        assert!(quality.contains("Excellent") || quality.contains("Good"));
    }

    #[test]
    fn test_generate_activities() {
        let organ = crate::wisdom::get_organ_for_hour(8); // Stomach
        let panchanga = get_combined_quality(Some(2), None);
        
        let activities = generate_activities(&organ, &panchanga);
        assert!(!activities.is_empty());
        
        // Should have eating-related activity during Stomach time
        assert!(activities.iter().any(|a| 
            a.activity.to_lowercase().contains("eat") || 
            a.activity.to_lowercase().contains("nourish")));
    }

    #[test]
    fn test_activity_favorability() {
        let organ = crate::wisdom::get_organ_for_hour(8); // Stomach
        let panchanga = get_combined_quality(Some(2), None); // Tritiya (auspicious)
        
        // Eating should be favorable during Stomach time
        let eating_score = get_activity_favorability("eating", &organ, &panchanga);
        assert!(eating_score > 0.5, "Eating should be favorable during Stomach time");
    }

    #[test]
    fn test_synthesize_organ_dosha() {
        let dt = Utc.with_ymd_and_hms(2024, 1, 1, 8, 0, 0).unwrap();
        let synthesis = synthesize_organ_dosha(dt, 0);
        
        assert!(synthesis.contains("Stomach"));
        assert!(synthesis.contains("Earth"));
    }
}
