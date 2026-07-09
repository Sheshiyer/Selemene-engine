import {
  CloudflareVectorizePatternStore,
  CloudflareVectorizePatternRetriever,
} from './patterns/cloudflare-vectorize-lite.js';
import type { ExtractedPattern } from './patterns/types.js';

interface Env {
  LLM_SECRETS: KVNamespace;
  REPORT_PATTERNS: VectorizeIndex;
  AI: Ai;
  PATTERNS_BUCKET?: R2Bucket;
  PATTERNS_D1?: D1Database;
}

const VERSION = '1.0.0';

export default {
  async fetch(request: Request, env: Env): Promise<Response> {
    const url = new URL(request.url);

    if (request.method === 'POST' && url.pathname === '/patterns') {
      return handleUpsert(request, env);
    }

    if (request.method === 'POST' && url.pathname === '/patterns/query') {
      return handleQuery(request, env);
    }

    if (request.method === 'GET' && url.pathname === '/') {
      return Response.json({
        name: 'selemene-pattern-memory',
        version: VERSION,
      });
    }

    return new Response('Not Found', { status: 404 });
  },
};

async function handleUpsert(request: Request, env: Env): Promise<Response> {
  let patterns: ExtractedPattern[];
  try {
    patterns = await request.json() as ExtractedPattern[];
  } catch {
    return new Response('Invalid JSON', { status: 400 });
  }

  if (!Array.isArray(patterns)) {
    return new Response('Expected an array of patterns', { status: 400 });
  }

  const store = new CloudflareVectorizePatternStore(env);
  const result = await store.upsertPatterns(patterns);

  return Response.json({ upserted: result.upserted });
}

async function handleQuery(request: Request, env: Env): Promise<Response> {
  let body: { query: string; filters?: Record<string, unknown>; limit?: number };
  try {
    body = await request.json() as typeof body;
  } catch {
    return new Response('Invalid JSON', { status: 400 });
  }

  if (!body.query || typeof body.query !== 'string') {
    return new Response('query is required', { status: 400 });
  }

  const retriever = new CloudflareVectorizePatternRetriever(env);
  const results = await retriever.retrieveSimilar(
    body.query,
    body.filters ?? {},
    body.limit ?? 10
  );

  return Response.json({ patterns: results });
}