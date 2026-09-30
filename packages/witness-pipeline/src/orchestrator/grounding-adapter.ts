// ─── Grounding Adapter ────────────────────────────────────────────────
// Bridges the existing PatternVectorRetriever interface (used by
// IntegratedReadingOrchestrator) to the legacy GroundingProvider interface
// defined in witness-agents/packages/orchestration/src/grounding.ts.
//
// Purpose: the reference-aligned route can consume the witness-wisdom-corpus
// via the existing retrieval contract without a new runtime or second client.
//
// Architecture:
//   PatternVectorRetriever.retrieveSimilar(query, filters?, limit?)
//     → translates query + filters into a RetrievalQuery-compatible shape
//     → calls the underlying GroundingProvider.retrieve(...)
//     → maps GroundedPassage[] → RetrievedPattern[]
//
// The underlying provider may be:
//   - A local Noop (no retrieval available → explicit RETRIEVAL_DISABLED status)
//   - A VectorizeGroundingProvider (wrangler CLI; read-only)
//   - Any other GroundingProvider implementation
//
// Retrieval result states are ALWAYS explicit (per contract-repair spec):
//   'success' | 'empty' | 'failure' | 'disabled'
// Callers may not treat missing retrieval as silent pass.

import type { PatternVectorRetriever, RetrievedPattern, RetrievalFilters } from '../patterns/retrieval.js';

// ─── Minimal GroundingProvider port (duck-typed; no import from sibling repo)

export interface GroundedPassage {
  id: string;
  source: string;
  excerpt: string;
  score: number;
  provenance: 'sourced-fact';
  metadata?: Record<string, unknown>;
}

export interface RetrievalQuery {
  subjectId: string;
  facts: Record<string, unknown>;
  perspective: string;
  taskId: string;
  maxPassages?: number;
}

export interface GroundingProvider {
  retrieve(query: RetrievalQuery): Promise<GroundedPassage[]>;
}

// ─── Retrieval result state ──────────────────────────────────────────

export type RetrievalState = 'success' | 'empty' | 'failure' | 'disabled';

export interface RetrievalReceipt {
  state: RetrievalState;
  count: number;
  query?: string;
  reason?: string;
}

// ─── Adapter ─────────────────────────────────────────────────────────

export interface GroundingAdapterOptions {
  /** Underlying provider. Pass NoopGroundingProvider for disabled state. */
  provider: GroundingProvider | null;
  /** Subject identifier for GroundingProvider scoping */
  subjectId?: string;
  /** Max passages to retrieve (default 5) */
  topK?: number;
  /** When true, this retriever is explicitly disabled (returns 'disabled' state) */
  disabled?: boolean;
}

/** Noop provider — always returns empty, distinct from failure. */
export const NoopGroundingProvider: GroundingProvider = {
  async retrieve(_query: RetrievalQuery): Promise<GroundedPassage[]> {
    return [];
  },
};

/**
 * An augmented PatternVectorRetriever that records the last retrieval state,
 * enabling downstream gates to distinguish success/empty/failure/disabled.
 */
export interface GroundingAdapterRetriever extends PatternVectorRetriever {
  lastReceipt: RetrievalReceipt;
  isDisabled(): boolean;
}

/**
 * Create a PatternVectorRetriever from a GroundingProvider.
 * The adapter translates query strings and RetrievalFilters into the
 * GroundingProvider's RetrievalQuery interface.
 *
 * Always returns explicit retrieval state; never treats failure as success.
 */
export function createGroundingAdapter(opts: GroundingAdapterOptions): GroundingAdapterRetriever {
  let lastReceipt: RetrievalReceipt = { state: 'disabled', count: 0, reason: 'not yet called' };

  return {
    get lastReceipt() { return lastReceipt; },
    isDisabled() { return !!opts.disabled || opts.provider === null; },

    async retrieveSimilar(
      query: string,
      filters?: RetrievalFilters,
      limit?: number,
    ): Promise<RetrievedPattern[]> {
      if (opts.disabled || opts.provider === null) {
        lastReceipt = { state: 'disabled', count: 0, query, reason: 'provider disabled or null' };
        return [];
      }

      const factContext: Record<string, unknown> = {};
      if (filters?.mode) factContext['mode'] = filters.mode;
      if (filters?.report_level) factContext['report_level'] = filters.report_level;
      if (filters?.language) factContext['language'] = filters.language;
      if (filters?.relationship_type) factContext['relationship_type'] = filters.relationship_type;
      if (filters?.systems) factContext['systems'] = filters.systems;

      const groundingQuery: RetrievalQuery = {
        subjectId: opts.subjectId ?? 'anonymous',
        facts: factContext,
        perspective: query,
        taskId: `retrieval-${Date.now()}`,
        maxPassages: limit ?? opts.topK ?? 5,
      };

      try {
        const passages = await opts.provider.retrieve(groundingQuery);

        if (passages.length === 0) {
          lastReceipt = { state: 'empty', count: 0, query, reason: 'provider returned no results' };
          return [];
        }

        const mapped: RetrievedPattern[] = passages.map((p) => ({
          text: p.excerpt,
          score: p.score,
          metadata: {
            id: p.id,
            source: p.source,
            provenance: p.provenance,
            ...p.metadata,
          },
        }));

        lastReceipt = { state: 'success', count: mapped.length, query };
        return mapped;
      } catch (err: unknown) {
        const reason = err instanceof Error ? err.message : String(err);
        lastReceipt = { state: 'failure', count: 0, query, reason };
        return [];
      }
    },
  };
}

/**
 * Create a disabled grounding adapter that records 'disabled' state.
 * Use when the witness-wisdom-corpus is not available in this run.
 */
export function createDisabledGroundingAdapter(): GroundingAdapterRetriever {
  return createGroundingAdapter({ provider: null, disabled: true });
}

/**
 * Render a retrieval receipt for inclusion in a section receipt or final report.
 * Renders differently per state so callers can never mistake failure for success.
 */
export function renderRetrievalReceipt(receipt: RetrievalReceipt): string {
  switch (receipt.state) {
    case 'success':
      return `Retrieval: success (${receipt.count} passages)`;
    case 'empty':
      return `Retrieval: empty result (0 passages returned for query)`;
    case 'failure':
      return `Retrieval: FAILED — ${receipt.reason ?? 'unknown error'}`;
    case 'disabled':
      return `Retrieval: DISABLED — ${receipt.reason ?? 'provider not configured'}`;
  }
}
