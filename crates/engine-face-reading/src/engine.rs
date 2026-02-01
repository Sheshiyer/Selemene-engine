//! ConsciousnessEngine implementation for Face Reading
//!
//! This is a stub implementation that returns mock data.
//! Full implementation requires MediaPipe integration for facial landmark detection.

use async_trait::async_trait;
use chrono::Utc;
use noesis_core::{
    CalculationMetadata, ConsciousnessEngine, EngineError, EngineInput, EngineOutput,
    ValidationResult,
};
use serde_json::{json, Value};
use std::time::Instant;

use crate::mock::generate_mock_analysis;
use crate::models::FaceAnalysis;
use crate::witness::generate_single_witness_prompt;

/// Face Reading consciousness engine
///
/// Combines multiple face reading traditions:
/// - Chinese Face Reading (Mian Xiang)
/// - Ayurvedic Face Analysis
/// - Western Physiognomy
///
/// This is currently a stub that returns mock analysis.
/// Full implementation requires image processing via MediaPipe.
pub struct FaceReadingEngine {
    engine_id: String,
    engine_name: String,
}

impl FaceReadingEngine {
    /// Create a new Face Reading engine instance
    pub fn new() -> Self {
        Self {
            engine_id: "face-reading".to_string(),
            engine_name: "Face Reading".to_string(),
        }
    }

    /// Serialize face analysis to JSON with additional metadata
    fn serialize_analysis(analysis: &FaceAnalysis) -> Value {
        let constitution = &analysis.constitution;
        let balance = &analysis.elemental_balance;

        json!({
            "analysis": {
                "constitution": {
                    "primary_dosha": constitution.primary_dosha,
                    "secondary_dosha": constitution.secondary_dosha,
                    "tcm_element": constitution.tcm_element,
                    "body_type": constitution.body_type,
                    "descriptions": {
                        "dosha": constitution.primary_dosha.description(),
                        "element": constitution.tcm_element.description(),
                        "body_type": constitution.body_type.description(),
                    }
                },
                "personality_indicators": analysis.personality_indicators.iter().map(|p| {
                    json!({
                        "trait_name": p.trait_name,
                        "facial_indicator": p.facial_indicator,
                        "description": p.description,
                    })
                }).collect::<Vec<_>>(),
                "elemental_balance": {
                    "wood": balance.wood,
                    "fire": balance.fire,
                    "earth": balance.earth,
                    "metal": balance.metal,
                    "water": balance.water,
                    "dominant": balance.dominant(),
                },
                "health_indicators": analysis.health_indicators.iter().map(|h| {
                    json!({
                        "zone": h.zone,
                        "associated_organ": h.associated_organ,
                        "observation": h.observation,
                    })
                }).collect::<Vec<_>>(),
                "is_mock_data": analysis.is_mock_data,
            },
            "notice": "This is simulated analysis. Full face reading requires image upload and MediaPipe processing.",
            "traditions": ["Chinese Mian Xiang", "Ayurvedic Face Analysis", "Western Physiognomy"],
            "future_capabilities": [
                "Real-time facial landmark detection",
                "Photo-based analysis",
                "Video stream processing",
                "Expression tracking over time",
                "Comparative analysis (before/after)",
            ],
            "disclaimer": "This information is for self-reflection purposes only and should not be used for medical diagnosis or treatment decisions."
        })
    }
}

impl Default for FaceReadingEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ConsciousnessEngine for FaceReadingEngine {
    fn engine_id(&self) -> &str {
        &self.engine_id
    }

    fn engine_name(&self) -> &str {
        &self.engine_name
    }

    fn required_phase(&self) -> u8 {
        1 // Requires self-reflection capacity
    }

    async fn calculate(&self, input: EngineInput) -> Result<EngineOutput, EngineError> {
        let start = Instant::now();

        // Extract optional seed for reproducibility
        let seed = input
            .options
            .get("seed")
            .and_then(|v| v.as_u64());

        // For future implementation: check for image data
        let _has_image = input.options.contains_key("image_data")
            || input.options.contains_key("image_url");

        // Generate mock analysis (in future, this would process actual image)
        let analysis = generate_mock_analysis(seed);

        // Get consciousness level for prompt generation
        let consciousness_level = input
            .options
            .get("consciousness_level")
            .and_then(|v| v.as_u64())
            .map(|v| v as u8)
            .unwrap_or(2);

        // Generate witness prompt
        let witness_prompt = generate_single_witness_prompt(&analysis, consciousness_level);

        // Ensure witness prompt is non-empty
        if witness_prompt.is_empty() {
            return Err(EngineError::CalculationError(
                "Witness prompt generation failed: empty result".to_string(),
            ));
        }

        let elapsed = start.elapsed();

        Ok(EngineOutput {
            engine_id: self.engine_id.clone(),
            result: Self::serialize_analysis(&analysis),
            witness_prompt,
            consciousness_level,
            metadata: CalculationMetadata {
                calculation_time_ms: elapsed.as_secs_f64() * 1000.0,
                backend: "mock-stub".to_string(),
                precision_achieved: "simulated".to_string(),
                cached: false,
                timestamp: Utc::now(),
            },
        })
    }

    async fn validate(&self, output: &EngineOutput) -> Result<ValidationResult, EngineError> {
        let mut messages = vec![];
        let mut valid = true;

        // Check witness prompt is non-empty
        if output.witness_prompt.is_empty() {
            messages.push("Witness prompt is empty".to_string());
            valid = false;
        }

        // Check result has expected structure
        if output.result.get("analysis").is_none() {
            messages.push("Missing 'analysis' field in result".to_string());
            valid = false;
        }

        // Check analysis has constitution
        if let Some(analysis) = output.result.get("analysis") {
            if analysis.get("constitution").is_none() {
                messages.push("Missing 'constitution' in analysis".to_string());
                valid = false;
            }
            if analysis.get("elemental_balance").is_none() {
                messages.push("Missing 'elemental_balance' in analysis".to_string());
                valid = false;
            }
            if analysis.get("personality_indicators").is_none() {
                messages.push("Missing 'personality_indicators' in analysis".to_string());
                valid = false;
            }

            // Verify mock data flag is present and true (for stub)
            if let Some(is_mock) = analysis.get("is_mock_data") {
                if !is_mock.as_bool().unwrap_or(false) {
                    messages.push("Stub implementation should have is_mock_data=true".to_string());
                }
            }
        }

        // Check traditions are listed
        if output.result.get("traditions").is_none() {
            messages.push("Missing 'traditions' field - should list face reading traditions".to_string());
            valid = false;
        }

        // Check notice is present (important for stub)
        if output.result.get("notice").is_none() {
            messages.push("Missing 'notice' field explaining stub status".to_string());
            valid = false;
        }

        let confidence = if valid { 1.0 } else { 0.0 };

        Ok(ValidationResult {
            valid,
            confidence,
            messages,
        })
    }

    fn cache_key(&self, input: &EngineInput) -> String {
        // For mock implementation, cache key is based on seed if provided
        if let Some(seed) = input.options.get("seed").and_then(|v| v.as_u64()) {
            format!("face-reading:mock:seed:{}", seed)
        } else {
            // Without seed, each call is unique (timestamp-based)
            format!("face-reading:mock:{}", Utc::now().timestamp_nanos_opt().unwrap_or(0))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use noesis_core::Precision;

    fn create_test_input() -> EngineInput {
        EngineInput {
            birth_data: None,
            current_time: Utc::now(),
            location: None,
            precision: Precision::Standard,
            options: HashMap::new(),
        }
    }

    fn create_seeded_input(seed: u64) -> EngineInput {
        let mut options = HashMap::new();
        options.insert("seed".to_string(), json!(seed));
        
        EngineInput {
            birth_data: None,
            current_time: Utc::now(),
            location: None,
            precision: Precision::Standard,
            options,
        }
    }

    #[tokio::test]
    async fn test_engine_creation() {
        let engine = FaceReadingEngine::new();
        assert_eq!(engine.engine_id(), "face-reading");
        assert_eq!(engine.engine_name(), "Face Reading");
        assert_eq!(engine.required_phase(), 1);
    }

    #[tokio::test]
    async fn test_calculate_returns_mock_data() {
        let engine = FaceReadingEngine::new();
        let input = create_test_input();

        let result = engine.calculate(input).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert_eq!(output.engine_id, "face-reading");
        
        // Check it's marked as mock data
        let is_mock = output.result
            .get("analysis")
            .and_then(|a| a.get("is_mock_data"))
            .and_then(|v| v.as_bool());
        assert_eq!(is_mock, Some(true));
    }

    #[tokio::test]
    async fn test_calculate_has_required_fields() {
        let engine = FaceReadingEngine::new();
        let input = create_test_input();

        let output = engine.calculate(input).await.unwrap();

        // Check top-level fields
        assert!(output.result.get("analysis").is_some());
        assert!(output.result.get("notice").is_some());
        assert!(output.result.get("traditions").is_some());
        assert!(output.result.get("future_capabilities").is_some());
        assert!(output.result.get("disclaimer").is_some());

        // Check analysis fields
        let analysis = output.result.get("analysis").unwrap();
        assert!(analysis.get("constitution").is_some());
        assert!(analysis.get("elemental_balance").is_some());
        assert!(analysis.get("personality_indicators").is_some());
        assert!(analysis.get("health_indicators").is_some());
    }

    #[tokio::test]
    async fn test_witness_prompt_non_empty() {
        let engine = FaceReadingEngine::new();
        let input = create_test_input();

        let output = engine.calculate(input).await.unwrap();
        assert!(!output.witness_prompt.is_empty());
        assert!(output.witness_prompt.contains('?'), "Witness prompt should be a question");
    }

    #[tokio::test]
    async fn test_seeded_reproducibility() {
        let engine = FaceReadingEngine::new();
        
        let output1 = engine.calculate(create_seeded_input(12345)).await.unwrap();
        let output2 = engine.calculate(create_seeded_input(12345)).await.unwrap();

        // With same seed, constitution should be identical
        let dosha1 = output1.result["analysis"]["constitution"]["primary_dosha"].clone();
        let dosha2 = output2.result["analysis"]["constitution"]["primary_dosha"].clone();
        assert_eq!(dosha1, dosha2);
    }

    #[tokio::test]
    async fn test_validation_passes_for_valid_output() {
        let engine = FaceReadingEngine::new();
        let input = create_test_input();

        let output = engine.calculate(input).await.unwrap();
        let validation = engine.validate(&output).await.unwrap();

        assert!(validation.valid, "Validation should pass: {:?}", validation.messages);
        assert_eq!(validation.confidence, 1.0);
    }

    #[tokio::test]
    async fn test_validation_fails_for_empty_prompt() {
        let engine = FaceReadingEngine::new();
        
        let output = EngineOutput {
            engine_id: "face-reading".to_string(),
            result: json!({
                "analysis": {
                    "constitution": {},
                    "elemental_balance": {},
                    "personality_indicators": [],
                    "is_mock_data": true,
                },
                "notice": "test",
                "traditions": [],
            }),
            witness_prompt: "".to_string(),
            consciousness_level: 2,
            metadata: CalculationMetadata {
                calculation_time_ms: 1.0,
                backend: "test".to_string(),
                precision_achieved: "test".to_string(),
                cached: false,
                timestamp: Utc::now(),
            },
        };

        let validation = engine.validate(&output).await.unwrap();
        assert!(!validation.valid);
        assert!(validation.messages.iter().any(|m| m.contains("empty")));
    }

    #[tokio::test]
    async fn test_cache_key_with_seed() {
        let engine = FaceReadingEngine::new();
        let input = create_seeded_input(42);

        let key = engine.cache_key(&input);
        assert!(key.contains("seed:42"));
    }

    #[tokio::test]
    async fn test_cache_key_without_seed() {
        let engine = FaceReadingEngine::new();
        let input = create_test_input();

        let key = engine.cache_key(&input);
        assert!(key.starts_with("face-reading:mock:"));
        assert!(!key.contains("seed"));
    }

    #[tokio::test]
    async fn test_metadata_indicates_stub() {
        let engine = FaceReadingEngine::new();
        let input = create_test_input();

        let output = engine.calculate(input).await.unwrap();
        assert_eq!(output.metadata.backend, "mock-stub");
        assert_eq!(output.metadata.precision_achieved, "simulated");
    }

    #[tokio::test]
    async fn test_traditions_listed() {
        let engine = FaceReadingEngine::new();
        let input = create_test_input();

        let output = engine.calculate(input).await.unwrap();
        let traditions = output.result.get("traditions").unwrap().as_array().unwrap();
        
        assert!(traditions.len() >= 3);
        let tradition_strs: Vec<&str> = traditions.iter()
            .filter_map(|v| v.as_str())
            .collect();
        
        assert!(tradition_strs.iter().any(|t| t.contains("Mian Xiang")));
        assert!(tradition_strs.iter().any(|t| t.contains("Ayurvedic")));
        assert!(tradition_strs.iter().any(|t| t.contains("Physiognomy")));
    }
}
