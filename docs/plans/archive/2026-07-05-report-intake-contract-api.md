# Report Intake Contract on Public API Implementation Plan

> **For Codex:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Make the public Selemene API (`/api/v1/assets/generate` and related) accept the full `ReportGenerationRequest` contract (including `report_level` L0-L5, multi-subject `subjects[]` with `NormalizedLocation`, gender/sex split, `relationship_context`, and output flags) so coding agents and future front-ends have a stable, complete form for integrated 5-system and synastry reports, while remaining additive and backward-compatible where possible.

**Architecture:** Extend the Rust request/response types to mirror the TypeScript intake types in `packages/witness-pipeline/src/intake/`. Update the additive assets handler to parse the rich request (supporting both legacy flat `birth_data` and new `subjects[]` shape), propagate `report_level`, normalized locations, and relationship context into the internal pipeline seeds and returned `source_pack` / response. Keep the current deterministic seed rendering for passes; full LLM orchestration stays in witness-pipeline. Enforce an `is_complete` gate equivalent to the TS `isCompleteReportRequest`. Update OpenAPI, contract tests, and validation.

**Tech Stack:** Rust (Axum + utoipa for OpenAPI), serde, existing noesis-core/noesis-api patterns, TypeScript witness-pipeline intake types as the source of truth for the contract.

---

## Background (for the implementer with zero context)

- The **TypeScript side** (`packages/witness-pipeline`) already defines the canonical intake:
  - `ReportGenerationRequest`
  - `ReportSubjectInput` (role, name, gender, sex_for_external_chart_source, birth fields, `normalized_location?: NormalizedLocation`)
  - `NormalizedLocation` (display_name, lat, lng, timezone, provider, confidence)
  - `relationship_context`
  - `report_level: 'L0' | 'L1' | ... | 'L5'`
  - `isCompleteReportRequest()` gate (every subject must have a confirmed normalized location)

- Current public **Rust API** (`crates/noesis-api`) only exposes a thin shape on `/assets/generate`:
  - `birth_data: { name?, date, time?, latitude, longitude, timezone }`
  - `mode: string`
  - `consciousness_level`
  - `options`

- The Rust handler in `assets.rs` does a minimal deterministic pass over a few engines and returns a witness-pipeline-*compatible* shape, but without the rich intake fields.

- Goal of this plan: Give agents and UIs one stable request body that carries everything the 5-system L0-L5 + synastry flows need.

- Keep it **additive**: do not break existing callers that send the old flat shape if we can reasonably support both.

- TDD, tiny steps, frequent commits. Exact commands in every task.

---

### Task 1: Create the plan file (meta – you are here)

**Files:**
- This plan already exists at `docs/plans/2026-07-05-report-intake-contract-api.md`

**Step:** Continue to Task 2.

---

### Task 2: Add rich intake types in Rust (NormalizedLocation + ReportSubject)

**Files:**
- Create: `crates/noesis-core/src/intake.rs` (new file for the public intake contract)
- Modify: `crates/noesis-core/src/lib.rs` (re-export the new module)
- Test: `crates/noesis-core/tests/intake_types_test.rs` (or add to existing if structure allows; we'll create a minimal test file)

**Step 1: Write the failing test (types do not exist yet)**

Create the test file with a basic construction test.

```rust
// crates/noesis-core/tests/intake_types_test.rs
use noesis_core::intake::{NormalizedLocation, ReportSubjectInput, ReportGenerationRequest};

#[test]
fn normalized_location_roundtrips() {
    let loc = NormalizedLocation {
        display_name: "Jamakhandi, Karnataka".to_string(),
        latitude: 16.5046,
        longitude: 75.2918,
        timezone: "Asia/Kolkata".to_string(),
        provider: "manual".to_string(),
        confidence: "manual".to_string(),
    };
    assert_eq!(loc.latitude, 16.5046);
}

#[test]
fn report_generation_request_requires_subjects() {
    let req = ReportGenerationRequest {
        report_level: "L0".to_string(),
        subjects: vec![],
        ..Default::default()
    };
    // We will later add a real is_complete gate; for now just construct.
    assert!(req.subjects.is_empty());
}
```

**Step 2: Run test to verify it fails**

Run:
```bash
cargo test -p noesis-core --test intake_types_test -- --nocapture
```

Expected: FAIL (module `intake` not found, or types not found).

**Step 3: Implement the types (minimal, YAGNI)**

Create `crates/noesis-core/src/intake.rs`:

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct NormalizedLocation {
    pub display_name: String,
    pub latitude: f64,
    pub longitude: f64,
    pub timezone: String,
    /// "manual" | "nominatim" | "google-places" | "mapbox" | "geonames"
    pub provider: String,
    /// "exact" | "selected" | "ambiguous" | "manual"
    pub confidence: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ReportSubjectInput {
    pub role: String, // "primary" | "partner" | ...
    pub name: Option<String>,
    pub gender: Option<String>,
    pub sex_for_external_chart_source: Option<String>,
    pub birth_date: String,
    pub birth_time: Option<String>,
    pub birth_time_confidence: Option<String>,
    pub birth_location_query: Option<String>,
    pub normalized_location: Option<NormalizedLocation>,
    pub relationship_label: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct RelationshipContext {
    pub r#type: Option<String>,
    pub mapping_goal: Option<String>,
    pub sensitivity_level: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ReportGenerationRequest {
    pub report_level: String, // "L0".."L5"
    pub report_mode: Option<String>,
    pub subjects: Vec<ReportSubjectInput>,
    pub relationship_context: Option<RelationshipContext>,
    pub output: Option<serde_json::Value>,
}
```

Re-export in `crates/noesis-core/src/lib.rs` (add near other pub mods):

```rust
pub mod intake;
```

**Step 4: Run test to verify it passes**

Run the same cargo test command.

Expected: PASS.

**Step 5: Commit**

```bash
git add crates/noesis-core/src/intake.rs crates/noesis-core/src/lib.rs crates/noesis-core/tests/intake_types_test.rs
git commit -m "feat(core): add NormalizedLocation + ReportSubjectInput + ReportGenerationRequest types"
```

---

### Task 3: Expose the new types in the API OpenAPI surface

**Files:**
- Modify: `crates/noesis-api/src/lib.rs:260-283` (add the new types to the components list)

**Step 1: Write a failing OpenAPI generation / compile check that expects the types**

For now, we will use a compile-time check by referencing them in the schema registration.

Add temporarily in a test or just reference in the code so the build will fail until registered.

**Step 2: Run typecheck / build to see failure**

```bash
cargo check -p noesis-api
```

Expect: cannot find the types in the api crate (they live in noesis-core).

**Step 3: Re-export from noesis-api and register with utoipa**

In `crates/noesis-api/src/lib.rs`, ensure noesis-core intake is visible and add to the `components(schemas(...))` list:

```rust
handlers::assets::...,
noesis_core::intake::NormalizedLocation,
noesis_core::intake::ReportSubjectInput,
noesis_core::intake::ReportGenerationRequest,
noesis_core::intake::RelationshipContext,
```

Also add a tag if needed.

**Step 4: Run `cargo check -p noesis-api` until green**

**Step 5: Commit**

```bash
git add crates/noesis-api/src/lib.rs
git commit -m "feat(api): register rich intake types in OpenAPI components"
```

---

### Task 4: Extend AssetGenerateRequest to accept the rich contract (additive)

**Files:**
- Modify: `crates/noesis-api/src/handlers/assets.rs:23-34`

**Step 1: Add the new optional fields to the request struct (write the struct change)**

Update to:

```rust
#[derive(Deserialize, ToSchema)]
pub struct AssetGenerateRequest {
    pub birth_data: Option<noesis_core::BirthData>, // legacy path
    pub mode: String,
    #[serde(default)]
    pub consciousness_level: u8,
    pub options: Option<Value>,

    // New rich intake (preferred for L0-L5 + synastry)
    pub report_level: Option<String>,
    pub subjects: Option<Vec<noesis_core::intake::ReportSubjectInput>>,
    pub relationship_context: Option<noesis_core::intake::RelationshipContext>,
}
```

**Step 2: Run `cargo check -p noesis-api` to see if it compiles (it should, but we will add usage later)**

**Step 3: Commit the request shape change**

```bash
git add crates/noesis-api/src/handlers/assets.rs
git commit -m "feat(api): extend AssetGenerateRequest with report_level + subjects + relationship_context"
```

---

### Task 5: Add a helper to convert legacy birth_data → subjects shape (internal)

**Files:**
- Modify: `crates/noesis-api/src/handlers/assets.rs` (add a small conversion function near the top of the file)

**Step 1: Write the helper as a failing compile first if needed, then implement**

```rust
fn legacy_birth_to_subjects(bd: &noesis_core::BirthData) -> Vec<noesis_core::intake::ReportSubjectInput> {
    vec![noesis_core::intake::ReportSubjectInput {
        role: "primary".to_string(),
        name: bd.name.clone(),
        gender: None,
        sex_for_external_chart_source: None,
        birth_date: bd.date.clone(),
        birth_time: bd.time.clone(),
        birth_time_confidence: None,
        birth_location_query: None,
        normalized_location: Some(noesis_core::intake::NormalizedLocation {
            display_name: "legacy".to_string(),
            latitude: bd.latitude,
            longitude: bd.longitude,
            timezone: bd.timezone.clone(),
            provider: "legacy".to_string(),
            confidence: "legacy".to_string(),
        }),
        relationship_label: None,
    }]
}
```

**Step 2-4:** Check, run relevant test, commit small.

---

### Task 6: Update the generate handler to use report_level and subjects when provided

**Files:**
- Modify: `crates/noesis-api/src/handlers/assets.rs` (the generate function, around lines 68-164)

**Step 1: Write a small failing test first that sends `report_level` and `subjects` and asserts they are echoed or used.**

Add to the existing contract test file or a new test.

**Step 2:** Run test → should fail because handler ignores the new fields.

**Step 3:** In the handler, prefer `req.report_level` over default, and derive subjects from `req.subjects` or fall back to legacy conversion.

Propagate `report_level` into the response if we want (or keep it inside source_pack for now).

**Step 4:** Run the contract test until it passes the new shape.

**Step 5:** Commit.

Exact command example:
```bash
cargo test -p noesis-api --test assets_generate_contract_test -- --nocapture
```

---

### Task 7: Add is_complete gate (Rust equivalent of TS isCompleteReportRequest)

**Files:**
- Modify: `crates/noesis-api/src/handlers/assets.rs` (add a validation function)
- Test: Extend contract test to assert 422 or clear error when a subject lacks normalized_location

**Step 1:** Write the validation function + failing test that sends incomplete subjects.

```rust
fn has_complete_locations(subjects: &[noesis_core::intake::ReportSubjectInput]) -> bool {
    !subjects.is_empty() && subjects.iter().all(|s| s.normalized_location.is_some())
}
```

Return a proper error response if not complete.

**Step 2-5:** Red → run → implement → green → commit.

---

### Task 8: Wire report_level and normalized subjects into engine calls and source_pack (where meaningful)

**Files:**
- Modify: `crates/noesis-api/src/handlers/assets.rs` (the make_input / engine call area and build_source_pack_with_audit call)

For now, we mainly want the data to flow into the returned `source_pack` and be visible in the response so the TS side (or future UI) can trust the contract was honored.

Add `report_level` to the source_pack json we emit, and include subject count or first normalized location summary.

**Step 1-5:** TDD a small assertion in contract test that the response now contains the report_level or subject info when sent that way.

---

### Task 9: Update contract tests for L0 + synastry shapes

**Files:**
- Modify: `crates/noesis-api/tests/assets_generate_contract_test.rs`

Add tests:
- Send `report_level: "L0"`, `subjects: [ { role: "primary", normalized_location: {...}, ... } ]`
- Assert response has the level and source_pack reflects it.
- Send two subjects (synastry shape) and assert no crash + engines_used is populated.

**Step 1-5 per new test:** write failing, run, implement support, run green, commit.

---

### Task 10: Ensure OpenAPI example / docs reflect the new fields

**Files:**
- Modify: `crates/noesis-api/src/handlers/assets.rs` (add examples or comments on the request)
- Optionally update any markdown in docs/api/

**Step 1:** Add a utoipa example on the struct for a rich request.
**Step 2:** Run `cargo test` or the openapi generation check that exists in the project.
**Step 3-5:** Commit.

---

### Task 11: Full verification + typecheck + existing tests still pass

**Commands (run in order):**

```bash
cargo check -p noesis-core -p noesis-api
cargo test -p noesis-core --test intake_types_test
cargo test -p noesis-api --test assets_generate_contract_test
cargo test -p noesis-api -- --quiet   # broader api tests
```

Add a commit if everything is green:
```bash
git add -A
git commit -m "chore: full verification of report intake contract on assets/generate"
```

---

### Task 12: (Optional but recommended) Update the TS smoke / L0 test scripts to call the new shape

**Files:**
- `packages/witness-pipeline/scripts/sapna-l0-test.ts` (or a new small script)
- Or just document that the rich shape is now legal on the API.

Keep minimal per YAGNI.

---

## Success Criteria

- A client can POST to `/api/v1/assets/generate` with `report_level`, `subjects[]` containing `normalized_location`, `gender`, etc. and get a 200 with the fields echoed in the response or source_pack.
- Legacy flat `birth_data` still works (additive).
- `is_complete` style validation rejects incomplete multi-subject requests.
- All existing contract tests + broader api tests remain green.
- Types are visible in the generated OpenAPI.

## Notes for the implementer

- The real heavy lifting for L0 11-pass kundali, rubric, pattern extraction, and LLM calls lives in `@noesis/witness-pipeline`.
- This plan only makes the **request contract** first-class on the wire and wires the key fields through the additive surface.
- Do not over-engineer the Rust stub rendering to become a full orchestrator — keep it additive.

---

**Plan complete and saved to `docs/plans/2026-07-05-report-intake-contract-api.md`.**

Two execution options:

1. **Subagent-Driven (this session)** — I dispatch fresh subagent per task, review between tasks, fast iteration.
2. **Parallel Session (separate)** — Open new session with `executing-plans`, batch execution with checkpoints.

Which approach?
