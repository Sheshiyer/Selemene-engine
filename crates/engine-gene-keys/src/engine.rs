//! ConsciousnessEngine trait implementation for Gene Keys
//!
//! Integrates Gene Keys calculations with the Noesis platform architecture.
//! Supports two input modes:
//! 1. birth_data → calculate HD first → derive Gene Keys
//! 2. hd_gates provided → directly map to Gene Keys

use async_trait::async_trait;
use chrono::Utc;
use noesis_core::{
    CalculationMetadata, ConsciousnessEngine, EngineError, EngineInput, EngineOutput,
    ValidationResult,
};
use serde_json::{json, Value};
use std::sync::Arc;
use std::time::Instant;

use crate::{
    frequency::assess_frequencies,
    models::{GeneKeyActivation, GeneKeysChart},
    wisdom::get_gene_key,
    witness::generate_witness_prompt,
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
    fn extract_hd_gates_from_options(
        options: &std::collections::HashMap<String, Value>,
    ) -> Result<(u8, u8, u8, u8), EngineError> {
        let hd_gates = options.get("hd_gates").ok_or_else(|| {
            EngineError::ValidationError("Missing 'hd_gates' in options".to_string())
        })?;

        let personality_sun = hd_gates
            .get("personality_sun")
            .and_then(|v| v.as_u64())
            .map(|v| v as u8)
            .ok_or_else(|| {
                EngineError::ValidationError(
                    "Missing or invalid 'personality_sun' in hd_gates".to_string(),
                )
            })?;

        let personality_earth = hd_gates
            .get("personality_earth")
            .and_then(|v| v.as_u64())
            .map(|v| v as u8)
            .ok_or_else(|| {
                EngineError::ValidationError(
                    "Missing or invalid 'personality_earth' in hd_gates".to_string(),
                )
            })?;

        let design_sun = hd_gates
            .get("design_sun")
            .and_then(|v| v.as_u64())
            .map(|v| v as u8)
            .ok_or_else(|| {
                EngineError::ValidationError(
                    "Missing or invalid 'design_sun' in hd_gates".to_string(),
                )
            })?;

        let design_earth = hd_gates
            .get("design_earth")
            .and_then(|v| v.as_u64())
            .map(|v| v as u8)
            .ok_or_else(|| {
                EngineError::ValidationError(
                    "Missing or invalid 'design_earth' in hd_gates".to_string(),
                )
            })?;

        // Validate gate ranges (1-64)
        for (name, gate) in [
            ("personality_sun", personality_sun),
            ("personality_earth", personality_earth),
            ("design_sun", design_sun),
            ("design_earth", design_earth),
        ] {
            if !(1..=64).contains(&gate) {
                return Err(EngineError::ValidationError(format!(
                    "Invalid gate number for {}: {} (must be 1-64)",
                    name, gate
                )));
            }
        }

        Ok((personality_sun, personality_earth, design_sun, design_earth))
    }

    /// Create Gene Keys chart from gates only (simplified version).
    ///
    /// # Known limitation: `line` is a placeholder on this path
    ///
    /// This constructor receives four gate numbers and nothing else, so the
    /// hexagram line is not derivable here and every activation is assigned a
    /// fixed `line: 3`. Any consumer reading `active_keys[*].line` from a chart
    /// built this way is reading a constant, not a calculation.
    ///
    /// This is not a value that can be guessed: the line comes from the
    /// fractional position within a gate, which requires the ephemeris
    /// longitude. The Human Design engine does compute it — its activations are
    /// shaped `{ gate, line, longitude }` — but the `hd_gates` option this path
    /// consumes carries only the four gate numbers (see
    /// [`Self::extract_hd_gates_from_options`]), so the line is discarded
    /// before it ever reaches here.
    ///
    /// Fixing it properly means widening the `hd_gates` option to carry the
    /// line alongside the gate and updating its producers. That is a contract
    /// change and deliberately out of scope; until then the limitation is
    /// documented rather than papered over.
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
                // Placeholder, not a calculation -- see the fn doc comment.
                line: 3,
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

    fn extract_gate_from_activations(
        activations: &serde_json::Map<String, Value>,
        planet: &str,
        block_name: &str,
    ) -> Result<u8, EngineError> {
        let gate = activations
            .get(planet)
            .and_then(|v| v.get("gate"))
            .and_then(|v| v.as_u64())
            .ok_or_else(|| {
                EngineError::CalculationError(format!(
                    "Missing or invalid '{}' gate in {}",
                    planet, block_name
                ))
            })?;

        if !(1..=64).contains(&gate) {
            return Err(EngineError::CalculationError(format!(
                "Invalid '{}' gate in {}: {} (must be 1-64)",
                planet, block_name, gate
            )));
        }

        Ok(gate as u8)
    }

    fn extract_hd_gates_from_hd_result(result: &Value) -> Result<(u8, u8, u8, u8), EngineError> {
        let personality = result
            .get("personality_activations")
            .and_then(|v| v.as_object())
            .ok_or_else(|| {
                EngineError::CalculationError(
                    "HD output missing personality_activations".to_string(),
                )
            })?;

        let design = result
            .get("design_activations")
            .and_then(|v| v.as_object())
            .ok_or_else(|| {
                EngineError::CalculationError("HD output missing design_activations".to_string())
            })?;

        let personality_sun =
            Self::extract_gate_from_activations(personality, "sun", "personality_activations")?;
        let personality_earth =
            Self::extract_gate_from_activations(personality, "earth", "personality_activations")?;
        let design_sun = Self::extract_gate_from_activations(design, "sun", "design_activations")?;
        let design_earth =
            Self::extract_gate_from_activations(design, "earth", "design_activations")?;

        Ok((personality_sun, personality_earth, design_sun, design_earth))
    }

    /// Serialize GeneKeysChart to JSON value.
    ///
    /// `consciousness_level` is threaded in because the frequency assessment
    /// derives `suggested_frequency` from it. Passing `None` there silently
    /// nulls that field for every active key, which is what this payload did
    /// until the level was wired through.
    fn serialize_chart(chart: &GeneKeysChart, consciousness_level: u8) -> Value {
        // Enrich active keys with full Gene Key data
        let enriched_keys: Vec<Value> = chart
            .active_keys
            .iter()
            .map(|ak| {
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
            })
            .collect();

        // Calculate frequency assessments
        let frequency_assessments = assess_frequencies(chart, Some(consciousness_level));

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
            let hd_engine = self.hd_engine.as_ref().ok_or_else(|| {
                EngineError::CalculationError(
                    "HD engine not available for birth_data calculation".to_string(),
                )
            })?;

            // Call HD engine to get HD chart
            let hd_output = hd_engine.calculate(input.clone()).await?;
            let (personality_sun, personality_earth, design_sun, design_earth) =
                Self::extract_hd_gates_from_hd_result(&hd_output.result)?;

            Self::create_chart_from_gates(
                personality_sun,
                personality_earth,
                design_sun,
                design_earth,
            )?
        } else if input.options.contains_key("hd_gates") {
            // Mode 2: Extract gates from options
            let (ps, pe, ds, de) = Self::extract_hd_gates_from_options(&input.options)?;
            Self::create_chart_from_gates(ps, pe, ds, de)?
        } else {
            return Err(EngineError::ValidationError(
                "Gene Keys requires either birth_data or hd_gates in options".to_string(),
            ));
        };

        // Get consciousness level from input
        let consciousness_level = input
            .options
            .get("consciousness_level")
            .and_then(|v| v.as_u64())
            .map(|v| v as u8)
            .unwrap_or(3); // Default to Gift level

        // Generate witness prompt
        let witness_prompt = generate_witness_prompt(&chart, consciousness_level);

        // Ensure witness prompt is non-empty (Rule 5)
        if witness_prompt.is_empty() {
            return Err(EngineError::CalculationError(
                "Witness prompt generation failed: empty result".to_string(),
            ));
        }

        let elapsed = start.elapsed();

        Ok(EngineOutput {
            engine_id: self.engine_id.clone(),
            result: Self::serialize_chart(&chart, consciousness_level),
            witness_prompt,
            consciousness_level,
            metadata: CalculationMetadata {
                calculation_time_ms: elapsed.as_secs_f64() * 1000.0,
                backend: if input.birth_data.is_some() {
                    "hd-derived"
                } else {
                    "hd-gates"
                }
                .to_string(),
                precision_achieved: format!("{:?}", input.precision),
                cached: false,
                timestamp: Utc::now(),
                engine_version: env!("CARGO_PKG_VERSION").to_string(),
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
            messages.push(format!(
                "Invalid consciousness_level: {}",
                output.consciousness_level
            ));
            valid = false;
        }

        // Check archetypal depth preserved (frequency_assessments should exist)
        if output.result.get("frequency_assessments").is_none() {
            messages.push(
                "Missing 'frequency_assessments' - archetypal depth not preserved".to_string(),
            );
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
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use noesis_core::Precision;
    use std::collections::HashMap;
    use std::sync::Arc;

    fn create_test_input_with_gates() -> EngineInput {
        let mut options = HashMap::new();
        options.insert(
            "hd_gates".to_string(),
            json!({
                "personality_sun": 17,
                "personality_earth": 18,
                "design_sun": 45,
                "design_earth": 26
            }),
        );

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
        options.insert(
            "hd_gates".to_string(),
            json!({
                "personality_sun": 65, // Invalid (> 64)
                "personality_earth": 18,
                "design_sun": 45,
                "design_earth": 26
            }),
        );

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

    /// Regression: `suggested_frequency` was null for every active key because
    /// `serialize_chart` called `assess_frequencies(chart, None)`. The level is
    /// now threaded through, so the field must be populated and must track the
    /// level: 0..=2 Shadow, 3..=4 Gift, 5..=6 Siddhi.
    #[tokio::test]
    async fn test_suggested_frequency_is_populated_and_tracks_level() {
        let engine = GeneKeysEngine::new();

        for (level, expected) in [(1_u64, "Shadow"), (3, "Gift"), (6, "Siddhi")] {
            let mut input = create_test_input_with_gates();
            input
                .options
                .insert("consciousness_level".to_string(), json!(level));

            let output = engine
                .calculate(input)
                .await
                .expect("calculation should succeed");

            let assessments = output.result["frequency_assessments"]
                .as_array()
                .expect("frequency_assessments should be an array");
            assert!(
                !assessments.is_empty(),
                "expected at least one frequency assessment"
            );

            for a in assessments {
                let suggested = &a["suggested_frequency"];
                assert!(
                    !suggested.is_null(),
                    "suggested_frequency must not be null at level {level}"
                );
                assert_eq!(
                    suggested.as_str(),
                    Some(expected),
                    "level {level} should map to {expected}"
                );
            }
        }
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
                engine_version: String::new(),
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
                engine_version: String::new(),
            },
        };

        let result = engine.validate(&output).await.unwrap();
        assert!(!result.valid);
        assert!(result
            .messages
            .iter()
            .any(|m| m.contains("frequency_assessments")));
    }

    #[tokio::test]
    async fn test_gk_birth_mode_derives_from_hd_engine() {
        let hd_engine = Arc::new(engine_human_design::HumanDesignEngine::new());
        let gk_engine = GeneKeysEngine::with_hd_engine(hd_engine.clone());

        let input = EngineInput {
            birth_data: Some(noesis_core::BirthData {
                name: Some("Canonical".to_string()),
                date: "1991-08-13".to_string(),
                time: Some("13:31".to_string()),
                latitude: 12.9340,
                longitude: 77.6214,
                timezone: "Asia/Kolkata".to_string(),
            }),
            current_time: Utc::now(),
            location: None,
            precision: Precision::Standard,
            options: HashMap::new(),
        };

        let hd_output = hd_engine
            .calculate(input.clone())
            .await
            .expect("HD calculation should succeed");
        let gk_output = gk_engine
            .calculate(input)
            .await
            .expect("GK calculation should succeed");

        let ps = hd_output.result["personality_activations"]["sun"]["gate"]
            .as_u64()
            .unwrap() as u8;
        let pe = hd_output.result["personality_activations"]["earth"]["gate"]
            .as_u64()
            .unwrap() as u8;
        let ds = hd_output.result["design_activations"]["sun"]["gate"]
            .as_u64()
            .unwrap() as u8;
        let de = hd_output.result["design_activations"]["earth"]["gate"]
            .as_u64()
            .unwrap() as u8;

        assert_eq!(
            gk_output.result["activation_sequence"]["lifes_work"][0]
                .as_u64()
                .unwrap(),
            ps as u64
        );
        assert_eq!(
            gk_output.result["activation_sequence"]["lifes_work"][1]
                .as_u64()
                .unwrap(),
            pe as u64
        );
        assert_eq!(
            gk_output.result["activation_sequence"]["evolution"][0]
                .as_u64()
                .unwrap(),
            ds as u64
        );
        assert_eq!(
            gk_output.result["activation_sequence"]["evolution"][1]
                .as_u64()
                .unwrap(),
            de as u64
        );

        // Canonical guard against historical 8/14 drift
        assert_eq!(ds, 23);
        assert_eq!(de, 43);
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
