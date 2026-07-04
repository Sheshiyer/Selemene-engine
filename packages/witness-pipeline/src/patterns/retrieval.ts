export interface RetrievedPattern {
  text: string;
  score?: number;
  metadata?: Record<string, unknown>;
}

export function renderRetrievedPatternsForPrompt(patterns: RetrievedPattern[]): string {
  if (patterns.length === 0) return '';
  return [
    'Retrieved synthesis patterns are not deterministic facts. Use them only for analogy, wording, and layering. Current chart data overrides retrieved context.',
    ...patterns.map((p, i) => `${i + 1}. ${p.text}`),
  ].join('\n');
}
