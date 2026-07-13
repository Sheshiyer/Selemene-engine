use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// A stored reading from an engine or workflow calculation.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Reading {
    pub id: Uuid,
    pub user_id: Uuid,
    pub engine_id: String,
    pub workflow_id: Option<String>,
    pub input_hash: String,
    pub input_data: serde_json::Value,
    pub result_data: serde_json::Value,
    pub witness_prompt: Option<String>,
    pub consciousness_level: i16,
    pub calculation_time_ms: Option<f64>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ReadingSyncRecord {
    pub id: Uuid,
    pub sync_cursor: i64,
    pub user_id: Uuid,
    pub engine_id: String,
    pub workflow_id: Option<String>,
    pub client_event_id: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Builder for inserting a new reading.
pub struct NewReading {
    pub user_id: Uuid,
    pub engine_id: String,
    pub workflow_id: Option<String>,
    pub input_hash: String,
    pub input_data: serde_json::Value,
    pub result_data: serde_json::Value,
    pub witness_prompt: Option<String>,
    pub consciousness_level: i16,
    pub calculation_time_ms: Option<f64>,
    pub client_event_id: Option<String>,
    pub client_device_id: Option<String>,
    pub device_platform: Option<String>,
    pub device_app_version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AdminReadingRecord {
    pub id: Uuid,
    pub user_id: Uuid,
    pub user_email: String,
    pub engine_id: String,
    pub workflow_id: Option<String>,
    pub input_hash: String,
    pub input_data: serde_json::Value,
    pub result_data: serde_json::Value,
    pub witness_prompt: Option<String>,
    pub consciousness_level: i16,
    pub calculation_time_ms: Option<f64>,
    pub created_at: DateTime<Utc>,
}
