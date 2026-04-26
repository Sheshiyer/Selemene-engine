//! Current Dasha calculation
//!
//! FAPI-039: Implement current Dasha calculation

use chrono::NaiveDate;

use crate::error::VedicApiResult;
use super::types::{DashaLord, DashaPeriod, VimshottariTimeline, CurrentDashas};

/// Get the current dasha periods at a specific date
pub fn get_current_dashas(timeline: &VimshottariTimeline, date: NaiveDate) -> CurrentDashas {
    timeline.current_dashas_at(date)
}

/// Find which Mahadasha is active at a given date
pub fn find_active_mahadasha(timeline: &VimshottariTimeline, date: NaiveDate) -> Option<&DashaPeriod> {
    timeline.mahadashas.iter().find(|md| md.contains_date(date))
}

/// Find which Antardasha is active at a given date
pub fn find_active_antardasha(timeline: &VimshottariTimeline, date: NaiveDate) -> Option<&DashaPeriod> {
    find_active_mahadasha(timeline, date)
        .and_then(|md| md.sub_periods.iter().find(|ad| ad.contains_date(date)))
}

/// Find which Pratyantardasha is active at a given date
pub fn find_active_pratyantardasha(timeline: &VimshottariTimeline, date: NaiveDate) -> Option<&DashaPeriod> {
    find_active_antardasha(timeline, date)
        .and_then(|ad| ad.sub_periods.iter().find(|pd| pd.contains_date(date)))
}

/// Get the current dasha string in format "Mahadasha-Antardasha-Pratyantardasha"
pub fn current_dasha_string(timeline: &VimshottariTimeline, date: NaiveDate) -> String {
    let dashas = get_current_dashas(timeline, date);
    dashas.display_string()
}

/// Calculate remaining days in current Mahadasha
pub fn mahadasha_days_remaining(timeline: &VimshottariTimeline, date: NaiveDate) -> Option<i64> {
    find_active_mahadasha(timeline, date).map(|md| md.days_remaining(date))
}

/// Calculate remaining days in current Antardasha
pub fn antardasha_days_remaining(timeline: &VimshottariTimeline, date: NaiveDate) -> Option<i64> {
    find_active_antardasha(timeline, date).map(|ad| ad.days_remaining(date))
}

/// Get percentage progress through current Mahadasha
pub fn mahadasha_progress(timeline: &VimshottariTimeline, date: NaiveDate) -> Option<f64> {
    find_active_mahadasha(timeline, date).map(|md| md.percent_elapsed(date))
}

/// Get percentage progress through current Antardasha
pub fn antardasha_progress(timeline: &VimshottariTimeline, date: NaiveDate) -> Option<f64> {
    find_active_antardasha(timeline, date).map(|ad| ad.percent_elapsed(date))
}

/// Current dasha summary with all relevant information
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CurrentDashaSummary {
    /// Current date
    pub as_of: NaiveDate,
    /// Current Mahadasha lord
    pub mahadasha_lord: Option<DashaLord>,
    /// Current Antardasha lord
    pub antardasha_lord: Option<DashaLord>,
    /// Current Pratyantardasha lord
    pub pratyantardasha_lord: Option<DashaLord>,
    /// Display string (e.g., "Sun-Moon-Mars")
    pub display_string: String,
    /// Mahadasha progress percentage
    pub mahadasha_progress: Option<f64>,
    /// Antardasha progress percentage
    pub antardasha_progress: Option<f64>,
    /// Days remaining in current Mahadasha
    pub mahadasha_days_remaining: Option<i64>,
    /// Days remaining in current Antardasha
    pub antardasha_days_remaining: Option<i64>,
}

/// Build a complete current dasha summary
pub fn build_current_summary(timeline: &VimshottariTimeline, date: NaiveDate) -> CurrentDashaSummary {
    let dashas = get_current_dashas(timeline, date);
    
    CurrentDashaSummary {
        as_of: date,
        mahadasha_lord: dashas.mahadasha.as_ref().map(|d| d.lord),
        antardasha_lord: dashas.antardasha.as_ref().map(|d| d.lord),
        pratyantardasha_lord: dashas.pratyantardasha.as_ref().map(|d| d.lord),
        display_string: dashas.display_string(),
        mahadasha_progress: mahadasha_progress(timeline, date),
        antardasha_progress: antardasha_progress(timeline, date),
        mahadasha_days_remaining: mahadasha_days_remaining(timeline, date),
        antardasha_days_remaining: antardasha_days_remaining(timeline, date),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::types::{DashaLevel, DashaBalance};
    use chrono::{NaiveDateTime, NaiveTime};

    fn sample_timeline() -> VimshottariTimeline {
        VimshottariTimeline {
            birth_datetime: NaiveDateTime::new(
                NaiveDate::from_ymd_opt(1990, 1, 1).unwrap(),
                NaiveTime::from_hms_opt(12, 0, 0).unwrap(),
            ),
            birth_nakshatra: "Ashwini".to_string(),
            birth_dasha_balance: DashaBalance {
                lord: DashaLord::Ketu,
                years: 5,
                months: 3,
                days: 10,
                total_days: 1930,
            },
            mahadashas: vec![
                DashaPeriod {
                    lord: DashaLord::Ketu,
                    level: DashaLevel::Mahadasha,
                    start_date: NaiveDate::from_ymd_opt(1990, 1, 1).unwrap(),
                    end_date: NaiveDate::from_ymd_opt(1995, 4, 11).unwrap(),
                    duration_days: 1926,
                    sub_periods: vec![
                        DashaPeriod {
                            lord: DashaLord::Ketu,
                            level: DashaLevel::Antardasha,
                            start_date: NaiveDate::from_ymd_opt(1990, 1, 1).unwrap(),
                            end_date: NaiveDate::from_ymd_opt(1990, 5, 28).unwrap(),
                            duration_days: 147,
                            sub_periods: vec![],
                        },
                    ],
                },
                DashaPeriod {
                    lord: DashaLord::Venus,
                    level: DashaLevel::Mahadasha,
                    start_date: NaiveDate::from_ymd_opt(1995, 4, 12).unwrap(),
                    end_date: NaiveDate::from_ymd_opt(2015, 4, 11).unwrap(),
                    duration_days: 7305,
                    sub_periods: vec![],
                },
            ],
        }
    }

    #[test]
    fn test_find_active_mahadasha() {
        let timeline = sample_timeline();
        
        // During Ketu mahadasha
        let date1 = NaiveDate::from_ymd_opt(1992, 6, 15).unwrap();
        let md = find_active_mahadasha(&timeline, date1).unwrap();
        assert_eq!(md.lord, DashaLord::Ketu);
        
        // During Venus mahadasha
        let date2 = NaiveDate::from_ymd_opt(2000, 1, 1).unwrap();
        let md = find_active_mahadasha(&timeline, date2).unwrap();
        assert_eq!(md.lord, DashaLord::Venus);
    }

    #[test]
    fn test_current_dasha_string() {
        let timeline = sample_timeline();
        let date = NaiveDate::from_ymd_opt(1990, 3, 15).unwrap();
        let result = current_dasha_string(&timeline, date);
        assert!(result.contains("Ketu"));
    }

    #[test]
    fn test_build_summary() {
        let timeline = sample_timeline();
        let date = NaiveDate::from_ymd_opt(1990, 3, 15).unwrap();
        let summary = build_current_summary(&timeline, date);
        
        assert_eq!(summary.mahadasha_lord, Some(DashaLord::Ketu));
        assert!(summary.mahadasha_progress.is_some());
    }
}
