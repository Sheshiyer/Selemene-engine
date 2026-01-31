//! Noesis Core — Shared traits and types for the Tryambakam consciousness engine platform
//!
//! All 13 consciousness engines implement the `ConsciousnessEngine` trait defined here.
//! This crate provides the universal interface, shared types, and error definitions.

pub mod types;
pub mod error;

pub use types::*;
pub use error::*;

use async_trait::async_trait;

#[cfg(feature = "openapi")]
use utoipa::ToSchema;

/// The universal trait that all consciousness engines must implement.
///
/// Rust engines implement this directly. TypeScript engines are wrapped
/// via `BridgeEngine` (in `noesis-bridge`) which adapts HTTP calls to this trait.
#[async_trait]
pub trait ConsciousnessEngine: Send + Sync {
    /// Unique identifier for this engine (e.g., "panchanga", "numerology", "human-design")
    fn engine_id(&self) -> &str;

    /// Human-readable name (e.g., "Panchanga", "Numerology", "Human Design")
    fn engine_name(&self) -> &str;

    /// Minimum consciousness phase level required to access this engine (0-5)
    fn required_phase(&self) -> u8;

    /// Execute the engine's core calculation
    async fn calculate(&self, input: EngineInput) -> Result<EngineOutput, EngineError>;

    /// Validate an engine output for correctness
    async fn validate(&self, output: &EngineOutput) -> Result<ValidationResult, EngineError>;

    /// Generate a deterministic cache key for the given input.
    /// Uses SHA-256 to ensure consistency across restarts.
    fn cache_key(&self, input: &EngineInput) -> String;
}

/// Result of validating an engine output
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct ValidationResult {
    /// Whether the output is valid
    pub valid: bool,
    /// Confidence level (0.0-1.0)
    pub confidence: f64,
    /// Validation messages or warnings
    pub messages: Vec<String>,
}
