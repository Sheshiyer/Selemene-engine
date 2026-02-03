//! Dasha alert types

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

/// Upcoming dasha transitions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpcomingDashaTransitions {
    pub current_mahadasha: String,
    pub current_antardasha: String,
    pub mahadasha_end: NaiveDate,
    pub antardasha_end: NaiveDate,
    pub upcoming: Vec<DashaTransitionSummary>,
}

/// Summary of a dasha transition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashaTransitionSummary {
    pub level: String,
    pub from_lord: String,
    pub to_lord: String,
    pub date: NaiveDate,
    pub days_away: i64,
}

/// Dasha alert
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashaAlert {
    pub id: String,
    pub transition_type: String,
    pub from_lord: String,
    pub to_lord: String,
    pub transition_date: NaiveDate,
    pub alert_date: NaiveDate,
    pub message: String,
    pub acknowledged: bool,
}

impl DashaAlert {
    pub fn new(
        transition_type: &str,
        from_lord: &str,
        to_lord: &str,
        transition_date: NaiveDate,
        days_before: u32,
    ) -> Self {
        let alert_date = transition_date - chrono::Duration::days(days_before as i64);
        Self {
            id: format!("{}_{}_{}_{}", transition_type, from_lord, to_lord, alert_date),
            transition_type: transition_type.to_string(),
            from_lord: from_lord.to_string(),
            to_lord: to_lord.to_string(),
            transition_date,
            alert_date,
            message: format!(
                "{} Dasha changing from {} to {} in {} days",
                transition_type, from_lord, to_lord, days_before
            ),
            acknowledged: false,
        }
    }
}
