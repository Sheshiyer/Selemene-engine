//! Current organ and time calculations
//!
//! Provides functions to determine the active organ window
//! based on current time and timezone.

use chrono::{DateTime, Timelike, Utc};
use crate::models::OrganWindow;
use crate::wisdom::get_organ_for_hour;

/// Get the organ window for the current datetime in a specific timezone
///
/// # Arguments
/// * `datetime` - UTC datetime
/// * `timezone_offset` - Offset from UTC in minutes (e.g., +330 for IST, -480 for PST)
///
/// # Returns
/// The OrganWindow active at the specified local time
pub fn get_current_organ(datetime: DateTime<Utc>, timezone_offset: i32) -> OrganWindow {
    let local_hour = get_local_hour(datetime, timezone_offset);
    get_organ_for_hour(local_hour)
}

/// Convert UTC datetime to local hour
///
/// # Arguments
/// * `datetime` - UTC datetime
/// * `timezone_offset` - Offset from UTC in minutes
///
/// # Returns
/// The hour (0-23) in local time
pub fn get_local_hour(datetime: DateTime<Utc>, timezone_offset: i32) -> u8 {
    let utc_hour = datetime.hour() as i32;
    let utc_minute = datetime.minute() as i32;
    
    // Convert to total minutes from midnight UTC
    let total_utc_minutes = utc_hour * 60 + utc_minute;
    
    // Add timezone offset
    let total_local_minutes = total_utc_minutes + timezone_offset;
    
    // Handle day wraparound
    let normalized_minutes = ((total_local_minutes % (24 * 60)) + (24 * 60)) % (24 * 60);
    
    // Convert back to hour
    (normalized_minutes / 60) as u8
}

/// Get minutes until the next organ transition
///
/// # Arguments
/// * `datetime` - UTC datetime
/// * `timezone_offset` - Offset from UTC in minutes
///
/// # Returns
/// Minutes until the next organ window begins
pub fn minutes_until_next_transition(datetime: DateTime<Utc>, timezone_offset: i32) -> u32 {
    let local_hour = get_local_hour(datetime, timezone_offset);
    let local_minute = datetime.minute();
    
    // Find the next transition hour (organs change on odd hours: 3, 5, 7, 9, 11, 13, 15, 17, 19, 21, 23, 1)
    let next_transition_hour = match local_hour {
        0 => 1,
        1 | 2 => 3,
        3 | 4 => 5,
        5 | 6 => 7,
        7 | 8 => 9,
        9 | 10 => 11,
        11 | 12 => 13,
        13 | 14 => 15,
        15 | 16 => 17,
        17 | 18 => 19,
        19 | 20 => 21,
        21 | 22 => 23,
        23 => 1, // Wraps to next day
        _ => 1,
    };
    
    let hours_until = if next_transition_hour <= local_hour {
        // Wraps to next day
        (24 - local_hour as u32) + next_transition_hour as u32
    } else {
        (next_transition_hour - local_hour) as u32
    };
    
    // Calculate total minutes
    hours_until * 60 - local_minute
}

/// Get the progress through the current organ window (0.0 - 1.0)
///
/// # Arguments
/// * `datetime` - UTC datetime
/// * `timezone_offset` - Offset from UTC in minutes
///
/// # Returns
/// Progress as a fraction (0.0 = just started, 1.0 = about to transition)
pub fn get_window_progress(datetime: DateTime<Utc>, timezone_offset: i32) -> f64 {
    let local_hour = get_local_hour(datetime, timezone_offset);
    let local_minute = datetime.minute();
    
    // Each window is 2 hours (120 minutes)
    // Find which hour within the 2-hour window we're in
    let window_start_hour = match local_hour {
        0 => 23,     // Gallbladder window (23-1)
        h if h % 2 == 1 => h, // Odd hours are window starts
        h => h - 1,   // Even hours are in the second hour of window
    };
    
    let minutes_into_window = if local_hour == window_start_hour {
        local_minute
    } else if local_hour == 0 && window_start_hour == 23 {
        60 + local_minute
    } else {
        60 + local_minute
    };
    
    minutes_into_window as f64 / 120.0
}

/// Format the current time window as a display string
pub fn format_time_window(datetime: DateTime<Utc>, timezone_offset: i32) -> String {
    let organ = get_current_organ(datetime, timezone_offset);
    organ.time_range_display()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_get_local_hour_utc() {
        let dt = Utc.with_ymd_and_hms(2024, 1, 1, 10, 30, 0).unwrap();
        assert_eq!(get_local_hour(dt, 0), 10);
    }

    #[test]
    fn test_get_local_hour_positive_offset() {
        // IST is UTC+5:30 (+330 minutes)
        let dt = Utc.with_ymd_and_hms(2024, 1, 1, 6, 0, 0).unwrap();
        assert_eq!(get_local_hour(dt, 330), 11); // 6:00 UTC = 11:30 IST
    }

    #[test]
    fn test_get_local_hour_negative_offset() {
        // PST is UTC-8 (-480 minutes)
        let dt = Utc.with_ymd_and_hms(2024, 1, 1, 10, 0, 0).unwrap();
        assert_eq!(get_local_hour(dt, -480), 2); // 10:00 UTC = 2:00 PST
    }

    #[test]
    fn test_get_local_hour_wraparound() {
        // Test midnight wraparound
        let dt = Utc.with_ymd_and_hms(2024, 1, 1, 2, 0, 0).unwrap();
        assert_eq!(get_local_hour(dt, -180), 23); // 2:00 UTC - 3 hours = 23:00 previous day
    }

    #[test]
    fn test_get_current_organ() {
        // Test morning organ (Stomach: 7-9 AM)
        let dt = Utc.with_ymd_and_hms(2024, 1, 1, 8, 0, 0).unwrap();
        let organ = get_current_organ(dt, 0);
        assert_eq!(organ.organ, crate::models::Organ::Stomach);
    }

    #[test]
    fn test_get_current_organ_with_offset() {
        // 6 AM UTC with IST offset (+330) = 11:30 AM local = Heart time
        let dt = Utc.with_ymd_and_hms(2024, 1, 1, 6, 0, 0).unwrap();
        let organ = get_current_organ(dt, 330);
        assert_eq!(organ.organ, crate::models::Organ::Heart);
    }

    #[test]
    fn test_minutes_until_next_transition() {
        // At 8:30, next transition is at 9:00 = 30 minutes
        let dt = Utc.with_ymd_and_hms(2024, 1, 1, 8, 30, 0).unwrap();
        let minutes = minutes_until_next_transition(dt, 0);
        // At 8:30, we're in the 7-9 window, next is at 9
        assert!(minutes <= 90); // Should be less than 90 minutes
    }

    #[test]
    fn test_window_progress() {
        // At the start of a window
        let dt = Utc.with_ymd_and_hms(2024, 1, 1, 7, 0, 0).unwrap();
        let progress = get_window_progress(dt, 0);
        assert!(progress < 0.1, "Progress at window start should be near 0");

        // At the end of a window
        let dt = Utc.with_ymd_and_hms(2024, 1, 1, 8, 50, 0).unwrap();
        let progress = get_window_progress(dt, 0);
        assert!(progress > 0.8, "Progress near window end should be > 0.8");
    }
}
