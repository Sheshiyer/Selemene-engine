//! Festival types

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

/// Request for festivals
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FestivalRequest {
    pub from_date: NaiveDate,
    pub to_date: NaiveDate,
    pub region: Option<String>,
    pub categories: Option<Vec<String>>,
}

/// Festival summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FestivalSummary {
    pub name: String,
    pub date: NaiveDate,
    pub category: String,
    pub is_holiday: bool,
}

/// Monthly festival calendar
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonthlyFestivalCalendar {
    pub year: i32,
    pub month: u32,
    pub festivals: Vec<FestivalSummary>,
    pub fasting_days: Vec<NaiveDate>,
    pub auspicious_days: Vec<NaiveDate>,
}

/// Festival reminder
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FestivalReminder {
    pub festival_name: String,
    pub date: NaiveDate,
    pub days_until: i64,
    pub preparation_tips: Vec<String>,
}
