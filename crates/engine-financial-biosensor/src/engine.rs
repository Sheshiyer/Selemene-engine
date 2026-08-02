//! The Financial Biosensor engine.
//!
//! A decision-reflection surface composed over five registered engines and one
//! optional biometric sample. It does not decide, and it does not forecast.
//!
//! Composition note: the sources are called directly rather than through the
//! orchestrator, so their own phase gates are not consulted on this route.
//! This engine therefore declares a `required_phase` at or above the maximum
//! of its sources'. See `docs/contributing/engine-onboarding.md`.

use async_trait::async_trait;
use chrono::Utc;
use futures::future::join_all;
use noesis_core::{
    CalculationMetadata, ConsciousnessEngine, EngineError, EngineInput, EngineOutput,
    ValidationResult,
};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::Instant;

use crate::composite::{ba_factor, compose};
use crate::contributors::{
    active_planetary_weather, chronofield, energetic_authority, gift_shadow_spectrum,
    heart_rate_variability, three_wave_cycle,
};
use crate::landscape::{
    monthly_alignment_calendar, weekly_risk_landscape, DateSeries, FORECAST_DAYS,
};
use crate::models::{
    Contributor, ContributorId, ContributorProvenance, ContributorStatus, DailyDecisionIndex,
    Declaration, FinancialBiosensorResult, ProvenanceBlock, FORMULA_VERSION,
};
use crate::reflection::decision_ownership_reflection;
use crate::witness::generate_witness_prompt;

pub const ENGINE_ID: &str = "financial-biosensor";
pub const ENGINE_NAME: &str = "Financial Biosensor";

/// The engine must sit at or above the maximum `required_phase` of its
/// sources: human-design 1, gene-keys 2, vimshottari 2, transits 0,
/// biorhythm 0.
pub const REQUIRED_PHASE: u8 = 3;

/// The five registered engines this surface composes.
pub struct SourceEngines {
    pub human_design: Arc<dyn ConsciousnessEngine>,
    pub gene_keys: Arc<dyn ConsciousnessEngine>,
    pub vimshottari: Arc<dyn ConsciousnessEngine>,
    pub transits: Arc<dyn ConsciousnessEngine>,
    pub biorhythm: Arc<dyn ConsciousnessEngine>,
}

pub struct FinancialBiosensorEngine {
    sources: SourceEngines,
}

impl FinancialBiosensorEngine {
    pub fn with_sources(sources: SourceEngines) -> Self {
        Self { sources }
    }

    /// Run one source and reduce it to `(result, engine_version)`.
    ///
    /// A source that fails is an absence, not a failure of this engine: the
    /// composite renormalizes over what could be read and names what could not.
    async fn read(
        engine: &Arc<dyn ConsciousnessEngine>,
        input: EngineInput,
    ) -> Option<(Value, String)> {
        match engine.calculate(input).await {
            Ok(output) => Some((output.result, output.metadata.engine_version)),
            Err(_) => None,
        }
    }

    /// Lift the four Sun/Earth gates out of a human-design result so that
    /// gene-keys can skip its own ephemeris pass.
    fn hd_gates(human_design: &Value) -> Option<Value> {
        let gate = |group: &str, body: &str| {
            human_design
                .pointer(&format!("/{}_activations/{}/gate", group, body))
                .and_then(Value::as_u64)
        };
        Some(json!({
            "personality_sun": gate("personality", "sun")?,
            "personality_earth": gate("personality", "earth")?,
            "design_sun": gate("design", "sun")?,
            "design_earth": gate("design", "earth")?,
        }))
    }
}

#[async_trait]
impl ConsciousnessEngine for FinancialBiosensorEngine {
    fn engine_id(&self) -> &str {
        ENGINE_ID
    }

    fn engine_name(&self) -> &str {
        ENGINE_NAME
    }

    fn required_phase(&self) -> u8 {
        REQUIRED_PHASE
    }

    async fn calculate(&self, input: EngineInput) -> Result<EngineOutput, EngineError> {
        let start = Instant::now();
        let now = input.current_time;
        let today = now.date_naive();

        // The biometric sample is read first: a malformed one is an error, and
        // there is no point spending four ephemeris passes to reject it after.
        let hrv = heart_rate_variability(&input.options, now)?;

        // Human Design runs alone so its gates can be handed to gene-keys,
        // which then skips a second ephemeris pass.
        let human_design = Self::read(&self.sources.human_design, input.clone()).await;

        let mut gene_keys_input = input.clone();
        if let Some(gates) = human_design.as_ref().and_then(|(r, _)| Self::hd_gates(r)) {
            gene_keys_input
                .options
                .insert("hd_gates".to_string(), gates);
        }

        let mut biorhythm_input = input.clone();
        biorhythm_input
            .options
            .insert("forecast_days".to_string(), json!(FORECAST_DAYS));

        let (gene_keys, vimshottari, transits, biorhythm) = {
            let mut results = join_all(vec![
                Self::read(&self.sources.gene_keys, gene_keys_input),
                Self::read(&self.sources.vimshottari, input.clone()),
                Self::read(&self.sources.transits, input.clone()),
                Self::read(&self.sources.biorhythm, biorhythm_input),
            ])
            .await
            .into_iter();
            (
                results.next().flatten(),
                results.next().flatten(),
                results.next().flatten(),
                results.next().flatten(),
            )
        };

        let contributors: Vec<Contributor> = vec![
            hrv,
            chronofield(vimshottari.as_ref()),
            active_planetary_weather(transits.as_ref()),
            three_wave_cycle(biorhythm.as_ref()),
            energetic_authority(human_design.as_ref(), &input.options),
            gift_shadow_spectrum(gene_keys.as_ref(), &input.options),
        ];

        let composite = compose(&contributors)?;
        let hrv_present = contributors
            .iter()
            .any(|c| c.id == ContributorId::HeartRateVariability && c.status.is_present());

        let series = DateSeries::build(biorhythm.as_ref(), vimshottari.as_ref());
        let saturn_pressure = transits
            .as_ref()
            .and_then(|(r, _)| r.pointer("/sade_sati/is_active"))
            .and_then(Value::as_bool)
            .unwrap_or(false);
        let period_quality = transits
            .as_ref()
            .and_then(|(r, _)| r.get("period_quality"))
            .and_then(Value::as_str)
            .map(str::to_string);

        let provenance = ProvenanceBlock {
            formula_version: FORMULA_VERSION.to_string(),
            engine_version: env!("CARGO_PKG_VERSION").to_string(),
            computed_at: Utc::now(),
            contributors: contributors
                .iter()
                .map(|c| ContributorProvenance {
                    contributor: c.id.as_str().to_string(),
                    leg: c.id.leg().to_string(),
                    engine_id: c.engine_id.clone(),
                    engine_version: c.engine_version.clone(),
                    fields_consumed: c.fields_consumed.clone(),
                    normalization: c.normalization.clone(),
                    raw: c.raw.clone(),
                    normalized: c.normalized,
                    status: c.status.clone(),
                    weight_declared: c.id.declared_weight(),
                    weight_effective: composite
                        .effective_weights
                        .get(&c.id)
                        .copied()
                        .unwrap_or(0.0),
                })
                .collect(),
            coverage: composite.coverage,
            ba_factor: ba_factor(hrv_present),
        };

        let result = FinancialBiosensorResult {
            declaration: Declaration::default(),
            daily_decision_index: DailyDecisionIndex {
                date: today,
                value: composite.value,
                convergence: composite.convergence,
                sufficiency: composite.sufficiency,
                coverage: composite.coverage,
                contributions: composite.contributions.clone(),
            },
            weekly_risk_landscape: weekly_risk_landscape(
                &contributors,
                &series,
                today,
                saturn_pressure,
            )?,
            monthly_alignment_calendar: monthly_alignment_calendar(&contributors, &series, today)?,
            decision_ownership_reflection: decision_ownership_reflection(
                &contributors,
                &input.options,
                period_quality,
            ),
            provenance,
        };

        let witness_prompt = generate_witness_prompt(&composite, &contributors);

        let result_json = serde_json::to_value(&result).map_err(|e| {
            EngineError::InternalError(format!(
                "Failed to serialize FinancialBiosensorResult: {}",
                e
            ))
        })?;

        Ok(EngineOutput {
            engine_id: ENGINE_ID.to_string(),
            result: result_json,
            witness_prompt,
            consciousness_level: REQUIRED_PHASE,
            metadata: CalculationMetadata {
                calculation_time_ms: start.elapsed().as_secs_f64() * 1000.0,
                backend: "composed-native-rust".to_string(),
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

        if output.engine_id != ENGINE_ID {
            messages.push(format!(
                "engine_id mismatch: expected '{}', got '{}'",
                ENGINE_ID, output.engine_id
            ));
            valid = false;
        }

        let parsed: Result<FinancialBiosensorResult, _> =
            serde_json::from_value(output.result.clone());

        let confidence = match parsed {
            Ok(result) => {
                if result.declaration.formula_version != FORMULA_VERSION {
                    messages.push(format!(
                        "formula_version mismatch: payload says '{}', this build is '{}'",
                        result.declaration.formula_version, FORMULA_VERSION
                    ));
                    valid = false;
                }

                if result.provenance.contributors.len() != ContributorId::ALL.len() {
                    messages.push(format!(
                        "provenance lists {} contributors, expected {}",
                        result.provenance.contributors.len(),
                        ContributorId::ALL.len()
                    ));
                    valid = false;
                }

                // The three-lock gate, checked rather than asserted: every
                // present contributor must name its source, its version, the
                // fields it read, and the normalization it applied.
                for c in &result.provenance.contributors {
                    match &c.status {
                        ContributorStatus::Present => {
                            if c.engine_id.is_empty()
                                || c.engine_version.is_empty()
                                || c.fields_consumed.is_empty()
                                || c.normalization.is_empty()
                            {
                                messages.push(format!(
                                    "{}: provenance is incomplete for a present contributor",
                                    c.contributor
                                ));
                                valid = false;
                            }
                            match c.normalized {
                                Some(v) if (0.0..=1.0).contains(&v) => {}
                                Some(v) => {
                                    messages.push(format!(
                                        "{}: normalized value {} is outside [0, 1]",
                                        c.contributor, v
                                    ));
                                    valid = false;
                                }
                                None => {
                                    messages.push(format!(
                                        "{}: present contributor carries no value",
                                        c.contributor
                                    ));
                                    valid = false;
                                }
                            }
                        }
                        ContributorStatus::Absent { reason, .. } => {
                            messages.push(format!(
                                "{}: absent ({}) — the {} leg is not represented in this reading.",
                                c.contributor, reason, c.leg
                            ));
                            if c.weight_effective != 0.0 {
                                messages.push(format!(
                                    "{}: absent contributor carries non-zero weight",
                                    c.contributor
                                ));
                                valid = false;
                            }
                        }
                    }
                }

                if let Some(value) = result.daily_decision_index.value {
                    if !(0.0..=1.0).contains(&value) {
                        messages.push(format!("daily_decision_index {} is outside [0, 1]", value));
                        valid = false;
                    }
                }

                if valid {
                    messages.push(format!(
                        "Coverage {:.2} over {} contributors; the index is a declared house model, \
                         not a forecast.",
                        result.provenance.coverage,
                        result.provenance.contributors.len()
                    ));
                }

                (result.provenance.coverage * result.provenance.ba_factor).clamp(0.0, 1.0)
            }
            Err(e) => {
                messages.push(format!("Failed to parse FinancialBiosensorResult: {}", e));
                valid = false;
                0.0
            }
        };

        Ok(ValidationResult {
            valid,
            confidence: if valid { confidence } else { 0.0 },
            messages,
        })
    }

    fn cache_key(&self, input: &EngineInput) -> String {
        let mut hasher = Sha256::new();
        hasher.update(b"financial-biosensor:");
        hasher.update(FORMULA_VERSION.as_bytes());
        hasher.update(b"|");

        if let Some(ref birth) = input.birth_data {
            if let Some(ref name) = birth.name {
                hasher.update(name.as_bytes());
            }
            hasher.update(b"|");
            hasher.update(birth.date.as_bytes());
            hasher.update(b"|");
            if let Some(ref time) = birth.time {
                hasher.update(time.as_bytes());
            }
            hasher.update(b"|");
            hasher.update(birth.latitude.to_string().as_bytes());
            hasher.update(b"|");
            hasher.update(birth.longitude.to_string().as_bytes());
            hasher.update(b"|");
            hasher.update(birth.timezone.as_bytes());
        }

        hasher.update(b"|");
        // The index is daily, so the clock time within the day is not part of
        // the key.
        hasher.update(input.current_time.date_naive().to_string().as_bytes());
        hasher.update(b"|");
        hasher.update(format!("{:?}", input.precision).as_bytes());

        // `options` is a HashMap, whose iteration order is not stable. Hash a
        // sorted, explicitly named subset so the key is deterministic.
        let mut relevant: BTreeMap<&str, String> = BTreeMap::new();
        for key in [
            "deliberation_hours",
            "gene_keys_frequency",
            "decision_context",
        ] {
            if let Some(v) = input.options.get(key) {
                relevant.insert(key, v.to_string());
            }
        }
        if let Some(hrv) = input.options.get("hrv") {
            for key in ["rmssd_ms", "baseline_rmssd_ms"] {
                if let Some(v) = hrv.get(key) {
                    relevant.insert(key, v.to_string());
                }
            }
            // Truncated to the hour: two samples from the same hour are the
            // same reading for caching purposes.
            if let Some(captured) = hrv.get("captured_at").and_then(Value::as_str) {
                if let Ok(ts) = chrono::DateTime::parse_from_rfc3339(captured) {
                    relevant.insert("captured_at_hour", ts.format("%Y-%m-%dT%H").to_string());
                }
            }
        }
        for (key, value) in relevant {
            hasher.update(b"|");
            hasher.update(key.as_bytes());
            hasher.update(b"=");
            hasher.update(value.as_bytes());
        }

        format!("financial-biosensor:{:x}", hasher.finalize())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
