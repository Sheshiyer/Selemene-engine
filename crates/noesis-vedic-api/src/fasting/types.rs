//! Fasting types

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

/// Request for fasting days
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FastingRequest {
    pub from_date: NaiveDate,
    pub to_date: NaiveDate,
    pub include_ekadashi: bool,
    pub include_pradosh: bool,
    pub include_special: bool,
}

/// Fasting day summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FastingDaySummary {
    pub date: NaiveDate,
    pub name: String,
    pub deity: String,
    pub is_ekadashi: bool,
}

/// Fasting reminder
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FastingReminder {
    pub vrat_name: String,
    pub date: NaiveDate,
    pub days_until: i64,
    pub preparation_tips: Vec<String>,
    pub parana_info: Option<String>,
}

/// Personal fasting preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FastingPreferences {
    pub observe_ekadashi: bool,
    pub observe_pradosh: bool,
    pub observe_monday: bool,  // Shiva vrat
    pub observe_saturday: bool,  // Saturn vrat
    pub observe_tuesday: bool,  // Hanuman vrat
    pub custom_vrats: Vec<String>,
}

impl Default for FastingPreferences {
    fn default() -> Self {
        Self {
            observe_ekadashi: true,
            observe_pradosh: false,
            observe_monday: false,
            observe_saturday: false,
            observe_tuesday: false,
            custom_vrats: vec![],
        }
    }
}
