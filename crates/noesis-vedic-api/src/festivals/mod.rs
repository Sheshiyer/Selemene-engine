//! Festivals Module
//!
//! FAPI-115: Hindu calendar festivals and events

pub mod types;
pub mod calendar;
pub mod regional;

pub use types::*;
pub use calendar::*;
pub use regional::*;

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

/// Festival category
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FestivalCategory {
    Major,
    Regional,
    Religious,
    Seasonal,
    Auspicious,
    Fasting,
    Eclipse,
}

/// Hindu festival
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Festival {
    /// Festival name
    pub name: String,
    /// Date of festival
    pub date: NaiveDate,
    /// Category
    pub category: FestivalCategory,
    /// Associated deity
    pub deity: Option<String>,
    /// Regions where celebrated
    pub regions: Vec<String>,
    /// Description
    pub description: String,
    /// Rituals and practices
    pub rituals: Vec<String>,
    /// Fasting associated
    pub fasting: Option<FastingInfo>,
    /// Panchang criteria
    pub panchang_criteria: PanchangCriteria,
}

/// Fasting information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FastingInfo {
    pub fasting_type: String,
    pub duration: String,
    pub breaking_time: String,
    pub exemptions: Vec<String>,
}

/// Panchang criteria for festival
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PanchangCriteria {
    pub tithi: Option<String>,
    pub nakshatra: Option<String>,
    pub month: Option<String>,
    pub paksha: Option<String>,
}

/// Festival list for a period
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FestivalList {
    pub from_date: NaiveDate,
    pub to_date: NaiveDate,
    pub festivals: Vec<Festival>,
    pub total_count: usize,
}
