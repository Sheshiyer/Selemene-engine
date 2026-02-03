use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WesternRequest {
    pub year: i32,
    pub month: u32,
    pub date: u32,
    pub hours: u32,
    pub minutes: u32,
    pub seconds: u32,
    pub latitude: f64,
    pub longitude: f64,
    pub timezone: f64,
    pub config: Option<WesternConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WesternConfig {
    pub house_system: Option<String>, // e.g., "Placidus", "Koch"
    pub zodiac_type: Option<String>,  // "Tropical" or "Sidereal"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanetData {
    pub name: String,
    pub longitude: f64,
    pub latitude: f64,
    pub speed: f64,
    pub house: f64,
    pub sign: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HouseData {
    pub house: u8,
    pub longitude: f64,
    pub sign: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WesternPlanetsResponse {
    pub output: Vec<PlanetData>, // Placeholder structure
    // We'll refine this once we see actual API output or use serde_json::Value
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WesternHousesResponse {
    pub output: Vec<HouseData>,
}
