# Selemene Engine Hygiene Report (2026-03-08)

> Update (2026-03-10): parts of this report describe an intermediate provider-routing experiment that is no longer the active runtime state. The current production path for `panchanga`, `vimshottari`, and `transits` is native Rust. Read this document as historical hygiene tracking plus issue context, not as the current runtime architecture.

## Context
This report consolidates engine hygiene findings for astrology-related engines and links issue tracking for remediation.

Reference input used for validation:
- Date: **1991-08-13**
- Time: **13:31 IST**
- Location: **12.9340, 77.6214**

## Key finding
The current API runtime path registers native/local engines directly for major astrology engines, so output drift likely comes from **calculation-path inconsistencies** and **cross-engine contract drift**, not only response mapping.

Additional architecture finding:
- Provider env vars (`FREE_ASTROLOGY_API_KEY`, `VEDIC_ENGINE_PROVIDER`) are present in config/docs but not currently used to select panchanga/vimshottari/human-design/gene-keys runtime in `noesis-api`.
- Tracking issue: **#508** (provider wiring gap).

Relevant code paths:
- `crates/noesis-api/src/lib.rs` (native engine registration)
- `crates/engine-panchanga/src/lib.rs` (metadata backend: `native-rust`)
- `crates/engine-vimshottari/src/engine.rs` (backend label: `swiss-ephemeris`)

---

## Challenge matrix

| Engine | Status | Observed challenge | Tracking issue |
|---|---|---|---|
| Panchanga | Fail | Potential sidereal/tropical mismatch and/or inconsistent moon basis | #502 |
| Vimshottari | Fail | Moon longitude → nakshatra → dasha chain appears shifted for canonical case | #503 |
| Human Design | Fail | Design-side (prenatal) activations drift, affecting type/authority/definition | #504 |
| Gene Keys | Suspect/Fail | Design-side outputs appear misaligned with HD-derived source activations | #505 |
| Sigil-forge | Fail | Runtime 500 / bridge error on execution | #506 |
| Vedic-clock | Suspect | Potential UTC-vs-local timezone basis mismatch | #507 |

Parent tracker: **#501**

---

## GitHub issue set created

- Parent: https://github.com/Sheshiyer/Selemene-engine/issues/501
- Panchanga: https://github.com/Sheshiyer/Selemene-engine/issues/502
- Vimshottari: https://github.com/Sheshiyer/Selemene-engine/issues/503
- Human Design: https://github.com/Sheshiyer/Selemene-engine/issues/504
- Gene Keys: https://github.com/Sheshiyer/Selemene-engine/issues/505
- Sigil-forge: https://github.com/Sheshiyer/Selemene-engine/issues/506
- Vedic-clock: https://github.com/Sheshiyer/Selemene-engine/issues/507

---

## Recommended fix order
1. **Panchanga + Vimshottari**: lock canonical sidereal baseline and cross-engine nakshatra consistency test.
2. **Human Design + Gene Keys**: fix design-side contract path and derive GK strictly from validated HD activations.
3. **Sigil-forge**: eliminate 500s and enforce structured error handling.
4. **Vedic-clock**: make timezone basis explicit in logic + output metadata.

---

## Verification strategy
- Add deterministic fixtures for canonical birth input.
- Add cross-engine consistency checks (shared moon/nakshatra expectations where applicable).
- Require regression tests for each issue acceptance criterion before close.

---

## Implementation update (2026-03-08, #502 + #503)

Completed in code:
- `crates/engine-panchanga/src/lib.rs`
  - Added Lahiri ayanamsa conversion helper.
  - Switched emitted longitudes/nakshatra derivation to sidereal basis.
  - Added canonical regression test asserting **Uttara Phalguni** for 1991-08-13 13:31 IST.
- `crates/engine-vimshottari/src/engine.rs`
  - Fixed birth datetime parsing to honor timezone (local -> UTC).
  - Converted Swiss tropical moon longitude to Lahiri sidereal before nakshatra/dasha derivation.
  - Added regression tests for timezone conversion and canonical **Uttara Phalguni** resolution.
- `crates/noesis-api/tests/engine_consistency_tests.rs`
  - Added cross-engine parity test: Panchanga nakshatra == Vimshottari birth nakshatra for canonical input.

Commands executed:
- `cargo test -p engine-panchanga`
- `cargo test -p engine-vimshottari`
- `cargo test -p noesis-api --test engine_consistency_tests`

Result: all tests passed.

## Implementation update (2026-03-08, #504 + #505)

Completed in code:
- `crates/engine-human-design/src/gate_sequence.rs`
  - Corrected Rave Mandala ordering transposition causing canonical design gate drift:
    - swapped `23/8` ordering in the 45°-56.25° segment
    - swapped opposite pair `43/14` in the 225°-236.25° segment
- `crates/engine-human-design/src/engine.rs`
  - Added canonical regression asserting core incarnation-cross gates for the canonical input:
    - personality: Sun=4, Earth=49
    - design: Sun=23, Earth=43
- `crates/engine-gene-keys/src/engine.rs`
  - Added contract test proving birth-mode Gene Keys derives sequence gates directly from HD output.
- `crates/noesis-api/tests/engine_consistency_tests.rs`
  - Added cross-engine canonical alignment test for HD ↔ GK sequence parity.

Commands executed (targeted):
- `cargo test -p engine-human-design test_canonical_profile_regression`
- `cargo test -p engine-gene-keys test_gk_birth_mode_derives_from_hd_engine`
- `cargo test -p noesis-api --test engine_consistency_tests`

Result: all targeted regression tests passed.

## Implementation update (2026-03-08, #507)

Completed in code:
- `crates/engine-vedic-clock/src/engine.rs`
  - Timezone resolution now honors `birth_data.timezone` when `options.timezone_offset` is absent.
  - Supports `Asia/Kolkata`, `UTC/GMT`, and explicit `+HH:MM` offsets.
  - Calculation and cache-key paths now use unified timezone resolution from full `EngineInput`.
  - Added test coverage for birth_data timezone fallback.

Command executed:
- `cargo test -p engine-vedic-clock`

Result: all tests passed.

## Implementation update (2026-03-08, #506 + #508)

Completed in code:
- `crates/noesis-bridge/src/lib.rs`
  - Added pre-validation for `sigil-forge`: requires intention/question alias in options.
  - Missing intention now returns `EngineError::ValidationError` (422 path), preventing immediate bridge 500s for malformed requests.
  - Non-2xx bridge responses are now mapped as:
    - 4xx -> `ValidationError`
    - 5xx -> `BridgeError`
- `crates/noesis-api/src/lib.rs`
  - Added explicit provider-mode visibility in readiness/status responses:
    - `configured_vedic_provider`
    - `effective_vedic_provider`
    - `vedic_engine_modes` map (per engine)
  - Added deterministic tests for provider-mode mapping.

Commands executed:
- `cargo test -p noesis-bridge`
- `cargo test -p noesis-api provider_mode_tests`

Result: tests passed.

## Implementation update (2026-03-08, #508 true switching pass)

Completed in code:
- `crates/noesis-api/Cargo.toml`
  - Added dependency on `noesis-vedic-api` for runtime provider dispatch.
- `crates/noesis-api/src/lib.rs`
  - Added provider-aware dispatch in `calculate_handler`:
    - if `VEDIC_ENGINE_PROVIDER=api` and engine is `panchanga`/`vimshottari`, call FreeAstrologyAPI via `CachedVedicClient`.
    - if provider call fails and `VEDIC_ENGINE_FALLBACK_ENABLED=true`, fallback to native engine path.
  - Added provider call mapping functions:
    - provider panchanga -> native-compatible panchanga result envelope
    - provider vimshottari -> native-compatible vimshottari result envelope (timeline/current-period subset)
  - Updated provider mode reporting:
    - `panchanga` and `vimshottari` now report `api` mode when configured provider is `api`.
- `crates/noesis-api/src/lib.rs` tests:
  - updated/verified provider mode tests for new routing semantics.

Validation:
- `cargo test -p noesis-api provider_mode_tests -- --nocapture` ✅

Notes:
- This pass implements real switching for `panchanga` and `vimshottari`.
- `human-design`, `gene-keys`, `transits`, and `vedic-clock` remain native in `api` mode (reported as `native (api-unwired)`).

### Verification expansion (post-switch hardening)

Added deterministic unit coverage in `crates/noesis-api/src/lib.rs` (`provider_mode_tests`):
- provider routing gate tests (`use_api_provider_for_engine`) for supported vs unsupported engines
- fallback env flag parsing tests (`VEDIC_ENGINE_FALLBACK_ENABLED`)
- birth component parsing test for canonical IANA timezone (`Asia/Kolkata`)
- provider->EngineOutput contract mapping tests for both:
  - panchanga
  - vimshottari

Validation run:
- `cargo test -p noesis-api provider_mode_tests -- --nocapture` -> **7 passed**
