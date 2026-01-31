//! ConsciousnessEngine trait implementation for Gene Keys
//!
//! Integrates Gene Keys calculations with the Noesis platform architecture.
//! Supports two input modes:
//! 1. birth_data → calculate HD first → derive Gene Keys
//! 2. hd_gates provided → directly map to Gene Keys

use async_trait::async_trait;
use chrono::Utc;
use noesis_core::{
    ConsciousnessEngine, EngineError, EngineInput, EngineOutput, ValidationResult,
    CalculationMetadata,
};
use serde_json::{json, Value};
use std::sync::Arc;
use std::time::Instant;

use crate::{
    mapping::{map_hd_to_gene_keys, calculate_activation_sequences},
    models::{GeneKeysChart, GeneKeyActivation},
    witness::generate_witness_prompt,
    frequency::assess_frequencies,
    wisdom::get_gene_key,
};

/// Gene Keys consciousness engine implementing the universal trait
pub struct GeneKeysEngine {
    engine_id: String,
    engine_name: String,
    hd_engine: Option<Arc<engine_human_design::HumanDesignEngine>>,
}

impl GeneKeysEngine {
    /// Create a new Gene Keys engine instance without HD engine dependency
    pub fn new() -> Self {
        Self {
            engine_id: "gene-keys".to_string(),
            engine_name: "Gene Keys".to_string(),
            hd_engine: None,
        }
    }
    
    /// Create a new Gene Keys engine with HD engine dependency
    pub fn with_hd_engine(hd_engine: Arc<engine_human_design::HumanDesignEngine>) -> Self {
        Self {
            engine_id: "gene-keys".to_string(),
            engine_name: "Gene Keys".to_string(),
            hd_engine: Some(hd_engine),
        }
    }
    
    /// Extract HD gates from options (Mode 2)
    fn extract_hd_gates_from_options(options: &std::collections::HashMap<String, Value>) 
        -> Result<(u8, u8, u8, u8), EngineError> {
        let hd_gates = options.get("hd_gates")
            .ok_or_else(|| EngineError::ValidationError(
                "Missing 'hd_gates' in options".to_string()
            ))?;

        let personality_sun = hd_gates.get("personality_sun")
            .and_then(|v| v.as_u64())
            .map(|v| v as u8)
            .ok_or_else(|| EngineError::ValidationError(
                "Missing or invalid 'personality_sun' in hd_gates".to_string()
            ))?;

        let personality_earth = hd_gates.get("personality_earth")
            .and_then(|v| v.as_u64())
            .map(|v| v as u8)
            .ok_or_else(|| EngineError::ValidationError(
                "Missing or invalid 'personality_earth' in hd_gates".to_string()
            ))?;

        let design_sun = hd_gates.get("design_sun")
            .and_then(|v| v.as_u64())
            .map(|v| v as u8)
            .ok_or_else(|| EngineError::ValidationError(
                "Missing or invalid 'design_sun' in hd_gates".to_string()
            ))?;

        let design_earth = hd_gates.get("design_earth")
            .and_then(|v| v.as_u64())
            .map(|v| v as u8)
            .ok_or_else(|| EngineError::ValidationError(
                "Missing or invalid 'design_earth' in hd_gates".to_string()
            ))?;

        // Validate gate ranges (1-64)
        for (name, gate) in [
            ("personality_sun", personality_sun),
            ("personality_earth", personality_earth),
            ("design_sun", design_sun),
            ("design_earth", design_earth),
        ] {
            if !(1..=64).contains(&gate) {
                return Err(EngineError::ValidationError(
                    format!("Invalid gate number for {}: {} (must be 1-64)", name, gate)
                ));
            }
        }
        
        Ok((personality_sun, personality_earth, design_sun, design_earth))
    }
    
    /// Create Gene Keys chart from gates only (simplified version)
    fn create_chart_from_gates(
        personality_sun: u8,
        personality_earth: u8,
        design_sun: u8,
        design_earth: u8,
    ) -> Result<GeneKeysChart, EngineError> {
        use crate::models::{ActivationSequence, ActivationSource};
        
        let activation_sequence = ActivationSequence {
            lifes_work: (personality_sun, personality_earth),
            evolution: (design_sun, design_earth),
            radiance: (personality_sun, design_sun),
            purpose: (personality_earth, design_earth),
        };
        
        // Create minimal active_keys with just the 4 core activations
        let active_keys = vec![
            GeneKeyActivation {
                key_number: personality_sun,
                line: 3, // Default line
                source: ActivationSource::PersonalitySun,
                gene_key_data: get_gene_key(personality_sun).cloned(),
            },
            GeneKeyActivation {
                key_number: personality_earth,
                line: 3,
                source: ActivationSource::PersonalityEarth,
                gene_key_data: get_gene_key(personality_earth).cloned(),
            },
            GeneKeyActivation {
                key_number: design_sun,
                line: 3,
                source: ActivationSource::DesignSun,
                gene_key_data: get_gene_key(design_sun).cloned(),
            },
            GeneKeyActivation {
                key_number: design_earth,
                line: 3,
                source: ActivationSource::DesignEarth,
                gene_key_data: get_gene_key(design_earth).cloned(),
            },
        ];
        
        Ok(GeneKeysChart {
            activation_sequence,
            active_keys,
        })
    }
    
    /// Serialize GeneKeysChart to JSON value
    fn serialize_chart(chart: &GeneKeysChart) -> Value {
        // Enrich active keys with full Gene Key data
        let enriched_keys: Vec<Value> = chart.active_keys.iter().map(|ak| {
            let mut key_data = json!({
                "key_number": ak.key_number,
                "line": ak.line,
                "source": format!("{:?}", ak.source),
            });
            
            if let Some(gk) = &ak.gene_key_data {
                key_data["name"] = json!(gk.name);
                key_data["shadow"] = json!(gk.shadow);
                key_data["gift"] = json!(gk.gift);
                key_data["siddhi"] = json!(gk.siddhi);
            }
            
            key_data
        }).collect();
        
        // Calculate frequency assessments
        let frequency_assessments = assess_frequencies(chart, None);
        
        json!({
            "activation_sequence": {
                "lifes_work": [chart.activation_sequence.lifes_work.0, chart.activation_sequence.lifes_work.1],
                "evolution": [chart.activation_sequence.evolution.0, chart.activation_sequence.evolution.1],
                "radiance": [chart.activation_sequence.radiance.0, chart.activation_sequence.radiance.1],
                "purpose": [chart.activation_sequence.purpose.0, chart.activation_sequence.purpose.1],
            },
            "active_keys": enriched_keys,
            "frequency_assessments": frequency_assessments,
        })
    }
}

impl Default for GeneKeysEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ConsciousnessEngine for GeneKeysEngine {
    fn engine_id(&self) -> &str {
        &self.engine_id
    }
    
    fn engine_name(&self) -> &str {
        &self.engine_name
    }
    
    fn required_phase(&self) -> u8 {
        2 // Requires deeper consciousness than HD (phase 1)
    }
    
    async fn calculate(&self, input: EngineInput) -> Result<EngineOutput, EngineError> {
        let start = Instant::now();
        
        let chart = if input.birth_data.is_some() {
            // Mode 1: Calculate from birth_data (requires HD engine)
            let hd_engine = self.hd_engine.as_ref()
                .ok_or_else(|| EngineError::CalculationError(
                    "HD engine not available for birth_data calculation".to_string()
                ))?;
            
            // Call HD engine to get HD chart
            let hd_output = hd_engine.calculate(input.clone()).await?;
            
            // Parse HD chart from output
            let hd_chart: engine_human_design::HDChart = serde_json::from_value(hd_output.result)
                .map_err(|e| EngineError::CalculationError(
                    format!("Failed to parse HD chart: {}", e)
                ))?;
            
            // Map HD to Gene Keys
            let mut gene_key_activations = map_hd_to_gene_keys(&hd_chart);
            
            // Enrich with full Gene Key data
            for activation in &mut gene_key_activations {
                activation.gene_key_data = get_gene_key(activation.key_number).cloned();
            }
            
            let activation_sequence = calculate_activation_sequences(&hd_chart)
                .map_err(|e| EngineError::CalculationError(
                    format!("Failed to calculate activation sequences: {}", e)
                ))?;
            
            GeneKeysChart {
                activation_sequence,
                active_keys: gene_key_activations,
            }
        } else if input.options.contains_key("hd_gates") {
            // Mode 2: Extract gates from options
            let (ps, pe, ds, de) = Self::extract_hd_gates_from_options(&input.options)?;
            Self::create_chart_from_gates(ps, pe, ds, de)?
        } else {
            return Err(EngineError::ValidationError(
                "Gene Keys requires either birth_data or hd_gates in options".to_string()
            ));
        };
        
        // Get consciousness level from input
        let consciousness_level = input.options.get("consciousness_level")
            .and_then(|v| v.as_u64())
            .map(|v| v as u8)
            .unwrap_or(3); // Default to Gift level
        
        // Generate witness prompt
        let witness_prompt = generate_witness_prompt(&chart, consciousness_level);
        
        // Ensure witness prompt is non-empty (Rule 5)
        if witness_prompt.is_empty() {
            return Err(EngineError::CalculationError(
                "Witness prompt generation failed: empty result".to_string()
            ));
        }
        
        let elapsed = start.elapsed();
        
        Ok(EngineOutput {
            engine_id: self.engine_id.clone(),
            result: Self::serialize_chart(&chart),
            witness_prompt,
            consciousness_level,
            metadata: CalculationMetadata {
                calculation_time_ms: elapsed.as_secs_f64() * 1000.0,
                backend: if input.birth_data.is_some() { "hd-derived" } else { "hd-gates" }.to_string(),
                precision_achieved: format!("{:?}", input.precision),
                cached: false,
                timestamp: Utc::now(),
            },
        })
    }
    
    async fn validate(&self, output: &EngineOutput) -> Result<ValidationResult, EngineError> {
        let mut messages = vec![];
        let mut valid = true;
        
        // Check witness prompt is non-empty (Rule 5)
        if output.witness_prompt.is_empty() {
            messages.push("Witness prompt is empty".to_string());
            valid = false;
        }
        
        // Check result has expected fields
        if output.result.get("activation_sequence").is_none() {
            messages.push("Missing 'activation_sequence' field in result".to_string());
            valid = false;
        }
        
        if output.result.get("active_keys").is_none() {
            messages.push("Missing 'active_keys' field in result".to_string());
            valid = false;
        }
        
        // Check activation_sequence has all 4 sequences
        if let Some(seq) = output.result.get("activation_sequence") {
            for field in ["lifes_work", "evolution", "radiance", "purpose"] {
                if seq.get(field).is_none() {
                    messages.push(format!("Missing '{}' in activation_sequence", field));
                    valid = false;
                }
            }
        }
        
        // Check consciousness level is in valid range
        if output.consciousness_level > 6 {
            messages.push(format!("Invalid consciousness_level: {}", output.consciousness_level));
            valid = false;
        }
        
        // Check archetypal depth preserved (frequency_assessments should exist)
        if output.result.get("frequency_assessments").is_none() {
            messages.push("Missing 'frequency_assessments' - archetypal depth not preserved".to_string());
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
        if let Some(birth_data) = &input.birth_data {
            // Mode 1: birth_data cache key
            format!(
                "gk:{}:{}:{:.4}:{:.4}",
                birth_data.date,
                birth_data.time.as_ref().unwrap_or(&"00:00".to_string()),
                birth_data.latitude,
                birth_data.longitude
            )
        } else if input.options.contains_key("hd_gates") {
            // Mode 2: hd_gates cache key
            if let Ok((ps, pe, ds, de)) = Self::extract_hd_gates_from_options(&input.options) {
                format!("gk:gates:{}:{}:{}:{}", ps, pe, ds, de)
            } else {
                format!("gk:invalid:{}", Utc::now().timestamp())
            }
        } else {
            format!("gk:invalid:{}", Utc::now().timestamp())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use noesis_core::{BirthData, Precision};
    use std::collections::HashMap;
    
    fn create_test_input_with_gates() -> EngineInput {
        let mut options = HashMap::new();
        options.insert("hd_gates".to_string(), json!({
            "personality_sun": 17,
            "personality_earth": 18,
            "design_sun": 45,
            "design_earth": 26
        }));
        
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
        let engine = GeneKeysEngine::new();
        assert_eq!(engine.engine_id(), "gene-keys");
        assert_eq!(engine.engine_name(), "Gene Keys");
        assert_eq!(engine.required_phase(), 2);
    }
    
    #[tokio::test]
    async fn test_extract_hd_gates_from_options() {
        let input = create_test_input_with_gates();
        let result = GeneKeysEngine::extract_hd_gates_from_options(&input.options);
        
        assert!(result.is_ok());
        let (ps, pe, ds, de) = result.unwrap();
        assert_eq!(ps, 17);
        assert_eq!(pe, 18);
        assert_eq!(ds, 45);
        assert_eq!(de, 26);
    }
    
    #[tokio::test]
    async fn test_invalid_gate_range() {
        let mut options = HashMap::new();
        options.insert("hd_gates".to_string(), json!({
            "personality_sun": 65, // Invalid (> 64)
            "personality_earth": 18,
            "design_sun": 45,
            "design_earth": 26
        }));
        
        let result = GeneKeysEngine::extract_hd_gates_from_options(&options);
        assert!(result.is_err());
    }
    
    #[tokio::test]
    async fn test_calculate_with_gates() {
        let engine = GeneKeysEngine::new();
        let input = create_test_input_with_gates();
        
        let result = engine.calculate(input).await;
        assert!(result.is_ok(), "Calculation should succeed with hd_gates");
        
        let output = result.unwrap();
        assert_eq!(output.engine_id, "gene-keys");
        assert!(!output.witness_prompt.is_empty());
        assert_eq!(output.consciousness_level, 3); // Default
    }
    
    #[tokio::test]
    async fn test_cache_key_with_gates() {
        let engine = GeneKeysEngine::new();
        let input = create_test_input_with_gates();
        
        let key = engine.cache_key(&input);
        assert!(key.starts_with("gk:gates:"));
        assert!(key.contains("17:18:45:26"));
    }
    
    #[tokio::test]
    async fn test_validation_checks_witness_prompt() {
        let engine = GeneKeysEngine::new();
        let mut output = EngineOutput {
            engine_id: "gene-keys".to_string(),
            result: json!({
                "activation_sequence": {
                    "lifes_work": [17, 18],
                    "evolution": [45, 26],
                    "radiance": [17, 45],
                    "purpose": [18, 26]
                },
                "active_keys": [],
                "frequency_assessments": []
            }),
            witness_prompt: "".to_string(), // Empty
            consciousness_level: 3,
            metadata: CalculationMetadata {
                calculation_time_ms: 10.0,
                backend: "test".to_string(),
                precision_achieved: "Standard".to_string(),
                cached: false,
                timestamp: Utc::now(),
            },
        };
        
        let result = engine.validate(&output).await.unwrap();
        assert!(!result.valid);
        assert!(result.messages.iter().any(|m| m.contains("empty")));
        
        // Fix it
        output.witness_prompt = "Test question?".to_string();
        let result = engine.validate(&output).await.unwrap();
        assert!(result.valid);
    }
    
    #[tokio::test]
    async fn test_validation_checks_archetypal_depth() {
        let engine = GeneKeysEngine::new();
        let output = EngineOutput {
            engine_id: "gene-keys".to_string(),
            result: json!({
                "activation_sequence": {
                    "lifes_work": [17, 18],
                    "evolution": [45, 26],
                    "radiance": [17, 45],
                    "purpose": [18, 26]
                },
                "active_keys": []
                // Missing frequency_assessments
            }),
            witness_prompt: "Test?".to_string(),
            consciousness_level: 3,
            metadata: CalculationMetadata {
                calculation_time_ms: 10.0,
                backend: "test".to_string(),
                precision_achieved: "Standard".to_string(),
                cached: false,
                timestamp: Utc::now(),
            },
        };
        
        let result = engine.validate(&output).await.unwrap();
        assert!(!result.valid);
        assert!(result.messages.iter().any(|m| m.contains("frequency_assessments")));
    }
    
    #[tokio::test]
    async fn test_missing_input_data() {
        let engine = GeneKeysEngine::new();
        let input = EngineInput {
            birth_data: None,
            current_time: Utc::now(),
            location: None,
            precision: Precision::Standard,
            options: HashMap::new(), // No hd_gates
        };
        
        let result = engine.calculate(input).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("requires either"));
    }
}
