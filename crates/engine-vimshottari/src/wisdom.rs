//! Wisdom data structures for JSON deserialization

use crate::models::VedicPlanet;
use serde::{Deserialize, Serialize};

/// Root structure for vimshottari_periods.json
#[derive(Debug, Clone, Deserialize)]
pub struct VimshottariPeriodsData {
    pub periods: std::collections::HashMap<String, PeriodInfo>,
    pub planetary_qualities: std::collections::HashMap<String, QualityDetails>,
}

/// Period information from JSON
#[derive(Debug, Clone, Deserialize)]
pub struct PeriodInfo {
    pub years: u8,
    pub element: String,
    pub qualities: Vec<String>,
    pub themes: Vec<String>,
}

/// Detailed qualities from JSON
#[derive(Debug, Clone, Deserialize)]
pub struct QualityDetails {
    pub consciousness_lessons: Vec<String>,
    pub optimal_practices: Vec<String>,
    pub challenges: Vec<String>,
}

/// Root structure for nakshatras.json
#[derive(Debug, Clone, Deserialize)]
pub struct NakshatrasData {
    pub nakshatras_info: NakshatraInfo,
    pub nakshatras: std::collections::HashMap<String, NakshatraEntry>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NakshatraInfo {
    pub name: String,
    pub description: String,
    pub total_nakshatras: u8,
    pub source: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NakshatraEntry {
    pub number: u8,
    pub name: String,
    pub start_degree: f64,
    pub end_degree: f64,
    pub ruling_planet: String,
    pub deity: String,
    pub symbol: String,
    pub nature: String,
    pub gana: String,
    pub qualities: Vec<String>,
    pub description: String,
}

/// Root structure for dasha_periods.json
#[derive(Debug, Clone, Deserialize)]
pub struct DashaPeriodsData {
    pub dasha_info: DashaInfo,
    pub mahadasha_periods: std::collections::HashMap<String, PeriodDuration>,
    pub planetary_order: Vec<String>,
    pub nakshatra_rulers: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DashaInfo {
    pub name: String,
    pub description: String,
    pub total_years: u16,
    pub source: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PeriodDuration {
    pub years: u8,
    pub months: u8,
    pub days: u8,
}
