//! ConsciousnessEngine implementation for VedicClock-TCM
//!
//! Provides the standard Noesis engine interface for temporal recommendations
//! based on TCM organ clock and Vedic time cycles.

use async_trait::async_trait;
use chrono::Utc;
use noesis_core::{
    CalculationMetadata, ConsciousnessEngine, EngineError, EngineInput, EngineOutput,
    ValidationResult,
};
use serde_json::{json, Value};
use std::time::Instant;

use crate::calculator::{get_current_organ, get_local_hour};
use crate::dosha::get_dosha_for_hour;
use crate::integration::{get_temporal_recommendation, synthesize_organ_dosha};
use crate::models::{Activity, UpcomingTransition, VedicClockResult};
use crate::recommendations::{get_optimal_timing, is_favorable_now};
use crate::witness::generate_witness_prompt;

/// VedicClock-TCM consciousness engine
pub struct VedicClockEngine {
    engine_id: String,
    engine_name: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct TimezoneResolution {
    offset_minutes: i32,
    source: &'static str,
}

impl VedicClockEngine {
    /// Create a new VedicClock engine instance
    pub fn new() -> Self {
        Self {
            engine_id: "vedic-clock".to_string(),
            engine_name: "Vedic Clock".to_string(),
        }
    }

    /// Parse timezone string into offset minutes.
    fn timezone_minutes_from_string(tz: &str) -> Option<i32> {
        if tz.starts_with('+') || tz.starts_with('-') {
            let parts: Vec<&str> = tz[1..].split(':').collect();
            let sign: i32 = if tz.starts_with('-') { -1 } else { 1 };
            let hours: i32 = parts.first()?.parse().ok()?;
            let minutes: i32 = parts.get(1).and_then(|m| m.parse().ok()).unwrap_or(0);
            return Some(sign * (hours * 60 + minutes));
        }

        match tz {
            "Asia/Kolkata" | "Asia/Calcutta" => Some(330),
            "UTC" | "GMT" => Some(0),
            _ => None,
        }
    }

    /// Resolve timezone offset in minutes from options or birth_data fallback.
    /// Priority:
    /// 1) options.timezone_offset (minutes)
    /// 2) birth_data.timezone (IANA or +HH:MM)
    /// 3) UTC (0)
    fn resolve_timezone(input: &EngineInput) -> TimezoneResolution {
        if let Some(offset) = input
            .options
            .get("timezone_offset")
            .and_then(|v| v.as_i64())
            .map(|v| v as i32)
        {
            return TimezoneResolution {
                offset_minutes: offset,
                source: "options.timezone_offset",
            };
        }

        if let Some(birth) = input.birth_data.as_ref() {
            if let Some(offset) = Self::timezone_minutes_from_string(&birth.timezone) {
                return TimezoneResolution {
                    offset_minutes: offset,
                    source: "birth_data.timezone",
                };
            }
        }

        TimezoneResolution {
            offset_minutes: 0,
            source: "default_utc",
        }
    }

    /// Extract optional activity from input options
    fn get_activity(options: &std::collections::HashMap<String, Value>) -> Option<Activity> {
        options
            .get("activity")
            .and_then(|v| v.as_str())
            .and_then(|s| match s.to_lowercase().as_str() {
                "meditation" => Some(Activity::Meditation),
                "exercise" => Some(Activity::Exercise),
                "work" => Some(Activity::Work),
                "eating" => Some(Activity::Eating),
                "sleep" => Some(Activity::Sleep),
                "creative" => Some(Activity::Creative),
                "social" => Some(Activity::Social),
                _ => None,
            })
    }

    /// Extract optional Panchanga indices from options
    fn get_panchanga_indices(
        options: &std::collections::HashMap<String, Value>,
    ) -> (Option<u8>, Option<u8>) {
        let tithi = options
            .get("tithi_index")
            .and_then(|v| v.as_u64())
            .map(|v| v as u8);
        let nakshatra = options
            .get("nakshatra_index")
            .and_then(|v| v.as_u64())
            .map(|v| v as u8);
        (tithi, nakshatra)
    }

    /// Build the result JSON
    fn build_result(
        &self,
        result: &VedicClockResult,
        activity: Option<Activity>,
        datetime: chrono::DateTime<Utc>,
        timezone: &TimezoneResolution,
    ) -> Value {
        let local_hour = get_local_hour(datetime, timezone.offset_minutes);
        let mut output = json!({
            "current_organ": {
                "organ": result.current_organ.organ,
                "element": result.current_organ.element,
                "time_window": result.current_organ.time_range_display(),
                "peak_energy": result.current_organ.peak_energy,
                "associated_emotion": result.current_organ.associated_emotion,
                "recommended_activities": result.current_organ.recommended_activities,
            },
            "current_dosha": {
                "dosha": result.current_dosha.dosha,
                "qualities": result.current_dosha.qualities,
            },
            "recommendation": {
                "time_window": result.recommendation.time_window,
                "organ": result.recommendation.organ,
                "dosha": result.recommendation.dosha,
                "activities": result.recommendation.activities,
                "panchanga_quality": result.recommendation.panchanga_quality,
            },
            "synthesis": synthesize_organ_dosha(datetime, timezone.offset_minutes),
            "calculated_for": result.calculated_for,
            "timezone": {
                "offset_minutes": timezone.offset_minutes,
                "source": timezone.source,
                "local_hour": local_hour,
            },
        });

        // Add activity-specific timing if requested
        if let Some(activity) = activity {
            let optimal_times = get_optimal_timing(activity, datetime, timezone.offset_minutes);
            let (is_favorable, reason) =
                is_favorable_now(activity, datetime, timezone.offset_minutes);

            output["activity_timing"] = json!({
                "activity": activity.display_name(),
                "is_favorable_now": is_favorable,
                "reason": reason,
                "optimal_windows": optimal_times.iter().map(|w| {
                    json!({
                        "time_window": w.time_range_display(),
                        "quality": w.quality,
                        "reason": w.reason,
                    })
                }).collect::<Vec<_>>(),
            });
        }

        // Add upcoming transitions if available
        if let Some(upcoming) = &result.upcoming {
            output["upcoming_transitions"] = json!(upcoming);
        }

        output
    }

    /// Generate upcoming transitions for the next few hours
    fn get_upcoming_transitions(
        datetime: chrono::DateTime<Utc>,
        timezone_offset: i32,
    ) -> Vec<UpcomingTransition> {
        let current_hour = get_local_hour(datetime, timezone_offset);
        let mut transitions = Vec::new();

        // Look ahead 6 hours for transitions
        for offset in 1..=6 {
            let future_hour = (current_hour as u32 + offset as u32) % 24;

            // Check if this is an organ transition (odd hours: 1, 3, 5, 7, 9, 11, 13, 15, 17, 19, 21, 23)
            if future_hour % 2 == 1 {
                let new_organ = crate::wisdom::get_organ_for_hour(future_hour as u8);
                transitions.push(UpcomingTransition {
                    time: format!("{}:00", future_hour),
                    description: format!("{} time begins", new_organ.organ.display_name()),
                    new_organ: Some(new_organ.organ),
                    new_dosha: None,
                });
            }

            // Check for dosha transitions (2, 6, 10, 14, 18, 22)
            if [2, 6, 10, 14, 18, 22].contains(&future_hour) {
                let new_dosha = get_dosha_for_hour(future_hour as u8);
                transitions.push(UpcomingTransition {
                    time: format!("{}:00", future_hour),
                    description: format!("{} period begins", new_dosha.dosha.display_name()),
                    new_organ: None,
                    new_dosha: Some(new_dosha.dosha),
                });
            }
        }

        transitions
    }
}

impl Default for VedicClockEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ConsciousnessEngine for VedicClockEngine {
    fn engine_id(&self) -> &str {
        &self.engine_id
    }

    fn engine_name(&self) -> &str {
        &self.engine_name
    }

    fn required_phase(&self) -> u8 {
        0 // Available at all consciousness phases
    }

    async fn calculate(&self, input: EngineInput) -> Result<EngineOutput, EngineError> {
        let start = Instant::now();

        // Get parameters
        let timezone = Self::resolve_timezone(&input);
        let activity = Self::get_activity(&input.options);
        let (tithi, nakshatra) = Self::get_panchanga_indices(&input.options);

        // Use current_time from input
        let datetime = input.current_time;

        // Calculate current organ and dosha
        let current_organ = get_current_organ(datetime, timezone.offset_minutes);
        let local_hour = get_local_hour(datetime, timezone.offset_minutes);
        let current_dosha = get_dosha_for_hour(local_hour);

        // Get temporal recommendation
        let recommendation =
            get_temporal_recommendation(datetime, timezone.offset_minutes, tithi, nakshatra);

        // Get upcoming transitions
        let upcoming = Some(Self::get_upcoming_transitions(datetime, timezone.offset_minutes));

        // Build the result
        let result = VedicClockResult {
            current_organ,
            current_dosha,
            recommendation,
            upcoming,
            calculated_for: datetime.to_rfc3339(),
        };

        // Generate witness prompt
        let consciousness_level = input
            .options
            .get("consciousness_level")
            .and_then(|v| v.as_u64())
            .map(|v| v as u8)
            .unwrap_or(2);

        let witness_prompt = generate_witness_prompt(
            &result.current_organ.organ,
            &result.current_dosha.dosha,
            consciousness_level,
        );

        // Ensure witness prompt is non-empty
        if witness_prompt.is_empty() {
            return Err(EngineError::CalculationError(
                "Witness prompt generation failed: empty result".to_string(),
            ));
        }

        let elapsed = start.elapsed();

        Ok(EngineOutput {
            engine_id: self.engine_id.clone(),
            result: self.build_result(&result, activity, datetime, &timezone),
            witness_prompt,
            consciousness_level,
            metadata: CalculationMetadata {
                calculation_time_ms: elapsed.as_secs_f64() * 1000.0,
                backend: "native".to_string(),
                precision_achieved: format!("{:?}", input.precision),
                cached: false,
                timestamp: Utc::now(),
                engine_version: env!("CARGO_PKG_VERSION").to_string(),
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
        if output.result.get("current_organ").is_none() {
            messages.push("Missing 'current_organ' field in result".to_string());
            valid = false;
        }

        if output.result.get("current_dosha").is_none() {
            messages.push("Missing 'current_dosha' field in result".to_string());
            valid = false;
        }

        if output.result.get("recommendation").is_none() {
            messages.push("Missing 'recommendation' field in result".to_string());
            valid = false;
        }

        // Check current_organ has required fields
        if let Some(organ) = output.result.get("current_organ") {
            for field in ["organ", "element", "time_window"] {
                if organ.get(field).is_none() {
                    messages.push(format!("Missing '{}' in current_organ", field));
                    valid = false;
                }
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
        let timezone = Self::resolve_timezone(input);
        let activity = Self::get_activity(&input.options);
        let (tithi, nakshatra) = Self::get_panchanga_indices(&input.options);

        // Cache key based on hour (organ windows are 2-hour), timezone, and optional parameters
        let local_hour = get_local_hour(input.current_time, timezone.offset_minutes);
        let hour_bucket = local_hour / 2; // Group by 2-hour windows

        format!(
            "vedic-clock:h{}:tz{}:a{:?}:t{:?}:n{:?}",
            hour_bucket, timezone.offset_minutes, activity, tithi, nakshatra
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use noesis_core::Precision;
    use std::collections::HashMap;

    fn create_test_input() -> EngineInput {
        let mut options = HashMap::new();
        options.insert("timezone_offset".to_string(), json!(0));

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
        let engine = VedicClockEngine::new();
        assert_eq!(engine.engine_id(), "vedic-clock");
        assert_eq!(engine.engine_name(), "Vedic Clock");
        assert_eq!(engine.required_phase(), 0);
    }

    #[tokio::test]
    async fn test_calculate_basic() {
        let engine = VedicClockEngine::new();
        let input = create_test_input();

        let result = engine.calculate(input).await;
        assert!(result.is_ok(), "Calculation should succeed");

        let output = result.unwrap();
        assert_eq!(output.engine_id, "vedic-clock");
        assert!(!output.witness_prompt.is_empty());
    }

    #[tokio::test]
    async fn test_calculate_with_activity() {
        let engine = VedicClockEngine::new();
        let mut options = HashMap::new();
        options.insert("timezone_offset".to_string(), json!(0));
        options.insert("activity".to_string(), json!("meditation"));

        let input = EngineInput {
            birth_data: None,
            current_time: Utc::now(),
            location: None,
            precision: Precision::Standard,
            options,
        };

        let result = engine.calculate(input).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.result.get("activity_timing").is_some());
    }

    #[tokio::test]
    async fn test_calculate_with_panchanga() {
        let engine = VedicClockEngine::new();
        let mut options = HashMap::new();
        options.insert("timezone_offset".to_string(), json!(0));
        options.insert("tithi_index".to_string(), json!(2)); // Tritiya
        options.insert("nakshatra_index".to_string(), json!(3));

        let input = EngineInput {
            birth_data: None,
            current_time: Utc::now(),
            location: None,
            precision: Precision::Standard,
            options,
        };

        let result = engine.calculate(input).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        let recommendation = output.result.get("recommendation").unwrap();
        assert!(recommendation.get("panchanga_quality").is_some());
    }

    #[tokio::test]
    async fn test_validate_valid_output() {
        let engine = VedicClockEngine::new();
        let input = create_test_input();

        let output = engine.calculate(input).await.unwrap();
        let validation = engine.validate(&output).await.unwrap();

        assert!(validation.valid);
        assert!((validation.confidence - 1.0).abs() < 0.001);
    }

    #[tokio::test]
    async fn test_validate_invalid_output() {
        let engine = VedicClockEngine::new();

        let output = EngineOutput {
            engine_id: "vedic-clock".to_string(),
            result: json!({}),              // Missing fields
            witness_prompt: "".to_string(), // Empty
            consciousness_level: 2,
            metadata: CalculationMetadata {
                calculation_time_ms: 1.0,
                backend: "test".to_string(),
                precision_achieved: "Standard".to_string(),
                cached: false,
                timestamp: Utc::now(),
                engine_version: String::new(),
            },
        };

        let validation = engine.validate(&output).await.unwrap();
        assert!(!validation.valid);
        assert!(!validation.messages.is_empty());
    }

    #[tokio::test]
    async fn test_cache_key_consistency() {
        let engine = VedicClockEngine::new();
        let input = create_test_input();

        let key1 = engine.cache_key(&input);
        let key2 = engine.cache_key(&input);

        assert_eq!(key1, key2, "Cache key should be deterministic");
    }

    #[test]
    fn test_resolve_timezone() {
        let mut input = EngineInput {
            birth_data: None,
            current_time: Utc::now(),
            location: None,
            precision: Precision::Standard,
            options: HashMap::new(),
        };

        assert_eq!(
            VedicClockEngine::resolve_timezone(&input),
            TimezoneResolution {
                offset_minutes: 0,
                source: "default_utc",
            }
        );

        input.options.insert("timezone_offset".to_string(), json!(330));
        assert_eq!(
            VedicClockEngine::resolve_timezone(&input),
            TimezoneResolution {
                offset_minutes: 330,
                source: "options.timezone_offset",
            }
        );

        input.options.clear();
        input.birth_data = Some(noesis_core::BirthData {
            name: Some("TZ Test".to_string()),
            date: "1991-08-13".to_string(),
            time: Some("13:31".to_string()),
            latitude: 12.9340,
            longitude: 77.6214,
            timezone: "Asia/Kolkata".to_string(),
        });
        assert_eq!(
            VedicClockEngine::resolve_timezone(&input),
            TimezoneResolution {
                offset_minutes: 330,
                source: "birth_data.timezone",
            }
        );
    }

    #[tokio::test]
    async fn test_fixed_timestamp_uses_birth_timezone_fallback_for_ist() {
        let engine = VedicClockEngine::new();
        let input = EngineInput {
            birth_data: Some(noesis_core::BirthData {
                name: Some("Canonical".to_string()),
                date: "1991-08-13".to_string(),
                time: Some("13:31".to_string()),
                latitude: 12.9340,
                longitude: 77.6214,
                timezone: "Asia/Kolkata".to_string(),
            }),
            current_time: Utc.with_ymd_and_hms(2026, 3, 8, 6, 43, 16).unwrap(),
            location: None,
            precision: Precision::Standard,
            options: HashMap::new(),
        };

        let output = engine.calculate(input).await.expect("vedic-clock should calculate");
        assert_eq!(output.result["timezone"]["offset_minutes"], 330);
        assert_eq!(output.result["timezone"]["source"], "birth_data.timezone");
        assert_eq!(output.result["timezone"]["local_hour"], 12);
        assert_eq!(output.result["current_organ"]["organ"], "Heart");
    }

    #[tokio::test]
    async fn test_fixed_timestamp_supports_explicit_non_utc_offset_strings() {
        let engine = VedicClockEngine::new();
        let input = EngineInput {
            birth_data: Some(noesis_core::BirthData {
                name: Some("Nepal Offset".to_string()),
                date: "1991-08-13".to_string(),
                time: Some("13:31".to_string()),
                latitude: 27.7172,
                longitude: 85.3240,
                timezone: "+05:45".to_string(),
            }),
            current_time: Utc.with_ymd_and_hms(2026, 3, 8, 6, 43, 16).unwrap(),
            location: None,
            precision: Precision::Standard,
            options: HashMap::new(),
        };

        let output = engine.calculate(input).await.expect("vedic-clock should calculate");
        assert_eq!(output.result["timezone"]["offset_minutes"], 345);
        assert_eq!(output.result["timezone"]["source"], "birth_data.timezone");
        assert_eq!(output.result["timezone"]["local_hour"], 12);
        assert_eq!(output.result["current_organ"]["organ"], "Heart");
    }

    #[test]
    fn test_get_activity() {
        let mut options = HashMap::new();
        assert!(VedicClockEngine::get_activity(&options).is_none());

        options.insert("activity".to_string(), json!("meditation"));
        assert_eq!(
            VedicClockEngine::get_activity(&options),
            Some(Activity::Meditation)
        );

        options.insert("activity".to_string(), json!("EXERCISE"));
        assert_eq!(
            VedicClockEngine::get_activity(&options),
            Some(Activity::Exercise)
        );
    }
}
