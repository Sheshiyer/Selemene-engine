# Phase 3: Capability and contract closure - Context

**Gathered:** 2026-09-06
**Status:** Ready for planning
**Mode:** User-authorized recommended defaults (`discuss --auto`), single pass

<domain>
## Phase Boundary

Complete Selemene's Wave 2 contract and routing convergence inside this repository. One canonical identity and schema authority must survive Rust, TypeScript, Python, HTTP, bridge, SDK, CLI/tool server, TUI, admin and workflow boundaries while current runtime availability remains truthful. This phase may repair contract adapters, discovery, routing, validation, error envelopes and synthesis status. Engine calculation semantics belong to Phase 4; stateful production behavior, package release and live deployment belong to Phases 5–7.

</domain>

<decisions>
## Implementation Decisions

### Catalogue identity and runtime state
- **D-01:** Keep the versioned 19-row engine registry as identity authority: 12 native, one database-conditional and six TypeScript runtime IDs, with 17 public mirror groups. Runtime discovery overlays observed availability; it does not create or remove identities.
- **D-02:** Preserve the canonical `declared`, `available`, `degraded` and `unavailable` capability states. Missing required dependencies fail closed as unavailable; optional or partially reachable dependencies become degraded. Responses carry bounded machine-readable dependency/reason data and never omit an engine silently.

### Cross-surface contract ownership
- **D-03:** Generate or validate every maintained catalogue and contract view from `contracts/v1`; surface-specific labels and presentation are allowed, but independent engine-ID lists, envelope shapes and auth conventions are not.
- **D-04:** Roll out additively through canonical fixtures and compatibility adapters. Supported v1 payloads remain valid; incompatible contract-version or schema skew fails closed with the canonical JSON error envelope and an actionable stable error code rather than permissive coercion.

### Routing, validation and failure behavior
- **D-05:** Freeze API-key transport, native/TypeScript/Python routing, unknown-ID behavior, timeouts and partial dependency failures in executable cross-language tests. Authentication and validation failures must cause zero downstream engine/provider calls and must not expose secrets or internal URLs.
- **D-06:** Keep `/engines/:id/validate` only if the server and engine boundary can provide real validation under the canonical contract. Otherwise remove the unsupported bridge/client assumption and return an explicit capability state; no stub-success validation is acceptable.

### Workflow synthesis truth
- **D-07:** Connect the public workflow path to the existing richer synthesis executor where the current architecture supports it. Any unsupported or failed synthesis remains explicit and carries per-engine partial-failure information; an empty or partial result cannot be labelled successful or LLM-powered.

### the agent's Discretion
The planner and executors may choose the exact generator, build step, module boundaries, fixture layout and compatibility-adapter internals. Prefer small dependency-ordered slices, current repository patterns and fail-closed validators. Do not expand engine semantics, call paid providers, mutate live services, alter production data or edit consumer repositories.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Acceptance and phase authority
- `ISA.md` — Acceptance ledger, isolation rule, stable criteria and critical production boundaries.
- `.planning/PROJECT.md` — Continuation scope and user-authorized recommended GSD defaults.
- `.planning/ROADMAP.md` — Phase 3 goal, requirements and dependency on verified Phase 2.
- `.planning/REQUIREMENTS.md` — `CON-01` and `CON-02` traceability.
- `.planning/phases/02-reproducible-gates-dependency-repair/02-VERIFICATION.md` — Exact prior-phase gates and production HOLD.

### Wave 2 and evidence authority
- `docs/plans/selemene-engine/ROADMAP.md` — Canonical Wave 2 tasks and exit criteria.
- `docs/plans/selemene-engine/CAPABILITY-LEDGER.md` — Current contract, platform, routing and synthesis discrepancies.
- `docs/plans/selemene-engine/RUNTIME-CAPABILITY-EVIDENCE.md` — Existing source and runtime capability receipts.
- `docs/plans/selemene-engine/RECOVERY-2026-09-05.md` — Recovery boundary and current-vs-deployed distinctions.
- `https://github.com/Sheshiyer/Selemene-engine/issues/894` — Preserved Wave 2 control issue and unchanged exit checkboxes.

### Canonical contracts and registry
- `contracts/v1/manifest.json` — Versioned schema and fixture authority.
- `contracts/v1/registries/engines.json` — Exact runtime identities, classes, owners and evidence references.
- `contracts/v1/schemas/engine-capability.schema.json` — Capability discovery shape.
- `contracts/v1/schemas/engine-request.schema.json` — Canonical request shape.
- `contracts/v1/schemas/engine-result.schema.json` — Canonical result envelope.
- `contracts/v1/schemas/error.schema.json` — Canonical JSON error envelope.
- `contracts/v1/schemas/consent.schema.json` — Consent contract.
- `contracts/v1/schemas/provenance.schema.json` — Runtime and provider provenance.

### Existing implementation seams
- `crates/noesis-core/src/contract.rs` — Maintained Rust v1 types and capability states.
- `crates/noesis-orchestrator/src/lib.rs` — Native registry, runtime inventory and workflow orchestration.
- `crates/noesis-api/src/lib.rs` — API routes and database-conditional registration.
- `crates/noesis-api/src/handlers/admin.rs` — Authenticated capability and admin catalogue handlers.
- `crates/noesis-bridge/src/lib.rs` — Rust-to-TypeScript route boundary.
- `ts-engines/src/server/registry.ts` — Six-engine TypeScript registration and capability projection.
- `python-services/shared/models.py` — Shared Python service models.
- `packages/noesis-engine-sdk/src/contract-v1.ts` — Maintained TypeScript v1 contract adapter.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `scripts/validate_contracts.py`: fail-closed schema, fixture, registry and source-anchor validator suitable for adding catalogue and cross-surface drift checks.
- `contracts/v1/fixtures/`: canonical request, result, error and capability examples already exercised by Rust and TypeScript contract suites.
- `EngineCapability` and `CapabilityAvailability`: existing Rust model already represents identity, runtime kind, dependencies and four availability states.
- `EngineRegistry.listCapabilities`: TypeScript projection already binds registered engines to the canonical six TypeScript rows.
- `register_database_conditional_engines`: shared API seam already proves absent/present database-dependent registration without a production database.

### Established Patterns
- Contract evolution is additive and versioned; validators reject unknown, malformed, duplicate, stale or unresolved authority.
- Engine identity, availability, calculation semantics, deployment and operations are separate claims.
- Negative fixtures and recording shims prove zero downstream work on rejected inputs.
- Required checks run through one database-free repository gate and strict GitHub `CI Gate`.

### Integration Points
- API `/api/v1/engines`, authenticated admin capability discovery and OpenAPI projections.
- Native orchestrator registry, conditional API registration, Rust bridge and TypeScript service registry.
- Rust SDK, general TypeScript SDK, focus engine SDK, CLI/tool server, TUI, admin and witness/workflow catalogues.
- Public workflow execution and the richer synthesis modules under `crates/noesis-orchestrator/src/workflow/`.

</code_context>

<specifics>
## Specific Ideas

The closure must turn the current contradiction register into executable checks: 19 runtime IDs versus 17 mirrors, unsupported TypeScript validation, list-envelope and API-key drift, hard-coded 16-engine client catalogues, default-healthy self-checks and `synthesis: None`. A green build or healthy service count remains insufficient without the corresponding boundary test.

</specifics>

<deferred>
## Deferred Ideas

- Per-engine calculation corrections, fallback provenance and real provider/media semantics belong to Phase 4.
- Migration, durable auth/token state, billing, cache and persisted-media behavior belong to Phase 5.
- Independent package publication and consumer compatibility belong to Phase 6.
- Railway, Cloudflare, DNS, production images, monitoring and rollback mutation belong to Phase 7 and retain critical HITL.

</deferred>

---

*Phase: 03-capability-and-contract-closure*
*Context gathered: 2026-09-06*
