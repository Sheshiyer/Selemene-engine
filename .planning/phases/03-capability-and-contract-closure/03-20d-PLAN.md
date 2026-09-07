---
phase: 03-capability-and-contract-closure
plan: "20d"
type: execute
wave: 11
depends_on: ["03-20"]
files_modified:
  - docs/portal/docs/engines/biofield.md
  - docs/portal/docs/engines/biorhythm.md
  - docs/portal/docs/engines/enneagram.md
  - docs/portal/docs/engines/face-reading.md
  - docs/portal/docs/engines/gene-keys.md
  - docs/portal/docs/engines/human-design.md
  - docs/portal/docs/engines/i-ching.md
  - docs/portal/docs/engines/nadabrahman.md
autonomous: true
requirements: [CON-02]
must_haves:
  truths:
    - "The first maintained portal engine pages describe operation, authentication and availability from canonical metadata."
    - "Engine pages do not claim unsupported workflow execution or stale catalogue totals."
  artifacts:
    - path: docs/portal/docs/engines/biofield.md
      provides: "Canonical biofield engine documentation"
    - path: docs/portal/docs/engines/nadabrahman.md
      provides: "Canonical nadabrahman engine documentation"
  key_links:
    - from: docs/portal/docs/engines/biofield.md
      to: contracts/v1/registries/engines.json
      via: "canonical identity and operation metadata"
---

<objective>Align the first portal engine-page slice with canonical identity, operation, auth and availability metadata.</objective>
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
@docs/portal/docs/engines/biofield.md
@docs/portal/docs/engines/biorhythm.md
@docs/portal/docs/engines/enneagram.md
@docs/portal/docs/engines/face-reading.md
@docs/portal/docs/engines/gene-keys.md
@docs/portal/docs/engines/human-design.md
@docs/portal/docs/engines/i-ching.md
@docs/portal/docs/engines/nadabrahman.md
</context>
<tasks>
<task type="auto">
  <name>Task 1: Align biofield through face-reading pages</name>
  <files>docs/portal/docs/engines/biofield.md, docs/portal/docs/engines/biorhythm.md, docs/portal/docs/engines/enneagram.md, docs/portal/docs/engines/face-reading.md</files>
  <read_first>docs/portal/docs/engines/biofield.md, docs/portal/docs/engines/biorhythm.md, docs/portal/docs/engines/enneagram.md, docs/portal/docs/engines/face-reading.md, contracts/v1/registries/engines.json, contracts/v1/manifest.json, contracts/v1/schemas/engine-capability-list.schema.json</read_first>
  <action>For each named identity, derive the canonical display name, calculate operation, public-mirror membership, authentication requirement and availability-state language from `contracts/v1`. Update the page claims and examples to match those values, retain truthful input/output descriptions, and remove stale inventory or executable Full Spectrum references. Keep route examples on established authenticated Rust paths.</action>
  <acceptance_criteria>All four pages have canonical identity/operation/auth claims, state availability accurately and contain no stale count or unsupported workflow execution claim.</acceptance_criteria>
  <verify><automated>rg -n 'biofield' docs/portal/docs/engines/biofield.md docs/portal/docs/engines/biorhythm.md docs/portal/docs/engines/enneagram.md docs/portal/docs/engines/face-reading.md &amp;&amp; rg -n 'biorhythm' docs/portal/docs/engines/biofield.md docs/portal/docs/engines/biorhythm.md docs/portal/docs/engines/enneagram.md docs/portal/docs/engines/face-reading.md &amp;&amp; rg -n 'enneagram' docs/portal/docs/engines/biofield.md docs/portal/docs/engines/biorhythm.md docs/portal/docs/engines/enneagram.md docs/portal/docs/engines/face-reading.md &amp;&amp; rg -n 'face-reading' docs/portal/docs/engines/biofield.md docs/portal/docs/engines/biorhythm.md docs/portal/docs/engines/enneagram.md docs/portal/docs/engines/face-reading.md &amp;&amp; rg -n '19' docs/portal/docs/engines/biofield.md docs/portal/docs/engines/biorhythm.md docs/portal/docs/engines/enneagram.md docs/portal/docs/engines/face-reading.md &amp;&amp; rg -n '17' docs/portal/docs/engines/biofield.md docs/portal/docs/engines/biorhythm.md docs/portal/docs/engines/enneagram.md docs/portal/docs/engines/face-reading.md &amp;&amp; rg -n 'X-API-Key' docs/portal/docs/engines/biofield.md docs/portal/docs/engines/biorhythm.md docs/portal/docs/engines/enneagram.md docs/portal/docs/engines/face-reading.md &amp;&amp; rg -n 'Authorization' docs/portal/docs/engines/biofield.md docs/portal/docs/engines/biorhythm.md docs/portal/docs/engines/enneagram.md docs/portal/docs/engines/face-reading.md &amp;&amp; rg -n 'unsupported' docs/portal/docs/engines/biofield.md docs/portal/docs/engines/biorhythm.md docs/portal/docs/engines/enneagram.md docs/portal/docs/engines/face-reading.md</automated></verify>
  <done>Biofield through face-reading pages are contract-parity aligned.</done>
</task>
<task type="auto">
  <name>Task 2: Align gene-keys through nadabrahman pages</name>
  <files>docs/portal/docs/engines/gene-keys.md, docs/portal/docs/engines/human-design.md, docs/portal/docs/engines/i-ching.md, docs/portal/docs/engines/nadabrahman.md</files>
  <read_first>docs/portal/docs/engines/gene-keys.md, docs/portal/docs/engines/human-design.md, docs/portal/docs/engines/i-ching.md, docs/portal/docs/engines/nadabrahman.md, contracts/v1/registries/engines.json, contracts/v1/manifest.json, contracts/v1/schemas/engine-capability-list.schema.json</read_first>
  <action>Derive each page's canonical identity, supported operations, public projection and auth/state semantics from `contracts/v1`; update examples and support language accordingly. Preserve the established engine route and bounded error guidance, and remove stale catalogue or Full Spectrum execution wording.</action>
  <acceptance_criteria>All four pages agree with registry operation/auth/state metadata and do not document unsupported workflow execution.</acceptance_criteria>
  <verify><automated>rg -n 'gene-keys' docs/portal/docs/engines/gene-keys.md docs/portal/docs/engines/human-design.md docs/portal/docs/engines/i-ching.md docs/portal/docs/engines/nadabrahman.md &amp;&amp; rg -n 'human-design' docs/portal/docs/engines/gene-keys.md docs/portal/docs/engines/human-design.md docs/portal/docs/engines/i-ching.md docs/portal/docs/engines/nadabrahman.md &amp;&amp; rg -n 'i-ching' docs/portal/docs/engines/gene-keys.md docs/portal/docs/engines/human-design.md docs/portal/docs/engines/i-ching.md docs/portal/docs/engines/nadabrahman.md &amp;&amp; rg -n 'nadabrahman' docs/portal/docs/engines/gene-keys.md docs/portal/docs/engines/human-design.md docs/portal/docs/engines/i-ching.md docs/portal/docs/engines/nadabrahman.md &amp;&amp; rg -n '19' docs/portal/docs/engines/gene-keys.md docs/portal/docs/engines/human-design.md docs/portal/docs/engines/i-ching.md docs/portal/docs/engines/nadabrahman.md &amp;&amp; rg -n '17' docs/portal/docs/engines/gene-keys.md docs/portal/docs/engines/human-design.md docs/portal/docs/engines/i-ching.md docs/portal/docs/engines/nadabrahman.md &amp;&amp; rg -n 'X-API-Key' docs/portal/docs/engines/gene-keys.md docs/portal/docs/engines/human-design.md docs/portal/docs/engines/i-ching.md docs/portal/docs/engines/nadabrahman.md &amp;&amp; rg -n 'Authorization' docs/portal/docs/engines/gene-keys.md docs/portal/docs/engines/human-design.md docs/portal/docs/engines/i-ching.md docs/portal/docs/engines/nadabrahman.md &amp;&amp; rg -n 'unsupported' docs/portal/docs/engines/gene-keys.md docs/portal/docs/engines/human-design.md docs/portal/docs/engines/i-ching.md docs/portal/docs/engines/nadabrahman.md</automated></verify>
  <done>Gene Keys through Nadabrahman pages are contract-parity aligned.</done>
</task>
</tasks>
<threat_model>
## Assets
Engine identity pages, operation/auth examples and availability claims.
## Trust Boundaries
Canonical registry and manifest data cross into public per-engine documentation.
## Abuse Cases
A page teaches a stale route, hides a runtime identity or turns a state into false availability.
## STRIDE Threat Register
| Threat | Category | Disposition | Blocking test |
|---|---|---|---|
| T3-01 | Spoofing | mitigate | Auth wording receipt. |
| T3-02 | Information Disclosure | mitigate | Bounded error guidance receipt. |
| T3-03 | Tampering/Availability | mitigate | State and identity parity receipt. |
| T3-04 | Tampering | mitigate | Operation/route names checked against registry. |
| T3-05 | Improper Inventory | mitigate | Canonical page identity count receipt. |
| T3-06 | Elevation of Privilege | mitigate | Protected-route-only examples. |
| T3-07 | Tampering | mitigate | Workflow support claims checked against schema. |
| T3-08 | Tampering | mitigate | Unsupported Full Spectrum claims rejected. |
</threat_model>
<verification>Run focused portal engine page parity receipts for both page groups.</verification>
<success_criteria>The first eight portal engine pages accurately reflect canonical operation, auth, state and support metadata.</success_criteria>
<output>Create `.planning/phases/03-capability-and-contract-closure/03-20d-SUMMARY.md`.</output>
