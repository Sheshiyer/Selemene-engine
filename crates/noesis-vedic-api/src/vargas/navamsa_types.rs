//! Navamsa (D9) DTOs

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NavamsaChartDto {
    pub positions: Vec<NavamsaPositionDto>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lagna: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vargottama: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NavamsaPositionDto {
    pub planet: String,
    pub sign: String,
    pub degree: f64,
}
