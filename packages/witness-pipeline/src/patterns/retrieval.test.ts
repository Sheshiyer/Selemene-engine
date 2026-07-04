import { describe, expect, it } from 'vitest';
import { renderRetrievedPatternsForPrompt } from './retrieval.js';

describe('renderRetrievedPatternsForPrompt', () => {
  it('labels retrieved patterns as non-deterministic context', () => {
    const rendered = renderRetrievedPatternsForPrompt([
      { text: 'Saturn pressure plus Projector pacing can be framed as delayed recognition.' },
    ]);

    expect(rendered).toContain('Retrieved synthesis patterns are not deterministic facts');
    expect(rendered).toContain('delayed recognition');
  });
});
