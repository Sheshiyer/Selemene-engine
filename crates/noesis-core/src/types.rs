//! Shared types used across all Noesis engines and services

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[cfg(feature = "openapi")]
use utoipa::ToSchema;

/// Input to any consciousness engine calculation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct EngineInput {
    /// Birth data (required for birth-chart-based engines)
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub birth_data: Option<BirthData>,
    /// Current timestamp for time-based calculations
    #[serde(default = "default_current_time")]
    pub current_time: DateTime<Utc>,
    /// Geographic location
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub location: Option<Coordinates>,
    /// Calculation precision level
    #[serde(default)]
    pub precision: Precision,
    /// Engine-specific options
    #[serde(default)]
    pub options: HashMap<String, Value>,
}

fn default_current_time() -> DateTime<Utc> {
    Utc::now()
}

/// Output from any consciousness engine
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct EngineOutput {
    /// Which engine produced this output
    pub engine_id: String,
    /// Engine-specific result data (each engine defines its own schema)
    pub result: Value,
    /// Self-inquiry question generated from the calculation
    pub witness_prompt: String,
    /// User's current consciousness development level (0-5)
    pub consciousness_level: u8,
    /// Calculation metadata (timing, backend, precision)
    pub metadata: CalculationMetadata,
}

/// Birth data for chart-based calculations
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct BirthData {
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub name: Option<String>,
    /// Date in YYYY-MM-DD format
    #[cfg_attr(feature = "openapi", schema(example = "1990-01-01"))]
    pub date: String,
    /// Time in HH:MM format
    #[cfg_attr(feature = "openapi", schema(example = "14:30", nullable = true))]
    pub time: Option<String>,
    /// Latitude in decimal degrees
    #[cfg_attr(feature = "openapi", schema(example = 12.9716))]
    pub latitude: f64,
    /// Longitude in decimal degrees
    #[cfg_attr(feature = "openapi", schema(example = 77.5946))]
    pub longitude: f64,
    /// IANA timezone identifier
    #[cfg_attr(feature = "openapi", schema(example = "Asia/Kolkata"))]
    pub timezone: String,
}

/// Geographic coordinates
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct Coordinates {
    /// Latitude in decimal degrees
    pub latitude: f64,
    /// Longitude in decimal degrees
    pub longitude: f64,
    /// Altitude in meters above sea level
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub altitude: Option<f64>,
}

/// Calculation precision levels
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub enum Precision {
    Standard = 1,
    High = 2,
    Extreme = 3,
}

impl Default for Precision {
    fn default() -> Self {
        Precision::Standard
    }
}

/// Metadata about how a calculation was performed
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct CalculationMetadata {
    /// Time taken for the calculation in milliseconds
    pub calculation_time_ms: f64,
    /// Backend used for calculation (e.g., "native", "swiss_ephemeris")
    pub backend: String,
    /// Precision level achieved
    pub precision_achieved: String,
    /// Whether the result was retrieved from cache
    pub cached: bool,
    /// Timestamp of calculation
    pub timestamp: DateTime<Utc>,
}

/// Multi-engine workflow definition
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct WorkflowDefinition {
    pub id: String,
    pub name: String,
    pub description: String,
    pub engine_ids: Vec<String>,
}

/// Result from executing a multi-engine workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct WorkflowResult {
    pub workflow_id: String,
    pub engine_outputs: HashMap<String, EngineOutput>,
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub synthesis: Option<Value>,
    pub total_time_ms: f64,
    pub timestamp: DateTime<Utc>,
}
