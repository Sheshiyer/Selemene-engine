---
phase: 03-capability-and-contract-closure
plan: "20e"
type: execute
wave: 11
depends_on: ["03-20"]
files_modified:
  - docs/portal/docs/engines/numerology.md
  - docs/portal/docs/engines/panchanga.md
  - docs/portal/docs/engines/sacred-geometry.md
  - docs/portal/docs/engines/sigil-forge.md
  - docs/portal/docs/engines/tarot.md
  - docs/portal/docs/engines/transits.md
  - docs/portal/docs/engines/vedic-clock.md
  - docs/portal/docs/engines/vimshottari.md
autonomous: true
requirements: [CON-02]
must_haves:
  truths:
    - "The remaining maintained portal engine pages describe canonical identity, operation, authentication and availability semantics."
    - "The complete portal engine page set has no stale inventory totals or unsupported workflow execution claim."
  artifacts:
    - path: docs/portal/docs/engines/numerology.md
      provides: "Canonical numerology engine documentation"
    - path: docs/portal/docs/engines/vimshottari.md
      provides: "Canonical vimshottari engine documentation"
  key_links:
    - from: docs/portal/docs/engines/vimshottari.md
      to: contracts/v1/registries/engines.json
      via: "canonical identity and operation metadata"
---

<objective>Align the remaining portal engine pages with canonical identity, operation, auth and availability metadata.</objective>
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
@docs/portal/docs/engines/numerology.md
@docs/portal/docs/engines/panchanga.md
@docs/portal/docs/engines/sacred-geometry.md
@docs/portal/docs/engines/sigil-forge.md
@docs/portal/docs/engines/tarot.md
@docs/portal/docs/engines/transits.md
@docs/portal/docs/engines/vedic-clock.md
@docs/portal/docs/engines/vimshottari.md
</context>
<tasks>
<task type="auto">
  <name>Task 1: Align numerology through sigil-forge pages</name>
  <files>docs/portal/docs/engines/numerology.md, docs/portal/docs/engines/panchanga.md, docs/portal/docs/engines/sacred-geometry.md, docs/portal/docs/engines/sigil-forge.md</files>
  <read_first>docs/portal/docs/engines/numerology.md, docs/portal/docs/engines/panchanga.md, docs/portal/docs/engines/sacred-geometry.md, docs/portal/docs/engines/sigil-forge.md, contracts/v1/registries/engines.json, contracts/v1/manifest.json, contracts/v1/schemas/engine-capability-list.schema.json</read_first>
  <action>Derive each named page's canonical identity, operation, public-mirror membership, authentication and availability-state language from `contracts/v1`. Update claims and examples to match the registry, retain truthful engine-specific input/output guidance, and remove stale inventory or executable Full Spectrum references. Keep route examples on established authenticated Rust paths.</action>
  <acceptance_criteria>All four pages have canonical operation/auth/state claims and no stale count or unsupported workflow execution claim.</acceptance_criteria>
  <verify><automated>rg -n 'numerology' docs/portal/docs/engines/numerology.md docs/portal/docs/engines/panchanga.md docs/portal/docs/engines/sacred-geometry.md docs/portal/docs/engines/sigil-forge.md &amp;&amp; rg -n 'panchanga' docs/portal/docs/engines/numerology.md docs/portal/docs/engines/panchanga.md docs/portal/docs/engines/sacred-geometry.md docs/portal/docs/engines/sigil-forge.md &amp;&amp; rg -n 'sacred-geometry' docs/portal/docs/engines/numerology.md docs/portal/docs/engines/panchanga.md docs/portal/docs/engines/sacred-geometry.md docs/portal/docs/engines/sigil-forge.md &amp;&amp; rg -n 'sigil-forge' docs/portal/docs/engines/numerology.md docs/portal/docs/engines/panchanga.md docs/portal/docs/engines/sacred-geometry.md docs/portal/docs/engines/sigil-forge.md &amp;&amp; rg -n '19' docs/portal/docs/engines/numerology.md docs/portal/docs/engines/panchanga.md docs/portal/docs/engines/sacred-geometry.md docs/portal/docs/engines/sigil-forge.md &amp;&amp; rg -n '17' docs/portal/docs/engines/numerology.md docs/portal/docs/engines/panchanga.md docs/portal/docs/engines/sacred-geometry.md docs/portal/docs/engines/sigil-forge.md &amp;&amp; rg -n 'X-API-Key' docs/portal/docs/engines/numerology.md docs/portal/docs/engines/panchanga.md docs/portal/docs/engines/sacred-geometry.md docs/portal/docs/engines/sigil-forge.md &amp;&amp; rg -n 'Authorization' docs/portal/docs/engines/numerology.md docs/portal/docs/engines/panchanga.md docs/portal/docs/engines/sacred-geometry.md docs/portal/docs/engines/sigil-forge.md &amp;&amp; rg -n 'unsupported' docs/portal/docs/engines/numerology.md docs/portal/docs/engines/panchanga.md docs/portal/docs/engines/sacred-geometry.md docs/portal/docs/engines/sigil-forge.md</automated></verify>
  <done>Numerology through Sigil Forge pages are contract-parity aligned.</done>
</task>
<task type="auto">
  <name>Task 2: Align tarot through vimshottari pages</name>
  <files>docs/portal/docs/engines/tarot.md, docs/portal/docs/engines/transits.md, docs/portal/docs/engines/vedic-clock.md, docs/portal/docs/engines/vimshottari.md</files>
  <read_first>docs/portal/docs/engines/tarot.md, docs/portal/docs/engines/transits.md, docs/portal/docs/engines/vedic-clock.md, docs/portal/docs/engines/vimshottari.md, contracts/v1/registries/engines.json, contracts/v1/manifest.json, contracts/v1/schemas/engine-capability-list.schema.json</read_first>
  <action>Derive each page's canonical identity, supported operations, public projection and auth/state semantics from `contracts/v1`; update examples and support language accordingly. Preserve the established engine route and bounded error guidance, and remove stale catalogue or Full Spectrum execution wording.</action>
  <acceptance_criteria>All four pages agree with registry operation/auth/state metadata and do not document unsupported workflow execution.</acceptance_criteria>
  <verify><automated>rg -n 'tarot' docs/portal/docs/engines/tarot.md docs/portal/docs/engines/transits.md docs/portal/docs/engines/vedic-clock.md docs/portal/docs/engines/vimshottari.md &amp;&amp; rg -n 'transits' docs/portal/docs/engines/tarot.md docs/portal/docs/engines/transits.md docs/portal/docs/engines/vedic-clock.md docs/portal/docs/engines/vimshottari.md &amp;&amp; rg -n 'vedic-clock' docs/portal/docs/engines/tarot.md docs/portal/docs/engines/transits.md docs/portal/docs/engines/vedic-clock.md docs/portal/docs/engines/vimshottari.md &amp;&amp; rg -n 'vimshottari' docs/portal/docs/engines/tarot.md docs/portal/docs/engines/transits.md docs/portal/docs/engines/vedic-clock.md docs/portal/docs/engines/vimshottari.md &amp;&amp; rg -n '19' docs/portal/docs/engines/tarot.md docs/portal/docs/engines/transits.md docs/portal/docs/engines/vedic-clock.md docs/portal/docs/engines/vimshottari.md &amp;&amp; rg -n '17' docs/portal/docs/engines/tarot.md docs/portal/docs/engines/transits.md docs/portal/docs/engines/vedic-clock.md docs/portal/docs/engines/vimshottari.md &amp;&amp; rg -n 'X-API-Key' docs/portal/docs/engines/tarot.md docs/portal/docs/engines/transits.md docs/portal/docs/engines/vedic-clock.md docs/portal/docs/engines/vimshottari.md &amp;&amp; rg -n 'Authorization' docs/portal/docs/engines/tarot.md docs/portal/docs/engines/transits.md docs/portal/docs/engines/vedic-clock.md docs/portal/docs/engines/vimshottari.md &amp;&amp; rg -n 'unsupported' docs/portal/docs/engines/tarot.md docs/portal/docs/engines/transits.md docs/portal/docs/engines/vedic-clock.md docs/portal/docs/engines/vimshottari.md</automated></verify>
  <done>Tarot through Vimshottari pages are contract-parity aligned.</done>
</task>
</tasks>
<threat_model>
## Assets
Remaining engine identity pages, operation/auth examples and availability claims.
## Trust Boundaries
Canonical registry and manifest data cross into public per-engine documentation.
## Abuse Cases
A page teaches a stale route, hides an identity or claims an unsupported workflow is executable.
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
<verification>Run focused portal engine page parity receipts for both remaining page groups.</verification>
<success_criteria>All remaining portal engine pages accurately reflect canonical operation, auth, state and support metadata.</success_criteria>
<output>Create `.planning/phases/03-capability-and-contract-closure/03-20e-SUMMARY.md`.</output>
