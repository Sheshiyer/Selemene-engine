//! Additive Rust adapters for the language-neutral `contracts/v1` authority.
//!
//! These types do not replace the runtime [`crate::EngineInput`] and
//! [`crate::EngineOutput`] DTOs. They give Rust consumers a typed boundary for
//! canonical fixtures while runtime migration remains a separate change.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

#[cfg(feature = "openapi")]
use utoipa::ToSchema;

pub const CONTRACT_VERSION: &str = "v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
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
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[serde(deny_unknown_fields)]
pub struct Consent {
    pub granted: bool,
    pub scopes: Vec<String>,
    pub timestamp: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
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
#[cfg_attr(feature = "openapi", derive(ToSchema))]
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
#[cfg_attr(feature = "openapi", derive(ToSchema))]
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
#[cfg_attr(feature = "openapi", derive(ToSchema))]
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
#[cfg_attr(feature = "openapi", derive(ToSchema))]
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub witness_prompts: Option<Vec<WitnessPrompt>>,
    pub calculated_at: DateTime<Utc>,
    pub processing_time_ms: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generated_image: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generated_audio: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provenance: Option<Provenance>,
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
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[serde(rename_all = "lowercase")]
pub enum CapabilityAvailability {
    Declared,
    Available,
    Degraded,
    Unavailable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
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
    /// A bounded reason is optional for legacy item fixtures and required by
    /// the canonical capability-list projection.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason_code: Option<CapabilityReasonCode>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dependency_observations: Option<Vec<DependencyObservation>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub operations: Option<CapabilityOperations>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CapabilityReasonCode {
    NotObserved,
    Registered,
    CapabilityAvailable,
    CapabilityDegraded,
    CapabilityUnavailable,
    RequiredDependencyUnavailable,
    OptionalDependencyUnavailable,
    DependencyUnavailable,
    DatabaseUnconfigured,
    ModuleUnavailable,
    Timeout,
    MalformedObservation,
    OperationUnsupported,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[serde(rename_all = "lowercase")]
pub enum DependencyKind {
    Rust,
    TypeScript,
    Python,
    Database,
    Network,
    Filesystem,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[serde(rename_all = "lowercase")]
pub enum DependencyRequirement {
    Required,
    Optional,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[serde(deny_unknown_fields)]
pub struct DependencyObservation {
    pub dependency_id: String,
    pub dependency_kind: DependencyKind,
    pub requirement: DependencyRequirement,
    pub availability: CapabilityAvailability,
    pub reason_code: CapabilityReasonCode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[serde(rename_all = "lowercase")]
pub enum OperationSupport {
    Supported,
    Unsupported,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[serde(deny_unknown_fields)]
pub struct CapabilityOperations {
    pub calculate: OperationSupport,
    pub validate: OperationSupport,
    pub witness_eligible: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[serde(deny_unknown_fields)]
pub struct EngineCapabilityList {
    pub contract_version: ContractVersion,
    pub capabilities: Vec<EngineCapability>,
    pub count: u8,
    pub public_mirror_count: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("contract validation failed: {0}")]
pub struct ContractValidationError(pub String);

impl EngineCapabilityList {
    /// Validate the structural invariants shared by every language reader.
    /// Registry-specific class and public-mirror metadata is checked by the
    /// repository authority validator; this method protects the wire envelope.
    pub fn validate(&self) -> Result<(), ContractValidationError> {
        if self.contract_version != ContractVersion::V1 {
            return Err(ContractValidationError(
                "unsupported contract version".into(),
            ));
        }
        if self.count != 19 || self.capabilities.len() != 19 {
            return Err(ContractValidationError(
                "capability list must contain exactly 19 rows".into(),
            ));
        }
        if self.public_mirror_count != 17 {
            return Err(ContractValidationError(
                "capability list must contain exactly 17 public mirrors".into(),
            ));
        }
        let mut ids = BTreeSet::new();
        for capability in &self.capabilities {
            if !ids.insert(&capability.engine_id) {
                return Err(ContractValidationError(format!(
                    "duplicate capability engine_id: {}",
                    capability.engine_id
                )));
            }
            if capability.reason_code.is_none()
                || capability.dependency_observations.is_none()
                || capability.operations.is_none()
            {
                return Err(ContractValidationError(format!(
                    "capability {} is missing canonical metadata",
                    capability.engine_id
                )));
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WorkflowExecutionStatus {
    Complete,
    Partial,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WorkflowSynthesisStatus {
    Available,
    Failed,
    Unsupported,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkflowEngineOutput {
    pub result: BTreeMap<String, Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provenance: Option<BTreeMap<String, Value>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkflowEngineFailure {
    pub engine_id: String,
    pub error_code: WorkflowErrorCode,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkflowErrorCode {
    #[serde(rename = "ENGINE_NOT_FOUND")]
    EngineNotFound,
    #[serde(rename = "DEPENDENCY_UNAVAILABLE")]
    DependencyUnavailable,
    #[serde(rename = "ENGINE_TIMEOUT")]
    EngineTimeout,
    #[serde(rename = "UPSTREAM_INVALID_RESPONSE")]
    UpstreamInvalidResponse,
    #[serde(rename = "OPERATION_UNSUPPORTED")]
    OperationUnsupported,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkflowSynthesis {
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkflowOutcome {
    pub contract_version: ContractVersion,
    pub workflow_id: String,
    pub requested_engine_ids: Vec<String>,
    pub engine_outputs: BTreeMap<String, WorkflowEngineOutput>,
    pub engine_failures: Vec<WorkflowEngineFailure>,
    pub execution_status: WorkflowExecutionStatus,
    pub synthesis_status: WorkflowSynthesisStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub synthesis: Option<WorkflowSynthesis>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub engine_results: Option<BTreeMap<String, WorkflowEngineOutput>>,
}

impl WorkflowOutcome {
    /// Check requested/output/failure conservation and synthesis truth before
    /// an outcome is adapted to any legacy workflow response.
    pub fn validate(&self) -> Result<(), ContractValidationError> {
        if self.contract_version != ContractVersion::V1 {
            return Err(ContractValidationError(
                "unsupported contract version".into(),
            ));
        }
        let requested = self.requested_engine_ids.iter().collect::<BTreeSet<_>>();
        if requested.len() != self.requested_engine_ids.len() || requested.is_empty() {
            return Err(ContractValidationError(
                "requested engine IDs must be unique and non-empty".into(),
            ));
        }
        let output_ids = self.engine_outputs.keys().collect::<BTreeSet<_>>();
        let failure_ids = self
            .engine_failures
            .iter()
            .map(|failure| &failure.engine_id)
            .collect::<BTreeSet<_>>();
        if output_ids.intersection(&failure_ids).next().is_some() {
            return Err(ContractValidationError(
                "an engine cannot both succeed and fail".into(),
            ));
        }
        if output_ids
            .union(&failure_ids)
            .cloned()
            .collect::<BTreeSet<_>>()
            != requested
        {
            return Err(ContractValidationError(
                "workflow engine outputs and failures must conserve requested IDs".into(),
            ));
        }
        if let Some(alias) = &self.engine_results {
            if alias != &self.engine_outputs {
                return Err(ContractValidationError(
                    "engine_results must equal engine_outputs".into(),
                ));
            }
        }
        match self.execution_status {
            WorkflowExecutionStatus::Complete
                if !failure_ids.is_empty() || output_ids.len() != requested.len() =>
            {
                return Err(ContractValidationError(
                    "complete workflow must have only successful outputs".into(),
                ));
            }
            WorkflowExecutionStatus::Partial if failure_ids.is_empty() || output_ids.is_empty() => {
                return Err(ContractValidationError(
                    "partial workflow must contain successes and failures".into(),
                ));
            }
            WorkflowExecutionStatus::Failed if !output_ids.is_empty() || failure_ids.is_empty() => {
                return Err(ContractValidationError(
                    "failed workflow must contain only failures".into(),
                ));
            }
            _ => {}
        }
        match self.synthesis_status {
            WorkflowSynthesisStatus::Available if self.synthesis.is_none() => {
                return Err(ContractValidationError(
                    "available synthesis requires text".into(),
                ));
            }
            WorkflowSynthesisStatus::Failed | WorkflowSynthesisStatus::Unsupported
                if self.synthesis.is_some() =>
            {
                return Err(ContractValidationError(
                    "failed or unsupported synthesis cannot contain powered text".into(),
                ));
            }
            _ => {}
        }
        Ok(())
    }
}
