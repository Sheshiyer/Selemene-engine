---
phase: 03-capability-and-contract-closure
plan: "20b"
type: execute
wave: 11
depends_on: ["03-20"]
files_modified:
  - llms.txt
  - docs/api/openapi.yaml
  - docs/api/TUI_INTEGRATION.md
  - docs/API_QUICKSTART.md
  - docs/engines/README.md
  - bridges/cli/README.md
autonomous: true
requirements: [CON-02]
must_haves:
  truths:
    - "The maintained API, CLI and engine documentation describes canonical 19 runtime identities, 17 public mirrors, current operations/auth and Full Spectrum unsupported status."
    - "Documentation examples and operation names are derived from contracts/v1 rather than stale inventory prose."
  artifacts:
    - path: llms.txt
      provides: "Canonical AI-facing API and capability catalogue summary"
    - path: docs/api/openapi.yaml
      provides: "Current authenticated API operation documentation"
    - path: docs/engines/README.md
      provides: "Canonical engine catalogue counts and states"
  key_links:
    - from: docs/api/openapi.yaml
      to: contracts/v1/manifest.json
      via: "documented route and auth contract"
    - from: bridges/cli/README.md
      to: contracts/v1/registries/engines.json
      via: "documented public operation projection"
---

<objective>Bring the maintained API, CLI and engine catalogue views into exact parity with the Phase 3 contracts.</objective>
<execution_context>
@/Users/sheshnarayaniyer/.codex/get-shit-done/workflows/execute-plan.md
@/Users/sheshnarayaniyer/.codex/get-shit-done/templates/summary.md
</execution_context>
<context>
@.planning/PROJECT.md
@.planning/phases/03-capability-and-contract-closure/03-CONTEXT.md
@.planning/phases/03-capability-and-contract-closure/03-RESEARCH.md
@contracts/v1/manifest.json
@contracts/v1/registries/engines.json
@contracts/v1/schemas/engine-capability-list.schema.json
@contracts/v1/schemas/workflow-outcome.schema.json
@llms.txt
@docs/api/openapi.yaml
@docs/api/TUI_INTEGRATION.md
@docs/API_QUICKSTART.md
@docs/engines/README.md
@bridges/cli/README.md
</context>
<tasks>
<task type="auto">
  <name>Task 1: Align API and AI-facing contract documentation</name>
  <files>llms.txt, docs/api/openapi.yaml, docs/api/TUI_INTEGRATION.md</files>
  <read_first>llms.txt, docs/api/openapi.yaml, docs/api/TUI_INTEGRATION.md, contracts/v1/manifest.json, contracts/v1/registries/engines.json, contracts/v1/schemas/engine-capability-list.schema.json, contracts/v1/schemas/workflow-outcome.schema.json, crates/noesis-api/src/lib.rs, crates/noesis-api/src/handlers/admin.rs, crates/noesis-api/src/handlers/witness.rs</read_first>
  <action>Derive documented route names, authenticated API-key/bearer behavior, capability operations, workflow support and counts from the canonical manifest, registry and schemas. Update `llms.txt`, `docs/api/openapi.yaml` and `docs/api/TUI_INTEGRATION.md` so examples describe 19 runtime identities, 17 public mirrors, the four availability states and Full Spectrum as explicitly unsupported pending a lossless adapter test. Keep the established authenticated per-engine calculate route and current workflow API paths; remove stale inventory or executable Full Spectrum claims without inventing routes.</action>
  <acceptance_criteria>These three views contain only current canonical operations/auth semantics, state 19 runtime identities and 17 public mirrors, and mark Full Spectrum unsupported.</acceptance_criteria>
  <verify><automated>uv run --project python-services --locked --extra dev python -m json.tool contracts/v1/manifest.json &gt;/dev/null &amp;&amp; rg -n '19' llms.txt docs/api/openapi.yaml docs/api/TUI_INTEGRATION.md &amp;&amp; rg -n '17' llms.txt docs/api/openapi.yaml docs/api/TUI_INTEGRATION.md &amp;&amp; rg -n 'X-API-Key' llms.txt docs/api/openapi.yaml docs/api/TUI_INTEGRATION.md &amp;&amp; rg -n 'Authorization' llms.txt docs/api/openapi.yaml docs/api/TUI_INTEGRATION.md &amp;&amp; rg -n 'unsupported' llms.txt docs/api/openapi.yaml docs/api/TUI_INTEGRATION.md &amp;&amp; rg -n 'Full Spectrum' llms.txt docs/api/openapi.yaml docs/api/TUI_INTEGRATION.md</automated></verify>
  <done>AI-facing and API integration docs are synchronized with the canonical contract views.</done>
</task>
<task type="auto">
  <name>Task 2: Align quickstart, engine and CLI documentation</name>
  <files>docs/API_QUICKSTART.md, docs/engines/README.md, bridges/cli/README.md</files>
  <read_first>docs/API_QUICKSTART.md, docs/engines/README.md, bridges/cli/README.md, contracts/v1/manifest.json, contracts/v1/registries/engines.json, contracts/v1/schemas/engine-capability-list.schema.json, contracts/v1/schemas/workflow-outcome.schema.json, bridges/cli/src/generators/langchain.ts, bridges/cli/src/core/types.ts</read_first>
  <action>Derive counts, public mirror membership, supported calculate operations, authentication headers and workflow support from `contracts/v1` and the existing CLI generator. Update the quickstart, engine catalogue and CLI README with 19 runtime identities and exactly 17 public mirrors, current protected Rust routes and explicit Full Spectrum unsupported wording. Keep generated-tool examples limited to canonical public operations and preserve safe credential handling; do not document direct sidecar or invented execute routes.</action>
  <acceptance_criteria>All three maintained views agree on 19/17 counts, canonical operation/auth examples and unsupported Full Spectrum status.</acceptance_criteria>
  <verify><automated>rg -n '19' docs/API_QUICKSTART.md docs/engines/README.md bridges/cli/README.md &amp;&amp; rg -n '17' docs/API_QUICKSTART.md docs/engines/README.md bridges/cli/README.md &amp;&amp; rg -n 'unsupported' docs/API_QUICKSTART.md docs/engines/README.md bridges/cli/README.md &amp;&amp; rg -n 'Full Spectrum' docs/API_QUICKSTART.md docs/engines/README.md bridges/cli/README.md &amp;&amp; rg -n 'X-API-Key' docs/API_QUICKSTART.md docs/engines/README.md bridges/cli/README.md &amp;&amp; rg -n 'Authorization' docs/API_QUICKSTART.md docs/engines/README.md bridges/cli/README.md</automated></verify>
  <done>Quickstart, engine catalogue and CLI docs no longer drift from contract authority.</done>
</task>
</tasks>
<threat_model>
## Assets
Public API documentation, route/auth claims, engine counts and workflow support claims.
## Trust Boundaries
Canonical contract files cross into human and AI-facing documentation consumed as integration guidance.
## Abuse Cases
Stale counts hide identities, docs teach unauthenticated calls, or unsupported workflow text causes clients to make calls.
## STRIDE Threat Register
| Threat | Category | Disposition | Blocking test |
|---|---|---|---|
| T3-01 | Spoofing | mitigate | Focused docs receipt checks API-key and bearer wording. |
| T3-02 | Information Disclosure | mitigate | Credential examples are bounded and header-only. |
| T3-03 | Tampering/Availability | mitigate | 19/17 and four-state parity assertions. |
| T3-04 | Tampering | mitigate | Current route names are checked against manifest/schema. |
| T3-05 | Improper Inventory | mitigate | Canonical registry-derived count assertions. |
| T3-06 | Elevation of Privilege | mitigate | Authenticated route documentation assertions. |
| T3-07 | Tampering | mitigate | Workflow support/unsupported wording checks. |
| T3-08 | Tampering | mitigate | No invented execute or direct-sidecar route assertions. |
</threat_model>
<verification>Run both focused documentation parity receipts against the canonical manifest, registry and schemas.</verification>
<success_criteria>Every maintained API/CLI/engine view in this slice states canonical counts, operations, authentication and truthful workflow support.</success_criteria>
<output>Create `.planning/phases/03-capability-and-contract-closure/03-20b-SUMMARY.md`.</output>
