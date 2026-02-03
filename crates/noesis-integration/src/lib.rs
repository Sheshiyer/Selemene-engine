//! Noesis Integration Layer
//!
//! This crate provides a unified interface for combining multiple consciousness
//! engines and the Vedic API into cohesive, multi-layered analyses.
//!
//! # Features
//! - Vedic API integration (Panchang, Muhurtas, Hora, Choghadiya)
//! - Vimshottari Dasha with planetary qualities
//! - Numerology (Pythagorean & Chaldean)
//! - TCM (Traditional Chinese Medicine) layering
//! - Bio-rhythm synchronization
//! - Unified verification with birth data
//!
//! # Example
//! ```no_run
//! use noesis_integration::{UnifiedAnalysis, BirthProfile};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let profile = BirthProfile::new(
//!         "1991-08-13",           // Birth date
//!         "13:31",                // Birth time
//!         12.9716,                // Latitude (Bengaluru)
//!         77.5946,                // Longitude (Bengaluru)
//!         "Asia/Kolkata",         // Timezone
//!     );
//!
//!     let analysis = UnifiedAnalysis::generate(&profile).await?;
//!     println!("Current Dasha: {}", analysis.vimshottari.current_mahadasha);
//!     println!("Life Path Number: {}", analysis.numerology.life_path_number);
//!
//!     Ok(())
//! }
//! ```

use chrono::{DateTime, Utc, Datelike, Timelike};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub mod analysis;
pub mod tcm_layer;
pub mod verification;
pub mod synthesis;

pub use analysis::{UnifiedAnalysis, LayeredInsight, UnifiedRecommendation, Priority as AnalysisPriority};
pub use tcm_layer::{TCMAnalysis, TCMElement, TCMOrgan};
pub use verification::{BirthProfile, DataVerifier, VerificationResult};
pub use synthesis::SynthesisEngine;

/// Re-export key types from Vedic API
pub use noesis_vedic_api::{
    CachedVedicClient,
    panchang::{
        CompletePanchang, Panchang, PanchangQuery,
        MuhurtaCollection, HoraTimings, ChoghadiyaTimings,
        Tithi, Nakshatra, Yoga, Karana, Vara, Paksha,
    },
    dasha::{VimshottariDasha, DashaPeriod, DashaPlanet, DashaLevel},
    chart::{BirthChart, NavamsaChart, ZodiacSign},
};

/// Re-export from Vimshottari engine
pub use engine_vimshottari::{
    VimshottariChart, Mahadasha, Antardasha, Pratyantardasha,
    VedicPlanet, Nakshatra as VimshottariNakshatra,
    CurrentPeriod, PlanetaryQualities,
};

/// Re-export from Numerology engine
pub use engine_numerology::{
    NumerologyEngine, NumerologyNumber, NumerologyResult,
};

/// Version of this crate
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Result type for integration operations
pub type Result<T> = std::result::Result<T, IntegrationError>;

/// Errors that can occur in the integration layer
#[derive(Debug, thiserror::Error)]
pub enum IntegrationError {
    #[error("Vedic API error: {0}")]
    VedicApi(#[from] noesis_vedic_api::VedicApiError),
    
    #[error("Engine error: {0}")]
    Engine(String),
    
    #[error("Verification failed: {0}")]
    Verification(String),
    
    #[error("Date parsing error: {0}")]
    DateParse(String),
    
    #[error("Configuration error: {0}")]
    Configuration(String),
    
    #[error("Synthesis error: {0}")]
    Synthesis(String),
}

/// Configuration for the integration layer
#[derive(Debug, Clone)]
pub struct IntegrationConfig {
    /// Whether to use the Vedic API or native engines
    pub use_vedic_api: bool,
    /// Whether to include TCM analysis
    pub include_tcm: bool,
    /// Whether to include numerology
    pub include_numerology: bool,
    /// Whether to include biorhythm
    pub include_biorhythm: bool,
    /// Precision level for calculations
    pub precision: noesis_core::Precision,
}

impl Default for IntegrationConfig {
    fn default() -> Self {
        Self {
            use_vedic_api: true,
            include_tcm: true,
            include_numerology: true,
            include_biorhythm: true,
            precision: noesis_core::Precision::High,
        }
    }
}

/// Complete birth profile for unified analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompleteBirthProfile {
    /// Basic birth data
    pub profile: BirthProfile,
    /// Name for numerology calculations
    pub name: Option<String>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Generate a unified analysis from a birth profile
pub async fn generate_unified_analysis(
    profile: &BirthProfile,
    config: &IntegrationConfig,
) -> Result<UnifiedAnalysis> {
    UnifiedAnalysis::generate_with_config(profile, config).await
}

/// Verify birth data against multiple calculation sources
pub async fn verify_birth_data(
    profile: &BirthProfile,
) -> Result<VerificationResult> {
    let verifier = DataVerifier::new();
    verifier.verify(profile).await
}

/// Get current Panchang with all Muhurtas for a location
pub async fn get_current_panchang(
    lat: f64,
    lng: f64,
    tz: f64,
) -> Result<CompletePanchang> {
    let client = CachedVedicClient::from_env()
        .map_err(|e| IntegrationError::Configuration(e.to_string()))?;
    
    let now = Utc::now();
    let panchang = client.get_complete_panchang(
        now.year(),
        now.month(),
        now.day(),
        now.hour() as u32,
        now.minute() as u32,
        now.second() as u32,
        lat,
        lng,
        tz,
    ).await?;
    
    Ok(panchang)
}

/// Calculate auspicious times for an activity
pub async fn find_auspicious_times(
    profile: &BirthProfile,
    activity: ActivityType,
    days: u32,
) -> Result<Vec<AuspiciousWindow>> {
    analysis::find_auspicious_windows(profile, activity, days).await
}

/// Types of activities for auspicious time finding
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActivityType {
    StartingNew,
    Business,
    Travel,
    Marriage,
    Medical,
    Spiritual,
    Education,
    Purchase,
    Construction,
}

/// An auspicious time window
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuspiciousWindow {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub quality: AuspiciousQuality,
    pub sources: Vec<String>,
    pub description: String,
}

/// Quality of an auspicious time
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuspiciousQuality {
    Excellent,
    Good,
    Moderate,
    Challenging,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }

    #[test]
    fn test_birth_profile_creation() {
        let profile = BirthProfile::new(
            "1991-08-13",
            "13:31",
            12.9716,
            77.5946,
            "Asia/Kolkata",
        );
        
        assert_eq!(profile.date, "1991-08-13");
        assert_eq!(profile.time, Some("13:31".to_string()));
    }
}
