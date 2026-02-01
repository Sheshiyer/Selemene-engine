//! Optimal timing recommendations
//!
//! Provides recommendations for optimal timing of various activities
//! based on the combined TCM organ clock, Ayurvedic dosha cycles,
//! and optional Panchanga qualities.

use chrono::{DateTime, Utc};
use crate::models::{Activity, TimeWindow, Organ, Dosha};
use crate::wisdom::get_organ_for_hour;
use crate::dosha::{get_dosha_for_hour, get_organ_dosha_affinity};

/// Get optimal time windows for a specific activity
///
/// # Arguments
/// * `activity` - The activity to find optimal timing for
/// * `datetime` - Current UTC datetime (used to prioritize upcoming windows)
/// * `timezone_offset` - Offset from UTC in minutes
///
/// # Returns
/// Sorted list of time windows from most to least favorable
pub fn get_optimal_timing(
    activity: Activity,
    datetime: DateTime<Utc>,
    timezone_offset: i32,
) -> Vec<TimeWindow> {
    let current_hour = crate::calculator::get_local_hour(datetime, timezone_offset);
    
    // Get all 24-hour windows scored for this activity
    let mut windows = get_all_windows_for_activity(activity);
    
    // Boost upcoming windows (within next 6 hours)
    for window in &mut windows {
        let hours_until = hours_until_window(current_hour, window.start_hour);
        if hours_until <= 6 {
            window.quality += 0.1 * (1.0 - hours_until as f64 / 6.0);
        }
        window.quality = window.quality.clamp(0.0, 1.0);
    }
    
    // Sort by quality (highest first)
    windows.sort_by(|a, b| b.quality.partial_cmp(&a.quality).unwrap_or(std::cmp::Ordering::Equal));
    
    // Return top 5 windows
    windows.into_iter().take(5).collect()
}

/// Get all time windows scored for an activity
fn get_all_windows_for_activity(activity: Activity) -> Vec<TimeWindow> {
    let favorable_organs = get_favorable_organs(activity);
    let favorable_doshas = get_favorable_doshas(activity);
    
    let mut windows = Vec::new();
    
    // Check each 2-hour organ window
    for hour in (1..24).step_by(2) {
        let organ = get_organ_for_hour(hour as u8);
        let dosha = get_dosha_for_hour(hour as u8);
        
        // Calculate quality score
        let organ_score = if favorable_organs.contains(&organ.organ) { 0.4 } else { 0.1 };
        let dosha_score = if favorable_doshas.contains(&dosha.dosha) { 0.3 } else { 0.1 };
        let affinity_score = if get_organ_dosha_affinity(&organ.organ) == dosha.dosha { 0.2 } else { 0.0 };
        
        let quality = organ_score + dosha_score + affinity_score;
        
        // Generate reason
        let reason = generate_reason(activity, &organ.organ, &dosha.dosha, quality);
        
        windows.push(TimeWindow {
            start_hour: organ.start_hour,
            end_hour: organ.end_hour,
            quality,
            reason,
        });
    }
    
    windows
}

/// Get organs favorable for a specific activity
fn get_favorable_organs(activity: Activity) -> Vec<Organ> {
    match activity {
        Activity::Meditation => vec![
            Organ::Lung,          // 3-5 AM - deep breathing, stillness
            Organ::Liver,         // 1-3 AM - spiritual insight
            Organ::TripleWarmer,  // 9-11 PM - relaxation
            Organ::Kidney,        // 5-7 PM - willpower, depth
        ],
        Activity::Exercise => vec![
            Organ::Bladder,       // 3-5 PM - stored energy
            Organ::Stomach,       // 7-9 AM - after nourishment
            Organ::LargeIntestine, // 5-7 AM - release, movement
        ],
        Activity::Work => vec![
            Organ::Spleen,        // 9-11 AM - mental clarity
            Organ::SmallIntestine, // 1-3 PM - sorting, discernment
            Organ::Bladder,       // 3-5 PM - productivity
        ],
        Activity::Eating => vec![
            Organ::Stomach,       // 7-9 AM - optimal digestion
            Organ::Spleen,        // 9-11 AM - assimilation
            Organ::SmallIntestine, // 1-3 PM - sorting nutrients
        ],
        Activity::Sleep => vec![
            Organ::TripleWarmer,  // 9-11 PM - sleep prep
            Organ::Gallbladder,   // 11 PM-1 AM - deep sleep
            Organ::Liver,         // 1-3 AM - restoration
        ],
        Activity::Creative => vec![
            Organ::Liver,         // 1-3 AM - vision (if awake)
            Organ::Heart,         // 11 AM-1 PM - joy, expression
            Organ::Pericardium,   // 7-9 PM - emotional opening
        ],
        Activity::Social => vec![
            Organ::Heart,         // 11 AM-1 PM - connection
            Organ::Pericardium,   // 7-9 PM - intimacy
            Organ::Stomach,       // 7-9 AM - sharing meals
        ],
    }
}

/// Get doshas favorable for a specific activity
fn get_favorable_doshas(activity: Activity) -> Vec<Dosha> {
    match activity {
        Activity::Meditation => vec![Dosha::Vata, Dosha::Kapha],  // Stillness, awareness
        Activity::Exercise => vec![Dosha::Kapha],                  // Best to move during heavy time
        Activity::Work => vec![Dosha::Pitta],                      // Sharp mind, drive
        Activity::Eating => vec![Dosha::Pitta],                    // Strong digestion
        Activity::Sleep => vec![Dosha::Kapha, Dosha::Pitta],       // Heaviness, repair
        Activity::Creative => vec![Dosha::Vata],                   // Creative flow
        Activity::Social => vec![Dosha::Kapha, Dosha::Pitta],      // Connection, warmth
    }
}

/// Generate a reason string for a time window
fn generate_reason(activity: Activity, organ: &Organ, dosha: &Dosha, quality: f64) -> String {
    let quality_desc = if quality >= 0.7 {
        "excellent"
    } else if quality >= 0.5 {
        "good"
    } else if quality >= 0.3 {
        "moderate"
    } else {
        "less favorable"
    };
    
    format!(
        "{} timing for {} — {} ({}) time during {} period",
        capitalize(quality_desc),
        activity.display_name().to_lowercase(),
        organ.display_name(),
        crate::wisdom::get_organ_element(organ).display_name(),
        dosha.display_name()
    )
}

/// Calculate hours until a specific window starts
fn hours_until_window(current_hour: u8, window_start: u8) -> u8 {
    if window_start == current_hour {
        0
    } else if window_start > current_hour {
        window_start - current_hour
    } else {
        24 - current_hour + window_start
    }
}

/// Capitalize first letter
fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

/// Get the best single time window for an activity
pub fn get_best_time(activity: Activity) -> TimeWindow {
    let windows = get_all_windows_for_activity(activity);
    windows.into_iter()
        .max_by(|a, b| a.quality.partial_cmp(&b.quality).unwrap_or(std::cmp::Ordering::Equal))
        .unwrap_or(TimeWindow {
            start_hour: 9,
            end_hour: 11,
            quality: 0.5,
            reason: "Default recommendation".to_string(),
        })
}

/// Check if an activity is favorable right now
pub fn is_favorable_now(
    activity: Activity,
    datetime: DateTime<Utc>,
    timezone_offset: i32,
) -> (bool, String) {
    let current_hour = crate::calculator::get_local_hour(datetime, timezone_offset);
    let organ = get_organ_for_hour(current_hour);
    let dosha = get_dosha_for_hour(current_hour);
    
    let favorable_organs = get_favorable_organs(activity);
    let favorable_doshas = get_favorable_doshas(activity);
    
    let is_favorable = favorable_organs.contains(&organ.organ) || favorable_doshas.contains(&dosha.dosha);
    
    let reason = if is_favorable {
        format!(
            "Current {} time ({}) supports {}",
            organ.organ.display_name(),
            dosha.dosha.display_name(),
            activity.display_name().to_lowercase()
        )
    } else {
        let best = get_best_time(activity);
        format!(
            "Consider waiting until {} for better {} conditions",
            best.time_range_display(),
            activity.display_name().to_lowercase()
        )
    };
    
    (is_favorable, reason)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_get_optimal_timing_returns_sorted() {
        let dt = Utc.with_ymd_and_hms(2024, 1, 1, 10, 0, 0).unwrap();
        let windows = get_optimal_timing(Activity::Meditation, dt, 0);
        
        assert!(!windows.is_empty());
        
        // Check windows are sorted by quality (descending)
        for i in 1..windows.len() {
            assert!(windows[i-1].quality >= windows[i].quality,
                "Windows should be sorted by quality descending");
        }
    }

    #[test]
    fn test_eating_favors_stomach_time() {
        let favorable = get_favorable_organs(Activity::Eating);
        assert!(favorable.contains(&Organ::Stomach));
    }

    #[test]
    fn test_meditation_favors_early_morning() {
        let favorable = get_favorable_organs(Activity::Meditation);
        assert!(favorable.contains(&Organ::Lung)); // 3-5 AM
    }

    #[test]
    fn test_work_favors_spleen_time() {
        let favorable = get_favorable_organs(Activity::Work);
        assert!(favorable.contains(&Organ::Spleen)); // 9-11 AM
    }

    #[test]
    fn test_get_best_time() {
        let best = get_best_time(Activity::Work);
        // Work should be favorable during Spleen (9-11 AM) or similar
        assert!(best.quality > 0.0);
    }

    #[test]
    fn test_is_favorable_now() {
        // At 8 AM (Stomach time), eating should be favorable
        let dt = Utc.with_ymd_and_hms(2024, 1, 1, 8, 0, 0).unwrap();
        let (favorable, reason) = is_favorable_now(Activity::Eating, dt, 0);
        
        assert!(favorable, "Eating should be favorable at 8 AM (Stomach time)");
        assert!(!reason.is_empty());
    }

    #[test]
    fn test_hours_until_window() {
        assert_eq!(hours_until_window(8, 10), 2);
        assert_eq!(hours_until_window(22, 3), 5); // Wraps around midnight
        assert_eq!(hours_until_window(5, 5), 0);
    }

    #[test]
    fn test_optimal_timing_limits_results() {
        let dt = Utc.with_ymd_and_hms(2024, 1, 1, 10, 0, 0).unwrap();
        let windows = get_optimal_timing(Activity::Exercise, dt, 0);
        
        // Should return at most 5 windows
        assert!(windows.len() <= 5);
    }
}
