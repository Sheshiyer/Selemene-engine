//! End-to-end tests over the composed engine.
//!
//! The three-lock gate is asserted rather than described: every present
//! contributor must name its source, its version, the fields it read, and the
//! normalization it applied, and removing a contributor must change the
//! number.

mod support;

use engine_financial_biosensor::{
    FinancialBiosensorEngine, FinancialBiosensorResult, SourceEngines, FORMULA_VERSION,
};
use noesis_core::ConsciousnessEngine;
use serde_json::json;
use support::*;

fn engine() -> FinancialBiosensorEngine {
    FinancialBiosensorEngine::with_sources(sources())
}

async fn run(options: &[(&str, serde_json::Value)]) -> FinancialBiosensorResult {
    let output = engine()
        .calculate(canonical_input(options))
        .await
        .expect("calculate should succeed");
    serde_json::from_value(output.result).expect("result should round-trip")
}

#[tokio::test]
async fn all_four_surfaces_are_produced() {
    let r = run(&[]).await;

    assert_eq!(r.daily_decision_index.date.to_string(), "2026-08-01");
    assert_eq!(r.weekly_risk_landscape.days.len(), 7);
    assert_eq!(r.monthly_alignment_calendar.days.len(), 31);
    assert_eq!(r.monthly_alignment_calendar.month, 8);
    assert!(!r.decision_ownership_reflection.authorship_checks.is_empty());
}

#[tokio::test]
async fn the_payload_declares_what_kind_of_claim_it_makes() {
    let r = run(&[]).await;
    assert_eq!(r.declaration.claim_mode, "house-model");
    assert_eq!(r.declaration.excluded_claim, "not-prediction");
    assert_eq!(r.declaration.formula_version, FORMULA_VERSION);
    assert!(r.declaration.statement.contains("not fitted to data"));
    assert!(r.declaration.authorship.contains("does not decide"));
}

/// The three-lock gate, as an assertion.
#[tokio::test]
async fn every_present_contributor_carries_complete_provenance() {
    let r = run(&[
        ("deliberation_hours", json!(36.0)),
        ("hrv", fresh_hrv(46.2, 51.0)),
        ("gene_keys_frequency", json!("gift")),
    ])
    .await;

    assert_eq!(r.provenance.contributors.len(), 6);
    assert_eq!(r.provenance.formula_version, FORMULA_VERSION);

    let mut present = 0;
    for c in &r.provenance.contributors {
        assert!(!c.contributor.is_empty());
        assert!(["Kha", "Ba", "La"].contains(&c.leg.as_str()));
        if c.normalized.is_some() {
            present += 1;
            assert!(!c.engine_id.is_empty(), "{}: no engine_id", c.contributor);
            assert!(
                !c.engine_version.is_empty(),
                "{}: no engine_version",
                c.contributor
            );
            assert!(
                !c.fields_consumed.is_empty(),
                "{}: no fields_consumed",
                c.contributor
            );
            assert!(
                !c.normalization.is_empty(),
                "{}: no normalization",
                c.contributor
            );
            assert!(c.weight_effective > 0.0);
        } else {
            assert_eq!(
                c.weight_effective, 0.0,
                "{}: absent but weighted",
                c.contributor
            );
        }
    }
    assert_eq!(present, 6, "all six should be readable with these options");
    assert_eq!(r.provenance.coverage, 1.0);
    assert_eq!(r.provenance.ba_factor, 1.0);
}

/// Structural consequence: remove the biometric sample and the number moves.
#[tokio::test]
async fn removing_a_contributor_changes_the_number() {
    let with_body = run(&[
        ("deliberation_hours", json!(36.0)),
        ("hrv", fresh_hrv(90.0, 45.0)),
    ])
    .await;
    let without_body = run(&[("deliberation_hours", json!(36.0))]).await;

    assert_ne!(
        with_body.daily_decision_index.value, without_body.daily_decision_index.value,
        "the index must depend on the contributor"
    );
    assert_eq!(with_body.provenance.ba_factor, 1.0);
    assert_eq!(without_body.provenance.ba_factor, 0.8);
    assert!(without_body.provenance.coverage < with_body.provenance.coverage);
}

#[tokio::test]
async fn an_unsampled_body_is_named_rather_than_absorbed() {
    let r = run(&[]).await;
    let hrv = r
        .provenance
        .contributors
        .iter()
        .find(|c| c.contributor == "heart_rate_variability")
        .expect("the biometric contributor is always listed");
    assert!(hrv.normalized.is_none());
    assert_eq!(hrv.leg, "Ba");
    assert!(r
        .decision_ownership_reflection
        .unconsulted
        .iter()
        .any(|u| u.starts_with("heart_rate_variability")));
}

/// Gene Keys ships `suggested_frequency` as null, so the spectrum is absent by
/// default and says which call site is responsible.
#[tokio::test]
async fn the_gift_shadow_spectrum_reports_its_own_unavailability() {
    let r = run(&[]).await;
    let spectrum = r
        .provenance
        .contributors
        .iter()
        .find(|c| c.contributor == "gift_shadow_spectrum")
        .expect("listed");
    let rendered = serde_json::to_string(&spectrum.status).expect("serializes");
    assert!(rendered.contains("source_field_null"), "got {}", rendered);
    assert!(rendered.contains("gene_keys_frequency"));
}

#[tokio::test]
async fn a_failing_source_degrades_rather_than_erroring() {
    let mut s = sources();
    s.transits = MockEngine::failing("transits");
    s.vimshottari = MockEngine::failing("vimshottari");
    let engine = FinancialBiosensorEngine::with_sources(s);

    let output = engine
        .calculate(canonical_input(&[]))
        .await
        .expect("a source outage is an absence, not an engine failure");
    let r: FinancialBiosensorResult = serde_json::from_value(output.result).expect("round-trips");

    // Only the three-wave cycle remains: 0.15 coverage, below the floor.
    assert_eq!(r.provenance.coverage, 0.15);
    assert_eq!(r.daily_decision_index.value, None);
    assert!(r.daily_decision_index.convergence.is_none());
}

#[tokio::test]
async fn every_source_failing_is_an_error() {
    let engine = FinancialBiosensorEngine::with_sources(SourceEngines {
        human_design: MockEngine::failing("human-design"),
        gene_keys: MockEngine::failing("gene-keys"),
        vimshottari: MockEngine::failing("vimshottari"),
        transits: MockEngine::failing("transits"),
        biorhythm: MockEngine::failing("biorhythm"),
    });
    assert!(engine.calculate(canonical_input(&[])).await.is_err());
}

#[tokio::test]
async fn a_malformed_biometric_sample_is_rejected_outright() {
    let bad = json!({ "rmssd_ms": 900.0, "baseline_rmssd_ms": 50.0 });
    assert!(engine()
        .calculate(canonical_input(&[("hrv", bad)]))
        .await
        .is_err());
}

#[tokio::test]
async fn the_weekly_series_moves_and_names_what_it_held_still() {
    let r = run(&[]).await;
    let w = &r.weekly_risk_landscape;

    let waves: Vec<Option<f64>> = w.days.iter().map(|d| d.three_wave).collect();
    assert!(
        waves.windows(2).any(|p| p[0] != p[1]),
        "the forward series should vary: {:?}",
        waves
    );
    assert!(w
        .held_constant
        .contains(&"active_planetary_weather".to_string()));
    assert!(!w.held_constant.contains(&"three_wave_cycle".to_string()));

    // Chronofield decrements toward the boundary at 45 days out.
    assert_eq!(w.days[0].days_to_transition, Some(45));
    assert_eq!(w.days[6].days_to_transition, Some(39));
}

#[tokio::test]
async fn saturn_pressure_is_context_not_a_term_in_the_index() {
    let mut s = sources();
    s.transits = MockEngine::arc("transits", transits_payload(true));
    let engine = FinancialBiosensorEngine::with_sources(s);
    let output = engine.calculate(canonical_input(&[])).await.unwrap();
    let pressured: FinancialBiosensorResult = serde_json::from_value(output.result).unwrap();

    let calm = run(&[]).await;

    assert!(pressured.weekly_risk_landscape.saturn_pressure);
    assert!(!calm.weekly_risk_landscape.saturn_pressure);
    // Sade Sati is reported, never folded into the number.
    assert_eq!(
        pressured.daily_decision_index.value,
        calm.daily_decision_index.value
    );
}

#[tokio::test]
async fn validate_reports_confidence_as_coverage_times_the_ba_factor() {
    let e = engine();
    let output = e
        .calculate(canonical_input(&[("deliberation_hours", json!(36.0))]))
        .await
        .unwrap();
    let result: FinancialBiosensorResult = serde_json::from_value(output.result.clone()).unwrap();

    let validation = e.validate(&output).await.unwrap();
    assert!(validation.valid, "messages: {:?}", validation.messages);

    let expected = result.provenance.coverage * result.provenance.ba_factor;
    assert!(
        (validation.confidence - expected).abs() < 1e-9,
        "confidence {} vs expected {}",
        validation.confidence,
        expected
    );
    assert!(validation
        .messages
        .iter()
        .any(|m| m.contains("heart_rate_variability")));
}

#[tokio::test]
async fn the_witness_prompt_meets_the_cross_engine_contract() {
    let output = engine().calculate(canonical_input(&[])).await.unwrap();
    let prompt = output.witness_prompt;

    assert!(prompt.trim_end().ends_with('?'), "got: {}", prompt);
    assert!(prompt.trim().len() >= 24);
    let lowered = prompt.to_lowercase();
    assert!(!lowered.contains("you should"));
    assert!(!lowered.contains("you must"));
}

/// `options` is a HashMap, so the key must be built from a sorted subset.
#[tokio::test]
async fn the_cache_key_is_stable_across_option_insertion_order() {
    let e = engine();
    let a = canonical_input(&[
        ("deliberation_hours", json!(36.0)),
        ("gene_keys_frequency", json!("gift")),
        ("decision_context", json!("the contract")),
        ("hrv", fresh_hrv(46.2, 51.0)),
    ]);
    let b = canonical_input(&[
        ("hrv", fresh_hrv(46.2, 51.0)),
        ("decision_context", json!("the contract")),
        ("gene_keys_frequency", json!("gift")),
        ("deliberation_hours", json!(36.0)),
    ]);

    assert_eq!(e.cache_key(&a), e.cache_key(&b));
    assert_eq!(e.cache_key(&a), e.cache_key(&a));
    assert!(e.cache_key(&a).starts_with("financial-biosensor:"));
}

#[tokio::test]
async fn the_cache_key_moves_when_a_consequential_input_moves() {
    let e = engine();
    let base = canonical_input(&[("deliberation_hours", json!(36.0))]);
    let moved = canonical_input(&[("deliberation_hours", json!(72.0))]);
    assert_ne!(e.cache_key(&base), e.cache_key(&moved));
}

#[tokio::test]
async fn engine_identity_matches_the_registry_contract() {
    let e = engine();
    assert_eq!(e.engine_id(), "financial-biosensor");
    assert_eq!(e.engine_name(), "Financial Biosensor");
    // At or above the maximum required_phase of its sources.
    assert_eq!(e.required_phase(), 3);
}
