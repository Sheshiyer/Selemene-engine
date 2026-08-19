# L0-to-L5 Integrated Kundali + Dyad/Synastry Implementation Plan

> **For Codex:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Extend the validated L0 pipeline upward through L1–L5 with consciousness-level-aware mode docs, templates, and rubric scaling, and build the dyad/synastry reading path (partner + composite) using the same orchestrator with dual-subject engine injection.

**Architecture:** Add per-level mode doc variants (`integrated-kundali-l0.md` → `-l1.md` through `-l5.md`) sharing the same 12-pass plan but with level-tuned target words, pass template depth, and guardrail scaling. Rewrite `partner-synastry.md` to conform to `ModeConfig` schema. Extend `IntegratedReadingOrchestrator` to detect `subject_count > 1` and format both subjects' engine results into comparative prompts. Add a dyad runner parallel to `solo-l0-runner.ts`. Keep rubric gates level-aware but reuse the same section-aware fidelity extraction.

**Tech Stack:** TypeScript, Vitest, Node.js, Command Code MiniMax M3 (LLM), Cloudflare Workers (pattern memory), tsx runner scripts.

---

## Discovery & Constraints

### Current State
- **L0 pipeline fully validated**: 12-pass orchestrator, 6 rubric gates, section-aware chart_fidelity, keyword extraction for biofield/nada, brand-aware HTML+PDF render, pattern memory Worker, llm-proxy Worker. 90 tests, 14/14 live solos PASS.
- **Register bands already exist**: `resolveRegister(level)` → `l1_l3` (≤3) or `l4_l5` (≥4). `register_variants` in mode doc frontmatter already supports per-band word targets and template overrides.
- **`integrated-kundali-l0.md`** already has both register bands configured: `l1_l3` (13k-17k words), `l4_l5` (17k-23k words), with a `final-synthesis-pass-l1-l3` override.
- **`partner-synastry.md`** is a 43-line prose design doc with **none** of the 10 required `ModeConfig` fields — would fail `parseModeDoc()` immediately.
- **Composite-dyad test fixture** (`src/modes/fixtures/composite-dyad.md`) already validates `subject_count: { min:2, max:2 }` and `roles: [subject-a, subject-b]` — the parser handles multi-subject mode docs.
- **`OrchestratorInput.engineResultsBySubject`** is `[][]` (multi-subject capable) but the orchestrator only passes `[0]` to the rubric and prompt renderer.
- **Intake types** already define `SubjectRole = 'partner'`, `relationship_context`, and multi-subject `ReportGenerationRequest` — but no code consumes them.
- **No dyad UI** exists in `apps/`. That is deferred to a separate plan.

### Constraints
- The 12-pass L0 structure is the canonical spine; L1-L5 reuse the same 12 pass IDs with level-tuned templates.
- All 90 existing tests must stay green.
- Command Code `MiniMaxAI/MiniMax-M3` is the primary LLM provider; live runs cost ~$0.50-$1.00 per solo at ~22k words.
- Pattern memory Worker (`workers/pattern-memory/`) is not deployed yet — deployment is out of scope for this plan.
- Dyad/synastry partner data exists in `723/Solos/<slug>/01_input.json` with multi-subject birth data.

---

## Agent Ownership Model

| Role | Agent | Concerns |
|------|-------|----------|
| Mode doc author | Codex / Claude | `integrated-kundali-l1.md` through `-l5.md`, `partner-synastry.md` rewrite, pass templates |
| Orchestrator engineer | Codex / Claude | Dual-subject engine injection, `formatEngineResultsForBothSubjects`, rubric scaling |
| Dyad runner builder | Codex / Claude | Runner parallel to `solo-l0-runner.ts` for `subject_count > 1` |
| Validation engineer | Codex / Claude | Tests, e2e smoke with composite-dyad fixture, spot-check live runs |

---

## Phase Map

| Phase | Goal | Waves | Est. Duration |
|-------|------|-------|---------------|
| **Phase 1** | L0-to-L5 integrated kundali mode docs | 3 | 2 days |
| **Phase 2** | Dyad orchestrator extensions | 3 | 2 days |
| **Phase 3** | Partner-synastry mode doc + pass templates | 4 | 3 days |
| **Phase 4** | Dyad runner, validation, closeout | 3 | 2 days |

**Total**: 13 waves, ~9 days.

---

## Phase 1: L0-to-L5 Integrated Kundali Mode Docs

L0 is the baseline. L1-L5 reuse the same 12 pass IDs (`opening`, `convergence-map`, `vedic-foundation`, `karmic-architecture`, `career-dharma`, `wealth`, `love-marriage`, `health`, `family-lineage`, `master-timeline`, `remedies-practices`, `final-synthesis`) but tune pass templates and word targets per level.

The key design decision: **one mode doc file per report level**, each named `integrated-kundali-l<N>.md`. Each is a full copy of `integrated-kundali-l0.md` with level-tuned `report_level`, `target_words`, `register_variants`, and pass template content where consciousness depth differs.

### Level Scaling Table

| Level | Words (l1_l3) | Words (l4_l5) | Template Depth | Shadow/Gift Language | Bridge Mandate Additions |
|-------|---------------|---------------|----------------|---------------------|--------------------------|
| **L0** | 13k–17k | 17k–23k | Full 12-pass, engine facts grounded | Permitted for karmic-architecture | 7 baseline mandates |
| **L1** | 8k–12k | 14k–18k | 12-pass, simpler language, skip deep shadow | Softened to "unconscious patterns" | Add: "Use plain language. Avoid HD/Gene-Key jargon unless defined." |
| **L2** | 10k–14k | 16k–20k | 12-pass, introduce shadow as tension | "Shadow dynamics" ok, no Siddhi | Add: "Frame shadow as pattern, not pathology." |
| **L3** | 12k–16k | 17k–22k | 12-pass, full shadow/gift/siddhi permitted | Full spectrum allowed | Same as L0 + "Allow more interpretive depth." |
| **L4** | 14k–18k | 18k–24k | 12-pass, deeper shadow/gift language | Explicit shadow/gift/siddhi, contemplative frame | Add: "Use Siddhi language only as possibility, not identity." |
| **L5** | 15k–19k | 20k–28k | 12-pass, full depth with caveats | Same as L4 + "Name what is unknown honestly." | Add: "Distinguish engine-certain from interpretive. Leave mysteries unsolved." |

### Pass Template Variants per Level

Not every pass template changes per level. Only these passes have consciousness-sensitive content:

| Pass ID | Level-Varying? | What Changes |
|---------|---------------|-------------|
| `opening` | Yes | L1: welcoming, simple. L5: ceremonial, acknowledges depth. |
| `karmic-architecture` | Yes | L1: "patterns from your chart." L5: full shadow/gift/siddhi, HD gate/channel enumeration. |
| `health` | Yes | L1: "vitality tendencies." L5: biofield areas + chakra readings + biorhythm detail. |
| `remedies-practices` | Yes | L1: 2-3 gentle suggestions. L5: full nada ragas + Vedic corrections + caution language. |
| `final-synthesis` | Yes | L1: warm close. L5: "what is known / what is unknown / what you are invited to explore." |
| `wealth`, `love-marriage`, `career-dharma`, `family-lineage`, `master-timeline`, `vedic-foundation`, `convergence-map` | No | Same grounding checklists; consciousness level already embedded via `{{register}}`. |

---

### Wave 1.1: Scaffold `integrated-kundali-l1.md` and L1 pass templates

#### Task 1: Copy L0 mode doc to L1

**Files:**
- Create: `packages/witness-pipeline/modes/integrated-kundali-l1.md`

**Step 1: Prepare and verify existing tests pass first**

```bash
pnpm --filter witness-pipeline test --run
```
Expected: 90 passed.

**Step 2: Copy L0 → L1 and tune frontmatter**

Copy `modes/integrated-kundali-l0.md` → `modes/integrated-kundali-l1.md`, then edit frontmatter:

```yaml
---
mode: integrated-kundali-l1
report_level: L1
subject_count:
  min: 1
  max: 1
roles:
  - subject
target_words:
  min: 8000
  max: 12000
architecture: linear
# ... same pass_plan, engine_overlay_weights, house_overlay, bridge_mandates as L0 ...
register_variants:
  l1_l3:
    target_words:
      min: 8000
      max: 12000
  l4_l5:
    target_words:
      min: 14000
      max: 18000
---
```

**Step 3: Tune L1 opening-pass template**

Replace the `## opening-pass` section with L1-level language:

```markdown
## opening-pass

Write the opening for {{subject_names}}'s L1 integrated kundali reading — a welcoming, accessible entry point.

Register: {{register}}.

Prior context:
{{prior_pass}}

Overlay:
{{overlay_summary}}

Mandates:
{{bridge_mandates}}

Engine results:
{{engine_results}}

Target ~{{target_words}} words. Pass: {{pass_id}}.

Use warm, simple language. Introduce the systems (Vedic astrology, Human Design, Gene Keys) in plain terms before using any jargon. The reader may not know what a "nakshatra" or "gate" is — briefly define key terms on first use. Open with the subject's name and a grounded, non-prescriptive welcome.
```

**Step 4: Tune L1 karmic-architecture-pass template**

Edit `## karmic-architecture-pass`: soften shadow/gift language.

```
Begin by naming the active Gene Key activations and HD channels from the engines.

Use "patterns from your chart" rather than "shadow" language. Frame Gene Key dynamics as developmental themes, not fixed traits. When naming a key, always follow it with what the key suggests the person is growing into, not what they are "stuck in."
```

**Step 5: Tune L1 remedies-practices-pass template**

Edit `## remedies-practices-pass`: reduce prescription density.

```
Begin by naming the biofield areas of attention and the 2-3 most resonant nadabrahman recommendations from the engines.

Offer 3-5 gentle, optional practices across Vedic, HD, and Gene Key domains. Use "you might consider" and "some people find helpful" language. Do not prescribe gemstones, mantras, or ritual timings as required. Keep each suggestion brief (2-3 sentences each).
```

**Step 6: Tune L1 final-synthesis-pass template**

L1 already has `final-synthesis-pass-l1-l3` from L0. Copy it but add "for L1" specificity. Actually, reuse the existing `final-synthesis-pass-l1-l3` template — L1 uses it natively (register ≤ 3). No change needed.

**Step 7: Add `integrated-kundali-l1` to parser tests**

**File:** `packages/witness-pipeline/src/modes/parser.file.test.ts`

```typescript
it('parses integrated-kundali-l1 mode doc with register variants', () => {
  const path = resolve(import.meta.dirname, '../../modes/integrated-kundali-l1.md');
  const doc = parseModeDoc(readFileSync(path, 'utf8'));
  expect(doc.frontmatter.mode).toBe('integrated-kundali-l1');
  expect(doc.frontmatter.report_level).toBe('L1');
  expect(doc.frontmatter.pass_plan.length).toBe(12);
  const l1 = doc.frontmatter.register_variants?.l1_l3;
  expect(l1?.target_words?.min).toBeLessThan(13000);
});
```

**Step 8: Run tests**

```bash
pnpm --filter witness-pipeline test --run
```
Expected: 91 passed.

**Step 9: Commit**

```bash
git add packages/witness-pipeline/modes/integrated-kundali-l1.md \
        packages/witness-pipeline/src/modes/parser.file.test.ts
git commit -m "feat(witness-pipeline): add integrated-kundali-l1 mode doc with level-tuned templates"
```

---

### Wave 1.2: Scaffold L2 through L4 mode docs

#### Task 2: Create L2, L3, L4 mode docs with parser tests

**Files:**
- Create: `packages/witness-pipeline/modes/integrated-kundali-l2.md`
- Create: `packages/witness-pipeline/modes/integrated-kundali-l3.md`
- Create: `packages/witness-pipeline/modes/integrated-kundali-l4.md`

Follow the same pattern as Task 1 for each, tuning:

- L2: `target_words: { min: 10000, max: 14000 }`, `register_variants.l1_l3: { min: 10000, max: 14000 }`, `register_variants.l4_l5: { min: 16000, max: 20000 }`. Bridge mandate addition: "Frame shadow as pattern, not pathology."
- L3: `target_words: { min: 12000, max: 16000 }`, `register_variants.l1_l3: { min: 12000, max: 16000 }`, `register_variants.l4_l5: { min: 17000, max: 22000 }`. Bridge mandate addition: "Allow more interpretive depth when multiple engines converge on a theme."
- L4: `target_words: { min: 14000, max: 18000 }`, `register_variants.l1_l3: { min: 14000, max: 18000 }`, `register_variants.l4_l5: { min: 18000, max: 24000 }`. Bridge mandate addition: "Use Siddhi language only as contemplative possibility, not identity claim."

For pass templates, L2-L4 use the same L0 templates but L3/L4 adopt the deeper shadow/gift language in `karmic-architecture-pass` and `health` pass. The `{{register}}` placeholder already indicates depth level to the LLM. Minimal template changes are needed — mostly the "Begin by naming..." grounding checklists stay identical across L0-L5 because the engine facts don't change, only the interpretive language depth changes.

Add one parser test per level (L2, L3, L4).

**Step: Run tests**

```bash
pnpm --filter witness-pipeline test --run
```
Expected: 93 passed (3 new tests).

**Step: Commit**

```bash
git add packages/witness-pipeline/modes/integrated-kundali-l{2,3,4}.md \
        packages/witness-pipeline/src/modes/parser.file.test.ts
git commit -m "feat(witness-pipeline): add integrated-kundali-l2 l3 l4 mode docs with level-tuned configs"
```

---

### Wave 1.3: Scaffold L5 and run a spot-check solo

#### Task 3: Create L5 mode doc (+ spot-check live run)

**Files:**
- Create: `packages/witness-pipeline/modes/integrated-kundali-l5.md`

L5 target: `target_words: { min: 15000, max: 19000 }`, `register_variants.l1_l3` omitted (L5 is always consciousness level 5 → l4_l5 band), `register_variants.l4_l5: { min: 20000, max: 28000 }`. Bridge mandate additions:
- "Distinguish engine-certain from interpretive. Leave mysteries unsolved."
- "Name what is unknown honestly. The reader earns depth by being given honest limits."

L5 pass templates get deeper shadow/gift/siddhi language in `karmic-architecture-pass`, `health`, `remedies-practices`, and `final-synthesis-pass`.

Add parser test for L5. Expected: 94 passed.

Then do a spot-check with a known-good solo (e.g., `prashanth`):

```bash
# Modify solo-l0-runner.ts to accept --mode flag (Task 4 handles this);
# for now, hack: copy L4 mode over L0, run solo.
cp packages/witness-pipeline/modes/integrated-kundali-l4.md packages/witness-pipeline/modes/integrated-kundali-l4-backup.md
cp packages/witness-pipeline/modes/integrated-kundali-l4.md packages/witness-pipeline/modes/integrated-kundali-l0.md
cd packages/witness-pipeline && npx tsx scripts/solo-l0-runner.ts "/Volumes/madara/2026/twc-vault/01-Projects/tryambakam-noesis/723/Solos/prashanth" --live
# Restore L0
mv packages/witness-pipeline/modes/integrated-kundali-l4-backup.md packages/witness-pipeline/modes/integrated-kundali-l0.md
```

Validate that L4 produces a valid run (PASS, word count in L4 range). Expected: verification PASS, ~14k-18k words.

```bash
git add packages/witness-pipeline/modes/integrated-kundali-l5.md \
        packages/witness-pipeline/src/modes/parser.file.test.ts
git commit -m "feat(witness-pipeline): add integrated-kundali-l5 mode with deepest template depth"
```

---

## Phase 2: Dyad Orchestrator Extensions

### Wave 2.1: Format engine results for both subjects

#### Task 4: Add `formatEngineResultsForBothSubjects` to engine-formatter

**Files:**
- Modify: `packages/witness-pipeline/src/orchestrator/engine-formatter.ts`
- Modify: `packages/witness-pipeline/src/orchestrator/engine-formatter.test.ts`

**Step 1: Write the failing test**

```typescript
it('formats two subjects side-by-side with subject labels', () => {
  const subjectA = [{ engine_id: 'panchanga', result: { nakshatra_name: 'Ashwini' } }];
  const subjectB = [{ engine_id: 'panchanga', result: { nakshatra_name: 'Bharani' } }];
  const out = formatEngineResultsForBothSubjects(subjectA, subjectB, ['Arathi', 'Rohan']);
  expect(out).toContain('Subject: Arathi');
  expect(out).toContain('Subject: Rohan');
  expect(out).toContain('Ashwini');
  expect(out).toContain('Bharani');
});

it('formats both subjects with shared engine overlay when weights provided', () => {
  const subjectA = [{ engine_id: 'panchanga', result: { nakshatra_name: 'Ashwini' } }];
  const subjectB = [{ engine_id: 'panchanga', result: { nakshatra_name: 'Bharani' } }];
  const out = formatEngineResultsForBothSubjects(subjectA, subjectB, ['A', 'B'], { panchanga: 1.0 });
  expect(out).toContain('Shared overlay weight: panchanga 1.0');
});
```

**Expected:** FAIL — `formatEngineResultsForBothSubjects` not defined.

**Step 2: Run test to verify it fails**

```bash
pnpm --filter witness-pipeline test -- --run -t 'formatEngineResultsForBothSubjects'
```
Expected: FAIL.

**Step 3: Write implementation**

Add to `engine-formatter.ts`:

```typescript
export function formatEngineResultsForBothSubjects(
  enginesA: any[],
  enginesB: any[],
  subjectNames: string[],
  overlayWeights?: Record<string, number>,
): string {
  const lines: string[] = ['## Deterministic Engine Results — Both Subjects'];
  if (subjectNames.length >= 2) {
    lines.push(`Subject: ${subjectNames[0]}`);
  }
  lines.push(formatEngineResultsForPrompt(enginesA));
  lines.push('');
  if (subjectNames.length >= 2) {
    lines.push(`Subject: ${subjectNames[1]}`);
  }
  lines.push(formatEngineResultsForPrompt(enginesB));
  if (overlayWeights && Object.keys(overlayWeights).length > 0) {
    lines.push('');
    lines.push('Shared overlay weights: ' + Object.entries(overlayWeights)
      .map(([k, v]) => `${k} ${v}`).join(', '));
  }
  return lines.join('\n');
}
```

**Step 4: Run test to verify it passes**

```bash
pnpm --filter witness-pipeline test --run
```
Expected: 95 passed.

**Step 5: Commit**

```bash
git add packages/witness-pipeline/src/orchestrator/engine-formatter.ts \
        packages/witness-pipeline/src/orchestrator/engine-formatter.test.ts
git commit -m "feat(engine-formatter): add formatEngineResultsForBothSubjects for dyad prompts"
```

---

### Wave 2.2: Extend orchestrator for multi-subject engine injection

#### Task 5: Wire dual-subject engine results into orchestrator prompts

**Files:**
- Modify: `packages/witness-pipeline/src/orchestrator/integrated.ts:122-145`
- Modify: `packages/witness-pipeline/src/orchestrator/integrated.test.ts`

**Step 1: Write failing test for dual-subject rendering**

```typescript
it('uses both subjects'' engine results when subject_count > 1', async () => {
  const engineA = [{ engine_id: 'panchanga', result: { nakshatra_name: 'Ashwini' } }];
  const engineB = [{ engine_id: 'panchanga', result: { nakshatra_name: 'Bharani' } }];
  // Create a mode doc with subject_count: { min:2, max:2 }
  const multiSubjectDoc = `---
mode: test-dyad
subject_count:
  min: 2
  max: 2
roles:
  - subject-a
  - subject-b
target_words:
  min: 500
  max: 1000
architecture: linear
pass_plan:
  - id: alpha
    title: Alpha
    target_words: 600
    template: pass-alpha
engine_overlay_weights:
  panchanga: 1.0
house_overlay: [1]
bridge_mandates:
  - "Test mandate"
svg_topology: dyad-arc
---
## pass-alpha
Write about {{subject_names}}.
{{engine_results}}
Target ~{{target_words}} words. Pass: {{pass_id}}.
`;
  const mode = parseModeDoc(multiSubjectDoc);
  const llm = async (_s: string, prompt: string, _opts: { max_tokens: number }) => prompt;
  const orchestrator = new IntegratedReadingOrchestrator({ mode, llm });
  const result = await orchestrator.run({
    subjectNames: ['Arathi', 'Rohan'],
    engineResultsBySubject: [engineA, engineB],
    consciousnessLevel: 2,
  });
  const output = result.passes[0].output;
  expect(output).toContain('Arathi');
  expect(output).toContain('Rohan');
  expect(output).toContain('Ashwini');
  expect(output).toContain('Bharani');
});
```

**Step 2: Run to verify failure**

```bash
pnpm --filter witness-pipeline test --run -t 'both subjects'
```
Expected: FAIL — Bharani not found (only Subject A engine results are rendered).

**Step 3: Implement dual-subject rendering in orchestrator**

In `IntegratedReadingOrchestrator.run()`, after line 176 (`const engineResults = formatEngineResultsForPrompt(...)`), add dual-subject detection:

```typescript
const isDualSubject = (this.mode.frontmatter.subject_count?.max ?? 1) > 1;

let engineResults: string;
if (isDualSubject && input.engineResultsBySubject.length >= 2) {
  engineResults = formatEngineResultsForBothSubjects(
    input.engineResultsBySubject[0] ?? [],
    input.engineResultsBySubject[1] ?? [],
    input.subjectNames,
    this.mode.frontmatter.engine_overlay_weights,
  );
} else {
  engineResults = formatEngineResultsForPrompt(input.engineResultsBySubject[0] ?? []);
}
```

**Step 4: Also extend rubric audit for dyad sections**

In the rubric audit call (line 143), pass both subjects' engine results when `isDualSubject`:

```typescript
const rubricEngines = isDualSubject
  ? [...(input.engineResultsBySubject[0] ?? []), ...(input.engineResultsBySubject[1] ?? [])]
  : (input.engineResultsBySubject[0] ?? []);
const rubric = auditSectionOutput({
  // ... existing fields ...
  engineResults: rubricEngines,
});
```

This gives the fidelity gate access to both subjects' facts so it can verify that both charts are mentioned.

**Step 5: Run tests**

```bash
pnpm --filter witness-pipeline test --run
```
Expected: 96 passed.

**Step 6: Commit**

```bash
git add packages/witness-pipeline/src/orchestrator/integrated.ts \
        packages/witness-pipeline/src/orchestrator/integrated.test.ts
git commit -m "feat(orchestrator): dual-subject engine result injection for dyad/synastry"
```

---

### Wave 2.3: Dyad-specific pattern kinds and retrieval filters

#### Task 6: Add dyad pattern kinds and retrieval filters

**Files:**
- Modify: `packages/witness-pipeline/src/patterns/types.ts:3`
- Modify: `packages/witness-pipeline/src/patterns/extractor.ts:27-33` (inferKind)
- Modify: `packages/witness-pipeline/src/patterns/retrieval.ts` (RetrievalFilters)
- Modify: `packages/witness-pipeline/src/patterns/extractor.test.ts`

**Step 1: Add dyad PatternKind variants**

```typescript
export type PatternKind =
  | 'convergence'
  | 'tension'
  | 'guardrail-safe-framing'
  | 'section-structure'
  | 'remedy-pattern'
  | 'compatibility-arc'       // dyad: shared complementary patterns
  | 'friction-point'           // dyad: chart tension between subjects
  | 'timing-synchronicity'     // dyad: aligned dasha/transit periods
  | 'shared-activation';       // dyad: same gate/channel/nakshatra
```

**Step 2: Extend `inferKind` in extractor.ts**

```typescript
if (/compat|shared|complement|harmon/i.test(sectionId)) return 'compatibility-arc';
if (/friction|conflict|tension|frict/i.test(sectionId)) return 'friction-point';
if (/timing|synchronicity|aligned/i.test(sectionId)) return 'timing-synchronicity';
if (/shared.activ|same.gate|overlap/i.test(sectionId)) return 'shared-activation';
```

**Step 3: Extend `RetrievalFilters`**

```typescript
export interface RetrievalFilters {
  mode?: string;
  report_level?: string;
  kind?: string;
  version?: string;
  systems?: string[];
  subject_count?: number;           // dyad: filter by 1 (solo) or 2+ (dyad)
  relationship_type?: string;       // dyad: partner-synastry, family, etc.
}
```

**Step 4: Add extractor test for dyad kinds**

```typecript
it('infers compatibility-arc from complement section id', () => {
  expect(inferKind('subject-complement-harmony')).toBe('compatibility-arc');
});
it('infers friction-point from conflict section id', () => {
  expect(inferKind('chart-friction-tension')).toBe('friction-point');
});
```

**Step 5: Run tests**

```bash
pnpm --filter witness-pipeline test --run
```
Expected: 98 passed.

**Step 6: Commit**

```bash
git add packages/witness-pipeline/src/patterns/types.ts \
        packages/witness-pipeline/src/patterns/extractor.ts \
        packages/witness-pipeline/src/patterns/retrieval.ts \
        packages/witness-pipeline/src/patterns/extractor.test.ts
git commit -m "feat(patterns): add dyad-specific PatternKind variants and retrieval filters"
```

---

## Phase 3: Partner-Synastry Mode Doc + Pass Templates

This is the largest piece. `partner-synastry.md` today is a 43-line prose spec with zero conforming fields. It must be rewritten from scratch.

### Design: Partner-Synastry 5-Pass Structure

Per the original prose spec (Interpretation Flow section), partner synastry has 5 conceptual stages. We formalize them as 5 passes:

| Pass ID | Title | Target Words | Purpose |
|---------|-------|-------------|---------|
| `subject-a` | Subject A — Individual Chart | 2500 | A's chart in isolation, grounding the reading in who A is |
| `subject-b` | Subject B — Individual Chart | 2500 | B's chart in isolation, grounding the reading in who B is |
| `aletheios-structural` | Aletheios — Structural Convergence | 3000 | Deterministic comparison: shared gates, overlapping dashas, complementary nakshatras |
| `pichet-energetic` | Pichet — Energetic Dance | 3000 | Interpretive: how the two charts interact energetically, tension and complement |
| `synthesis` | Synthesis — What This Pair Is Invited Into | 2500 | Braided takeaway: what works, what's hard, what's possible |

Total: ~13,500 words (dyad baseline).

Register variants:
- `l1_l3`: 11k–14k words, softer language
- `l4_l5`: 14k–18k words, deeper shadow/gift permitted

---

### Wave 3.1: Scaffold conforming partner-synastry mode doc

#### Task 7: Rewrite `partner-synastry.md` with full ModeConfig schema

**Files:**
- Rewrite: `packages/witness-pipeline/modes/partner-synastry.md`
- Modify: `packages/witness-pipeline/src/modes/parser.file.test.ts`

**Step 1: Write the full mode doc**

```markdown
---
mode: partner-synastry
report_level: L2
subject_count:
  min: 2
  max: 2
roles:
  - subject-a
  - subject-b
target_words:
  min: 11000
  max: 14000
architecture: linear
pass_plan:
  - id: subject-a
    title: Subject A — Individual Chart
    target_words: 2500
    template: subject-a-pass
  - id: subject-b
    title: Subject B — Individual Chart
    target_words: 2500
    template: subject-b-pass
  - id: aletheios-structural
    title: Aletheios — Structural Convergence
    target_words: 3000
    template: aletheios-structural-pass
  - id: pichet-energetic
    title: Pichet — Energetic Dance
    target_words: 3000
    template: pichet-energetic-pass
  - id: synthesis
    title: Synthesis — What This Pair Is Invited Into
    target_words: 2500
    template: synthesis-pass
engine_overlay_weights:
  panchanga: 1.0
  human-design: 0.9
  vimshottari: 0.85
  gene-keys: 0.75
  transits: 0.7
  numerology: 0.45
house_overlay: [1, 4, 5, 7, 8, 10, 11]
bridge_mandates:
  - "Braid deterministic facts from both charts before any interpretation."
  - "Name structural overlaps (shared gates, channels, nakshatras, overlapping dashas) explicitly."
  - "Surface tension points honestly but without predicting relationship outcomes."
  - "Use Aletheios for structure (deterministic comparison) and Pichet for vitality (energetic reading)."
  - "Do not predict marriage, divorce, relationship duration, or outcome. Readings are pattern-observation, not fate."
  - "When both charts converge on a theme, name it as convergence. When they diverge, name it as difference without ranking."
  - "Cite only facts present in the supplied engine results. Never invent or generalize chart positions."
svg_topology: dyad-arc
register_variants:
  l1_l3:
    target_words:
      min: 11000
      max: 14000
    overrides:
      - pass_id: synthesis
        template: synthesis-pass-l1-l3
  l4_l5:
    target_words:
      min: 14000
      max: 18000
---

## subject-a-pass

Write Subject A's individual chart reading for the partner synastry report. Subject A: {{subject_names}}.

Register: {{register}}.

Prior context:
{{prior_pass}}

Overlay:
{{overlay_summary}}

Mandates:
{{bridge_mandates}}

Engine results (Subject A):
{{engine_results}}

Target ~{{target_words}} words. Pass: {{pass_id}}.

Begin by naming Subject A's lagna, Moon nakshatra, HD type/profile/authority, and current mahadasha from the engines.

Describe who Subject A is individually — their structural patterns, vitality tendencies, core drives, and what they bring to relationship. This is NOT a comparison yet. Ground every claim in engine facts. Use accessible language. Leave space for the other person — this is half the picture.
```

(The remaining 4 pass templates follow the same pattern: `## subject-b-pass`, `## aletheios-structural-pass`, `## pichet-energetic-pass`, `## synthesis-pass` — each with grounding checklists and guardrail language.)

For the full doc, include all 5 pass templates with:
- `{{engine_results}}` placeholder (the orchestrator will populate with both subjects' data when dual-subject mode is detected)
- "Begin by naming..." grounding checklists
- Guardrail language (no relationship outcome prediction, no marriage/divorce prediction)

**Step 2: Add parser test**

```typescript
it('parses partner-synastry with dual subjects and 5-pass plan', () => {
  const path = resolve(import.meta.dirname, '../../modes/partner-synastry.md');
  const doc = parseModeDoc(readFileSync(path, 'utf8'));
  expect(doc.frontmatter.mode).toBe('partner-synastry');
  expect(doc.frontmatter.subject_count?.min).toBe(2);
  expect(doc.frontmatter.subject_count?.max).toBe(2);
  expect(doc.frontmatter.pass_plan.length).toBe(5);
});
```

**Step 3: Run tests**

```bash
pnpm --filter witness-pipeline test --run
```
Expected: 99 passed.

**Step 4: Commit**

```bash
git add packages/witness-pipeline/modes/partner-synastry.md \
        packages/witness-pipeline/src/modes/parser.file.test.ts
git commit -m "feat(witness-pipeline): rewrite partner-synastry mode doc with conforming schema and 5-pass plan"
```

---

### Wave 3.2: Synastry-specific rubric thresholds and guardrails

#### Task 8: Add dyad rubric thresholds and synastry guardrails

**Files:**
- Modify: `packages/witness-pipeline/src/orchestrator/rubric.ts`
- Modify: `packages/witness-pipeline/src/orchestrator/rubric.test.ts`

**Step 1: Add dyad-specific guardrails**

```typescript
const GUARDRAILS: Record<string, RegExp[]> = {
  // ... existing ...
  'subject-a': [/will marry/i, /destined for/i, /inevitable/i],
  'subject-b': [/will marry/i, /destined for/i, /inevitable/i],
  'aletheios-structural': [/guarantee/i, /perfect match/i, /soul.?mate/i],
  'pichet-energetic': [/guarantee/i, /perfect match/i, /must/i],
  synthesis: [/guarantee/i, /definitely/i, /will last/i, /soul.?mate/i],
};
```

**Step 2: Add dyad section thresholds**

```typescript
const SECTION_THRESHOLDS: Record<string, { minFacts: number; minLayers: number }> = {
  // ... existing ...
  'subject-a': { minFacts: 10, minLayers: 4 },
  'subject-b': { minFacts: 10, minLayers: 4 },
  'aletheios-structural': { minFacts: 8, minLayers: 5 },  // needs facts from BOTH charts
  'pichet-energetic': { minFacts: 4, minLayers: 3 },       // interpretive, fewer engine facts required
};
```

**Step 3: Write tests for dyad guardrails**

```typescript
it('flags soulmate language in synthesis pass', () => {
  const input = { sectionId: 'synthesis', title: 'Synth', targetWords: 2000, output: 'You are soulmates destined for each other.', modelRequested: 'x', modelUsed: 'x', latencyMs: 100 };
  const rubric = auditSectionOutput(input);
  expect(rubric.guardrail_gate).toBe('fail');
});
```

**Step 4: Run tests**

```bash
pnpm --filter witness-pipeline test --run
```
Expected: 100 passed.

**Step 5: Commit**

```bash
git add packages/witness-pipeline/src/orchestrator/rubric.ts \
        packages/witness-pipeline/src/orchestrator/rubric.test.ts
git commit -m "feat(rubric): add dyad-specific guardrails and section thresholds for synastry passes"
```

---

### Wave 3.3: Extend section-fact extraction for dyad sections

#### Task 9: Add `extractSectionFacts` cases for dyad passes

**Files:**
- Modify: `packages/witness-pipeline/src/orchestrator/rubric.ts:191` (extractSectionFacts switch)
- Modify: `packages/witness-pipeline/src/orchestrator/rubric.test.ts`

The `subject-a` and `subject-b` passes use the full generic fact set (like `vedic-foundation`). The `aletheios-structural` pass extracts facts from both subjects' overlapping data (shared nakshatras, gates, channels, overlapping dasha timelines, shared defined centers).

```typescript
case 'subject-a':
case 'subject-b':
  facts = extractGenericFacts(engines); // full facts for individual chart
  break;
case 'aletheios-structural': {
  // Extract shared/overlapping facts: common gates, channels, nakshatras, overlapping dashas
  // engines now contain both subjects' data (merged from engineResultsBySubject)
  const allGates = new Set<number>();
  // ... collect all gate numbers from both charts, report on overlaps ...
  break;
}
```

**Test**: Write a test that passes two subjects' merged engine results and verifies shared-fact extraction.

```bash
pnpm --filter witness-pipeline test --run
# Expected: 101 passed.
```

```bash
git add packages/witness-pipeline/src/orchestrator/rubric.ts \
        packages/witness-pipeline/src/orchestrator/rubric.test.ts
git commit -m "feat(rubric): add dyad section-specific fact extraction for synastry passes"
```

---

### Wave 3.4: Dyad-specific guardrail scaling for partner-synastry

#### Task 10: Add per-fidelity thresholds for dyad sections

**Files:**
- Modify: `packages/witness-pipeline/src/orchestrator/rubric.ts:61-74` (FIDELITY_THRESHOLDS)

Add entries for the 5 dyad pass IDs:

```typescript
'subject-a': { pass: 0.35, warn: 0.2 },
'subject-b': { pass: 0.35, warn: 0.2 },
'aletheios-structural': { pass: 0.25, warn: 0.1 },  // harder: facts from both charts
'pichet-energetic': { pass: 0.2, warn: 0.1 },       // interpretive, lower bar
```

```bash
git add packages/witness-pipeline/src/orchestrator/rubric.ts
git commit -m "feat(rubric): add fidelity thresholds for partner-synastry dyad passes"
```

---

## Phase 4: Dyad Runner, Validation, Closeout

### Wave 4.1: Build dyad runner script

#### Task 11: Create `dyad-l0-runner.ts`

**Files:**
- Create: `packages/witness-pipeline/scripts/dyad-synastry-runner.ts`

Parallel to `solo-l0-runner.ts`. Key differences:

1. Reads `request.json` and expects `request.subjects.length >= 2`.
2. Extracts both `subject[0]` and `subject[1]` with normalized locations.
3. Calls `fetchAllEngines()` for each subject (or uses pre-fetched `engines.json` for each from `new-l0-flow/engines-a.json` and `engines-b.json`).
4. Passes `engineResultsBySubject: [enginesA, enginesB]` to the orchestrator.
5. Uses the `partner-synastry` mode doc.
6. Builds `ReportGenerationRequest` with `subjects: [{ role: 'subject-a', ... }, { role: 'subject-b', ... }]`.

```typescript
// Usage: tsx scripts/dyad-synastry-runner.ts <solo-dir> [--live]
// Expects: <solo-dir>/new-l0-flow/engines-subject-a.json + engines-subject-b.json
//          or: <solo-dir>/new-l0-flow/engines.json (single file with both)
```

**Step 1: Write basic integration test**

In `packages/witness-pipeline/scripts/` create a test that uses the composite-dyad fixture and stub LLM:

```typescript
// dyad-synastry-runner.test.ts
it('runs dyad runner with stub LLM against composite-dyad fixture', async () => {
  // Use the composite-dyad mode fixture, stub engines, stub LLM
  // Expect: verification PASS, 5 passes, ~11k-14k words
});
```

**Step 2: Run tests**

```bash
pnpm --filter witness-pipeline test --run
```
Expected: 102 passed.

**Step 3: Commit**

```bash
git add packages/witness-pipeline/scripts/dyad-synastry-runner.ts \
        packages/witness-pipeline/scripts/dyad-synastry-runner.test.ts
git commit -m "feat(runner): add dyad-synastry runner for multi-subject partner readings"
```

---

### Wave 4.2: Live dyad spot-check

#### Task 12: Run partner-synastry with a known dyad pair

Pick a dyad pair that has engine data pre-fetched (e.g., `723/Solos/arathi-rohan` if it exists, otherwise create a minimal dyad fixture with real birth data from any two existing solos). Run:

```bash
cd packages/witness-pipeline && npx tsx scripts/dyad-synastry-runner.ts "/Volumes/madara/2026/twc-vault/01-Projects/tryambakam-noesis/723/Solos/<dyad-pair>" --live 2>&1 | tail -30
```

Expected output: verification PASS or FAIL, passes count = 5, word count in dyad range, both subjects' chart facts mentioned in output.

If no dyad pair fixture exists, create one rapidly:

```bash
mkdir -p 723/Solos/arathi-rohan/new-l0-flow
# Copy engines from solo runs
cp 723/Solos/arathi/new-l0-flow/engines.json 723/Solos/arathi-rohan/new-l0-flow/engines-subject-a.json
cp 723/Solos/rohan/new-l0-flow/engines.json 723/Solos/arathi-rohan/new-l0-flow/engines-subject-b.json
# Write request.json with both subjects
```

**Commit spot-check results:**

```bash
git add 723/Solos/arathi-rohan/
git commit -m "test(dyad): add arathi-rohan dyad fixture and spot-check run"
```

---

### Wave 4.3: Documentation and closeout

#### Task 13: Update README with L1-L5 and dyad usage

**Files:**
- Modify: `packages/witness-pipeline/README.md`

Add sections:

```markdown
### Running L1-L5 Integrated Kundali Readings

Each level uses its own mode doc. L1 is the most accessible; L5 is the deepest.

```bash
tsx scripts/solo-l0-runner.ts <solo-dir> --mode integrated-kundali-l3 --live
```

Word targets scale per level: L1 (8k-12k) through L5 (15k-19k for l1_l3, 20k-28k for l4_l5).

### Running Dyad / Partner Synastry

```bash
tsx scripts/dyad-synastry-runner.ts <dyad-dir> --live
```

Requires `new-l0-flow/engines-subject-a.json` and `engines-subject-b.json` (or a single `engines.json` with both subjects' data). Uses the 5-pass partner-synastry mode.
```

#### Task 14: Run full test suite and closeout

```bash
pnpm --filter witness-pipeline test --run
```

Expected: 102+ tests, all green. Update `MASTER-AUTO-RESEARCH.md` with Phase 1-4 completion status.

```bash
git add packages/witness-pipeline/README.md MASTER-AUTO-RESEARCH.md
git commit -m "docs: add L1-L5 and dyad/synastry usage to README and auto-research closeout"
```

---

## Phase Dependency Graph

```
Phase 1 (L1-L4 mode docs) ──► (parallel to Phase 2, no dependency)
Phase 2 (orchestrator dyad) ──► Phase 3 (synastry mode doc depends on dual-subject rendering)
Phase 3 (synastry mode doc) ──► Phase 4 (dyad runner depends on synastry mode doc)
```

Phases 1 and 2 can run in parallel. Phase 3 depends on Phase 2 (orchestrator must support dual-subject before synastry mode doc can work). Phase 4 depends on Phase 3.

---

## Timeline

| Phase | Days | Deliverable |
|--------|------|-------------|
| Phase 1 | 2 | L1-L5 mode docs with parser tests |
| Phase 2 | 2 | `formatEngineResultsForBothSubjects`, orchestrator dual-subject, dyad pattern kinds |
| Phase 3 | 3 | `partner-synastry.md` with 5-pass plan, dyad rubric thresholds, dyad fact extraction |
| Phase 4 | 2 | Dyad runner, live spot-check, docs |
| **Total** | **9** | **L0-L5 pipeline + dyad/synastry with live validation** |

---

## GitHub Roadmap Mapping

| Issue | Title | Labels | Depends on |
|--------|-------|--------|------------|
| #5 | Add L1-L5 integrated kundali mode docs with level-tuned templates | `enhancement`, `witness-pipeline`, `L0-L5` | — |
| #6 | Extend orchestrator for dual-subject dyad engine injection | `enhancement`, `witness-pipeline`, `dyad` | — |
| #7 | Rewrite partner-synastry mode doc with conforming schema | `enhancement`, `witness-pipeline`, `dyad` | #6 |
| #8 | Build dyad runner and closeout | `enhancement`, `runner`, `dyad` | #5, #7 |

---

## Risks & Fallbacks

| Risk | Fallback |
|--------|----------|
| L5 pass templates produce overly verbose output (exceeding max_tokens limits) | Cap at L4-style templates; L5 only differs in mandated language, not pass count. |
| Dyad LLM runs cost 2x solo (two charts, more tokens) | Validate with stub LLM first; live spot-check only one dyad pair. |
| `partner-synastry.md` 5-pass structure doesn't produce good output | Reduce to 3 passes (individual-A, individual-B, synthesis) and iterate. |
| Live dyad pair doesn't exist in 723/Solos/ | Rapid-create a fixture from any two existing solos with real birth data. |
| `command-code` usage limits block live runs | Use stub LLM only for dyad; live validation already proven for solo L0. |

---

## Verification Strategy

- Parse every new mode doc through `parseModeDoc()` — 4 new parser tests per level.
- `pnpm --filter witness-pipeline test --run` green after every task (103+ tests expected at closeout).
- Rubric guardrails fire correctly for dyad passes (no "soulmate" / "destined" language tolerated).
- Dual-subject engine formatting renders both subjects' chart facts.
- Dyad runner stub-LLM test produces 5 passes with correct word targets.
- Live dyad spot-check on one pair produces a PASS with both subjects' engine facts present.

---

## Execution Handoff

Plan complete and saved to `docs/plans/2026-07-09-l0-to-l5-integrated-kundali-and-dyad-synastry.md`.

Two execution options:

**1. Subagent-Driven (this session)** — I dispatch a fresh subagent per task, review between tasks, fast iteration. Fits well here because phases 1 and 2 are parallelizable.

**2. Parallel Session (separate)** — Open a new session in the worktree with `superpowers:executing-plans` for batch execution with checkpoints.

**Which approach?**