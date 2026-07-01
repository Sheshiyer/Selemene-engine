# Selemene Verification System Implementation Plan

> **For Codex:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build `packages/verification/` — a centralized TypeScript test suite that runs premium readings through the Selemene engine, compares output against golden fixtures from authoritative sources, and reports per-engine accuracy to harden the engine over time.

**Architecture:** A matrix test runner loads subjects and golden files per engine, fetches live Selemene API output via the existing `@noesis/witness-pipeline` fetcher, compares fields using weighted scoring, and fails tests on accuracy regression. Results feed a benchmark reporter for per-engine accuracy tracking.

**Tech Stack:** TypeScript, Vitest, `@noesis/witness-pipeline`, pnpm workspaces, GitHub Actions.

---

## Phase 1: Foundation

### Task 1: Create package skeleton

**Files:**
- Create: `packages/verification/package.json`
- Create: `packages/verification/tsconfig.json`
- Create: `packages/verification/vitest.config.ts`

**Step 1: Write `packages/verification/package.json`**

```json
{
  "name": "@noesis/verification",
  "version": "0.1.0",
  "description": "Premium reading verification and regression test suite",
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
    "benchmark": "tsx src/cli/benchmark.ts"
  },
  "dependencies": {
    "@noesis/witness-pipeline": "workspace:*"
  },
  "devDependencies": {
    "@types/node": "^22.0.0",
    "tsx": "^4.21.0",
    "typescript": "^5.7.0",
    "vitest": "^2.1.8"
  }
}
```

**Step 2: Write `packages/verification/tsconfig.json`**

```json
{
  "compilerOptions": {
    "target": "ES2022",
    "module": "NodeNext",
    "moduleResolution": "NodeNext",
    "lib": ["ES2022"],
    "outDir": "dist",
    "rootDir": "src",
    "strict": true,
    "esModuleInterop": true,
    "skipLibCheck": true,
    "forceConsistentCasingInFileNames": true,
    "resolveJsonModule": true,
    "declaration": true,
    "declarationMap": true,
    "sourceMap": true
  },
  "include": ["src/**/*"],
  "exclude": ["dist", "node_modules", "**/*.test.ts"]
}
```

**Step 3: Write `packages/verification/vitest.config.ts`**

```typescript
import { defineConfig } from 'vitest/config';

export default defineConfig({
  test: {
    globals: true,
    environment: 'node',
    include: ['tests/**/*.test.ts'],
    env: {
      SELEMENE_API_URL: process.env.SELEMENE_API_URL ?? 'https://selemene.tryambakam.space',
    },
  },
});
```

**Step 4: Run install**

Run: `pnpm install`
Expected: Adds `@noesis/verification` workspace link and dependencies.

**Step 5: Commit**

```bash
git add packages/verification/package.json packages/verification/tsconfig.json packages/verification/vitest.config.ts pnpm-lock.yaml
git commit -m "feat(verification): scaffold verification package"
```

---

### Task 2: Add shared types

**Files:**
- Create: `packages/verification/src/types.ts`

**Step 1: Write the types**

```typescript
import type { SelemeneEngineId } from '@noesis/witness-pipeline';

export interface Location {
  place: string;
  latitude: number;
  longitude: number;
}

export interface Subject {
  id: string;
  name: string;
  birth: {
    date: string;
    time: string;
    timezone: string;
    location: Location;
  };
  ayanamsa?: string;
  added: string;
  source?: string;
}

export type Severity = 'P0' | 'P1' | 'P2' | 'P3' | 'P4';

export interface GoldenField {
  expected: unknown;
  weight: number;
  severity?: Severity;
  notes?: string;
}

export interface GoldenFile {
  subject: string;
  engine: SelemeneEngineId;
  source: string;
  captured: string;
  minAccuracy?: number;
  fields: Record<string, GoldenField>;
}

export interface FieldOutcome {
  pass: boolean;
  expected: unknown;
  actual: unknown;
  weight: number;
}

export interface VerificationResult {
  subject: string;
  engine: SelemeneEngineId;
  accuracy: number;
  fields: Record<string, FieldOutcome>;
  missingFields: string[];
  error?: string;
}
```

**Step 2: Commit**

```bash
git add packages/verification/src/types.ts
git commit -m "feat(verification): add shared verification types"
```

---

### Task 3: Add Selemene source adapter

**Files:**
- Create: `packages/verification/src/sources/selemene.ts`

**Step 1: Write the adapter**

```typescript
import { fetchAllEngines, loadSelemeneKey, type BirthData, type SelemeneEngineId } from '@noesis/witness-pipeline';
import type { Subject } from '../types.js';

export interface SelemeneOptions {
  baseUrl?: string;
  apiKey?: string;
}

export async function fetchEngineResult(subject: Subject, engineId: SelemeneEngineId, opts: SelemeneOptions = {}): Promise<unknown> {
  const apiKey = opts.apiKey ?? (await loadSelemeneKey());
  if (!apiKey) {
    throw new Error('SELEMENE_API_KEY not found. Set env var or add to ~/.claude/.env');
  }

  const birthData: BirthData = {
    date: subject.birth.date,
    time: subject.birth.time,
    timezone: subject.birth.timezone,
    latitude: subject.birth.location.latitude,
    longitude: subject.birth.location.longitude,
    name: subject.name,
  };

  const [result] = await fetchAllEngines(birthData, {
    api_key: apiKey,
    base_url: opts.baseUrl,
    engines: [engineId],
  });

  if (result._error) {
    throw new Error(`Selemene ${engineId} failed: ${result._error}`);
  }

  return result.result;
}
```

**Step 2: Commit**

```bash
git add packages/verification/src/sources/selemene.ts
git commit -m "feat(verification): add Selemene API source adapter"
```

---

### Task 4: Add scoring module

**Files:**
- Create: `packages/verification/src/scoring.ts`
- Test: `packages/verification/src/scoring.test.ts`

**Step 1: Write the scoring test**

```typescript
import { describe, it, expect } from 'vitest';
import { calculateAccuracy, compareField } from './scoring.js';

describe('compareField', () => {
  it('passes on exact match', () => {
    const outcome = compareField('Shanivara', 'Shanivara', 0.2);
    expect(outcome.pass).toBe(true);
    expect(outcome.weight).toBe(0.2);
  });

  it('fails on mismatch', () => {
    const outcome = compareField('Shanivara', 'Shukravara', 0.2);
    expect(outcome.pass).toBe(false);
  });
});

describe('calculateAccuracy', () => {
  it('returns 1.0 when all fields pass', () => {
    const fields = {
      a: { pass: true, expected: 1, actual: 1, weight: 0.5 },
      b: { pass: true, expected: 2, actual: 2, weight: 0.5 },
    };
    expect(calculateAccuracy(fields)).toBe(1.0);
  });

  it('returns weighted accuracy when some fields fail', () => {
    const fields = {
      a: { pass: true, expected: 1, actual: 1, weight: 0.8 },
      b: { pass: false, expected: 2, actual: 3, weight: 0.2 },
    };
    expect(calculateAccuracy(fields)).toBe(0.8);
  });
});
```

**Step 2: Run test to verify it fails**

Run: `pnpm --filter @noesis/verification test scoring.test.ts`
Expected: FAIL — `compareField` and `calculateAccuracy` not defined.

**Step 3: Write the implementation**

```typescript
import type { FieldOutcome } from './types.js';

export function compareField(expected: unknown, actual: unknown, weight: number): FieldOutcome {
  return {
    pass: deepEqual(expected, actual),
    expected,
    actual,
    weight,
  };
}

export function calculateAccuracy(fields: Record<string, FieldOutcome>): number {
  const totalWeight = Object.values(fields).reduce((sum, f) => sum + f.weight, 0);
  if (totalWeight === 0) return 0;
  const correctWeight = Object.values(fields)
    .filter((f) => f.pass)
    .reduce((sum, f) => sum + f.weight, 0);
  return correctWeight / totalWeight;
}

function deepEqual(a: unknown, b: unknown): boolean {
  if (a === b) return true;
  if (typeof a !== typeof b) return false;
  if (a === null || b === null) return false;
  if (Array.isArray(a) && Array.isArray(b)) {
    if (a.length !== b.length) return false;
    return a.every((v, i) => deepEqual(v, b[i]));
  }
  if (typeof a === 'object' && typeof b === 'object') {
    const aKeys = Object.keys(a as object);
    const bKeys = Object.keys(b as object);
    if (aKeys.length !== bKeys.length) return false;
    return aKeys.every((k) => deepEqual((a as Record<string, unknown>)[k], (b as Record<string, unknown>)[k]));
  }
  return false;
}
```

**Step 4: Run test to verify it passes**

Run: `pnpm --filter @noesis/verification test scoring.test.ts`
Expected: PASS.

**Step 5: Commit**

```bash
git add packages/verification/src/scoring.ts packages/verification/src/scoring.test.ts
git commit -m "feat(verification): add field-level scoring"
```

---

### Task 5: Add runner core

**Files:**
- Create: `packages/verification/src/runner.ts`
- Create: `packages/verification/src/extract.ts`
- Test: `packages/verification/src/runner.test.ts`

**Step 1: Write extractor helpers**

```typescript
// src/extract.ts
import type { SelemeneEngineId } from '@noesis/witness-pipeline';

export function extract(result: unknown, path: string): unknown {
  if (!result || typeof result !== 'object') return undefined;
  return path.split('.').reduce<unknown>((acc, key) => {
    if (acc === undefined || acc === null) return undefined;
    if (Array.isArray(acc)) {
      const idx = Number(key);
      return Number.isNaN(idx) ? undefined : acc[idx];
    }
    return (acc as Record<string, unknown>)[key];
  }, result);
}

export const ENGINE_FIELD_EXTRACTORS: Partial<Record<SelemeneEngineId, Record<string, string>>> = {
  panchanga: {
    vara: 'vara_name',
    tithi: 'tithi_name',
    nakshatra: 'nakshatra_name',
    yoga: 'yoga_name',
    karana: 'karana_name',
  },
  'human-design': {
    type: 'type',
    authority: 'authority',
    profile: 'profile',
    definition: 'definition',
    cross: 'cross',
    defined_centers: 'defined_centers',
    active_channels: 'active_channels',
  },
  'gene-keys': {
    incarnation_cross: 'active_keys',
    activation_sequence: 'activation_sequence',
  },
  'vedic-chart': {
    lagna_sign: 'd1.ascendant.sign',
    lagna_nakshatra: 'd1.ascendant.nakshatra',
    moon_sign: 'd1.planets.Moon.sign',
    moon_nakshatra: 'd1.planets.Moon.nakshatra',
  },
  vimshottari: {
    current_mahadasha: 'current_mahadasha.planet',
    current_antardasha: 'current_antardasha.planet',
  },
  numerology: {
    life_path_number: 'life_path_number',
    expression: 'expression',
    soul_urge: 'soul_urge',
  },
};
```

**Step 2: Write runner test**

```typescript
import { describe, it, expect, vi } from 'vitest';
import { MatrixRunner } from './runner.js';
import type { GoldenFile, Subject } from './types.js';

describe('MatrixRunner.verify', () => {
  it('returns 100% accuracy when all fields match', async () => {
    const subject: Subject = {
      id: 'sahil',
      name: 'Sahil Singh Sabharwal',
      birth: {
        date: '1992-03-14',
        time: '02:22:00',
        timezone: 'Asia/Kolkata',
        location: { place: 'Bengaluru', latitude: 12.9716, longitude: 77.5946 },
      },
      added: '2026-06-24',
    };

    const golden: GoldenFile = {
      subject: 'sahil',
      engine: 'panchanga',
      source: 'drikpanchang.com',
      captured: '2026-06-24',
      fields: {
        vara: { expected: 'Shanivara (Saturday)', weight: 1.0 },
      },
    };

    const runner = new MatrixRunner({
      engine: 'panchanga',
      fetchEngine: vi.fn().mockResolvedValue({ vara_name: 'Shanivara (Saturday)' }),
    });

    const result = await runner.verify(subject, golden);

    expect(result.accuracy).toBe(1.0);
    expect(result.fields.vara.pass).toBe(true);
  });
});
```

**Step 3: Run test to verify it fails**

Run: `pnpm --filter @noesis/verification test runner.test.ts`
Expected: FAIL — `MatrixRunner` not defined.

**Step 4: Write the runner**

```typescript
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import type { SelemeneEngineId } from '@noesis/witness-pipeline';
import { fetchEngineResult } from './sources/selemene.js';
import { calculateAccuracy, compareField } from './scoring.js';
import { extract, ENGINE_FIELD_EXTRACTORS } from './extract.js';
import type { GoldenFile, Subject, VerificationResult, SelemeneOptions } from './types.js';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const FIXTURES_ROOT = path.resolve(__dirname, '..', '..', 'fixtures');

export interface RunnerOptions {
  engine: SelemeneEngineId;
  fetchEngine?: (subject: Subject, engine: SelemeneEngineId) => Promise<unknown>;
  selemene?: SelemeneOptions;
  fixturesRoot?: string;
}

export class MatrixRunner {
  private engine: SelemeneEngineId;
  private fetchEngine: (subject: Subject, engine: SelemeneEngineId) => Promise<unknown>;
  private fixturesRoot: string;

  constructor(opts: RunnerOptions) {
    this.engine = opts.engine;
    this.fixturesRoot = opts.fixturesRoot ?? FIXTURES_ROOT;
    this.fetchEngine =
      opts.fetchEngine ??
      (async (subject, engine) => fetchEngineResult(subject, engine, opts.selemene ?? {}));
  }

  loadSubjects(): Subject[] {
    const dir = path.join(this.fixturesRoot, 'subjects');
    return fs
      .readdirSync(dir)
      .filter((f) => f.endsWith('.json'))
      .map((f) => JSON.parse(fs.readFileSync(path.join(dir, f), 'utf-8')) as Subject);
  }

  loadGoldens(engine: SelemeneEngineId): Record<string, GoldenFile> {
    const dir = path.join(this.fixturesRoot, 'golden', engine);
    if (!fs.existsSync(dir)) return {};
    return fs
      .readdirSync(dir)
      .filter((f) => f.endsWith('.json'))
      .reduce<Record<string, GoldenFile>>((acc, f) => {
        const golden = JSON.parse(fs.readFileSync(path.join(dir, f), 'utf-8')) as GoldenFile;
        acc[golden.subject] = golden;
        return acc;
      }, {});
  }

  async verify(subject: Subject, golden: GoldenFile): Promise<VerificationResult> {
    try {
      const result = await this.fetchEngine(subject, this.engine);
      const extractors = ENGINE_FIELD_EXTRACTORS[this.engine] ?? {};
      const fields: VerificationResult['fields'] = {};
      const missingFields: string[] = [];

      for (const [fieldName, goldenField] of Object.entries(golden.fields)) {
        const actual = extract(result, extractors[fieldName] ?? fieldName);
        if (actual === undefined) {
          missingFields.push(fieldName);
        }
        fields[fieldName] = compareField(goldenField.expected, actual, goldenField.weight);
      }

      return {
        subject: subject.id,
        engine: this.engine,
        accuracy: calculateAccuracy(fields),
        fields,
        missingFields,
      };
    } catch (err) {
      return {
        subject: subject.id,
        engine: this.engine,
        accuracy: 0,
        fields: {},
        missingFields: Object.keys(golden.fields),
        error: err instanceof Error ? err.message : String(err),
      };
    }
  }
}
```

**Step 5: Run test to verify it passes**

Run: `pnpm --filter @noesis/verification test runner.test.ts`
Expected: PASS.

**Step 6: Commit**

```bash
git add packages/verification/src/extract.ts packages/verification/src/runner.ts packages/verification/src/runner.test.ts
git commit -m "feat(verification): add matrix runner core"
```

---

### Task 6: Add console reporter

**Files:**
- Create: `packages/verification/src/reporters/console.ts`
- Create: `packages/verification/src/reporters/json.ts`

**Step 1: Write console reporter**

```typescript
import type { VerificationResult } from '../types.js';

export function printResult(result: VerificationResult): void {
  const status = result.accuracy === 1 ? '✓' : '✗';
  const percent = `${(result.accuracy * 100).toFixed(0)}%`;
  console.log(`  ${status} ${result.subject}: ${percent}`);

  for (const [name, outcome] of Object.entries(result.fields)) {
    if (!outcome.pass) {
      console.log(`      ✗ ${name}: expected ${JSON.stringify(outcome.expected)}, got ${JSON.stringify(outcome.actual)}`);
    }
  }

  if (result.missingFields.length) {
    console.log(`      missing fields: ${result.missingFields.join(', ')}`);
  }
  if (result.error) {
    console.log(`      error: ${result.error}`);
  }
}

export function printSummary(results: VerificationResult[]): void {
  const passing = results.filter((r) => r.accuracy === 1).length;
  const total = results.length;
  const avg = total > 0 ? results.reduce((s, r) => s + r.accuracy, 0) / total : 0;
  console.log(`\n${passing}/${total} subjects at 100%, average accuracy ${(avg * 100).toFixed(1)}%`);
}
```

**Step 2: Write JSON reporter**

```typescript
import type { VerificationResult } from '../types.js';

export interface BenchmarkReport {
  generated: string;
  subjects: number;
  engines: Record<string, {
    accuracy: number;
    subjects: number;
    failures: string[];
  }>;
}

export function buildBenchmark(results: VerificationResult[]): BenchmarkReport {
  const byEngine: Record<string, VerificationResult[]> = {};
  for (const r of results) {
    byEngine[r.engine] = byEngine[r.engine] ?? [];
    byEngine[r.engine].push(r);
  }

  const engines: BenchmarkReport['engines'] = {};
  for (const [engine, list] of Object.entries(byEngine)) {
    const accuracy = list.reduce((s, r) => s + r.accuracy, 0) / list.length;
    engines[engine] = {
      accuracy,
      subjects: list.length,
      failures: list
        .flatMap((r) =>
          Object.entries(r.fields)
            .filter(([, o]) => !o.pass)
            .map(([name]) => `${r.subject}.${name}`)
        ),
    };
  }

  return {
    generated: new Date().toISOString(),
    subjects: new Set(results.map((r) => r.subject)).size,
    engines,
  };
}
```

**Step 3: Commit**

```bash
git add packages/verification/src/reporters/console.ts packages/verification/src/reporters/json.ts
git commit -m "feat(verification): add console and JSON reporters"
```

---

### Task 7: Create Sahil subject fixture

**Files:**
- Create: `packages/verification/fixtures/subjects/sahil.json`

**Step 1: Write the fixture**

```json
{
  "id": "sahil",
  "name": "Sahil Singh Sabharwal",
  "birth": {
    "date": "1992-03-14",
    "time": "02:22:00",
    "timezone": "Asia/Kolkata",
    "location": {
      "place": "Bengaluru, Karnataka, India",
      "latitude": 12.9716,
      "longitude": 77.5946
    }
  },
  "ayanamsa": "lahiri",
  "added": "2026-06-24",
  "source": "723/sahil-reading"
}
```

**Step 2: Commit**

```bash
git add packages/verification/fixtures/subjects/sahil.json
git commit -m "feat(verification): add sahil subject fixture"
```

---

### Task 8: Create panchanga golden fixture and test for Sahil

**Files:**
- Create: `packages/verification/fixtures/golden/panchanga/sahil.json`
- Create: `packages/verification/tests/panchanga.test.ts`

**Step 1: Write the golden fixture**

```json
{
  "subject": "sahil",
  "engine": "panchanga",
  "source": "drikpanchang.com",
  "captured": "2026-06-24",
  "minAccuracy": 0.8,
  "fields": {
    "vara": {
      "expected": "Shanivara (Saturday)",
      "weight": 0.2,
      "severity": "P1",
      "notes": "Engine returns Shukravara (Friday) due to UTC timezone bug"
    },
    "tithi": {
      "expected": "Dashami (Shukla)",
      "weight": 0.2,
      "severity": "P3"
    },
    "nakshatra": {
      "expected": "Punarvasu",
      "weight": 0.2,
      "severity": "P3"
    },
    "yoga": {
      "expected": "Saubhagya",
      "weight": 0.2,
      "severity": "P3"
    },
    "karana": {
      "expected": "Taitila",
      "weight": 0.2,
      "severity": "P3"
    }
  }
}
```

**Step 2: Write the matrix test**

```typescript
import { describe, it, expect } from 'vitest';
import { MatrixRunner } from '../src/runner.js';
import { printResult, printSummary } from '../src/reporters/console.js';

const runner = new MatrixRunner({ engine: 'panchanga' });
const subjects = runner.loadSubjects();
const goldens = runner.loadGoldens('panchanga');

describe('panchanga', () => {
  for (const subject of subjects) {
    const golden = goldens[subject.id];
    if (!golden) continue;

    it(`${subject.id} matches golden`, async () => {
      const result = await runner.verify(subject, golden);
      printResult(result);
      expect(result.accuracy).toBeGreaterThanOrEqual(golden.minAccuracy ?? 1.0);
    });
  }

  it('summary', () => {
    // This runs after all subjects; vitest collects results separately.
    // Summary is printed by individual tests above.
  });
});
```

**Step 3: Run the test**

Run: `pnpm --filter @noesis/verification test panchanga.test.ts`
Expected: FAIL if Selemene still has the Vara bug; PASS if it doesn't. The `minAccuracy: 0.8` allows the known bug.

**Step 4: Commit**

```bash
git add packages/verification/fixtures/golden/panchanga/sahil.json packages/verification/tests/panchanga.test.ts
git commit -m "feat(verification): add panchanga verification for sahil"
```

---

### Task 9: Create human-design golden fixture and test for Sahil

**Files:**
- Create: `packages/verification/fixtures/golden/human-design/sahil.json`
- Create: `packages/verification/tests/human-design.test.ts`

**Step 1: Write the golden fixture**

```json
{
  "subject": "sahil",
  "engine": "human-design",
  "source": "humdes.com",
  "captured": "2026-06-24",
  "minAccuracy": 0.6,
  "fields": {
    "type": {
      "expected": "Generator",
      "weight": 0.15,
      "severity": "P0"
    },
    "authority": {
      "expected": "Emotional",
      "weight": 0.15,
      "severity": "P0"
    },
    "profile": {
      "expected": "1/4",
      "weight": 0.15,
      "severity": "P0"
    },
    "definition": {
      "expected": "Single",
      "weight": 0.15,
      "severity": "P0"
    },
    "cross": {
      "expected": "36/6/11/12",
      "weight": 0.15,
      "severity": "P0"
    },
    "defined_centers": {
      "expected": ["Sacral", "SolarPlexus", "Root", "Spleen"],
      "weight": 0.15,
      "severity": "P1",
      "notes": "Engine currently omits Spleen"
    },
    "active_channels": {
      "expected": 13,
      "weight": 0.10,
      "severity": "P1",
      "notes": "Engine currently returns 2"
    }
  }
}
```

**Step 2: Write the matrix test**

```typescript
import { describe, it } from 'vitest';
import { MatrixRunner } from '../src/runner.js';
import { printResult } from '../src/reporters/console.js';

const runner = new MatrixRunner({ engine: 'human-design' });
const subjects = runner.loadSubjects();
const goldens = runner.loadGoldens('human-design');

describe('human-design', () => {
  for (const subject of subjects) {
    const golden = goldens[subject.id];
    if (!golden) continue;

    it(`${subject.id} matches golden`, async () => {
      const result = await runner.verify(subject, golden);
      printResult(result);
      expect(result.accuracy).toBeGreaterThanOrEqual(golden.minAccuracy ?? 1.0);
    });
  }
});
```

**Step 3: Run the test**

Run: `pnpm --filter @noesis/verification test human-design.test.ts`
Expected: Runs and reports accuracy.

**Step 4: Commit**

```bash
git add packages/verification/fixtures/golden/human-design/sahil.json packages/verification/tests/human-design.test.ts
git commit -m "feat(verification): add human-design verification for sahil"
```

---

### Task 10: Create gene-keys golden fixture and test for Sahil

**Files:**
- Create: `packages/verification/fixtures/golden/gene-keys/sahil.json`
- Create: `packages/verification/tests/gene-keys.test.ts`

**Step 1: Write the golden fixture**

```json
{
  "subject": "sahil",
  "engine": "gene-keys",
  "source": "humdes.com",
  "captured": "2026-06-24",
  "minAccuracy": 0.4,
  "fields": {
    "life_work": {
      "expected": { "key_number": 36, "line": 3 },
      "weight": 0.09,
      "severity": "P0"
    },
    "evolution": {
      "expected": { "key_number": 6, "line": 3 },
      "weight": 0.09,
      "severity": "P0"
    },
    "radiance": {
      "expected": { "key_number": 11, "line": 3 },
      "weight": 0.09,
      "severity": "P0"
    },
    "purpose": {
      "expected": { "key_number": 12, "line": 3 },
      "weight": 0.09,
      "severity": "P0"
    },
    "pearl": {
      "expected": { "key_number": 40, "line": 3 },
      "weight": 0.09,
      "severity": "P0",
      "notes": "Engine missing"
    },
    "vocation": {
      "expected": { "key_number": 5, "line": 3 },
      "weight": 0.09,
      "severity": "P0",
      "notes": "Engine missing"
    },
    "culture": {
      "expected": { "key_number": 64, "line": 4 },
      "weight": 0.09,
      "severity": "P0",
      "notes": "Engine missing"
    },
    "sq": {
      "expected": { "key_number": 1, "line": 1 },
      "weight": 0.09,
      "severity": "P0",
      "notes": "Engine missing"
    },
    "eq": {
      "expected": { "key_number": 49, "line": 1 },
      "weight": 0.09,
      "severity": "P0",
      "notes": "Engine missing"
    },
    "iq": {
      "expected": { "key_number": 30, "line": 6 },
      "weight": 0.09,
      "severity": "P0",
      "notes": "Engine missing"
    },
    "attraction": {
      "expected": { "key_number": 24, "line": 2 },
      "weight": 0.09,
      "severity": "P0",
      "notes": "Engine missing"
    },
    "reaction_shadow": {
      "expected": "Self-Sabotage",
      "weight": 0.09,
      "severity": "P0",
      "notes": "EQ shadow for legacy terminology"
    }
  }
}
```

**Step 2: Update extractor for gene-keys reaction_shadow**

Modify `packages/verification/src/extract.ts` line 8-48 to include gene-keys reaction extractor:

```typescript
export const ENGINE_FIELD_EXTRACTORS: Partial<Record<SelemeneEngineId, Record<string, string>>> = {
  // ... existing entries ...
  'gene-keys': {
    life_work: 'hologenetic_profile.life_work',
    evolution: 'hologenetic_profile.evolution',
    radiance: 'hologenetic_profile.radiance',
    purpose: 'hologenetic_profile.purpose',
    pearl: 'hologenetic_profile.pearl',
    vocation: 'hologenetic_profile.vocation',
    culture: 'hologenetic_profile.culture',
    sq: 'hologenetic_profile.sq',
    eq: 'hologenetic_profile.eq',
    iq: 'hologenetic_profile.iq',
    attraction: 'hologenetic_profile.attraction',
    reaction_shadow: 'hologenetic_profile.eq.shadow',
  },
  // ... rest unchanged ...
};
```

**Step 3: Write the matrix test**

```typescript
import { describe, it } from 'vitest';
import { MatrixRunner } from '../src/runner.js';
import { printResult } from '../src/reporters/console.js';

const runner = new MatrixRunner({ engine: 'gene-keys' });
const subjects = runner.loadSubjects();
const goldens = runner.loadGoldens('gene-keys');

describe('gene-keys', () => {
  for (const subject of subjects) {
    const golden = goldens[subject.id];
    if (!golden) continue;

    it(`${subject.id} matches golden`, async () => {
      const result = await runner.verify(subject, golden);
      printResult(result);
      expect(result.accuracy).toBeGreaterThanOrEqual(golden.minAccuracy ?? 1.0);
    });
  }
});
```

**Step 4: Run the test**

Run: `pnpm --filter @noesis/verification test gene-keys.test.ts`
Expected: Runs with low accuracy (engine missing positions).

**Step 5: Commit**

```bash
git add packages/verification/fixtures/golden/gene-keys/sahil.json packages/verification/tests/gene-keys.test.ts packages/verification/src/extract.ts
git commit -m "feat(verification): add gene-keys verification for sahil"
```

---

### Task 11: Create vedic-chart golden fixture and test for Sahil

**Files:**
- Create: `packages/verification/fixtures/golden/vedic-chart/sahil.json`
- Create: `packages/verification/tests/vedic-chart.test.ts`

**Step 1: Write the golden fixture**

```json
{
  "subject": "sahil",
  "engine": "vedic-chart",
  "source": "kundli-tool",
  "captured": "2026-06-24",
  "minAccuracy": 0.9,
  "fields": {
    "lagna_sign": {
      "expected": "Sagittarius",
      "weight": 0.15,
      "severity": "P0"
    },
    "lagna_nakshatra": {
      "expected": "Purva Ashadha",
      "weight": 0.10,
      "severity": "P2"
    },
    "moon_sign": {
      "expected": "Gemini",
      "weight": 0.15,
      "severity": "P0"
    },
    "moon_nakshatra": {
      "expected": "Punarvasu",
      "weight": 0.15,
      "severity": "P0"
    },
    "mars_dignity": {
      "expected": "exalted",
      "weight": 0.15,
      "severity": "P2",
      "notes": "Engine missing dignity field"
    },
    "mercury_dignity": {
      "expected": "debilitated",
      "weight": 0.15,
      "severity": "P2",
      "notes": "Engine missing dignity field"
    },
    "saturn_dignity": {
      "expected": "own_sign",
      "weight": 0.15,
      "severity": "P2",
      "notes": "Engine missing dignity field"
    }
  }
}
```

**Step 2: Update extractor**

Add to `packages/verification/src/extract.ts`:

```typescript
'vedic-chart': {
  lagna_sign: 'd1.ascendant.sign',
  lagna_nakshatra: 'd1.ascendant.nakshatra',
  moon_sign: 'd1.planets.Moon.sign',
  moon_nakshatra: 'd1.planets.Moon.nakshatra',
  mars_dignity: 'd1.planets.Mars.dignity',
  mercury_dignity: 'd1.planets.Mercury.dignity',
  saturn_dignity: 'd1.planets.Saturn.dignity',
},
```

**Step 3: Write the matrix test**

```typescript
import { describe, it } from 'vitest';
import { MatrixRunner } from '../src/runner.js';
import { printResult } from '../src/reporters/console.js';

const runner = new MatrixRunner({ engine: 'vedic-chart' });
const subjects = runner.loadSubjects();
const goldens = runner.loadGoldens('vedic-chart');

describe('vedic-chart', () => {
  for (const subject of subjects) {
    const golden = goldens[subject.id];
    if (!golden) continue;

    it(`${subject.id} matches golden`, async () => {
      const result = await runner.verify(subject, golden);
      printResult(result);
      expect(result.accuracy).toBeGreaterThanOrEqual(golden.minAccuracy ?? 1.0);
    });
  }
});
```

**Step 4: Commit**

```bash
git add packages/verification/fixtures/golden/vedic-chart/sahil.json packages/verification/tests/vedic-chart.test.ts packages/verification/src/extract.ts
git commit -m "feat(verification): add vedic-chart verification for sahil"
```

---

### Task 12: Create vimshottari golden fixture and test for Sahil

**Files:**
- Create: `packages/verification/fixtures/golden/vimshottari/sahil.json`
- Create: `packages/verification/tests/vimshottari.test.ts`

**Step 1: Write the golden fixture**

```json
{
  "subject": "sahil",
  "engine": "vimshottari",
  "source": "kundli-tool",
  "captured": "2026-06-24",
  "minAccuracy": 0.5,
  "fields": {
    "current_mahadasha": {
      "expected": "Mercury",
      "weight": 0.5,
      "severity": "P0"
    },
    "current_antardasha": {
      "expected": "Ketu",
      "weight": 0.5,
      "severity": "P0"
    }
  }
}
```

**Step 2: Write the matrix test**

```typescript
import { describe, it } from 'vitest';
import { MatrixRunner } from '../src/runner.js';
import { printResult } from '../src/reporters/console.js';

const runner = new MatrixRunner({ engine: 'vimshottari' });
const subjects = runner.loadSubjects();
const goldens = runner.loadGoldens('vimshottari');

describe('vimshottari', () => {
  for (const subject of subjects) {
    const golden = goldens[subject.id];
    if (!golden) continue;

    it(`${subject.id} matches golden`, async () => {
      const result = await runner.verify(subject, golden);
      printResult(result);
      expect(result.accuracy).toBeGreaterThanOrEqual(golden.minAccuracy ?? 1.0);
    });
  }
});
```

**Step 3: Commit**

```bash
git add packages/verification/fixtures/golden/vimshottari/sahil.json packages/verification/tests/vimshottari.test.ts
git commit -m "feat(verification): add vimshottari verification for sahil"
```

---

### Task 13: Create numerology golden fixture and test for Sahil

**Files:**
- Create: `packages/verification/fixtures/golden/numerology/sahil.json`
- Create: `packages/verification/tests/numerology.test.ts`

**Step 1: Write the golden fixture**

```json
{
  "subject": "sahil",
  "engine": "numerology",
  "source": "manual-calculation",
  "captured": "2026-06-24",
  "minAccuracy": 0.85,
  "fields": {
    "life_path_number": {
      "expected": 11,
      "weight": 0.25,
      "severity": "P0"
    },
    "expression": {
      "expected": 11,
      "weight": 0.25,
      "severity": "P0"
    },
    "soul_urge": {
      "expected": 22,
      "weight": 0.25,
      "severity": "P0"
    },
    "destiny_number": {
      "expected": 11,
      "weight": 0.25,
      "severity": "P2",
      "notes": "Alias for life_path_number; engine missing field"
    }
  }
}
```

**Step 2: Update extractor**

Add to `packages/verification/src/extract.ts`:

```typescript
numerology: {
  life_path_number: 'life_path_number',
  expression: 'expression',
  soul_urge: 'soul_urge',
  destiny_number: 'destiny_number',
},
```

**Step 3: Write the matrix test**

```typescript
import { describe, it } from 'vitest';
import { MatrixRunner } from '../src/runner.js';
import { printResult } from '../src/reporters/console.js';

const runner = new MatrixRunner({ engine: 'numerology' });
const subjects = runner.loadSubjects();
const goldens = runner.loadGoldens('numerology');

describe('numerology', () => {
  for (const subject of subjects) {
    const golden = goldens[subject.id];
    if (!golden) continue;

    it(`${subject.id} matches golden`, async () => {
      const result = await runner.verify(subject, golden);
      printResult(result);
      expect(result.accuracy).toBeGreaterThanOrEqual(golden.minAccuracy ?? 1.0);
    });
  }
});
```

**Step 4: Commit**

```bash
git add packages/verification/fixtures/golden/numerology/sahil.json packages/verification/tests/numerology.test.ts packages/verification/src/extract.ts
git commit -m "feat(verification): add numerology verification for sahil"
```

---

### Task 14: Add benchmark CLI

**Files:**
- Create: `packages/verification/src/cli/benchmark.ts`
- Modify: `packages/verification/package.json` scripts

**Step 1: Write benchmark CLI**

```typescript
import fs from 'node:fs';
import path from 'node:path';
import { MatrixRunner } from '../runner.js';
import { buildBenchmark } from '../reporters/json.js';
import type { SelemeneEngineId } from '@noesis/witness-pipeline';

const ENGINES: SelemeneEngineId[] = ['panchanga', 'human-design', 'gene-keys', 'vedic-chart', 'vimshottari', 'numerology'];

async function main() {
  const allResults = [];

  for (const engine of ENGINES) {
    const runner = new MatrixRunner({ engine });
    const subjects = runner.loadSubjects();
    const goldens = runner.loadGoldens(engine);

    for (const subject of subjects) {
      const golden = goldens[subject.id];
      if (!golden) continue;
      const result = await runner.verify(subject, golden);
      allResults.push(result);
    }
  }

  const report = buildBenchmark(allResults);
  const out = path.resolve(process.cwd(), 'benchmarks');
  fs.mkdirSync(out, { recursive: true });
  const file = path.join(out, `${new Date().toISOString().slice(0, 10)}.json`);
  fs.writeFileSync(file, JSON.stringify(report, null, 2));
  console.log(JSON.stringify(report, null, 2));
}

main();
```

**Step 2: Update package.json scripts**

Add `"benchmark": "tsx src/cli/benchmark.ts"` to `packages/verification/package.json` scripts.

**Step 3: Commit**

```bash
git add packages/verification/src/cli/benchmark.ts packages/verification/package.json
git commit -m "feat(verification): add benchmark CLI"
```

---

### Task 15: Wire root package scripts

**Files:**
- Modify: `package.json` (root)

**Step 1: Add scripts**

```json
{
  "scripts": {
    "verify": "pnpm --filter @noesis/verification test",
    "verify:benchmark": "pnpm --filter @noesis/verification benchmark",
    "verify:typecheck": "pnpm --filter @noesis/verification typecheck"
  }
}
```

**Step 2: Commit**

```bash
git add package.json
git commit -m "chore: add root verification scripts"
```

---

### Task 16: Add GitHub Actions verification workflow

**Files:**
- Create: `.github/workflows/verification.yml`

**Step 1: Write workflow**

```yaml
name: Verification Suite

on:
  pull_request:
    branches: [main]
  push:
    branches: [main]

jobs:
  verify:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: ./.github/actions/setup-node-pnpm
      - name: Typecheck verification package
        run: pnpm verify:typecheck
        env:
          SELEMENE_API_KEY: ${{ secrets.SELEMENE_API_KEY }}
      - name: Run verification suite
        run: pnpm verify
        env:
          SELEMENE_API_KEY: ${{ secrets.SELEMENE_API_KEY }}
      - name: Generate benchmark
        run: pnpm verify:benchmark
        env:
          SELEMENE_API_KEY: ${{ secrets.SELEMENE_API_KEY }}
      - name: Upload benchmark
        uses: actions/upload-artifact@v4
        with:
          name: verification-benchmark
          path: benchmarks/*.json
```

**Step 2: Commit**

```bash
git add .github/workflows/verification.yml
git commit -m "ci: add verification suite workflow"
```

---

### Task 17: Update PATCH-REGISTRY with verification linkage

**Files:**
- Modify: `docs/patches/PATCH-REGISTRY.md`

**Step 1: Add verification column mapping**

Change the table header from:
```markdown
| ID | Severity | Engine | Status | Found In | Summary | Verified In |
```

Add a section after the registry:

```markdown
## Verification Coverage

Each patch is linked to one or more golden fixture fields. When the patch is merged, remove `minAccuracy` exceptions and update this table.

| Patch ID | Fixture Field(s) | Test File |
|----------|------------------|-----------|
| P0.1 | `gene-keys/sahil.json:pearl, vocation, culture, sq, eq, iq, attraction` | `tests/gene-keys.test.ts` |
| P0.2 | `gene-keys/sahil.json:reaction_shadow` | `tests/gene-keys.test.ts` |
| P1.1 | `panchanga/sahil.json:vara` | `tests/panchanga.test.ts` |
| P1.2 | `human-design/sahil.json:defined_centers, active_channels` | `tests/human-design.test.ts` |
| P2.3 | `vedic-chart/sahil.json:mars_dignity, mercury_dignity, saturn_dignity` | `tests/vedic-chart.test.ts` |
| P2.5 | `numerology/sahil.json:destiny_number` | `tests/numerology.test.ts` |
```

**Step 2: Commit**

```bash
git add docs/patches/PATCH-REGISTRY.md
git commit -m "docs: link patch registry to verification fixtures"
```

---

### Task 18: Final Phase 1 verification

**Files:** none

**Step 1: Run typecheck**

Run: `pnpm verify:typecheck`
Expected: PASS.

**Step 2: Run all verification tests**

Run: `pnpm verify`
Expected: Tests run against live Selemene API. Some fail due to known bugs captured in `minAccuracy` thresholds.

**Step 3: Run benchmark**

Run: `pnpm verify:benchmark`
Expected: Generates `benchmarks/YYYY-MM-DD.json` with per-engine accuracy.

**Step 4: Commit**

```bash
git add benchmarks/
git commit -m "chore(verification): add baseline benchmark"
```

---

## Phase 2: Coverage Expansion

### Task 19: Import witnessalchemist reading

**Files:**
- Create: `packages/verification/fixtures/subjects/witnessalchemist.json`
- Create: `packages/verification/fixtures/golden/*/witnessalchemist.json` (for engines you want to verify)

**Step 1: Extract subject from 723/**

Read `/Volumes/madara/2026/twc-vault/01-Projects/tryambakam-noesis/723/witnessalchemist-reading/subjects/01_witnessalchemist.json` and convert to verification fixture format.

**Step 2: Write subject fixture**

```json
{
  "id": "witnessalchemist",
  "name": "Cumbipuram Nateshan Sheshnarayan Iyer (The Witness Alchemist)",
  "birth": {
    "date": "1991-08-13",
    "time": "13:31:00",
    "timezone": "Asia/Kolkata",
    "location": {
      "place": "Bengaluru, Karnataka, India",
      "latitude": 12.97,
      "longitude": 77.59
    }
  },
  "added": "2026-06-24",
  "source": "723/witnessalchemist-reading"
}
```

**Step 3: Run each engine to capture output**

Run: `pnpm --filter @noesis/verification test panchanga.test.ts`
Observe output, create `fixtures/golden/panchanga/witnessalchemist.json` with expected values from humdes/Kundli/drikpanchang sources.

**Step 4: Commit**

```bash
git add packages/verification/fixtures/subjects/witnessalchemist.json packages/verification/fixtures/golden/*/witnessalchemist.json
git commit -m "feat(verification): add witnessalchemist reading fixtures"
```

---

### Task 20: Import anitha-nateshan reading

**Files:**
- Create: `packages/verification/fixtures/subjects/anitha.json`
- Create: `packages/verification/fixtures/golden/*/anitha.json`

**Step 1: Read subject data from `/Volumes/madara/2026/twc-vault/01-Projects/tryambakam-noesis/723/anitha-nateshan-reading/subjects/*.json`.

**Step 2: Convert and commit subject + goldens using the same workflow as Task 19.

---

### Task 21: Import cs-nateshan reading

**Files:**
- Create: `packages/verification/fixtures/subjects/cs-nateshan.json`
- Create: `packages/verification/fixtures/golden/*/cs-nateshan.json`

**Step 1: Read subject data from `/Volumes/madara/2026/twc-vault/01-Projects/tryambakam-noesis/723/cs-nateshan-reading/subjects/*.json`.

**Step 2: Convert and commit.

---

### Task 22: Import harshita reading

**Files:**
- Create: `packages/verification/fixtures/subjects/harshita.json`
- Create: `packages/verification/fixtures/golden/*/harshita.json`

**Step 1: Read subject data from `/Volumes/madara/2026/twc-vault/01-Projects/tryambakam-noesis/723/harshita-reading/subjects/*.json`.

**Step 2: Convert and commit.

---

### Task 23: Backfill subject JSONs for readings without them

**Files:**
- Create: `/Volumes/madara/2026/twc-vault/01-Projects/tryambakam-noesis/723/chitra-reading/subjects/01_chitra.json`
- Create: `/Volumes/madara/2026/twc-vault/01-Projects/tryambakam-noesis/723/durgaprasad-reading/subjects/01_durgaprasad.json`
- Create: `/Volumes/madara/2026/twc-vault/01-Projects/tryambakam-noesis/723/mohan-reading/subjects/01_mohan.json`
- Create: `/Volumes/madara/2026/twc-vault/01-Projects/tryambakam-noesis/723/varsha-reading/subjects/01_varsha.json`

**Step 1: Extract birth data from each reading's source files**
- Look at HTML/DOCX/PDF for explicit birth data
- Check input image filenames for date hints
- Ask the user if birth data is not extractable

**Step 2: Write minimal subject JSON for each**

```json
{
  "subject": "Chitra Shivanagowda",
  "birth_date": "YYYY-MM-DD",
  "birth_time": "HH:MM:SS",
  "birth_place": "...",
  "latitude": 0.0,
  "longitude": 0.0,
  "timezone": "Asia/Kolkata",
  "source_path": "01-Projects/723/chitra-reading/inputs/",
  "output_dir": "/Volumes/madara/2026/twc-vault/01-Projects/723/chitra-reading/solos/chitra"
}
```

**Step 3: Commit**

```bash
git add 723/chitra-reading/subjects/ 723/durgaprasad-reading/subjects/ 723/mohan-reading/subjects/ 723/varsha-reading/subjects/
git commit -m "chore(723): backfill subject JSONs for chitra, durgaprasad, mohan, varsha"
```

---

### Task 24: Import chitra, durgaprasad, mohan, varsha into verification

**Files:**
- Create: `packages/verification/fixtures/subjects/{chitra,durgaprasad,mohan,varsha}.json`
- Create: `packages/verification/fixtures/golden/*/{chitra,durgaprasad,mohan,varsha}.json`

**Step 1:** Convert each 723 subject JSON to verification fixture format.

**Step 2:** Run each engine test, capture outputs, create goldens from authoritative sources.

**Step 3:** Commit.

---

### Task 25: Phase 2 final run

**Files:** none

**Step 1: Run full verification suite**

Run: `pnpm verify`
Expected: All tests run; failures only where `minAccuracy` thresholds are not met.

**Step 2: Run benchmark**

Run: `pnpm verify:benchmark`
Expected: Benchmark now covers 8+ subjects per engine.

**Step 3: Commit**

```bash
git add benchmarks/
git commit -m "chore(verification): add phase 2 benchmark"
```

---

## Testing Strategy

1. **Unit tests:** `scoring.test.ts`, `runner.test.ts` verify core logic without API calls.
2. **Integration tests:** Each engine test file fetches live Selemene API and compares against goldens.
3. **Regression prevention:** Once a patch is merged, update the golden file to require 100% accuracy and remove `minAccuracy` exception.
4. **Local development:** Tests require `SELEMENE_API_KEY` env var.

---

## Environment Setup

Add to `~/.claude/.env`:

```bash
SELEMENE_API_KEY=your_key_here
```

Or export before running tests:

```bash
export SELEMENE_API_KEY=your_key_here
pnpm verify
```

---

## Notes for Implementer

- The `@noesis/witness-pipeline` package already handles Selemene API auth and fetching. Reuse it.
- Golden fixture `expected` values must match the **exact** shape returned by `extract()` in `extract.ts`. Run tests first to see actual API output.
- If a field's path changes in the engine response, update `ENGINE_FIELD_EXTRACTORS` in `extract.ts`.
- `minAccuracy` is a temporary allowance for known bugs. Do not add new `minAccuracy` exceptions for newly discovered bugs; fix them or mark as P4/docs.
- Commit after every fixture/test pair. Do not batch large commits.
