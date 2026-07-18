//! TypeScript engines HTTP client
//!
//! Provides request/response adapters for the Bun-based TS engines service.
//! Handles the translation between Rust `EngineInput` and the TS engine's
//! expected JSON format.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Request format expected by TypeScript engines
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TsEngineRequest {
    /// Consciousness level (0-5) - maps to required_phase
    pub consciousness_level: u8,
    /// Engine-specific parameters
    pub parameters: HashMap<String, Value>,
    /// Optional random seed for deterministic results
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<u64>,
    /// Optional question/intention
    #[serde(skip_serializing_if = "Option::is_none")]
    pub question: Option<String>,
    /// FROZEN top-level image reference.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_data: Option<Value>,
    /// FROZEN top-level audio reference.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio_ref: Option<Value>,
    /// FROZEN top-level consent grant.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub consent: Option<Value>,
    /// FROZEN top-level quality specification.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quality: Option<Value>,
}

/// Response format from TypeScript engines
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TsEngineResponse {
    /// Engine that produced this result
    pub engine_id: String,
    /// The calculation result (engine-specific structure)
    pub result: Value,
    /// Witness prompts for self-inquiry
    pub witness_prompts: Vec<WitnessPrompt>,
    /// ISO timestamp when calculation completed
    pub calculated_at: String,
    /// Processing time in milliseconds (float-safe: TS engines may return fractional ms)
    pub processing_time_ms: f64,
    /// FROZEN top-level generated image.
    #[serde(default)]
    pub generated_image: Option<Value>,
    /// FROZEN top-level generated audio.
    #[serde(default)]
    pub generated_audio: Option<Value>,
}

/// A prompt for self-reflection/witnessing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WitnessPrompt {
    /// The prompt text
    pub prompt: String,
    /// Context about why this prompt is relevant
    pub context: String,
    /// Thematic tags
    pub themes: Vec<String>,
}

/// Health check response from TS engines service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TsHealthResponse {
    pub status: String,
    pub engines: Vec<String>,
    pub uptime_ms: u64,
    pub version: String,
}

/// Engine metadata from TS engines service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TsEngineMetadata {
    pub id: String,
    pub name: String,
    pub description: String,
    pub version: String,
    pub required_phase: u8,
    pub input_schema: Value,
}

/// List engines response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TsEnginesListResponse {
    pub engines: Vec<TsEngineMetadata>,
    pub count: usize,
}

impl TsEngineRequest {
    /// Create a new request with consciousness level and parameters
    pub fn new(consciousness_level: u8, parameters: HashMap<String, Value>) -> Self {
        Self {
            consciousness_level,
            parameters,
            seed: None,
            question: None,
            image_data: None,
            audio_ref: None,
            consent: None,
            quality: None,
        }
    }

    /// Add a question/intention to the request
    pub fn with_question(mut self, question: impl Into<String>) -> Self {
        self.question = Some(question.into());
        self
    }

    /// Add a random seed for deterministic results
    pub fn with_seed(mut self, seed: u64) -> Self {
        self.seed = Some(seed);
        self
    }
}
