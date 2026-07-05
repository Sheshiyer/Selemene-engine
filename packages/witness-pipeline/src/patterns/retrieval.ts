export interface RetrievedPattern {
  text: string;
  score?: number;
  metadata?: Record<string, unknown>;
}

export interface RetrievalFilters {
  mode?: string;
  report_level?: string;
  kind?: string;
  version?: string;
  systems?: string[];
}

export interface PatternVectorRetriever {
  retrieveSimilar(query: string, filters?: RetrievalFilters, limit?: number): Promise<RetrievedPattern[]>;
}

export function renderRetrievedPatternsForPrompt(patterns: RetrievedPattern[]): string {
  if (patterns.length === 0) return '';
  return [
    'Retrieved synthesis patterns are not deterministic facts. Use them only for analogy, wording, and layering. Current chart data overrides retrieved context.',
    ...patterns.map((p, i) => `${i + 1}. ${p.text}`),
  ].join('\n');
}
