import { describe, expect, it } from 'vitest';
import { NoopPatternVectorStore } from './vector-store.js';

describe('NoopPatternVectorStore', () => {
  it('accepts patterns without external side effects', async () => {
    const store = new NoopPatternVectorStore();
    await expect(store.upsertPatterns([])).resolves.toEqual({ upserted: 0, skipped: 0 });
  });
});
