import { describe, expect, it } from 'vitest';
import { renderRetrievedPatternsForPrompt, type RetrievalFilters } from './retrieval.js';

describe('renderRetrievedPatternsForPrompt', () => {
  it('labels retrieved patterns as non-deterministic context', () => {
    const rendered = renderRetrievedPatternsForPrompt([
      { text: 'Saturn pressure plus Projector pacing can be framed as delayed recognition.' },
    ]);

    expect(rendered).toContain('Retrieved synthesis patterns are not deterministic facts');
    expect(rendered).toContain('delayed recognition');
  });
});

describe('RetrievalFilters', () => {
  it('accepts relationship_type and language filters (shape only)', () => {
    const f: RetrievalFilters = { mode: 'mother-son', report_level: 'L2', relationship_type: 'family', language: 'hi' };
    expect(f.relationship_type).toBe('family');
    expect(f.language).toBe('hi');
  });
});
