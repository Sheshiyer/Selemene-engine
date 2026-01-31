//! Shared types used across all Noesis engines and services

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Input to any consciousness engine calculation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineInput {
    /// Birth data (required for birth-chart-based engines)
    pub birth_data: Option<BirthData>,
    /// Current timestamp for time-based calculations
    pub current_time: DateTime<Utc>,
    /// Geographic location
    pub location: Option<Coordinates>,
    /// Calculation precision level
    pub precision: Precision,
    /// Engine-specific options
    #[serde(default)]
    pub options: HashMap<String, Value>,
}

/// Output from any consciousness engine
#[derive(Debug, Clone, Serialize, Deserialize)]
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
pub struct BirthData {
    pub name: Option<String>,
    pub date: String,       // YYYY-MM-DD
    pub time: Option<String>, // HH:MM
    pub latitude: f64,
    pub longitude: f64,
    pub timezone: String,
}

/// Geographic coordinates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Coordinates {
    pub latitude: f64,
    pub longitude: f64,
    pub altitude: Option<f64>,
}

/// Calculation precision levels
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
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
pub struct CalculationMetadata {
    pub calculation_time_ms: f64,
    pub backend: String,
    pub precision_achieved: String,
    pub cached: bool,
    pub timestamp: DateTime<Utc>,
}

/// Multi-engine workflow definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowDefinition {
    pub id: String,
    pub name: String,
    pub description: String,
    pub engine_ids: Vec<String>,
}

/// Result from executing a multi-engine workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowResult {
    pub workflow_id: String,
    pub engine_outputs: HashMap<String, EngineOutput>,
    pub synthesis: Option<Value>,
    pub total_time_ms: f64,
    pub timestamp: DateTime<Utc>,
}
