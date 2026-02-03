//! Daily panchang types

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

/// Request for daily panchang
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyPanchangRequest {
    pub date: NaiveDate,
    pub latitude: f64,
    pub longitude: f64,
    pub timezone: f64,
}

/// Monthly panchang summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonthlyPanchang {
    pub year: i32,
    pub month: u32,
    pub days: Vec<DailyPanchangSummary>,
    pub festivals: Vec<FestivalEntry>,
    pub eclipses: Vec<EclipseEntry>,
}

/// Summary for a single day
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyPanchangSummary {
    pub date: NaiveDate,
    pub vara: String,
    pub tithi: String,
    pub nakshatra: String,
    pub is_auspicious: bool,
    pub festivals: Vec<String>,
}

/// Festival entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FestivalEntry {
    pub date: NaiveDate,
    pub name: String,
    pub category: String,
}

/// Eclipse entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EclipseEntry {
    pub date: NaiveDate,
    pub eclipse_type: String,
    pub visibility: String,
}
