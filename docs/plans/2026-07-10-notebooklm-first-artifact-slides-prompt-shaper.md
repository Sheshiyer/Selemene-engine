# NotebookLM First Artifact: Slides Prompt Shaper

> **For Codex:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Deliver the smallest valuable NotebookLM revival artifact — a pure deterministic function that consumes an OrchestratorOutput (plus optional language) and emits a self-contained, guardrail-injected NotebookLM prompt for an 8–12 slide professional slide deck.

**Architecture:** Place a thin pure-transformer module inside the existing witness-pipeline package (`src/notebooklm/slides-prompt.ts`). It reads the already-rich surfaces produced by the orchestrator (relationship_header, passes, assembled, patterns, mode, register) and the language that callers already hold. No new LLM calls, no new packages, no external services. The generated prompt must be self-describing: it explicitly carries the relationship_header, the chosen language, a "Facts only. No prediction or diagnosis." directive, and relevant bridge_mandates so that NotebookLM cannot drift into prescriptive or romantic framing.

**Tech Stack:** TypeScript + Vitest (same runner and tsconfig used by @noesis/witness-pipeline). Pure functions only for this slice.

**Key Decisions Encoded in This Plan (answers to the three questions):**
- Smallest valuable artifact first: a **slides prompt** (structured 8–12 slide outline with speaker notes + citations) — higher leverage than a raw audio teaser.
- Location: **inside witness-pipeline as pure transformers** (co-located with the data source, trivial to test, no deployment surface yet).
- Guardrail strictness: **embed non-prescriptive + relationship rules directly into the generated prompt text** (do not rely solely on the source `assembled`).

**Preparation Notes (read before any task):**
- Work in a dedicated git worktree if following the full writing-plans discipline.
- Always run the exact command listed and verify the exact expected output before the next step.
- Every commit must be small and focused.
- All changes are additive. Existing 88-test suite must remain green.
- Use only existing types (`OrchestratorOutput`, `PassResult`, etc.). Do not invent new engine contracts.
- Reference skills: @using-superpowers, @executing-plans, @dispatching-parallel-agents.

---

### Task 1: Create the notebooklm module skeleton with a failing "exports function" test

**Files:**
- Create: `packages/witness-pipeline/src/notebooklm/slides-prompt.ts`
- Create: `packages/witness-pipeline/src/notebooklm/slides-prompt.test.ts`
- Modify: `packages/witness-pipeline/src/index.ts` (add re-export for discoverability)

**Step 1: Write the failing test**

```ts
// packages/witness-pipeline/src/notebooklm/slides-prompt.test.ts
import { describe, it, expect } from 'vitest';
import { generateSlidesPrompt } from './slides-prompt.js';
import type { OrchestratorOutput } from '../orchestrator/integrated.js';

describe('generateSlidesPrompt', () => {
  it('exports generateSlidesPrompt as a pure function', () => {
    const out: OrchestratorOutput = {
      mode: 'test-mode',
      subject_names: ['Test'],
      register: 'l1_l3',
      passes: [],
      assembled: 'test',
      patterns: [],
    };
    const prompt = generateSlidesPrompt(out);
    expect(typeof prompt).toBe('string');
    expect(prompt.length).toBeGreaterThan(0);
  });
});
```

**Step 2: Run test to verify it fails (module or export missing)**

Run:
```bash
pnpm --filter @noesis/witness-pipeline test -- src/notebooklm/slides-prompt.test.ts -t "exports generateSlidesPrompt"
```

Expected: FAIL (Cannot find module './slides-prompt.js' or generateSlidesPrompt is not a function).

**Step 3: Create the minimal module that satisfies the test**

```ts
// packages/witness-pipeline/src/notebooklm/slides-prompt.ts
import type { OrchestratorOutput } from '../orchestrator/integrated.js';

export function generateSlidesPrompt(
  output: OrchestratorOutput,
  opts?: { language?: string }
): string {
  const language = opts?.language ?? 'en';
  return `Language: ${language}\nMode: ${output.mode}\nSlides prompt placeholder`;
}
```

**Step 4: Run test to verify it passes**

Run the same command.

Expected: PASS.

**Step 5: Commit**

```bash
git add packages/witness-pipeline/src/notebooklm/slides-prompt.ts packages/witness-pipeline/src/notebooklm/slides-prompt.test.ts packages/witness-pipeline/src/index.ts
git commit -m "feat(notebooklm): scaffold slides-prompt module with failing export test"
```

---

### Task 2: Re-export the new module from the package index (additive)

**Files:**
- Modify: `packages/witness-pipeline/src/index.ts`

**Step 1: Write a small test that importing from the package root works**

Add to the same test file (or a new one-liner test):

```ts
it('is re-exported from package root', async () => {
  const mod = await import('@noesis/witness-pipeline');
  expect(typeof mod.generateSlidesPrompt).toBe('function');
});
```

**Step 2: Run to see it fail (not yet exported)**

```bash
pnpm --filter @noesis/witness-pipeline test -- src/notebooklm/slides-prompt.test.ts -t "re-exported from package root"
```

Expected: FAIL (generateSlidesPrompt undefined on root export).

**Step 3: Add the re-export (minimal, additive)**

In `packages/witness-pipeline/src/index.ts`, add:

```ts
export * from './notebooklm/slides-prompt.js';
```

**Step 4: Run to verify it passes**

Expected: PASS.

**Step 5: Commit**

```bash
git add packages/witness-pipeline/src/index.ts
git commit -m "feat(notebooklm): re-export slides prompt generator from package root"
```

---

### Task 3: Make the generated prompt include relationship_header when present (TDD)

**Files:**
- Modify: `packages/witness-pipeline/src/notebooklm/slides-prompt.ts`
- Modify: `packages/witness-pipeline/src/notebooklm/slides-prompt.test.ts`

**Step 1: Write the failing test (mother-son case)**

```ts
it('injects relationship_header and language into the prompt when present', () => {
  const out: OrchestratorOutput = {
    mode: 'mother-son-lineage',
    subject_names: ['Aarav', 'Vikram'],
    register: 'l1_l3',
    relationship_header: 'Mother-Son Lineage Mapping — non-predictive pattern witness',
    passes: [
      { id: 'opening', title: 'Opening', output: 'Fact one.', rubric: { guardrail_gate: 'pass' } as any },
    ],
    assembled: 'Mother-Son...',
    patterns: [],
  };
  const prompt = generateSlidesPrompt(out, { language: 'hi' });
  expect(prompt).toContain('Language: hi');
  expect(prompt).toContain('Mother-Son Lineage Mapping — non-predictive pattern witness');
  expect(prompt).toContain('Facts only. No prediction');
});
```

**Step 2: Run to see it fail**

```bash
pnpm --filter @noesis/witness-pipeline test -- src/notebooklm/slides-prompt.test.ts -t "injects relationship_header"
```

Expected: FAIL (header or language or guardrail sentence not present).

**Step 3: Implement minimal injection logic**

Update `generateSlidesPrompt`:

```ts
export function generateSlidesPrompt(
  output: OrchestratorOutput,
  opts?: { language?: string }
): string {
  const language = opts?.language ?? 'en';
  const header = output.relationship_header
    ? output.relationship_header
    : 'Solo Reading — non-predictive pattern witness';

  const guard = 'Facts only. No prediction. No diagnosis. Use only observable patterns and engine data.';

  const slides = [
    `# NotebookLM Slides Prompt`,
    ``,
    `Language: ${language}`,
    `Mode: ${output.mode}`,
    `Register: ${output.register}`,
    ``,
    `## Relationship / Framing (MANDATORY — copy verbatim into every slide)`,
    header,
    guard,
    ``,
    `## Slide Outline (8-12 slides recommended)`,
    `1. Title: ${header}`,
    `2. Subjects & Context`,
    `3-8. One slide per major pass or pattern cluster (cite pass id)`,
    `9. Synthesis (non-predictive)`,
    `10. Reflection questions`,
    ``,
    `Source assembled length (chars): ${output.assembled.length}`,
    `Pass count: ${output.passes.length}`,
  ];

  return slides.join('\n');
}
```

**Step 4: Run to verify it passes**

Expected: PASS.

**Step 5: Commit**

```bash
git add packages/witness-pipeline/src/notebooklm/slides-prompt.ts packages/witness-pipeline/src/notebooklm/slides-prompt.test.ts
git commit -m "feat(notebooklm): inject relationship_header + language + facts-only guard into slides prompt"
```

---

### Task 4: Add a solo-path test (no relationship_header) and assert clean structure

**Files:**
- Modify: `packages/witness-pipeline/src/notebooklm/slides-prompt.test.ts`

**Step 1: Write the failing test**

```ts
it('produces a clean solo prompt when no relationship_context is present', () => {
  const out: OrchestratorOutput = {
    mode: 'birth-blueprint',
    subject_names: ['Subject'],
    register: 'l4_l5',
    passes: [],
    assembled: 'Solo content',
    patterns: [],
  };
  const prompt = generateSlidesPrompt(out, { language: 'en' });
  expect(prompt).toContain('Solo Reading — non-predictive pattern witness');
  expect(prompt).not.toContain('Mother-Son');
});
```

**Step 2: Run to see it fail**

```bash
pnpm --filter @noesis/witness-pipeline test -- src/notebooklm/slides-prompt.test.ts -t "clean solo prompt"
```

Expected: FAIL.

**Step 3: Adjust implementation if needed (keep minimal)**

Usually the previous implementation already handles this via the ternary. If it does, the test will pass after the run in step 4.

**Step 4: Run to verify it passes**

Expected: PASS.

**Step 5: Commit**

```bash
git add packages/witness-pipeline/src/notebooklm/slides-prompt.test.ts
git commit -m "test(notebooklm): add solo-path case for slides prompt"
```

---

### Task 5: Consume real matrix-style data (use a minimal inline fixture that mirrors integrated.matrix.test.ts)

**Files:**
- Modify: `packages/witness-pipeline/src/notebooklm/slides-prompt.test.ts`

**Step 1: Add a test that feeds a richer output resembling the matrix test output**

```ts
it('produces usable NotebookLM prompt from a mother-son matrix-style output', () => {
  const motherSonOut: OrchestratorOutput = {
    mode: 'mother-son-lineage',
    subject_names: ['Aarav', 'Vikram'],
    register: 'l1_l3',
    relationship_header: 'Mother-Son Lineage Mapping — non-predictive pattern witness',
    passes: [
      { id: 'opening', title: 'Opening', output: 'Observable fact A.', rubric: { guardrail_gate: 'pass' } as any },
      { id: 'lineage', title: 'Lineage Field', output: 'Observable fact B.', rubric: { guardrail_gate: 'pass' } as any },
    ],
    assembled: 'Mother-Son Lineage Mapping...\n\n## Opening\n\nObservable fact A.\n\n## Lineage Field\n\nObservable fact B.',
    patterns: [],
  };

  const prompt = generateSlidesPrompt(motherSonOut, { language: 'hi' });

  expect(prompt).toContain('Language: hi');
  expect(prompt).toContain('Mother-Son Lineage Mapping — non-predictive pattern witness');
  expect(prompt).toContain('Facts only. No prediction');
  expect(prompt).toContain('Pass count: 2');
  // The prompt should be safe to paste directly into NotebookLM
  expect(prompt.length).toBeGreaterThan(200);
});
```

**Step 2: Run to see it fail (if the outline is too rigid)**

```bash
pnpm --filter @noesis/witness-pipeline test -- src/notebooklm/slides-prompt.test.ts -t "usable NotebookLM prompt from a mother-son"
```

Expected: FAIL or PASS depending on previous flexibility. If FAIL, proceed to minimal fix.

**Step 3: Make minimal adjustment so the test passes while keeping the prompt structured**

Ensure the implementation lists actual pass titles when available (small enhancement inside the outline section).

**Step 4: Run to verify it passes**

Expected: PASS.

**Step 5: Commit**

```bash
git add packages/witness-pipeline/src/notebooklm/slides-prompt.test.ts
git commit -m "test(notebooklm): add rich mother-son matrix-style fixture test for slides prompt"
```

---

### Task 6: Ensure the prompt embeds at least one bridge_mandate when the mode provides them (YAGNI but valuable)

**Files:**
- Modify: `packages/witness-pipeline/src/notebooklm/slides-prompt.ts`
- Modify: `packages/witness-pipeline/src/notebooklm/slides-prompt.test.ts`

**Step 1: Write a failing test that checks for a known mandate string**

```ts
it('includes a bridge mandate when supplied via a minimal mode-like surface (future: pass mode doc)', () => {
  // For v1 we simulate by allowing an optional mandates array on the call
  const out: OrchestratorOutput = {
    mode: 'business-partners',
    subject_names: ['Priya', 'Rahul'],
    register: 'l1_l3',
    relationship_header: 'Business-Partners Synergy Audit — non-predictive pattern witness',
    passes: [],
    assembled: '',
    patterns: [],
  };
  const prompt = generateSlidesPrompt(out, { language: 'en' });
  // In the first slice we accept an extension point; the test documents the intent
  expect(prompt).toContain('No investment or outcome guarantees');
});
```

**Step 2: Run to see it fail**

Expected: FAIL (mandate not present).

**Step 3: Extend the function signature minimally (additive) and hard-code the known business mandate for this test**

```ts
export function generateSlidesPrompt(
  output: OrchestratorOutput,
  opts?: { language?: string; bridgeMandates?: string[] }
): string {
  const language = opts?.language ?? 'en';
  const header = output.relationship_header || 'Solo Reading — non-predictive pattern witness';
  const guard = 'Facts only. No prediction. No diagnosis. Use only observable patterns and engine data.';
  const mandates = (opts?.bridgeMandates || []).map(m => `- ${m}`).join('\n');

  const slides = [
    `# NotebookLM Slides Prompt`,
    ``,
    `Language: ${language}`,
    `Mode: ${output.mode}`,
    `Register: ${output.register}`,
    ``,
    `## Relationship / Framing (MANDATORY — copy verbatim)`,
    header,
    guard,
    mandates ? `## Mode Mandates\n${mandates}` : '',
    ``,
    `## Slide Outline`,
    `1. Title slide using the exact header above`,
    `...`,
  ].filter(Boolean);

  return slides.join('\n');
}
```

Update the business test call to pass the mandate:

```ts
const prompt = generateSlidesPrompt(out, {
  language: 'en',
  bridgeMandates: ['No investment or outcome guarantees'],
});
```

**Step 4: Run to verify it passes**

Expected: PASS.

**Step 5: Commit**

```bash
git add packages/witness-pipeline/src/notebooklm/slides-prompt.ts packages/witness-pipeline/src/notebooklm/slides-prompt.test.ts
git commit -m "feat(notebooklm): support bridgeMandates injection in slides prompt for guardrail fidelity"
```

---

### Task 7: Run the full witness-pipeline test suite and verify no regression

**Step 1: Run the exact full command**

```bash
pnpm --filter @noesis/witness-pipeline test
```

**Step 2: Verify output**

Expected: All tests green (22+ files, 88+ tests, the new notebooklm tests included).

**Step 3: If any failure appears, fix only the minimal thing that broke (usually an export or type). Re-run.**

**Step 4: Commit the verification state**

```bash
git add -u
git commit -m "test(notebooklm): full suite green after slides-prompt addition (88+ tests)"
```

---

### Task 8 (optional but recommended): Add a one-line usage example in the test file as living documentation

**Step 1: Append a comment block at the bottom of the test file**

```ts
/*
Example usage (copy into a script or NotebookLM workflow):

import { generateSlidesPrompt } from '@noesis/witness-pipeline';
const prompt = generateSlidesPrompt(orchestratorResult, { language: 'hi' });
// Paste `prompt` into NotebookLM "Create slides from text" or "Audio overview" source.
*/
```

**Step 2: No test change needed — this is documentation only.**

**Step 3: Run the notebooklm test file to confirm nothing broke**

```bash
pnpm --filter @noesis/witness-pipeline test -- src/notebooklm/slides-prompt.test.ts
```

Expected: PASS.

**Step 4: Commit**

```bash
git add packages/witness-pipeline/src/notebooklm/slides-prompt.test.ts
git commit -m "docs(notebooklm): add usage example comment for slides prompt"
```

---

## Execution Handoff

Plan complete and saved to `docs/plans/2026-07-10-notebooklm-first-artifact-slides-prompt-shaper.md`.

**Two execution options:**

**1. Subagent-Driven (this session)** — I dispatch fresh subagent per task, review between tasks, fast iteration.

**2. Parallel Session (separate)** — Open new session with executing-plans, batch execution with checkpoints.

**Which approach?**

**If Subagent-Driven chosen:**
- REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development
- Stay in this session
- Fresh subagent per task + code review

**If Parallel Session chosen:**
- Guide them to open new session in worktree
- REQUIRED SUB-SKILL: New session uses superpowers:executing-plans
