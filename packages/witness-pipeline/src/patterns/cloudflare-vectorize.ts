import type { ExtractedPattern } from './types.js';
import type { PatternVectorStore, PatternVectorStoreResult } from './vector-store.js';
import type { PatternVectorRetriever, RetrievedPattern, RetrievalFilters } from './retrieval.js';

interface VectorizeIndex {
  upsert(vectors: Array<{ id: string; values: number[]; metadata?: Record<string, string | number | boolean | string[]> }>): Promise<void>;
  query(vector: number[], options?: { topK?: number; filter?: Record<string, any>; returnMetadata?: boolean }): Promise<{ matches: Array<{ id: string; score: number; metadata?: Record<string, any> }> }>;
}

interface AiBinding {
  run(model: string, input: { text: string[] }): Promise<{ data: number[][] }>;
}

interface R2Binding {
  put(key: string, value: string | ArrayBuffer, opts?: any): Promise<any>;
  get(key: string): Promise<{ text(): Promise<string> } | null>;
}

interface D1Binding {
  prepare(sql: string): { bind(...args: any[]): { run(): Promise<any>; first<T = any>(): Promise<T | null> } };
}

interface CloudflarePatternEnv {
  REPORT_PATTERNS: VectorizeIndex;
  AI: AiBinding;
  PATTERNS_BUCKET?: R2Binding;
  PATTERNS_D1?: D1Binding;
}

function containsPrivateBirthData(pattern: ExtractedPattern): boolean {
  const t = `${pattern.text} ${JSON.stringify(pattern.metadata)} ${pattern.source_section_id}`;
  const re = [
    /\b(19|20)\d{2}[-/]\d{1,2}[-/]\d{1,2}\b/,
    /\b\d{1,2}[-/]\d{1,2}[-/](19|20)\d{2}\b/,
    /\b([01]?[0-9]|2[0-3]):[0-5][0-9]\b/,
    /\b-?\d{1,3}\.\d{2,}\s*[, ]\s*-?\d{1,3}\.\d{2,}\b/,
    /\b(?:lat|latitude|lng|long|longitude|tz|timezone|birthplace|born on|born at)\b/i,
    /\b(jan|feb|mar|apr|may|jun|jul|aug|sep|oct|nov|dec)[a-z]*\s+\d{1,2},?\s+(19|20)\d{2}\b/i,
  ];
  return re.some((r) => r.test(t));
}

export class CloudflareVectorizePatternStore implements PatternVectorStore {
  constructor(private env: CloudflarePatternEnv) {}

  async upsertPatterns(patterns: ExtractedPattern[]): Promise<PatternVectorStoreResult> {
    if (!patterns.length) return { upserted: 0, skipped: 0 };

    const safe = patterns.filter((p) => !containsPrivateBirthData(p));
    const skipped = patterns.length - safe.length;
    if (!safe.length) return { upserted: 0, skipped };

    let vectors: number[][];
    try {
      const res = await this.env.AI.run('@cf/baai/bge-small-en-v1.5', { text: safe.map((p) => p.text) });
      vectors = res.data ?? [];
    } catch {
      return { upserted: 0, skipped: patterns.length };
    }
    if (vectors.length !== safe.length) {
      return { upserted: 0, skipped: patterns.length };
    }

    const upserts: Array<{ id: string; values: number[]; metadata: Record<string, any> }> = [];
    for (let i = 0; i < safe.length; i++) {
      const p = safe[i];
      const vec = vectors[i];
      const meta: Record<string, any> = {
        mode: p.metadata.mode,
        report_level: p.metadata.report_level,
        kind: p.kind,
        version: p.metadata.version,
        systems: p.metadata.systems,
      };

      const fullRecord = {
        ...p,
        persisted_at: new Date().toISOString(),
      };
      let durable = false;
      const key = `patterns/${p.id}.json`;
      try {
        if (this.env.PATTERNS_BUCKET) {
          await this.env.PATTERNS_BUCKET.put(key, JSON.stringify(fullRecord), {
            httpMetadata: { contentType: 'application/json' },
          });
          durable = true;
        } else if (this.env.PATTERNS_D1) {
          await this.env.PATTERNS_D1
            .prepare(
              'INSERT OR REPLACE INTO report_patterns (id, text, kind, source_section_id, source_rubric_score, metadata, created_at) VALUES (?, ?, ?, ?, ?, ?, ?)'
            )
            .bind(
              p.id,
              p.text,
              p.kind,
              p.source_section_id,
              p.source_rubric_score,
              JSON.stringify(p.metadata),
              new Date().toISOString()
            )
            .run();
          durable = true;
        }
      } catch {
        // fall through to meta fallback
      }

      if (!durable) {
        meta.text = p.text;
      }

      upserts.push({ id: p.id, values: vec, metadata: meta });
    }

    if (upserts.length) {
      await this.env.REPORT_PATTERNS.upsert(upserts);
    }
    return { upserted: upserts.length, skipped: skipped + (safe.length - upserts.length) };
  }
}

export class CloudflareVectorizePatternRetriever implements PatternVectorRetriever {
  constructor(private env: CloudflarePatternEnv) {}

  async retrieveSimilar(query: string, filters: RetrievalFilters = {}, limit = 5): Promise<RetrievedPattern[]> {
    let vec: number[];
    try {
      const res = await this.env.AI.run('@cf/baai/bge-small-en-v1.5', { text: [query] });
      vec = res.data?.[0] ?? [];
    } catch {
      return [];
    }
    if (!vec.length) return [];

    const filter = this.buildFilter(filters);
    let qres: { matches: Array<{ id: string; score: number; metadata?: Record<string, any> }> };
    try {
      qres = await this.env.REPORT_PATTERNS.query(vec, {
        topK: limit,
        filter,
        returnMetadata: true,
      });
    } catch {
      return [];
    }

    const results: RetrievedPattern[] = [];
    for (const m of qres.matches ?? []) {
      let text = (m.metadata?.text as string) || '';
      const id = m.id;
      try {
        if (this.env.PATTERNS_BUCKET) {
          const obj = await this.env.PATTERNS_BUCKET.get(`patterns/${id}.json`);
          if (obj) {
            const rec = JSON.parse(await obj.text());
            text = rec.text || text;
          }
        } else if (this.env.PATTERNS_D1) {
          const row = await this.env.PATTERNS_D1
            .prepare('SELECT text FROM report_patterns WHERE id = ?')
            .bind(id)
            .first<{ text?: string }>();
          if (row?.text) text = row.text;
        }
      } catch {
        // keep fallback text
      }
      if (text) {
        results.push({ text, score: m.score, metadata: m.metadata });
      }
    }
    return results;
  }

  private buildFilter(f: RetrievalFilters): Record<string, any> | undefined {
    const fl: Record<string, any> = {};
    if (f.mode) fl.mode = f.mode;
    if (f.report_level) fl.report_level = f.report_level;
    if (f.kind) fl.kind = f.kind;
    if (f.version) fl.version = f.version;
    if (f.systems && f.systems.length > 0) {
      fl.systems = f.systems;
    }
    return Object.keys(fl).length > 0 ? fl : undefined;
  }
}
