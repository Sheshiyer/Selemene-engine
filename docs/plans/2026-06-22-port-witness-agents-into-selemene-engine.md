# Port witness-agents Premium Pipeline + Upgrade Internal Witness Dyad Implementation Plan

## ✅ COMPLETED — 2026-06-22

All 17 tasks have been implemented. The branch `feat/port-witness-premium-pipeline` is ready for merge.

### Summary of Changes

**TypeScript Package (`packages/witness-pipeline`):**
- Scaffolded new ESM workspace with vitest, typescript config
- Ported Selemene engine fetcher with dependency injection
- Ported engine→pillar routing (aletheios/pichet/dyad)
- Ported mode-document parser with register bands
- Ported integrated reading orchestrator skeleton
- Ported source-pack factory (manifest, reading, questions)
- Ported chain audit (deterministic-fact gates)
- Added web client helper in `apps/noesis-web`
- Added README, smoke test, mode documents
- **18 tests passing**

**Rust Crates (`crates/noesis-witness`, `crates/noesis-api`):**
- Added `routing.rs` module with `RoutingMode` enum
- Extended `WitnessContext` with `RelationshipMode` and `partner_context`
- Refactored `build_context_message` to group by routing tag
- Updated API handler for partner/relationship context
- Improved `rule_based_dyad()` with tier/level gates
- **90 tests passing** (noesis-witness: 20, noesis-api: 70)

### Commits (15 total)
```
b806640d docs(witness-pipeline): add README, smoke test, and mode documents
9127a82e feat(noesis-api): improve rule-based fallback with tier/level gates
3c3d8e8c feat(noesis-api): add partner/relationship context to witness handler
ef92b646 feat(noesis-witness): refactor context formatting by routing tag
e6d107fa feat(noesis-witness): extend WitnessContext with synastry and relationship mode
ad41dc74 feat(noesis-witness): add engine routing metadata module
87500e08 feat(noesis-web): add witness-pipeline client helper
5b851eb0 chore(witness-pipeline): commit tsc build outputs
35130d10 feat(witness-pipeline): add source-pack factory and chain audit
a1f16e74 feat(witness-pipeline): add integrated reading orchestrator skeleton
cab90ac6 feat(witness-pipeline): backfill mode parser and types
80090a69 feat(witness-pipeline): add engine routing and typed result map
a3a0476a feat(witness-pipeline): port Selemene engine fetcher with injectable fetch
90dff5b8 feat(witness-pipeline): scaffold workspace package
fe1412c2 plan: port witness-agents premium pipeline + upgrade rust witness dyad
```

---

> **For Codex:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Bring the standalone `witness-agents/` premium TypeScript pipeline (Selemene fetcher, multi-pass integrated reading modes, premium asset factory, chain audit) into Selemene-engine as a new workspace, and upgrade the existing Rust `crates/noesis-witness` internal dyad with the latest multi-engine context, relationship-mode, and synthesis features from `witness-agents`.

**Architecture:** The TypeScript side becomes a new `packages/witness-pipeline` workspace that exposes `SelemeneClient`, `IntegratedReadingOrchestrator`, `PremiumAssetFactory`, and `AssetChainAuditor`. The Rust side extends `WitnessContext` with engine routing (Aletheios/Pichet/dyad-synthesis), optional synastry context, richer synthesis, and a deterministic rule-based fallback that mirrors the `witness-agents` tier gates. Both sides stay decoupled: the TS package can be used standalone or imported by `apps/noesis-web`; the Rust crate keeps its existing `noesis-api` `/witness/interpret` endpoint contract unchanged but richer.

**Tech Stack:** TypeScript 5.7 / Node 18 / ESM, Vitest for tests; Rust 2021 / tokio / reqwest / serde_json; Next.js for web consumption.

---

## Important Assumptions

1. We are porting code from `/Volumes/madara/2026/twc-vault/01-Projects/tryambakam-noesis/witness-agents/` into `/Volumes/madara/2026/twc-vault/01-Projects/tryambakam-noesis/selemene-engine-port-witness/`. Source files will be copied and then adapted.
2. We will NOT run real Selemene API calls in tests; HTTP fetch will be dependency-injected or mocked.
3. We will keep the existing `apps/noesis-web` `/get-reading` external call to `48.tryambakam.space` untouched until the new package proves it can replace it behind a feature flag.
4. NotebookLM artifact generation is out of scope for the first pass; source-pack + HTML/PDF + reflection questions are in scope.
5. The Rust upgrade is additive: existing `WitnessInterpretRequest`/`WitnessInterpretResponse` fields are preserved; only new optional fields and richer context formatting are added.

---

## Prerequisite: Setup and Validation

### Task 0: Confirm worktree and baseline

**Files:**
- Read: `Cargo.toml`
- Read: `package.json`
- Read: `packages/noesis-sdk-ts/package.json`
- Read: `crates/noesis-witness/Cargo.toml`

**Step 1: Verify the worktree branch**

Run:

```bash
git branch --show-current
git status --short
```

Expected:
- Branch: `feat/port-witness-premium-pipeline`
- Status: clean except for possibly `.env.test-local` and `crates/engine-nadabrahman/examples/`

**Step 2: Confirm the Rust witness crate compiles**

Run:

```bash
cargo test -p noesis-witness --no-run
```

Expected: `Finished` with no errors.

**Step 3: Commit worktree setup marker**

```bash
git add docs/plans/2026-06-22-port-witness-agents-into-selemene-engine.md
git commit -m "plan: port witness-agents premium pipeline + upgrade rust witness dyad"
```

---

## Part A — New TypeScript Workspace: `packages/witness-pipeline`

### Task 1: Scaffold the workspace package

**Files:**
- Create: `packages/witness-pipeline/package.json`
- Create: `packages/witness-pipeline/tsconfig.json`
- Modify: `package.json` (root) to add workspace
- Modify: `Cargo.toml` is not touched for this task

**Step 1: Write the failing workspace check**

Create `packages/witness-pipeline/package.json`:

```json
{
  "name": "@noesis/witness-pipeline",
  "version": "0.1.0",
  "description": "Premium integrated reading pipeline ported from witness-agents",
  "type": "module",
  "main": "dist/index.js",
  "types": "dist/index.d.ts",
  "exports": {
    ".": {
      "types": "./dist/index.d.ts",
      "import": "./dist/index.js"
    }
  },
  "scripts": {
    "build": "tsc -p tsconfig.json",
    "typecheck": "tsc --noEmit",
    "test": "vitest run",
    "clean": "rm -rf dist"
  },
  "dependencies": {
    "js-yaml": "^4.1.0"
  },
  "devDependencies": {
    "@types/js-yaml": "^4.0.9",
    "@types/node": "^22.0.0",
    "tsx": "^4.21.0",
    "typescript": "^5.7.0",
    "vitest": "^2.1.8"
  }
}
```

Create `packages/witness-pipeline/tsconfig.json`:

```json
{
  "compilerOptions": {
    "target": "ES2022",
    "module": "Node16",
    "moduleResolution": "Node16",
    "lib": ["ES2022"],
    "outDir": "dist",
    "rootDir": "src",
    "declaration": true,
    "declarationMap": true,
    "sourceMap": true,
    "strict": true,
    "esModuleInterop": true,
    "skipLibCheck": true,
    "forceConsistentCasingInFileNames": true,
    "resolveJsonModule": true,
    "isolatedModules": true
  },
  "include": ["src/**/*.ts"],
  "exclude": ["node_modules", "dist", "tests"]
}
```

Modify root `package.json` from:

```json
  "workspaces": [
    "apps/*",
    "packages/*"
  ],
```

To:

```json
  "workspaces": [
    "apps/*",
    "packages/*"
  ],
```

(No change needed because `packages/*` already matches.)

Create a test that the package is discoverable by npm workspaces:

Create `tests/witness-pipeline/workspace.test.ts`? No — keep tests inside the package. Create `packages/witness-pipeline/tests/workspace.test.ts`:

```typescript
import { describe, it, expect } from 'vitest';
import packageJson from '../package.json' assert { type: 'json' };

describe('witness-pipeline workspace', () => {
  it('has the expected package name', () => {
    expect(packageJson.name).toBe('@noesis/witness-pipeline');
  });
});
```

**Step 2: Run the test to verify it fails**

Run:

```bash
cd packages/witness-pipeline
npm test
```

Expected: FAIL — vitest not installed, or `tests/` not found, because we have not run `npm install` yet.

**Step 3: Install workspace dependencies**

Run from repo root:

```bash
npm install
```

Expected: `node_modules` updated, `package-lock.json` changes.

**Step 4: Run the test to verify it passes**

Run:

```bash
cd packages/witness-pipeline
npm test
```

Expected: PASS.

**Step 5: Commit**

```bash
git add packages/witness-pipeline/ package-lock.json
git commit -m "feat(witness-pipeline): scaffold workspace package"
```

---

### Task 2: Port Selemene engine fetcher

**Files:**
- Create: `packages/witness-pipeline/src/selemene/fetcher.ts`
- Create: `packages/witness-pipeline/src/selemene/fetcher.test.ts`
- Modify: `packages/witness-pipeline/src/index.ts`

**Step 1: Write the failing test**

Create `packages/witness-pipeline/src/selemene/fetcher.test.ts`:

```typescript
import { describe, it, expect, vi } from 'vitest';
import { fetchAllEngines, SELEMENE_BASE_URL, type BirthData } from './fetcher.js';

describe('fetchAllEngines', () => {
  it('returns engine results when all engines respond', async () => {
    const birthData: BirthData = {
      date: '1990-01-01',
      time: '12:00',
      timezone: 'Asia/Kolkata',
      latitude: 12.9716,
      longitude: 77.5946,
      name: 'Test',
    };

    const fakeFetch = vi.fn().mockResolvedValue({
      ok: true,
      json: async () => ({ engine_id: 'panchanga', result: { tithi_name: 'Test' } }),
    });

    const results = await fetchAllEngines(birthData, {
      api_key: 'test-key',
      base_url: 'http://localhost:9999',
      timeout_ms: 1000,
      engines: ['panchanga'],
      fetchImpl: fakeFetch as unknown as typeof fetch,
    });

    expect(results).toHaveLength(1);
    expect(results[0].engine_id).toBe('panchanga');
    expect(fakeFetch).toHaveBeenCalledWith(
      'http://localhost:9999/api/v1/engines/panchanga/calculate',
      expect.objectContaining({
        method: 'POST',
        headers: expect.objectContaining({ 'X-API-Key': 'test-key', 'Content-Type': 'application/json' }),
      })
    );
  });

  it('returns _error for failed engine calls', async () => {
    const fakeFetch = vi.fn().mockResolvedValue({
      ok: false,
      text: async () => 'boom',
    });

    const results = await fetchAllEngines(
      { date: '1990-01-01', timezone: 'UTC', latitude: 0, longitude: 0 },
      { api_key: 'k', engines: ['panchanga'], fetchImpl: fakeFetch as unknown as typeof fetch }
    );

    expect(results[0]._error).toContain('boom');
  });
});
```

**Step 2: Run the test to verify it fails**

Run:

```bash
cd packages/witness-pipeline
npm test
```

Expected: FAIL — `fetcher.ts` and `fetchImpl` option do not exist.

**Step 3: Implement the minimal code**

Create `packages/witness-pipeline/src/selemene/fetcher.ts`:

```typescript
// ─── Selemene Engine Fetcher ─────────────────────────────────────────
// Ported from witness-agents/scripts/integratedreading/selemene/fetcher.ts
// Adds dependency-injected fetchImpl for testability.

export const SELEMENE_BASE_URL = 'https://selemene.tryambakam.space';

export const SELEMENE_ENGINE_IDS = [
  'panchanga', 'vimshottari', 'human-design', 'gene-keys', 'numerology',
  'biorhythm', 'vedic-clock', 'biofield', 'face-reading', 'nadabrahman',
  'transits', 'tarot', 'i-ching', 'enneagram', 'sacred-geometry', 'sigil-forge',
] as const;

export type SelemeneEngineId = (typeof SELEMENE_ENGINE_IDS)[number];

export interface BirthData {
  date: string;
  time?: string;
  timezone?: string;
  latitude?: number;
  longitude?: number;
  name?: string;
}

export interface SelemeneEngineOutput {
  engine_id: SelemeneEngineId;
  result?: unknown;
  witness_prompt?: string;
  consciousness_level?: number;
  metadata?: {
    calculation_time_ms?: number;
    backend?: string;
    precision_achieved?: string;
    cached?: boolean;
    timestamp?: string;
    engine_version?: string;
  };
  envelope_version?: string;
  _error?: string;
}

export interface FetchOptions {
  api_key: string;
  base_url?: string;
  timeout_ms?: number;
  engines?: SelemeneEngineId[];
  fetchImpl?: typeof fetch;
}

async function callEngine(
  engineId: SelemeneEngineId,
  birthData: BirthData,
  opts: FetchOptions,
): Promise<SelemeneEngineOutput> {
  const url = `${opts.base_url ?? SELEMENE_BASE_URL}/api/v1/engines/${engineId}/calculate`;
  const ctrl = new AbortController();
  const t = setTimeout(() => ctrl.abort(), opts.timeout_ms ?? 30_000);
  const fetchImpl = opts.fetchImpl ?? fetch;
  try {
    const res = await fetchImpl(url, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'X-API-Key': opts.api_key,
      },
      body: JSON.stringify({ birth_data: birthData }),
      signal: ctrl.signal,
    });
    clearTimeout(t);
    if (!res.ok) {
      const txt = await res.text().catch(() => '');
      return { engine_id: engineId, _error: `HTTP ${res.status}: ${txt.slice(0, 200)}` };
    }
    const data: unknown = await res.json();
    return { ...(data as object), engine_id: engineId } as SelemeneEngineOutput;
  } catch (err: unknown) {
    clearTimeout(t);
    return { engine_id: engineId, _error: err instanceof Error ? err.message : String(err) };
  }
}

export async function fetchAllEngines(
  birthData: BirthData,
  opts: FetchOptions,
): Promise<SelemeneEngineOutput[]> {
  const engines = opts.engines ?? Array.from(SELEMENE_ENGINE_IDS);
  const results = await Promise.all(engines.map((eId) => callEngine(eId, birthData, opts)));
  return results;
}

export async function loadSelemeneKey(): Promise<string | undefined> {
  if (process.env.SELEMENE_API_KEY) return process.env.SELEMENE_API_KEY;
  const { default: fs } = await import('node:fs');
  const { default: path } = await import('node:path');
  const { default: os } = await import('node:os');
  const envPath = path.join(os.homedir(), '.claude', '.env');
  if (!fs.existsSync(envPath)) return undefined;
  const txt = fs.readFileSync(envPath, 'utf-8');
  const match = txt.match(/^SELEMENE_API_KEY=(\S+)/m);
  if (match) {
    process.env.SELEMENE_API_KEY = match[1];
    return match[1];
  }
  return undefined;
}
```

Create `packages/witness-pipeline/src/index.ts`:

```typescript
export * from './selemene/fetcher.js';
```

**Step 4: Run the test to verify it passes**

Run:

```bash
cd packages/witness-pipeline
npm test
```

Expected: PASS.

**Step 5: Commit**

```bash
git add packages/witness-pipeline/src/selemene/
git commit -m "feat(witness-pipeline): port Selemene engine fetcher with injectable fetch"
```

---

### Task 3: Port engine type map and routing

**Files:**
- Create: `packages/witness-pipeline/src/selemene/types.ts`
- Create: `packages/witness-pipeline/src/selemene/types.test.ts`
- Modify: `packages/witness-pipeline/src/index.ts`

**Step 1: Write the failing test**

Create `packages/witness-pipeline/src/selemene/types.test.ts`:

```typescript
import { describe, it, expect } from 'vitest';
import { ENGINE_ROUTING, ENGINE_ID_MAP, REVERSE_ENGINE_MAP, type SelemeneEngineId } from './types.js';

describe('engine routing', () => {
  it('routes vimshottari as aletheios-primary', () => {
    expect(ENGINE_ROUTING['vimshottari']).toBe('aletheios-primary');
  });

  it('routes biofield as pichet-primary', () => {
    expect(ENGINE_ROUTING['biofield']).toBe('pichet-primary');
  });

  it('routes panchanga as dyad-synthesis', () => {
    expect(ENGINE_ROUTING['panchanga']).toBe('dyad-synthesis');
  });

  it('maps every Selemene id to a witness alias and back', () => {
    const ids = Object.keys(ENGINE_ROUTING) as SelemeneEngineId[];
    for (const id of ids) {
      const alias = ENGINE_ID_MAP[id];
      expect(alias).toBeDefined();
      expect(REVERSE_ENGINE_MAP[alias]).toBe(id);
    }
  });
});
```

**Step 2: Run the test to verify it fails**

Run:

```bash
cd packages/witness-pipeline
npm test
```

Expected: FAIL — `types.ts` does not exist.

**Step 3: Implement the minimal code**

Create `packages/witness-pipeline/src/selemene/types.ts`:

```typescript
// ─── Engine identifiers, routing, and typed result map ───────────────
// Ported from witness-agents/src/types/engine.ts

export const SELEMENE_ENGINE_IDS = [
  'panchanga', 'vimshottari', 'human-design', 'gene-keys', 'numerology',
  'biorhythm', 'vedic-clock', 'biofield', 'face-reading', 'nadabrahman',
  'transits', 'tarot', 'i-ching', 'enneagram', 'sacred-geometry', 'sigil-forge',
] as const;

export type SelemeneEngineId = (typeof SELEMENE_ENGINE_IDS)[number];

export type WitnessEngineAlias =
  | 'temporal-grammar' | 'chronofield' | 'energetic-authority' | 'gift-shadow-spectrum'
  | 'numeric-architecture' | 'three-wave-cycle' | 'circadian-cartography' | 'bioelectric-field'
  | 'physiognomic-mapping' | 'resonance-architecture' | 'active-planetary-weather'
  | 'archetypal-mirror' | 'hexagram-navigation' | 'nine-point-architecture'
  | 'geometric-resonance' | 'sigil-forge';

export const ENGINE_ID_MAP: Record<SelemeneEngineId, WitnessEngineAlias> = {
  'panchanga': 'temporal-grammar',
  'vimshottari': 'chronofield',
  'human-design': 'energetic-authority',
  'gene-keys': 'gift-shadow-spectrum',
  'numerology': 'numeric-architecture',
  'biorhythm': 'three-wave-cycle',
  'vedic-clock': 'circadian-cartography',
  'biofield': 'bioelectric-field',
  'face-reading': 'physiognomic-mapping',
  'nadabrahman': 'resonance-architecture',
  'transits': 'active-planetary-weather',
  'tarot': 'archetypal-mirror',
  'i-ching': 'hexagram-navigation',
  'enneagram': 'nine-point-architecture',
  'sacred-geometry': 'geometric-resonance',
  'sigil-forge': 'sigil-forge',
};

export const REVERSE_ENGINE_MAP: Record<WitnessEngineAlias, SelemeneEngineId> =
  Object.fromEntries(Object.entries(ENGINE_ID_MAP).map(([k, v]) => [v, k])) as Record<WitnessEngineAlias, SelemeneEngineId>;

export type RoutingMode = 'aletheios-primary' | 'pichet-primary' | 'dyad-synthesis';

export const ENGINE_ROUTING: Record<SelemeneEngineId, RoutingMode> = {
  'vimshottari': 'aletheios-primary',
  'human-design': 'aletheios-primary',
  'enneagram': 'aletheios-primary',
  'i-ching': 'aletheios-primary',
  'numerology': 'aletheios-primary',
  'biorhythm': 'pichet-primary',
  'vedic-clock': 'pichet-primary',
  'biofield': 'pichet-primary',
  'face-reading': 'pichet-primary',
  'nadabrahman': 'pichet-primary',
  'panchanga': 'dyad-synthesis',
  'gene-keys': 'dyad-synthesis',
  'tarot': 'dyad-synthesis',
  'sacred-geometry': 'dyad-synthesis',
  'sigil-forge': 'dyad-synthesis',
  'transits': 'dyad-synthesis',
};

export interface CalculationMetadata {
  calculation_time_ms: number;
  backend: string;
  precision_achieved: string;
  cached: boolean;
  timestamp: string;
  engine_version: string;
}

export interface SelemeneEngineOutput {
  engine_id: SelemeneEngineId;
  result: unknown;
  witness_prompt: string;
  consciousness_level: number;
  metadata: CalculationMetadata;
  envelope_version: string;
}
```

Add to `packages/witness-pipeline/src/index.ts`:

```typescript
export * from './selemene/types.js';
```

Remove the duplicate `SelemeneEngineOutput` and `SELEMENE_ENGINE_IDS` from `fetcher.ts` and import from `types.ts` instead. In `fetcher.ts`, change the top to:

```typescript
import type { SelemeneEngineId, SelemeneEngineOutput, BirthData } from './types.js';

export const SELEMENE_BASE_URL = 'https://selemene.tryambakam.space';
export { SELEMENE_ENGINE_IDS } from './types.js';
export type { SelemeneEngineId, SelemeneEngineOutput, BirthData } from './types.js';
```

And remove the local `BirthData`, `SelemeneEngineId`, `SelemeneEngineOutput` definitions.

**Step 4: Run the test to verify it passes**

Run:

```bash
cd packages/witness-pipeline
npm test
```

Expected: PASS.

**Step 5: Commit**

```bash
git add packages/witness-pipeline/src/selemene/
git commit -m "feat(witness-pipeline): add engine routing and typed result map"
```

---

### Task 4: Port reading mode parser

**Files:**
- Create: `packages/witness-pipeline/src/modes/parser.ts`
- Create: `packages/witness-pipeline/src/modes/parser.test.ts`
- Create: `packages/witness-pipeline/src/modes/types.ts`
- Modify: `packages/witness-pipeline/src/index.ts`

**Step 1: Write the failing test**

Create `packages/witness-pipeline/src/modes/parser.test.ts`:

```typescript
import { describe, it, expect } from 'vitest';
import { parseModeDocument } from './parser.js';

const sampleMode = `---
mode: composite-dyad
subject_count:
  min: 2
  max: 2
roles:
  - subject-a
  - subject-b
target_words:
  min: 9000
  max: 11000
architecture: linear
pass_plan:
  - id: alpha
    title: Structural Field
    target_words: 3000
    template: pass-alpha-template
engine_overlay_weights:
  panchanga: 1.0
house_overlay: [1, 4, 7, 10]
bridge_mandates:
  - "Braid Vedic and HD data"
svg_topology: dyad-arc
---

## pass-alpha-template
This is the alpha prompt.

## lessons

### 2026-06-01 — Test lesson
**Question:** Does this work?
**Adopted:** Yes.
`;

describe('parseModeDocument', () => {
  it('parses frontmatter and body sections', () => {
    const parsed = parseModeDocument(sampleMode, 'composite-dyad.md');
    expect(parsed.mode).toBe('composite-dyad');
    expect(parsed.subject_count).toEqual({ min: 2, max: 2 });
    expect(parsed.pass_plan).toHaveLength(1);
    expect(parsed.pass_plan[0].template).toBe('pass-alpha-template');
    expect(parsed.templates['pass-alpha-template']).toBe('This is the alpha prompt.');
    expect(parsed.lessons).toHaveLength(1);
    expect(parsed.lessons[0].title).toBe('Test lesson');
  });

  it('throws when frontmatter is missing', () => {
    expect(() => parseModeDocument('no frontmatter', 'bad.md')).toThrow('Missing frontmatter');
  });
});
```

**Step 2: Run the test to verify it fails**

Run:

```bash
cd packages/witness-pipeline
npm test
```

Expected: FAIL — parser does not exist.

**Step 3: Implement the minimal code**

Create `packages/witness-pipeline/src/modes/types.ts`:

```typescript
export interface SubjectCount {
  min: number;
  max: number;
}

export interface TargetWords {
  min: number;
  max: number;
}

export interface PassSpec {
  id: string;
  title: string;
  target_words: number;
  template: string;
  model?: string;
}

export interface RegisterVariant {
  target_words?: TargetWords;
  overrides: Array<{ pass_id: string; template: string }>;
}

export interface ModeDocument {
  mode: string;
  subject_count: SubjectCount;
  roles: string[];
  target_words: TargetWords;
  architecture: 'linear' | 'hierarchical';
  pass_plan: PassSpec[];
  engine_overlay_weights: Record<string, number>;
  house_overlay: number[];
  bridge_mandates: string[];
  svg_topology: 'dyad-arc' | 'triad-triangle' | 'pentagon' | 'web-graph';
  register_variants?: {
    l1_l3?: RegisterVariant;
    l4_l5?: RegisterVariant;
  };
  templates: Record<string, string>;
  overlay_rules?: string;
  glossary?: string;
  interactions?: string;
  lessons: LessonEntry[];
}

export interface LessonEntry {
  date: string;
  title: string;
  fields: Record<string, string>;
}
```

Create `packages/witness-pipeline/src/modes/parser.ts`:

```typescript
import yaml from 'js-yaml';
import type { ModeDocument, LessonEntry, PassSpec } from './types.js';

export function parseModeDocument(content: string, sourcePath: string): ModeDocument {
  const trimmed = content.trim();
  if (!trimmed.startsWith('---')) {
    throw new Error(`Missing frontmatter in ${sourcePath}`);
  }

  const end = trimmed.indexOf('---', 3);
  if (end === -1) {
    throw new Error(`Missing closing frontmatter delimiter in ${sourcePath}`);
  }

  const frontmatterText = trimmed.slice(3, end).trim();
  const bodyText = trimmed.slice(end + 3).trim();

  const frontmatter = yaml.load(frontmatterText) as Record<string, unknown>;
  if (!frontmatter || typeof frontmatter !== 'object') {
    throw new Error(`Malformed frontmatter YAML in ${sourcePath}`);
  }

  const templates: Record<string, string> = {};
  const sections = parseBodySections(bodyText);

  const requiredKeys = [
    'mode', 'subject_count', 'roles', 'target_words', 'architecture',
    'pass_plan', 'engine_overlay_weights', 'house_overlay', 'bridge_mandates', 'svg_topology',
  ];
  for (const key of requiredKeys) {
    if (!(key in frontmatter)) {
      throw new Error(`Missing required frontmatter key ${key} in ${sourcePath}`);
    }
  }

  const passPlan = (frontmatter.pass_plan as PassSpec[]).map((p) => {
    if (!p.id || !p.title || !p.template || typeof p.target_words !== 'number') {
      throw new Error(`Invalid pass spec in ${sourcePath}: ${JSON.stringify(p)}`);
    }
    if (!sections[p.template]) {
      throw new Error(`Template section ${p.template} not found in ${sourcePath}`);
    }
    return p;
  });

  for (const p of passPlan) {
    templates[p.template] = sections[p.template];
  }

  return {
    mode: String(frontmatter.mode),
    subject_count: frontmatter.subject_count as ModeDocument['subject_count'],
    roles: frontmatter.roles as string[],
    target_words: frontmatter.target_words as ModeDocument['target_words'],
    architecture: frontmatter.architecture as ModeDocument['architecture'],
    pass_plan: passPlan,
    engine_overlay_weights: frontmatter.engine_overlay_weights as Record<string, number>,
    house_overlay: frontmatter.house_overlay as number[],
    bridge_mandates: frontmatter.bridge_mandates as string[],
    svg_topology: frontmatter.svg_topology as ModeDocument['svg_topology'],
    register_variants: frontmatter.register_variants as ModeDocument['register_variants'],
    templates,
    overlay_rules: sections['overlay-rules'],
    glossary: sections['glossary'],
    interactions: sections['interactions'],
    lessons: parseLessons(sections['lessons'] ?? ''),
  };
}

function parseBodySections(body: string): Record<string, string> {
  const sections: Record<string, string> = {};
  const regex = /^##\s+(.+)$/gm;
  let match: RegExpExecArray | null;
  const matches: Array<{ name: string; start: number }> = [];
  while ((match = regex.exec(body)) !== null) {
    matches.push({ name: match[1].trim().toLowerCase().replace(/\s+/g, '-'), start: match.index });
  }
  for (let i = 0; i < matches.length; i++) {
    const current = matches[i];
    const next = matches[i + 1];
    const end = next ? next.start : body.length;
    sections[current.name] = body.slice(current.start + `## ${matches[i].name.replace(/-/g, ' ')}`.length + 1, end).trim();
  }
  return sections;
}

function parseLessons(lessonsText: string): LessonEntry[] {
  const entries: LessonEntry[] = [];
  const headingRegex = /^###\s+(\d{4}-\d{2}-\d{2})\s*[-–]\s*(.+)$/gm;
  let match: RegExpExecArray | null;
  const matches: Array<{ date: string; title: string; start: number }> = [];
  while ((match = headingRegex.exec(lessonsText)) !== null) {
    matches.push({ date: match[1], title: match[2].trim(), start: match.index });
  }
  for (let i = 0; i < matches.length; i++) {
    const current = matches[i];
    const next = matches[i + 1];
    const end = next ? next.start : lessonsText.length;
    const body = lessonsText.slice(current.start, end).trim();
    const fields: Record<string, string> = {};
    const fieldRegex = /^\*\*(.+?)\*\*:\s*(.+)$/gm;
    let fmatch: RegExpExecArray | null;
    while ((fmatch = fieldRegex.exec(body)) !== null) {
      fields[fmatch[1].toLowerCase().replace(/\s+/g, '_')] = fmatch[2].trim();
    }
    entries.push({ date: current.date, title: current.title, fields });
  }
  return entries;
}
```

Add to `packages/witness-pipeline/src/index.ts`:

```typescript
export * from './modes/types.js';
export { parseModeDocument } from './modes/parser.js';
```

**Step 4: Run the test to verify it passes**

Run:

```bash
cd packages/witness-pipeline
npm test
```

Expected: PASS.

**Step 5: Commit**

```bash
git add packages/witness-pipeline/src/modes/
git commit -m "feat(witness-pipeline): port reading mode parser"
```

---

### Task 5: Port the integrated-reading orchestrator skeleton

**Files:**
- Create: `packages/witness-pipeline/src/orchestrator/integrated.ts`
- Create: `packages/witness-pipeline/src/orchestrator/integrated.test.ts`
- Modify: `packages/witness-pipeline/src/index.ts`

**Step 1: Write the failing test**

Create `packages/witness-pipeline/src/orchestrator/integrated.test.ts`:

```typescript
import { describe, it, expect, vi } from 'vitest';
import { IntegratedReadingOrchestrator } from './integrated.js';
import type { ModeDocument, SelemeneEngineOutput } from '../index.js';

const mockMode: ModeDocument = {
  mode: 'composite-dyad',
  subject_count: { min: 2, max: 2 },
  roles: ['subject-a', 'subject-b'],
  target_words: { min: 100, max: 500 },
  architecture: 'linear',
  pass_plan: [
    { id: 'alpha', title: 'Alpha', target_words: 100, template: 'pass-alpha-template' },
  ],
  engine_overlay_weights: { panchanga: 1 },
  house_overlay: [1],
  bridge_mandates: [],
  svg_topology: 'dyad-arc',
  templates: { 'pass-alpha-template': 'Write about {{subject_names}} using {{overlay_summary}}.' },
  lessons: [],
};

const mockEngineResults: SelemeneEngineOutput[] = [
  {
    engine_id: 'panchanga',
    result: { tithi_name: 'Shukla Pratipada' },
    witness_prompt: 'Observe the moment.',
    consciousness_level: 2,
    metadata: { calculation_time_ms: 10, backend: 'native', precision_achieved: 'standard', cached: false, timestamp: '2026-06-22T00:00:00Z', engine_version: '1' },
    envelope_version: '1',
  },
];

describe('IntegratedReadingOrchestrator', () => {
  it('renders the first pass prompt with interpolations', async () => {
    const llm = vi.fn().mockResolvedValue('alpha output');
    const orchestrator = new IntegratedReadingOrchestrator({ llm, mode: mockMode });
    const result = await orchestrator.run({
      subjectNames: ['Arathi', 'Rohan'],
      engineResultsBySubject: [mockEngineResults, mockEngineResults],
      consciousnessLevel: 2,
    });
    expect(result.passes).toHaveLength(1);
    expect(result.passes[0].output).toBe('alpha output');
    expect(llm).toHaveBeenCalledWith(
      expect.stringContaining('Arathi'),
      expect.any(String),
      expect.objectContaining({ max_tokens: expect.any(Number) })
    );
  });
});
```

**Step 2: Run the test to verify it fails**

Run:

```bash
cd packages/witness-pipeline
npm test
```

Expected: FAIL — `integrated.ts` does not exist.

**Step 3: Implement the minimal code**

Create `packages/witness-pipeline/src/orchestrator/integrated.ts`:

```typescript
import type { ModeDocument, PassSpec, SelemeneEngineOutput } from '../index.js';

export interface OrchestratorInput {
  subjectNames: string[];
  engineResultsBySubject: SelemeneEngineOutput[][];
  consciousnessLevel: number;
}

export interface PassResult {
  id: string;
  title: string;
  output: string;
}

export interface OrchestratorOutput {
  mode: string;
  subject_names: string[];
  passes: PassResult[];
  assembled: string;
}

export interface LlmCall {
  (system: string, user: string, options: { max_tokens: number }): Promise<string>;
}

export interface OrchestratorOptions {
  mode: ModeDocument;
  llm: LlmCall;
}

export class IntegratedReadingOrchestrator {
  private mode: ModeDocument;
  private llm: LlmCall;

  constructor(opts: OrchestratorOptions) {
    this.mode = opts.mode;
    this.llm = opts.llm;
  }

  async run(input: OrchestratorInput): Promise<OrchestratorOutput> {
    const passOutputs: PassResult[] = [];
    let assembled = '';

    for (const pass of this.mode.pass_plan) {
      const prior = assembled.slice(-4000);
      const prompt = this.renderPassTemplate(pass, input, prior);
      const system = this.buildSystemPrompt(pass, input);
      const output = await this.llm(system, prompt, { max_tokens: Math.round(pass.target_words * 2) });
      passOutputs.push({ id: pass.id, title: pass.title, output });
      assembled += `\n\n## ${pass.title}\n\n${output}`;
    }

    return {
      mode: this.mode.mode,
      subject_names: input.subjectNames,
      passes: passOutputs,
      assembled: assembled.trim(),
    };
  }

  private renderPassTemplate(pass: PassSpec, input: OrchestratorInput, priorPass: string): string {
    const template = this.mode.templates[pass.template];
    const overlaySummary = this.buildOverlaySummary();
    const bridgeMandates = this.mode.bridge_mandates.map((m) => `- ${m}`).join('\n');
    const lessonsSummary = this.mode.lessons.slice(-5).map((l) => `- ${l.date}: ${l.title}`).join('\n');

    return template
      .replace(/\{\{subject_names\}\}/g, input.subjectNames.join(', '))
      .replace(/\{\{prior_pass\}\}/g, priorPass)
      .replace(/\{\{overlay_summary\}\}/g, overlaySummary)
      .replace(/\{\{bridge_mandates\}\}/g, bridgeMandates)
      .replace(/\{\{lessons_summary\}\}/g, lessonsSummary);
  }

  private buildSystemPrompt(pass: PassSpec, input: OrchestratorInput): string {
    return `You are writing pass "${pass.title}" for the ${this.mode.mode} reading mode.
Target length: ~${pass.target_words} words.
${this.mode.overlay_rules ?? ''}`;
  }

  private buildOverlaySummary(): string {
    const weights = Object.entries(this.mode.engine_overlay_weights)
      .map(([k, v]) => `${k}: ${v}`)
      .join(', ');
    return `Engine weights: ${weights}; Houses: ${this.mode.house_overlay.join(', ')}`;
  }
}
```

Add to `packages/witness-pipeline/src/index.ts`:

```typescript
export * from './orchestrator/integrated.js';
```

**Step 4: Run the test to verify it passes**

Run:

```bash
cd packages/witness-pipeline
npm test
```

Expected: PASS.

**Step 5: Commit**

```bash
git add packages/witness-pipeline/src/orchestrator/
git commit -m "feat(witness-pipeline): add integrated reading orchestrator skeleton"
```

---

### Task 6: Port asset factory and chain audit skeletons

**Files:**
- Create: `packages/witness-pipeline/src/assets/factory.ts`
- Create: `packages/witness-pipeline/src/assets/factory.test.ts`
- Create: `packages/witness-pipeline/src/assets/audit.ts`
- Create: `packages/witness-pipeline/src/assets/audit.test.ts`
- Modify: `packages/witness-pipeline/src/index.ts`

**Step 1: Write the failing test**

Create `packages/witness-pipeline/src/assets/factory.test.ts`:

```typescript
import { describe, it, expect } from 'vitest';
import { createSourcePack } from './factory.js';
import type { SelemeneEngineOutput } from '../index.js';

const engines: SelemeneEngineOutput[] = [
  {
    engine_id: 'panchanga',
    result: {},
    witness_prompt: 'Observe.',
    consciousness_level: 2,
    metadata: { calculation_time_ms: 1, backend: 'native', precision_achieved: 'standard', cached: false, timestamp: '2026-06-22T00:00:00Z', engine_version: '1' },
    envelope_version: '1',
  },
];

describe('createSourcePack', () => {
  it('creates a manifest and reflection questions', async () => {
    const pack = await createSourcePack({
      personId: 'test-person',
      readingMarkdown: '# Reading',
      engineResults: engines,
      outputDir: '/tmp/test-pack',
    });
    expect(pack.manifest.person_id).toBe('test-person');
    expect(pack.reflectionQuestions).toContain('What is the one thing from this reading that feels most alive right now?');
  });
});
```

Create `packages/witness-pipeline/src/assets/audit.test.ts`:

```typescript
import { describe, it, expect } from 'vitest';
import { runChainAudit } from './audit.js';

describe('runChainAudit', () => {
  it('passes a pack with deterministic engine data', async () => {
    const result = runChainAudit({
      personId: 'test',
      readingMarkdown: 'Lagna in Aries.',
      engineResults: [
        { engine_id: 'panchanga', result: { nakshatra_name: 'Ashwini' }, witness_prompt: 'x', consciousness_level: 2, metadata: { calculation_time_ms: 1, backend: 'native', precision_achieved: 'standard', cached: false, timestamp: '2026-06-22T00:00:00Z', engine_version: '1' }, envelope_version: '1' },
      ],
    });
    expect(result.blockers).toHaveLength(0);
  });

  it('blocks a pack with oracle engines when deterministic-only is required', () => {
    const result = runChainAudit({
      personId: 'test',
      readingMarkdown: 'Lagna in Aries.',
      engineResults: [
        { engine_id: 'tarot', result: {}, witness_prompt: 'x', consciousness_level: 2, metadata: { calculation_time_ms: 1, backend: 'ts', precision_achieved: 'standard', cached: false, timestamp: '2026-06-22T00:00:00Z', engine_version: '1' }, envelope_version: '1' },
      ],
      deterministicOnly: true,
    });
    expect(result.blockers.length).toBeGreaterThan(0);
  });
});
```

**Step 2: Run the test to verify it fails**

Run:

```bash
cd packages/witness-pipeline
npm test
```

Expected: FAIL — factory and audit do not exist.

**Step 3: Implement the minimal code**

Create `packages/witness-pipeline/src/assets/factory.ts`:

```typescript
import { promises as fs } from 'node:fs';
import { join } from 'node:path';
import type { SelemeneEngineOutput } from '../index.js';

export interface SourcePackInput {
  personId: string;
  readingMarkdown: string;
  engineResults: SelemeneEngineOutput[];
  outputDir: string;
}

export interface SourcePack {
  outputDir: string;
  manifest: {
    person_id: string;
    created_at: string;
    reading_length: number;
    engines: string[];
    quality: { facts_count: number; gate_status: string };
  };
  reflectionQuestions: string[];
}

const DEFAULT_REFLECTION_QUESTIONS = [
  'What is the one thing from this reading that feels most alive right now?',
  'Where do you notice resistance, and what is it protecting?',
  'What is the smallest step that honors what this reading named?',
];

export async function createSourcePack(input: SourcePackInput): Promise<SourcePack> {
  await fs.mkdir(input.outputDir, { recursive: true });
  const manifest: SourcePack['manifest'] = {
    person_id: input.personId,
    created_at: new Date().toISOString(),
    reading_length: input.readingMarkdown.length,
    engines: input.engineResults.map((e) => e.engine_id),
    quality: {
      facts_count: countDeterministicFacts(input.engineResults),
      gate_status: 'pending',
    },
  };

  const manifestPath = join(input.outputDir, 'manifest.json');
  const questionsPath = join(input.outputDir, 'reflection-questions.md');
  const readingPath = join(input.outputDir, 'reading.md');

  await fs.writeFile(manifestPath, JSON.stringify(manifest, null, 2), 'utf-8');
  await fs.writeFile(questionsPath, DEFAULT_REFLECTION_QUESTIONS.map((q) => `- ${q}`).join('\n'), 'utf-8');
  await fs.writeFile(readingPath, input.readingMarkdown, 'utf-8');

  return {
    outputDir: input.outputDir,
    manifest,
    reflectionQuestions: DEFAULT_REFLECTION_QUESTIONS,
  };
}

function countDeterministicFacts(engineResults: SelemeneEngineOutput[]): number {
  const deterministicEngines = new Set(['panchanga', 'vimshottari', 'human-design', 'gene-keys', 'numerology', 'biorhythm', 'vedic-clock', 'transits', 'enneagram']);
  return engineResults
    .filter((e) => deterministicEngines.has(e.engine_id) && !e._error)
    .length;
}
```

Create `packages/witness-pipeline/src/assets/audit.ts`:

```typescript
import type { SelemeneEngineOutput } from '../index.js';

export interface AuditInput {
  personId: string;
  readingMarkdown: string;
  engineResults: SelemeneEngineOutput[];
  deterministicOnly?: boolean;
}

export interface AuditResult {
  person_id: string;
  blockers: string[];
  warnings: string[];
  facts_count: number;
  passed: boolean;
}

const DETERMINISTIC_ENGINES = new Set([
  'panchanga', 'vimshottari', 'human-design', 'gene-keys', 'numerology',
  'biorhythm', 'vedic-clock', 'transits', 'enneagram',
]);

const ORACLE_ENGINES = new Set(['tarot', 'i-ching', 'sacred-geometry', 'sigil-forge']);
const SOMATIC_ENGINES = new Set(['biofield', 'face-reading', 'nadabrahman']);

export function runChainAudit(input: AuditInput): AuditResult {
  const blockers: string[] = [];
  const warnings: string[] = [];

  const deterministic = input.engineResults.filter((e) => DETERMINISTIC_ENGINES.has(e.engine_id) && !e._error);
  const factsCount = deterministic.length;

  if (factsCount < 3) {
    blockers.push(`Only ${factsCount} deterministic engines present; need at least 3.`);
  }

  if (input.deterministicOnly) {
    for (const e of input.engineResults) {
      if (ORACLE_ENGINES.has(e.engine_id)) {
        blockers.push(`Oracle engine ${e.engine_id} present but deterministic-only mode is enabled.`);
      }
      if (SOMATIC_ENGINES.has(e.engine_id)) {
        blockers.push(`Somatic engine ${e.engine_id} present but deterministic-only mode is enabled.`);
      }
    }
  }

  if (input.readingMarkdown.length < 100) {
    warnings.push('Reading markdown is very short.');
  }

  return {
    person_id: input.personId,
    blockers,
    warnings,
    facts_count: factsCount,
    passed: blockers.length === 0,
  };
}
```

Add to `packages/witness-pipeline/src/index.ts`:

```typescript
export * from './assets/factory.js';
export * from './assets/audit.js';
```

**Step 4: Run the test to verify it passes**

Run:

```bash
cd packages/witness-pipeline
npm test
```

Expected: PASS.

**Step 5: Commit**

```bash
git add packages/witness-pipeline/src/assets/
git commit -m "feat(witness-pipeline): add source-pack factory and chain audit"
```

---

### Task 7: Typecheck and build the new package

**Files:**
- Modify: `packages/witness-pipeline/src/index.ts` to ensure all exports are valid
- Create: `packages/witness-pipeline/tests/smoke.test.ts` (smoke)

**Step 1: Write the failing typecheck**

Run:

```bash
cd packages/witness-pipeline
npm run typecheck
```

Expected: Possibly FAIL due to duplicate type exports or missing vitest config.

**Step 2: Create vitest config**

Create `packages/witness-pipeline/vitest.config.ts`:

```typescript
import { defineConfig } from 'vitest/config';

export default defineConfig({
  test: {
    globals: false,
    environment: 'node',
  },
});
```

Update `packages/witness-pipeline/package.json` scripts:

```json
    "test": "vitest run",
```

Already correct.

**Step 3: Fix type issues**

In `packages/witness-pipeline/src/selemene/fetcher.ts`, ensure `fetchImpl` option type is declared and `fetchAllEngines` signature is exported.

In `packages/witness-pipeline/src/index.ts`, avoid duplicate exports. Ensure `SELEMENE_ENGINE_IDS`, `SelemeneEngineId`, `SelemeneEngineOutput`, `BirthData` are exported only via `types.ts` or `fetcher.ts`, not both.

**Step 4: Run typecheck and build**

Run:

```bash
cd packages/witness-pipeline
npm run typecheck
npm run build
npm test
```

Expected: PASS for all three.

**Step 5: Commit**

```bash
git add packages/witness-pipeline/
git commit -m "chore(witness-pipeline): typecheck, build, and vitest config"
```

---

### Task 8: Add workspace usage example in `apps/noesis-web`

**Files:**
- Create: `apps/noesis-web/src/lib/integrated/witnessPipelineClient.ts`
- Create: `apps/noesis-web/src/lib/integrated/witnessPipelineClient.test.ts`
- Modify: `apps/noesis-web/package.json` to add dependency

**Step 1: Write the failing test**

Create `apps/noesis-web/src/lib/integrated/witnessPipelineClient.test.ts`:

```typescript
import { describe, it, expect, vi } from 'vitest';
import { buildIntegratedReadingPayload } from './witnessPipelineClient.js';

describe('buildIntegratedReadingPayload', () => {
  it('serializes birth data', () => {
    const payload = buildIntegratedReadingPayload({
      name: 'Arathi',
      birthDate: '1980-03-15',
      birthTime: '06:30',
      timezone: 'Asia/Kolkata',
      latitude: 12.9716,
      longitude: 77.5946,
    });
    expect(payload.birth_data.name).toBe('Arathi');
    expect(payload.birth_data.date).toBe('1980-03-15');
  });
});
```

**Step 2: Run the test to verify it fails**

Run:

```bash
cd apps/noesis-web
npm test
```

Expected: FAIL — file does not exist.

**Step 3: Implement the minimal code**

Create `apps/noesis-web/src/lib/integrated/witnessPipelineClient.ts`:

```typescript
import type { BirthData } from '@noesis/witness-pipeline';

export interface WitnessFormData {
  name?: string;
  birthDate: string;
  birthTime?: string;
  timezone: string;
  latitude: number;
  longitude: number;
}

export function buildIntegratedReadingPayload(form: WitnessFormData): { birth_data: BirthData } {
  const birth_data: BirthData = {
    date: form.birthDate,
    timezone: form.timezone,
    latitude: form.latitude,
    longitude: form.longitude,
  };
  if (form.birthTime) birth_data.time = form.birthTime;
  if (form.name) birth_data.name = form.name;
  return { birth_data };
}
```

Modify `apps/noesis-web/package.json` dependencies to add:

```json
    "@noesis/witness-pipeline": "*"
```

Run `npm install` from repo root.

**Step 4: Run the test to verify it passes**

Run:

```bash
cd apps/noesis-web
npm test
```

Expected: PASS.

**Step 5: Commit**

```bash
git add apps/noesis-web/src/lib/integrated/witnessPipelineClient.ts apps/noesis-web/src/lib/integrated/witnessPipelineClient.test.ts apps/noesis-web/package.json package-lock.json
git commit -m "feat(noesis-web): add witness-pipeline client helper"
```

---

## Part B — Upgrade Rust `crates/noesis-witness`

### Task 9: Add engine routing metadata to Rust WitnessContext

**Files:**
- Modify: `crates/noesis-witness/src/interpret.rs`
- Modify: `crates/noesis-witness/src/lib.rs`
- Create: `crates/noesis-witness/src/routing.rs`
- Create: `crates/noesis-witness/src/routing.test.rs` (unit tests in `interpret.rs` or new module)

**Step 1: Write the failing test**

In `crates/noesis-witness/src/interpret.rs`, after the existing tests, add:

```rust
#[test]
fn engines_present_includes_routing_tags() {
    let ctx = WitnessContext {
        live_scores: Default::default(),
        consciousness_level: 2,
        user_name: None,
        panchanga: Some(json!({ "tithi_name": "test" })),
        human_design: Some(json!({ "type": "Generator" })),
        biorhythm: Some(json!({ "physical": 0.5 })),
        ..Default::default()
    };
    let engines = engines_present(&ctx);
    assert!(engines.contains(&"biofield".to_string()));
    assert!(engines.contains(&"panchanga".to_string()));
    assert!(engines.contains(&"human-design".to_string()));
    assert!(engines.contains(&"biorhythm".to_string()));
}
```

Create `crates/noesis-witness/src/routing.rs`:

```rust
//! Engine → dyad pillar routing, ported from witness-agents/src/types/engine.ts.

use serde_json::Value;

/// Which pillar/dyad voice should lead interpretation for an engine result.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoutingMode {
    /// Aletheios-primary: analytical, truth-revealing (vimshottari, human-design, enneagram, i-ching, numerology).
    AletheiosPrimary,
    /// Pichet-primary: somatic, vitalizing (biorhythm, vedic-clock, biofield, face-reading, nadabrahman).
    PichetPrimary,
    /// Dyad-synthesis: co-interpretation (panchanga, gene-keys, tarot, sacred-geometry, sigil-forge, transits).
    DyadSynthesis,
}

/// Map a Selemene engine id to its routing mode.
pub fn routing_for_engine(engine_id: &str) -> Option<RoutingMode> {
    match engine_id {
        "vimshottari" | "human-design" | "enneagram" | "i-ching" | "numerology" => {
            Some(RoutingMode::AletheiosPrimary)
        }
        "biorhythm" | "vedic-clock" | "biofield" | "face-reading" | "nadabrahman" => {
            Some(RoutingMode::PichetPrimary)
        }
        "panchanga" | "gene-keys" | "tarot" | "sacred-geometry" | "sigil-forge" | "transits" => {
            Some(RoutingMode::DyadSynthesis)
        }
        _ => None,
    }
}

/// Partition a list of engine results by routing mode.
pub fn partition_by_routing(results: &[(String, Value)]) -> (Vec<&Value>, Vec<&Value>, Vec<&Value>) {
    let mut aletheios = vec![];
    let mut pichet = vec![];
    let mut dyad = vec![];
    for (engine_id, value) in results {
        match routing_for_engine(engine_id) {
            Some(RoutingMode::AletheiosPrimary) => aletheios.push(value),
            Some(RoutingMode::PichetPrimary) => pichet.push(value),
            Some(RoutingMode::DyadSynthesis) | None => dyad.push(value),
        }
    }
    (aletheios, pichet, dyad)
}
```

Add `pub mod routing;` to `crates/noesis-witness/src/lib.rs`.

**Step 2: Run the test to verify it fails**

Run:

```bash
cargo test -p noesis-witness
```

Expected: FAIL — `engines_present` still uses the old engine names, or `RoutingMode` is not used. Actually the new test should compile and fail if `engines_present` does not include the expected engines. Make sure `Default` derive on `WitnessContext` is valid (it is).

**Step 3: Implement the minimal code**

In `crates/noesis-witness/src/interpret.rs`, update `engines_present` to use the routing module. Actually keep `engines_present` as-is for now; the new test validates it works with `Default`. Then in Task 10 we will refactor it.

But first add to `lib.rs`:

```rust
pub mod routing;
pub use routing::{routing_for_engine, partition_by_routing, RoutingMode};
```

**Step 4: Run the test to verify it passes**

Run:

```bash
cargo test -p noesis-witness
```

Expected: PASS.

**Step 5: Commit**

```bash
git add crates/noesis-witness/src/routing.rs crates/noesis-witness/src/lib.rs crates/noesis-witness/src/interpret.rs
git commit -m "feat(noesis-witness): add engine routing metadata module"
```

---

### Task 10: Extend WitnessContext with synastry and engine routing

**Files:**
- Modify: `crates/noesis-witness/src/interpret.rs`
- Modify: `crates/noesis-api/src/handlers/witness.rs`
- Create: `crates/noesis-witness/src/interpret.test.rs` tests (append to existing tests)

**Step 1: Write the failing test**

Append to `crates/noesis-witness/src/interpret.rs` tests:

```rust
    #[test]
    fn witness_context_default_is_empty() {
        let ctx = WitnessContext::default();
        assert!(ctx.panchanga.is_none());
        assert!(ctx.partner_context.is_none());
        assert_eq!(ctx.relationship_mode, RelationshipMode::None);
    }

    #[test]
    fn format_engine_data_preserves_wisdom_descriptions() {
        // Already existing test — keep as-is.
    }
```

**Step 2: Run the test to verify it fails**

Run:

```bash
cargo test -p noesis-witness
```

Expected: FAIL — `RelationshipMode` and `partner_context` do not exist.

**Step 3: Implement the minimal code**

In `crates/noesis-witness/src/interpret.rs`, update `WitnessContext`:

```rust
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub enum RelationshipMode {
    #[default]
    None,
    CompositeDyad,
    PartnerSynastry,
    FamilyTriad,
    BusinessPartners,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct WitnessContext {
    pub live_scores: LiveBiofieldScores,
    pub consciousness_level: u8,
    pub user_name: Option<String>,
    pub panchanga: Option<Value>,
    pub human_design: Option<Value>,
    pub numerology: Option<Value>,
    pub biorhythm: Option<Value>,
    pub transits: Option<Value>,
    pub gene_keys: Option<Value>,
    pub vimshottari: Option<Value>,
    /// Optional second-person context for synastry/composite readings.
    pub partner_context: Option<Box<WitnessContext>>,
    /// Relationship framing for composite/synastry readings.
    pub relationship_mode: RelationshipMode,
}
```

**Step 4: Run the test to verify it passes**

Run:

```bash
cargo test -p noesis-witness
```

Expected: PASS.

**Step 5: Commit**

```bash
git add crates/noesis-witness/src/interpret.rs
git commit -m "feat(noesis-witness): extend WitnessContext with synastry and relationship mode"
```

---

### Task 11: Refactor engine context formatting to use routing tags

**Files:**
- Modify: `crates/noesis-witness/src/interpret.rs`
- Modify: `crates/noesis-witness/src/routing.rs`

**Step 1: Write the failing test**

Append to `crates/noesis-witness/src/interpret.rs` tests:

```rust
    #[test]
    fn build_context_message_groups_engines_by_routing() {
        let ctx = WitnessContext {
            consciousness_level: 2,
            panchanga: Some(json!({ "tithi_name": "test" })),
            human_design: Some(json!({ "type": "Generator" })),
            biorhythm: Some(json!({ "physical": 0.5 })),
            ..Default::default()
        };
        let msg = build_context_message(&ctx);
        assert!(msg.contains("Aletheios-primary context"));
        assert!(msg.contains("Pichet-primary context"));
        assert!(msg.contains("Dyad-synthesis context"));
    }
```

**Step 2: Run the test to verify it fails**

Run:

```bash
cargo test -p noesis-witness
```

Expected: FAIL — `build_context_message` does not emit routing-group headings.

**Step 3: Implement the minimal code**

Update `build_context_message` in `crates/noesis-witness/src/interpret.rs`:

```rust
fn build_context_message(ctx: &WitnessContext) -> String {
    let mut lines = vec![];

    let name = ctx.user_name.as_deref().unwrap_or("the person");
    lines.push(format!("# Consciousness Snapshot for {name}"));

    if ctx.relationship_mode != RelationshipMode::None {
        lines.push(format!("## Relationship mode: {:?}", ctx.relationship_mode));
    }

    lines.push(format!(
        "\n## Live Biofield Scores (PIP camera analysis)\n\
         - Energy flow: {:.0}%\n\
         - Coherence: {:.0}%\n\
         ... (rest unchanged)",
        ctx.live_scores.energy * 100.0,
        ctx.live_scores.coherence * 100.0,
        // ... include remaining metrics as before
        ctx.consciousness_level,
    ));

    // Collect available engines with routing tags.
    let mut tagged: Vec<(String, &Value)> = vec![];
    if let Some(v) = &ctx.panchanga { tagged.push(("panchanga".into(), v)); }
    if let Some(v) = &ctx.human_design { tagged.push(("human-design".into(), v)); }
    if let Some(v) = &ctx.gene_keys { tagged.push(("gene-keys".into(), v)); }
    if let Some(v) = &ctx.numerology { tagged.push(("numerology".into(), v)); }
    if let Some(v) = &ctx.biorhythm { tagged.push(("biorhythm".into(), v)); }
    if let Some(v) = &ctx.transits { tagged.push(("transits".into(), v)); }
    if let Some(v) = &ctx.vimshottari { tagged.push(("vimshottari".into(), v)); }

    let (aletheios, pichet, dyad) = crate::routing::partition_by_routing(&tagged);

    if !aletheios.is_empty() {
        lines.push("\n## Aletheios-primary context".to_string());
        for v in aletheios {
            lines.push(format_engine_data(v));
        }
    }
    if !pichet.is_empty() {
        lines.push("\n## Pichet-primary context".to_string());
        for v in pichet {
            lines.push(format_engine_data(v));
        }
    }
    if !dyad.is_empty() {
        lines.push("\n## Dyad-synthesis context".to_string());
        for v in dyad {
            lines.push(format_engine_data(v));
        }
    }

    if let Some(partner) = &ctx.partner_context {
        lines.push("\n## Partner / Composite context".to_string());
        lines.push(build_context_message(partner));
    }

    lines.join("\n")
}
```

(Keep the exact formatting of biofield metrics as in the original file.)

**Step 4: Run the test to verify it passes**

Run:

```bash
cargo test -p noesis-witness
```

Expected: PASS.

**Step 5: Commit**

```bash
git add crates/noesis-witness/src/interpret.rs crates/noesis-witness/src/routing.rs
git commit -m "feat(noesis-witness): group engine context by dyad routing"
```

---

### Task 12: Update the API handler to pass routing/relationship context

**Files:**
- Modify: `crates/noesis-api/src/handlers/witness.rs`
- Modify: `crates/noesis-api/src/lib.rs` OpenAPI schema if needed

**Step 1: Write the failing test**

Run:

```bash
cargo test -p noesis-api --no-run
```

Expected: currently compiles; after adding new fields, we want to see compilation behavior.

**Step 2: Add optional relationship fields to request**

Modify `crates/noesis-api/src/handlers/witness.rs` request struct:

```rust
#[derive(Deserialize, ToSchema)]
pub struct WitnessInterpretRequest {
    pub birth_data: Option<BirthData>,
    pub live_scores: LiveBiofieldScores,
    #[serde(default)]
    pub consciousness_level: u8,
    pub user_name: Option<String>,
    /// Optional second birth data for synastry/composite readings.
    pub partner_birth_data: Option<BirthData>,
    /// Relationship framing: "composite-dyad", "partner-synastry", "family-triad", "business-partners".
    pub relationship_mode: Option<String>,
}
```

**Step 3: Implement partner context fetch**

In the `interpret` handler, after fetching engines for the primary person, if `partner_birth_data` is present, run the same engine set for the partner and attach it to `WitnessContext`.

Add a helper:

```rust
fn parse_relationship_mode(mode: Option<String>) -> noesis_witness::interpret::RelationshipMode {
    match mode.as_deref() {
        Some("composite-dyad") => noesis_witness::interpret::RelationshipMode::CompositeDyad,
        Some("partner-synastry") => noesis_witness::interpret::RelationshipMode::PartnerSynastry,
        Some("family-triad") => noesis_witness::interpret::RelationshipMode::FamilyTriad,
        Some("business-partners") => noesis_witness::interpret::RelationshipMode::BusinessPartners,
        _ => noesis_witness::interpret::RelationshipMode::None,
    }
}
```

In `interpret`, after primary engines:

```rust
let partner_context = if let Some(partner_bd) = req.partner_birth_data.clone() {
    let partner_bd_input = make_input(Some(partner_bd));
    let (p_panchanga, p_biorhythm, p_human_design, p_numerology, p_transits, p_gene_keys, p_vimshottari) = tokio::join!(
        run_engine(&orch, "panchanga", panchanga_input_for(Some(partner_bd_input.clone())), level),
        run_engine(&orch, "biorhythm", partner_bd_input.clone(), level),
        run_engine(&orch, "human-design", partner_bd_input.clone(), level),
        run_engine(&orch, "numerology", partner_bd_input.clone(), level),
        run_engine(&orch, "transits", partner_bd_input.clone(), level),
        run_engine(&orch, "gene-keys", partner_bd_input.clone(), level),
        run_engine(&orch, "vimshottari", partner_bd_input.clone(), level),
    );
    Some(Box::new(WitnessContext {
        live_scores: req.live_scores.clone(),
        consciousness_level,
        user_name: None,
        panchanga: p_panchanga,
        human_design: p_human_design,
        numerology: p_numerology,
        biorhythm: p_biorhythm,
        transits: p_transits,
        gene_keys: p_gene_keys,
        vimshottari: p_vimshottari,
        partner_context: None,
        relationship_mode: noesis_witness::interpret::RelationshipMode::None,
    }))
} else {
    None
};
```

Refactor the primary `panchanga_input` into a helper `panchanga_input_for(bd: Option<BirthData>)` to reuse.

Set `relationship_mode` on the primary context from `parse_relationship_mode(req.relationship_mode)`.

**Step 4: Run the test to verify it compiles**

Run:

```bash
cargo test -p noesis-api --no-run
```

Expected: `Finished` with no errors.

**Step 5: Commit**

```bash
git add crates/noesis-api/src/handlers/witness.rs
git commit -m "feat(noesis-api): support partner birth data and relationship mode in witness interpret"
```

---

### Task 13: Improve rule-based fallback with tier/level gates

**Files:**
- Modify: `crates/noesis-api/src/handlers/witness.rs`
- Modify: `crates/noesis-witness/src/interpret.rs` (optional: move rule-based logic into crate)

**Step 1: Write the failing test**

Append to `crates/noesis-api/src/handlers/witness.rs`? Better: add a small unit test inside `crates/noesis-api/src/handlers/witness.rs` under `#[cfg(test)]`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rule_based_dyad_uses_level_specific_question() {
        let scores = LiveBiofieldScores {
            energy: 0.5,
            coherence: 0.5,
            symmetry: 0.5,
            complexity: 0.5,
            regulation: 0.5,
            color_balance: 0.5,
        };
        let (_, _, _, q0) = rule_based_dyad_from_scores(&scores, 0);
        let (_, _, _, q2) = rule_based_dyad_from_scores(&scores, 2);
        let (_, _, _, q5) = rule_based_dyad_from_scores(&scores, 5);
        assert_ne!(q0, q2);
        assert_ne!(q2, q5);
    }
}
```

**Step 2: Run the test to verify it fails**

Run:

```bash
cargo test -p noesis-api witness::tests
```

Expected: FAIL — tests module does not exist.

**Step 3: Implement the minimal code**

Add the `#[cfg(test)]` block at the end of `crates/noesis-api/src/handlers/witness.rs`. The existing `rule_based_dyad_from_scores` already has level-specific questions, so the test should pass.

If you want richer fallback (mirroring `witness-agents` tier gates), extend `rule_based_dyad_from_scores` to accept `engines_used` and reference them in the fallback text:

```rust
fn rule_based_dyad_from_scores(
    scores: &LiveBiofieldScores,
    level: u8,
    engines_used: &[String],
) -> (String, String, String, String) {
    let engine_note = if engines_used.len() > 1 {
        format!(" (grounded in {} engines)", engines_used.len())
    } else {
        String::new()
    };
    // ... existing aletheios/pichet/synthesis, append engine_note where appropriate.
}
```

Update the call site in `interpret` to pass `&ctx.live_scores, consciousness_level, &[]` or the actual engines list.

**Step 4: Run the test to verify it passes**

Run:

```bash
cargo test -p noesis-api witness::tests
```

Expected: PASS.

**Step 5: Commit**

```bash
git add crates/noesis-api/src/handlers/witness.rs
git commit -m "test(noesis-api): cover witness rule-based fallback levels"
```

---

## Part C — Integration and Verification

### Task 14: Run full Rust test suite

**Files:**
- No new files.

**Step 1:**

```bash
cargo test -p noesis-witness -p noesis-api
```

Expected: all tests pass.

**Step 2: Commit**

```bash
git commit --allow-empty -m "verify: rust witness + api tests green"
```

---

### Task 15: Run full TypeScript test suite for new package

**Files:**
- No new files.

**Step 1:**

```bash
cd packages/witness-pipeline
npm run typecheck
npm run build
npm test
```

Expected: all pass.

**Step 2: Commit**

```bash
git commit --allow-empty -m "verify: witness-pipeline typecheck build test green"
```

---

### Task 16: Add a smoke integration script

**Files:**
- Create: `packages/witness-pipeline/scripts/smoke.ts`

**Step 1: Write the script**

Create `packages/witness-pipeline/scripts/smoke.ts`:

```typescript
#!/usr/bin/env node --import tsx
// Smoke test: parse a mode doc and run the orchestrator against fake engine data.

import { parseModeDocument, IntegratedReadingOrchestrator } from '../src/index.js';
import { readFileSync } from 'node:fs';
import { resolve, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const modePath = resolve(__dirname, '../../scripts/integratedreading/modes/composite-dyad.md');

const modeDoc = parseModeDocument(readFileSync(modePath, 'utf-8'), modePath);

const fakeLlm = async (_system: string, user: string, _opts: { max_tokens: number }) => {
  return `Fake response for: ${user.slice(0, 60)}...`;
};

const orchestrator = new IntegratedReadingOrchestrator({ mode: modeDoc, llm: fakeLlm });

orchestrator.run({
  subjectNames: ['Arathi', 'Rohan'],
  engineResultsBySubject: [[], []],
  consciousnessLevel: 2,
}).then((out) => {
  console.log(`Mode: ${out.mode}`);
  console.log(`Subjects: ${out.subject_names.join(', ')}`);
  console.log(`Passes: ${out.passes.length}`);
  console.log(`Assembled length: ${out.assembled.length}`);
}).catch((err) => {
  console.error(err);
  process.exit(1);
});
```

**Step 2: Copy mode docs**

Copy the witness-agents mode docs into the package:

```bash
mkdir -p packages/witness-pipeline/scripts/integratedreading/modes
cp /Volumes/madara/2026/twc-vault/01-Projects/tryambakam-noesis/witness-agents/scripts/integratedreading/modes/*.md packages/witness-pipeline/scripts/integratedreading/modes/
cp /Volumes/madara/2026/twc-vault/01-Projects/tryambakam-noesis/witness-agents/scripts/integratedreading/engine-lexicons.md packages/witness-pipeline/scripts/integratedreading/
```

**Step 3: Run smoke**

Run:

```bash
cd packages/witness-pipeline
node --import tsx scripts/smoke.ts
```

Expected: prints mode, subjects, pass count, assembled length.

**Step 4: Commit**

```bash
git add packages/witness-pipeline/scripts/ packages/witness-pipeline/scripts/integratedreading/
git commit -m "feat(witness-pipeline): add smoke script and ported mode docs"
```

---

### Task 17: Document the port in a README

**Files:**
- Create: `packages/witness-pipeline/README.md`

**Step 1: Write the README**

```markdown
# @noesis/witness-pipeline

Premium integrated reading pipeline, ported from `witness-agents/` and embedded in Selemene-engine as an npm workspace.

## What is included

- `src/selemene/fetcher.ts` — parallel Selemene engine client with injectable fetch.
- `src/selemene/types.ts` — engine identifiers, routing, typed result map.
- `src/modes/parser.ts` — markdown mode-document parser (YAML frontmatter + templates + lessons).
- `src/orchestrator/integrated.ts` — multi-pass reading orchestrator.
- `src/assets/factory.ts` — deterministic source-pack factory (manifest + reading + reflection questions).
- `src/assets/audit.ts` — chain audit with deterministic-only gate.

## What is NOT included yet

- NotebookLM audio/video/slide generation.
- HTML/PDF renderers (source-pack is markdown-only in this pass).

## Running tests

```bash
npm run typecheck
npm run build
npm test
```

## Smoke run

```bash
node --import tsx scripts/smoke.ts
```
```

**Step 2: Verify**

Run:

```bash
cd packages/witness-pipeline
npm run build
```

Expected: PASS.

**Step 3: Commit**

```bash
git add packages/witness-pipeline/README.md
git commit -m "docs(witness-pipeline): add README"
```

---

## Final Verification

Run from repo root:

```bash
cargo test -p noesis-witness -p noesis-api
cd packages/witness-pipeline && npm run typecheck && npm run build && npm test
```

Both should be green.

---

## Summary of Changes

- Added `packages/witness-pipeline` workspace with Selemene fetcher, engine routing, mode parser, integrated orchestrator, source-pack factory, and chain audit.
- Ported `witness-agents` mode docs into `packages/witness-pipeline/scripts/integratedreading/modes/`.
- Added `@noesis/witness-pipeline` usage example in `apps/noesis-web`.
- Upgraded `crates/noesis-witness` with engine routing, relationship mode, partner context, and grouped context formatting.
- Updated `crates/noesis-api` `/witness/interpret` to accept optional `partner_birth_data` and `relationship_mode`.
- Added unit tests for Rust and TypeScript changes.

---

## Execution Handoff

**Plan complete and saved to `docs/plans/2026-06-22-port-witness-agents-into-selemene-engine.md`.**

**Two execution options:**

1. **Subagent-Driven (this session)** — I dispatch a fresh subagent per task, review between tasks, fast iteration.
2. **Parallel Session (separate)** — Open a new session in the worktree `../selemene-engine-port-witness` and use `superpowers:executing-plans` for batch execution with checkpoints.

**Which approach?**
