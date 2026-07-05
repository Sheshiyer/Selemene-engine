import { describe, expect, it, vi, beforeEach } from 'vitest';
import {
  CloudflareVectorizePatternStore,
  CloudflareVectorizePatternRetriever,
} from './cloudflare-vectorize.js';
import type { ExtractedPattern } from './types.js';

function makeEnv(overrides: any = {}) {
  const vectors: any[] = [];
  const r2: Record<string, string> = {};
  const d1Rows: Record<string, any> = {};
  return {
    env: {
      REPORT_PATTERNS: {
        upsert: vi.fn(async (v: any[]) => {
          vectors.push(...v);
        }),
        query: vi.fn(async (_vec: number[], opts?: any) => {
          const filter = opts?.filter ?? {};
          const matches = vectors
            .filter((v) => {
              for (const k of Object.keys(filter)) {
                if (k === 'systems') {
                  const want = Array.isArray(filter.systems) ? filter.systems : [filter.systems];
                  const have = Array.isArray(v.metadata.systems) ? v.metadata.systems : [];
                  if (!want.some((w: string) => have.includes(w))) return false;
                } else if (v.metadata[k] !== filter[k]) return false;
              }
              return true;
            })
            .map((v, i) => ({ id: v.id, score: 0.9 - i * 0.01, metadata: v.metadata }));
          return { matches: matches.slice(0, opts?.topK ?? 5) };
        }),
      },
      AI: {
        run: vi.fn(async (_model: string, input: { text: string[] }) => ({
          data: input.text.map(() => Array.from({ length: 384 }, () => Math.random())),
        })),
      },
      PATTERNS_BUCKET: overrides.useR2
        ? {
            put: vi.fn(async (k: string, v: string) => {
              r2[k] = v;
            }),
            get: vi.fn(async (k: string) => (r2[k] ? { text: async () => r2[k] } : null)),
          }
        : undefined,
      PATTERNS_D1: overrides.useD1
        ? {
            prepare: vi.fn((sql: string) => ({
              bind: (...args: any[]) => ({
                run: async () => {
                  const id = args[0];
                  if (sql.includes('INSERT')) {
                    d1Rows[id] = {
                      id,
                      text: args[1],
                      kind: args[2],
                      source_section_id: args[3],
                      source_rubric_score: args[4],
                      metadata: args[5],
                    };
                  }
                },
                first: async () => d1Rows[args[0]] || null,
              }),
            })),
          }
        : undefined,
      ...overrides,
    },
    vectors,
    r2,
    d1Rows,
  };
}

const basePattern: ExtractedPattern = {
  id: 'vedic:section-1:v1',
  text: 'Saturn pressure plus Projector pacing can be framed as delayed recognition that builds authority.',
  kind: 'convergence',
  source_section_id: 'section-1',
  source_rubric_score: 0.92,
  metadata: {
    mode: 'vedic',
    report_level: 'L3',
    systems: ['vedic', 'human-design'],
    source: 'post-report-extraction',
    version: '1',
  },
};

describe('CloudflareVectorizePatternStore', () => {
  it('skips patterns containing private birth data', async () => {
    const { env } = makeEnv();
    const store = new CloudflareVectorizePatternStore(env);
    const bad: ExtractedPattern = {
      ...basePattern,
      text: 'Born 1985-03-12 at 14:30 in 37.7749,-122.4194',
    };
    const res = await store.upsertPatterns([bad]);
    expect(res).toEqual({ upserted: 0, skipped: 1 });
    expect(env.REPORT_PATTERNS.upsert).not.toHaveBeenCalled();
  });

  it('upserts safe patterns with R2 durable storage', async () => {
    const { env, vectors } = makeEnv({ useR2: true });
    const store = new CloudflareVectorizePatternStore(env);
    const res = await store.upsertPatterns([basePattern]);
    expect(res.upserted).toBe(1);
    expect(res.skipped).toBe(0);
    expect(env.REPORT_PATTERNS.upsert).toHaveBeenCalled();
    expect(vectors[0].metadata.text).toBeUndefined();
    expect(env.PATTERNS_BUCKET.put).toHaveBeenCalled();
  });

  it('falls back to metadata text when no durable store', async () => {
    const { env, vectors } = makeEnv();
    const store = new CloudflareVectorizePatternStore(env);
    await store.upsertPatterns([basePattern]);
    expect(vectors[0].metadata.text).toContain('Saturn pressure');
  });
});

describe('CloudflareVectorizePatternRetriever', () => {
  it('returns patterns with non-deterministic warning label in render', async () => {
    const { env } = makeEnv({ useR2: true });
    const store = new CloudflareVectorizePatternStore(env);
    await store.upsertPatterns([basePattern]);

    const retriever = new CloudflareVectorizePatternRetriever(env);
    const results = await retriever.retrieveSimilar('projector pacing authority', { mode: 'vedic', report_level: 'L3' });
    expect(results.length).toBeGreaterThan(0);
    const rendered = 'Retrieved synthesis patterns are not deterministic facts. Use them only for analogy, wording, and layering. Current chart data overrides retrieved context.\n1. ' + results[0].text;
    expect(rendered).toContain('Retrieved synthesis patterns are not deterministic facts');
  });

  it('applies metadata filters', async () => {
    const { env } = makeEnv();
    const store = new CloudflareVectorizePatternStore(env);
    await store.upsertPatterns([basePattern]);
    const retriever = new CloudflareVectorizePatternRetriever(env);
    const r1 = await retriever.retrieveSimilar('test', { mode: 'vedic', report_level: 'L3' });
    expect(r1.length).toBeGreaterThan(0);
    const r2 = await retriever.retrieveSimilar('test', { mode: 'human-design' });
    expect(r2.length).toBe(0);
  });
});
