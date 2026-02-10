//! Report Generator Module
//!
//! FAPI-111: Comprehensive astrological report generation

use serde::{Deserialize, Serialize};

pub mod birth_report;
pub mod compatibility_report;
pub mod transit_report;
pub mod types;

pub use birth_report::*;
pub use compatibility_report::*;
pub use transit_report::*;
pub use types::*;

/// Report section type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReportSection {
    PersonalInfo,
    BirthChart,
    PlanetPositions,
    HouseCusps,
    DivisionalCharts,
    Yogas,
    DashaPeriods,
    Remedies,
    Summary,
}

/// Report format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ReportFormat {
    #[default]
    Text,
    Html,
    Json,
    Pdf,
}

/// Report configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportConfig {
    pub format: ReportFormat,
    pub sections: Vec<ReportSection>,
    pub include_remedies: bool,
    pub language: String,
    pub detail_level: DetailLevel,
}

impl Default for ReportConfig {
    fn default() -> Self {
        Self {
            format: ReportFormat::Text,
            sections: vec![
                ReportSection::PersonalInfo,
                ReportSection::BirthChart,
                ReportSection::PlanetPositions,
                ReportSection::Yogas,
                ReportSection::DashaPeriods,
                ReportSection::Summary,
            ],
            include_remedies: true,
            language: "en".to_string(),
            detail_level: DetailLevel::Standard,
        }
    }
}

/// Detail level for reports
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum DetailLevel {
    Brief,
    #[default]
    Standard,
    Detailed,
    Expert,
}
