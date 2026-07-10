# Integrated Report Variables: Language, L4/L5, Synastry Matrix, Retrieval & NotebookLM Readiness

> **For Codex:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add first-class language support (default English, injectable into all prompts and metadata), complete L4/L5 explicit coverage, fill critical synastry relationship type + topology gaps (unmarried/married-partners, triad-triangle, web-graph), ensure retrieval/pattern extraction respect relationship_type + report_level, and make the engine ready for NotebookLM revival with language.

**Architecture:** Treat `language` as a first-class field on ReportGenerationRequest → OrchestratorInput → prompt rendering + pattern metadata + retrieval filters (additive, optional, defaults to 'en'). Use existing mode frontmatter + pass templates for injection via `{{language}}`. Add minimal conforming mode docs for missing relationship types and exercise remaining topologies. Add L4/L5 report_level declarations. Introduce a cross-product matrix test that enumerates modes/levels/relationships and asserts basic invariants (no crash, header when relationship present, rubric pass on clean output). Keep all changes YAGNI/DRY/TDD with frequent small commits.

**Tech Stack:** TypeScript + Vitest in `packages/witness-pipeline`, existing orchestrator/modes/intake/patterns layers, no new external services for this plan (NotebookLM generator revival is noted as future additive).

---

## Preparation Notes (read before starting any task)

- Work in a dedicated git worktree (per writing-plans skill guidance).
- Always run the exact test command listed and verify the exact expected output before moving to the next step.
- Every commit must be small and focused.
- Language is additive: existing tests and solo flows must continue to work with no language supplied (default 'en').
- Relationship header and guardrails from the 2026-07-10 relationship-mapping-contract plan are already present; this plan builds on them.
- NotebookLM generator code does not exist in the current tree (only historical references). This plan adds language readiness so revival is cheap later. Do not build a full NotebookLM generator.

---

### Task 1: Add `language` field to core intake types

**Files:**
- Modify: `packages/witness-pipeline/src/intake/types.ts:86-92`

**Step 1: Write the failing test**

```ts
// Add to packages/witness-pipeline/src/intake/types.test.ts
it('accepts optional language on ReportGenerationRequest (defaults to en in usage)', () => {
  const req = {
    ...createMotherSonRequest(),
    language: 'hi',
  } as any;
  expect(req.language).toBe('hi');
});
```

**Step 2: Run test to verify it fails (type or runtime)**

Run: `pnpm --filter @noesis/witness-pipeline test -- src/intake/types.test.ts -t "accepts optional language"`

Expected: FAIL (language not declared on the interface).

**Step 3: Extend the interface (minimal)**

In `packages/witness-pipeline/src/intake/types.ts`, update ReportGenerationRequest:

```ts
export interface ReportGenerationRequest {
  report_level: ReportLevel;
  report_mode: ReportMode;
  subjects: ReportSubjectInput[];
  relationship_context?: RelationshipContext;
  language?: string; // e.g. 'en', 'hi', 'es' — injected into prompts and metadata
  output: { format: 'markdown' | 'docx' | 'pdf' | 'source-pack'; include_rubric: boolean; include_pattern_extraction: boolean };
}
```

Also add to the three builder functions (createMotherSonRequest, createBusinessDyadRequest, createFamilyPentaRequest) for test convenience:
```ts
language: 'en',
```

**Step 4: Run test to verify it passes**

Run: `pnpm --filter @noesis/witness-pipeline test -- src/intake/types.test.ts -t "accepts optional language"`

Expected: PASS.

**Step 5: Commit**

```bash
git add packages/witness-pipeline/src/intake/types.ts packages/witness-pipeline/src/intake/types.test.ts
git commit -m "feat(intake): add optional language field to ReportGenerationRequest"
```

---

### Task 2: Surface language in intake questions (optional but visible)

**Files:**
- Modify: `packages/witness-pipeline/src/intake/questions.ts`
- Modify: `packages/witness-pipeline/src/intake/questions.test.ts`

**Step 1: Add a language question helper (non-breaking)**

Add at the end of `buildReportIntakeQuestions` (or a new exported function if you prefer):

```ts
export function getLanguageQuestion(): IntakeQuestion {
  return {
    header: 'Language',
    question: 'In which language should the report and any generated assets be produced?',
    options: [
      { label: 'en', description: 'English (default)' },
      { label: 'hi', description: 'Hindi' },
      { label: 'es', description: 'Spanish' },
      // Add more as needed; free-text fallback allowed downstream
    ],
  };
}
```

**Step 2: Run existing questions tests to ensure nothing broke**

Run: `pnpm --filter @noesis/witness-pipeline test -- src/intake/questions.test.ts -v`

Expected: all PASS (no change to existing behavior).

**Step 3: Add a tiny test for the new helper**

```ts
it('exposes language question with en as default option', () => {
  const q = getLanguageQuestion();
  expect(q.header).toBe('Language');
  const labels = (q.options ?? []).map(o => o.label);
  expect(labels).toContain('en');
});
```

**Step 4: Run to verify**

Run: `pnpm --filter @noesis/witness-pipeline test -- src/intake/questions.test.ts -t "language question"`

Expected: PASS.

**Step 5: Commit**

```bash
git add packages/witness-pipeline/src/intake/questions.ts packages/witness-pipeline/src/intake/questions.test.ts
git commit -m "feat(intake): add language question helper"
```

---

### Task 3: Propagate language into OrchestratorInput and render paths

**Files:**
- Modify: `packages/witness-pipeline/src/orchestrator/integrated.ts:24-34` (interface)
- Modify: `packages/witness-pipeline/src/orchestrator/integrated.ts` (renderPassTemplate + buildSystemPrompt + run output)

**Step 1: Extend the interface (additive)**

```ts
export interface OrchestratorInput {
  // ... existing fields
  language?: string; // default 'en' at call sites
}
```

**Step 2: Update renderPassTemplate to substitute {{language}}**

In the return template chain, add:

```ts
.replace(/\{\{language\}\}/g, input.language ?? 'en')
```

**Step 3: Update buildSystemPrompt to mention language**

Add one line:

```ts
Language: ${input.language ?? 'en'}.
```

**Step 4: Write a failing test that asserts language token reaches the prompt**

In `packages/witness-pipeline/src/orchestrator/integrated.test.ts` (extend existing mother-son test or add small one):

```ts
it('injects language into prompts when supplied', async () => {
  const llm = vi.fn().mockResolvedValue('ok');
  const orchestrator = new IntegratedReadingOrchestrator({ mode: mockMode, llm });
  await orchestrator.run({
    subjectNames: ['A'],
    engineResultsBySubject: [mockEngineResults],
    consciousnessLevel: 2,
    language: 'hi',
  });
  const user = llm.mock.calls[0][1] as string;
  expect(user).toContain('hi'); // via {{language}} or system line
});
```

**Step 5: Run to see it fail (token not present yet)**

Run the specific test.

Expected: FAIL (no 'hi' in prompt).

**Step 6: Implement the substitutions**

Apply the two small replaces + system line from steps 2-3.

**Step 7: Run test to verify it passes**

Run the test.

Expected: PASS.

**Step 8: Commit**

```bash
git add packages/witness-pipeline/src/orchestrator/integrated.ts packages/witness-pipeline/src/orchestrator/integrated.test.ts
git commit -m "feat(orchestrator): propagate language into prompt templates and system prompt"
```

---

### Task 4: Pass language from ReportGenerationRequest through callers (minimal)

**Files:**
- Modify: `packages/witness-pipeline/src/assets/factory.ts` (if it calls orchestrator) or the places that build OrchestratorInput (search for new IntegratedReadingOrchestrator calls).
- For now, focus on the orchestrator test path + one integration point.

**Step 1: Ensure the e2e mother-son test also passes language**

In the mother-son e2e test, add `language: 'en'` explicitly to the input object.

**Step 2: Run the e2e relationship tests**

Run: `pnpm --filter @noesis/witness-pipeline test -- src/orchestrator/integrated.e2e.test.ts -t "mother-son|business-partners|family-penta"`

Expected: still PASS.

**Step 3: Commit**

Small commit documenting the explicit language passthrough.

---

### Task 5: Include language in pattern metadata and retrieval filters

**Files:**
- Modify: `packages/witness-pipeline/src/patterns/extractor.ts:68-74`
- Modify: `packages/witness-pipeline/src/patterns/retrieval.ts:7-13`
- Modify: `packages/witness-pipeline/src/patterns/cloudflare-vectorize.ts:68-74` (meta)

**Step 1: Extend ExtractReportPatternsInput and usage**

Add `language?: string` to the input interface.

In the returned metadata:

```ts
metadata: {
  mode: input.mode,
  report_level: input.reportLevel,
  language: (input as any).language ?? 'en',
  ...
}
```

**Step 2: Add language to RetrievalFilters**

```ts
export interface RetrievalFilters {
  mode?: string;
  report_level?: string;
  language?: string;
  // ... existing
}
```

**Step 3: Write a small failing test in extractor.test.ts**

```ts
it('includes language in extracted pattern metadata when provided', () => {
  const patterns = extractReportPatterns({
    mode: 'test',
    reportLevel: 'L2',
    subjectNames: [],
    passes: [/* minimal passing pass */],
    language: 'hi',
  } as any);
  expect(patterns[0]?.metadata?.language).toBe('hi');
});
```

**Step 4: Run → see fail → implement → run → pass**

**Step 5: Update vectorize meta emission to include language (if present)**

**Step 6: Commit**

```bash
git add packages/witness-pipeline/src/patterns/extractor.ts packages/witness-pipeline/src/patterns/retrieval.ts packages/witness-pipeline/src/patterns/cloudflare-vectorize.ts packages/witness-pipeline/src/patterns/extractor.test.ts
git commit -m "feat(patterns): include language in metadata and retrieval filters"
```

---

### Task 6: Add L4 and L5 explicit report_level coverage

**Files:**
- Modify: `packages/witness-pipeline/modes/integrated-reading.md` (or create a thin L4 variant)
- Modify: `packages/witness-pipeline/modes/integrated-kundali-l0.md` (mark or copy for L5 depth example)
- Test: update or add assertions in parser + e2e

**Step 1: Add report_level: L4 to one existing deep mode (or create integrated-reading-l4.md as thin alias)**

For YAGNI, simply add to the frontmatter of `integrated-reading.md` a variant note, or create a minimal L4 file that reuses most content.

Minimal: create `packages/witness-pipeline/modes/integrated-reading-l4.md` with:

```
---
mode: integrated-reading-l4
report_level: L4
subject_count: { min: 1, max: 2 }
roles: [subject]
... (copy key frontmatter from integrated-reading, increase target words)
svg_topology: dyad-arc
---
```

**Step 2: Parser test that L4 is accepted**

Add to parser.test.ts a small doc string with report_level: L4 and assert it parses.

**Step 3: Run parser tests**

Expected: PASS.

**Step 4: Add a minimal L5 marker on the L0 kundali mode or a new thin L5 doc (for matrix purposes)**

**Step 5: Run full test suite for modes/parser**

Expected: green.

**Step 6: Commit**

---

### Task 7: Add minimal mode docs for missing relationship types + exercise remaining topologies

**Files:**
- Create: `packages/witness-pipeline/modes/unmarried-partners.md`
- Create: `packages/witness-pipeline/modes/married-partners.md`
- Create or extend: a triad or web-graph mode (e.g. update family-penta or add a small triad example)
- Test: parser + one e2e

**Step 1: Write the unmarried-partners.md (minimal conforming)**

Use the same shape as business-partners.md but with relationship_types: ['unmarried-partners'], roles appropriate, svg_topology: 'dyad-arc'.

**Step 2: Same for married-partners.md**

**Step 3: Add a parser test that these load and declare correct relationship_types**

**Step 4: Run parser tests**

**Step 5: Add one test that uses triad-triangle (even if just parsing + basic orchestrator run with mock)**

You may reuse or lightly extend an existing triad-capable fixture.

**Step 6: Commit the new modes + tests**

---

### Task 8: Build a cross-product matrix test harness (enumeration)

**Files:**
- Create or extend: `packages/witness-pipeline/src/orchestrator/integrated.matrix.test.ts` (or add to e2e)

**Step 1: Write a test that enumerates a small matrix**

Example (pseudo, make it real):

```ts
const combos = [
  { mode: 'mother-son-lineage', level: 'L2', rel: 'family' },
  { mode: 'business-partners', level: 'L2', rel: 'business-partners' },
  { mode: 'family-penta', level: 'L3', rel: 'family' },
  // add L4, unmarried, etc. as they are added
];

for (const c of combos) {
  it(`runs ${c.mode} ${c.level} ${c.rel} without crash and produces header when relationship present`, async () => {
    // load mode, run with mock LLM + relationshipContext, assert no throw + rubric pass + header if applicable
  });
}
```

**Step 2: Start with 3-4 combos that we know work.**

**Step 3: Run the matrix test file**

Expected: all green.

**Step 4: Expand the matrix in later tasks (after adding more modes/levels).**

**Step 5: Commit**

---

### Task 9: Re-audit retrieval filters with relationship_type (additive)

**Files:**
- Modify: `packages/witness-pipeline/src/patterns/retrieval.ts` (add relationship_type?: string to filters)
- Modify: `packages/witness-pipeline/src/patterns/cloudflare-vectorize.ts` (emit + filter if present)
- Update extractor metadata to also carry relationship_type when context is supplied (pass through orchestrator if needed)

**Step 1: Extend RetrievalFilters**

```ts
relationship_type?: string;
```

**Step 2: Add a test that filters by relationship_type**

**Step 3: Run retrieval + vectorize tests**

**Step 4: Commit**

---

### Task 10: Language + relationship readiness note for NotebookLM / premium assets

**Files:**
- Modify: `packages/noesis-sdk-ts/src/premium-assets.ts` (add language?: string to PremiumAssetInput and pass it downstream when calling orchestrator)
- Add a comment in the plan + a small test or assertion that language flows to orchestrator_output when provided.

**Step 1: Add language to PremiumAssetInput**

**Step 2: Wire it into the local orchestrator call path (if language present)**

**Step 3: Add a tiny test that language appears in the returned orchestrator_output when supplied**

**Step 4: Run the premium-assets contract test**

**Step 5: Commit + add a note in the file that full NotebookLM generator revival should consume this language field.**

---

### Task 11: Final matrix expansion + full suite run + re-audit

**Step 1: Expand the matrix test to include at least one L4, one unmarried, and one triad usage (as they become available).**

**Step 2: Run the entire witness-pipeline test suite**

Run: `pnpm --filter @noesis/witness-pipeline test`

Expected: all green.

**Step 3: Spot-check that language='hi' + family relationship + L3 mode produces header in assembled and no family: guardrail violations on clean output.**

**Step 4: Commit the final matrix + any small fixes**

```bash
git commit -m "test: expand cross-product matrix for language + relationship + L-levels + topologies"
```

**Step 5: Update the 2026-07-10-relationship-mapping-contract.md progress snapshot (optional but recommended)**

---

## Execution Handoff

Plan complete and saved to `docs/plans/2026-07-10-report-variables-language-l4-l5-synastry-matrix.md`.

**Two execution options:**

1. **Subagent-Driven (this session)** — I dispatch fresh subagent per task, review between tasks, fast iteration. Uses superpowers:subagent-driven-development.

2. **Parallel Session (separate)** — Open new session in the worktree with superpowers:executing-plans, batch execution with checkpoints.

Which approach?
