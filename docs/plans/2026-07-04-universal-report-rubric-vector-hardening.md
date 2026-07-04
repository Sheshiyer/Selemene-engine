# Universal Report Rubric and Pattern Hardening Implementation Plan

> **For Codex:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add a universal per-section rubric matrix and post-report pattern-hardening loop to all L0-L5 report generation modes so every report records quality evidence, extracts reusable synthesis patterns, and hardens future inference through Cloudflare Vectorize.

**Architecture:** Treat every generated report as a sequence of measurable passes plus a learning tail. The orchestrator remains mode-doc driven, but each `PassResult` gains rubric metadata computed by deterministic audit functions after the model returns; after final assembly, a pattern extractor distills anonymized reusable synthesis patterns and writes approved embeddings to Cloudflare Vectorize. Source packs and API responses carry both section evidence and pattern-learning provenance instead of only whole-reading quality.

**Tech Stack:** TypeScript/Vitest in `packages/witness-pipeline` and `packages/noesis-sdk-ts`, Rust/Axum contract parity in `crates/noesis-api`, existing mode docs under `packages/witness-pipeline/modes`, existing NVIDIA/OpenRouter model routing through the injected LLM function and `crates/noesis-witness/src/llm.rs`, Cloudflare Workers AI for embeddings, Cloudflare Vectorize for semantic pattern memory, and R2/D1/Postgres for canonical source records.

---

## Engineering Problem Statement

The current premium report pipeline has mode docs, pass plans, target word counts, and prompt safeguards, but the system cannot prove section quality across all report levels. It can say a pass targeted 1200 words, but it cannot report whether the output was close to 1200 words. It can ask the model to cite deterministic facts, but it cannot count chart facts per section. It can include guardrail text in prompts, but it cannot produce an auditable pass/fail matrix for sensitive domains. It can call an LLM per pass, but it does not record model requested, model used, or latency per section.

The bigger issue: each finished report currently ends as an artifact, not as training signal. The best synthesis patterns inside a report are not extracted, anonymized, embedded, scored, versioned, or reused. The system therefore relearns the same integrations on every run.

The solution is not to add more prompt prose. The solution is to define report quality and report learning as measurable primitives:

1. Every section emits a deterministic quality vector.
2. Every full report emits a post-report pattern set.
3. Only safe, anonymized, high-scoring patterns are embedded into Vectorize.
4. Future reports retrieve these patterns as optional synthesis context, never as deterministic facts.

## Scope Correction: Report Levels vs Consciousness Registers

Current code already has `consciousness_level` and `register`:

- `consciousness_level <= 3` maps to `l1_l3`.
- `consciousness_level >= 4` maps to `l4_l5`.

For this plan, introduce report generation levels as a separate product concept:

| Report Level | Purpose | Typical Shape | Learning Output |
|---|---|---|---|
| L0 | baseline exemplar-quality long report | full sectioned report | canonical pattern seeds |
| L1 | concise personal mirror | 1-2 passes | lightweight pattern examples |
| L2 | guided report with basic cross-system synthesis | 2-4 passes | convergence/tension patterns |
| L3 | integrated report | 3-6 passes | reusable section patterns |
| L4 | deep dyad report | 6-10 passes | high-density multi-layer patterns |
| L5 | comprehensive premium report | 10+ passes, source pack, audit | strongest reusable synthesis patterns |

Do not overload `consciousness_level` to mean report level. Add `report_level?: 'L0' | 'L1' | 'L2' | 'L3' | 'L4' | 'L5'` to mode metadata or premium asset options later.

## Required Intake Layer Before Report Generation

All CLI/OpenCode/Codex/Claude UI report flows must produce a normalized `ReportGenerationRequest` before generation. The intake layer is the contract between human-facing structured questions and the report engine.

The subject shape must include location normalization and gender/sex fields separately from chart data:

```ts
export interface ReportSubjectInput {
  role: 'primary' | 'partner' | 'family_member' | 'friend' | 'business_partner' | 'custom';
  relationship_label?: string;
  name: string;
  gender?: 'female' | 'male' | 'nonbinary' | 'other' | 'prefer_not_to_say' | 'unknown';
  sex_for_external_chart_source?: 'female' | 'male' | 'unknown';
  birth_date: string;
  birth_time?: string;
  birth_time_confidence: 'exact' | 'approximate' | 'unknown';
  birth_location_query: string;
  normalized_location?: NormalizedLocation;
}

export interface NormalizedLocation {
  display_name: string;
  latitude: number;
  longitude: number;
  timezone: string;
  provider: 'manual' | 'nominatim' | 'google-places' | 'mapbox' | 'geonames';
  confidence: 'exact' | 'selected' | 'ambiguous' | 'manual';
}
```

Gender must be asked because report language, relationship role labels, and external chart providers may need it. It must not be used to make deterministic claims unless a downstream engine explicitly requires sex/gender input. Keep `gender` for reader-facing identity and `sex_for_external_chart_source` only for provider compatibility.

Location must be resolved through a picker, not silent best-guess geocoding. Existing repo precedent: `tools/humdes-extractor/humdes_to_selemene.py` uses cached, rate-limited Nominatim lookup and stores `lat`, `lng`, `display_name`, `timezone`, and `humdes_sex`. Reuse that pattern conceptually, but expose ambiguity to the user in interactive flows.

Recommended provider strategy:

| Surface | Default | Fallback | Why |
|---|---|---|---|
| CLI / local agent | Nominatim cached lookup | manual lat/long/timezone | free, enough for operator use |
| Web UI | Places/Mapbox picker | manual lat/long/timezone | better UX and disambiguation |
| Batch/import | pre-normalized lat/long/timezone | Nominatim with cache | reproducible imports |
| Private/sensitive mode | manual coordinates | no network lookup | avoids sending birthplace to third party |

The picker output must be shown back to the user:

```text
Birthplace selected: Jamakhandi, Bagalkote, Karnataka, India
Latitude: 16.5046
Longitude: 75.2918
Timezone: Asia/Kolkata
Use this location?
```

No report should run until every required subject has either a confirmed `normalized_location` or an explicit manual coordinate/timezone override.

## First-Principles Model

A generated report section has six measurable dimensions:

```text
S_i = section i
O_i = model output for section i
T_i = target word count for section i
F_i = deterministic fact references in O_i
L_i = integrated system layers represented in O_i
G_i = guardrail violations in O_i
M_i = model metadata for O_i
R_i = runtime metadata for O_i
```

The section quality vector is:

```text
Q_i = {
  word_fit_i,
  fact_grounding_i,
  layering_i,
  guardrail_i,
  model_i,
  latency_i
}
```

The pass gate is:

```text
pass_i = word_fit_i >= threshold
      AND fact_grounding_i >= threshold
      AND layering_i >= threshold
      AND guardrail_i == pass
```

Use simple deterministic proxies first. Do not introduce LLM-as-judge until deterministic gates are in place.

The learning tail adds another vector:

```text
P_j = extracted synthesis pattern j
C_j = source claim / conceptual chunk
E_j = embedding vector
H_j = hardening score
V_j = version/provenance metadata
```

Pattern eligibility:

```text
eligible(P_j) = anonymized(P_j)
             AND no_private_birth_data(P_j)
             AND rubric_source_sections_passed(P_j)
             AND guardrail_safe(P_j)
             AND usefulness_score(P_j) >= threshold
```

Future retrieval must obey this invariant:

```text
retrieved_pattern != deterministic_fact
```

Retrieved patterns can influence wording, analogical layering, and section structure. They must not override current chart data.

## Rubric Definition

Each section rubric should be shaped like this:

```ts
export interface SectionRubric {
  section_id: string;
  title: string;
  target_words: number;
  actual_words: number;
  word_count_fit: 'pass' | 'warn' | 'fail';
  word_count_ratio: number;
  deterministic_fact_count: number;
  deterministic_fact_gate: 'pass' | 'warn' | 'fail';
  integrated_layer_count: number;
  integrated_layering_gate: 'pass' | 'warn' | 'fail';
  guardrail_gate: 'pass' | 'fail';
  guardrail_violations: string[];
  model_requested: string;
  model_used: string;
  latency_ms: number;
}
```

Recommended default gates for all modes:

| Report Level | Min Facts/Section | Min Layers/Section | Word Fit | Learning Tail |
|---|---:|---:|---|---|
| L0 | mode-specific | mode-specific | 80-125% pass | required |
| L1 | 1 | 1 | 70-140% pass | optional |
| L2 | 2 | 2 | 70-135% pass | optional |
| L3 | 3 | 2 | 75-130% pass | required |
| L4 | 4 | 3 | 80-125% pass | required |
| L5 | 5 | 4 | 80-120% pass | required + human-review capable |

Recommended gates for `integrated-kundali-l0` override the generic defaults:

| Section | Target | Min Facts | Min Layers | Hard Guardrails |
|---|---:|---:|---:|---|
| opening | 450 | 1 | 2 | no prediction |
| convergence-map | 1600 | 6 | 4 | convergence needs 3 systems |
| vedic-foundation | 3800 | 12 | 1 | no inferred missing chart data |
| karmic-architecture | 1500 | 6 | 3 | symbolic karma, not literal proof |
| career-dharma | 1600 | 6 | 3 | no absolute job prescription |
| wealth | 1200 | 5 | 3 | no financial guarantees/advice |
| love-marriage | 1500 | 5 | 3 | no marriage inevitability/fixed spouse claims |
| health | 1100 | 4 | 2 | no diagnosis/treatment/disease forecast |
| family-lineage | 1050 | 4 | 2 | no childbirth/family event prediction |
| master-timeline | 2200 | 8 | 3 | exact dates separated from interpretation |
| remedies-practices | 1700 | 4 | 3 | no guaranteed remedy fixes |
| final-synthesis | 1300 | 5 | 4 | prediction framed as pattern-not-fate |

Word fit formula:

```ts
const ratio = actualWords / targetWords;
if (ratio >= 0.8 && ratio <= 1.25) return 'pass';
if (ratio >= 0.65 && ratio <= 1.45) return 'warn';
return 'fail';
```

Fact detection formula, first version:

```ts
Count explicit references to known deterministic systems and concrete values:
- Vedic, Lagna, house, planet, nakshatra, pada, dasha, antardasha, Sade Sati
- Human Design, HD, gate, channel, profile, authority, type, center
- Gene Keys, Life's Work, Evolution, Vocation, Pearl, Radiance, Purpose
- transit, panchanga, tithi, yoga, karana
```

Layering formula, first version:

```ts
integrated_layer_count = count distinct systems referenced in the section.
```

Guardrail formula, first version:

```ts
Fail if sensitive sections contain banned certainty phrases near sensitive domains.
Examples:
- wealth: "will become rich", "guaranteed", "investment advice", "definitely inherit"
- love-marriage: "will marry", "must divorce", "spouse will be", "inevitable"
- health: "diagnosis", "you have", "will develop", "treatment", "cure"
- family-lineage: "will have children", "infertile", "parent will die", "child will"
```

## Systems View

Current loop:

```text
mode prompt -> model output -> assembled report -> weak whole-reading audit
```

This creates a weak feedback loop: the system asks for grounded quality, but only verifies that enough deterministic engines were present. Model hallucination, overlong sections, missing integration, or unsafe certainty can pass.

Target loop:

```text
mode section contract -> model output -> deterministic section rubric -> source-pack evidence -> retry/human checkpoint if failed
```

The leverage point is the pass boundary. Measuring at the section boundary is cheaper and more useful than auditing only the final assembled report.

---

### Task 1: Add Section Rubric Types

**Files:**
- Modify: `packages/witness-pipeline/src/orchestrator/integrated.ts`
- Test: `packages/witness-pipeline/src/orchestrator/integrated.test.ts`

**Step 1: Write the failing test**

Add to `integrated.test.ts`:

```ts
it('returns per-pass rubric metadata', async () => {
  const llm = vi.fn().mockResolvedValue('Vedic Lagna and Human Design gate 34 are named here.');
  const orchestrator = new IntegratedReadingOrchestrator({ mode: mockMode, llm });

  const result = await orchestrator.run({
    subjectNames: ['A'],
    engineResultsBySubject: [mockEngineResults],
    consciousnessLevel: 4,
  });

  expect(result.passes[0].rubric).toMatchObject({
    section_id: 'alpha',
    title: 'Alpha',
    target_words: 100,
    actual_words: expect.any(Number),
    model_requested: 'tier-default',
    model_used: 'tier-default',
    latency_ms: expect.any(Number),
  });
});
```

**Step 2: Run test to verify it fails**

Run: `pnpm --filter @noesis/witness-pipeline test -- src/orchestrator/integrated.test.ts`

Expected: FAIL because `rubric` is missing on `PassResult`.

**Step 3: Write minimal implementation**

In `integrated.ts`, add:

```ts
export type RubricGate = 'pass' | 'warn' | 'fail';

export interface SectionRubric {
  section_id: string;
  title: string;
  target_words: number;
  actual_words: number;
  word_count_fit: RubricGate;
  word_count_ratio: number;
  deterministic_fact_count: number;
  deterministic_fact_gate: RubricGate;
  integrated_layer_count: number;
  integrated_layering_gate: RubricGate;
  guardrail_gate: 'pass' | 'fail';
  guardrail_violations: string[];
  model_requested: string;
  model_used: string;
  latency_ms: number;
}
```

Update `PassResult`:

```ts
export interface PassResult {
  id: string;
  title: string;
  output: string;
  rubric: SectionRubric;
}
```

Add a minimal `buildSectionRubric()` returning counts and timing.

**Step 4: Run test to verify it passes**

Run: `pnpm --filter @noesis/witness-pipeline test -- src/orchestrator/integrated.test.ts`

Expected: PASS.

**Step 5: Commit**

```bash
git add packages/witness-pipeline/src/orchestrator/integrated.ts packages/witness-pipeline/src/orchestrator/integrated.test.ts
git commit -m "feat: add section rubric metadata"
```

### Task 2: Extract Rubric Logic Into a Dedicated Audit Module

**Files:**
- Create: `packages/witness-pipeline/src/orchestrator/rubric.ts`
- Create: `packages/witness-pipeline/src/orchestrator/rubric.test.ts`
- Modify: `packages/witness-pipeline/src/orchestrator/integrated.ts`

**Step 1: Write failing tests**

Create `rubric.test.ts`:

```ts
import { describe, expect, it } from 'vitest';
import { auditSectionOutput } from './rubric.js';

describe('auditSectionOutput', () => {
  it('passes word count when output is within 80-125 percent of target', () => {
    const output = Array.from({ length: 90 }, (_, i) => `word${i}`).join(' ');
    const rubric = auditSectionOutput({
      sectionId: 'opening',
      title: 'Opening',
      targetWords: 100,
      output,
      modelRequested: 'tier-default',
      modelUsed: 'tier-default',
      latencyMs: 10,
    });

    expect(rubric.word_count_fit).toBe('pass');
    expect(rubric.actual_words).toBe(90);
  });

  it('counts deterministic fact references and integrated layers', () => {
    const output = 'Vedic Lagna, Vimshottari dasha, Human Design profile, Gene Keys Pearl, and transits converge.';
    const rubric = auditSectionOutput({
      sectionId: 'convergence-map',
      title: 'Part I',
      targetWords: 20,
      output,
      modelRequested: 'tier-default',
      modelUsed: 'tier-default',
      latencyMs: 10,
    });

    expect(rubric.deterministic_fact_count).toBeGreaterThanOrEqual(5);
    expect(rubric.integrated_layer_count).toBeGreaterThanOrEqual(4);
  });

  it('fails health guardrail on diagnostic language', () => {
    const rubric = auditSectionOutput({
      sectionId: 'health',
      title: 'Health',
      targetWords: 20,
      output: 'This is a diagnosis and treatment plan for disease.',
      modelRequested: 'tier-default',
      modelUsed: 'tier-default',
      latencyMs: 10,
    });

    expect(rubric.guardrail_gate).toBe('fail');
    expect(rubric.guardrail_violations.length).toBeGreaterThan(0);
  });
});
```

**Step 2: Run test to verify it fails**

Run: `pnpm --filter @noesis/witness-pipeline test -- src/orchestrator/rubric.test.ts`

Expected: FAIL because `rubric.ts` does not exist.

**Step 3: Implement the audit module**

Create `rubric.ts`:

```ts
import type { SectionRubric } from './integrated.js';

export interface AuditSectionInput {
  sectionId: string;
  title: string;
  targetWords: number;
  output: string;
  modelRequested: string;
  modelUsed: string;
  latencyMs: number;
}

const SYSTEM_PATTERNS = [
  /\b(Vedic|Lagna|house|planet|nakshatra|pada|dasha|antardasha|Sade Sati)\b/gi,
  /\b(Human Design|HD|gate|channel|profile|authority|type|center)\b/gi,
  /\b(Gene Keys|Life's Work|Evolution|Vocation|Pearl|Radiance|Purpose)\b/gi,
  /\b(transit|panchanga|tithi|yoga|karana)\b/gi,
  /\b(Vimshottari|mahadasha|pratyantardasha)\b/gi,
];

const LAYERS: Array<[string, RegExp]> = [
  ['vedic', /\b(Vedic|Lagna|nakshatra|house|planet|Rahu|Ketu)\b/i],
  ['vimshottari', /\b(Vimshottari|mahadasha|antardasha|dasha)\b/i],
  ['human-design', /\b(Human Design|HD|gate|channel|profile|authority|center)\b/i],
  ['gene-keys', /\b(Gene Keys|Life's Work|Evolution|Vocation|Pearl|Radiance|Purpose)\b/i],
  ['transits', /\b(transit|Sade Sati|Saturn|Jupiter)\b/i],
  ['panchanga', /\b(panchanga|tithi|karana|yoga)\b/i],
];

const GUARDRAILS: Record<string, RegExp[]> = {
  wealth: [/guarantee/i, /investment advice/i, /will become rich/i, /definitely inherit/i],
  'love-marriage': [/will marry/i, /inevitable/i, /must divorce/i, /spouse will be/i],
  health: [/diagnos/i, /treatment/i, /cure/i, /will develop/i, /you have .*disease/i],
  'family-lineage': [/will have children/i, /infertile/i, /parent will die/i, /child will/i],
};

export function auditSectionOutput(input: AuditSectionInput): SectionRubric {
  const words = input.output.trim().split(/\s+/).filter(Boolean).length;
  const ratio = input.targetWords > 0 ? words / input.targetWords : 0;
  const deterministicFactCount = SYSTEM_PATTERNS.reduce((sum, re) => {
    return sum + (input.output.match(re)?.length ?? 0);
  }, 0);
  const integratedLayerCount = LAYERS.filter(([, re]) => re.test(input.output)).length;
  const guardrailViolations = (GUARDRAILS[input.sectionId] ?? [])
    .filter((re) => re.test(input.output))
    .map((re) => re.source);

  return {
    section_id: input.sectionId,
    title: input.title,
    target_words: input.targetWords,
    actual_words: words,
    word_count_fit: wordCountGate(ratio),
    word_count_ratio: Number(ratio.toFixed(3)),
    deterministic_fact_count: deterministicFactCount,
    deterministic_fact_gate: deterministicFactCount >= 3 ? 'pass' : deterministicFactCount >= 1 ? 'warn' : 'fail',
    integrated_layer_count: integratedLayerCount,
    integrated_layering_gate: integratedLayerCount >= 3 ? 'pass' : integratedLayerCount >= 2 ? 'warn' : 'fail',
    guardrail_gate: guardrailViolations.length === 0 ? 'pass' : 'fail',
    guardrail_violations: guardrailViolations,
    model_requested: input.modelRequested,
    model_used: input.modelUsed,
    latency_ms: input.latencyMs,
  };
}

function wordCountGate(ratio: number): 'pass' | 'warn' | 'fail' {
  if (ratio >= 0.8 && ratio <= 1.25) return 'pass';
  if (ratio >= 0.65 && ratio <= 1.45) return 'warn';
  return 'fail';
}
```

**Step 4: Wire into orchestrator**

In `integrated.ts`, import `auditSectionOutput`. Around the LLM call:

```ts
const started = Date.now();
const output = await this.llm(system, prompt, { max_tokens: Math.round(max * 2) });
const latencyMs = Date.now() - started;
const modelRequested = pass.model ?? 'tier-default';
const rubric = auditSectionOutput({
  sectionId: pass.id,
  title: pass.title,
  targetWords: pass.target_words,
  output,
  modelRequested,
  modelUsed: modelRequested,
  latencyMs,
});
passOutputs.push({ id: pass.id, title: pass.title, output, rubric });
```

**Step 5: Run tests**

Run: `pnpm --filter @noesis/witness-pipeline test -- src/orchestrator/rubric.test.ts src/orchestrator/integrated.test.ts`

Expected: PASS.

**Step 6: Commit**

```bash
git add packages/witness-pipeline/src/orchestrator/rubric.ts packages/witness-pipeline/src/orchestrator/rubric.test.ts packages/witness-pipeline/src/orchestrator/integrated.ts packages/witness-pipeline/src/orchestrator/integrated.test.ts
git commit -m "feat: audit report sections"
```

### Task 3: Add Kundali-Specific Thresholds

**Files:**
- Modify: `packages/witness-pipeline/src/orchestrator/rubric.ts`
- Modify: `packages/witness-pipeline/src/orchestrator/rubric.test.ts`

**Step 1: Write failing test**

Add:

```ts
it('uses stricter deterministic thresholds for kundali timeline sections', () => {
  const rubric = auditSectionOutput({
    sectionId: 'master-timeline',
    title: 'Part IX',
    targetWords: 100,
    output: 'Vimshottari dasha and Sade Sati are mentioned once.',
    modelRequested: 'tier-default',
    modelUsed: 'tier-default',
    latencyMs: 1,
  });

  expect(rubric.deterministic_fact_gate).toBe('fail');
});
```

**Step 2: Run test to verify it fails**

Run: `pnpm --filter @noesis/witness-pipeline test -- src/orchestrator/rubric.test.ts`

Expected: FAIL because generic threshold may mark this as warn/pass.

**Step 3: Implement thresholds**

Add:

```ts
const SECTION_THRESHOLDS: Record<string, { minFacts: number; minLayers: number }> = {
  opening: { minFacts: 1, minLayers: 2 },
  'convergence-map': { minFacts: 6, minLayers: 4 },
  'vedic-foundation': { minFacts: 12, minLayers: 1 },
  'karmic-architecture': { minFacts: 6, minLayers: 3 },
  'career-dharma': { minFacts: 6, minLayers: 3 },
  wealth: { minFacts: 5, minLayers: 3 },
  'love-marriage': { minFacts: 5, minLayers: 3 },
  health: { minFacts: 4, minLayers: 2 },
  'family-lineage': { minFacts: 4, minLayers: 2 },
  'master-timeline': { minFacts: 8, minLayers: 3 },
  'remedies-practices': { minFacts: 4, minLayers: 3 },
  'final-synthesis': { minFacts: 5, minLayers: 4 },
};
```

Use these thresholds in `auditSectionOutput`.

**Step 4: Run tests**

Run: `pnpm --filter @noesis/witness-pipeline test -- src/orchestrator/rubric.test.ts`

Expected: PASS.

**Step 5: Commit**

```bash
git add packages/witness-pipeline/src/orchestrator/rubric.ts packages/witness-pipeline/src/orchestrator/rubric.test.ts
git commit -m "feat: add kundali rubric thresholds"
```

### Task 4: Add Rubric Matrix To Source Packs

**Files:**
- Modify: `packages/witness-pipeline/src/assets/factory.ts`
- Modify: `packages/witness-pipeline/src/assets/factory.test.ts`

**Step 1: Write failing test**

In `factory.test.ts`, add:

```ts
it('stores section rubric matrix in manifest when provided', async () => {
  const dir = mkdtempSync(join(tmpdir(), 'source-pack-rubric-'));
  try {
    const pack = await createSourcePack({
      personId: 'p1',
      readingMarkdown: 'reading text',
      engineResults: [makeEngine('panchanga'), makeEngine('vimshottari'), makeEngine('human-design')],
      outputDir: dir,
      sectionRubrics: [{
        section_id: 'wealth',
        title: 'Part V',
        target_words: 1200,
        actual_words: 1180,
        word_count_fit: 'pass',
        word_count_ratio: 0.983,
        deterministic_fact_count: 6,
        deterministic_fact_gate: 'pass',
        integrated_layer_count: 3,
        integrated_layering_gate: 'pass',
        guardrail_gate: 'pass',
        guardrail_violations: [],
        model_requested: 'tier-default',
        model_used: 'tier-default',
        latency_ms: 100,
      }],
    });

    expect(pack.manifest.quality.sections).toHaveLength(1);
    expect(pack.manifest.quality.sections[0].section_id).toBe('wealth');
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
});
```

**Step 2: Run test to verify it fails**

Run: `pnpm --filter @noesis/witness-pipeline test -- src/assets/factory.test.ts`

Expected: FAIL because `sectionRubrics` is not accepted.

**Step 3: Implement source pack support**

Add `sectionRubrics?: SectionRubric[]` to `SourcePackInput` and `sections?: SectionRubric[]` to `quality`.

**Step 4: Run tests**

Run: `pnpm --filter @noesis/witness-pipeline test -- src/assets/factory.test.ts`

Expected: PASS.

**Step 5: Commit**

```bash
git add packages/witness-pipeline/src/assets/factory.ts packages/witness-pipeline/src/assets/factory.test.ts
git commit -m "feat: include section rubrics in source packs"
```

### Task 5: Add Orchestrator E2E Coverage For Kundali Rubrics

**Files:**
- Modify: `packages/witness-pipeline/src/orchestrator/integrated.e2e.test.ts`

**Step 1: Write failing test**

Add to the existing kundali mode test:

```ts
const llm = vi.fn(async (_system, user) => {
  if (user.includes('wealth')) return 'Vedic 2nd house, Vimshottari dasha, Human Design profile, and Gene Keys Pearl are framed without guarantee.';
  return 'Vedic Lagna, Vimshottari dasha, Human Design gate, Gene Keys Pearl, and transit data are cited.';
});
const orchestrator = new IntegratedReadingOrchestrator({ mode, llm });
const result = await orchestrator.run({
  subjectNames: ['KundaliMode'],
  engineResultsBySubject: [mockEngines],
  consciousnessLevel: 4,
});

expect(result.passes).toHaveLength(12);
expect(result.passes.every((p) => p.rubric)).toBe(true);
expect(result.passes.find((p) => p.id === 'wealth')?.rubric.guardrail_gate).toBe('pass');
```

**Step 2: Run test to verify it fails if Task 1 not done**

Run: `pnpm --filter @noesis/witness-pipeline test -- src/orchestrator/integrated.e2e.test.ts`

Expected: PASS if previous tasks are complete; otherwise FAIL on missing rubric.

**Step 3: Adjust only if needed**

If thresholds make the test brittle, add enough deterministic terms to the stub output. Do not lower thresholds to satisfy a weak test.

**Step 4: Commit**

```bash
git add packages/witness-pipeline/src/orchestrator/integrated.e2e.test.ts
git commit -m "test: cover kundali section rubrics"
```

### Task 6: Add SDK Type Compatibility

**Files:**
- Modify: `packages/noesis-sdk-ts/src/premium-assets.ts`
- Modify: `packages/noesis-sdk-ts/src/index.premium-contract.test.ts`

**Step 1: Write failing test**

Add:

```ts
it('surfaces orchestrator section rubrics for local premium generation', async () => {
  const mode = resolveModeDoc('kundali-l0');
  expect(mode.frontmatter.pass_plan[0].target_words).toBeGreaterThan(0);
});
```

If SDK types already compile from witness-pipeline exports, this can remain a lightweight compatibility test.

**Step 2: Run test**

Run: `pnpm --filter @noesis/sdk test -- src/index.premium-contract.test.ts`

Expected: PASS after witness-pipeline type exports compile.

**Step 3: Build type output**

Run: `pnpm --filter @noesis/witness-pipeline build && pnpm --filter @noesis/sdk typecheck`

Expected: PASS.

**Step 4: Commit**

```bash
git add packages/noesis-sdk-ts/src/premium-assets.ts packages/noesis-sdk-ts/src/index.premium-contract.test.ts
git commit -m "test: keep sdk rubric compatibility"
```

### Task 7: Add Rust API Rubric Parity Shape

**Files:**
- Modify: `crates/noesis-api/src/handlers/assets.rs`
- Modify: `crates/noesis-api/tests/assets_generate_contract_test.rs`

**Step 1: Write failing test**

Extend `assets_generate_supports_integrated_kundali_l0_mode`:

```rust
let sp = &json["source_pack"];
let sections = sp["quality"]["sections"].as_array().expect("sections rubric matrix");
assert_eq!(sections.len(), 12);
assert!(sections[0].get("target_words").is_some());
assert!(sections[0].get("actual_words").is_some());
assert!(sections[0].get("model_used").is_some());
assert!(sections[0].get("latency_ms").is_some());
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p noesis-api --test assets_generate_contract_test assets_generate_supports_integrated_kundali_l0_mode`

Expected: FAIL because server source pack has no `quality.sections`.

**Step 3: Implement minimal parity**

In `assets.rs`, create a small `build_section_rubrics(&passes)` for server-only deterministic seed rendering. Use `model_used: "server-seed-renderer"` and `latency_ms: 0` until Rust LLM orchestration exists.

**Step 4: Run test**

Run: `cargo test -p noesis-api --test assets_generate_contract_test assets_generate_supports_integrated_kundali_l0_mode`

Expected: PASS.

**Step 5: Commit**

```bash
git add crates/noesis-api/src/handlers/assets.rs crates/noesis-api/tests/assets_generate_contract_test.rs
git commit -m "feat: expose server section rubric matrix"
```

### Task 8: Add Report Intake Schema

**Files:**
- Create: `packages/witness-pipeline/src/intake/types.ts`
- Create: `packages/witness-pipeline/src/intake/types.test.ts`
- Modify: `packages/witness-pipeline/src/index.ts`

**Step 1: Write failing test**

Create `types.test.ts`:

```ts
import { describe, expect, it } from 'vitest';
import { isCompleteReportRequest } from './types.js';

describe('isCompleteReportRequest', () => {
  it('requires normalized location for every subject', () => {
    const complete = isCompleteReportRequest({
      report_level: 'L3',
      report_mode: 'integrated-reading',
      subjects: [{
        role: 'primary',
        name: 'A',
        gender: 'female',
        birth_date: '1990-01-01',
        birth_time: '12:00',
        birth_time_confidence: 'exact',
        birth_location_query: 'Bangalore, India',
        normalized_location: {
          display_name: 'Bengaluru, Karnataka, India',
          latitude: 12.9716,
          longitude: 77.5946,
          timezone: 'Asia/Kolkata',
          provider: 'manual',
          confidence: 'selected',
        },
      }],
      output: { format: 'markdown', include_rubric: true, include_pattern_extraction: true },
    });

    expect(complete).toBe(true);
  });
});
```

**Step 2: Run test to verify it fails**

Run: `pnpm --filter @noesis/witness-pipeline test -- src/intake/types.test.ts`

Expected: FAIL because `intake/types.ts` does not exist.

**Step 3: Implement types**

Create `types.ts`:

```ts
import type { ReportLevel } from '../modes/types.js';

export type ReportMode = 'kundali' | 'birth-blueprint' | 'integrated-reading' | 'synastry' | string;
export type Gender = 'female' | 'male' | 'nonbinary' | 'other' | 'prefer_not_to_say' | 'unknown';
export type ExternalChartSex = 'female' | 'male' | 'unknown';
export type SubjectRole = 'primary' | 'partner' | 'family_member' | 'friend' | 'business_partner' | 'custom';

export interface NormalizedLocation {
  display_name: string;
  latitude: number;
  longitude: number;
  timezone: string;
  provider: 'manual' | 'nominatim' | 'google-places' | 'mapbox' | 'geonames';
  confidence: 'exact' | 'selected' | 'ambiguous' | 'manual';
}

export interface ReportSubjectInput {
  role: SubjectRole;
  relationship_label?: string;
  name: string;
  gender?: Gender;
  sex_for_external_chart_source?: ExternalChartSex;
  birth_date: string;
  birth_time?: string;
  birth_time_confidence: 'exact' | 'approximate' | 'unknown';
  birth_location_query: string;
  normalized_location?: NormalizedLocation;
}

export interface ReportGenerationRequest {
  report_level: ReportLevel;
  report_mode: ReportMode;
  subjects: ReportSubjectInput[];
  relationship_context?: {
    type: 'family' | 'friends' | 'business-partners' | 'unmarried-partners' | 'married-partners' | 'custom';
    mapping_goal: string;
    sensitivity_level: 'low' | 'medium' | 'high';
  };
  output: { format: 'markdown' | 'docx' | 'pdf' | 'source-pack'; include_rubric: boolean; include_pattern_extraction: boolean };
}

export function isCompleteReportRequest(request: ReportGenerationRequest): boolean {
  return request.subjects.length > 0 && request.subjects.every((subject) => Boolean(subject.normalized_location));
}
```

**Step 4: Export intake types**

In `packages/witness-pipeline/src/index.ts`:

```ts
export * from './intake/types.js';
```

**Step 5: Run tests**

Run: `pnpm --filter @noesis/witness-pipeline test -- src/intake/types.test.ts`

Expected: PASS.

**Step 6: Commit**

```bash
git add packages/witness-pipeline/src/intake/types.ts packages/witness-pipeline/src/intake/types.test.ts packages/witness-pipeline/src/index.ts
git commit -m "feat: add report intake schema"
```

### Task 9: Add Location Normalization Interface

**Files:**
- Create: `packages/witness-pipeline/src/intake/location.ts`
- Create: `packages/witness-pipeline/src/intake/location.test.ts`

**Step 1: Write failing test**

Create `location.test.ts`:

```ts
import { describe, expect, it } from 'vitest';
import { normalizeManualLocation } from './location.js';

describe('normalizeManualLocation', () => {
  it('normalizes manually supplied coordinates and timezone', () => {
    const location = normalizeManualLocation({
      displayName: 'Bengaluru, Karnataka, India',
      latitude: '12.9716',
      longitude: '77.5946',
      timezone: 'Asia/Kolkata',
    });

    expect(location).toEqual({
      display_name: 'Bengaluru, Karnataka, India',
      latitude: 12.9716,
      longitude: 77.5946,
      timezone: 'Asia/Kolkata',
      provider: 'manual',
      confidence: 'manual',
    });
  });
});
```

**Step 2: Run test to verify it fails**

Run: `pnpm --filter @noesis/witness-pipeline test -- src/intake/location.test.ts`

Expected: FAIL because `location.ts` does not exist.

**Step 3: Implement manual normalization**

Create `location.ts`:

```ts
import type { NormalizedLocation } from './types.js';

export interface ManualLocationInput {
  displayName: string;
  latitude: string | number;
  longitude: string | number;
  timezone: string;
}

export function normalizeManualLocation(input: ManualLocationInput): NormalizedLocation {
  const latitude = Number(input.latitude);
  const longitude = Number(input.longitude);
  if (!Number.isFinite(latitude) || latitude < -90 || latitude > 90) throw new Error('Invalid latitude');
  if (!Number.isFinite(longitude) || longitude < -180 || longitude > 180) throw new Error('Invalid longitude');
  if (!input.timezone.trim()) throw new Error('Timezone is required');
  return {
    display_name: input.displayName.trim(),
    latitude,
    longitude,
    timezone: input.timezone.trim(),
    provider: 'manual',
    confidence: 'manual',
  };
}
```

**Step 4: Run tests**

Run: `pnpm --filter @noesis/witness-pipeline test -- src/intake/location.test.ts`

Expected: PASS.

**Step 5: Commit**

```bash
git add packages/witness-pipeline/src/intake/location.ts packages/witness-pipeline/src/intake/location.test.ts
git commit -m "feat: add manual location normalization"
```

### Task 10: Add Location Picker Question Contract

**Files:**
- Create: `packages/witness-pipeline/src/intake/questions.ts`
- Create: `packages/witness-pipeline/src/intake/questions.test.ts`

**Step 1: Write failing test**

Create `questions.test.ts`:

```ts
import { describe, expect, it } from 'vitest';
import { buildReportIntakeQuestions } from './questions.js';

describe('buildReportIntakeQuestions', () => {
  it('includes gender and location confirmation questions', () => {
    const questions = buildReportIntakeQuestions({ subjectCount: 1, relationship: false });
    const headers = questions.map((q) => q.header);

    expect(headers).toContain('Gender');
    expect(headers).toContain('Birthplace');
    expect(headers).toContain('Confirm Location');
  });
});
```

**Step 2: Run test to verify it fails**

Run: `pnpm --filter @noesis/witness-pipeline test -- src/intake/questions.test.ts`

Expected: FAIL because `questions.ts` does not exist.

**Step 3: Implement question builder**

Create `questions.ts`:

```ts
export interface IntakeQuestion {
  header: string;
  question: string;
  options?: Array<{ label: string; description: string }>;
  multiple?: boolean;
}

export function buildReportIntakeQuestions(_opts: { subjectCount: number; relationship: boolean }): IntakeQuestion[] {
  return [
    {
      header: 'Report Type',
      question: 'What kind of report are we generating?',
      options: [
        { label: 'Individual', description: 'One person report' },
        { label: 'Synastry', description: 'Two or more people relationship mapping' },
      ],
    },
    {
      header: 'Gender',
      question: 'What gender should the report use for reader-facing language?',
      options: [
        { label: 'Female', description: 'Use female reader-facing language where relevant' },
        { label: 'Male', description: 'Use male reader-facing language where relevant' },
        { label: 'Nonbinary', description: 'Use neutral reader-facing language' },
        { label: 'Prefer not to say', description: 'Avoid gendered language' },
      ],
    },
    { header: 'Birthplace', question: 'What birthplace should we normalize into latitude, longitude, and timezone?' },
    {
      header: 'Confirm Location',
      question: 'Confirm the selected place, latitude, longitude, and timezone before report generation.',
      options: [
        { label: 'Use selected', description: 'Accept normalized location' },
        { label: 'Pick another', description: 'Choose from alternate geocoding results' },
        { label: 'Enter manually', description: 'Provide lat/long/timezone directly' },
      ],
    },
  ];
}
```

**Step 4: Run tests**

Run: `pnpm --filter @noesis/witness-pipeline test -- src/intake/questions.test.ts`

Expected: PASS.

**Step 5: Commit**

```bash
git add packages/witness-pipeline/src/intake/questions.ts packages/witness-pipeline/src/intake/questions.test.ts
git commit -m "feat: add report intake questions"
```

### Task 11: Add Geocoding Provider Design Note

**Files:**
- Create: `docs/plans/2026-07-04-report-location-normalization.md`

**Step 1: Write design note**

Create a short design note with:

- Existing precedent: `tools/humdes-extractor/humdes_to_selemene.py` uses cached/rate-limited Nominatim.
- CLI/local default: cached Nominatim picker with manual fallback.
- Web default: Places/Mapbox picker with manual fallback.
- Batch default: require pre-normalized coordinates, optionally Nominatim cache.
- Privacy mode: manual coordinates only, no third-party geocoding.
- Timezone must be confirmed as IANA timezone.

**Step 2: Verify file exists**

Run: `test -f docs/plans/2026-07-04-report-location-normalization.md`

Expected: exit 0.

**Step 3: Commit**

```bash
git add docs/plans/2026-07-04-report-location-normalization.md
git commit -m "docs: plan report location normalization"
```

### Task 12: Generalize Mode Metadata For L0-L5 Report Levels

**Files:**
- Modify: `packages/witness-pipeline/src/modes/types.ts`
- Modify: `packages/witness-pipeline/src/modes/parser.ts`
- Modify: `packages/witness-pipeline/src/modes/parser.test.ts`

**Step 1: Write failing test**

Add to `parser.test.ts`:

```ts
it('parses optional report level metadata', () => {
  const doc = parseModeDocument(`---
mode: test-report
report_level: L3
subject_count: { min: 1, max: 1 }
roles: [subject]
target_words: { min: 100, max: 200 }
architecture: linear
pass_plan:
  - id: synthesis
    title: Synthesis
    target_words: 100
    template: synthesis-pass
engine_overlay_weights: { panchanga: 1 }
house_overlay: [1]
bridge_mandates: ["Use facts"]
svg_topology: dyad-arc
---

## synthesis-pass
Write.
`, 'test-report.md');

  expect(doc.frontmatter.report_level).toBe('L3');
});
```

**Step 2: Run test to verify it fails**

Run: `pnpm --filter @noesis/witness-pipeline test -- src/modes/parser.test.ts`

Expected: FAIL because `report_level` is not typed.

**Step 3: Implement minimal type support**

Add to `modes/types.ts`:

```ts
export type ReportLevel = 'L0' | 'L1' | 'L2' | 'L3' | 'L4' | 'L5';

export interface ModeConfig {
  // existing fields...
  report_level?: ReportLevel;
}
```

In `parser.ts`, validate optional `report_level` only if present:

```ts
const VALID_REPORT_LEVELS = ['L0', 'L1', 'L2', 'L3', 'L4', 'L5'];
if ('report_level' in obj && !VALID_REPORT_LEVELS.includes(obj.report_level as string)) {
  throw new Error(`Mode doc ${path}: invalid report_level '${obj.report_level}'`);
}
```

**Step 4: Add report levels to existing modes**

Set:

- `packages/witness-pipeline/modes/integrated-kundali-l0.md` → `report_level: L0`
- `packages/witness-pipeline/modes/birth-blueprint.md` → `report_level: L1`
- `packages/witness-pipeline/modes/integrated-reading.md` → `report_level: L3`

Do not alter legacy `partner-synastry.md` in this task because it uses a different frontmatter shape.

**Step 5: Run tests**

Run: `pnpm --filter @noesis/witness-pipeline test -- src/modes/parser.test.ts src/orchestrator/integrated.e2e.test.ts`

Expected: PASS.

**Step 6: Commit**

```bash
git add packages/witness-pipeline/src/modes/types.ts packages/witness-pipeline/src/modes/parser.ts packages/witness-pipeline/src/modes/parser.test.ts packages/witness-pipeline/modes/integrated-kundali-l0.md packages/witness-pipeline/modes/birth-blueprint.md packages/witness-pipeline/modes/integrated-reading.md
git commit -m "feat: add report level metadata"
```

### Task 13: Add Post-Report Pattern Extraction Types

**Files:**
- Create: `packages/witness-pipeline/src/patterns/types.ts`
- Create: `packages/witness-pipeline/src/patterns/extractor.ts`
- Create: `packages/witness-pipeline/src/patterns/extractor.test.ts`
- Modify: `packages/witness-pipeline/src/index.ts`

**Step 1: Write failing test**

Create `extractor.test.ts`:

```ts
import { describe, expect, it } from 'vitest';
import { extractReportPatterns } from './extractor.js';

describe('extractReportPatterns', () => {
  it('extracts anonymized reusable patterns from passing sections', () => {
    const patterns = extractReportPatterns({
      mode: 'integrated-reading',
      reportLevel: 'L3',
      subjectNames: ['Private Name'],
      passes: [
        {
          id: 'synthesis',
          title: 'Synthesis',
          output: 'Private Name has Vedic Saturn pressure and Human Design Projector pacing, forming a pattern of delayed recognition.',
          rubric: {
            section_id: 'synthesis', title: 'Synthesis', target_words: 20, actual_words: 18,
            word_count_fit: 'pass', word_count_ratio: 0.9,
            deterministic_fact_count: 4, deterministic_fact_gate: 'pass',
            integrated_layer_count: 2, integrated_layering_gate: 'pass',
            guardrail_gate: 'pass', guardrail_violations: [],
            model_requested: 'tier-default', model_used: 'tier-default', latency_ms: 1,
          },
        },
      ],
    });

    expect(patterns).toHaveLength(1);
    expect(patterns[0].text).not.toContain('Private Name');
    expect(patterns[0].metadata.mode).toBe('integrated-reading');
    expect(patterns[0].metadata.report_level).toBe('L3');
  });
});
```

**Step 2: Run test to verify it fails**

Run: `pnpm --filter @noesis/witness-pipeline test -- src/patterns/extractor.test.ts`

Expected: FAIL because the module does not exist.

**Step 3: Implement pattern types**

Create `types.ts`:

```ts
import type { ReportLevel } from '../modes/types.js';

export type PatternKind = 'convergence' | 'tension' | 'guardrail-safe-framing' | 'section-structure' | 'remedy-pattern';

export interface ExtractedPattern {
  id: string;
  text: string;
  kind: PatternKind;
  source_section_id: string;
  source_rubric_score: number;
  metadata: {
    mode: string;
    report_level: ReportLevel;
    systems: string[];
    source: 'post-report-extraction';
    version: string;
  };
}
```

**Step 4: Implement extractor**

Create `extractor.ts`:

```ts
import type { PassResult } from '../orchestrator/integrated.js';
import type { ReportLevel } from '../modes/types.js';
import type { ExtractedPattern } from './types.js';

export interface ExtractReportPatternsInput {
  mode: string;
  reportLevel: ReportLevel;
  subjectNames: string[];
  passes: PassResult[];
}

export function extractReportPatterns(input: ExtractReportPatternsInput): ExtractedPattern[] {
  return input.passes
    .filter((p) => p.rubric.guardrail_gate === 'pass')
    .filter((p) => p.rubric.integrated_layering_gate === 'pass')
    .map((p) => ({
      id: `${input.mode}:${p.id}:v1`,
      text: anonymize(extractPatternText(p.output), input.subjectNames),
      kind: inferKind(p.id),
      source_section_id: p.id,
      source_rubric_score: scoreRubric(p.rubric),
      metadata: {
        mode: input.mode,
        report_level: input.reportLevel,
        systems: inferSystems(p.output),
        source: 'post-report-extraction',
        version: '1',
      },
    }))
    .filter((p) => p.text.length >= 40);
}
```

Keep `anonymize`, `extractPatternText`, `inferKind`, `scoreRubric`, and `inferSystems` small and deterministic.

**Step 5: Export module**

In `packages/witness-pipeline/src/index.ts`:

```ts
export * from './patterns/types.js';
export * from './patterns/extractor.js';
```

**Step 6: Run tests**

Run: `pnpm --filter @noesis/witness-pipeline test -- src/patterns/extractor.test.ts`

Expected: PASS.

**Step 7: Commit**

```bash
git add packages/witness-pipeline/src/patterns/types.ts packages/witness-pipeline/src/patterns/extractor.ts packages/witness-pipeline/src/patterns/extractor.test.ts packages/witness-pipeline/src/index.ts
git commit -m "feat: extract post-report patterns"
```

### Task 14: Attach Pattern Extraction To Orchestrator Output

**Files:**
- Modify: `packages/witness-pipeline/src/orchestrator/integrated.ts`
- Modify: `packages/witness-pipeline/src/orchestrator/integrated.test.ts`

**Step 1: Write failing test**

In `integrated.test.ts`, add:

```ts
it('extracts pattern candidates after report assembly', async () => {
  const llm = vi.fn().mockResolvedValue('Vedic Saturn and Human Design Projector timing create delayed recognition pattern.');
  const orchestrator = new IntegratedReadingOrchestrator({ mode: mockMode, llm });
  const result = await orchestrator.run({
    subjectNames: ['A'],
    engineResultsBySubject: [mockEngineResults],
    consciousnessLevel: 4,
  });

  expect(result.patterns).toEqual(expect.any(Array));
});
```

**Step 2: Run test to verify it fails**

Run: `pnpm --filter @noesis/witness-pipeline test -- src/orchestrator/integrated.test.ts`

Expected: FAIL because `patterns` is missing.

**Step 3: Implement output field**

In `OrchestratorOutput`, add:

```ts
patterns: ExtractedPattern[];
```

At the end of `run()`:

```ts
const patterns = extractReportPatterns({
  mode: this.mode.frontmatter.mode,
  reportLevel: this.mode.frontmatter.report_level ?? 'L3',
  subjectNames: input.subjectNames,
  passes: passOutputs,
});
```

Return `patterns` with the output.

**Step 4: Run tests**

Run: `pnpm --filter @noesis/witness-pipeline test -- src/orchestrator/integrated.test.ts src/orchestrator/integrated.e2e.test.ts`

Expected: PASS.

**Step 5: Commit**

```bash
git add packages/witness-pipeline/src/orchestrator/integrated.ts packages/witness-pipeline/src/orchestrator/integrated.test.ts
git commit -m "feat: attach pattern candidates to reports"
```

### Task 15: Add Vectorize Writer Interface With No-Op Default

**Files:**
- Create: `packages/witness-pipeline/src/patterns/vector-store.ts`
- Create: `packages/witness-pipeline/src/patterns/vector-store.test.ts`

**Step 1: Write failing test**

Create `vector-store.test.ts`:

```ts
import { describe, expect, it } from 'vitest';
import { NoopPatternVectorStore } from './vector-store.js';

describe('NoopPatternVectorStore', () => {
  it('accepts patterns without external side effects', async () => {
    const store = new NoopPatternVectorStore();
    await expect(store.upsertPatterns([])).resolves.toEqual({ upserted: 0, skipped: 0 });
  });
});
```

**Step 2: Run test to verify it fails**

Run: `pnpm --filter @noesis/witness-pipeline test -- src/patterns/vector-store.test.ts`

Expected: FAIL because module does not exist.

**Step 3: Implement interface**

Create `vector-store.ts`:

```ts
import type { ExtractedPattern } from './types.js';

export interface PatternVectorStoreResult {
  upserted: number;
  skipped: number;
}

export interface PatternVectorStore {
  upsertPatterns(patterns: ExtractedPattern[]): Promise<PatternVectorStoreResult>;
}

export class NoopPatternVectorStore implements PatternVectorStore {
  async upsertPatterns(patterns: ExtractedPattern[]): Promise<PatternVectorStoreResult> {
    return { upserted: 0, skipped: patterns.length };
  }
}
```

**Step 4: Run test**

Run: `pnpm --filter @noesis/witness-pipeline test -- src/patterns/vector-store.test.ts`

Expected: PASS.

**Step 5: Commit**

```bash
git add packages/witness-pipeline/src/patterns/vector-store.ts packages/witness-pipeline/src/patterns/vector-store.test.ts
git commit -m "feat: add pattern vector store interface"
```

### Task 16: Add Cloudflare Vectorize Worker Plan Surface

**Files:**
- Create: `docs/plans/2026-07-04-cloudflare-vectorize-pattern-memory.md`

**Step 1: Write plan document**

Create a separate Cloudflare implementation plan rather than mixing bindings into the library task. It must specify:

- Worker binding `REPORT_PATTERNS` for Vectorize.
- AI binding for embeddings.
- R2 or D1/Postgres canonical storage for full pattern text.
- Metadata indexes: `mode`, `report_level`, `kind`, `version`.
- No private birth data in metadata or vector text.

Cloudflare `wrangler.jsonc` target shape:

```jsonc
{
  "vectorize": [
    {
      "binding": "REPORT_PATTERNS",
      "index_name": "selemene-report-patterns"
    }
  ],
  "ai": {
    "binding": "AI"
  }
}
```

**Step 2: Verify file exists**

Run: `test -f docs/plans/2026-07-04-cloudflare-vectorize-pattern-memory.md`

Expected: exit 0.

**Step 3: Commit**

```bash
git add docs/plans/2026-07-04-cloudflare-vectorize-pattern-memory.md
git commit -m "docs: plan vectorize pattern memory"
```

### Task 17: Add Source Pack Learning Provenance

**Files:**
- Modify: `packages/witness-pipeline/src/assets/factory.ts`
- Modify: `packages/witness-pipeline/src/assets/factory.test.ts`

**Step 1: Write failing test**

Add to `factory.test.ts`:

```ts
it('stores pattern learning provenance when provided', async () => {
  const dir = mkdtempSync(join(tmpdir(), 'source-pack-learning-'));
  try {
    const pack = await createSourcePack({
      personId: 'p1',
      readingMarkdown: 'reading text',
      engineResults: [makeEngine('panchanga'), makeEngine('vimshottari'), makeEngine('human-design')],
      outputDir: dir,
      patternLearning: { extracted: 2, upserted: 1, skipped: 1 },
    });

    expect(pack.manifest.quality.pattern_learning).toEqual({ extracted: 2, upserted: 1, skipped: 1 });
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
});
```

**Step 2: Run test to verify it fails**

Run: `pnpm --filter @noesis/witness-pipeline test -- src/assets/factory.test.ts`

Expected: FAIL because `patternLearning` is not supported.

**Step 3: Implement manifest fields**

Add optional input:

```ts
patternLearning?: { extracted: number; upserted: number; skipped: number };
```

Add to manifest quality as `pattern_learning` when present.

**Step 4: Run tests**

Run: `pnpm --filter @noesis/witness-pipeline test -- src/assets/factory.test.ts`

Expected: PASS.

**Step 5: Commit**

```bash
git add packages/witness-pipeline/src/assets/factory.ts packages/witness-pipeline/src/assets/factory.test.ts
git commit -m "feat: record pattern learning provenance"
```

### Task 18: Retrieval Safety Contract

**Files:**
- Create: `packages/witness-pipeline/src/patterns/retrieval.ts`
- Create: `packages/witness-pipeline/src/patterns/retrieval.test.ts`

**Step 1: Write failing test**

Create `retrieval.test.ts`:

```ts
import { describe, expect, it } from 'vitest';
import { renderRetrievedPatternsForPrompt } from './retrieval.js';

describe('renderRetrievedPatternsForPrompt', () => {
  it('labels retrieved patterns as non-deterministic context', () => {
    const rendered = renderRetrievedPatternsForPrompt([
      { text: 'Saturn pressure plus Projector pacing can be framed as delayed recognition.' },
    ]);

    expect(rendered).toContain('Retrieved synthesis patterns are not deterministic facts');
    expect(rendered).toContain('delayed recognition');
  });
});
```

**Step 2: Run test to verify it fails**

Run: `pnpm --filter @noesis/witness-pipeline test -- src/patterns/retrieval.test.ts`

Expected: FAIL because module does not exist.

**Step 3: Implement retrieval renderer**

Create `retrieval.ts`:

```ts
export interface RetrievedPattern {
  text: string;
  score?: number;
  metadata?: Record<string, unknown>;
}

export function renderRetrievedPatternsForPrompt(patterns: RetrievedPattern[]): string {
  if (patterns.length === 0) return '';
  return [
    'Retrieved synthesis patterns are not deterministic facts. Use them only for analogy, wording, and layering. Current chart data overrides retrieved context.',
    ...patterns.map((p, i) => `${i + 1}. ${p.text}`),
  ].join('\n');
}
```

**Step 4: Run tests**

Run: `pnpm --filter @noesis/witness-pipeline test -- src/patterns/retrieval.test.ts`

Expected: PASS.

**Step 5: Commit**

```bash
git add packages/witness-pipeline/src/patterns/retrieval.ts packages/witness-pipeline/src/patterns/retrieval.test.ts
git commit -m "feat: label retrieved pattern context"
```

### Task 19: Full Verification

**Files:**
- No edits.

**Step 1: Run witness-pipeline tests**

Run: `pnpm --filter @noesis/witness-pipeline test`

Expected: all tests pass.

**Step 2: Run SDK tests**

Run: `pnpm --filter @noesis/sdk test`

Expected: all tests pass.

**Step 3: Run API contract tests**

Run: `cargo test -p noesis-api --test assets_generate_contract_test`

Expected: all tests pass.

**Step 4: Run typechecks**

Run: `pnpm --filter @noesis/witness-pipeline typecheck && pnpm --filter @noesis/sdk typecheck`

Expected: both pass.

**Step 5: Verify no private data appears in pattern tests**

Run: `pnpm --filter @noesis/witness-pipeline test -- src/patterns/extractor.test.ts src/patterns/retrieval.test.ts`

Expected: all tests pass, including anonymization tests.

**Step 6: Commit if any verification-only fixes were needed**

```bash
git add <changed-files>
git commit -m "fix: stabilize kundali rubric verification"
```

## Implementation Notes

- Keep deterministic rubric checks simple first. LLM-as-judge can come later.
- Do not write private reading text directly to Vectorize. Extracted patterns must be anonymized and provenance-scored.
- Vectorize stores embeddings and compact metadata; canonical pattern text should live in R2/D1/Postgres so it can be audited, deleted, and versioned.
- Retrieved patterns are synthesis aids, not facts. The prompt renderer must label them as non-deterministic context.
- Do not block generation on `warn` in the first pass. Block only hard guardrail `fail` if/when retry logic is added.
- `model_used` is only accurate if the LLM adapter returns it. Until then use `pass.model ?? 'tier-default'` and document that it means requested model.
- Avoid duplicate embedded mode docs long term; the SDK currently embeds mode strings for built-in resolution, but mode docs should eventually be packaged or loaded from `@noesis/witness-pipeline`.

## Success Criteria

- Report intake schema captures gender, external chart sex compatibility, birth time confidence, and relationship role per subject.
- Every subject must confirm a normalized birthplace with display name, latitude, longitude, and IANA timezone before generation.
- Location normalization supports manual coordinates first, then provider-backed picker workflows without silent best-guess generation.
- Every pass in every parsed report mode can return a `rubric` object.
- Mode metadata can distinguish report level `L0` through `L5` without overloading `consciousness_level`.
- Source pack manifests include `quality.sections` for section-level evidence.
- Sensitive sections fail guardrails on unsafe certainty language.
- Word-count fit is measured deterministically.
- Model and latency metadata exist per section.
- Finished reports extract anonymized pattern candidates.
- Pattern learning provenance is recorded in source packs.
- Vectorize integration has a binding-safe plan and no private birth data in vector text.
- Retrieved patterns are explicitly labeled as non-deterministic context.
- Existing `/witness/interpret` contract remains unchanged.
