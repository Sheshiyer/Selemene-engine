---
phase: 03-capability-and-contract-closure
plan: "20c"
type: execute
wave: 11
depends_on: ["03-20"]
files_modified:
  - docs/portal/README.md
  - docs/portal/docs/index.md
  - docs/portal/docs/authentication.md
  - docs/portal/docs/engines/index.md
  - docs/portal/docs/workflows/index.md
autonomous: true
requirements: [CON-02]
must_haves:
  truths:
    - "The portal shell and navigation describe the canonical 19 runtime identities, 17 public mirrors, authentication and workflow support."
    - "Portal indexes do not present Full Spectrum as executable."
  artifacts:
    - path: docs/portal/docs/engines/index.md
      provides: "Portal canonical engine catalogue index"
    - path: docs/portal/docs/workflows/index.md
      provides: "Portal workflow support index"
  key_links:
    - from: docs/portal/docs/engines/index.md
      to: contracts/v1/registries/engines.json
      via: "canonical engine count and public mirror listing"
    - from: docs/portal/docs/workflows/index.md
      to: contracts/v1/schemas/workflow-outcome.schema.json
      via: "workflow support status"
---

<objective>Synchronize the portal shell, authentication, engine index and workflow index with canonical Phase 3 contract metadata.</objective>
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
@docs/portal/README.md
@docs/portal/docs/index.md
@docs/portal/docs/authentication.md
@docs/portal/docs/engines/index.md
@docs/portal/docs/workflows/index.md
</context>
<tasks>
<task type="auto">
  <name>Task 1: Align portal shell and authentication claims</name>
  <files>docs/portal/README.md, docs/portal/docs/index.md, docs/portal/docs/authentication.md</files>
  <read_first>docs/portal/README.md, docs/portal/docs/index.md, docs/portal/docs/authentication.md, contracts/v1/manifest.json, contracts/v1/registries/engines.json, crates/noesis-api/src/middleware.rs</read_first>
  <action>Derive portal counts, authenticated route behavior, API-key/bearer header rules and workflow availability from canonical contract metadata and existing middleware. Replace stale catalogue/workflow totals with 19 runtime identities and 17 public mirrors, preserve the actual authentication flow, and state that Full Spectrum is unsupported pending a lossless adapter test.</action>
  <acceptance_criteria>Portal shell and authentication docs state canonical counts and current auth semantics, with Full Spectrum clearly unsupported.</acceptance_criteria>
  <verify><automated>rg -n '19' docs/portal/README.md docs/portal/docs/index.md docs/portal/docs/authentication.md &amp;&amp; rg -n '17' docs/portal/README.md docs/portal/docs/index.md docs/portal/docs/authentication.md &amp;&amp; rg -n 'X-API-Key' docs/portal/README.md docs/portal/docs/index.md docs/portal/docs/authentication.md &amp;&amp; rg -n 'Authorization' docs/portal/README.md docs/portal/docs/index.md docs/portal/docs/authentication.md &amp;&amp; rg -n 'unsupported' docs/portal/README.md docs/portal/docs/index.md docs/portal/docs/authentication.md &amp;&amp; rg -n 'Full Spectrum' docs/portal/README.md docs/portal/docs/index.md docs/portal/docs/authentication.md</automated></verify>
  <done>Portal shell and authentication guidance are contract-parity aligned.</done>
</task>
<task type="auto">
  <name>Task 2: Align portal engine and workflow indexes</name>
  <files>docs/portal/docs/engines/index.md, docs/portal/docs/workflows/index.md</files>
  <read_first>docs/portal/docs/engines/index.md, docs/portal/docs/workflows/index.md, contracts/v1/registries/engines.json, contracts/v1/schemas/engine-capability-list.schema.json, contracts/v1/schemas/workflow-outcome.schema.json, contracts/v1/fixtures/workflow-outcome-unsupported.json</read_first>
  <action>Derive the engine index, public mirror projection, supported operations and workflow status from `contracts/v1`. Update portal navigation to cover all canonical engine identities and 17 public mirrors, preserve four availability-state language, and link Full Spectrum as explicitly unsupported rather than executable. Keep index links to the maintained pages and do not invent operation names or routes.</action>
  <acceptance_criteria>Portal indexes expose complete canonical navigation, exact 19/17 counts and explicit unsupported Full Spectrum status.</acceptance_criteria>
  <verify><automated>rg -n '19' docs/portal/docs/engines/index.md docs/portal/docs/workflows/index.md &amp;&amp; rg -n '17' docs/portal/docs/engines/index.md docs/portal/docs/workflows/index.md &amp;&amp; rg -n 'unsupported' docs/portal/docs/engines/index.md docs/portal/docs/workflows/index.md &amp;&amp; rg -n 'Full Spectrum' docs/portal/docs/engines/index.md docs/portal/docs/workflows/index.md &amp;&amp; rg -n 'calculate' docs/portal/docs/engines/index.md docs/portal/docs/workflows/index.md &amp;&amp; rg -n 'workflow' docs/portal/docs/engines/index.md docs/portal/docs/workflows/index.md</automated></verify>
  <done>Portal engine and workflow indexes reflect canonical catalogue and support metadata.</done>
</task>
</tasks>
<threat_model>
## Assets
Portal navigation, authentication guidance, engine inventory and workflow support claims.
## Trust Boundaries
Canonical manifest/registry/schema data crosses into public documentation and client navigation.
## Abuse Cases
Users follow stale counts, omit required auth or invoke an unsupported workflow from a portal link.
## STRIDE Threat Register
| Threat | Category | Disposition | Blocking test |
|---|---|---|---|
| T3-01 | Spoofing | mitigate | Header/auth parity receipt. |
| T3-02 | Information Disclosure | mitigate | No credential values in docs assertions. |
| T3-03 | Tampering/Availability | mitigate | 19/17/state parity receipt. |
| T3-04 | Tampering | mitigate | Index links and operation names checked against registry. |
| T3-05 | Improper Inventory | mitigate | Canonical registry count receipt. |
| T3-06 | Elevation of Privilege | mitigate | Authenticated route guidance receipt. |
| T3-07 | Tampering | mitigate | Workflow schema support receipt. |
| T3-08 | Tampering | mitigate | Full Spectrum unsupported index assertion. |
</threat_model>
<verification>Run focused portal shell/index parity receipts against contract metadata.</verification>
<success_criteria>Portal shell, authentication, engine index and workflow index tell the same canonical story as `contracts/v1`.</success_criteria>
<output>Create `.planning/phases/03-capability-and-contract-closure/03-20c-SUMMARY.md`.</output>
