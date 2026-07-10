# Relationship Mapping Contract — L0-L5 Dyad & Synastry Implementation Plan

> **Status:** Authoritative. Supersedes partial mentions in 2026-07-04 rubric plan and archive ports.
> **Date:** 2026-07-10
> **For Codex:** Use superpowers:executing-plans + the dispatching-parallel-agents skill when 3+ independent domains appear. Always read this plan before touching intake, modes, orchestrator, rubric, or Folio B-surface.

**Goal:** Make every report path (L0-L5, synastry, dyad, penta) explicitly declare and propagate non-presumptive relationship semantics from intake form through mode docs, orchestrator prompts, rubric guardrails, tests, and B-surface (Folio) output headers — without ever assuming romance, marriage, or generic "partner" framing.

**Core Principle:** The form, the mode, the prompt, the rubric, and the rendered header must all be able to say "Mother-Son Lineage Mapping — non-predictive pattern witness" or "Business Partners Synergy Audit" with the same explicit roles and mapping goal that the caller supplied. No silent defaults to romantic dyad.

---

## Relationship Mapping Contract (Authoritative)

### 1. Canonical Relationship Types (relationship_context.type)

```ts
export const RELATIONSHIP_TYPES = [
  'family',              // lineage, parent-child, siblings, multi-gen, penta
  'friends',
  'business-partners',   // colleagues, co-founders, client-vendor, team
  'unmarried-partners',  // romantic but not married
  'married-partners',
  'custom'
] as const;

export type RelationshipType = typeof RELATIONSHIP_TYPES[number];
```

Mapping from legacy Rust `RelationshipMode` (for witness-dyad LLM path only):
- `CompositeDyad` → generic or custom
- `PartnerSynastry` → 'unmarried-partners' | 'married-partners' (explicit)
- `FamilyTriad` → 'family'
- `BusinessPartners` → 'business-partners'
- `None` → solo or custom

**Rust enum remains narrow for now.** The rich contract lives in the pipeline intake + mode system. Rust side receives a string or maps at the API boundary.

### 2. Subject Roles (per-subject, explicit, non-presumptive)

```ts
export const SUBJECT_ROLES = [
  // Generic
  'primary', 'partner',
  // Family / lineage
  'mother', 'father', 'parent', 'son', 'daughter', 'child',
  'sibling', 'brother', 'sister',
  'grandmother', 'grandfather', 'grandparent',
  // Family penta / extended (variable)
  'child1', 'child2', 'child3', 'child4',
  'aunt', 'uncle', 'cousin',
  // Business / team
  'business-partner', 'colleague', 'client', 'vendor', 'team-member', 'founder',
  // Social
  'friend', 'mentor', 'student',
  // Fallback
  'custom'
] as const;

export type SubjectRole = typeof SUBJECT_ROLES[number];
```

Each `ReportSubjectInput` carries:
- `role: SubjectRole`
- `relationship_label?: string`  // free-text override or qualifier, e.g. "eldest son", "co-founder & CTO"

### 3. Relationship Context (propagates from form to every layer)

```ts
export interface RelationshipContext {
  type: RelationshipType;
  mapping_goal: string;           // e.g. "understand lineage patterns without outcome prediction"
  sensitivity_level: 'low' | 'medium' | 'high';
}
```

**Propagation rules (mandatory):**
- Intake form → `ReportGenerationRequest.relationship_context` + per-subject `role` + `relationship_label`
- `ReportGenerationRequest` → orchestrator input (new fields: `relationshipContext`, `subjectRoles[]`)
- Mode doc frontmatter → may declare `roles: string[]` (authoritative for that mode) and optional `relationship_types: RelationshipType[]`
- Orchestrator render → every pass prompt receives:
  - `{{subject_roles}}` → e.g. "mother (Aarav), son (Vikram)"
  - `{{relationship_context}}` → JSON or structured block with type, mapping_goal, sensitivity
  - `{{relationship_header}}` → human string for B-surface, e.g. "Mother-Son Lineage Mapping"
- Rubric / guardrails → keyed by `relationship_context.type`:
  - 'family' → stricter "no outcome prediction", forbid "child will...", "parent will die"
  - 'business-partners' → forbid romantic language, investment guarantees
  - 'married-partners' / 'unmarried-partners' → existing love-marriage guardrails apply
- Parser + topology → 'pentagon' for family-penta (5 subjects), 'triad-triangle' for family-triad, 'dyad-arc' for pairs
- B-surface (Folio) header → MUST declare: `"{Explicit Relationship Header} — non-predictive pattern witness"`
  Example: "Mother-Son Lineage Mapping — non-predictive pattern witness"
- Source pack + API response → carry the original `relationship_context` and labeled subject list

**B-surface (Folio) specific contract (from SYSTEM.md + user confirmation):**
- Voice: parchment canvas, ink-bronze body, ink-iron headings, illuminated style.
- Header must appear at top of long-form reading, before any engine data.
- Must be declarative and non-presumptive: role names + mapping purpose + "non-predictive pattern witness".
- Never default to "Partners" or "Spouses" if the intake said mother/son or business-partners.

### 4. Family Penta Role Assignment Rules

- Minimum 3, typical 5 subjects for penta.
- Explicit roles required: e.g. mother, father, child1, child2, child3 (or named variants).
- `relationship_context.type = 'family'`
- `mapping_goal` must be supplied (e.g. "family field dynamics and transmission patterns").
- Topology in mode doc: `svg_topology: 'pentagon'`
- No automatic inference of birth order or gender roles; caller supplies.

### 5. Intake Form Presentation (no romance presumption)

`buildReportIntakeQuestions` + UI must:
- Allow selecting report type: Individual | Multi-subject Relationship Mapping
- For multi-subject: per-subject role picker (from SUBJECT_ROLES) + optional free-text `relationship_label`
- Separate `relationship_context` block:
  - Type selector (from RELATIONSHIP_TYPES)
  - Mapping goal (free text, required)
  - Sensitivity (low/medium/high)
- Never pre-select "partner" or "married" as default for 2-person.
- For 2-person mother-son or business, the form must let the caller choose those roles explicitly.

---

## Current State vs Requirement (Gap Summary — 2026-07-10)

- `RelationshipMode` (Rust) exists but labels are cosmetic; only used in witness-dyad path.
- `SubjectRole` + `relationship_context` in intake/types.ts exist but **never consumed** by questions, parser, orchestrator, rubric, or B-surface.
- Mode fixtures only declare `roles: ['subject-a', 'subject-b']`.
- Orchestrator only substitutes `{{subject_names}}`.
- Rubric guardrails are generic.
- No tests for mother-son, business dyad, family-penta.
- L0-L5 plan (07-04) acknowledges subjects[] but does not model relationship semantics.
- Witness-agents history had the richer taxonomy; Selemene side does not.

**Blocking:** Cannot claim any L0-L5 dyad or synastry support is complete until this contract is wired end-to-end.

---

## Phases & Task List (Updated)

### Phase 0 — Plan & Contract (this document)
- [x] Write this plan with explicit taxonomy, roles, propagation, B-header contract.
- [ ] Re-audit after Phase 1-3 before claiming "done".

### Phase 1 — Intake Form & Canonical Taxonomy (COMPLETE 2026-07-10)
1. ✅ Add canonical consts + types to `packages/witness-pipeline/src/intake/types.ts` (RELATIONSHIP_TYPES, SUBJECT_ROLES, strengthened interfaces, RelationshipContext type, builder helpers).
2. ✅ Extend `buildReportIntakeQuestions` (and its test) to surface Relationship Type, Mapping Goal, Sensitivity, and per-subject Role + relationship_label questions. No romantic defaults.
3. ✅ Update `ReportGenerationRequest` handling — TS now exports RelationshipContext; Rust core + API already carry the fields (no crash on rich shape); test contract updated to use 'unmarried-partners'.
4. ✅ Added unit tests in types.test.ts and questions.test.ts that validate mother-son, business-dyad, family-penta request shapes and that builders produce correct roles/context.
5. ✅ All witness-pipeline tests green (78/78). Intake now produces explicit non-presumptive requests. Phase 1 verification passed.

**Artifacts:**
- `packages/witness-pipeline/src/intake/types.ts` (consts + builders)
- `packages/witness-pipeline/src/intake/questions.ts` + `questions.test.ts`
- `packages/witness-pipeline/src/intake/types.test.ts` (new cases)
- Rust test contract updated in `crates/noesis-api/tests/assets_generate_contract_test.rs`

### Phase 2 — Mode Docs & Parser (COMPLETE 2026-07-10)
1. ✅ Extended `ModeConfig` with optional `relationship_types?: string[]`.
2. ✅ Created conforming minimal mode docs:
   - `packages/witness-pipeline/modes/mother-son-lineage.md` (roles: mother/son, family, dyad-arc, L2)
   - `packages/witness-pipeline/modes/business-partners.md` (roles: business-partner x2, business-partners, dyad-arc, L2)
   - `packages/witness-pipeline/modes/family-penta.md` (roles: mother/father/child1-3, family, pentagon, L3)
3. ✅ Parser (`assertModeConfig`) now validates roles against SUBJECT_ROLES (plus 'custom' + legacy aliases 'subject','subject-a','subject-b' for compat).
4. ✅ Added parser tests for mother-son mode + rejection of unknown roles. All 82 tests green.

Legacy composite-dyad / integrated-reading fixtures continue to load (subject-a/b/subject aliases added).

### Phase 3 — Orchestrator Injection + Prompt Templates (COMPLETE 2026-07-10)
1. ✅ Extended `OrchestratorInput` with `subjectRoles?: SubjectRoleInfo[]` and `relationshipContext?: RelationshipContextInfo`.
2. ✅ `renderPassTemplate` now substitutes:
   - `{{subject_roles}}` → "Aarav (mother), Vikram (son)"
   - `{{relationship_header}}` → "mother-son family — non-predictive pattern witness"
   - `{{relationship_context}}`, `{{mapping_goal}}`
3. ✅ `buildSystemPrompt` includes roles and relationship context lines.
4. ✅ Added integrated.test.ts case that feeds mother-son request + context and asserts token presence in rendered prompt.
5. Existing templates that do not yet use the tokens are unaffected (additive). Dual-subject routing unchanged. All tests green.

Follow-up (later phase): update canonical pass templates in mode docs to include the new tokens by default.

### Phase 4 — Rubric Guardrails by Type (COMPLETE 2026-07-10)
1. ✅ Extended `AuditSectionInput` with `relationshipType?`.
2. ✅ Added `RELATIONSHIP_GUARDRAILS` map (family, business-partners, unmarried/married-partners).
3. ✅ `auditSectionOutput` now unions base section guardrails + relationship-type patterns when `relationshipType` provided.
4. ✅ Added rubric.test.ts cases for family (child/parent outcome language) and business-partners (romantic + guarantee language) — both fail as expected.
5. ✅ Orchestrator call site passes `relationshipType`; e2e tests (via parallel agents) feed relationshipContext and assert rubric behavior + guardrail pass on clean outputs.
6. ✅ Full package suite green (87 tests).

Status: relationship-scoped guardrails are live, tested, and exercised from intake → orchestrator → rubric.

### Phase 5 — Tests & Fixtures (Mother-Son, Business Dyad, Family-Penta)
1. Create `tests/fixtures/mother-son-lineage.md` (or use inline parse docs).
2. Parser + orchestrator tests that feed explicit roles + context and assert substitution + rubric behavior.
3. E2E-lite test that runs a 2-subject mother-son L2 and a 2-subject business L2 and a 5-subject family-penta L3 (mocked engines) and checks header + prompt fragments + guardrails.
4. Update composite-dyad tests to also assert relationship_context passthrough.

### Phase 6 — B-Surface (Folio) Header Contract
1. In the report renderer / asset response assembler that produces Folio long-form, prepend or structure the header using `relationship_header` + labeled subjects.
2. Example output start:
   ```
   Mother-Son Lineage Mapping — non-predictive pattern witness

   Subjects: Aarav (mother), Vikram (son)
   Mapping goal: understand transmission patterns without outcome prediction
   Sensitivity: high
   ```
3. Verify in visual or snapshot tests that the header appears and uses correct Folio typography.

### Phase 7 — Rust/API Boundary + Witness Dyad Alignment
1. Ensure `noesis-api` AssetGenerateRequest + WitnessInterpretRequest can carry the rich relationship fields.
2. Map incoming `relationship_context.type` to the narrow `RelationshipMode` where the LLM dyad path is used.
3. Add contract test that round-trips mother-son + business contexts.

### Phase 8 — Verification & Re-audit
1. Full test suite green (unit + integration + e2e where present).
2. Manual review of one L0 family-penta and one business dyad source-pack.
3. Update this plan's verification section with exact commands and expected outputs.
4. Only then mark the contract complete.

---

## Success Criteria (updated 2026-07-10)

Phase 1-3 checkpoint:
- ✅ Intake produces explicit mother-son, business-dyad, family-penta `ReportGenerationRequest` objects (builders + questions).
- ✅ Parser accepts variable-role modes with relationship_types and rejects unknown roles.
- ✅ Orchestrator accepts + injects `subjectRoles` + `relationshipContext` (tokens visible in prompts).
- All 82 witness-pipeline tests green.
- Legacy solo / composite-dyad paths continue to work (no breakage).

End-state (after Phase 4-8):
- A caller can POST a `ReportGenerationRequest` with two subjects (roles: mother + son, type: family, mapping_goal: "...") and receive a report whose prompt, rubric, and Folio header all reflect "Mother-Son Lineage Mapping — non-predictive pattern witness".
- Same for business-partners (no romantic language possible).
- Same for 5-person family-penta using pentagon topology.
- No code path silently defaults 2-person reports to romantic partner framing.
- All changes are additive where possible; legacy solo + generic composite-dyad paths continue to work.

---

## Anti-Patterns (Forbidden)

- Hard-coding `roles: ['subject-a', 'subject-b']` in new modes.
- Assuming `relationship_context` is only for UI and skipping propagation.
- Using "partner" or "spouse" language in templates when the request said family or business.
- Increasing timeouts instead of wiring explicit context.
- Treating the old Rust `RelationshipMode` enum as the full product taxonomy.

---

## Progress Snapshot (2026-07-10)

**Phase 1 (Intake + Taxonomy):** COMPLETE
- Canonical `RELATIONSHIP_TYPES`, `SUBJECT_ROLES`, `RelationshipContext`, builders (`createMotherSonRequest`, etc.).
- `buildReportIntakeQuestions` emits type/mapping-goal/sensitivity + per-subject role + label pickers.
- Tests validate explicit non-presumptive shapes; all package tests green.

**Phase 2 (Modes + Parser):** COMPLETE
- `ModeConfig` + `relationship_types`.
- Parser validates roles from SUBJECT_ROLES (+ legacy compat aliases).
- New modes: `mother-son-lineage.md`, `business-partners.md`, `family-penta.md`.
- Parser tests + file fixtures pass.

**Phase 3 (Orchestrator Injection):** COMPLETE
- `OrchestratorInput` accepts `subjectRoles?` + `relationshipContext?`.
- `renderPassTemplate` + `buildSystemPrompt` inject `{{subject_roles}}`, `{{relationship_header}}`, `{{mapping_goal}}`, `{{relationship_context}}`.
- Rubric call site passes `relationshipType`.
- Integrated test asserts injection for mother-son.

**Phase 4 (Rubric Guardrails):** PARTIAL
- `RELATIONSHIP_GUARDRAILS` + runtime union in `auditSectionOutput`.
- Rubric tests for family + business-partners stricter failures.
- Next: full e2e with context → rubric violations; update mode templates to declare tokens.

**Remaining (Phases 5-8):**
- Dedicated parser/orchestrator fixtures + e2e for mother-son/business/penta end-to-end.
- B-surface (Folio) header renderer using relationship_header + labeled subjects.
- Rust/API boundary + narrow RelationshipMode mapping for witness-dyad.
- Full re-audit + source-pack review before declaring contract complete.

All current changes are additive; legacy solo/composite paths remain functional.

---

## References

- Gap analysis (user session 2026-07-10)
- `packages/witness-pipeline/src/intake/types.ts`
- `packages/witness-pipeline/src/intake/questions.ts`
- `packages/witness-pipeline/src/modes/{types,parser}.ts`
- `packages/witness-pipeline/src/orchestrator/{integrated,rubric}.ts`
- `crates/noesis-witness/src/interpret.rs` (RelationshipMode)
- `crates/noesis-core/src/intake.rs`
- `docs/design-system/SYSTEM.md` (Folio voice)
- tools/humdes-extractor/INTEGRATION.md (historical witness-agents taxonomy)
