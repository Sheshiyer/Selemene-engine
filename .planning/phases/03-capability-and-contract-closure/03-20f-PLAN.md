---
phase: 03-capability-and-contract-closure
plan: "20f"
type: execute
wave: 11
depends_on: ["03-20"]
files_modified:
  - docs/portal/docs/workflows/birth-blueprint.md
  - docs/portal/docs/workflows/creative-expression.md
  - docs/portal/docs/workflows/daily-practice.md
  - docs/portal/docs/workflows/decision-support.md
  - docs/portal/docs/workflows/full-spectrum.md
  - docs/portal/docs/workflows/self-inquiry.md
autonomous: true
requirements: [CON-02]
must_haves:
  truths:
    - "Every maintained portal workflow page states support and outcome semantics from the canonical workflow registry/schema."
    - "Full Spectrum is explicitly unsupported on its page and cannot be mistaken for an executable workflow."
  artifacts:
    - path: docs/portal/docs/workflows/full-spectrum.md
      provides: "Explicit unsupported Full Spectrum documentation"
    - path: docs/portal/docs/workflows/birth-blueprint.md
      provides: "Canonical supported workflow documentation"
  key_links:
    - from: docs/portal/docs/workflows/full-spectrum.md
      to: contracts/v1/fixtures/workflow-outcome-unsupported.json
      via: "unsupported outcome status"
---

<objective>Align every maintained portal workflow page with canonical workflow support and outcome metadata.</objective>
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
@contracts/v1/schemas/workflow-outcome.schema.json
@contracts/v1/fixtures/workflow-outcome-complete.json
@contracts/v1/fixtures/workflow-outcome-partial.json
@contracts/v1/fixtures/workflow-outcome-failed.json
@contracts/v1/fixtures/workflow-outcome-unsupported.json
@docs/portal/docs/workflows/index.md
@docs/portal/docs/workflows/birth-blueprint.md
@docs/portal/docs/workflows/creative-expression.md
@docs/portal/docs/workflows/daily-practice.md
@docs/portal/docs/workflows/decision-support.md
@docs/portal/docs/workflows/full-spectrum.md
@docs/portal/docs/workflows/self-inquiry.md
</context>
<tasks>
<task type="auto">
  <name>Task 1: Align supported workflow pages</name>
  <files>docs/portal/docs/workflows/birth-blueprint.md, docs/portal/docs/workflows/creative-expression.md, docs/portal/docs/workflows/daily-practice.md</files>
  <read_first>docs/portal/docs/workflows/birth-blueprint.md, docs/portal/docs/workflows/creative-expression.md, docs/portal/docs/workflows/daily-practice.md, contracts/v1/manifest.json, contracts/v1/registries/engines.json, contracts/v1/schemas/workflow-outcome.schema.json, contracts/v1/fixtures/workflow-outcome-complete.json, contracts/v1/fixtures/workflow-outcome-partial.json</read_first>
  <action>Derive each workflow's supported engine set, operation/auth requirements, output/failure semantics and status vocabulary from the canonical registry, manifest and workflow-outcome fixtures. Update the three pages to describe actual producer-backed authenticated workflow support, preserve failure/conservation semantics and state 19 runtime identities versus 17 public mirrors where relevant. Remove stale totals and any claim that Full Spectrum is executable.</action>
  <acceptance_criteria>These supported workflow pages describe canonical inputs, auth, outcome statuses and engine projections without unsupported Full Spectrum execution claims.</acceptance_criteria>
  <verify><automated>rg -n 'workflow-outcome' docs/portal/docs/workflows/birth-blueprint.md docs/portal/docs/workflows/creative-expression.md docs/portal/docs/workflows/daily-practice.md &amp;&amp; rg -n 'complete' docs/portal/docs/workflows/birth-blueprint.md docs/portal/docs/workflows/creative-expression.md docs/portal/docs/workflows/daily-practice.md &amp;&amp; rg -n 'partial' docs/portal/docs/workflows/birth-blueprint.md docs/portal/docs/workflows/creative-expression.md docs/portal/docs/workflows/daily-practice.md &amp;&amp; rg -n 'failed' docs/portal/docs/workflows/birth-blueprint.md docs/portal/docs/workflows/creative-expression.md docs/portal/docs/workflows/daily-practice.md &amp;&amp; rg -n '19' docs/portal/docs/workflows/birth-blueprint.md docs/portal/docs/workflows/creative-expression.md docs/portal/docs/workflows/daily-practice.md &amp;&amp; rg -n '17' docs/portal/docs/workflows/birth-blueprint.md docs/portal/docs/workflows/creative-expression.md docs/portal/docs/workflows/daily-practice.md &amp;&amp; rg -n 'X-API-Key' docs/portal/docs/workflows/birth-blueprint.md docs/portal/docs/workflows/creative-expression.md docs/portal/docs/workflows/daily-practice.md &amp;&amp; rg -n 'Authorization' docs/portal/docs/workflows/birth-blueprint.md docs/portal/docs/workflows/creative-expression.md docs/portal/docs/workflows/daily-practice.md &amp;&amp; rg -n 'unsupported' docs/portal/docs/workflows/birth-blueprint.md docs/portal/docs/workflows/creative-expression.md docs/portal/docs/workflows/daily-practice.md</automated></verify>
  <done>Supported workflow documentation matches producer and contract semantics.</done>
</task>
<task type="auto">
  <name>Task 2: Align decision, Full Spectrum and self-inquiry pages</name>
  <files>docs/portal/docs/workflows/decision-support.md, docs/portal/docs/workflows/full-spectrum.md, docs/portal/docs/workflows/self-inquiry.md</files>
  <read_first>docs/portal/docs/workflows/decision-support.md, docs/portal/docs/workflows/full-spectrum.md, docs/portal/docs/workflows/self-inquiry.md, docs/portal/docs/workflows/index.md, contracts/v1/manifest.json, contracts/v1/registries/engines.json, contracts/v1/schemas/workflow-outcome.schema.json, contracts/v1/fixtures/workflow-outcome-unsupported.json, crates/noesis-orchestrator/src/workflow/registry.rs, crates/noesis-orchestrator/src/workflow/synthesis/mod.rs</read_first>
  <action>Derive workflow support, engine projections, auth requirements and outcome statuses from canonical metadata and actual `WorkflowRegistry`/synthesis modules. Update decision-support and self-inquiry with producer-backed semantics; rewrite Full Spectrum as explicitly `unsupported` pending a lossless adapter test, with no executable route or promise. Preserve portal links and state 19 runtime identities/17 public mirrors where catalogue claims appear.</action>
  <acceptance_criteria>Decision support and self-inquiry pages match canonical producer support; Full Spectrum is visibly unsupported, and no page documents an invented execution route.</acceptance_criteria>
  <verify><automated>rg -n 'workflow-outcome' docs/portal/docs/workflows/decision-support.md docs/portal/docs/workflows/full-spectrum.md docs/portal/docs/workflows/self-inquiry.md &amp;&amp; rg -n 'unsupported' docs/portal/docs/workflows/decision-support.md docs/portal/docs/workflows/full-spectrum.md docs/portal/docs/workflows/self-inquiry.md &amp;&amp; rg -n 'Full Spectrum' docs/portal/docs/workflows/decision-support.md docs/portal/docs/workflows/full-spectrum.md docs/portal/docs/workflows/self-inquiry.md &amp;&amp; rg -n '19' docs/portal/docs/workflows/decision-support.md docs/portal/docs/workflows/full-spectrum.md docs/portal/docs/workflows/self-inquiry.md &amp;&amp; rg -n '17' docs/portal/docs/workflows/decision-support.md docs/portal/docs/workflows/full-spectrum.md docs/portal/docs/workflows/self-inquiry.md &amp;&amp; rg -n 'X-API-Key' docs/portal/docs/workflows/decision-support.md docs/portal/docs/workflows/full-spectrum.md docs/portal/docs/workflows/self-inquiry.md &amp;&amp; rg -n 'Authorization' docs/portal/docs/workflows/decision-support.md docs/portal/docs/workflows/full-spectrum.md docs/portal/docs/workflows/self-inquiry.md</automated></verify>
  <done>Decision-support, self-inquiry, and Full Spectrum pages state truthful support and unsupported semantics.</done>
</task>
</tasks>
<threat_model>
## Assets
Workflow support claims, engine projections, auth guidance and outcome status semantics.
## Trust Boundaries
Canonical workflow schema/fixtures and producer registry cross into public workflow documentation.
## Abuse Cases
Documentation causes unsupported execution, hides failures or confuses semantic output with engine availability.
## STRIDE Threat Register
| Threat | Category | Disposition | Blocking test |
|---|---|---|---|
| T3-01 | Spoofing | mitigate | Auth guidance receipt. |
| T3-02 | Information Disclosure | mitigate | Bounded outcome/error guidance receipt. |
| T3-03 | Tampering/Availability | mitigate | Outcome status and conservation parity receipt. |
| T3-04 | Tampering | mitigate | Workflow/engine operations checked against registry. |
| T3-05 | Improper Inventory | mitigate | 19/17 projection assertions. |
| T3-06 | Elevation of Privilege | mitigate | Authenticated workflow route guidance. |
| T3-07 | Tampering | mitigate | Complete/partial/failed fixture parity. |
| T3-08 | Tampering | mitigate | Full Spectrum unsupported and no invented route assertions. |
</threat_model>
<verification>Run both focused portal workflow parity receipts against canonical workflow metadata and fixtures.</verification>
<success_criteria>All seven maintained workflow pages state producer-backed support and explicitly unsupported Full Spectrum semantics.</success_criteria>
<output>Create `.planning/phases/03-capability-and-contract-closure/03-20f-SUMMARY.md`.</output>
