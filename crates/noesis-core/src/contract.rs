//! Additive Rust adapters for the language-neutral `contracts/v1` authority.
//!
//! These types do not replace the runtime [`crate::EngineInput`] and
//! [`crate::EngineOutput`] DTOs. They give Rust consumers a typed boundary for
//! canonical fixtures while runtime migration remains a separate change.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

pub const CONTRACT_VERSION: &str = "v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContractVersion {
    #[serde(rename = "v1")]
    V1,
}

impl ContractVersion {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::V1 => CONTRACT_VERSION,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Consent {
    pub granted: bool,
    pub scopes: Vec<String>,
    pub timestamp: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Quality {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sufficient: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub score: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_coherence: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scores: Option<BTreeMap<String, f64>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diagnostics: Option<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ImageData {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub b64: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub consent: Option<Consent>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AudioReference {
    pub reference: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub consent: Option<Consent>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContractEngineRequest {
    pub contract_version: ContractVersion,
    pub consciousness_level: u8,
    pub parameters: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub question: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub birth_data: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_time: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub precision: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_data: Option<ImageData>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio_ref: Option<AudioReference>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub consent: Option<Consent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quality: Option<Quality>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WitnessPrompt {
    pub prompt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub themes: Option<Vec<String>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RuntimeKind {
    #[serde(rename = "native")]
    Native,
    #[serde(rename = "typescript")]
    TypeScript,
    #[serde(rename = "python")]
    Python,
    #[serde(rename = "database-conditional")]
    DatabaseConditional,
    #[serde(rename = "composed")]
    Composed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Provenance {
    pub runtime_kind: RuntimeKind,
    pub implementation_version: String,
    pub cached: bool,
    pub fallback_used: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub backend_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContractEngineResult {
    pub contract_version: ContractVersion,
    pub engine_id: String,
    pub result: Value,
    pub consciousness_level: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub witness_prompt: Option<String>,
    pub witness_prompts: Vec<WitnessPrompt>,
    pub calculated_at: DateTime<Utc>,
    pub processing_time_ms: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generated_image: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generated_audio: Option<Value>,
    pub provenance: Provenance,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContractError {
    pub contract_version: ContractVersion,
    pub status: u16,
    pub error_code: String,
    pub message: String,
    pub error: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<Value>,
    pub trace_id: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CapabilityAvailability {
    Declared,
    Available,
    Degraded,
    Unavailable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EngineCapability {
    pub contract_version: ContractVersion,
    pub engine_id: String,
    pub display_name: String,
    pub availability: CapabilityAvailability,
    pub runtime_kind: RuntimeKind,
    pub dependencies: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required_phase: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub implementation_version: Option<String>,
}
