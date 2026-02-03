//! DTOs for Panchang API request/response payloads

use serde::{Deserialize, Serialize};

/// Request payload for Panchang API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PanchangRequestDto {
    pub year: i32,
    pub month: u32,
    pub date: u32,
    pub hours: u32,
    pub minutes: u32,
    pub seconds: u32,
    pub latitude: f64,
    pub longitude: f64,
    pub timezone: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub config: Option<PanchangConfigDto>,
}

/// Config payload for Panchang API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PanchangConfigDto {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub observation_point: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ayanamsha: Option<String>,
}

/// Top-level Panchang API response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PanchangResponseDto {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: Option<PanchangDataDto>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub result: Option<PanchangDataDto>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output: Option<PanchangDataDto>,
}

/// Panchang data section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PanchangDataDto {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tithi: Option<TithiDto>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nakshatra: Option<NakshatraDto>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub yoga: Option<YogaDto>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub karana: Option<KaranaDto>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub day: Option<DayDto>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub paksha: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sunrise: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sunset: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub day_duration: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub night_duration: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_sunrise: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ayanamsha: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub planets: Option<PlanetsDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TithiDto {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub number: Option<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub end: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub paksha: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NakshatraDto {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub number: Option<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pada: Option<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub end: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub longitude: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YogaDto {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub number: Option<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub end: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KaranaDto {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub karana_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub end: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DayDto {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub number: Option<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanetsDto {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sun: Option<PlanetDto>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub moon: Option<PlanetDto>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mars: Option<PlanetDto>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mercury: Option<PlanetDto>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub jupiter: Option<PlanetDto>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub venus: Option<PlanetDto>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub saturn: Option<PlanetDto>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rahu: Option<PlanetDto>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ketu: Option<PlanetDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanetDto {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub longitude: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latitude: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub speed: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sign: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nakshatra: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pada: Option<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_retrograde: Option<bool>,
}
