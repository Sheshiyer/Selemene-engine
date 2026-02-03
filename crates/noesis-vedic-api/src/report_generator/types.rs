//! Report generator types

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

/// Generated report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedReport {
    /// Report title
    pub title: String,
    /// Subject name
    pub subject_name: String,
    /// Birth details
    pub birth_datetime: NaiveDateTime,
    /// Report generation timestamp
    pub generated_at: NaiveDateTime,
    /// Report sections
    pub sections: Vec<ReportSectionContent>,
    /// Overall summary
    pub summary: String,
}

/// Content for a report section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportSectionContent {
    /// Section title
    pub title: String,
    /// Section content (text or html)
    pub content: String,
    /// Key points
    pub key_points: Vec<String>,
    /// Optional chart data
    pub chart_data: Option<String>,
}

/// Birth details for report generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BirthDetails {
    pub name: String,
    pub datetime: NaiveDateTime,
    pub latitude: f64,
    pub longitude: f64,
    pub timezone: f64,
    pub place_name: Option<String>,
}

/// Compatibility pair for reports
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompatibilityPair {
    pub person1: BirthDetails,
    pub person2: BirthDetails,
}
