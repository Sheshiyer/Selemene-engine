//! Progression types for predictive astrology

use serde::{Deserialize, Serialize};

use crate::chart::ZodiacSign;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressionRequest {
    pub birth_date: String,
    pub birth_time: String,
    pub target_date: String,
    pub latitude: f64,
    pub longitude: f64,
    pub timezone: f64,
    pub method: ProgressionMethod,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProgressionMethod {
    Secondary,
    SolarArc,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressionChart {
    pub method: ProgressionMethod,
    pub target_date: String,
    pub planets: Vec<ProgressedPlanet>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressedPlanet {
    pub name: String,
    pub longitude: f64,
    pub sign: ZodiacSign,
    pub degree: f64,
    pub is_retrograde: bool,
}
