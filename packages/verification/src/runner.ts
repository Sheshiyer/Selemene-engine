import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import type { SelemeneEngineId } from '@noesis/witness-pipeline';
import { fetchEngineResult } from './sources/selemene.js';
import { calculateAccuracy, compareField } from './scoring.js';
import { extract, ENGINE_FIELD_EXTRACTORS } from './extract.js';
import type { GoldenFile, Subject, VerificationResult } from './types.js';
import type { SelemeneOptions } from './sources/selemene.js';

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
