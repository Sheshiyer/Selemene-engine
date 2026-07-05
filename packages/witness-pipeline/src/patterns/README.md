# Cloudflare Vectorize Pattern Memory

Minimal library-side surface for pattern memory using Cloudflare Vectorize.

## Interface

```ts
export interface PatternVectorStore {
  upsertPatterns(patterns: ExtractedPattern[]): Promise<PatternVectorStoreResult>;
}

export interface PatternVectorRetriever {
  retrieveSimilar(query: string, filters?: RetrievalFilters, limit?: number): Promise<RetrievedPattern[]>;
}

export interface RetrievedPattern {
  text: string;
  score?: number;
  metadata?: Record<string, unknown>;
}
```

## Worker Usage

```ts
import {
  CloudflareVectorizePatternStore,
  CloudflareVectorizePatternRetriever,
} from '@noesis/witness-pipeline/patterns/cloudflare-vectorize';
import { extractReportPatterns } from '@noesis/witness-pipeline/patterns/extractor';
import { renderRetrievedPatternsForPrompt } from '@noesis/witness-pipeline/patterns/retrieval';

export interface Env {
  REPORT_PATTERNS: VectorizeIndex;
  AI: Ai;
  PATTERNS_BUCKET?: R2Bucket;
  PATTERNS_D1?: D1Database;
}

export default {
  async fetch(req: Request, env: Env) {
    const store = new CloudflareVectorizePatternStore(env);
    const retriever = new CloudflareVectorizePatternRetriever(env);

    // Write path (after report generation)
    const patterns = extractReportPatterns({ mode, reportLevel, subjectNames, passes });
    const { upserted, skipped } = await store.upsertPatterns(patterns);

    // Retrieval path (before prompt construction)
    const similar = await retriever.retrieveSimilar('projector pacing authority', { mode: 'vedic', report_level: 'standard' }, 4);
    const contextBlock = renderRetrievedPatternsForPrompt(similar);

    return new Response(JSON.stringify({ upserted, skipped, contextBlock }));
  },
};
```

## wrangler.jsonc Bindings

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
  },
  "r2_buckets": [
    { "binding": "PATTERNS_BUCKET", "bucket_name": "selemene-patterns" }
  ],
  "d1_databases": [
    { "binding": "PATTERNS_D1", "database_name": "selemene-patterns" }
  ]
}
```

## Privacy Contract

- Never embed or store private birth data: names, dates, times, lat/lng, birthplaces, timezones, or raw charts.
- `extractReportPatterns` already anonymizes names; the Vectorize store additionally rejects any candidate containing private patterns via `containsPrivateBirthData`.
- Only safe metadata is indexed: `mode`, `report_level`, `kind`, `version`, `systems`.
- Full text lives in R2 (preferred) or D1. Vectorize holds vectors + safe metadata only.
- Retrieved patterns are **always** prefixed by the warning label in `renderRetrievedPatternsForPrompt`:
  > "Retrieved synthesis patterns are not deterministic facts. Use them only for analogy, wording, and layering. Current chart data overrides retrieved context."

## Durable Storage Fallback

- If neither `PATTERNS_BUCKET` nor `PATTERNS_D1` is bound, the implementation falls back to embedding the text into Vectorize metadata (still safe fields only).
- Prefer R2 for blobs + D1/Postgres for metadata when the Worker platform supports it.

## Tests

All new behavior is covered in `cloudflare-vectorize.test.ts` using mocks. Noop path remains for unit tests and non-Worker environments.
