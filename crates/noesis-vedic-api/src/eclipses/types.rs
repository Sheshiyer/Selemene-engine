//! Eclipse types

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

/// Request for eclipse data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EclipseRequest {
    pub from_date: NaiveDate,
    pub to_date: NaiveDate,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub include_vedic: bool,
}

/// Eclipse list response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EclipseList {
    pub from_date: NaiveDate,
    pub to_date: NaiveDate,
    pub solar_eclipses: Vec<EclipseSummary>,
    pub lunar_eclipses: Vec<EclipseSummary>,
    pub total_count: usize,
}

/// Eclipse summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EclipseSummary {
    pub eclipse_type: String,
    pub date: NaiveDate,
    pub zodiac_sign: String,
    pub is_visible_at_location: Option<bool>,
}

/// Eclipse reminder
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EclipseReminder {
    pub eclipse_type: String,
    pub date: NaiveDate,
    pub days_until: i64,
    pub sutak_advisory: Option<String>,
    pub preparation_tips: Vec<String>,
}
