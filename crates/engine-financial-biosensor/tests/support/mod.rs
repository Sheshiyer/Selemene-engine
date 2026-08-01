//! Mock source engines carrying the exact payload shapes the real engines
//! emit, so this crate's composition logic can be exercised deterministically
//! without a Swiss Ephemeris pass or a live registry.
//!
//! The shapes here are transcribed from the source engines' own serializers:
//! `engine-human-design/src/engine.rs`, `engine-gene-keys/src/engine.rs`,
//! `engine-vimshottari/src/engine.rs`, `engine-transits/src/engine.rs`, and
//! `engine-biorhythm/src/calculator.rs`.

#![allow(dead_code)]

use async_trait::async_trait;
use chrono::{Duration, TimeZone, Utc};
use noesis_core::{
    BirthData, CalculationMetadata, ConsciousnessEngine, EngineError, EngineInput, EngineOutput,
    Precision, ValidationResult,
};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;

pub const MOCK_VERSION: &str = "3.3.1-mock";

pub struct MockEngine {
    id: String,
    payload: Option<Value>,
}

impl MockEngine {
    pub fn arc(id: &str, payload: Value) -> Arc<dyn ConsciousnessEngine> {
        Arc::new(Self {
            id: id.to_string(),
            payload: Some(payload),
        })
    }

    /// A source that fails. The composite should treat this as an absence.
    pub fn failing(id: &str) -> Arc<dyn ConsciousnessEngine> {
        Arc::new(Self {
            id: id.to_string(),
            payload: None,
        })
    }
}

#[async_trait]
impl ConsciousnessEngine for MockEngine {
    fn engine_id(&self) -> &str {
        &self.id
    }

    fn engine_name(&self) -> &str {
        &self.id
    }

    fn required_phase(&self) -> u8 {
        0
    }

    async fn calculate(&self, _input: EngineInput) -> Result<EngineOutput, EngineError> {
        let payload = self
            .payload
            .clone()
            .ok_or_else(|| EngineError::CalculationError(format!("{} is down", self.id)))?;

        Ok(EngineOutput {
            engine_id: self.id.clone(),
            result: payload,
            witness_prompt: "What do you notice?".to_string(),
            consciousness_level: 0,
            metadata: CalculationMetadata {
                calculation_time_ms: 0.0,
                backend: "mock".to_string(),
                precision_achieved: "exact".to_string(),
                cached: false,
                timestamp: Utc::now(),
                engine_version: MOCK_VERSION.to_string(),
            },
        })
    }

    async fn validate(&self, _output: &EngineOutput) -> Result<ValidationResult, EngineError> {
        Ok(ValidationResult {
            valid: true,
            confidence: 1.0,
            messages: vec![],
        })
    }

    fn cache_key(&self, _input: &EngineInput) -> String {
        format!("{}:mock", self.id)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

// ---------------------------------------------------------------------------
// Payloads
// ---------------------------------------------------------------------------

pub fn human_design_payload(authority: &str) -> Value {
    json!({
        "hd_type": "Generator",
        "authority": authority,
        "profile": "1/3",
        "personality_activations": {
            "sun": { "gate": 13, "line": 3, "longitude": 140.5 },
            "earth": { "gate": 7, "line": 3, "longitude": 320.5 },
        },
        "design_activations": {
            "sun": { "gate": 26, "line": 2, "longitude": 52.1 },
            "earth": { "gate": 45, "line": 2, "longitude": 232.1 },
        },
    })
}

/// Gene Keys as it actually ships: `suggested_frequency` is always null,
/// because `serialize_chart` calls `assess_frequencies(chart, None)`.
pub fn gene_keys_payload() -> Value {
    json!({
        "frequency_assessments": [
            { "key_number": 13, "suggested_frequency": Value::Null },
            { "key_number": 7, "suggested_frequency": Value::Null },
        ],
    })
}

pub fn vimshottari_payload(days_until: i64) -> Value {
    let date = Utc.with_ymd_and_hms(2026, 8, 1, 0, 0, 0).unwrap() + Duration::days(days_until);
    json!({
        "current_period": {
            "mahadasha": { "planet": "Venus", "years": 20.0 },
            "antardasha": { "planet": "Ketu", "years": 1.4 },
            "pratyantardasha": { "planet": "Sun", "days": 25.2 },
        },
        "upcoming_transitions": [{
            "type": "Pratyantardasha",
            "from_planet": "Sun",
            "to_planet": "Moon",
            "date": date.to_rfc3339(),
            "days_until": days_until,
        }],
    })
}

pub fn transits_payload(sade_sati: bool) -> Value {
    json!({
        "aspects": [
            { "transiting_planet": "Jupiter", "natal_planet": "Sun",
              "aspect_type": "Trine", "orb": 1.5, "is_applying": true, "nature": "Harmonious" },
            { "transiting_planet": "Saturn", "natal_planet": "Moon",
              "aspect_type": "Square", "orb": 3.0, "is_applying": false, "nature": "Challenging" },
            { "transiting_planet": "Mars", "natal_planet": "Mercury",
              "aspect_type": "Conjunction", "orb": 2.0, "is_applying": true, "nature": "Neutral" },
        ],
        "sade_sati": {
            "is_active": sade_sati,
            "phase": if sade_sati { json!("Peak (conjunct Moon)") } else { Value::Null },
            "saturn_sign": "Pisces",
            "moon_sign": "Pisces",
        },
        "period_quality": "Mixed",
        "retrograde_planets": ["Mercury"],
    })
}

/// A biorhythm payload whose forward series actually varies by date, so the
/// weekly and monthly surfaces have something real to read.
pub fn biorhythm_payload() -> Value {
    let start = chrono::NaiveDate::from_ymd_opt(2026, 8, 1).unwrap();
    let forecast: Vec<Value> = (1..=35_i64)
        .map(|offset| {
            let pct = 50.0 + 40.0 * ((offset as f64) * 0.3).sin();
            json!({
                "date": (start + Duration::days(offset)).format("%Y-%m-%d").to_string(),
                "days_alive": 12000 + offset,
                "physical": pct,
                "emotional": pct,
                "intellectual": pct,
                "intuitive": pct,
                "aesthetic": pct,
                "spiritual": pct,
                "overall_energy": pct,
            })
        })
        .collect();

    json!({
        "days_alive": 12000,
        "target_date": "2026-08-01",
        "physical": { "value": 0.2, "percentage": 60.0, "phase": "Rising",
                      "days_until_peak": 3, "days_until_critical": 8,
                      "is_critical": false, "cycle_day": 5 },
        "emotional": { "value": 0.0, "percentage": 50.0, "phase": "Critical",
                       "days_until_peak": 7, "days_until_critical": 0,
                       "is_critical": true, "cycle_day": 0 },
        "intellectual": { "value": 0.4, "percentage": 70.0, "phase": "Rising",
                          "days_until_peak": 2, "days_until_critical": 10,
                          "is_critical": false, "cycle_day": 6 },
        "intuitive": { "value": 0.1, "percentage": 55.0, "phase": "Rising",
                       "days_until_peak": 5, "days_until_critical": 9,
                       "is_critical": false, "cycle_day": 4 },
        "aesthetic": { "value": 0.1, "percentage": 55.0, "phase": "Rising",
                       "days_until_peak": 5, "days_until_critical": 9,
                       "is_critical": false, "cycle_day": 4 },
        "spiritual": { "value": 0.1, "percentage": 55.0, "phase": "Rising",
                       "days_until_peak": 5, "days_until_critical": 9,
                       "is_critical": false, "cycle_day": 4 },
        "mastery": 65.0,
        "passion": 55.0,
        "wisdom": 60.0,
        "critical_days": ["2026-08-04", "2026-08-19"],
        "overall_energy": 57.5,
        "forecast": forecast,
    })
}

// ---------------------------------------------------------------------------
// Assembly
// ---------------------------------------------------------------------------

pub fn sources() -> engine_financial_biosensor::SourceEngines {
    engine_financial_biosensor::SourceEngines {
        human_design: MockEngine::arc("human-design", human_design_payload("Emotional")),
        gene_keys: MockEngine::arc("gene-keys", gene_keys_payload()),
        vimshottari: MockEngine::arc("vimshottari", vimshottari_payload(45)),
        transits: MockEngine::arc("transits", transits_payload(false)),
        biorhythm: MockEngine::arc("biorhythm", biorhythm_payload()),
    }
}

/// The canonical fixture used across the workspace's conformance tests.
pub fn canonical_input(options: &[(&str, Value)]) -> EngineInput {
    let mut map: HashMap<String, Value> = HashMap::new();
    for (k, v) in options {
        map.insert((*k).to_string(), v.clone());
    }

    EngineInput {
        birth_data: Some(BirthData {
            name: Some("Shesh".to_string()),
            date: "1991-08-13".to_string(),
            time: Some("13:31".to_string()),
            latitude: 12.9340,
            longitude: 77.6214,
            timezone: "Asia/Kolkata".to_string(),
        }),
        current_time: Utc.with_ymd_and_hms(2026, 8, 1, 6, 30, 0).unwrap(),
        location: None,
        precision: Precision::Standard,
        options: map,
    }
}

/// A fresh biometric sample, captured shortly before the reading.
pub fn fresh_hrv(rmssd: f64, baseline: f64) -> Value {
    json!({
        "rmssd_ms": rmssd,
        "baseline_rmssd_ms": baseline,
        "captured_at": "2026-08-01T05:12:00Z",
        "source": "apple-health",
    })
}
