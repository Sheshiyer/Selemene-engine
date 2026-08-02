//! Vocabulary gate.
//!
//! The house vocabulary rules are not decoration on the prose layer: they bind
//! anything a reader ever sees, and this engine's payload is read directly.
//! The primary gate scans the serialized output rather than the source, which
//! covers every string that reaches a reader while avoiding false positives
//! from `std::path`.

mod support;

use engine_financial_biosensor::FinancialBiosensorEngine;
use noesis_core::ConsciousnessEngine;
use serde_json::json;
use support::*;

/// Substrings that must not appear in anything a reader sees.
///
/// `optimiz` catches the whole family; `manifest` catches manifesting.
/// `abundance` is a categorical fail in its own right — the house instruction
/// is to remove the frame, not substitute a synonym.
const BANNED_SUBSTRINGS: &[&str] = &[
    "abundance",
    "optimiz",
    "manifest",
    "journey",
    "healing",
    "productivity",
    "witnessos",
    "higher self",
    "authentic self",
    "vibration",
];

/// Banned as standalone words only, because they appear inside ordinary
/// technical vocabulary.
const BANNED_WORDS: &[&str] = &["path", "paths", "ai", "hacks", "tribe"];

fn assert_clean(label: &str, text: &str) {
    let lowered = text.to_lowercase();

    for banned in BANNED_SUBSTRINGS {
        assert!(
            !lowered.contains(banned),
            "{label} contains the banned term '{banned}'"
        );
    }

    let words: Vec<String> = lowered
        .split(|c: char| !c.is_ascii_alphanumeric())
        .map(str::to_string)
        .collect();
    for banned in BANNED_WORDS {
        assert!(
            !words.iter().any(|w| w == banned),
            "{label} contains the banned word '{banned}'"
        );
    }
}

async fn full_output(options: &[(&str, serde_json::Value)]) -> (String, String) {
    let engine = FinancialBiosensorEngine::with_sources(sources());
    let output = engine
        .calculate(canonical_input(options))
        .await
        .expect("calculate should succeed");
    let body = serde_json::to_string(&output.result).expect("serializes");
    (body, output.witness_prompt)
}

#[tokio::test]
async fn a_full_reading_carries_no_banned_vocabulary() {
    let (body, prompt) = full_output(&[
        ("deliberation_hours", json!(36.0)),
        ("hrv", fresh_hrv(46.2, 51.0)),
        ("gene_keys_frequency", json!("gift")),
        ("decision_context", json!("a decision with real stakes")),
    ])
    .await;

    assert_clean("the serialized result", &body);
    assert_clean("the witness prompt", &prompt);
}

/// The degraded route emits different strings — absence reasons, withheld
/// values, extra self-inquiry lines — so it is gated separately.
#[tokio::test]
async fn a_degraded_reading_carries_no_banned_vocabulary() {
    let (body, prompt) = full_output(&[]).await;

    assert_clean("the degraded serialized result", &body);
    assert_clean("the degraded witness prompt", &prompt);
}

#[tokio::test]
async fn validation_messages_carry_no_banned_vocabulary() {
    let engine = FinancialBiosensorEngine::with_sources(sources());
    let output = engine
        .calculate(canonical_input(&[]))
        .await
        .expect("calculate should succeed");
    let validation = engine.validate(&output).await.expect("validate should run");

    assert_clean("the validation messages", &validation.messages.join(" "));
}

/// A softer scan of the crate's own source, so a banned term cannot be
/// introduced in a comment or an identifier and lie dormant until some future
/// route serializes it. `std::path` and friends are allowed.
#[test]
fn the_crate_source_carries_no_banned_vocabulary() {
    let src = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut checked = 0;

    for entry in std::fs::read_dir(&src).expect("src should read") {
        let file = entry.expect("dir entry").path();
        if file.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        let text = std::fs::read_to_string(&file).expect("source should read");
        let lowered = text.to_lowercase();

        for banned in BANNED_SUBSTRINGS {
            assert!(
                !lowered.contains(banned),
                "{} contains the banned term '{}'",
                file.display(),
                banned
            );
        }
        checked += 1;
    }

    assert!(
        checked >= 7,
        "expected to scan every module, saw {}",
        checked
    );
}
