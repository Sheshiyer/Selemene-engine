# HV-T03 — Wire panchanga + numerology engines into humdes validation

> Self-contained handoff packet. Read this only, then begin work.

## Issue
[#855 — Wire panchanga + numerology engines into humdes validation harness](https://github.com/Sheshiyer/Selemene-engine/issues/855)

## Identity
- **Agent:** Copilot (cloud/backend role; pattern replication is the dominant skill here)
- **Branch:** `swarm/humdes-validation/p1-w2/engines/855-copilot`
- **Worktree:** `.worktrees/855-copilot`
- **Wave:** Phase 1, Wave 2, Swarm A
- **Soft-blocked on:** HV-T02 (fixtures may grow new fields; cardinal fields you read are stable)

## Goal
Copy the proven HD validation pattern to two more engines so we get smoke + report coverage across the workspace. Catches whole classes of bugs (ephemeris range, timezone edge cases, NaN propagation) for almost no cost.

## Allowed edit surface
| File | Reason |
|---|---|
| `crates/engine-panchanga/tests/humdes_smoke_tests.rs` | NEW |
| `crates/engine-numerology/tests/humdes_smoke_tests.rs` | NEW |
| `crates/engine-panchanga/Cargo.toml` | Only if `[dev-dependencies]` need `tokio` / `serde_json` added |
| `crates/engine-numerology/Cargo.toml` | Same |
| `tests/fixtures/humdes/README.md` | Append to "Tier 4 — Cross-engine validation" with current status |

## Forbidden surface
- The engine source files themselves (no `crates/engine-*/src/*`). If you find a bug, file a child issue.
- `crates/engine-human-design/` — that's HV-T01's territory.
- `tests/fixtures/humdes/` files other than `README.md` — those belong to HV-T02.

## Required reads
1. [`crates/engine-human-design/tests/humdes_validation_tests.rs`](../../../crates/engine-human-design/tests/humdes_validation_tests.rs) — your template
2. [`crates/engine-panchanga/src/lib.rs`](../../../crates/engine-panchanga/src/lib.rs) — to see the engine's public API
3. [`crates/engine-numerology/src/lib.rs`](../../../crates/engine-numerology/src/lib.rs) — same
4. [`crates/noesis-core/src/types.rs`](../../../crates/noesis-core/src/types.rs) — `EngineInput`, `ConsciousnessEngine` trait
5. [`tests/fixtures/humdes/README.md`](../../../tests/fixtures/humdes/README.md) — Tier-4 section for the README append

## Implementation outline
For **each** engine, the test file should be a stripped-down version of `humdes_validation_tests.rs`:

```rust
//! Smoke validation: every humdes fixture runs to completion.
use engine_panchanga::PanchangaEngine;       // or NumerologyEngine
use noesis_core::{ConsciousnessEngine, EngineInput};
use serde::Deserialize;
use std::{fs, path::PathBuf};

#[derive(Debug, Deserialize)]
struct HumdesIndex { entries: Vec<HumdesEntry> }
#[derive(Debug, Deserialize)]
struct HumdesEntry {
    #[serde(rename = "type")] reading_type: String,
    reading_hash: String,
    input: String,
    has_coords: bool,
}

fn fixtures_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent().and_then(|p| p.parent()).unwrap()
        .join("tests/fixtures/humdes")
}

#[test]
fn humdes_fixtures_present() {
    assert!(fixtures_root().join("_index.json").exists(),
        "run humdes_to_selemene.py first");
}

#[tokio::test]
#[ignore = "long-running smoke against all 89 humdes fixtures"]
async fn humdes_smoke_all_fixtures() {
    let root = fixtures_root();
    let idx: HumdesIndex = serde_json::from_str(
        &fs::read_to_string(root.join("_index.json")).unwrap()).unwrap();
    let engine = PanchangaEngine::new();  // or NumerologyEngine
    let (mut ok, mut fail) = (0_usize, 0_usize);
    let mut failures: Vec<String> = vec![];
    for e in &idx.entries {
        if !e.has_coords { continue; }
        let input: EngineInput = serde_json::from_str(
            &fs::read_to_string(root.join(&e.input)).unwrap()).unwrap();
        match engine.calculate(input).await {
            Ok(_)  => ok += 1,
            Err(err) => { fail += 1; if failures.len() < 5 {
                failures.push(format!("{} {}/{}  {:?}",
                    e.reading_type, &e.reading_hash[..10], e.input, err));
            }}
        }
    }
    println!("\n=== panchanga humdes smoke ===");
    println!("  ok    : {}", ok);
    println!("  fail  : {}", fail);
    for f in &failures { println!("  {}", f); }
}
```

Numerology is simpler — it doesn't need lat/long/time, just date. Loop the same fixtures but don't gate on `has_coords`.

## Suggested first commands
```bash
cd /Volumes/madara/2026/twc-vault/01-Projects/tryambakam-noesis/Selemene-engine
git worktree add .worktrees/855-copilot swarm/humdes-validation/p1-w2/engines/855-copilot
cd .worktrees/855-copilot

# Inspect the template
cat crates/engine-human-design/tests/humdes_validation_tests.rs | head -120

# Inspect each engine's public surface
cargo doc --no-deps --package engine-panchanga --open  # or use less:
grep -E "^pub " crates/engine-panchanga/src/lib.rs
grep -E "^pub " crates/engine-numerology/src/lib.rs
```

## Verification before opening PR
```bash
cargo fmt --check --package engine-panchanga --package engine-numerology
cargo clippy --package engine-panchanga --package engine-numerology -- -D warnings
cargo test --package engine-panchanga --test humdes_smoke_tests
cargo test --package engine-numerology --test humdes_smoke_tests
cargo test --package engine-panchanga --test humdes_smoke_tests -- --ignored --nocapture
cargo test --package engine-numerology --test humdes_smoke_tests -- --ignored --nocapture
```
Both `--ignored` runs should show ok > 0 and either fail = 0 or per-failure context.

## PR template
```
Title: [HV-T03 #855] Cross-engine humdes smoke tests for panchanga + numerology

Closes #855

## Summary
- New `humdes_smoke_tests.rs` in panchanga + numerology crates
- Both engines run end-to-end against all 89 humdes fixtures
- Long-running reports gated behind `--ignored`

## Reports
```
=== panchanga ===  ok=X / Y  fail=Z
=== numerology === ok=X / Y  fail=Z
```

## Findings
- [list any engines that panicked or returned errors; file child issues if any]
```

## Escalation triggers
- If an engine panics on a real birth time: open a child bug issue per engine, exclude that fixture from your smoke (with a `// known: child-issue-#XXX` comment), keep the PR mergeable.
- If an engine doesn't implement `ConsciousnessEngine` trait: stop and escalate; that's a deeper integration gap.
