//! ConsciousnessEngine implementation for Biofield
//!
//! Stub implementation that returns mock biofield data.
//! Full implementation requires PIP (Polycontrast Interference Photography) hardware.

use async_trait::async_trait;
use chrono::Utc;
use noesis_core::{
    ConsciousnessEngine, EngineError, EngineInput, EngineOutput, ValidationResult,
    CalculationMetadata,
};
use serde_json::{json, Value};
use std::time::Instant;

use crate::mock::{generate_mock_metrics, generate_metrics_for_user};
use crate::models::{BiofieldAnalysis, BiofieldMetrics};
use crate::wisdom::{get_metric_interpretation, get_chakra_wisdom};
use crate::witness::generate_witness_prompt;

/// Biofield consciousness engine
///
/// Analyzes biofield energy patterns from PIP device data.
/// Currently returns mock data - full implementation requires hardware.
pub struct BiofieldEngine {
    engine_id: String,
    engine_name: String,
}

impl BiofieldEngine {
    /// Create a new Biofield engine instance
    pub fn new() -> Self {
        Self {
            engine_id: "biofield".to_string(),
            engine_name: "Biofield".to_string(),
        }
    }
    
    /// Generate interpretation text from metrics
    fn generate_interpretation(metrics: &BiofieldMetrics) -> String {
        let mut parts = Vec::new();
        
        // Interpret vitality
        if metrics.vitality_index > 0.7 {
            parts.push("Overall vitality appears strong".to_string());
        } else if metrics.vitality_index < 0.4 {
            parts.push("Vitality may benefit from attention".to_string());
        } else {
            parts.push("Vitality is in a moderate range".to_string());
        }
        
        // Interpret coherence
        if let Some(interp) = get_metric_interpretation("coherence") {
            if metrics.coherence < interp.optimal_range.0 {
                parts.push("Coherence patterns suggest potential for more integration".to_string());
            } else if metrics.coherence > interp.optimal_range.1 {
                parts.push("Coherence is notably high, suggesting aligned energy flow".to_string());
            }
        }
        
        // Interpret symmetry
        if metrics.symmetry < 0.5 {
            parts.push("Left-right balance shows some asymmetry".to_string());
        }
        
        // Note about mock data
        parts.push("Note: This interpretation is based on simulated data".to_string());
        
        parts.join(". ") + "."
    }
    
    /// Identify areas that may benefit from attention
    fn identify_areas_of_attention(metrics: &BiofieldMetrics) -> Vec<String> {
        let mut areas = Vec::new();
        
        // Check each chakra
        for reading in &metrics.chakra_readings {
            if reading.activity_level < 0.4 {
                if let Some(wisdom) = get_chakra_wisdom(reading.chakra) {
                    areas.push(format!(
                        "{} ({}) - lower activity observed",
                        wisdom.name, wisdom.location
                    ));
                }
            }
            
            if reading.balance.abs() > 0.4 {
                let side = if reading.balance > 0.0 { "right-dominant" } else { "left-dominant" };
                areas.push(format!(
                    "{} chakra shows {} pattern",
                    reading.chakra.name(), side
                ));
            }
        }
        
        // Check overall metrics
        if metrics.coherence < 0.4 {
            areas.push("Overall coherence may benefit from grounding practices".to_string());
        }
        
        if metrics.entropy < 0.3 {
            areas.push("Energy patterns appear uniform - gentle movement may help".to_string());
        }
        
        if areas.is_empty() {
            areas.push("No specific areas require immediate attention".to_string());
        }
        
        areas
    }
    
    /// Perform the biofield analysis (currently returns mock data)
    fn analyze(&self, input: &EngineInput) -> Result<BiofieldAnalysis, EngineError> {
        // Get seed from options if provided for reproducible results
        let seed = input.options.get("seed")
            .and_then(|v| v.as_u64());
        
        // Or use user_id for consistent personal readings
        let metrics = if let Some(user_id) = input.options.get("user_id").and_then(|v| v.as_str()) {
            generate_metrics_for_user(user_id)
        } else {
            generate_mock_metrics(seed)
        };
        
        let interpretation = Self::generate_interpretation(&metrics);
        let areas_of_attention = Self::identify_areas_of_attention(&metrics);
        
        Ok(BiofieldAnalysis {
            metrics,
            interpretation,
            areas_of_attention,
            is_mock_data: true,
        })
    }
    
    /// Serialize analysis result to JSON
    fn serialize_result(analysis: &BiofieldAnalysis) -> Value {
        // Serialize chakra readings with additional metadata
        let chakra_readings: Vec<Value> = analysis.metrics.chakra_readings.iter().map(|r| {
            let mut reading = json!({
                "chakra": format!("{:?}", r.chakra),
                "chakra_name": r.chakra.name(),
                "activity_level": r.activity_level,
                "balance": r.balance,
                "color_intensity": r.color_intensity,
            });
            
            // Add wisdom data if available
            if let Some(wisdom) = get_chakra_wisdom(r.chakra) {
                reading["location"] = json!(wisdom.location);
                reading["element"] = json!(wisdom.element);
            }
            
            reading
        }).collect();
        
        json!({
            "metrics": {
                "fractal_dimension": analysis.metrics.fractal_dimension,
                "entropy": analysis.metrics.entropy,
                "coherence": analysis.metrics.coherence,
                "symmetry": analysis.metrics.symmetry,
                "vitality_index": analysis.metrics.vitality_index,
                "timestamp": analysis.metrics.timestamp,
            },
            "chakra_readings": chakra_readings,
            "interpretation": analysis.interpretation,
            "areas_of_attention": analysis.areas_of_attention,
            "is_mock_data": analysis.is_mock_data,
        })
    }
}

impl Default for BiofieldEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ConsciousnessEngine for BiofieldEngine {
    fn engine_id(&self) -> &str {
        &self.engine_id
    }
    
    fn engine_name(&self) -> &str {
        &self.engine_name
    }
    
    fn required_phase(&self) -> u8 {
        1  // Requires somatic awareness (phase 1)
    }
    
    async fn calculate(&self, input: EngineInput) -> Result<EngineOutput, EngineError> {
        let start = Instant::now();
        
        // Perform analysis
        let analysis = self.analyze(&input)?;
        
        // Generate witness prompt
        let witness_prompt = generate_witness_prompt(&analysis);
        
        // Ensure witness prompt is non-empty
        if witness_prompt.is_empty() {
            return Err(EngineError::CalculationError(
                "Witness prompt generation failed: empty result".to_string()
            ));
        }
        
        // Get consciousness level from input
        let consciousness_level = input.options.get("consciousness_level")
            .and_then(|v| v.as_u64())
            .map(|v| v as u8)
            .unwrap_or(1);
        
        let elapsed = start.elapsed();
        
        // Build result with mock notice
        let mut result = Self::serialize_result(&analysis);
        result["notice"] = json!(
            "This is simulated data. Full biofield analysis requires PIP hardware integration."
        );
        result["future_capabilities"] = json!([
            "Real-time biofield imaging",
            "Chakra activity measurement",
            "Aura color spectrum analysis",
            "Energy flow pattern tracking",
            "Biofield coherence monitoring",
            "Before/after intervention comparison",
        ]);
        
        Ok(EngineOutput {
            engine_id: self.engine_id.clone(),
            result,
            witness_prompt,
            consciousness_level,
            metadata: CalculationMetadata {
                calculation_time_ms: elapsed.as_secs_f64() * 1000.0,
                backend: "mock".to_string(),
                precision_achieved: "simulated".to_string(),
                cached: false,
                timestamp: Utc::now(),
            },
        })
    }
    
    async fn validate(&self, output: &EngineOutput) -> Result<ValidationResult, EngineError> {
        let mut messages = Vec::new();
        let mut valid = true;
        
        // Check witness prompt is non-empty
        if output.witness_prompt.is_empty() {
            messages.push("Witness prompt is empty".to_string());
            valid = false;
        }
        
        // Check result has expected fields
        if output.result.get("metrics").is_none() {
            messages.push("Missing 'metrics' field in result".to_string());
            valid = false;
        }
        
        if output.result.get("chakra_readings").is_none() {
            messages.push("Missing 'chakra_readings' field in result".to_string());
            valid = false;
        }
        
        // Check is_mock_data flag is present
        if output.result.get("is_mock_data").is_none() {
            messages.push("Missing 'is_mock_data' flag".to_string());
            valid = false;
        }
        
        // Validate metrics are in range
        if let Some(metrics) = output.result.get("metrics") {
            if let Some(fd) = metrics.get("fractal_dimension").and_then(|v| v.as_f64()) {
                if fd < 1.0 || fd > 2.0 {
                    messages.push(format!("fractal_dimension {} out of range [1.0, 2.0]", fd));
                    valid = false;
                }
            }
            
            for field in ["entropy", "coherence", "symmetry", "vitality_index"] {
                if let Some(val) = metrics.get(field).and_then(|v| v.as_f64()) {
                    if val < 0.0 || val > 1.0 {
                        messages.push(format!("{} {} out of range [0.0, 1.0]", field, val));
                        valid = false;
                    }
                }
            }
        }
        
        // Check chakra readings count
        if let Some(readings) = output.result.get("chakra_readings").and_then(|v| v.as_array()) {
            if readings.len() != 7 {
                messages.push(format!("Expected 7 chakra readings, got {}", readings.len()));
                valid = false;
            }
        }
        
        let confidence = if valid { 1.0 } else { 0.0 };
        
        Ok(ValidationResult {
            valid,
            confidence,
            messages,
        })
    }
    
    fn cache_key(&self, input: &EngineInput) -> String {
        // Include seed or user_id in cache key for reproducibility
        if let Some(user_id) = input.options.get("user_id").and_then(|v| v.as_str()) {
            format!("biofield:user:{}", user_id)
        } else if let Some(seed) = input.options.get("seed").and_then(|v| v.as_u64()) {
            format!("biofield:seed:{}", seed)
        } else {
            // No caching for unseeded mock data
            format!("biofield:random:{}", Utc::now().timestamp_nanos_opt().unwrap_or(0))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use noesis_core::Precision;
    use std::collections::HashMap;
    
    fn create_test_input() -> EngineInput {
        let mut options = HashMap::new();
        options.insert("seed".to_string(), json!(42));
        
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
        let engine = BiofieldEngine::new();
        assert_eq!(engine.engine_id(), "biofield");
        assert_eq!(engine.engine_name(), "Biofield");
        assert_eq!(engine.required_phase(), 1);
    }
    
    #[tokio::test]
    async fn test_calculate_returns_mock_data() {
        let engine = BiofieldEngine::new();
        let input = create_test_input();
        
        let result = engine.calculate(input).await;
        assert!(result.is_ok(), "Calculation should succeed");
        
        let output = result.unwrap();
        assert_eq!(output.engine_id, "biofield");
        assert!(!output.witness_prompt.is_empty());
        
        // Check is_mock_data flag
        let is_mock = output.result.get("is_mock_data")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        assert!(is_mock, "Should indicate mock data");
        
        // Check notice is present
        assert!(output.result.get("notice").is_some(), "Should have notice about mock data");
        
        // Check future_capabilities is present
        assert!(output.result.get("future_capabilities").is_some());
    }
    
    #[tokio::test]
    async fn test_calculate_with_seed_is_reproducible() {
        let engine = BiofieldEngine::new();
        
        let mut options = HashMap::new();
        options.insert("seed".to_string(), json!(12345));
        
        let input1 = EngineInput {
            birth_data: None,
            current_time: Utc::now(),
            location: None,
            precision: Precision::Standard,
            options: options.clone(),
        };
        
        let input2 = EngineInput {
            birth_data: None,
            current_time: Utc::now(),
            location: None,
            precision: Precision::Standard,
            options,
        };
        
        let output1 = engine.calculate(input1).await.unwrap();
        let output2 = engine.calculate(input2).await.unwrap();
        
        // Compare metrics (excluding timestamp)
        let fd1 = output1.result["metrics"]["fractal_dimension"].as_f64();
        let fd2 = output2.result["metrics"]["fractal_dimension"].as_f64();
        assert_eq!(fd1, fd2, "Same seed should produce same results");
    }
    
    #[tokio::test]
    async fn test_calculate_with_user_id() {
        let engine = BiofieldEngine::new();
        
        let mut options = HashMap::new();
        options.insert("user_id".to_string(), json!("test_user_123"));
        
        let input = EngineInput {
            birth_data: None,
            current_time: Utc::now(),
            location: None,
            precision: Precision::Standard,
            options,
        };
        
        let output = engine.calculate(input).await.unwrap();
        assert!(output.result.get("metrics").is_some());
    }
    
    #[tokio::test]
    async fn test_validate_output() {
        let engine = BiofieldEngine::new();
        let input = create_test_input();
        
        let output = engine.calculate(input).await.unwrap();
        let validation = engine.validate(&output).await.unwrap();
        
        assert!(validation.valid, "Valid output should pass validation: {:?}", validation.messages);
        assert_eq!(validation.confidence, 1.0);
    }
    
    #[tokio::test]
    async fn test_validate_detects_empty_witness_prompt() {
        let engine = BiofieldEngine::new();
        
        let output = EngineOutput {
            engine_id: "biofield".to_string(),
            result: json!({
                "metrics": {
                    "fractal_dimension": 1.5,
                    "entropy": 0.5,
                    "coherence": 0.6,
                    "symmetry": 0.7,
                    "vitality_index": 0.6,
                },
                "chakra_readings": [],
                "is_mock_data": true,
            }),
            witness_prompt: "".to_string(),  // Empty!
            consciousness_level: 1,
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
        let engine = BiofieldEngine::new();
        let input = create_test_input();
        
        let key = engine.cache_key(&input);
        assert!(key.starts_with("biofield:seed:"));
        assert!(key.contains("42"));
    }
    
    #[tokio::test]
    async fn test_cache_key_with_user_id() {
        let engine = BiofieldEngine::new();
        
        let mut options = HashMap::new();
        options.insert("user_id".to_string(), json!("user123"));
        
        let input = EngineInput {
            birth_data: None,
            current_time: Utc::now(),
            location: None,
            precision: Precision::Standard,
            options,
        };
        
        let key = engine.cache_key(&input);
        assert!(key.starts_with("biofield:user:"));
        assert!(key.contains("user123"));
    }
    
    #[tokio::test]
    async fn test_output_has_chakra_readings() {
        let engine = BiofieldEngine::new();
        let input = create_test_input();
        
        let output = engine.calculate(input).await.unwrap();
        
        let readings = output.result.get("chakra_readings")
            .and_then(|v| v.as_array())
            .expect("Should have chakra_readings array");
        
        assert_eq!(readings.len(), 7, "Should have 7 chakra readings");
        
        // Check first reading has expected fields
        let first = &readings[0];
        assert!(first.get("chakra").is_some());
        assert!(first.get("activity_level").is_some());
        assert!(first.get("balance").is_some());
        assert!(first.get("color_intensity").is_some());
    }
    
    #[tokio::test]
    async fn test_metrics_in_valid_ranges() {
        let engine = BiofieldEngine::new();
        
        for seed in 0..20 {
            let mut options = HashMap::new();
            options.insert("seed".to_string(), json!(seed));
            
            let input = EngineInput {
                birth_data: None,
                current_time: Utc::now(),
                location: None,
                precision: Precision::Standard,
                options,
            };
            
            let output = engine.calculate(input).await.unwrap();
            let metrics = output.result.get("metrics").unwrap();
            
            let fd = metrics["fractal_dimension"].as_f64().unwrap();
            assert!(fd >= 1.0 && fd <= 2.0, "fractal_dimension {} out of range", fd);
            
            for field in ["entropy", "coherence", "symmetry", "vitality_index"] {
                let val = metrics[field].as_f64().unwrap();
                assert!(val >= 0.0 && val <= 1.0, "{} {} out of range", field, val);
            }
        }
    }
}
