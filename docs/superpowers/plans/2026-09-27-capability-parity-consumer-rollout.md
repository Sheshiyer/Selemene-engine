# Capability Parity Consumer Rollout Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make the `contracts/v1` capability-discovery and provenance surface (built on branch `codex/selemene-task3-task4-capability-parity`, PRs Sheshiyer/Selemene-engine#1486 and #1487) actually reachable and consumed by the five sibling clients: urania-137, noesis-raycast, antahkarana, sankalpa, noesismirror-web.

**Architecture:** Phase A closes three engine-side gaps that currently make the branch's work invisible to every client: (1) the `confidence` field exists only in the TypeScript type, not in the JSON schema, Rust struct, or SDKs; (2) the Rust bridge drops the TypeScript `provenance` block and the API always serialises `provenance: null`; (3) the only Rust capability route is admin-gated (`admin:system:read`), which none of the clients' `nk_` keys can reach. Phase A adds a public `GET /api/v1/engines/capabilities`, passes provenance through the bridge, aligns the contract, and exposes both in the two SDKs. Phase B then wires each client to the new surface using that client's existing transport pattern (raw fetch, Tauri command, Electron IPC, vendored SDK), with a test in each client's existing runner.

**Tech Stack:** Rust (axum, serde, cargo), TypeScript (bun, vitest, node:test), Elysia ts-engines, Cloudflare Pages Functions, Tauri 2 (reqwest), Electron IPC, Raycast API.

**Spec:** `docs/plans/selemene-engine/RUNTIME-CAPABILITY-EVIDENCE.md` (slices 1 to 3) plus the survey findings recorded in the "Survey Findings" section below. This plan is the spec's consumer-side continuation.

## Survey Findings (2026-09-27, read-only)

| Repo | Stack / runner | How it reaches Selemene today | Capability / provenance today |
|---|---|---|---|
| Selemene-engine (this branch) | Rust + ts-engines (bun) + python | n/a | TS `/engines/capabilities` done; Rust route is admin-only; bridge drops provenance; `confidence` not in schema/Rust/SDKs |
| urania-137 (`main`) | React 19 + Vite, Cloudflare Pages Functions, vitest 4 | Browser to `/api/selemene/*` proxy (`functions/lib/engine-proxy.ts`) which injects `X-API-Key` and forwards to `https://selemene.tryambakam.space` | none; `EngineStatusPanel.tsx:29` fakes availability from health strings |
| noesis-raycast (`codex/noesis-store-resubmission-cleanup`) | Raycast ext, bun test + node:test | `SelemeneApiClient` in `src/lib/api.ts`, `X-API-Key`, base `https://selemene.tryambakam.space` | none |
| antahkarana (`codex/antahkarana-foundation-recovery`) | Tauri 2, Rust reqwest, bun contract scripts | Tauri commands in `src-tauri/src/lib.rs`, `X-API-Key` from keychain, `SELEMENE_API_URL` at `lib.rs:37` | none; roster hardcodes 18 engine ids, `/api/v1/engines` commented out |
| sankalpa (`archive-20260728`, TOMBSTONED) | Electron + React, vitest | `DesktopEngineGateway` wraps vendored `@selemene/engine-sdk` 0.1.0 tarball; `apiUrl` set so every call is `/api/v1/...` | none; SDK has no `capabilities()` |
| noesismirror-web (`version-0.2`, local-only) | Vite + R3F, no test runner | **No Selemene integration exists at all** | none |

Sankalpa's `TOMBSTONE.md` (2026-07-28, ruling R4) says "read it, port from it, do not ship it" and names `twc-shell` as successor. `twc-shell` does not exist under `tryambakam-noesis/`. Sankalpa tasks below are therefore scoped to keeping its gateway contract-current so the port has a correct baseline; nothing here ships sankalpa.

## Global Constraints

- Contract version string is exactly `v1`; `contract_version` is `"v1"` on every capability and result record.
- `availability` enum is exactly `declared | available | degraded | unavailable` (`contracts/v1/schemas/engine-capability.schema.json`).
- `runtime_kind` enum is exactly `native | typescript | python | database-conditional | composed`.
- `confidence`, when present, is a number in `[0, 1]`.
- No capability or provenance code path may call a provider, generation API, database, or remote beyond the existing bridge readiness self-check (spec "Boundary").
- All public client-facing Selemene calls use `X-API-Key` (antahkarana `lib.rs:353`, raycast `api.ts:559`, urania `engine-proxy.ts:49`). Admin JWT routes are not a client surface.
- Every consumer test uses an injected or stubbed `fetch`; no test hits the network.
- Rust: `cargo build --workspace --locked` must stay green; `pnpm run gate:contracts` must pass after Phase A.
- Do not push, deploy, or mutate GitHub issues from any task. Commits are local; PR creation is a separate, explicitly authorised step.

## Review Focus

1. **TypeScript engine result carrying `confidence` crosses the Rust API.** Today `Provenance` is `deny_unknown_fields` with no `confidence`; if anything ever deserialised it, tarot results would 500. Task A1's Rust round-trip test on the fixture pins this.
2. **Client hits `/api/v1/engines/capabilities` against a deployment that predates Phase A and gets 404.** Every client must degrade to "declared" (metadata-only) rather than crash. Tasks B1, B2, B3, B4 each pin a 404 test.
3. **Capability array is empty because bridge readiness errored.** The Rust route must still return native engines as `available` and bridge engines as `unavailable`, never an empty list. Task A3 test pins this.
4. **A capability row arrives with an `availability` value outside the enum (future server).** Clients must normalise unknown values to `declared`, not throw. Tasks B1 and B2 pin this.
5. **Python sidecar health lacks `capability_status` (old sidecar).** `engine-sdk` `health()` must type it optional and callers must not assume presence. Task A4 pins this.

---

# Phase A: Selemene-engine

All Phase A work happens in `/Volumes/madara/2026/Projects/tryambakam-noesis/Selemene-engine` on a new branch `codex/selemene-capability-consumer-surface` created from `codex/selemene-task3-task4-capability-parity`.

```bash
cd /Volumes/madara/2026/Projects/tryambakam-noesis/Selemene-engine
git checkout -b codex/selemene-capability-consumer-surface codex/selemene-task3-task4-capability-parity
```

### Task A1: Align `confidence` across contract schema, fixture, Rust, and both SDKs

**Files:**
- Modify: `contracts/v1/schemas/provenance.schema.json`
- Modify: `contracts/v1/fixtures/engine-result.json`
- Modify: `crates/noesis-core/src/contract.rs:137-148` (`Provenance`)
- Modify: `packages/noesis-engine-sdk/src/contract-v1.ts:16-24` (`ContractProvenance`)
- Modify: `packages/noesis-sdk-ts/src/index.ts` (`ContractProvenance`, near line 60-75)
- Test: `crates/noesis-core/tests/contract_v1_authority.rs`
- Test: `packages/noesis-engine-sdk/tests/contract-v1.test.ts`

**Interfaces:**
- Consumes: existing `ContractProvenance` in `ts-engines/src/types/engine.ts:137-146` (already has `confidence?: number`).
- Produces: `Provenance.confidence: Option<f64>` (Rust), `ContractProvenance.confidence?: number` (both SDKs), schema property `confidence: { type: number, minimum: 0, maximum: 1 }`.

- [ ] **Step 1: Write the failing Rust test**

Append to `crates/noesis-core/tests/contract_v1_authority.rs`:

```rust
#[test]
fn canonical_result_provenance_round_trips_confidence() {
    let source = include_str!("../../../contracts/v1/fixtures/engine-result.json");
    let result: noesis_core::contract::ContractEngineResult =
        serde_json::from_str(source).expect("fixture must deserialise");
    let provenance = result.provenance.expect("fixture carries provenance");
    assert_eq!(provenance.confidence, Some(1.0));
    let json = serde_json::to_value(&provenance).unwrap();
    assert_eq!(json["confidence"], 1.0);
}
```

- [ ] **Step 2: Run it to verify it fails**

Run: `cargo test -p noesis-core --test contract_v1_authority canonical_result_provenance_round_trips_confidence --locked`
Expected: FAIL, compile error `no field confidence on type Provenance`.

- [ ] **Step 3: Add the field to the Rust struct**

In `crates/noesis-core/src/contract.rs`, inside `pub struct Provenance`, after `provider_id`:

```rust
    /// Confidence in the result, 0-1. Populated per-engine; omitted where not yet computed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidence: Option<f64>,
```

Because `Provenance` derives `Eq`, change its derive line to `#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]` (drop `Eq`; `f64` is not `Eq`). Fix any `Provenance { .. }` literals in `crates/` by adding `confidence: None` (search: `grep -rn "Provenance {" crates --include='*.rs'`).

- [ ] **Step 4: Update schema and fixture**

`contracts/v1/schemas/provenance.schema.json`, add under `properties`:

```json
    "confidence": { "type": "number", "minimum": 0, "maximum": 1 }
```

`contracts/v1/fixtures/engine-result.json`: inside the existing `"provenance": { ... }` object add `"confidence": 1`.

- [ ] **Step 5: Run the Rust test and the whole authority suite**

Run: `cargo test -p noesis-core --test contract_v1_authority --locked`
Expected: all PASS including the new test.

- [ ] **Step 6: Write the failing engine-sdk test**

Append to `packages/noesis-engine-sdk/tests/contract-v1.test.ts`:

```ts
import { describe, expect, it } from 'bun:test'
import type { ContractProvenance } from '../src/contract-v1'

describe('ContractProvenance confidence', () => {
  it('accepts an optional 0-1 confidence', () => {
    const p: ContractProvenance = {
      runtime_kind: 'typescript',
      implementation_version: '1.0.0',
      cached: false,
      fallback_used: false,
      confidence: 1,
    }
    expect(p.confidence).toBe(1)
  })
})
```

- [ ] **Step 7: Run it to verify it fails**

Run: `cd packages/noesis-engine-sdk && bun run typecheck`
Expected: FAIL, `Object literal may only specify known properties, and 'confidence' does not exist`.

- [ ] **Step 8: Add `confidence` to both SDK types**

`packages/noesis-engine-sdk/src/contract-v1.ts`, inside `export interface ContractProvenance`, after `provider_id?: string`:

```ts
  /** Confidence in the result, 0-1. Omitted where not yet computed. */
  confidence?: number
```

`packages/noesis-sdk-ts/src/index.ts`, inside its `export interface ContractProvenance` (search `export interface ContractProvenance`), add the identical two lines.

- [ ] **Step 9: Run both SDK gates**

Run: `pnpm --filter @selemene/engine-sdk test && pnpm --filter @selemene/engine-sdk typecheck && pnpm --filter @noesis/sdk test && pnpm --filter @noesis/sdk typecheck`
Expected: PASS.

- [ ] **Step 10: Commit**

```bash
git add contracts/v1 crates/noesis-core packages/noesis-engine-sdk packages/noesis-sdk-ts
git commit -m "feat(contract): add optional provenance.confidence across schema, Rust, and SDKs"
```

---

### Task A2: Pass TypeScript provenance through the Rust bridge and API envelope

**Files:**
- Modify: `crates/noesis-bridge/src/ts_client.rs:40-56` (`TsEngineResponse`)
- Modify: `crates/noesis-bridge/src/lib.rs:515-548` (bridge `calculate` conversion)
- Modify: `crates/noesis-api/src/lib.rs:1527-1550` (`impl From<EngineOutput> for ApiEngineOutputResponse`)
- Test: `crates/noesis-bridge/src/lib.rs` (existing `#[cfg(test)]` block near line 1009)
- Test: `crates/noesis-api/tests/integration_tests.rs`

**Interfaces:**
- Consumes: `noesis_core::contract::Provenance` with `confidence` from Task A1; `CalculationMetadata { backend, cached, engine_version }` from `crates/noesis-core/src/types.rs:153`.
- Produces: API calculate responses always include `provenance` (never `null`). Bridge engines carry the sidecar's provenance verbatim; native engines get a derived one: `runtime_kind: native`, `implementation_version: metadata.engine_version`, `cached: metadata.cached`, `fallback_used: false`.

Design note: `EngineOutput` has 63 struct-literal constructors across engine crates, so it does not gain a field. The bridge stashes the sidecar provenance at `result["provenance"]` (the same technique already used for `generated_image`), and the API envelope lifts it back out.

- [ ] **Step 1: Write the failing bridge unit test**

In `crates/noesis-bridge/src/lib.rs` `#[cfg(test)] mod tests`, add:

```rust
    #[test]
    fn ts_response_preserves_provenance_block() {
        let response: crate::ts_client::TsEngineResponse = serde_json::from_value(json!({
            "engine_id": "tarot",
            "result": {"cards": []},
            "witness_prompts": [],
            "calculated_at": "2026-09-27T00:00:00.000Z",
            "processing_time_ms": 1.5,
            "provenance": {
                "runtime_kind": "typescript",
                "implementation_version": "1.0.0",
                "cached": false,
                "fallback_used": false,
                "confidence": 1
            }
        }))
        .expect("response with provenance must deserialise");
        let prov = response.provenance.expect("provenance retained");
        assert_eq!(prov["confidence"], 1);
    }
```

- [ ] **Step 2: Run it to verify it fails**

Run: `cargo test -p noesis-bridge ts_response_preserves_provenance_block --locked`
Expected: FAIL, `no field provenance`.

- [ ] **Step 3: Add the field and merge it into `result`**

`crates/noesis-bridge/src/ts_client.rs`, in `TsEngineResponse` after `generated_audio`:

```rust
    /// contracts/v1 provenance block emitted by the sidecar (optional; older sidecars omit it).
    #[serde(default)]
    pub provenance: Option<Value>,
```

`crates/noesis-bridge/src/lib.rs`, inside the `if let Some(object) = result.as_object_mut()` block after the `generated_audio` insert:

```rust
            if let Some(provenance) = ts_response.provenance {
                object.insert("provenance".to_string(), provenance);
            }
```

- [ ] **Step 4: Run bridge tests**

Run: `cargo test -p noesis-bridge --locked`
Expected: PASS.

- [ ] **Step 5: Write the failing API integration test**

Append to `crates/noesis-api/tests/integration_tests.rs` (reuse that file's existing token helper; it already has `test_calculate_panchanga_success` at line 278 as the pattern):

```rust
#[tokio::test]
async fn test_calculate_native_engine_reports_derived_provenance() {
    let token = generate_test_token();
    let body = serde_json::json!({
        "contract_version": "v1",
        "consciousness_level": 1,
        "parameters": {},
        "birth_data": {"date": "1990-01-01", "time": "12:00", "latitude": 12.97, "longitude": 77.59, "timezone": "Asia/Kolkata"}
    });
    let (status, resp) = common::make_authenticated_request(
        "POST", "/api/v1/engines/panchanga/calculate", &token, Some(body),
    ).await;
    assert_eq!(status, StatusCode::OK, "body={resp:?}");
    let prov = &resp["provenance"];
    assert_eq!(prov["runtime_kind"], "native", "body={resp:?}");
    assert_eq!(prov["fallback_used"], false);
    assert!(prov["implementation_version"].is_string());
    assert!(resp["result"].get("provenance").is_none(), "provenance must be lifted out of result");
}
```

If `generate_test_token` is named differently in that file, use the helper the neighbouring `test_calculate_panchanga_success` uses.

- [ ] **Step 6: Run it to verify it fails**

Run: `cargo test -p noesis-api --test integration_tests test_calculate_native_engine_reports_derived_provenance --locked`
Expected: FAIL, `provenance` is `null`.

- [ ] **Step 7: Lift or derive provenance in the API envelope**

In `crates/noesis-api/src/lib.rs`, replace the body of `impl From<EngineOutput> for ApiEngineOutputResponse` so that before `Self { ... }`:

```rust
        let mut output = output;
        let lifted: Option<noesis_core::contract::Provenance> = output
            .result
            .as_object_mut()
            .and_then(|o| o.remove("provenance"))
            .and_then(|v| serde_json::from_value(v).ok());
        let provenance = Some(lifted.unwrap_or_else(|| noesis_core::contract::Provenance {
            runtime_kind: match output.metadata.backend.as_str() {
                "typescript" => noesis_core::contract::RuntimeKind::TypeScript,
                "python" => noesis_core::contract::RuntimeKind::Python,
                _ => noesis_core::contract::RuntimeKind::Native,
            },
            implementation_version: if output.metadata.engine_version.is_empty() {
                env!("CARGO_PKG_VERSION").to_string()
            } else {
                output.metadata.engine_version.clone()
            },
            cached: output.metadata.cached,
            fallback_used: false,
            backend_id: None,
            provider_id: None,
            confidence: None,
        }));
```

and set `provenance,` in the `Self { ... }` literal instead of `provenance: None,`. Keep `generated_image` and `generated_audio` extraction as-is (they read `output.result` before the `Self` literal; make sure they run before the `remove`, or after, it does not matter since they use different keys).

- [ ] **Step 8: Run the API tests and contract gate**

Run: `cargo test -p noesis-api --test integration_tests test_calculate_ --locked && cargo test -p noesis-api --test openapi_schema_tests --locked && pnpm run gate:contracts`
Expected: PASS. If `openapi_schema_tests` snapshots the envelope, regenerate per that test's instructions (`provenance` is no longer nullable in the example).

- [ ] **Step 9: Commit**

```bash
git add crates/noesis-bridge crates/noesis-api
git commit -m "feat(api): carry sidecar provenance through the bridge and always emit contract-v1 provenance"
```

---

### Task A3: Public `GET /api/v1/engines/capabilities` (API-key surface)

**Files:**
- Create: `crates/noesis-api/src/capabilities.rs`
- Modify: `crates/noesis-api/src/lib.rs:1021` (route registration next to `/engines`), plus `mod capabilities;`
- Modify: `crates/noesis-api/src/handlers/admin.rs:4634-4688` (`engine_capabilities` delegates to shared collector)
- Test: `crates/noesis-api/tests/capability_route_tests.rs`

**Interfaces:**
- Consumes: `state.orchestrator.list_engines() -> Vec<String>` (used by `list_engines_handler` at `lib.rs:3137`); `state.bridge().engines()` and `state.bridge().readiness_status()` (used in admin handler).
- Produces: `pub async fn collect_capabilities(state: &AppState) -> Vec<noesis_core::contract::EngineCapability>` and route `GET /api/v1/engines/capabilities` returning `{ "capabilities": [...], "count": n }` (same envelope as the TypeScript route in `ts-engines/src/server/app.ts:131-141`).

- [ ] **Step 1: Write the failing route tests**

Append to `crates/noesis-api/tests/capability_route_tests.rs`:

```rust
#[tokio::test]
async fn test_public_capabilities_route_is_reachable_with_user_token() {
    let token = generate_user_token();
    let (status, body) =
        common::make_authenticated_request("GET", "/api/v1/engines/capabilities", &token, None)
            .await;
    assert_eq!(status, StatusCode::OK, "body={body:?}");
    let caps = body["capabilities"].as_array().expect("capabilities array");
    assert_eq!(body["count"].as_u64().unwrap() as usize, caps.len());
    assert!(!caps.is_empty(), "must never be empty even if bridge readiness fails");
    let native = caps.iter().find(|c| c["engine_id"] == "panchanga").expect("native engine listed");
    assert_eq!(native["runtime_kind"], "native");
    assert_eq!(native["availability"], "available");
    for id in KNOWN_TS_ENGINES {
        let row = caps.iter().find(|c| c["engine_id"] == id).unwrap_or_else(|| panic!("{id} missing"));
        assert_eq!(row["runtime_kind"], "typescript");
        assert!(matches!(row["availability"].as_str(), Some("available" | "unavailable")));
    }
}

#[tokio::test]
async fn test_public_capabilities_route_requires_auth() {
    let (status, _) =
        common::make_unauthenticated_request("GET", "/api/v1/engines/capabilities", None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}
```

- [ ] **Step 2: Run to verify they fail**

Run: `cargo test -p noesis-api --test capability_route_tests test_public_capabilities --locked`
Expected: FAIL with 404.

- [ ] **Step 3: Create the shared collector**

`crates/noesis-api/src/capabilities.rs`:

```rust
//! Shared contract-v1 capability collection for the public and admin routes.
//! Uses only the orchestrator registry and the bridge readiness self-check.

use std::collections::{HashMap, HashSet};

use noesis_core::contract::{CapabilityAvailability, ContractVersion, EngineCapability, RuntimeKind};

use crate::AppState;

pub async fn collect_capabilities(state: &AppState) -> Vec<EngineCapability> {
    let readiness = state.bridge().readiness_status().await;
    let healthy_by_id: HashMap<String, bool> = match &readiness {
        Ok(status) => status.engines.iter().map(|e| (e.engine_id.clone(), e.healthy)).collect(),
        Err(_) => HashMap::new(),
    };

    let mut rows: Vec<EngineCapability> = state
        .bridge()
        .engines()
        .iter()
        .map(|engine| {
            let engine_id = engine.engine_id().to_string();
            let availability = match healthy_by_id.get(&engine_id) {
                Some(true) => CapabilityAvailability::Available,
                _ => CapabilityAvailability::Unavailable,
            };
            EngineCapability {
                contract_version: ContractVersion::V1,
                engine_id,
                display_name: engine.engine_name().to_string(),
                availability,
                runtime_kind: RuntimeKind::TypeScript,
                dependencies: Vec::new(),
                required_phase: Some(engine.required_phase()),
                implementation_version: None,
            }
        })
        .collect();

    let bridge_ids: HashSet<String> = rows.iter().map(|r| r.engine_id.clone()).collect();
    for engine_id in state.orchestrator.list_engines() {
        if bridge_ids.contains(&engine_id) {
            continue;
        }
        rows.push(EngineCapability {
            contract_version: ContractVersion::V1,
            engine_id: engine_id.clone(),
            display_name: engine_id,
            availability: CapabilityAvailability::Available,
            runtime_kind: RuntimeKind::Native,
            dependencies: Vec::new(),
            required_phase: None,
            implementation_version: Some(env!("CARGO_PKG_VERSION").to_string()),
        });
    }
    rows.sort_by(|a, b| a.engine_id.cmp(&b.engine_id));
    rows
}
```

- [ ] **Step 4: Register the public handler**

In `crates/noesis-api/src/lib.rs` add `mod capabilities;` near the other module declarations, then next to line 1021:

```rust
        .route("/engines", get(list_engines_handler))
        .route("/engines/capabilities", get(engine_capabilities_handler))
```

and add the handler near `list_engines_handler`:

```rust
#[derive(Serialize, ToSchema)]
struct EngineCapabilitiesResponse {
    capabilities: Vec<noesis_core::contract::EngineCapability>,
    count: usize,
}

/// GET /api/v1/engines/capabilities -- contract-v1 capability discovery (API-key surface)
async fn engine_capabilities_handler(State(state): State<AppState>) -> Json<EngineCapabilitiesResponse> {
    let capabilities = capabilities::collect_capabilities(&state).await;
    let count = capabilities.len();
    Json(EngineCapabilitiesResponse { capabilities, count })
}
```

If `EngineCapability` does not derive `ToSchema`, add `#[cfg_attr(feature = "openapi", derive(ToSchema))]` to `EngineCapability`, `CapabilityAvailability`, and `RuntimeKind` in `crates/noesis-core/src/contract.rs` (`RuntimeKind` may already have it; check).

- [ ] **Step 5: Make the admin handler delegate**

Replace the body of `engine_capabilities` in `handlers/admin.rs` after the permission check with:

```rust
    let capabilities = crate::capabilities::collect_capabilities(&state).await;
    Ok((StatusCode::OK, Json(capabilities)).into_response())
```

The existing admin test `test_capability_route_returns_contract_v1_shape_for_all_ts_engines` asserts `capabilities.len() == KNOWN_TS_ENGINES.len()`; native engines now appear too. Update that assertion to filter `runtime_kind == "typescript"` before comparing length, and remove the now-unused `HashMap` import.

- [ ] **Step 6: Run the route tests, workspace build, and contract gate**

Run: `cargo test -p noesis-api --test capability_route_tests --locked && cargo build --workspace --locked && pnpm run gate:contracts`
Expected: PASS.

- [ ] **Step 7: Commit**

```bash
git add crates/noesis-api crates/noesis-core
git commit -m "feat(api): public GET /api/v1/engines/capabilities with native + bridge rows"
```

---

### Task A4: SDK capability methods and Python health typing

**Files:**
- Modify: `packages/noesis-engine-sdk/src/client.ts:135-142` (add `capabilities()`), `packages/noesis-engine-sdk/src/types.ts:138-144` (`PythonSidecarHealthResponse`)
- Modify: `packages/noesis-engine-sdk/src/index.ts` (export `ContractEngineCapability` if not already)
- Modify: `packages/noesis-sdk-ts/src/index.ts` (add `EngineCapability`, `EngineCapabilitiesResponse`, `NoesisClient.listCapabilities()`)
- Modify: `packages/noesis-engine-sdk/package.json` version `0.1.0` to `0.2.0`; `packages/noesis-sdk-ts/package.json` version `3.3.1` to `3.4.0`
- Test: `packages/noesis-engine-sdk/tests/engine-client.test.ts`
- Test: `packages/noesis-sdk-ts/src/index.test.ts`

**Interfaces:**
- Produces (engine-sdk): `EngineClient.capabilities(): Promise<{ capabilities: ContractEngineCapability[]; count: number }>` hitting `${apiUrl}/api/v1/engines/capabilities` when `apiUrl` is configured, else `${tsUrl}/engines/capabilities`.
- Produces (noesis-sdk-ts): `NoesisClient.listCapabilities(options?): Promise<EngineCapabilitiesResponse>` at `/api/v1/engines/capabilities`.
- Produces: `PythonSidecarHealthResponse.capability_status?: 'available' | 'degraded' | 'unavailable'`.

- [ ] **Step 1: Write failing engine-sdk tests**

Append to `packages/noesis-engine-sdk/tests/engine-client.test.ts` (reuse that file's `mockFetch` helper):

```ts
describe('capabilities()', () => {
  const row = {
    contract_version: 'v1', engine_id: 'tarot', display_name: 'Tarot',
    availability: 'available', runtime_kind: 'typescript', dependencies: [], required_phase: 1,
  }
  it('uses the ts server route when no apiUrl is set', async () => {
    const { fetchImpl, calls } = mockFetch(() =>
      new Response(JSON.stringify({ capabilities: [row], count: 1 }), { status: 200 }))
    const client = new EngineClient({ tsEnginesUrl: 'http://ts.local:3001', fetchImpl })
    const res = await client.capabilities()
    expect(calls[0].url).toBe('http://ts.local:3001/engines/capabilities')
    expect(res.count).toBe(1)
    expect(res.capabilities[0].availability).toBe('available')
  })
  it('uses the api route when apiUrl is set', async () => {
    const { fetchImpl, calls } = mockFetch(() =>
      new Response(JSON.stringify({ capabilities: [], count: 0 }), { status: 200 }))
    const client = new EngineClient({ apiUrl: 'https://api.local', fetchImpl })
    await client.capabilities()
    expect(calls[0].url).toBe('https://api.local/api/v1/engines/capabilities')
  })
  it('surfaces 404 as EngineSdkError so callers can fall back to declared', async () => {
    const { fetchImpl } = mockFetch(() => new Response('not found', { status: 404 }))
    const client = new EngineClient({ apiUrl: 'https://old.local', fetchImpl })
    await expect(client.capabilities()).rejects.toBeInstanceOf(EngineSdkError)
  })
})
```

- [ ] **Step 2: Run to verify failure**

Run: `cd packages/noesis-engine-sdk && bun test tests/engine-client.test.ts`
Expected: FAIL, `client.capabilities is not a function`.

- [ ] **Step 3: Implement in engine-sdk**

`packages/noesis-engine-sdk/src/client.ts`, after `health()`:

```ts
  /** contracts/v1 capability discovery (api route when P4 configured, else ts server). */
  async capabilities(): Promise<{ capabilities: ContractEngineCapability[]; count: number }> {
    const url = this.apiUrl
      ? `${this.apiUrl}/api/v1/engines/capabilities`
      : `${this.tsUrl}/engines/capabilities`
    return this.request('GET', url)
  }
```

Add `ContractEngineCapability` to the `contract-v1.js` type import at the top of `client.ts`. In `types.ts` `PythonSidecarHealthResponse` add `capability_status?: 'available' | 'degraded' | 'unavailable'` and `mediapipe_available?: boolean`. Ensure `index.ts` exports type `ContractEngineCapability` (it is defined in `contract-v1.ts:66`; add to the `export type { ... } from './contract-v1.js'` list if absent).

- [ ] **Step 4: Run engine-sdk gate**

Run: `pnpm --filter @selemene/engine-sdk test && pnpm --filter @selemene/engine-sdk typecheck`
Expected: PASS.

- [ ] **Step 5: Write failing noesis-sdk-ts test**

In `packages/noesis-sdk-ts/src/index.test.ts` inside `describe("NoesisClient")`:

```ts
  it("lists contract-v1 capabilities", async () => {
    const fetchMock = vi.fn(async () =>
      new Response(JSON.stringify({ capabilities: [{
        contract_version: "v1", engine_id: "tarot", display_name: "Tarot",
        availability: "available", runtime_kind: "typescript", dependencies: [],
      }], count: 1 }), { status: 200 }),
    );
    vi.stubGlobal("fetch", fetchMock);
    const client = new NoesisClient("https://example.com", { authToken: "token" });
    const res = await client.listCapabilities();
    expect(res.count).toBe(1);
    expect(res.capabilities[0].engine_id).toBe("tarot");
    expect(fetchMock.mock.calls[0][0]).toBe("https://example.com/api/v1/engines/capabilities");
  });
```

- [ ] **Step 6: Run to verify failure**

Run: `pnpm --filter @noesis/sdk test`
Expected: FAIL, `listCapabilities is not a function`.

- [ ] **Step 7: Implement in noesis-sdk-ts**

In `packages/noesis-sdk-ts/src/index.ts`, next to `EngineInfo`:

```ts
export type CapabilityAvailability = "declared" | "available" | "degraded" | "unavailable";
export type RuntimeKind = "native" | "typescript" | "python" | "database-conditional" | "composed";

export interface EngineCapability {
  contract_version: "v1";
  engine_id: string;
  display_name: string;
  availability: CapabilityAvailability;
  runtime_kind: RuntimeKind;
  dependencies: string[];
  required_phase?: number;
  implementation_version?: string;
}

export interface EngineCapabilitiesResponse {
  capabilities: EngineCapability[];
  count: number;
}
```

and in `NoesisClient` after `listEngines`:

```ts
  /** GET /api/v1/engines/capabilities -- contract-v1 runtime capability discovery */
  async listCapabilities(options?: RequestOptions): Promise<EngineCapabilitiesResponse> {
    return this.request<EngineCapabilitiesResponse>("/api/v1/engines/capabilities", { method: "GET" }, options);
  }
```

- [ ] **Step 8: Bump versions, run gates, commit**

Set `"version": "0.2.0"` in `packages/noesis-engine-sdk/package.json` and `"version": "3.4.0"` in `packages/noesis-sdk-ts/package.json`.

Run: `pnpm run gate:contracts`
Expected: PASS.

```bash
git add packages/noesis-engine-sdk packages/noesis-sdk-ts
git commit -m "feat(sdk): capabilities() / listCapabilities() and python capability_status typing"
```

---

### Task A5: Build the vendored tarball for sankalpa and record evidence

**Files:**
- Create: `/Volumes/madara/2026/Projects/tryambakam-noesis/sankalpa/vendor/selemene-engine-sdk-0.2.0.tgz`
- Modify: `docs/plans/selemene-engine/RUNTIME-CAPABILITY-EVIDENCE.md` (append "Slice 4")

- [ ] **Step 1: Pack the SDK**

```bash
cd /Volumes/madara/2026/Projects/tryambakam-noesis/Selemene-engine/packages/noesis-engine-sdk
pnpm build
pnpm pack --pack-destination /Volumes/madara/2026/Projects/tryambakam-noesis/sankalpa/vendor/
ls -la /Volumes/madara/2026/Projects/tryambakam-noesis/sankalpa/vendor/
```

Expected: `selemene-engine-sdk-0.2.0.tgz` exists (the pack name follows the package name `@selemene/engine-sdk` and becomes `selemene-engine-sdk-0.2.0.tgz`).

- [ ] **Step 2: Append Slice 4 to the evidence doc**

Add a section `## Slice 4: Consumer surface (2026-09-27)` listing: `confidence` alignment (A1), bridge provenance pass-through (A2), public route (A3), SDK methods and versions (A4), and the RED/GREEN command lines actually run with their pass counts. Copy exact numbers from the terminal; do not estimate.

- [ ] **Step 3: Commit**

```bash
cd /Volumes/madara/2026/Projects/tryambakam-noesis/Selemene-engine
git add docs/plans/selemene-engine/RUNTIME-CAPABILITY-EVIDENCE.md
git commit -m "docs(capability): slice 4 consumer-surface evidence"
```

---

# Phase B: Consumers

Each consumer task is independent of the others and depends only on Phase A being merged and deployed (until deployed, the 404 fallback paths are what run in production). Work each in its own repo on a new branch `codex/<repo>-capability-parity` from that repo's current branch listed in the survey table.

### Task B1: urania-137 capability client, typed provenance, live status panel

**Files:**
- Modify: `src/types/index.ts:77` (`EngineResult`), append `EngineCapability` types
- Modify: `src/lib/selemeneApi.ts:73` (add `fetchCapabilities`)
- Modify: `src/hooks/useEngineStatus.ts:17`
- Modify: `src/components/panels/EngineStatusPanel.tsx:29`
- Modify: `scripts/verify/golden-parity.mjs` (add capabilities probe)
- Test: `src/lib/__tests__/selemeneApi.test.ts`

**Interfaces:**
- Consumes: proxy base `PROXY_BASE` (`'/api/selemene'`), `forwardToEngineFromEnv` already forwards any `/api/selemene/*` path so no server change is needed.
- Produces: `fetchCapabilities(): Promise<EngineCapability[]>` that returns `[]` on 404; `normalizeAvailability(value: unknown): CapabilityAvailability`; `EngineStatus.capabilities: EngineCapability[]`.

- [ ] **Step 1: Write failing tests**

Append to `src/lib/__tests__/selemeneApi.test.ts` (follow the file's existing `vi.stubGlobal('fetch', ...)` pattern):

```ts
import { fetchCapabilities, normalizeAvailability } from '../selemeneApi'

describe('fetchCapabilities', () => {
  it('GETs /api/selemene/api/v1/engines/capabilities and returns rows', async () => {
    const fetchMock = vi.fn(async () => new Response(JSON.stringify({
      capabilities: [{ contract_version: 'v1', engine_id: 'tarot', display_name: 'Tarot',
        availability: 'available', runtime_kind: 'typescript', dependencies: [] }],
      count: 1,
    }), { status: 200 }))
    vi.stubGlobal('fetch', fetchMock)
    const rows = await fetchCapabilities()
    expect(fetchMock.mock.calls[0][0]).toBe('/api/selemene/api/v1/engines/capabilities')
    expect(rows).toHaveLength(1)
    expect(rows[0].availability).toBe('available')
  })
  it('returns [] on 404 (pre-capability deployment)', async () => {
    vi.stubGlobal('fetch', vi.fn(async () => new Response('nope', { status: 404 })))
    expect(await fetchCapabilities()).toEqual([])
  })
  it('normalises unknown availability to declared', () => {
    expect(normalizeAvailability('available')).toBe('available')
    expect(normalizeAvailability('turbo')).toBe('declared')
    expect(normalizeAvailability(undefined)).toBe('declared')
  })
})
```

- [ ] **Step 2: Run to verify failure**

Run: `cd /Volumes/madara/2026/Projects/tryambakam-noesis/urania-137 && npx vitest run src/lib/__tests__/selemeneApi.test.ts`
Expected: FAIL, `fetchCapabilities` is not exported.

- [ ] **Step 3: Add types**

`src/types/index.ts`, after `EngineResult` (line 77):

```ts
export type CapabilityAvailability = 'declared' | 'available' | 'degraded' | 'unavailable'
export type RuntimeKind = 'native' | 'typescript' | 'python' | 'database-conditional' | 'composed'

export interface EngineProvenance {
  runtime_kind: RuntimeKind
  implementation_version: string
  cached: boolean
  fallback_used: boolean
  backend_id?: string
  provider_id?: string
  confidence?: number
}

export interface EngineCapability {
  contract_version: 'v1'
  engine_id: string
  display_name: string
  availability: CapabilityAvailability
  runtime_kind: RuntimeKind
  dependencies: string[]
  required_phase?: number
  implementation_version?: string
}
```

and add `provenance?: EngineProvenance` to `EngineResult`. Add `capabilities: EngineCapability[]` to `EngineStatus` (line 280).

- [ ] **Step 4: Implement `fetchCapabilities`**

`src/lib/selemeneApi.ts`, next to `fetchEngines`:

```ts
const AVAILABILITY: readonly CapabilityAvailability[] = ['declared', 'available', 'degraded', 'unavailable']

export function normalizeAvailability(value: unknown): CapabilityAvailability {
  return AVAILABILITY.includes(value as CapabilityAvailability) ? (value as CapabilityAvailability) : 'declared'
}

export async function fetchCapabilities(): Promise<EngineCapability[]> {
  const res = await fetch(`${PROXY_BASE}/api/v1/engines/capabilities`, { headers: { Accept: 'application/json' } })
  if (res.status === 404) return []
  if (!res.ok) throw new Error(`capabilities ${res.status}`)
  const body = (await res.json()) as { capabilities?: unknown[] }
  return (body.capabilities ?? []).map((raw) => {
    const row = raw as Record<string, unknown>
    return {
      contract_version: 'v1',
      engine_id: String(row.engine_id),
      display_name: String(row.display_name ?? row.engine_id),
      availability: normalizeAvailability(row.availability),
      runtime_kind: (row.runtime_kind as RuntimeKind) ?? 'native',
      dependencies: Array.isArray(row.dependencies) ? (row.dependencies as string[]) : [],
      required_phase: typeof row.required_phase === 'number' ? row.required_phase : undefined,
      implementation_version: typeof row.implementation_version === 'string' ? row.implementation_version : undefined,
    }
  })
}
```

Import `CapabilityAvailability`, `EngineCapability`, `RuntimeKind` from `../types`.

- [ ] **Step 5: Run tests**

Run: `npx vitest run src/lib/__tests__/selemeneApi.test.ts`
Expected: PASS.

- [ ] **Step 6: Wire hook and panel**

`src/hooks/useEngineStatus.ts`: add `fetchCapabilities()` to the existing `Promise.all` alongside health/ready/engines and put the result in `capabilities`. `src/components/panels/EngineStatusPanel.tsx:29`: replace the local `availability(value)` helper with a lookup `capabilities.find(c => c.engine_id === id)?.availability ?? 'declared'` and render `degraded` in the same amber style `unavailable` currently uses for errors.

Run: `npm run typecheck && npm test`
Expected: PASS.

- [ ] **Step 7: Add the golden-parity probe**

In `scripts/verify/golden-parity.mjs`, next to the `/api/v1/engines` probe, add a `GET /api/v1/engines/capabilities` probe that asserts `Array.isArray(body.capabilities)` and every row has `contract_version === 'v1'`. Run: `node scripts/verify/golden-parity.mjs --dry-run` if the script supports it, otherwise `node --check scripts/verify/golden-parity.mjs`.

- [ ] **Step 8: Commit**

```bash
git add src scripts/verify/golden-parity.mjs
git commit -m "feat(engine-status): consume contract-v1 capabilities and typed provenance"
```

---

### Task B2: noesis-raycast capabilities in the API client and Engines command

**Files:**
- Modify: `src/lib/types.ts:58-63` (`EngineSummary`), `:150-160` (`EngineExecutionResult`)
- Modify: `src/lib/api.ts` (`SelemeneApiClient` near `:493`; `fetchRemoteSnapshot` near `:234`; `toEngineExecutionResult` at `:857`)
- Modify: `src/engines.tsx` (the Engines command; show availability accessory)
- Test: `src/lib/api.test.ts`

**Interfaces:**
- Consumes: `requestJson` transport, `X-API-Key` header, `optionalRequest` (`api.ts:1090`) which turns failures into `SyncIssue`.
- Produces: `SelemeneApiClient.getCapabilities(): Promise<EngineCapability[]>`; `EngineSummary.availability: CapabilityAvailability`; `EngineExecutionResult.provenance?: EngineProvenance`; `RemoteSnapshot.capabilities: EngineCapability[]`.

- [ ] **Step 1: Write failing tests**

Append to `src/lib/api.test.ts` (use that file's `fetchStub` and `withExecutionRoute` helpers):

```ts
test("getCapabilities hits /api/v1/engines/capabilities with X-API-Key", async () => {
  const calls: Array<{ url: string; init?: RequestInit }> = [];
  const fetchStub: typeof fetch = async (url, init) => {
    calls.push({ url: String(url), init });
    return new Response(JSON.stringify({ capabilities: [{
      contract_version: "v1", engine_id: "tarot", display_name: "Tarot",
      availability: "available", runtime_kind: "typescript", dependencies: [] }], count: 1 }), { status: 200 });
  };
  const client = new SelemeneApiClient({ baseUrl: "https://selemene.tryambakam.space", apiKey: "k", fetchImpl: fetchStub });
  const rows = await client.getCapabilities();
  assert.equal(calls[0].url, "https://selemene.tryambakam.space/api/v1/engines/capabilities");
  assert.equal((calls[0].init?.headers as Record<string, string>)["X-API-Key"], "k");
  assert.equal(rows[0].availability, "available");
});

test("getCapabilities returns [] on 404 and unknown availability becomes declared", async () => {
  const notFound: typeof fetch = async () => new Response("x", { status: 404 });
  const client = new SelemeneApiClient({ baseUrl: "https://selemene.tryambakam.space", apiKey: "k", fetchImpl: notFound });
  assert.deepEqual(await client.getCapabilities(), []);
  const weird: typeof fetch = async () => new Response(JSON.stringify({ capabilities: [{
    contract_version: "v1", engine_id: "x", display_name: "X", availability: "turbo", runtime_kind: "native", dependencies: [] }] }), { status: 200 });
  const client2 = new SelemeneApiClient({ baseUrl: "https://selemene.tryambakam.space", apiKey: "k", fetchImpl: weird });
  assert.equal((await client2.getCapabilities())[0].availability, "declared");
});

test("toEngineExecutionResult keeps provenance", () => {
  const out = toEngineExecutionResult({ engine_id: "tarot", result: {}, metadata: {},
    provenance: { runtime_kind: "typescript", implementation_version: "1.0.0", cached: false, fallback_used: false, confidence: 1 } });
  assert.equal(out.provenance?.confidence, 1);
});
```

If `SelemeneApiClient`'s constructor takes a different options shape than `{baseUrl, apiKey, fetchImpl}`, match the shape the existing tests in that file already use. Export `toEngineExecutionResult` from `api.ts` if it is currently module-private.

- [ ] **Step 2: Run to verify failure**

Run: `cd /Volumes/madara/2026/Projects/tryambakam-noesis/noesis-raycast && npm test`
Expected: FAIL on the three new tests.

- [ ] **Step 3: Add types**

`src/lib/types.ts`:

```ts
export type CapabilityAvailability = "declared" | "available" | "degraded" | "unavailable";
export type RuntimeKind = "native" | "typescript" | "python" | "database-conditional" | "composed";

export interface EngineCapability {
  contractVersion: "v1";
  engineId: string;
  displayName: string;
  availability: CapabilityAvailability;
  runtimeKind: RuntimeKind;
  dependencies: string[];
  requiredPhase?: number;
  implementationVersion?: string;
}

export interface EngineProvenance {
  runtime_kind: RuntimeKind;
  implementation_version: string;
  cached: boolean;
  fallback_used: boolean;
  backend_id?: string;
  provider_id?: string;
  confidence?: number;
}
```

Add `availability?: CapabilityAvailability` to `EngineSummary`, `provenance?: EngineProvenance` to `EngineExecutionResult`, and `capabilities: EngineCapability[]` to `RemoteSnapshot`.

- [ ] **Step 4: Implement client method and mapping**

In `src/lib/api.ts` inside `SelemeneApiClient` next to `getEngineInfo`:

```ts
  async getCapabilities(): Promise<EngineCapability[]> {
    try {
      const body = await this.request<{ capabilities?: Array<Record<string, unknown>> }>("GET", "/api/v1/engines/capabilities");
      return (body.capabilities ?? []).map(toEngineCapability);
    } catch (error) {
      if (error instanceof JsonRequestError && error.kind === "http" && error.status === 404) return [];
      throw error;
    }
  }
```

and module-level:

```ts
const AVAILABILITY: readonly CapabilityAvailability[] = ["declared", "available", "degraded", "unavailable"];
export function normalizeAvailability(value: unknown): CapabilityAvailability {
  return AVAILABILITY.includes(value as CapabilityAvailability) ? (value as CapabilityAvailability) : "declared";
}
function toEngineCapability(row: Record<string, unknown>): EngineCapability {
  return {
    contractVersion: "v1",
    engineId: String(row.engine_id),
    displayName: String(row.display_name ?? row.engine_id),
    availability: normalizeAvailability(row.availability),
    runtimeKind: (row.runtime_kind as RuntimeKind) ?? "native",
    dependencies: Array.isArray(row.dependencies) ? (row.dependencies as string[]) : [],
    requiredPhase: typeof row.required_phase === "number" ? row.required_phase : undefined,
    implementationVersion: typeof row.implementation_version === "string" ? row.implementation_version : undefined,
  };
}
```

In `toEngineExecutionResult` add `provenance: raw.provenance as EngineProvenance | undefined,`. In `fetchRemoteSnapshot`, fetch capabilities through `optionalRequest` (so a failure becomes a `SyncIssue` with `resource: "capabilities"`), then stamp `availability` onto each `EngineSummary` by `engineId`.

- [ ] **Step 5: Run tests, lint**

Run: `npm test && npm run lint`
Expected: PASS.

- [ ] **Step 6: Show availability in the Engines command**

In `src/engines.tsx` add a `List.Item.Accessory` tag per engine: `{ tag: { value: engine.availability ?? "declared", color: engine.availability === "available" ? Color.Green : engine.availability === "degraded" ? Color.Orange : engine.availability === "unavailable" ? Color.Red : Color.SecondaryText } }`.

Run: `npm run build`
Expected: `ray build` succeeds.

- [ ] **Step 7: Commit**

```bash
git add src
git commit -m "feat(engines): show contract-v1 availability and keep provenance on results"
```

---

### Task B3: antahkarana `selemene_capabilities` Tauri command feeding the roster

**Files:**
- Modify: `src-tauri/src/lib.rs` (new command after `selemene_health` at `:287-324`; register at `:2376-2384`)
- Modify: `src/features/roster/model.ts:35,151,188` and `src/features/roster/selemene.ts:68`
- Create: `tests/fixtures/selemene_capabilities.json`
- Modify: `scripts/check-truth-table.mjs` (assert the new invoke string and fixture shape)
- Test: `src-tauri/src/lib.rs` `#[cfg(test)]` (pure mapping test) and `bun run check:truth`

**Interfaces:**
- Consumes: `SELEMENE_API_URL` (`lib.rs:37`), keychain token via the same lookup `selemene_daily_practice` uses, header `X-API-Key`.
- Produces: Tauri command `selemene_capabilities` returning `{ ok: bool, status: u16, worker: String, body: serde_json::Value }` (same envelope as `selemene_health`); `parseCapabilitiesFromBody(body: unknown): SelemeneEngine[]` in `roster/model.ts`.

- [ ] **Step 1: Write the failing Rust mapping test**

In `src-tauri/src/lib.rs` `mod tests`:

```rust
    #[test]
    fn availability_to_roster_status_maps_enum() {
        assert_eq!(availability_to_roster_status("available"), "active");
        assert_eq!(availability_to_roster_status("degraded"), "idle");
        assert_eq!(availability_to_roster_status("unavailable"), "offline");
        assert_eq!(availability_to_roster_status("declared"), "idle");
        assert_eq!(availability_to_roster_status("turbo"), "offline");
    }
```

- [ ] **Step 2: Run to verify failure**

Run: `cd /Volumes/madara/2026/Projects/tryambakam-noesis/antahkarana && bun run test:containment`
Expected: FAIL, `availability_to_roster_status` not found.

- [ ] **Step 3: Implement command and helper**

After `selemene_health` in `lib.rs`:

```rust
/// Map contracts/v1 availability to the roster status vocabulary used by
/// `src/features/roster/model.ts` (`active | idle | offline | error`).
pub fn availability_to_roster_status(availability: &str) -> &'static str {
    match availability {
        "available" => "active",
        "degraded" | "declared" => "idle",
        _ => "offline",
    }
}

#[tauri::command]
fn selemene_capabilities() -> Result<serde_json::Value, String> {
    let token = read_selemene_token()?; // reuse the keychain lookup selemene_daily_practice uses
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| e.to_string())?;
    let url = format!("{SELEMENE_API_URL}/api/v1/engines/capabilities");
    let resp = client
        .get(&url)
        .header("X-API-Key", &token)
        .header("Accept", "application/json")
        .send()
        .map_err(|e| e.to_string())?;
    let status = resp.status().as_u16();
    let body: serde_json::Value = resp.json().unwrap_or(serde_json::Value::Null);
    Ok(serde_json::json!({
        "ok": status == 200,
        "status": status,
        "worker": SELEMENE_API_URL,
        "body": body,
    }))
}
```

If the keychain read is inlined in `selemene_daily_practice` rather than a helper, extract it into `fn read_selemene_token() -> Result<String, String>` and use it from both. Register `selemene_capabilities` in the `generate_handler![...]` list at `lib.rs:2376-2384`.

- [ ] **Step 4: Run Rust tests**

Run: `bun run test:containment`
Expected: PASS.

- [ ] **Step 5: Fixture and truth table**

`tests/fixtures/selemene_capabilities.json`:

```json
{
  "ok": true,
  "status": 200,
  "worker": "https://selemene.tryambakam.space",
  "body": {
    "capabilities": [
      { "contract_version": "v1", "engine_id": "tarot", "display_name": "Tarot", "availability": "available", "runtime_kind": "typescript", "dependencies": [], "required_phase": 1 },
      { "contract_version": "v1", "engine_id": "panchanga", "display_name": "panchanga", "availability": "available", "runtime_kind": "native", "dependencies": [], "implementation_version": "3.3.1" }
    ],
    "count": 2
  }
}
```

In `scripts/check-truth-table.mjs`, next to the existing `selemene_health` assertions (`:47-72`), add: the string `invoke("selemene_capabilities")` must appear in `src/features/roster/selemene.ts`, and the fixture's every `body.capabilities[i].availability` must be one of the four enum values and `contract_version` must be `"v1"`.

Run: `bun run check:truth`
Expected: FAIL (invoke string not yet present).

- [ ] **Step 6: Frontend parsing**

`src/features/roster/model.ts`, add:

```ts
export type CapabilityAvailability = 'declared' | 'available' | 'degraded' | 'unavailable'

const AVAILABILITY: readonly CapabilityAvailability[] = ['declared', 'available', 'degraded', 'unavailable']

export function normalizeAvailability(value: unknown): CapabilityAvailability {
  return AVAILABILITY.includes(value as CapabilityAvailability) ? (value as CapabilityAvailability) : 'declared'
}

export function availabilityToStatus(a: CapabilityAvailability): SelemeneEngine['status'] {
  if (a === 'available') return 'active'
  if (a === 'unavailable') return 'offline'
  return 'idle'
}

export function parseCapabilitiesFromBody(body: unknown): SelemeneEngine[] {
  const rows = (body as { capabilities?: unknown[] } | null)?.capabilities
  if (!Array.isArray(rows)) return []
  return rows.map((raw) => {
    const row = raw as Record<string, unknown>
    const id = String(row.engine_id)
    return {
      id,
      name: String(row.display_name ?? id),
      kosha: ENGINE_KOSHA_MAP[id] ?? 'unmapped',
      status: availabilityToStatus(normalizeAvailability(row.availability)),
      role: typeof row.runtime_kind === 'string' ? row.runtime_kind : undefined,
    }
  })
}
```

In `src/features/roster/selemene.ts:68`, after the existing `invoke("selemene_health")`, call `invoke("selemene_capabilities")`; when `ok` is true and `body.capabilities` is an array, use `parseCapabilitiesFromBody` for the engine list instead of `parseEnginesFromGatewayBody`; when status is 404 keep the current health-derived roster (the `summary-only` state). Keep 401/403 mapping to `key-rotation-pending` as today.

Run: `bun run check:truth && bun run check:secrets && bunx tsc --noEmit`
Expected: PASS.

- [ ] **Step 7: Commit**

```bash
git add src-tauri/src/lib.rs src/features/roster tests/fixtures/selemene_capabilities.json scripts/check-truth-table.mjs
git commit -m "feat(roster): live engine availability via selemene_capabilities command"
```

---

### Task B4: sankalpa gateway `capabilities` IPC on the 0.2.0 SDK (baseline for the twc-shell port)

Context: sankalpa is tombstoned (R4, 2026-07-28). This task keeps its gateway contract-current so the future port copies a correct pattern. It does not build a release.

**Files:**
- Modify: `package.json:39` (`"@selemene/engine-sdk": "file:vendor/selemene-engine-sdk-0.2.0.tgz"`)
- Modify: `src/shared/engineGateway.ts` (types), `src/main/engineGateway.ts` (`capabilities()`), `src/main/main.ts:26-40` (IPC handler `sankalpa:engine:capabilities`), `src/preload/preload.ts`, `src/renderer/engine/gateway.ts`
- Test: `tests/engine-gateway-main.test.ts`

**Interfaces:**
- Consumes: `EngineClient.capabilities()` from Task A4 via the tarball from Task A5.
- Produces: `DesktopGatewayApi.capabilities(): Promise<GatewayResponse<GatewayCapability[]>>`; `GatewayCapability = { engineId, displayName, availability, runtimeKind, requiredPhase? }`; renderer `getGatewayCapabilities()`.

- [ ] **Step 1: Swap the vendored SDK**

```bash
cd /Volumes/madara/2026/Projects/tryambakam-noesis/sankalpa
git rm -q vendor/selemene-engine-sdk-0.1.0.tgz
sed -i '' 's#selemene-engine-sdk-0.1.0.tgz#selemene-engine-sdk-0.2.0.tgz#' package.json
npm install
```

Expected: `node_modules/@selemene/engine-sdk/dist/client.js` contains `capabilities(`. `tests/desktop-foundation.test.ts:82` asserts the dependency name only, so it still passes.

- [ ] **Step 2: Write the failing gateway test**

Append to `tests/engine-gateway-main.test.ts` (reuse its `fetchImpl` injection and `DesktopEngineGateway` construction pattern):

```ts
it("capabilities() calls /api/v1/engines/capabilities and maps rows", async () => {
  const calls: string[] = [];
  const fetchImpl = (async (url: string) => {
    calls.push(url);
    return new Response(JSON.stringify({ capabilities: [{
      contract_version: "v1", engine_id: "raaga", display_name: "Raaga",
      availability: "degraded", runtime_kind: "typescript", dependencies: [] }], count: 1 }), { status: 200 });
  }) as typeof fetch;
  const gateway = new DesktopEngineGateway({ baseUrl: "https://engines.example", bearer: "t" }, fetchImpl);
  const res = await gateway.capabilities();
  expect(calls[0]).toBe("https://engines.example/api/v1/engines/capabilities");
  expect(res.ok).toBe(true);
  if (res.ok) expect(res.data[0]).toMatchObject({ engineId: "raaga", availability: "degraded" });
});

it("capabilities() on 404 returns ok with an empty list (declared-only)", async () => {
  const fetchImpl = (async () => new Response("x", { status: 404 })) as typeof fetch;
  const gateway = new DesktopEngineGateway({ baseUrl: "https://engines.example", bearer: "t" }, fetchImpl);
  const res = await gateway.capabilities();
  expect(res.ok).toBe(true);
  if (res.ok) expect(res.data).toEqual([]);
});
```

Match the constructor signature to what the file's existing tests use for `DesktopEngineGateway`.

- [ ] **Step 3: Run to verify failure**

Run: `npm test -- tests/engine-gateway-main.test.ts`
Expected: FAIL, `capabilities is not a function`.

- [ ] **Step 4: Implement shared types, main, preload, renderer**

`src/shared/engineGateway.ts`:

```ts
export type CapabilityAvailability = "declared" | "available" | "degraded" | "unavailable";
export interface GatewayCapability {
  engineId: string;
  displayName: string;
  availability: CapabilityAvailability;
  runtimeKind: string;
  requiredPhase?: number;
}
```

and add `capabilities(): Promise<GatewayResponse<GatewayCapability[]>>` to `DesktopGatewayApi`.

`src/main/engineGateway.ts`, inside `DesktopEngineGateway`:

```ts
  async capabilities(): Promise<GatewayResponse<GatewayCapability[]>> {
    try {
      const res = await this.client.capabilities();
      const data = res.capabilities.map((row) => ({
        engineId: row.engine_id,
        displayName: row.display_name,
        availability: (["declared", "available", "degraded", "unavailable"] as const).includes(row.availability)
          ? row.availability : "declared",
        runtimeKind: row.runtime_kind,
        requiredPhase: row.required_phase,
      }));
      return { ok: true, data, provenance: this.provenance() };
    } catch (error) {
      if (error instanceof EngineSdkError && error.status === 404) {
        return { ok: true, data: [], provenance: this.provenance() };
      }
      return { ok: false, error: classifyGatewayError(error) };
    }
  }
```

Use whatever the class already calls its provenance builder (the object `{source:"remote", transport:"desktop-gateway", authenticated, receivedAt}`); if it is inlined, extract `private provenance(): GatewayProvenance`.

`src/main/main.ts`: `ipcMain.handle("sankalpa:engine:capabilities", () => gateway.capabilities());`
`src/preload/preload.ts`: add `capabilities: () => ipcRenderer.invoke("sankalpa:engine:capabilities")` to the exposed `engine` object.
`src/renderer/engine/gateway.ts`: `export function getGatewayCapabilities() { return window.sankalpa.engine.capabilities(); }`.

- [ ] **Step 5: Run tests and typecheck**

Run: `npm test && npm run typecheck`
Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add package.json package-lock.json vendor src tests
git commit -m "feat(gateway): capabilities IPC on engine-sdk 0.2.0 (port baseline, not a release)"
```

---

### Task B5: noesismirror-web minimal Selemene capability client with a test runner

Context: no Selemene integration exists. The design doc plans a witness-agents pipeline, not engine calls. This task installs the smallest correct foundation: Vitest (which the bootstrap plan at `docs/plans/2026-06-27-noesismirror-web-bootstrap.md:181-188` already intended) and one env-configured capability client, so the world-config builder can later gate beacons on engine availability.

**Files:**
- Modify: `package.json` (add `"test": "vitest run"`, devDeps `vitest`)
- Create: `vitest.config.ts`
- Create: `src/lib/selemene/capabilities.ts`
- Create: `src/lib/selemene/capabilities.test.ts`
- Create: `.env.example` with `VITE_SELEMENE_BASE_URL=https://selemene.tryambakam.space`

**Interfaces:**
- Produces: `fetchCapabilities(baseUrl: string, fetchImpl?: typeof fetch): Promise<EngineCapability[]>`; `EngineCapability` type identical in shape to urania's (Task B1 Step 3). No API key is used: this is a browser SPA and `GET /api/v1/engines/capabilities` requires auth, so this client is only meaningful through a same-origin proxy later; the test pins the URL and the 404/401 fallback to `[]`.

- [ ] **Step 1: Install vitest and config**

```bash
cd /Volumes/madara/2026/Projects/tryambakam-noesis/noesismirror-web
npm install --save-dev vitest
```

`vitest.config.ts`:

```ts
import { defineConfig } from 'vitest/config'
export default defineConfig({ test: { environment: 'node', include: ['src/**/*.test.ts'] } })
```

Add `"test": "vitest run"` to `package.json` scripts.

- [ ] **Step 2: Write the failing test**

`src/lib/selemene/capabilities.test.ts`:

```ts
import { describe, expect, it, vi } from 'vitest'
import { fetchCapabilities } from './capabilities'

describe('fetchCapabilities', () => {
  it('GETs <base>/api/v1/engines/capabilities', async () => {
    const fetchImpl = vi.fn(async () => new Response(JSON.stringify({ capabilities: [
      { contract_version: 'v1', engine_id: 'tarot', display_name: 'Tarot', availability: 'available', runtime_kind: 'typescript', dependencies: [] },
    ], count: 1 }), { status: 200 })) as unknown as typeof fetch
    const rows = await fetchCapabilities('https://selemene.tryambakam.space/', fetchImpl)
    expect((fetchImpl as unknown as ReturnType<typeof vi.fn>).mock.calls[0][0]).toBe('https://selemene.tryambakam.space/api/v1/engines/capabilities')
    expect(rows[0].availability).toBe('available')
  })
  it('returns [] on 401 or 404', async () => {
    for (const status of [401, 404]) {
      const fetchImpl = (async () => new Response('x', { status })) as unknown as typeof fetch
      expect(await fetchCapabilities('https://x', fetchImpl)).toEqual([])
    }
  })
})
```

- [ ] **Step 3: Run to verify failure**

Run: `npm test`
Expected: FAIL, cannot resolve `./capabilities`.

- [ ] **Step 4: Implement**

`src/lib/selemene/capabilities.ts`:

```ts
export type CapabilityAvailability = 'declared' | 'available' | 'degraded' | 'unavailable'
export type RuntimeKind = 'native' | 'typescript' | 'python' | 'database-conditional' | 'composed'
export interface EngineCapability {
  contract_version: 'v1'
  engine_id: string
  display_name: string
  availability: CapabilityAvailability
  runtime_kind: RuntimeKind
  dependencies: string[]
  required_phase?: number
  implementation_version?: string
}

const AVAILABILITY: readonly CapabilityAvailability[] = ['declared', 'available', 'degraded', 'unavailable']

export function normalizeAvailability(value: unknown): CapabilityAvailability {
  return AVAILABILITY.includes(value as CapabilityAvailability) ? (value as CapabilityAvailability) : 'declared'
}

export async function fetchCapabilities(baseUrl: string, fetchImpl: typeof fetch = fetch): Promise<EngineCapability[]> {
  const base = baseUrl.replace(/\/+$/, '')
  const res = await fetchImpl(`${base}/api/v1/engines/capabilities`, { headers: { Accept: 'application/json' } })
  if (res.status === 401 || res.status === 404) return []
  if (!res.ok) throw new Error(`capabilities ${res.status}`)
  const body = (await res.json()) as { capabilities?: unknown[] }
  return (body.capabilities ?? []).map((raw) => {
    const row = raw as Record<string, unknown>
    return {
      contract_version: 'v1',
      engine_id: String(row.engine_id),
      display_name: String(row.display_name ?? row.engine_id),
      availability: normalizeAvailability(row.availability),
      runtime_kind: (row.runtime_kind as RuntimeKind) ?? 'native',
      dependencies: Array.isArray(row.dependencies) ? (row.dependencies as string[]) : [],
      required_phase: typeof row.required_phase === 'number' ? row.required_phase : undefined,
      implementation_version: typeof row.implementation_version === 'string' ? row.implementation_version : undefined,
    }
  })
}
```

`.env.example`: one line `VITE_SELEMENE_BASE_URL=https://selemene.tryambakam.space`.

- [ ] **Step 5: Run tests and commit**

Run: `npm test`
Expected: PASS (2 tests).

```bash
git add package.json package-lock.json vitest.config.ts src/lib/selemene .env.example
git commit -m "feat(selemene): vitest + contract-v1 capability client foundation"
```

---

# Phase C: Verification and handoff

### Task C1: Cross-repo verification receipt

**Files:**
- Modify: `Selemene-engine/docs/plans/selemene-engine/RUNTIME-CAPABILITY-EVIDENCE.md` (append "Consumer rollout receipts")

- [ ] **Step 1: Run every gate and record exact output counts**

```bash
cd /Volumes/madara/2026/Projects/tryambakam-noesis/Selemene-engine && pnpm run gate:contracts && cargo build --workspace --locked && (cd ts-engines && bun run typecheck && bun test)
cd /Volumes/madara/2026/Projects/tryambakam-noesis/urania-137 && npm test && npm run typecheck
cd /Volumes/madara/2026/Projects/tryambakam-noesis/noesis-raycast && npm test && npm run lint
cd /Volumes/madara/2026/Projects/tryambakam-noesis/antahkarana && bun run test:containment && bun run check:truth && bun run check:secrets
cd /Volumes/madara/2026/Projects/tryambakam-noesis/sankalpa && npm test && npm run typecheck
cd /Volumes/madara/2026/Projects/tryambakam-noesis/noesismirror-web && npm test
```

- [ ] **Step 2: Live readback (read-only, after Phase A is deployed by the user)**

```bash
curl -s -o /dev/null -w '%{http_code}\n' https://selemene.tryambakam.space/api/v1/engines/capabilities
```

Record the status code. `401` means the route is deployed and auth-gated (expected). `404` means Phase A is not yet deployed and all consumers are running their fallback paths. Do not include any API key in the receipt.

- [ ] **Step 3: Commit**

```bash
cd /Volumes/madara/2026/Projects/tryambakam-noesis/Selemene-engine
git add docs/plans/selemene-engine/RUNTIME-CAPABILITY-EVIDENCE.md
git commit -m "docs(capability): consumer rollout verification receipts"
```

## Self-review notes

- Spec coverage: slices 1 to 3 of the evidence doc are consumed (A1 to A4); each of the five named repos has exactly one task (B1 to B5); the admin-gated route limitation the evidence doc did not mention is closed by A3.
- Type consistency: `EngineCapability` field names are snake_case on the wire everywhere; raycast alone maps to camelCase inside `toEngineCapability`, matching its existing `EngineSummary` convention. `normalizeAvailability` has the same signature in B1, B2, B3, B5.
- Review Focus items 1 to 5 are pinned by tests in A1 (round trip), B1/B2/B3/B4 (404 fallbacks), A3 (never-empty list), B1/B2/B3 (unknown enum to `declared`), A4 (optional `capability_status`).
- Sankalpa's tombstone and noesismirror-web's absent integration are stated as constraints on scope, not silently narrowed.
