//! Extraction and normalization, one function per contributor.
//!
//! Each function reads a source engine's serialized result (or a
//! practitioner-declared option) and returns a [`Contributor`] that is either
//! `Present` with a value in `[0.0, 1.0]`, or `Absent` with a machine-stable
//! reason. Absence is an ordinary outcome here, not an error: five of the six
//! contributors can be missing and the engine still reports honestly on what
//! remains.

use chrono::{DateTime, Utc};
use noesis_core::EngineError;
use serde_json::{json, Value};
use std::collections::HashMap;

use crate::models::{Contributor, ContributorId, ContributorStatus};

/// Lower bound for a physiologically plausible RMSSD sample, in milliseconds.
pub const HRV_MIN_MS: f64 = 1.0;
/// Upper bound for a physiologically plausible RMSSD sample, in milliseconds.
pub const HRV_MAX_MS: f64 = 300.0;
/// A biometric sample older than this is not treated as a present-tense
/// reading.
pub const HRV_MAX_AGE_HOURS: i64 = 36;
/// Declared window, in days, over which distance from a Vimshottari period
/// boundary is scaled. One civil quarter.
pub const CHRONOFIELD_WINDOW_DAYS: f64 = 90.0;

fn clamp01(v: f64) -> f64 {
    v.clamp(0.0, 1.0)
}

// ---------------------------------------------------------------------------
// Three-Wave Cycle — biorhythm
// ---------------------------------------------------------------------------

/// Mean of the three named cycles, each already reported on a 0..100 scale.
///
/// The engine also emits `overall_energy`, but that is the mean of six cycles;
/// the specification names exactly three, so `overall_energy` is not used.
pub fn three_wave_cycle(biorhythm: Option<&(Value, String)>) -> Contributor {
    let id = ContributorId::ThreeWaveCycle;
    let Some((result, version)) = biorhythm else {
        return Contributor::absent(
            id,
            "biorhythm",
            "source_unavailable",
            "the biorhythm engine returned no result",
        );
    };

    let read = |cycle: &str| {
        result
            .pointer(&format!("/{}/percentage", cycle))
            .and_then(Value::as_f64)
    };
    let (Some(physical), Some(emotional), Some(intellectual)) =
        (read("physical"), read("emotional"), read("intellectual"))
    else {
        return Contributor::absent(
            id,
            "biorhythm",
            "source_field_missing",
            "one of physical/emotional/intellectual percentage was absent",
        );
    };

    let normalized = clamp01((physical + emotional + intellectual) / 300.0);

    Contributor {
        id,
        engine_id: "biorhythm".to_string(),
        engine_version: version.clone(),
        fields_consumed: vec![
            "/physical/percentage".to_string(),
            "/emotional/percentage".to_string(),
            "/intellectual/percentage".to_string(),
        ],
        normalization: "(physical + emotional + intellectual) / 300.0, each already 0..100"
            .to_string(),
        raw: json!({
            "physical": physical,
            "emotional": emotional,
            "intellectual": intellectual,
        }),
        normalized: Some(normalized),
        status: ContributorStatus::Present,
        observation: format!(
            "The three cycles read {:.0}, {:.0} and {:.0} out of 100.",
            physical, emotional, intellectual
        ),
    }
}

/// The same normalization applied to one row of the biorhythm forecast series.
pub fn three_wave_from_forecast(day: &Value) -> Option<f64> {
    let physical = day.get("physical")?.as_f64()?;
    let emotional = day.get("emotional")?.as_f64()?;
    let intellectual = day.get("intellectual")?.as_f64()?;
    Some(clamp01((physical + emotional + intellectual) / 300.0))
}

// ---------------------------------------------------------------------------
// Chronofield — vimshottari
// ---------------------------------------------------------------------------

/// Distance from the nearest upcoming period boundary, scaled over a declared
/// 90-day window. Nearer a boundary reads lower.
pub fn chronofield(vimshottari: Option<&(Value, String)>) -> Contributor {
    let id = ContributorId::Chronofield;
    let Some((result, version)) = vimshottari else {
        return Contributor::absent(
            id,
            "vimshottari",
            "source_unavailable",
            "the vimshottari engine returned no result",
        );
    };

    if result
        .get("current_period")
        .map(Value::is_null)
        .unwrap_or(true)
    {
        return Contributor::absent(
            id,
            "vimshottari",
            "no_current_period",
            "vimshottari reported no active period for this moment",
        );
    }

    let days = min_days_until(result);
    let Some(days) = days else {
        return Contributor::absent(
            id,
            "vimshottari",
            "no_upcoming_transitions",
            "vimshottari emitted no upcoming transitions to measure against",
        );
    };

    let normalized = clamp01(days as f64 / CHRONOFIELD_WINDOW_DAYS);

    Contributor {
        id,
        engine_id: "vimshottari".to_string(),
        engine_version: version.clone(),
        fields_consumed: vec![
            "/current_period".to_string(),
            "/upcoming_transitions/*/days_until".to_string(),
        ],
        normalization: format!("clamp(min(days_until) / {}, 0, 1)", CHRONOFIELD_WINDOW_DAYS),
        raw: json!({ "min_days_until": days }),
        normalized: Some(normalized),
        status: ContributorStatus::Present,
        observation: format!("The nearest period boundary is {} days away.", days),
    }
}

/// Smallest non-negative `days_until` across the emitted transitions.
pub fn min_days_until(vimshottari_result: &Value) -> Option<i64> {
    vimshottari_result
        .get("upcoming_transitions")?
        .as_array()?
        .iter()
        .filter_map(|t| t.get("days_until").and_then(Value::as_i64))
        .filter(|d| *d >= 0)
        .min()
}

/// The chronofield sub-score for a day `offset` days after the reading.
pub fn chronofield_at_offset(min_days: i64, offset: i64) -> f64 {
    clamp01((min_days - offset).max(0) as f64 / CHRONOFIELD_WINDOW_DAYS)
}

// ---------------------------------------------------------------------------
// Active Planetary Weather — transits
// ---------------------------------------------------------------------------

/// Default orb allowance per aspect type, mirroring
/// `engine_transits::models::AspectType::default_orb`.
fn default_orb(aspect_type: &str) -> Option<f64> {
    match aspect_type {
        "Conjunction" | "Opposition" => Some(8.0),
        "Trine" | "Square" => Some(6.0),
        "Sextile" => Some(4.0),
        _ => None,
    }
}

/// Orb-weighted balance of harmonious against challenging aspects.
///
/// Neutral aspects are counted in the denominator but contribute no polarity,
/// so a chart of tight conjunctions reads mid-scale rather than being ignored.
/// `period_quality` and `sade_sati` are deliberately not folded in: both are
/// computed from this same aspect set, and including them would count it twice.
pub fn active_planetary_weather(transits: Option<&(Value, String)>) -> Contributor {
    let id = ContributorId::ActivePlanetaryWeather;
    let Some((result, version)) = transits else {
        return Contributor::absent(
            id,
            "transits",
            "source_unavailable",
            "the transits engine returned no result",
        );
    };

    let aspects = result.get("aspects").and_then(Value::as_array);
    let Some(aspects) = aspects else {
        return Contributor::absent(
            id,
            "transits",
            "source_field_missing",
            "the transits result carried no aspects array",
        );
    };

    let mut raw_sum = 0.0_f64;
    let mut norm = 0.0_f64;
    let mut counted = 0_usize;

    for aspect in aspects {
        let (Some(kind), Some(orb), Some(nature)) = (
            aspect.get("aspect_type").and_then(Value::as_str),
            aspect.get("orb").and_then(Value::as_f64),
            aspect.get("nature").and_then(Value::as_str),
        ) else {
            continue;
        };
        let Some(max_orb) = default_orb(kind) else {
            continue;
        };
        let tightness = clamp01(1.0 - orb / max_orb);
        let polarity = match nature {
            "Harmonious" => 1.0,
            "Challenging" => -1.0,
            _ => 0.0,
        };
        raw_sum += polarity * tightness;
        norm += tightness;
        counted += 1;
    }

    if norm <= 0.0 {
        return Contributor::absent(
            id,
            "transits",
            "no_aspects_in_orb",
            "no transiting aspect fell within orb of a natal position",
        );
    }

    let normalized = clamp01((raw_sum / norm + 1.0) / 2.0);

    Contributor {
        id,
        engine_id: "transits".to_string(),
        engine_version: version.clone(),
        fields_consumed: vec![
            "/aspects/*/aspect_type".to_string(),
            "/aspects/*/orb".to_string(),
            "/aspects/*/nature".to_string(),
        ],
        normalization: "tightness = clamp(1 - orb / default_orb(type)); polarity = +1 harmonious, \
             -1 challenging, 0 neutral; score = (sum(polarity*tightness) / sum(tightness) + 1) / 2"
            .to_string(),
        raw: json!({
            "aspects_counted": counted,
            "polarity_sum": raw_sum,
            "tightness_sum": norm,
        }),
        normalized: Some(normalized),
        status: ContributorStatus::Present,
        observation: format!(
            "{} aspects were within orb, balancing to {:.2} on a harmonious-to-challenging scale.",
            counted, normalized
        ),
    }
}

// ---------------------------------------------------------------------------
// Energetic Authority — human-design plus declared deliberation
// ---------------------------------------------------------------------------

/// Declared latency each authority asks for before a decision settles, in hours.
///
/// This is not a ranking of authority types. The sub-score measures how much of
/// the waiting a practitioner's own authority asks for has already elapsed —
/// the La leg, which only exists because the practitioner supplied it.
pub fn required_hours(authority: &str) -> Option<f64> {
    match authority {
        "Sacral" | "Splenic" | "Heart" => Some(0.0),
        "GCenter" => Some(24.0),
        "Mental" => Some(72.0),
        // One emotional wave.
        "Emotional" => Some(72.0),
        // One lunar cycle, 28 days.
        "Lunar" => Some(672.0),
        _ => None,
    }
}

pub fn energetic_authority(
    human_design: Option<&(Value, String)>,
    options: &HashMap<String, Value>,
) -> Contributor {
    let id = ContributorId::EnergeticAuthority;
    let Some((result, version)) = human_design else {
        return Contributor::absent(
            id,
            "human-design",
            "source_unavailable",
            "the human-design engine returned no result",
        );
    };

    let Some(authority) = result.get("authority").and_then(Value::as_str) else {
        return Contributor::absent(
            id,
            "human-design",
            "source_field_missing",
            "the human-design result carried no authority field",
        );
    };

    let Some(required) = required_hours(authority) else {
        return Contributor::absent(
            id,
            "human-design",
            "unknown_authority",
            format!("no declared latency for authority '{}'", authority),
        );
    };

    let declared = options
        .get("deliberation_hours")
        .and_then(Value::as_f64)
        .filter(|h| *h >= 0.0);

    let normalized = if required == 0.0 {
        // A zero-latency authority has nothing to wait out.
        1.0
    } else {
        match declared {
            Some(hours) => clamp01(hours / required),
            None => {
                return Contributor::absent(
                    id,
                    "human-design",
                    "no_deliberation_declared",
                    format!(
                        "authority '{}' asks for {} hours; no deliberation_hours was declared",
                        authority, required
                    ),
                );
            }
        }
    };

    Contributor {
        id,
        engine_id: "human-design".to_string(),
        engine_version: version.clone(),
        fields_consumed: vec![
            "/authority".to_string(),
            "options.deliberation_hours".to_string(),
        ],
        normalization:
            "clamp(deliberation_hours / required_hours(authority), 0, 1); 1.0 when the authority \
             asks for no latency"
                .to_string(),
        raw: json!({
            "authority": authority,
            "required_hours": required,
            "deliberation_hours": declared,
        }),
        normalized: Some(normalized),
        status: ContributorStatus::Present,
        observation: if required == 0.0 {
            format!("{} authority asks for no waiting.", authority)
        } else {
            format!(
                "{} authority asks for {:.0} hours; {:.0} have been declared.",
                authority,
                required,
                declared.unwrap_or(0.0)
            )
        },
    }
}

// ---------------------------------------------------------------------------
// Gift-Shadow Spectrum — gene-keys, or declared
// ---------------------------------------------------------------------------

fn frequency_score(label: &str) -> Option<f64> {
    match label.to_ascii_lowercase().as_str() {
        "shadow" => Some(0.0),
        "gift" => Some(0.5),
        "siddhi" => Some(1.0),
        _ => None,
    }
}

/// Read the spectrum from gene-keys if it carries one, otherwise from a
/// declared option.
///
/// The gene-keys engine currently calls `assess_frequencies(chart, None)`, and
/// the field is computed as `consciousness_level.and_then(..)`, so
/// `suggested_frequency` is always null in the shipped payload. This
/// contributor is therefore absent by default, and says so by name rather than
/// substituting a value.
pub fn gift_shadow_spectrum(
    gene_keys: Option<&(Value, String)>,
    options: &HashMap<String, Value>,
) -> Contributor {
    let id = ContributorId::GiftShadowSpectrum;

    if let Some(label) = options.get("gene_keys_frequency").and_then(Value::as_str) {
        if let Some(score) = frequency_score(label) {
            return Contributor {
                id,
                engine_id: "practitioner-declared".to_string(),
                engine_version: "declared".to_string(),
                fields_consumed: vec!["options.gene_keys_frequency".to_string()],
                normalization: "shadow = 0.0, gift = 0.5, siddhi = 1.0".to_string(),
                raw: json!({ "declared_frequency": label }),
                normalized: Some(score),
                status: ContributorStatus::Present,
                observation: format!("The spectrum was declared as {}.", label),
            };
        }
        return Contributor::absent(
            id,
            "practitioner-declared",
            "unrecognized_frequency",
            format!("'{}' is not one of shadow, gift, siddhi", label),
        );
    }

    let Some((result, version)) = gene_keys else {
        return Contributor::absent(
            id,
            "gene-keys",
            "source_unavailable",
            "the gene-keys engine returned no result and no frequency was declared",
        );
    };

    let suggested = result
        .get("frequency_assessments")
        .and_then(Value::as_array)
        .and_then(|rows| {
            rows.iter()
                .filter_map(|r| r.get("suggested_frequency").and_then(Value::as_str))
                .next()
                .map(str::to_string)
        });

    let Some(label) = suggested else {
        return Contributor::absent(
            id,
            "gene-keys",
            "source_field_null",
            "gene-keys serializes frequency assessments with a null consciousness level, so \
             suggested_frequency is null; declare options.gene_keys_frequency to supply it",
        );
    };

    let Some(score) = frequency_score(&label) else {
        return Contributor::absent(
            id,
            "gene-keys",
            "unrecognized_frequency",
            format!("'{}' is not one of shadow, gift, siddhi", label),
        );
    };

    Contributor {
        id,
        engine_id: "gene-keys".to_string(),
        engine_version: version.clone(),
        fields_consumed: vec!["/frequency_assessments/*/suggested_frequency".to_string()],
        normalization: "shadow = 0.0, gift = 0.5, siddhi = 1.0".to_string(),
        raw: json!({ "suggested_frequency": label }),
        normalized: Some(score),
        status: ContributorStatus::Present,
        observation: format!("Gene Keys reports the spectrum at {}.", label),
    }
}

// ---------------------------------------------------------------------------
// Heart rate variability — declared only
// ---------------------------------------------------------------------------

/// Read a biometric sample from `options.hrv`.
///
/// There is no ingestion layer and no stored biometric record. The sample is
/// supplied per call, range-gated, and never persisted. A malformed sample is
/// an error; a missing sample is an ordinary absence.
///
/// Normalization is log-symmetric about the practitioner's own baseline,
/// because a doubling and a halving of RMSSD are the same size of departure.
/// A population norm is not used: that would be an empirical claim this engine
/// cannot source.
pub fn heart_rate_variability(
    options: &HashMap<String, Value>,
    now: DateTime<Utc>,
) -> Result<Contributor, EngineError> {
    let id = ContributorId::HeartRateVariability;
    let Some(hrv) = options.get("hrv") else {
        return Ok(Contributor::absent(
            id,
            "practitioner-declared",
            "no_sample",
            "no biometric sample was supplied for this reading",
        ));
    };

    let Some(rmssd) = hrv.get("rmssd_ms").and_then(Value::as_f64) else {
        return Err(EngineError::ValidationError(
            "options.hrv is present but carries no numeric rmssd_ms".to_string(),
        ));
    };
    if !(HRV_MIN_MS..=HRV_MAX_MS).contains(&rmssd) {
        return Err(EngineError::ValidationError(format!(
            "options.hrv.rmssd_ms is {} ms, outside the plausible range {}..={} ms",
            rmssd, HRV_MIN_MS, HRV_MAX_MS
        )));
    }

    let Some(baseline) = hrv.get("baseline_rmssd_ms").and_then(Value::as_f64) else {
        return Ok(Contributor::absent(
            id,
            "practitioner-declared",
            "no_personal_baseline",
            "a sample was supplied without a personal baseline_rmssd_ms to read it against",
        ));
    };
    if !(HRV_MIN_MS..=HRV_MAX_MS).contains(&baseline) {
        return Err(EngineError::ValidationError(format!(
            "options.hrv.baseline_rmssd_ms is {} ms, outside the plausible range {}..={} ms",
            baseline, HRV_MIN_MS, HRV_MAX_MS
        )));
    }

    let source = hrv
        .get("source")
        .and_then(Value::as_str)
        .unwrap_or("declared")
        .to_string();

    if let Some(captured) = hrv.get("captured_at").and_then(Value::as_str) {
        match DateTime::parse_from_rfc3339(captured) {
            Ok(ts) => {
                let age_hours = (now - ts.with_timezone(&Utc)).num_hours();
                if age_hours > HRV_MAX_AGE_HOURS {
                    return Ok(Contributor::absent(
                        id,
                        "practitioner-declared",
                        "sample_stale",
                        format!(
                            "the sample was captured {} hours ago, beyond the {} hour window",
                            age_hours, HRV_MAX_AGE_HOURS
                        ),
                    ));
                }
            }
            Err(e) => {
                return Err(EngineError::ValidationError(format!(
                    "options.hrv.captured_at is not a valid RFC 3339 timestamp: {}",
                    e
                )));
            }
        }
    }

    let ratio = rmssd / baseline;
    let normalized = clamp01(0.5 + 0.5 * ratio.ln() / std::f64::consts::LN_2);

    Ok(Contributor {
        id,
        engine_id: "practitioner-declared".to_string(),
        engine_version: source.clone(),
        fields_consumed: vec![
            "options.hrv.rmssd_ms".to_string(),
            "options.hrv.baseline_rmssd_ms".to_string(),
            "options.hrv.captured_at".to_string(),
        ],
        normalization:
            "clamp(0.5 + 0.5 * ln(rmssd_ms / baseline_rmssd_ms) / ln 2, 0, 1); log-symmetric \
             about the practitioner's own baseline"
                .to_string(),
        raw: json!({
            "rmssd_ms": rmssd,
            "baseline_rmssd_ms": baseline,
            "ratio": ratio,
            "source": source,
        }),
        normalized: Some(normalized),
        status: ContributorStatus::Present,
        observation: format!(
            "The sample reads {:.1} ms against a baseline of {:.1} ms.",
            rmssd, baseline
        ),
    })
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn now() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 8, 1, 12, 0, 0).unwrap()
    }

    fn opts(pairs: &[(&str, Value)]) -> HashMap<String, Value> {
        pairs
            .iter()
            .map(|(k, v)| ((*k).to_string(), v.clone()))
            .collect()
    }

    fn bio(p: f64, e: f64, i: f64) -> (Value, String) {
        (
            json!({
                "physical": { "percentage": p },
                "emotional": { "percentage": e },
                "intellectual": { "percentage": i },
            }),
            "test-fixture".to_string(),
        )
    }

    // -- three wave ------------------------------------------------------

    #[test]
    fn three_wave_peaks_at_one() {
        let c = three_wave_cycle(Some(&bio(100.0, 100.0, 100.0)));
        assert_eq!(c.normalized, Some(1.0));
    }

    #[test]
    fn three_wave_bottoms_at_zero() {
        let c = three_wave_cycle(Some(&bio(0.0, 0.0, 0.0)));
        assert_eq!(c.normalized, Some(0.0));
    }

    #[test]
    fn three_wave_midpoint_is_half() {
        let c = three_wave_cycle(Some(&bio(50.0, 50.0, 50.0)));
        assert_eq!(c.normalized, Some(0.5));
    }

    #[test]
    fn three_wave_absent_without_source() {
        let c = three_wave_cycle(None);
        assert!(!c.status.is_present());
        assert!(c.normalized.is_none());
    }

    // -- chronofield -----------------------------------------------------

    fn vim(days: Option<i64>, has_period: bool) -> (Value, String) {
        let transitions = match days {
            Some(d) => json!([{ "type": "Antardasha", "from_planet": "Ketu",
                                "to_planet": "Venus", "date": "2026-09-01T00:00:00Z",
                                "days_until": d }]),
            None => json!([]),
        };
        (
            json!({
                "current_period": if has_period { json!({ "mahadasha": {} }) } else { Value::Null },
                "upcoming_transitions": transitions,
            }),
            "test-fixture".to_string(),
        )
    }

    #[test]
    fn chronofield_scales_over_the_declared_window() {
        assert_eq!(chronofield(Some(&vim(Some(0), true))).normalized, Some(0.0));
        assert_eq!(
            chronofield(Some(&vim(Some(45), true))).normalized,
            Some(0.5)
        );
        assert_eq!(
            chronofield(Some(&vim(Some(90), true))).normalized,
            Some(1.0)
        );
        assert_eq!(
            chronofield(Some(&vim(Some(400), true))).normalized,
            Some(1.0)
        );
    }

    #[test]
    fn chronofield_absent_without_current_period() {
        let c = chronofield(Some(&vim(Some(30), false)));
        assert!(matches!(
            c.status,
            ContributorStatus::Absent { ref reason, .. } if reason == "no_current_period"
        ));
    }

    #[test]
    fn chronofield_absent_without_transitions() {
        let c = chronofield(Some(&vim(None, true)));
        assert!(matches!(
            c.status,
            ContributorStatus::Absent { ref reason, .. } if reason == "no_upcoming_transitions"
        ));
    }

    #[test]
    fn chronofield_offset_decrements_and_floors_at_zero() {
        assert_eq!(chronofield_at_offset(90, 0), 1.0);
        assert_eq!(chronofield_at_offset(90, 45), 0.5);
        assert_eq!(chronofield_at_offset(3, 10), 0.0);
    }

    // -- planetary weather -----------------------------------------------

    fn aspect(kind: &str, orb: f64, nature: &str) -> Value {
        json!({ "aspect_type": kind, "orb": orb, "nature": nature })
    }

    fn transits(aspects: Value) -> (Value, String) {
        (json!({ "aspects": aspects }), "test-fixture".to_string())
    }

    #[test]
    fn weather_all_harmonious_reads_one() {
        let c =
            active_planetary_weather(Some(&transits(json!([aspect("Trine", 0.0, "Harmonious")]))));
        assert_eq!(c.normalized, Some(1.0));
    }

    #[test]
    fn weather_all_challenging_reads_zero() {
        let c = active_planetary_weather(Some(&transits(json!([aspect(
            "Square",
            0.0,
            "Challenging"
        )]))));
        assert_eq!(c.normalized, Some(0.0));
    }

    #[test]
    fn weather_balanced_reads_midpoint() {
        let c = active_planetary_weather(Some(&transits(json!([
            aspect("Trine", 0.0, "Harmonious"),
            aspect("Square", 0.0, "Challenging"),
        ]))));
        let v = c.normalized.unwrap();
        assert!((v - 0.5).abs() < 1e-9, "got {}", v);
    }

    #[test]
    fn weather_neutral_only_reads_midpoint_rather_than_absent() {
        let c = active_planetary_weather(Some(&transits(json!([aspect(
            "Conjunction",
            0.0,
            "Neutral"
        )]))));
        assert_eq!(c.normalized, Some(0.5));
    }

    #[test]
    fn weather_absent_when_nothing_is_in_orb() {
        let c = active_planetary_weather(Some(&transits(json!([]))));
        assert!(matches!(
            c.status,
            ContributorStatus::Absent { ref reason, .. } if reason == "no_aspects_in_orb"
        ));
    }

    // -- energetic authority ---------------------------------------------

    fn hd(authority: &str) -> (Value, String) {
        (
            json!({ "authority": authority }),
            "test-fixture".to_string(),
        )
    }

    #[test]
    fn zero_latency_authority_is_present_without_a_declaration() {
        let c = energetic_authority(Some(&hd("Sacral")), &HashMap::new());
        assert_eq!(c.normalized, Some(1.0));
    }

    #[test]
    fn waiting_authority_scales_with_declared_hours() {
        let c = energetic_authority(
            Some(&hd("Emotional")),
            &opts(&[("deliberation_hours", json!(36.0))]),
        );
        assert_eq!(c.normalized, Some(0.5));

        let c = energetic_authority(
            Some(&hd("Lunar")),
            &opts(&[("deliberation_hours", json!(672.0))]),
        );
        assert_eq!(c.normalized, Some(1.0));
    }

    #[test]
    fn waiting_authority_absent_without_a_declaration() {
        let c = energetic_authority(Some(&hd("Emotional")), &HashMap::new());
        assert!(matches!(
            c.status,
            ContributorStatus::Absent { ref reason, .. } if reason == "no_deliberation_declared"
        ));
    }

    // -- gift shadow spectrum --------------------------------------------

    #[test]
    fn spectrum_absent_when_gene_keys_serializes_null() {
        let gk = (
            json!({ "frequency_assessments": [{ "suggested_frequency": Value::Null }] }),
            "test-fixture".to_string(),
        );
        let c = gift_shadow_spectrum(Some(&gk), &HashMap::new());
        assert!(matches!(
            c.status,
            ContributorStatus::Absent { ref reason, .. } if reason == "source_field_null"
        ));
    }

    #[test]
    fn spectrum_reads_a_declared_label() {
        let c = gift_shadow_spectrum(None, &opts(&[("gene_keys_frequency", json!("gift"))]));
        assert_eq!(c.normalized, Some(0.5));
        assert_eq!(c.engine_id, "practitioner-declared");
    }

    // -- heart rate variability ------------------------------------------

    fn hrv_opts(
        rmssd: f64,
        baseline: Option<f64>,
        captured: Option<&str>,
    ) -> HashMap<String, Value> {
        let mut body = json!({ "rmssd_ms": rmssd, "source": "apple-health" });
        if let Some(b) = baseline {
            body["baseline_rmssd_ms"] = json!(b);
        }
        if let Some(c) = captured {
            body["captured_at"] = json!(c);
        }
        opts(&[("hrv", body)])
    }

    #[test]
    fn hrv_is_log_symmetric_about_the_baseline() {
        let at = |r: f64, b: f64| {
            heart_rate_variability(&hrv_opts(r, Some(b), None), now())
                .unwrap()
                .normalized
                .unwrap()
        };
        assert!((at(50.0, 50.0) - 0.5).abs() < 1e-9);
        assert!((at(100.0, 50.0) - 1.0).abs() < 1e-9);
        assert!((at(25.0, 50.0) - 0.0).abs() < 1e-9);
        assert!((at(200.0, 50.0) - 1.0).abs() < 1e-9);
    }

    #[test]
    fn hrv_absent_without_a_sample() {
        let c = heart_rate_variability(&HashMap::new(), now()).unwrap();
        assert!(matches!(
            c.status,
            ContributorStatus::Absent { ref reason, .. } if reason == "no_sample"
        ));
    }

    #[test]
    fn hrv_absent_without_a_personal_baseline() {
        let c = heart_rate_variability(&hrv_opts(46.2, None, None), now()).unwrap();
        assert!(matches!(
            c.status,
            ContributorStatus::Absent { ref reason, .. } if reason == "no_personal_baseline"
        ));
    }

    #[test]
    fn hrv_absent_when_the_sample_is_stale() {
        let c = heart_rate_variability(
            &hrv_opts(46.2, Some(50.0), Some("2026-07-30T12:00:00Z")),
            now(),
        )
        .unwrap();
        assert!(matches!(
            c.status,
            ContributorStatus::Absent { ref reason, .. } if reason == "sample_stale"
        ));
    }

    #[test]
    fn hrv_out_of_range_is_an_error_not_an_absence() {
        assert!(heart_rate_variability(&hrv_opts(0.5, Some(50.0), None), now()).is_err());
        assert!(heart_rate_variability(&hrv_opts(400.0, Some(50.0), None), now()).is_err());
        assert!(heart_rate_variability(&hrv_opts(46.0, Some(900.0), None), now()).is_err());
    }

    #[test]
    fn hrv_malformed_timestamp_is_an_error() {
        assert!(
            heart_rate_variability(&hrv_opts(46.0, Some(50.0), Some("yesterday")), now()).is_err()
        );
    }
}
