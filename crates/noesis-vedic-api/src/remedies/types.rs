//! Remedy types

use serde::{Deserialize, Serialize};

/// Gemstone recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GemstoneRecommendation {
    pub gemstone: String,
    pub planet: String,
    pub weight_carats: String,
    pub metal: String,
    pub finger: String,
    pub day_to_wear: String,
    pub nakshatra: Option<String>,
    pub mantra_before_wearing: String,
    pub benefits: Vec<String>,
    pub alternatives: Vec<String>,
}

/// Mantra recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MantraRecommendation {
    pub mantra: String,
    pub planet: String,
    pub count: u32,
    pub timing: String,
    pub direction: Option<String>,
    pub duration: String,
    pub benefits: Vec<String>,
}

/// Donation recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DonationRecommendation {
    pub planet: String,
    pub items: Vec<String>,
    pub day: String,
    pub to_whom: String,
    pub benefits: String,
}

/// Yantra recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YantraRecommendation {
    pub name: String,
    pub planet: String,
    pub material: String,
    pub placement: String,
    pub worship_method: String,
}
