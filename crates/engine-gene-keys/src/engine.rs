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
    models::{ActivationSequence, ActivationSource, GeneKeyActivation, GeneKeysChart},
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
            .filter(|v| (1..=64).contains(v))
            .map(|v| v as u8)
            .ok_or_else(|| {
                EngineError::ValidationError(
                    "Missing or invalid 'personality_sun' in hd_gates".to_string(),
                )
            })?;

        let personality_earth = hd_gates
            .get("personality_earth")
            .and_then(|v| v.as_u64())
            .filter(|v| (1..=64).contains(v))
            .map(|v| v as u8)
            .ok_or_else(|| {
                EngineError::ValidationError(
                    "Missing or invalid 'personality_earth' in hd_gates".to_string(),
                )
            })?;

        let design_sun = hd_gates
            .get("design_sun")
            .and_then(|v| v.as_u64())
            .filter(|v| (1..=64).contains(v))
            .map(|v| v as u8)
            .ok_or_else(|| {
                EngineError::ValidationError(
                    "Missing or invalid 'design_sun' in hd_gates".to_string(),
                )
            })?;

        let design_earth = hd_gates
            .get("design_earth")
            .and_then(|v| v.as_u64())
            .filter(|v| (1..=64).contains(v))
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

    /// Preserve gate-only compatibility without pretending an HD line is known.
    /// Sentinel 0 is internal; serialized output uses null for unavailable lines.
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
                line: 0,
                source: ActivationSource::PersonalitySun,
                gene_key_data: get_gene_key(personality_sun).cloned(),
            },
            GeneKeyActivation {
                key_number: personality_earth,
                line: 0,
                source: ActivationSource::PersonalityEarth,
                gene_key_data: get_gene_key(personality_earth).cloned(),
            },
            GeneKeyActivation {
                key_number: design_sun,
                line: 0,
                source: ActivationSource::DesignSun,
                gene_key_data: get_gene_key(design_sun).cloned(),
            },
            GeneKeyActivation {
                key_number: design_earth,
                line: 0,
                source: ActivationSource::DesignEarth,
                gene_key_data: get_gene_key(design_earth).cloned(),
            },
        ];

        Ok(GeneKeysChart {
            activation_sequence,
            active_keys,
        })
    }

    /// Preserve both values from the actual public HD result shape.
    /// Missing/invalid birth lines fail rather than receiving a placeholder.
    fn create_chart_from_hd_result(result: &Value) -> Result<GeneKeysChart, EngineError> {
        let read_activation = |block_name: &str, planet: &str, source: ActivationSource| {
            let activation = result
                .get(block_name)
                .and_then(Value::as_object)
                .and_then(|block| block.get(planet))
                .ok_or_else(|| {
                    EngineError::CalculationError(format!(
                        "HD output missing {block_name}.{planet} activation"
                    ))
                })?;
            let read_integer = |field: &str, maximum: u64| {
                activation.get(field).and_then(Value::as_u64)
                    .filter(|value| (1..=maximum).contains(value))
                    .map(|value| value as u8)
                    .ok_or_else(|| EngineError::CalculationError(format!(
                        "Missing or invalid {field} in {block_name}.{planet} (must be integer 1-{maximum})"
                    )))
            };
            let gate = read_integer("gate", 64)?;
            let line = read_integer("line", 6)?;
            Ok::<_, EngineError>(GeneKeyActivation {
                key_number: gate,
                line,
                source,
                gene_key_data: get_gene_key(gate).cloned(),
            })
        };
        let active_keys = vec![
            read_activation(
                "personality_activations",
                "sun",
                ActivationSource::PersonalitySun,
            )?,
            read_activation(
                "personality_activations",
                "earth",
                ActivationSource::PersonalityEarth,
            )?,
            read_activation("design_activations", "sun", ActivationSource::DesignSun)?,
            read_activation("design_activations", "earth", ActivationSource::DesignEarth)?,
        ];
        let activation_sequence = ActivationSequence::from_activations(
            active_keys[0].key_number,
            active_keys[1].key_number,
            active_keys[2].key_number,
            active_keys[3].key_number,
        );
        Ok(GeneKeysChart {
            activation_sequence,
            active_keys,
        })
    }

    /// Serialize GeneKeysChart to JSON value.
    ///
    /// `consciousness_level` is threaded in because the frequency assessment
    /// derives `suggested_frequency` from it. Passing `None` there silently
    /// nulls that field for every active key, which is what this payload did
    /// until the level was wired through.
    fn serialize_chart(chart: &GeneKeysChart, consciousness_level: u8) -> Value {
        // Include available framework records; record presence is not completeness.
        let enriched_keys: Vec<Value> = chart
            .active_keys
            .iter()
            .map(|ak| {
                let mut key_data = json!({
                    "key_number": ak.key_number,
                    "line": (1..=6).contains(&ak.line).then_some(ak.line),
                    "source": format!("{:?}", ak.source),
                    "meaning_source_quality": if ak.gene_key_data.is_some() {
                        "unverified_incomplete"
                    } else {
                        "unavailable"
                    },
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
            "activation_sequence_semantics": "legacy_gate_pair_relationships_not_canonical_spheres",
            "activation_spheres": chart.activation_spheres(),
            "line_provenance": if chart.active_keys.iter().all(|key| (1..=6).contains(&key.line)) {
                "preserved_human_design_activations"
            } else {
                "unavailable_gate_only_input"
            },
            "active_keys": enriched_keys,
            "meaning_source_quality": {
                "status": "unverified_incomplete",
                "source": "bundled_gene_keys_framework_records",
                "note": "Bundled records include generic templates and unverified labels/descriptions; a complete authorized verified wisdom dataset has not been supplied."
            },
            "frequency_assessments": frequency_assessments,
            "frequency_assessment_context": {
                "basis": "configured_service_level",
                "measurement_status": "not_measured",
                "note": "Suggested frequency is a configured reflection lens, not a measured user consciousness or frequency. Framework records may be incomplete."
            },
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
            Self::create_chart_from_hd_result(&hd_output.result)?
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
                "gk:v2:birth:{}:{}:{}:{:.4}:{:.4}",
                birth_data.date,
                birth_data.time.as_ref().unwrap_or(&"00:00".to_string()),
                birth_data.timezone,
                birth_data.latitude,
                birth_data.longitude
            )
        } else if input.options.contains_key("hd_gates") {
            // Mode 2: hd_gates cache key
            if let Ok((ps, pe, ds, de)) = Self::extract_hd_gates_from_options(&input.options) {
                format!("gk:v2:gates:{}:{}:{}:{}", ps, pe, ds, de)
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
        assert!(key.starts_with("gk:v2:gates:"));
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

    // Synthetic adapter fixture, not an astronomical reference chart.
    fn distinct_line_hd_fixture() -> Value {
        json!({
            "personality_activations": {
                "sun": {"gate": 17, "line": 1, "longitude": 120.5},
                "earth": {"gate": 18, "line": 2, "longitude": 300.5}
            },
            "design_activations": {
                "sun": {"gate": 45, "line": 4, "longitude": 45.3},
                "earth": {"gate": 26, "line": 6, "longitude": 225.3}
            }
        })
    }

    #[test]
    fn test_hd_fixture_preserves_four_lines_and_canonical_sources() {
        let chart =
            GeneKeysEngine::create_chart_from_hd_result(&distinct_line_hd_fixture()).unwrap();
        let output = GeneKeysEngine::serialize_chart(&chart, 3);
        for (sphere, index, gate, line, source) in [
            ("lifes_work", 0, 17, 1, "PersonalitySun"),
            ("evolution", 1, 18, 2, "PersonalityEarth"),
            ("radiance", 2, 45, 4, "DesignSun"),
            ("purpose", 3, 26, 6, "DesignEarth"),
        ] {
            assert_eq!(output["active_keys"][index]["key_number"], gate);
            assert_eq!(output["active_keys"][index]["line"], line);
            assert_eq!(output["active_keys"][index]["source"], source);
            assert_eq!(
                output["activation_spheres"][sphere],
                json!({"key_number":gate,"line":line,"source":source})
            );
        }
        assert_eq!(
            output["activation_sequence"],
            json!({"lifes_work":[17,18],"evolution":[45,26],"radiance":[17,45],"purpose":[18,26]})
        );
        assert_eq!(
            output["activation_sequence_semantics"],
            "legacy_gate_pair_relationships_not_canonical_spheres"
        );
    }

    #[test]
    fn test_hd_birth_result_requires_valid_gate_line_and_source_blocks() {
        for value in [
            Value::Null,
            json!(0),
            json!(7),
            json!(3.5),
            json!(-1),
            json!("3"),
        ] {
            let mut fixture = distinct_line_hd_fixture();
            fixture["personality_activations"]["sun"]["line"] = value;
            assert!(GeneKeysEngine::create_chart_from_hd_result(&fixture).is_err());
        }
        for value in [
            json!(0),
            json!(65),
            json!(273),
            json!(17.5),
            json!(-1),
            json!("17"),
        ] {
            let mut fixture = distinct_line_hd_fixture();
            fixture["design_activations"]["earth"]["gate"] = value;
            assert!(GeneKeysEngine::create_chart_from_hd_result(&fixture).is_err());
        }
        let mut fixture = distinct_line_hd_fixture();
        fixture["design_activations"]["earth"]
            .as_object_mut()
            .unwrap()
            .remove("line");
        assert!(GeneKeysEngine::create_chart_from_hd_result(&fixture).is_err());
        for block in ["personality_activations", "design_activations"] {
            let mut fixture = distinct_line_hd_fixture();
            fixture.as_object_mut().unwrap().remove(block);
            assert!(GeneKeysEngine::create_chart_from_hd_result(&fixture).is_err());
        }
    }

    #[test]
    fn test_gate_options_reject_overflow_before_narrowing() {
        let mut input = create_test_input_with_gates();
        input.options.get_mut("hd_gates").unwrap()["personality_sun"] = json!(273);
        assert!(GeneKeysEngine::extract_hd_gates_from_options(&input.options).is_err());
    }

    #[test]
    fn test_birth_cache_uses_semantic_namespace_and_timezone() {
        let engine = GeneKeysEngine::new();
        let mut input = create_test_input_with_gates();
        input.birth_data = Some(noesis_core::BirthData {
            name: Some("Test Reader".to_string()),
            date: "1990-01-01".to_string(),
            time: Some("12:00".to_string()),
            latitude: 12.9716,
            longitude: 77.5946,
            timezone: "Asia/Kolkata".to_string(),
        });
        let india = engine.cache_key(&input);
        input.birth_data.as_mut().unwrap().timezone = "UTC".to_string();
        let utc = engine.cache_key(&input);
        assert!(india.starts_with("gk:v2:birth:"));
        assert!(india.contains("Asia/Kolkata"));
        assert_ne!(india, utc);
    }
}
