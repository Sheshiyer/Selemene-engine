use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Main error type for the Selemene Engine
#[derive(Debug, thiserror::Error)]
pub enum EngineError {
    #[error("Calculation error: {0}")]
    CalculationError(String),
    #[error("Validation error: {0}")]
    ValidationError(String),
    #[error("Database error: {0}")]
    DatabaseError(String),
    #[error("Cache error: {0}")]
    CacheError(String),
    #[error("Configuration error: {0}")]
    ConfigError(String),
    #[error("Authentication error: {0}")]
    AuthError(String),
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
    #[error("Internal server error: {0}")]
    InternalError(String),
    #[error("Swiss Ephemeris error: {0}")]
    SwissEphemerisError(String),
}

/// Panchanga calculation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PanchangaRequest {
    pub date: String,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub timezone: Option<String>,
    pub precision: Option<PrecisionLevel>,
    pub include_details: Option<bool>,
}

/// Panchanga calculation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PanchangaResult {
    pub date: String,
    pub tithi: Option<f64>,
    pub nakshatra: Option<f64>,
    pub yoga: Option<f64>,
    pub karana: Option<f64>,
    pub vara: Option<f64>,
    pub solar_longitude: f64,
    pub lunar_longitude: f64,
    pub precision: u8,
    pub backend: String,
    pub calculation_time: Option<DateTime<Utc>>,
}

/// Precision levels for calculations
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum PrecisionLevel {
    Standard = 1,
    High = 2,
    Extreme = 3,
}

/// Astronomical coordinates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Coordinates {
    pub latitude: f64,
    pub longitude: f64,
    pub altitude: Option<f64>,
}

/// Time zone information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeZone {
    pub offset_hours: i32,
    pub offset_minutes: i32,
    pub name: String,
}

/// Julian Day number
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct JulianDay(pub f64);

impl JulianDay {
    pub fn from_date_time(_dt: DateTime<Utc>) -> Self {
        // TODO: Implement proper Julian Day calculation
        Self(2451545.0)
    }
    
    pub fn to_date_time(self) -> DateTime<Utc> {
        // TODO: Implement proper Julian Day to DateTime conversion
        Utc::now()
    }
}

/// Solar position information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolarPosition {
    pub longitude: f64,
    pub latitude: f64,
    pub distance: f64,
}

/// Lunar position information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LunarPosition {
    pub longitude: f64,
    pub latitude: f64,
    pub distance: f64,
}

/// Planetary position information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanetaryPosition {
    pub longitude: f64,
    pub latitude: f64,
    pub distance: f64,
}

/// House system enumeration
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum HouseSystem {
    Placidus,
    Koch,
    Equal,
    WholeSign,
}

/// Tithi (lunar day) information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tithi {
    pub number: u8,
    pub name: String,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub duration: Option<f64>,
}

/// Nakshatra (lunar mansion) information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Nakshatra {
    pub number: u8,
    pub name: String,
    pub start_longitude: f64,
    pub end_longitude: f64,
    pub ruler: String,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
}

/// Yoga (astrological combination) information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Yoga {
    pub number: u8,
    pub name: String,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
}

/// Karana (half-Tithi) information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Karana {
    pub number: u8,
    pub name: String,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
}

/// Vara (weekday) information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vara {
    pub number: u8,
    pub name: String,
    pub ruler: String,
}

/// Batch calculation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchRequest {
    pub requests: Vec<PanchangaRequest>,
    pub parallel: Option<bool>,
    pub max_concurrent: Option<usize>,
}

/// Batch calculation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchResult {
    pub results: Vec<PanchangaResult>,
    pub total_time: f64,
    pub success_count: usize,
    pub error_count: usize,
    pub errors: Vec<String>,
}

/// Health check response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    pub status: String,
    pub timestamp: DateTime<Utc>,
    pub version: String,
    pub components: ComponentHealth,
}

/// Component health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealth {
    pub database: ComponentStatus,
    pub cache: ComponentStatus,
    pub swiss_ephemeris: ComponentStatus,
    pub native_engines: ComponentStatus,
}

/// Individual component status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentStatus {
    pub status: String,
    pub message: Option<String>,
    pub last_check: DateTime<Utc>,
}

impl ComponentStatus {
    pub fn healthy() -> Self {
        Self {
            status: "healthy".to_string(),
            message: None,
            last_check: Utc::now(),
        }
    }
    
    pub fn unhealthy(message: String) -> Self {
        Self {
            status: "unhealthy".to_string(),
            message: Some(message),
            last_check: Utc::now(),
        }
    }
}

/// Metrics data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineMetrics {
    pub requests_total: u64,
    pub requests_successful: u64,
    pub requests_failed: u64,
    pub average_response_time: f64,
    pub cache_hit_rate: f64,
    pub backend_usage: BackendUsageMetrics,
}

/// Backend usage metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendUsageMetrics {
    pub native_engine: u64,
    pub swiss_ephemeris: u64,
    pub validated: u64,
}

/// API response wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
    pub timestamp: DateTime<Utc>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            timestamp: Utc::now(),
        }
    }
    
    pub fn error(error: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(error),
            timestamp: Utc::now(),
        }
    }
}

/// Ghati time request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GhatiTimeRequest {
    pub utc_time: Option<DateTime<Utc>>,
    pub location: Coordinates,
    pub calculation_method: Option<String>,
    pub precision: Option<String>,
}

/// Ghati time response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GhatiTimeResponse {
    pub ghati: u8,
    pub pala: u8,
    pub vipala: u8,
    pub utc_timestamp: DateTime<Utc>,
    pub local_time: DateTime<Utc>, // TODO: Convert to local timezone
    pub calculation_method: String,
    pub precision: String,
    pub next_ghati_transition: Option<GhatiTransitionInfo>,
}

/// Ghati transition information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GhatiTransitionInfo {
    pub from_ghati: u8,
    pub to_ghati: u8,
    pub transition_time: DateTime<Utc>,
    pub time_until_transition: String, // Human readable duration
}

/// Ghati boundaries request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GhatiBoundariesRequest {
    pub date: String, // YYYY-MM-DD format
    pub location: Coordinates,
    pub calculation_method: Option<String>,
}

/// Ghati boundaries response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GhatiBoundariesResponse {
    pub date: String,
    pub location: Coordinates,
    pub boundaries: Vec<GhatiBoundaryInfo>,
    pub calculation_method: String,
}

/// Ghati boundary information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GhatiBoundaryInfo {
    pub ghati_number: u8,
    pub utc_timestamp: DateTime<Utc>,
    pub local_time: DateTime<Utc>, // TODO: Convert to local timezone
    pub time_since_midnight: String, // Human readable duration
}
