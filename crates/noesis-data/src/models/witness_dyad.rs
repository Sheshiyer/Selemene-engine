use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct WitnessDyadExecutionRecord {
    pub id: Uuid,
    pub user_id: Uuid,
    pub tier: String,
    pub consciousness_level: i32,
    pub live_scores: Value,
    pub relationship_mode: String,
    pub engines_available: Value,
    pub aletheios: Option<String>,
    pub pichet: Option<String>,
    pub synthesis: Option<String>,
    pub witness_question: Option<String>,
    pub engines_used: Value,
    pub llm_powered: bool,
    pub llm_provider: Option<String>,
    pub llm_model_aletheios: Option<String>,
    pub llm_model_pichet: Option<String>,
    pub llm_model_synthesis: Option<String>,
    pub llm_duration_ms: Option<f64>,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct NewWitnessDyadExecution {
    pub user_id: Uuid,
    pub tier: String,
    pub consciousness_level: i16,
    pub live_scores: Value,
    pub relationship_mode: String,
    pub engines_available: Vec<String>,
    pub aletheios: Option<String>,
    pub pichet: Option<String>,
    pub synthesis: Option<String>,
    pub witness_question: Option<String>,
    pub engines_used: Vec<String>,
    pub llm_powered: bool,
    pub llm_provider: Option<String>,
    pub llm_model_aletheios: Option<String>,
    pub llm_model_pichet: Option<String>,
    pub llm_model_synthesis: Option<String>,
    pub llm_duration_ms: Option<f64>,
    pub error_message: Option<String>,
    pub request_ip_hash: Option<String>,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct WitnessDyadExecutionAdminRecord {
    pub id: Uuid,
    pub user_id: Uuid,
    pub user_email: String,
    pub tier: String,
    pub consciousness_level: i32,
    pub live_scores: Value,
    pub relationship_mode: String,
    pub engines_available: Value,
    pub aletheios: Option<String>,
    pub pichet: Option<String>,
    pub synthesis: Option<String>,
    pub witness_question: Option<String>,
    pub engines_used: Value,
    pub llm_powered: bool,
    pub llm_provider: Option<String>,
    pub llm_model_aletheios: Option<String>,
    pub llm_model_pichet: Option<String>,
    pub llm_model_synthesis: Option<String>,
    pub llm_duration_ms: Option<f64>,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct WitnessDyadModeBreakdown {
    pub llm_powered: bool,
    pub count: i64,
}