# Customer ChatGPT plugin: current code audit

Observed 2026-09-30, read-only source investigation plus bounded synthetic tests. Checkout: `codex/selemene-capability-consumer-surface`, HEAD `3e9ecb61e3f4a4b40ac624a5ecefa5753876b26f`. Existing README/witness-pipeline modifications and untracked reports/modes were preserved. This document describes local code, not deployed Railway/Cloudflare behavior. No live reading, provider generation, credential export, deployment, or account mutation was performed.

## Main-lineage comparison

Also inspected the existing local `origin/main` reference at `ebd97fe940a3454b0425ff60d56003b07337564e`, without fetch or checkout. Auth service, middleware, witness handler, assets handler, and universal bridge files have no diff between that reference and the audited checkout: the shared-credential/header mismatch, report phase escalation call paths, and seed-stub behavior are therefore present in both source lineages. Main also has output-only `/validate` (`lib.rs:3044-3084`), metadata-only `/info` (`:3121-3125`), reading persistence/profile population (`:2806-2869`, `:3263`), and silent workflow engine omission (`noesis-orchestrator/src/lib.rs:501-530`).

The ordinary-user `/api/v1/engines/capabilities` route is present locally but absent from this main reference; main has the admin capabilities route. This matters for adapter discovery: do not depend on the new user route until that exact code is deployed. Main additionally has billing disabled/free/Dodo release-mode controls (`crates/noesis-api/src/billing.rs:12-35`), so the presence of quota and emitter code does not mean paid billing is active. Deployment lineage and live endpoint observations must come from the separate infrastructure audit; no live exploit or production persistence test was performed here.

## Decision

The Rust API is a useful foundation for a customer MCP adapter. The existing universal bridge is not safe or compatible as the customer endpoint without substantial changes. Authentication/account linking, accurate tool contracts, persistence disclosure, and explicit partial-result handling precede a connected customer trial. Report generation remains a separate integration lane.

## Findings requiring correction before customer exposure

### 1. Existing bridge uses one global credential, has no incoming customer authentication, and sends an API key through the wrong header

`bridges/universal-tool-server/main.py:14` reads one process-wide `SELEMENE_API_KEY`; `:45-77` accepts arbitrary callers of `/tools/execute` with no authenticated principal; `:63-64` sends that key as `Authorization: Bearer`. Rust treats Bearer exclusively as JWT and immediately returns 401 for invalid JWTs (`crates/noesis-api/src/middleware.rs:181-204`); API keys must arrive as `X-API-Key` (`:209-224`). Therefore a normal Selemene API key configured in this bridge does not authenticate correctly. Replacing the header alone would still collapse all callers into the key owner's identity, quota, reading history, profile, and phase.

The bridge exposes raw TypeScript sidecar routes (`main.py:29-32`) and returns upstream bodies verbatim in errors (`:74-77`). `/tools` and `/tools/execute` are ordinary REST routes; there is no MCP initialization, protocol negotiation, `tools/list`, or `tools/call` implementation in this file. The adapter must use only an allowlisted Rust API surface, encode validated path identifiers, sanitize errors, and bind every calculation to a verified Selemene user.

### 2. The initial specification incorrectly maps input validation to output validation

`POST /api/v1/engines/:id/validate` accepts `EngineOutput` and calls `engine.validate(&output)`, not birth input (`crates/noesis-api/src/lib.rs:3124-3164`). It cannot implement the proposed `selemene_validate_input` tool. `GET .../info` returns only `engine_id`, `engine_name`, `required_phase` (`:3184-3205`), not an input schema or required birth fields.

Input preparation must use a verified per-engine schema registry or a new input-preflight API. Do not present `/validate` as input validation. The calculation boundary has `ApiEngineInput`, denies unknown top-level fields, and normalizes canonical and legacy input (`:1320-1507`). Explicit `contract_version: v1` requires a bounded `consciousness_level` and `parameters` map (`:1365-1390`); the direct calculation handler then overwrites the phase with authenticated phase (`:2853-2862`). Workflow execution still accepts legacy `EngineInput` (`:3279-3285`), so the two request shapes are not interchangeable.

### 3. Two report handlers allow the caller to raise the engine phase

Source-confirmed authorization inconsistency: `witness::interpret` sets `req.consciousness_level.max(user.consciousness_level)` (`crates/noesis-api/src/handlers/witness.rs:78`) and sends that value into engine calls (`:99`, `:131-138`). `assets::generate` does the same (`crates/noesis-api/src/handlers/assets.rs:160`, `:229-241`). Both request fields are deserialized as `u8` without the canonical input's 0–5 check.

The call path is direct: `WorkflowOrchestrator::execute_engine` passes the supplied phase to `registry.execute_routed` (`crates/noesis-orchestrator/src/lib.rs:417-425`); the gate rejects only when `engine.required_phase() > user_phase` (`:209-224`). For an authenticated phase-0 caller and request phase 5, both handlers pass 5, permitting an otherwise phase-1 engine such as Human Design (`crates/engine-human-design/src/engine.rs:134`). This is a static call-path proof, not an exercised production exploit. Direct `/engines/:id/calculate` correctly uses the authenticated phase and overwrites the SDK hint (`crates/noesis-api/src/lib.rs:2853-2862`). Fix the two report handlers to use authenticated phase; add phase-0/request-5 regression coverage before exposing report tools.

### 4. Personal calculations have persistent side effects

Successful calculations serialize input and enqueue reading persistence (`crates/noesis-api/src/lib.rs:2847-2851`, `:2884-2913`); the only engine excluded by `should_persist_engine_output` is `biofield-capture` (`:1966-1968`). They also auto-populate the user profile from birth data, award XP, and promote phase (`:2924-2949`). Workflows persist input/results and perform the same profile actions (`:3334-3397`). Persistence is asynchronous and failures are logged rather than reflected in the successful response.

Consequently the initial promise of no persistent birth-input storage by default is not implemented. The plugin needs an explicit product choice: disclose existing account history/profile behavior, or add a real opt-out propagated through reading, profile, usage, and auxiliary persistence paths. These tools are not purely read-only even when they return mathematical calculations. Current API calculation handlers set `client_event_id: None` (`:2898`, `:3347`), so the repository's optional idempotency mechanism is not used here; automatic retries may repeat usage/profile side effects.

Reading lookup correctly includes both reading ID and authenticated user ID (`crates/noesis-api/src/lib.rs:3765-3774`; `crates/noesis-data/src/repositories/readings_repository.rs:170-175`). This is good source evidence for row isolation; it is not a two-account live test.

### 5. Workflow success does not mean a complete reading

Individual failed, absent, or phase-gated engines are logged and omitted, while the workflow returns `Ok` with the remaining `engine_outputs` and `synthesis: None` (`crates/noesis-orchestrator/src/lib.rs:430-434`, `:501-530`). No structured omission-reason array is returned. The adapter can compare workflow `engine_ids` with returned keys and identify missing engines, but cannot reliably distinguish phase denial, outage, invalid input, or provider failure from absence alone. Expose `partial` plus missing IDs, and prefer an additive backend `engine_errors` contract. Never claim a complete synthesis merely because HTTP 200 was returned.

### 6. Capability availability is not uniform readiness or authorization

The source registry declares 19 IDs (`crates/noesis-orchestrator/src/lib.rs:255-275`). Runtime `/engines` lists registered IDs (`crates/noesis-api/src/lib.rs:3221-3224`). `/engines/capabilities` uses shared collection (`:3246-3254`): TS engine availability comes from sidecar readiness, cached for 10 seconds (`crates/noesis-api/src/capabilities.rs:17-21`, `:62-103`); native registered engines are unconditionally marked available with `required_phase: None` (`:108-121`). Neither response establishes that a specific user can execute every engine or that native provider/media dependencies are operational. Enrich discovery with `/info` phase metadata and label readiness provenance. Do not hardcode “19 available engines.”

### 7. Account linking is unimplemented in the examined source

Registered public auth routes are register/login/forgot/reset password (`crates/noesis-api/src/lib.rs:758-769`). Middleware supports Cloudflare Access identity, Selemene JWT, and Selemene API key (`crates/noesis-api/src/middleware.rs:112-235`). Targeted code search across crates, workers, packages, ts-engines, and apps found no OAuth authorization-code/PKCE/authorization-server metadata implementation. That is a source search conclusion, not proof about an external identity service.

JWTs are one-hour local tokens (`crates/noesis-auth/src/lib.rs:29`, `:306-329`), with claims carrying user/tier/permissions/phase (`:185-195`); revocation state is process memory (`:145-160`). PostgreSQL API keys are hashed, active/expiry checked, and bound to one user (`:249-303`). A customer connection needs a verified external-account-to-Selemene-user binding and refresh/revocation design. Never collect passwords or long-lived owner keys in chat. Existing Cloudflare operator access is not evidence of customer OAuth.

## Report and provider behavior

- `/witness/interpret` is an actual separate LLM dyad path, with required `live_scores`, optional birth/partner details, `engines_used`, and `llm_powered` (`handlers/witness.rs:23-65`). Missing individual engine context is silently tolerated (`:130-138`). It can persist live scores and generated text via the admin repository (`:235-265`). NVIDIA/OpenRouter are selected from environment; NVIDIA can fall back to OpenRouter (`crates/noesis-witness/src/llm.rs:93-150`). Provider error bodies are logged (`:202-206`); review privacy before enabling customer report traffic. The shown client is constructed without an explicit request timeout (`:115-120`) and completion uses `.send().await` (`:193-200`). Do not invent live scores to use this route for an ordinary birth reading.
- `/assets/generate` is explicitly a Rust seed-rendering stub for the premium pipeline, not the real TS multipass LLM runner (`handlers/assets.rs:260-295`). It gathers seeds from the first rich subject (`:56-72`) and supports a limited embedded mode allowlist (`:351-390`). Multi-subject metadata in the source pack does not establish multi-subject calculation parity. The new untracked 12-pass modes are not demonstrated as live API modes.
- General authenticated routes receive rate limits (`crates/noesis-api/src/lib.rs:1064-1072`). The free-tier monthly calculation quota is 50, fails open if billing storage is absent/errors, and increments asynchronously (`:2730-2795`). Direct engine and workflow handlers use that quota; witness/assets handlers do not invoke that helper. This is not evidence of absent general rate limiting, and provider key presence is not proof of billing activation.

## Current dirty witness work

The pre-existing diff improves Human Design activation attribution, multilingual folio text, per-pass bridge mandates, evidence/date discipline, and raises completion token headroom to `max(4096, target_max_words * 10)` (`packages/witness-pipeline/src/orchestrator/integrated.ts:168-216`, `:334-354`; `engine-facts.ts:63-75`). This changes cost/latency potential for the TS pipeline and must not be described as deployed by virtue of local files. No WIP was reverted or absorbed.

## Recommended first customer tool surface

1. `selemene_list_engines`: authenticated catalog plus capabilities, with phase/access and readiness-source labels.
2. `selemene_engine_info`: existing metadata plus a reviewed adapter input-schema registry; do not claim backend `/info` supplies schemas.
3. `selemene_prepare_reading`: local structured input-preflight with missing fields and intended persistence disclosure; no `/validate` mapping.
4. `selemene_calculate`: allowlisted non-media engines, authenticated user-derived phase, strict input schemas, provenance preserved, no blind retries.
5. `selemene_list_workflows` and `selemene_workflow_info`: existing authenticated discovery routes.
6. `selemene_run_workflow`: correct legacy request translation, expected/returned engine comparison, explicit partial results.

Keep admin, billing, raw sidecar, media capture, witness dyad, and premium report generation outside the initial tools until their individual contracts and authority are proven. Existing saved-reading tools can be added later using authenticated row ownership and explicit user intent.

## Verification performed and remaining evidence

Executed successfully on the current dirty tree:

```text
pnpm --filter @noesis/witness-pipeline exec vitest run src/orchestrator/engine-facts.test.ts src/orchestrator/folio-header.test.ts src/modes/parser.file.test.ts src/assets/factory.test.ts
4 test files passed; 12 tests passed; Vitest 2.1.9; duration 693 ms.
```

These are synthetic local formatting/factory/mode tests; they do not exercise MCP, provider APIs, account isolation, or deployed reports. Existing Rust tests reviewed but not run include capability route auth/shape tests (`crates/noesis-api/tests/capability_route_tests.rs:51-189`), assets shape/register tests (`crates/noesis-api/tests/assets_generate_contract_test.rs:11-135`), and workflow omission/phase tests (`crates/noesis-orchestrator/src/lib.rs:1045-1099`). No Rust full build or production test was started.

Required focused new coverage: low-user/high-request phase denial on both report handlers; true input-preflight semantics; two customer identity bindings and cross-user reading denial; owner-key absence; accurate partial workflow response; persistence choice enforcement; provider timeout/cost behavior; MCP initialization/discovery/call interoperability. Final connected proof requires a real ChatGPT client plus one explicitly authorized personal reading.
