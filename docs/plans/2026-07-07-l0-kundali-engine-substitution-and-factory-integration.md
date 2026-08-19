# L0 Integrated Kundali Engine Substitution & Factory Integration Implementation Plan

> **For Codex:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Fix the `integrated-kundali-l0` pipeline so deterministic engine results are substituted into the generated narrative, placeholder text is caught by verification gates, and the final HTML/PDF render lands in the canonical premium-assets factory structure using brand-docs-final tokens.

**Architecture:** Add an `engineResults` prompt injection path in the orchestrator, a placeholder-detection gate in the rubric, a brand-aware report renderer module outside the source-pack factory, and a migration that relocates the orphan `new-l0-flow` folder into the factory's `local/` output directory.

**Tech Stack:** TypeScript, Vitest, Node.js, js-yaml, Playwright (for PDF), Git worktrees, GitHub Issues.

---

## Discovery & Constraints

- `packages/witness-pipeline/src/orchestrator/integrated.ts` renders pass prompts but never injects `engineResults`.
- `packages/witness-pipeline/src/orchestrator/rubric.ts` counts astrology keywords as "facts" and computes `chart_fidelity_score` but does not gate on it.
- `packages/witness-pipeline/src/assets/factory.ts` explicitly excludes HTML/PDF rendering.
- The parent `manifest.json` outputs to `723/witness-agents-archive/.premium-assets-witness-{person}/{person}/` with subfolders `source-pack`, `local`, `audio`, `video`, `reports`, `slide-decks`, `quiz`, `flashcards`, `mind-map`.
- `new-l0-flow/` is not referenced by the factory manifest.
- `brand-docs-final/` defines Panchang/Satoshi/SF Mono and color tokens but has no CSS/PDF template.

## Agent Ownership Model (Swarm Architect)

| Role | Agent | Concerns |
|---|---|---|
| Orchestrator engineer | Codex / Claude | `integrated.ts`, `rubric.ts`, prompt wiring, gates |
| Assets engineer | Codex / Claude | `factory.ts`, renderer module, brand loader, PDF |
| Validation engineer | Gemini / Claude | tests, end-to-end smoke, regression checks |
| Integration / GitHub | Human or orchestrator | worktree, PR, issue sync, migration |

## Phase Map

| Phase | Goal | Waves | Est. Duration |
|---|---|---|---|
| **Phase 1** | Inject engine results into LLM prompts and detect placeholders | 3 | 2 days |
| **Phase 2** | Harden verification: placeholder gate + fidelity gate | 2 | 1 day |
| **Phase 3** | Build brand-aware renderer and PDF output | 3 | 3 days |
| **Phase 4** | Migrate orphan artifact, document, closeout | 2 | 1 day |

---

## Phase 1: Engine Result Injection

### Wave 1.1: Serialize Engine Results for Prompts

#### Task 1: Create `formatEngineResultsForPrompt`

**Files:**
- Create: `packages/witness-pipeline/src/orchestrator/engine-formatter.ts`
- Test: `packages/witness-pipeline/src/orchestrator/engine-formatter.test.ts`

**Step 1: Write the failing test**

```typescript
import { describe, it, expect } from 'vitest';
import { formatEngineResultsForPrompt } from './engine-formatter.js';

const sampleEngines = [
  {
    engine_id: 'panchanga',
    result: { tithi_name: 'Navami (Krishna)', nakshatra_name: 'Pushya' },
  },
  {
    engine_id: 'vimshottari',
    result: {
      current_period: {
        mahadasha: { planet: 'Ketu' },
        antardasha: { planet: 'Mercury' },
        pratyantardasha: { planet: 'Moon' },
      },
    },
  },
];

describe('formatEngineResultsForPrompt', () => {
  it('includes deterministic facts from each engine', () => {
    const out = formatEngineResultsForPrompt(sampleEngines);
    expect(out).toContain('panchanga');
    expect(out).toContain('Navami (Krishna)');
    expect(out).toContain('Pushya');
    expect(out).toContain('vimshottari');
    expect(out).toContain('Ketu');
    expect(out).toContain('Mercury');
    expect(out).toContain('Moon');
  });
});
```

**Step 2: Run test to verify it fails**

Run:
```bash
pnpm --filter @noesis/witness-pipeline test src/orchestrator/engine-formatter.test.ts
```

Expected: FAIL — `formatEngineResultsForPrompt` not found.

**Step 3: Write minimal implementation**

```typescript
import type { SelemeneEngineOutput } from '../index.js';

export function formatEngineResultsForPrompt(
  engineResults: SelemeneEngineOutput[],
): string {
  const blocks = engineResults.map((engine) => {
    const result = engine.result ?? {};
    return `### ${engine.engine_id}\n\n${jsonBlock(result)}`;
  });
  return `## Deterministic Engine Results\n\n${blocks.join('\n\n')}`;
}

function jsonBlock(value: unknown): string {
  return '```json\n' + JSON.stringify(value, null, 2) + '\n```';
}
```

**Step 4: Run test to verify it passes**

Run:
```bash
pnpm --filter @noesis/witness-pipeline test src/orchestrator/engine-formatter.test.ts
```

Expected: PASS.

**Step 5: Commit**

```bash
git add packages/witness-pipeline/src/orchestrator/engine-formatter.ts \
  packages/witness-pipeline/src/orchestrator/engine-formatter.test.ts
git commit -m "feat(witness-pipeline): add engine result prompt formatter"
```

---

### Wave 1.2: Inject Engine Results into Pass Prompts

#### Task 2: Add `{{engine_results}}` substitution to orchestrator

**Files:**
- Modify: `packages/witness-pipeline/src/orchestrator/integrated.ts:163-183`
- Test: `packages/witness-pipeline/src/orchestrator/integrated.test.ts`

**Step 1: Write the failing test**

Append to `integrated.test.ts`:

```typescript
it('includes engine results in the rendered prompt', async () => {
  const prompts: string[] = [];
  const llm = async (_system: string, prompt: string) => {
    prompts.push(prompt);
    return 'ok';
  };
  const orchestrator = new IntegratedReadingOrchestrator({ mode: parsedModeDoc, llm });
  await orchestrator.run({
    subjectNames: ['Harshita'],
    engineResultsBySubject: [sampleEngineResults],
    consciousnessLevel: 2,
  });
  expect(prompts.length).toBeGreaterThan(0);
  expect(prompts[0]).toContain('## Deterministic Engine Results');
  expect(prompts[0]).toContain('Pushya');
});
```

**Step 2: Run test to verify it fails**

Run:
```bash
pnpm --filter @noesis/witness-pipeline test src/orchestrator/integrated.test.ts
```

Expected: FAIL — prompt does not contain engine results.

**Step 3: Write minimal implementation**

In `integrated.ts`, update `renderPassTemplate`:

```typescript
private renderPassTemplate(
  template: string,
  pass: { id: string; title: string; target_words: number },
  input: OrchestratorInput,
  priorPass: string,
  register: RegisterBand,
): string {
  const overlaySummary = this.buildOverlaySummary();
  const bridgeMandates = this.mode.frontmatter.bridge_mandates.map((m) => `- ${m}`).join('\n');
  const lessonsSummary = summarizeLessons(this.mode.lessons, 5);
  const engineResults = formatEngineResultsForPrompt(input.engineResultsBySubject[0] ?? []);

  return template
    .replace(/\{\{subject_names\}\}/g, input.subjectNames.join(', '))
    .replace(/\{\{prior_pass\}\}/g, priorPass)
    .replace(/\{\{overlay_summary\}\}/g, overlaySummary)
    .replace(/\{\{bridge_mandates\}\}/g, bridgeMandates)
    .replace(/\{\{lessons_summary\}\}/g, lessonsSummary)
    .replace(/\{\{engine_results\}\}/g, engineResults)
    .replace(/\{\{register\}\}/g, register)
    .replace(/\{\{pass_id\}\}/g, pass.id)
    .replace(/\{\{target_words\}\}/g, String(pass.target_words));
}
```

Add import at top:

```typescript
import { formatEngineResultsForPrompt } from './engine-formatter.js';
```

**Step 4: Run test to verify it passes**

Run:
```bash
pnpm --filter @noesis/witness-pipeline test src/orchestrator/integrated.test.ts
```

Expected: PASS.

**Step 5: Commit**

```bash
git add packages/witness-pipeline/src/orchestrator/integrated.ts \
  packages/witness-pipeline/src/orchestrator/integrated.test.ts
git commit -m "feat(witness-pipeline): inject engine results into pass prompts"
```

---

#### Task 3: Update mode doc to reference `{{engine_results}}`

**Files:**
- Modify: `packages/witness-pipeline/modes/integrated-kundali-l0.md:93-110`

**Step 1: Write the failing test**

In `parser.test.ts`, add:

```typescript
it('contains engine_results placeholder in pass templates', () => {
  const doc = parseModeDoc('packages/witness-pipeline/modes/integrated-kundali-l0.md');
  const opening = doc.sections['opening-pass'];
  expect(opening).toContain('{{engine_results}}');
});
```

**Step 2: Run test to verify it fails**

Run:
```bash
pnpm --filter @noesis/witness-pipeline test src/modes/parser.test.ts
```

Expected: FAIL — `{{engine_results}}` not found in `opening-pass`.

**Step 3: Write minimal implementation**

Insert after the `Mandates:` block in every pass template (e.g., after line 106 of `integrated-kundali-l0.md`):

```markdown
Engine results:
{{engine_results}}
```

Repeat for all pass templates.

**Step 4: Run test to verify it passes**

Run:
```bash
pnpm --filter @noesis/witness-pipeline test src/modes/parser.test.ts
```

Expected: PASS.

**Step 5: Commit**

```bash
git add packages/witness-pipeline/modes/integrated-kundali-l0.md \
  packages/witness-pipeline/src/modes/parser.test.ts
git commit -m "docs(witness-pipeline): add engine_results placeholder to L0 mode"
```

---

## Phase 2: Verification Gates

### Wave 2.1: Placeholder Detection

#### Task 4: Add placeholder detector

**Files:**
- Modify: `packages/witness-pipeline/src/orchestrator/rubric.ts`
- Test: `packages/witness-pipeline/src/orchestrator/rubric.test.ts`

**Step 1: Write the failing test**

Append to `rubric.test.ts`:

```typescript
it('flags unsubstituted placeholders', () => {
  const rubric = auditSectionOutput({
    sectionId: 'opening',
    title: 'Opening',
    targetWords: 450,
    output: 'Your Lagna is [exact sign from engine results].',
    modelRequested: 'test',
    modelUsed: 'test',
    latencyMs: 0,
    engineResults: sampleEngineResults,
  });
  expect(rubric.placeholder_gate).toBe('fail');
});
```

**Step 2: Run test to verify it fails**

Run:
```bash
pnpm --filter @noesis/witness-pipeline test src/orchestrator/rubric.test.ts
```

Expected: FAIL — `placeholder_gate` property missing.

**Step 3: Write minimal implementation**

Add to `rubric.ts`:

```typescript
const PLACEHOLDER_RE = /\[[^\]]+\bfrom engine results\b[^\]]*\]/gi;

function hasPlaceholder(output: string): boolean {
  return PLACEHOLDER_RE.test(output);
}
```

Update `SectionRubric` interface:

```typescript
export interface SectionRubric {
  // ... existing fields ...
  placeholder_gate: 'pass' | 'fail';
}
```

Update `auditSectionOutput` return:

```typescript
return {
  // ... existing fields ...
  placeholder_gate: hasPlaceholder(output) ? 'fail' : 'pass',
};
```

**Step 4: Run test to verify it passes**

Run:
```bash
pnpm --filter @noesis/witness-pipeline test src/orchestrator/rubric.test.ts
```

Expected: PASS.

**Step 5: Commit**

```bash
git add packages/witness-pipeline/src/orchestrator/rubric.ts \
  packages/witness-pipeline/src/orchestrator/rubric.test.ts
git commit -m "feat(witness-pipeline): detect unsubstituted engine placeholders"
```

---

### Wave 2.2: Final Verification & Fidelity Gate

#### Task 5: Create final verification function

**Files:**
- Create: `packages/witness-pipeline/src/orchestrator/final-verification.ts`
- Test: `packages/witness-pipeline/src/orchestrator/final-verification.test.ts`

**Step 1: Write the failing test**

```typescript
import { describe, it, expect } from 'vitest';
import { runFinalVerification } from './final-verification.js';
import type { PassResult } from './integrated.js';

const failingPass = {
  id: 'opening',
  rubric: { placeholder_gate: 'fail', chart_fidelity_gate: 'pass' },
} as unknown as PassResult;

const passingPass = {
  id: 'opening',
  rubric: { placeholder_gate: 'pass', chart_fidelity_gate: 'pass' },
} as unknown as PassResult;

describe('runFinalVerification', () => {
  it('fails when any section has placeholder_gate fail', () => {
    const result = runFinalVerification({ passes: [failingPass], pdfPath: 'x.pdf' });
    expect(result.passed).toBe(false);
    expect(result.blockers).toContain('opening:placeholder_gate');
  });

  it('passes when all gates pass and pdf exists', () => {
    const result = runFinalVerification({ passes: [passingPass], pdfPath: 'x.pdf' });
    expect(result.passed).toBe(true);
  });
});
```

**Step 2: Run test to verify it fails**

Run:
```bash
pnpm --filter @noesis/witness-pipeline test src/orchestrator/final-verification.test.ts
```

Expected: FAIL — `runFinalVerification` not defined.

**Step 3: Write minimal implementation**

```typescript
import type { PassResult } from './integrated.js';
import fs from 'node:fs';

export interface FinalVerificationInput {
  passes: PassResult[];
  pdfPath?: string;
}

export interface FinalVerificationResult {
  passed: boolean;
  blockers: string[];
}

export function runFinalVerification(input: FinalVerificationInput): FinalVerificationResult {
  const blockers: string[] = [];
  for (const pass of input.passes) {
    const r = pass.rubric as any;
    if (r.placeholder_gate === 'fail') blockers.push(`${pass.id}:placeholder_gate`);
    if (r.chart_fidelity_gate === 'fail') blockers.push(`${pass.id}:chart_fidelity_gate`);
  }
  if (input.pdfPath && !fs.existsSync(input.pdfPath)) {
    blockers.push('pdf:missing');
  }
  return { passed: blockers.length === 0, blockers };
}
```

**Step 4: Run test to verify it passes**

Run:
```bash
pnpm --filter @noesis/witness-pipeline test src/orchestrator/final-verification.test.ts
```

Expected: PASS.

**Step 5: Commit**

```bash
git add packages/witness-pipeline/src/orchestrator/final-verification.ts \
  packages/witness-pipeline/src/orchestrator/final-verification.test.ts
git commit -m "feat(witness-pipeline): add final verification with placeholder gate"
```

---

#### Task 6: Add `chart_fidelity_gate`

**Files:**
- Modify: `packages/witness-pipeline/src/orchestrator/rubric.ts`
- Test: `packages/witness-pipeline/src/orchestrator/rubric.test.ts`

**Step 1: Write the failing test**

Append to `rubric.test.ts`:

```typescript
it('fails chart fidelity when score is below threshold', () => {
  const rubric = auditSectionOutput({
    sectionId: 'opening',
    title: 'Opening',
    targetWords: 450,
    output: 'Generic text with no specific facts.',
    modelRequested: 'test',
    modelUsed: 'test',
    latencyMs: 0,
    engineResults: sampleEngineResults,
  });
  expect(rubric.chart_fidelity_gate).toBe('fail');
});
```

**Step 2: Run test to verify it fails**

Run:
```bash
pnpm --filter @noesis/witness-pipeline test src/orchestrator/rubric.test.ts
```

Expected: FAIL — `chart_fidelity_gate` missing.

**Step 3: Write minimal implementation**

Update `SectionRubric` interface:

```typescript
export interface SectionRubric {
  // ... existing fields ...
  chart_fidelity_gate: 'pass' | 'warn' | 'fail';
}
```

Update `extractKeyFactsFromEngines` to handle the actual engine result shapes in `engines.json`:

```typescript
function extractKeyFactsFromEngines(engines: any[]): Set<string> {
  const facts = new Set<string>();
  for (const e of engines) {
    const r = e.result || {};
    if (r.lagna_sign) facts.add(`lagna:${String(r.lagna_sign).toLowerCase()}`);
    if (r.lagna) facts.add(`lagna:${String(r.lagna).toLowerCase()}`);
    if (r.tithi_name) facts.add(`tithi:${r.tithi_name}`.toLowerCase());
    if (r.nakshatra_name) facts.add(`nakshatra:${r.nakshatra_name}`.toLowerCase());
    if (r.current_period?.mahadasha?.planet) {
      facts.add(`dasha:${r.current_period.mahadasha.planet}`.toLowerCase());
    }
    if (r.current_period?.antardasha?.planet) {
      facts.add(`antardasha:${r.current_period.antardasha.planet}`.toLowerCase());
    }
    if (Array.isArray(r.active_channels)) {
      r.active_channels.forEach((c: string) => facts.add(`channel:${c}`));
    }
    if (r.design_activations) {
      Object.values(r.design_activations).forEach((a: any) => {
        if (a.gate) facts.add(`gate:${a.gate}`);
      });
    }
    if (r.personality_activations) {
      Object.values(r.personality_activations).forEach((a: any) => {
        if (a.gate) facts.add(`gate:${a.gate}`);
      });
    }
  }
  return facts;
}
```

Update `auditSectionOutput` return:

```typescript
const fidelityGate = fid.score >= 0.8 ? 'pass' : fid.score >= 0.5 ? 'warn' : 'fail';

return {
  // ... existing fields ...
  chart_fidelity_gate: engineFacts.size > 0 ? fidelityGate : 'warn',
};
```

**Step 4: Run test to verify it passes**

Run:
```bash
pnpm --filter @noesis/witness-pipeline test src/orchestrator/rubric.test.ts
```

Expected: PASS.

**Step 5: Commit**

```bash
git add packages/witness-pipeline/src/orchestrator/rubric.ts \
  packages/witness-pipeline/src/orchestrator/rubric.test.ts
git commit -m "feat(witness-pipeline): gate on chart fidelity score"
```

---

## Phase 3: Brand-Aware Renderer & PDF

### Wave 3.1: Brand Token Loader

#### Task 7: Create brand token loader

**Files:**
- Create: `packages/witness-pipeline/src/assets/brand-loader.ts`
- Test: `packages/witness-pipeline/src/assets/brand-loader.test.ts`

**Step 1: Write the failing test**

```typescript
import { describe, it, expect } from 'vitest';
import { loadBrandTokens } from './brand-loader.js';

describe('loadBrandTokens', () => {
  it('loads colors and fonts from brand-config.yaml', async () => {
    const tokens = await loadBrandTokens('/Volumes/madara/2026/twc-vault/01-Projects/tryambakam-noesis/brand-docs-final/tryambakam-noesis-aleph/brand-config.yaml');
    expect(tokens.colors.voidBlack).toBe('#070B1D');
    expect(tokens.fonts.display).toBe('Panchang');
  });
});
```

**Step 2: Run test to verify it fails**

Run:
```bash
pnpm --filter @noesis/witness-pipeline test src/assets/brand-loader.test.ts
```

Expected: FAIL — `loadBrandTokens` not defined.

**Step 3: Write minimal implementation**

```typescript
import { readFile } from 'node:fs/promises';
import yaml from 'js-yaml';

export interface BrandTokens {
  colors: Record<string, string>;
  fonts: Record<string, string>;
}

export async function loadBrandTokens(configPath: string): Promise<BrandTokens> {
  const raw = await readFile(configPath, 'utf-8');
  const doc = yaml.load(raw) as any;
  return {
    colors: doc.colors ?? {},
    fonts: doc.typography ?? {},
  };
}
```

**Step 4: Run test to verify it passes**

Run:
```bash
pnpm --filter @noesis/witness-pipeline test src/assets/brand-loader.test.ts
```

Expected: PASS.

**Step 5: Commit**

```bash
git add packages/witness-pipeline/src/assets/brand-loader.ts \
  packages/witness-pipeline/src/assets/brand-loader.test.ts
git commit -m "feat(witness-pipeline): load brand tokens from yaml"
```

---

### Wave 3.2: HTML Renderer

#### Task 8: Create HTML renderer

**Files:**
- Create: `packages/witness-pipeline/src/assets/html-renderer.ts`
- Test: `packages/witness-pipeline/src/assets/html-renderer.test.ts`

**Step 1: Write the failing test**

```typescript
import { describe, it, expect } from 'vitest';
import { renderReadingToHtml } from './html-renderer.js';
import type { BrandTokens } from './brand-loader.js';

const tokens: BrandTokens = {
  colors: { voidBlack: '#070B1D', sacredGold: '#C5A017', parchment: '#F0EDE3' },
  fonts: { display: 'Panchang', body: 'Satoshi', mono: 'SF Mono' },
};

describe('renderReadingToHtml', () => {
  it('wraps markdown in brand-styled HTML', async () => {
    const html = await renderReadingToHtml({
      title: 'L0 Integrated Kundali — Harshita',
      markdown: '# Hello\n\nYour nakshatra is Pushya.',
      brandTokens: tokens,
    });
    expect(html).toContain('<html');
    expect(html).toContain('Pushya');
    expect(html).toContain('--void-black: #070B1D');
  });
});
```

**Step 2: Run test to verify it fails**

Run:
```bash
pnpm --filter @noesis/witness-pipeline test src/assets/html-renderer.test.ts
```

Expected: FAIL — `renderReadingToHtml` not defined.

**Step 3: Write minimal implementation**

```typescript
import type { BrandTokens } from './brand-loader.js';

export interface RenderHtmlInput {
  title: string;
  markdown: string;
  brandTokens: BrandTokens;
}

export async function renderReadingToHtml(input: RenderHtmlInput): Promise<string> {
  const { colors, fonts } = input.brandTokens;
  const colorVars = Object.entries(colors)
    .map(([k, v]) => `  --${k.replace(/([A-Z])/g, '-$1').toLowerCase()}: ${v};`)
    .join('\n');

  return `<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <title>${escapeHtml(input.title)}</title>
  <style>
    :root {
${colorVars}
    }
    body {
      font-family: ${fonts.body ?? 'serif'}, serif;
      background: var(--parchment);
      color: var(--void-black);
      max-width: 680px;
      margin: 0 auto;
      padding: 2rem;
    }
    h1, h2, h3 { font-family: ${fonts.display ?? 'serif'}, serif; }
  </style>
</head>
<body>
  ${markdownToHtml(input.markdown)}
</body>
</html>`;
}

function escapeHtml(text: string): string {
  return text
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;');
}

function markdownToHtml(markdown: string): string {
  return markdown
    .replace(/^# (.*$)/gim, '<h1>$1</h1>')
    .replace(/^## (.*$)/gim, '<h2>$1</h2>')
    .replace(/\*\*(.*?)\*\*/g, '<strong>$1</strong>')
    .replace(/\n/g, '\n  ');
}
```

**Step 4: Run test to verify it passes**

Run:
```bash
pnpm --filter @noesis/witness-pipeline test src/assets/html-renderer.test.ts
```

Expected: PASS.

**Step 5: Commit**

```bash
git add packages/witness-pipeline/src/assets/html-renderer.ts \
  packages/witness-pipeline/src/assets/html-renderer.test.ts
git commit -m "feat(witness-pipeline): brand-aware HTML renderer"
```

---

### Wave 3.3: PDF Renderer

#### Task 9: Add Playwright for PDF conversion

**Files:**
- Modify: `packages/witness-pipeline/package.json`

**Step 1: Write the failing test**

Create `packages/witness-pipeline/src/assets/pdf-renderer.test.ts`:

```typescript
import { describe, it, expect } from 'vitest';
import { renderHtmlToPdf } from './pdf-renderer.js';
import { writeFile, unlink } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';

describe('renderHtmlToPdf', () => {
  it('writes a PDF file', async () => {
    const htmlPath = join(tmpdir(), 'test.html');
    const pdfPath = join(tmpdir(), 'test.pdf');
    await writeFile(htmlPath, '<html><body>Hello</body></html>');
    await renderHtmlToPdf({ htmlPath, pdfPath });
    const stats = await stat(pdfPath);
    expect(stats.size).toBeGreaterThan(0);
    await unlink(htmlPath);
    await unlink(pdfPath);
  });
});
```

**Step 2: Run test to verify it fails**

Run:
```bash
pnpm --filter @noesis/witness-pipeline test src/assets/pdf-renderer.test.ts
```

Expected: FAIL — `renderHtmlToPdf` not defined or Playwright not installed.

**Step 3: Write minimal implementation**

Add dependency:

```bash
pnpm --filter @noesis/witness-pipeline add -D playwright
```

Create `packages/witness-pipeline/src/assets/pdf-renderer.ts`:

```typescript
import { chromium } from 'playwright';

export interface RenderPdfInput {
  htmlPath: string;
  pdfPath: string;
}

export async function renderHtmlToPdf(input: RenderPdfInput): Promise<void> {
  const browser = await chromium.launch();
  const page = await browser.newPage();
  await page.goto(`file://${input.htmlPath}`);
  await page.pdf({ path: input.pdfPath, format: 'A4', printBackground: true });
  await browser.close();
}
```

**Step 4: Run test to verify it passes**

Run:
```bash
pnpm --filter @noesis/witness-pipeline test src/assets/pdf-renderer.test.ts
```

Expected: PASS (may require `pnpm exec playwright install chromium`).

**Step 5: Commit**

```bash
git add packages/witness-pipeline/package.json pnpm-lock.yaml \
  packages/witness-pipeline/src/assets/pdf-renderer.ts \
  packages/witness-pipeline/src/assets/pdf-renderer.test.ts
git commit -m "feat(witness-pipeline): add Playwright PDF renderer"
```

---

#### Task 10: Orchestrate render pipeline

**Files:**
- Create: `packages/witness-pipeline/src/assets/render-pipeline.ts`
- Test: `packages/witness-pipeline/src/assets/render-pipeline.test.ts`

**Step 1: Write the failing test**

```typescript
import { describe, it, expect } from 'vitest';
import { renderLocalArtifacts } from './render-pipeline.js';
import { mkdtempSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';

describe('renderLocalArtifacts', () => {
  it('creates reading.html and reading.pdf in output dir', async () => {
    const dir = mkdtempSync(join(tmpdir(), 'render-test-'));
    writeFileSync(join(dir, 'reading.md'), '# Hello\n\nPushya');
    writeFileSync(join(dir, 'engines.json'), JSON.stringify([]));

    const result = await renderLocalArtifacts({
      sourcePackDir: dir,
      outputDir: dir,
      brandConfigPath: '/Volumes/madara/2026/twc-vault/01-Projects/tryambakam-noesis/brand-docs-final/tryambakam-noesis-aleph/brand-config.yaml',
    });
    expect(result.htmlPath).toBe(join(dir, 'reading.html'));
    expect(result.pdfPath).toBe(join(dir, 'reading.pdf'));
  });
});
```

**Step 2: Run test to verify it fails**

Run:
```bash
pnpm --filter @noesis/witness-pipeline test src/assets/render-pipeline.test.ts
```

Expected: FAIL — `renderLocalArtifacts` not defined.

**Step 3: Write minimal implementation**

```typescript
import { readFile, writeFile } from 'node:fs/promises';
import { join } from 'node:path';
import { loadBrandTokens } from './brand-loader.js';
import { renderReadingToHtml } from './html-renderer.js';
import { renderHtmlToPdf } from './pdf-renderer.js';

export interface RenderPipelineInput {
  sourcePackDir: string;
  outputDir: string;
  brandConfigPath: string;
}

export interface RenderPipelineOutput {
  htmlPath: string;
  pdfPath: string;
}

export async function renderLocalArtifacts(input: RenderPipelineInput): Promise<RenderPipelineOutput> {
  const readingMd = await readFile(join(input.sourcePackDir, 'reading.md'), 'utf-8');
  const manifest = JSON.parse(await readFile(join(input.sourcePackDir, 'manifest.json'), 'utf-8'));
  const brandTokens = await loadBrandTokens(input.brandConfigPath);
  const title = `L0 Integrated Kundali — ${manifest.person_id}`;

  const html = await renderReadingToHtml({ title, markdown: readingMd, brandTokens });
  const htmlPath = join(input.outputDir, 'reading.html');
  const pdfPath = join(input.outputDir, 'reading.pdf');

  await writeFile(htmlPath, html, 'utf-8');
  await renderHtmlToPdf({ htmlPath, pdfPath });

  return { htmlPath, pdfPath };
}
```

**Step 4: Run test to verify it passes**

Run:
```bash
pnpm --filter @noesis/witness-pipeline test src/assets/render-pipeline.test.ts
```

Expected: PASS.

**Step 5: Commit**

```bash
git add packages/witness-pipeline/src/assets/render-pipeline.ts \
  packages/witness-pipeline/src/assets/render-pipeline.test.ts
git commit -m "feat(witness-pipeline): orchestrate HTML+PDF render pipeline"
```

---

## Phase 4: Migration & Closeout

### Wave 4.1: Adopt Canonical Factory Path

#### Task 11: Update factory manifest to include `report_level`

**Files:**
- Modify: `packages/witness-pipeline/src/assets/factory.ts`
- Test: `packages/witness-pipeline/src/assets/factory.test.ts`

**Step 1: Write the failing test**

Append to `factory.test.ts`:

```typescript
it('includes report_level in manifest', async () => {
  const pack = await createSourcePack({
    personId: 'harshita',
    readingMarkdown: '# Test',
    engineResults: [],
    outputDir: tmpdir(),
    reportLevel: 'L0',
  });
  expect(pack.manifest.report_level).toBe('L0');
});
```

**Step 2: Run test to verify it fails**

Run:
```bash
pnpm --filter @noesis/witness-pipeline test src/assets/factory.test.ts
```

Expected: FAIL — `reportLevel` not accepted.

**Step 3: Write minimal implementation**

Update `SourcePackInput` interface:

```typescript
export interface SourcePackInput {
  // ... existing fields ...
  reportLevel?: string;
}
```

Update manifest:

```typescript
const manifest: SourcePack['manifest'] = {
  person_id: input.personId,
  created_at: new Date().toISOString(),
  reading_length: input.readingMarkdown.length,
  engines: input.engineResults.map((e) => e.engine_id),
  report_level: input.reportLevel ?? 'L3',
  quality,
};
```

Update `SourcePack.manifest` type to include `report_level?: string`.

**Step 4: Run test to verify it passes**

Run:
```bash
pnpm --filter @noesis/witness-pipeline test src/assets/factory.test.ts
```

Expected: PASS.

**Step 5: Commit**

```bash
git add packages/witness-pipeline/src/assets/factory.ts \
  packages/witness-pipeline/src/assets/factory.test.ts
git commit -m "feat(witness-pipeline): include report_level in source-pack manifest"
```

---

#### Task 12: Wire renderer into L0 smoke script

**Files:**
- Modify: `packages/witness-pipeline/scripts/sapna-l0-test.ts`
- Or create: `packages/witness-pipeline/scripts/l0-render.ts`

**Step 1: Write the failing test**

Create `packages/witness-pipeline/scripts/l0-render.test.ts`:

```typescript
import { describe, it, expect } from 'vitest';
import { buildL0ArtifactPath } from './l0-render.js';

describe('buildL0ArtifactPath', () => {
  it('returns canonical factory local path', () => {
    const path = buildL0ArtifactPath('harshita');
    expect(path).toContain('witness-agents-archive/.premium-assets-witness-harshita/harshita/local');
  });
});
```

**Step 2: Run test to verify it fails**

Run:
```bash
pnpm --filter @noesis/witness-pipeline test scripts/l0-render.test.ts
```

Expected: FAIL — `l0-render.ts` not found.

**Step 3: Write minimal implementation**

Create `packages/witness-pipeline/scripts/l0-render.ts`:

```typescript
import { renderLocalArtifacts } from '../src/assets/render-pipeline.js';

export function buildL0ArtifactPath(personId: string): string {
  return `/Volumes/madara/2026/twc-vault/01-Projects/tryambakam-noesis/723/witness-agents-archive/.premium-assets-witness-${personId}/${personId}/local`;
}

export async function renderL0Local(personId: string) {
  const sourcePackDir = buildL0ArtifactPath(personId).replace('/local', '/source-pack');
  const outputDir = buildL0ArtifactPath(personId);
  return renderLocalArtifacts({
    sourcePackDir,
    outputDir,
    brandConfigPath: '/Volumes/madara/2026/twc-vault/01-Projects/tryambakam-noesis/brand-docs-final/tryambakam-noesis-aleph/brand-config.yaml',
  });
}
```

**Step 4: Run test to verify it passes**

Run:
```bash
pnpm --filter @noesis/witness-pipeline test scripts/l0-render.test.ts
```

Expected: PASS.

**Step 5: Commit**

```bash
git add packages/witness-pipeline/scripts/l0-render.ts \
  packages/witness-pipeline/scripts/l0-render.test.ts
git commit -m "feat(witness-pipeline): add L0 local render helper"
```

---

### Wave 4.2: Migration & Documentation

#### Task 13: Migrate harshita `new-l0-flow` to canonical path

**Files:**
- Shell script: `runbooks/migrate-harshita-l0.sh`

**Step 1: Write the migration script**

```bash
#!/usr/bin/env bash
set -euo pipefail

SRC="/Volumes/madara/2026/twc-vault/01-Projects/tryambakam-noesis/723/Solos/harshita/new-l0-flow"
DST="/Volumes/madara/2026/twc-vault/01-Projects/tryambakam-noesis/723/witness-agents-archive/.premium-assets-witness-harshita/harshita"

mkdir -p "$DST/source-pack" "$DST/local"
cp "$SRC/source-pack/manifest.json" "$DST/source-pack/manifest.json"
cp "$SRC/source-pack/reading.md" "$DST/source-pack/reading.md"
cp "$SRC/source-pack/reflection-questions.md" "$DST/source-pack/reflection-questions.md"
cp "$SRC/engines.json" "$DST/source-pack/engines.json"
cp "$SRC/report.html" "$DST/local/reading.html"
cp "$SRC/report.pdf" "$DST/local/reading.pdf"

echo "Migrated $SRC -> $DST"
```

**Step 2: Run the migration script**

```bash
chmod +x runbooks/migrate-harshita-l0.sh
./runbooks/migrate-harshita-l0.sh
```

Expected: files copied to canonical path.

**Step 3: Verify the canonical path structure**

```bash
ls -R /Volumes/madara/2026/twc-vault/01-Projects/tryambakam-noesis/723/witness-agents-archive/.premium-assets-witness-harshita/harshita
```

Expected: `source-pack/` and `local/` contents present.

**Step 4: Commit**

```bash
git add runbooks/migrate-harshita-l0.sh
git commit -m "chore: add harshita L0 migration script"
```

---

#### Task 14: Update witness-pipeline README

**Files:**
- Modify: `packages/witness-pipeline/README.md`

**Step 1: Add renderer section**

Insert after the existing usage section:

```markdown
## Rendering L0 artifacts to HTML/PDF

After generating a source pack, render the `local/` artifacts:

```bash
pnpm --filter @noesis/witness-pipeline exec tsx scripts/l0-render.ts harshita
```

This writes:
- `local/reading.html`
- `local/reading.pdf`

Brand tokens are loaded from `brand-docs-final/tryambakam-noesis-aleph/brand-config.yaml`.
```

**Step 2: Commit**

```bash
git add packages/witness-pipeline/README.md
git commit -m "docs(witness-pipeline): document L0 render pipeline"
```

---

#### Task 15: End-to-end smoke test

**Files:**
- Modify: `packages/witness-pipeline/scripts/sapna-l0-test.ts`

**Step 1: Add final verification print**

After the existing rubric summary, add:

```typescript
import { runFinalVerification } from '../src/orchestrator/final-verification.js';

const verification = runFinalVerification({
  passes: result.passes,
  pdfPath: rendered?.pdfPath,
});
console.log('Final verification:', verification.passed ? 'PASS' : 'FAIL', verification.blockers);
if (!verification.passed) process.exit(1);
```

**Step 2: Run smoke test**

```bash
pnpm --filter @noesis/witness-pipeline smoke
```

Expected: PASS with no placeholder or fidelity blockers.

**Step 3: Commit**

```bash
git add packages/witness-pipeline/scripts/sapna-l0-test.ts
git commit -m "feat(witness-pipeline): enforce final verification in L0 smoke"
```

---

## Dependency Graph

```
Task 1 (engine formatter)
  │
  ▼
Task 2 (orchestrator injection) ──► Task 3 (mode doc update)
  │
  ▼
Task 4 (placeholder detector)
  │
  ▼
Task 5 (final verification)
  │
  ▼
Task 6 (fidelity gate)
  │
  ▼
Task 7 (brand loader) ──► Task 8 (HTML renderer) ──► Task 9 (PDF renderer) ──► Task 10 (render pipeline)
  │
  ▼
Task 11 (factory report_level) ──► Task 12 (L0 render helper)
  │
  ▼
Task 13 (migration) ──► Task 14 (docs) ──► Task 15 (smoke test)
```

## Timeline

| Phase | Days | Deliverable |
|---|---|---|
| Phase 1 | 2 | Engine results injected into LLM prompts |
| Phase 2 | 1 | Placeholder + fidelity gates wired |
| Phase 3 | 3 | Brand-aware HTML/PDF renderer |
| Phase 4 | 1 | Migration, docs, smoke test |
| **Total** | **7** | **L0 pipeline produces factory-integrated, branded PDF** |

## GitHub Roadmap Mapping

Create one GitHub issue per phase. Each issue contains the wave/task checklist.

| Issue | Title | Labels | Depends on |
|---|---|---|---|
| #1 | Fix engine result ingestion in L0 orchestrator | `enhancement`, `witness-pipeline`, `L0` | — |
| #2 | Harden L0 verification gates | `enhancement`, `quality`, `L0` | #1 |
| #3 | Build brand-aware HTML/PDF renderer | `enhancement`, `renderer`, `L0` | #1 |
| #4 | Migrate orphan `new-l0-flow` and closeout | `chore`, `migration`, `L0` | #2, #3 |

Use the migration script in Task 13 to move the existing harshita artifact once #1–#3 land.

## Verification Strategy

- Unit tests for every new module (`engine-formatter`, `rubric` gates, `brand-loader`, `html-renderer`, `pdf-renderer`, `render-pipeline`, `final-verification`).
- `pnpm --filter @noesis/witness-pipeline test` must be green after each task.
- `pnpm --filter @noesis/witness-pipeline typecheck` must pass.
- `pnpm --filter @noesis/witness-pipeline smoke` must pass and produce a `reading.pdf` with no placeholder blockers.
- Manual inspection of the generated `local/reading.html` to confirm engine facts are substituted.

## Risks & Fallbacks

| Risk | Fallback |
|---|---|
| Playwright/Chromium install is heavy | Swap to `puppeteer-core` with system Chrome, or render HTML-only in phase 3 and defer PDF. |
| Brand-config.yaml shape differs from assumptions | Fail test early; add schema validation in `brand-loader.ts`. |
| `engines.json` shapes vary per subject | Expand `extractKeyFactsFromEngines` with more field paths and add fixtures for each engine. |
| Mode doc already used in production without `{{engine_results}}` | Keep change additive; the placeholder is ignored if not present, and existing templates without it continue to work. |

---

## Execution Handoff

Plan complete and saved to `docs/plans/2026-07-07-l0-kundali-engine-substitution-and-factory-integration.md`.

Two execution options:

**1. Subagent-Driven (this session)** - Dispatch a fresh subagent per task, review between tasks, fast iteration.

**2. Parallel Session (separate)** - Open a new session in the worktree with `superpowers:executing-plans` for batch execution with checkpoints.

Which approach?
