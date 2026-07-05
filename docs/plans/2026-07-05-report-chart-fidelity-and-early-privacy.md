# Report Chart Fidelity Grounding and Early Privacy Hardening

> **For Codex:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-step.

**Goal:** Add deterministic chart data fidelity checks to the per-section rubric and early private birth data filtering before pattern extraction, so reports are provably grounded in the provided engine results and never leak private data into learning surfaces.

**Architecture:** Extend `auditSectionOutput` to receive the raw `SelemeneEngineOutput[]` for the subject and compute a simple alignment score against key deterministic facts present in those results (Lagna sign, specific gates/channels, dasha names + periods, panchanga tithi/nakshatra when available). Add an early `scrubAndValidatePrivateData` step inside `extractReportPatterns` (or a new pre-filter) that rejects or redacts any pass output containing birth identifiers before anonymization and pattern extraction. Strengthen the L0 mode bridge mandates and system prompt template with explicit "cite only from engines" language. Keep all checks deterministic first.

**Tech Stack:** TypeScript (Vitest), existing regex + lightweight token extraction from engine JSON, no new LLM judge yet.

---

## Background & Lessons Integrated

From the 2026-07-05 solo L0 test runs (anitha-nateshan, sapna-sabharwal, cumbipuram-subramaniam-nateshan):

- Rubric currently only measures *presence* of system keywords in model output, not *fidelity* to the actual deterministic engine results supplied in `engineResultsBySubject`.
- Private birth data (names, dates, times, lat/lng, timezones) can leak from pass output into extracted patterns; Vectorize filter catches it too late.
- Stub LLM over-generation and forbidden surface phrases caused word-count and guardrail failures even when facts/layers were high.
- Source packs now carry `quality.sections` and `pattern_learning`; we need to protect what goes into them.

This plan closes the two highest-leverage gaps: **chart grounding** and **early privacy**.

---

### Task 1: Extend SectionRubric with chart_fidelity_score

**Files:**
- Modify: `packages/witness-pipeline/src/orchestrator/integrated.ts:18-34`
- Modify: `packages/witness-pipeline/src/orchestrator/rubric.ts:1-95`
- Test: `packages/witness-pipeline/src/orchestrator/rubric.test.ts`

**Step 1: Write the failing test (add fidelity field)**

```ts
it('computes chart_fidelity_score when engine results are supplied', () => {
  const engines = [makeEngineWithLagna('aries'), makeEngineWithGate(34)];
  const rubric = auditSectionOutput({
    sectionId: 'opening',
    title: 'Opening',
    targetWords: 100,
    output: 'Lagna is Aries and Human Design gate 34 is active.',
    modelRequested: 'tier-default',
    modelUsed: 'tier-default',
    latencyMs: 10,
    engineResults: engines,
  });

  expect(rubric.chart_fidelity_score).toBeGreaterThanOrEqual(0.5);
  expect(rubric.chart_fidelity_score).toBeLessThanOrEqual(1.0);
});
```

**Step 2: Run test to verify it fails**

Run: `pnpm --filter @noesis/witness-pipeline test -- src/orchestrator/rubric.test.ts -t "chart_fidelity_score"`

Expected: FAIL (chart_fidelity_score undefined or 0).

**Step 3: Add field to SectionRubric interface**

In `integrated.ts`:

```ts
export interface SectionRubric {
  // ... existing fields
  chart_fidelity_score?: number; // 0.0 - 1.0 alignment with provided engine results
  chart_fidelity_details?: string[];
}
```

**Step 4: Implement lightweight fidelity extraction in rubric.ts**

Add helper:

```ts
function extractKeyFactsFromEngines(engines: SelemeneEngineOutput[]): Set<string> {
  const facts = new Set<string>();
  for (const e of engines) {
    const r = e.result || {};
    if (r.lagna_sign) facts.add(`lagna:${String(r.lagna_sign).toLowerCase()}`);
    if (r.lagna) facts.add(`lagna:${String(r.lagna).toLowerCase()}`);
    if (Array.isArray(r.gates)) r.gates.forEach((g: any) => facts.add(`gate:${g}`));
    if (Array.isArray(r.channels)) r.channels.forEach((c: any) => facts.add(`channel:${c}`));
    if (r.mahadasha || r.current_mahadasha) facts.add(`dasha:${(r.mahadasha || r.current_mahadasha)}`.toLowerCase());
    if (r.tithi_name) facts.add(`tithi:${r.tithi_name}`.toLowerCase());
    if (r.nakshatra_name) facts.add(`nakshatra:${r.nakshatra_name}`.toLowerCase());
  }
  return facts;
}

function computeFidelity(output: string, engineFacts: Set<string>): { score: number; details: string[] } {
  if (engineFacts.size === 0) return { score: 0, details: ['no engine facts provided'] };
  const lower = output.toLowerCase();
  let hits = 0;
  const details: string[] = [];
  for (const f of engineFacts) {
    const token = f.split(':')[1] || f;
    if (lower.includes(token)) { hits++; details.push(`hit:${f}`); }
  }
  return { score: hits / engineFacts.size, details };
}
```

Update `auditSectionOutput` signature and call:

```ts
export function auditSectionOutput(input: AuditSectionInput & { engineResults?: SelemeneEngineOutput[] }): SectionRubric {
  // ... existing
  const engineFacts = extractKeyFactsFromEngines(input.engineResults || []);
  const fid = computeFidelity(input.output, engineFacts);
  return {
    // ... existing fields
    chart_fidelity_score: engineFacts.size > 0 ? Number(fid.score.toFixed(3)) : undefined,
    chart_fidelity_details: engineFacts.size > 0 ? fid.details : undefined,
  };
}
```

**Step 5: Run test to verify it passes**

Run: `pnpm --filter @noesis/witness-pipeline test -- src/orchestrator/rubric.test.ts -t "chart_fidelity_score"`

Expected: PASS.

**Step 6: Commit**

```bash
git add packages/witness-pipeline/src/orchestrator/integrated.ts packages/witness-pipeline/src/orchestrator/rubric.ts packages/witness-pipeline/src/orchestrator/rubric.test.ts
git commit -m "feat(rubric): add chart_fidelity_score from engine results"
```

---

### Task 2: Wire engineResults into auditSectionOutput from orchestrator

**Files:**
- Modify: `packages/witness-pipeline/src/orchestrator/integrated.ts:99-112`

**Step 1: Write the failing test (integrated test that fidelity appears)**

Add to `integrated.test.ts` or `integrated.e2e.test.ts`:

```ts
it('populates chart_fidelity_score when engines are passed', async () => {
  const llm = vi.fn().mockResolvedValue('Lagna is Aries with gate 34 and Vimshottari dasha active.');
  const orchestrator = new IntegratedReadingOrchestrator({ mode: mockKundaliMode, llm });
  const result = await orchestrator.run({
    subjectNames: ['A'],
    engineResultsBySubject: [[makeEngineWithLagnaAndGate('aries', 34)]],
    consciousnessLevel: 5,
  });
  expect(result.passes[0].rubric.chart_fidelity_score).toBeGreaterThan(0);
});
```

**Step 2: Run test to verify it fails**

Run: `pnpm --filter @noesis/witness-pipeline test -- src/orchestrator/integrated.test.ts -t "chart_fidelity_score"`

Expected: FAIL (score undefined).

**Step 3: Pass engines to auditSectionOutput**

In `run()` loop:

```ts
const rubric = auditSectionOutput({
  sectionId: pass.id,
  title: pass.title,
  targetWords: pass.target_words,
  output,
  modelRequested: model,
  modelUsed: model,
  latencyMs,
  engineResults: input.engineResultsBySubject[0] ?? [], // or per-subject if multi
});
```

**Step 4: Run test to verify it passes**

Run: `pnpm --filter @noesis/witness-pipeline test -- src/orchestrator/integrated.test.ts -t "chart_fidelity_score"`

Expected: PASS.

**Step 5: Commit**

```bash
git add packages/witness-pipeline/src/orchestrator/integrated.ts packages/witness-pipeline/src/orchestrator/integrated.test.ts
git commit -m "feat: pass engine results to rubric for fidelity scoring"
```

---

### Task 3: Add early private birth data scrub before pattern extraction

**Files:**
- Create: `packages/witness-pipeline/src/patterns/privacy.ts`
- Create: `packages/witness-pipeline/src/patterns/privacy.test.ts`
- Modify: `packages/witness-pipeline/src/patterns/extractor.ts`
- Modify: `packages/witness-pipeline/src/patterns/extractor.test.ts`

**Step 1: Write the failing test (privacy scrub rejects leaks)**

In new `privacy.test.ts`:

```ts
it('rejects pass output containing birth date or coordinates', () => {
  const { scrubbed, hadPrivate } = scrubPrivateBirthData('Born 1960-06-01 at 12.97,77.59');
  expect(hadPrivate).toBe(true);
  expect(scrubbed).not.toMatch(/1960-06-01|12\.97|77\.59/);
});
```

**Step 2: Run test to verify it fails**

Run: `pnpm --filter @noesis/witness-pipeline test -- src/patterns/privacy.test.ts`

Expected: FAIL (module not found).

**Step 3: Implement scrubber**

Create `privacy.ts`:

```ts
const PRIVATE_PATTERNS = [
  /\b(19|20)\d{2}[-/]\d{1,2}[-/]\d{1,2}\b/,           // dates
  /\b\d{1,2}:\d{2}(:\d{2})?\b/,                       // times
  /\b-?\d{1,3}\.\d{3,}\b,\s*-?\d{1,3}\.\d{3,}\b/,    // lat,lng
  /\b(lat|latitude|lng|longitude)\s*[:=]?\s*-?\d/i,
  /\b(timezone|tz)\s*[:=]?\s*[A-Za-z/]+/i,
];

export function scrubPrivateBirthData(text: string): { scrubbed: string; hadPrivate: boolean } {
  let scrubbed = text;
  let hadPrivate = false;
  for (const re of PRIVATE_PATTERNS) {
    if (re.test(scrubbed)) hadPrivate = true;
    scrubbed = scrubbed.replace(re, '[REDACTED]');
  }
  return { scrubbed, hadPrivate };
}
```

**Step 4: Run test to verify it passes**

Run: `pnpm --filter @noesis/witness-pipeline test -- src/patterns/privacy.test.ts`

Expected: PASS.

**Step 5: Integrate into extractReportPatterns (early reject or scrub)**

In `extractor.ts`:

```ts
import { scrubPrivateBirthData } from './privacy.js';

export function extractReportPatterns(...) {
  return input.passes
    .filter(p => p.rubric.guardrail_gate === 'pass' && p.rubric.integrated_layering_gate === 'pass')
    .map(p => {
      const { scrubbed, hadPrivate } = scrubPrivateBirthData(p.output);
      if (hadPrivate) {
        // Option A (strict): return null and filter later
        return null;
      }
      return { ... , text: anonymize(extractPatternText(scrubbed), ...) };
    })
    .filter(Boolean) as ExtractedPattern[];
}
```

**Step 6: Run existing extractor tests + new privacy tests**

Run: `pnpm --filter @noesis/witness-pipeline test -- src/patterns/`

Expected: all PASS.

**Step 7: Commit**

```bash
git add packages/witness-pipeline/src/patterns/privacy.ts packages/witness-pipeline/src/patterns/privacy.test.ts packages/witness-pipeline/src/patterns/extractor.ts packages/witness-pipeline/src/patterns/extractor.test.ts
git commit -m "feat(privacy): early scrub of birth data before pattern extraction"
```

---

### Task 4: Strengthen L0 mode grounding language

**Files:**
- Modify: `packages/witness-pipeline/modes/integrated-kundali-l0.md:70-76` (bridge_mandates)
- Modify: `packages/witness-pipeline/modes/integrated-kundali-l0.md` (system prompt sections if present)

**Step 1: Add explicit grounding mandate**

Append to bridge_mandates:

```yaml
- "Cite only facts present in the supplied engine results. Never invent or generalize chart positions."
- "For every major claim (Lagna, key gates, current dasha, tithi/nakshatra), name the exact value from the engines."
```

**Step 2: Update the system prompt template for L0 passes**

In the relevant pass templates (or overlay-rules section), add:

```
Grounding rule: You are given deterministic engine results. Every specific claim about the chart (signs, gates, dashas, tithis, nakshatras) must be traceable to those results. If a fact is not in the engines, do not state it as true for this person.
```

**Step 3: Run parser + e2e tests that load the mode**

Run: `pnpm --filter @noesis/witness-pipeline test -- src/modes/parser.test.ts src/orchestrator/integrated.e2e.test.ts`

Expected: PASS (mode still parses).

**Step 4: Commit**

```bash
git add packages/witness-pipeline/modes/integrated-kundali-l0.md
git commit -m "docs: strengthen L0 grounding mandates and prompt rules"
```

---

### Task 5: Add fidelity + privacy regression tests in e2e

**Files:**
- Modify: `packages/witness-pipeline/src/orchestrator/integrated.e2e.test.ts`

**Step 1: Add test that fidelity is non-zero when correct facts are cited**

```ts
it('records non-zero chart fidelity when output cites real engine facts', async () => {
  // ...
  expect(result.passes.find(p => p.id === 'opening')?.rubric.chart_fidelity_score).toBeGreaterThan(0);
});
```

**Step 2: Add test that patterns are dropped when private data is present (before anonymize)**

```ts
it('drops patterns that contained private birth data even after scrub', async () => {
  // stub llm returns text with date + coords
  const result = await orchestrator.run(...);
  // after extraction, patterns should be empty or heavily filtered
  expect(result.patterns.filter(p => p.text.includes('1960') || p.text.includes('12.97')).length).toBe(0);
});
```

**Step 3: Run e2e**

Run: `pnpm --filter @noesis/witness-pipeline test -- src/orchestrator/integrated.e2e.test.ts`

Expected: PASS.

**Step 4: Commit**

```bash
git add packages/witness-pipeline/src/orchestrator/integrated.e2e.test.ts
git commit -m "test: add fidelity and early privacy regression coverage for L0"
```

---

### Task 6: Update source pack tests if they assert rubric shape

**Files:**
- Modify: `packages/witness-pipeline/src/assets/factory.test.ts` (if they snapshot rubric fields)

**Step 1:** Add `chart_fidelity_score` to any test objects that construct full rubrics.

**Step 2:** Run factory tests.

**Step 3:** Commit.

---

### Task 7: Full verification

**Commands (exact order):**

```bash
pnpm --filter @noesis/witness-pipeline test -- src/orchestrator/rubric.test.ts src/orchestrator/integrated.test.ts src/patterns/ src/assets/factory.test.ts
pnpm --filter @noesis/witness-pipeline test -- src/orchestrator/integrated.e2e.test.ts
pnpm --filter @noesis/witness-pipeline typecheck
```

Expected: all green.

**Step:** Commit verification if any small fixes were needed.

```bash
git add -A
git commit -m "chore: full verification after fidelity + early privacy"
```

---

## Success Criteria

- `chart_fidelity_score` is populated (0-1) for passes when engines are supplied.
- Private birth data (dates, times, coords, timezones, names) is scrubbed/rejected before `extractReportPatterns` returns any pattern.
- L0 mode explicitly requires citing engine facts.
- Existing tests + new regression tests pass.
- No private data reaches Vectorize or source-pack pattern_learning in the happy path.

## Open Questions / Follow-ups (out of scope for this plan)

- Full deterministic fact alignment (exact dasha periods, exact gate activations with lines, etc.) may later move to a dedicated `chartAlignment.ts` module.
- LLM-as-judge for semantic fidelity is explicitly deferred.
- Real Cloudflare Vectorize + R2/D1 wiring remains in the existing Vectorize plan.

---

**End of plan.**
