//! ConsciousnessEngine trait implementation for Human Design
//!
//! Integrates HD calculations with the Noesis platform architecture.

use async_trait::async_trait;
use chrono::{NaiveDate, NaiveTime, TimeZone, Utc};
use noesis_core::{
    ConsciousnessEngine, EngineError, EngineInput, EngineOutput, ValidationResult,
    CalculationMetadata,
};
use serde_json::json;
use std::time::Instant;

use crate::{
    generate_hd_chart, initialize_ephemeris, witness::generate_witness_prompt, HDChart,
};

/// Human Design consciousness engine implementing the universal trait
pub struct HumanDesignEngine {
    engine_id: String,
    engine_name: String,
}

impl HumanDesignEngine {
    /// Create a new Human Design engine instance
    pub fn new() -> Self {
        Self {
            engine_id: "human-design".to_string(),
            engine_name: "Human Design".to_string(),
        }
    }

    /// Convert EngineInput to HD calculation parameters
    fn extract_birth_params(input: &EngineInput) -> Result<(NaiveDate, NaiveTime, String, f64, f64), EngineError> {
        let birth_data = input
            .birth_data
            .as_ref()
            .ok_or_else(|| EngineError::ValidationError("birth_data required for Human Design".to_string()))?;

        // Parse date
        let date = NaiveDate::parse_from_str(&birth_data.date, "%Y-%m-%d")
            .map_err(|e| EngineError::ValidationError(format!("Invalid date format: {}", e)))?;

        // Parse time
        let time_str = birth_data
            .time
            .as_ref()
            .ok_or_else(|| EngineError::ValidationError("birth_time required for Human Design".to_string()))?;
        
        let time = NaiveTime::parse_from_str(time_str, "%H:%M")
            .or_else(|_| NaiveTime::parse_from_str(time_str, "%H:%M:%S"))
            .map_err(|e| EngineError::ValidationError(format!("Invalid time format: {}", e)))?;

        let timezone = birth_data.timezone.clone();
        let latitude = birth_data.latitude;
        let longitude = birth_data.longitude;

        Ok((date, time, timezone, latitude, longitude))
    }

    /// Serialize HDChart to JSON value
    fn serialize_chart(chart: &HDChart) -> serde_json::Value {
        // Extract defined centers as an array
        let defined_centers: Vec<String> = chart.centers
            .iter()
            .filter(|(_, state)| state.defined)
            .map(|(center, _)| format!("{:?}", center))
            .collect();
        
        // Format channels as "gate1-gate2" strings
        let active_channels: Vec<String> = chart.channels
            .iter()
            .map(|ch| format!("{}-{}", ch.gate1, ch.gate2))
            .collect();
        
        // Convert activations to a more readable format
        let personality_activations: serde_json::Map<String, serde_json::Value> = chart.personality_activations
            .iter()
            .map(|act| {
                let key = format!("{:?}", act.planet).to_lowercase();
                let value = json!({
                    "gate": act.gate,
                    "line": act.line,
                    "longitude": act.longitude
                });
                (key, value)
            })
            .collect();
        
        let design_activations: serde_json::Map<String, serde_json::Value> = chart.design_activations
            .iter()
            .map(|act| {
                let key = format!("{:?}", act.planet).to_lowercase();
                let value = json!({
                    "gate": act.gate,
                    "line": act.line,
                    "longitude": act.longitude
                });
                (key, value)
            })
            .collect();
        
        json!({
            "hd_type": format!("{:?}", chart.hd_type),
            "authority": format!("{:?}", chart.authority),
            "profile": format!("{}/{}", chart.profile.conscious_line, chart.profile.unconscious_line),
            "definition": format!("{:?}", chart.definition),
            "defined_centers": defined_centers,
            "active_channels": active_channels,
            "personality_activations": personality_activations,
            "design_activations": design_activations,
        })
    }
}

impl Default for HumanDesignEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ConsciousnessEngine for HumanDesignEngine {
    fn engine_id(&self) -> &str {
        &self.engine_id
    }

    fn engine_name(&self) -> &str {
        &self.engine_name
    }

    fn required_phase(&self) -> u8 {
        1 // Basic consciousness required for HD
    }

    async fn calculate(&self, input: EngineInput) -> Result<EngineOutput, EngineError> {
        let start = Instant::now();

        // Extract birth parameters from input
        let (date, time, timezone_str, latitude, longitude) = Self::extract_birth_params(&input)?;

        // Initialize ephemeris (idempotent operation)
        initialize_ephemeris("");

        // Parse timezone
        let tz: chrono_tz::Tz = timezone_str
            .parse()
            .map_err(|e| EngineError::ValidationError(format!("Invalid timezone: {}", e)))?;

        // Create naive datetime and convert to UTC
        let naive_dt = date.and_time(time);
        let local_dt = tz
            .from_local_datetime(&naive_dt)
            .single()
            .ok_or_else(|| EngineError::ValidationError("Ambiguous local time".to_string()))?;
        let utc_dt = local_dt.with_timezone(&Utc);

        // Generate HD chart
        let chart = generate_hd_chart(utc_dt, "")
            .map_err(|e| EngineError::CalculationError(format!("Chart generation failed: {}", e)))?;

        // Get consciousness level from input options or default to 1
        let consciousness_level = input
            .options
            .get("consciousness_level")
            .and_then(|v| v.as_u64())
            .map(|v| v as u8)
            .unwrap_or(1);

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
            result: Self::serialize_chart(&chart),
            witness_prompt,
            consciousness_level,
            metadata: CalculationMetadata {
                calculation_time_ms: elapsed.as_secs_f64() * 1000.0,
                backend: "swiss-ephemeris".to_string(),
                precision_achieved: format!("{:?}", input.precision),
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

        // Check result has expected fields
        if output.result.get("hd_type").is_none() {
            messages.push("Missing 'hd_type' field in result".to_string());
            valid = false;
        }

        if output.result.get("authority").is_none() {
            messages.push("Missing 'authority' field in result".to_string());
            valid = false;
        }

        if output.result.get("profile").is_none() {
            messages.push("Missing 'profile' field in result".to_string());
            valid = false;
        }

        // Check consciousness level is in valid range
        if output.consciousness_level > 5 {
            messages.push(format!("Invalid consciousness_level: {}", output.consciousness_level));
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
        // Generate deterministic cache key from birth data
        if let Some(birth_data) = &input.birth_data {
            format!(
                "hd:{}:{}:{:.4}:{:.4}",
                birth_data.date,
                birth_data.time.as_ref().unwrap_or(&"00:00".to_string()),
                birth_data.latitude,
                birth_data.longitude
            )
        } else {
            format!("hd:invalid:{}", chrono::Utc::now().timestamp())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use noesis_core::{BirthData, Precision};
    use std::collections::HashMap;

    fn create_test_input() -> EngineInput {
        EngineInput {
            birth_data: Some(BirthData {
                name: Some("Test".to_string()),
                date: "1987-01-01".to_string(),
                time: Some("12:00".to_string()),
                latitude: 51.5074,
                longitude: -0.1278,
                timezone: "Europe/London".to_string(),
            }),
            current_time: Utc::now(),
            location: None,
            precision: Precision::Standard,
            options: HashMap::new(),
        }
    }

    #[tokio::test]
    async fn test_engine_creation() {
        let engine = HumanDesignEngine::new();
        assert_eq!(engine.engine_id(), "human-design");
        assert_eq!(engine.engine_name(), "Human Design");
        assert_eq!(engine.required_phase(), 1);
    }

    #[tokio::test]
    async fn test_cache_key_generation() {
        let engine = HumanDesignEngine::new();
        let input = create_test_input();
        
        let key = engine.cache_key(&input);
        assert!(key.starts_with("hd:"));
        assert!(key.contains("1987-01-01"));
        assert!(key.contains("12:00"));
    }

    #[tokio::test]
    async fn test_extract_birth_params() {
        let input = create_test_input();
        let result = HumanDesignEngine::extract_birth_params(&input);
        assert!(result.is_ok());
        
        let (date, time, tz, lat, lon) = result.unwrap();
        assert_eq!(date.to_string(), "1987-01-01");
        assert_eq!(time.format("%H:%M").to_string(), "12:00");
        assert_eq!(tz, "Europe/London");
        assert_eq!(lat, 51.5074);
        assert_eq!(lon, -0.1278);
    }

    #[tokio::test]
    async fn test_missing_birth_data() {
        let engine = HumanDesignEngine::new();
        let mut input = create_test_input();
        input.birth_data = None;
        
        let result = engine.calculate(input).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_validation_checks_witness_prompt() {
        let engine = HumanDesignEngine::new();
        let mut output = EngineOutput {
            engine_id: "human-design".to_string(),
            result: json!({"hd_type": "Generator", "authority": "Sacral", "profile": "1/3"}),
            witness_prompt: "".to_string(), // Empty
            consciousness_level: 1,
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
}
