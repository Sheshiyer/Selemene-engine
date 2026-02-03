//! Daily Panchang Module
//!
//! FAPI-112: Daily panchang and calendar service

pub mod types;
pub mod calculator;
pub mod formatter;

pub use types::*;
pub use calculator::*;
pub use formatter::*;

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

/// Complete daily panchang data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyPanchang {
    /// Date
    pub date: NaiveDate,
    /// Day of week
    pub vara: String,
    /// Tithi details
    pub tithi: TithiInfo,
    /// Nakshatra details
    pub nakshatra: NakshatraInfo,
    /// Yoga details
    pub yoga: YogaInfo,
    /// Karana details
    pub karana: KaranaInfo,
    /// Sunrise time
    pub sunrise: String,
    /// Sunset time
    pub sunset: String,
    /// Moonrise time
    pub moonrise: Option<String>,
    /// Moonset time
    pub moonset: Option<String>,
    /// Rahu Kalam timings
    pub rahu_kalam: TimePeriod,
    /// Yama Gandam timings
    pub yama_gandam: TimePeriod,
    /// Gulika Kaal timings
    pub gulika_kaal: TimePeriod,
    /// Auspicious timings
    pub auspicious_periods: Vec<TimePeriod>,
    /// Festivals for this day
    pub festivals: Vec<String>,
    /// Hindu calendar month
    pub hindu_month: String,
    /// Hindu calendar year
    pub hindu_year: String,
    /// Special notes
    pub notes: Vec<String>,
}

/// Time period with label
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimePeriod {
    pub name: String,
    pub start: String,
    pub end: String,
    pub is_auspicious: bool,
}

/// Tithi information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TithiInfo {
    pub name: String,
    pub number: u8,
    pub paksha: String,  // Shukla or Krishna
    pub end_time: String,
    pub deity: String,
}

/// Nakshatra information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NakshatraInfo {
    pub name: String,
    pub number: u8,
    pub end_time: String,
    pub deity: String,
    pub ruler: String,
}

/// Yoga information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YogaInfo {
    pub name: String,
    pub number: u8,
    pub end_time: String,
    pub meaning: String,
}

/// Karana information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KaranaInfo {
    pub name: String,
    pub number: u8,
    pub end_time: String,
}
