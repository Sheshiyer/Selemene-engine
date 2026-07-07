import { describe, it, expect } from 'vitest';
import { buildL0ArtifactPath } from './l0-render.js';

describe('buildL0ArtifactPath', () => {
  it('returns canonical factory local path', () => {
    const path = buildL0ArtifactPath('harshita');
    expect(path).toContain('witness-agents-archive/.premium-assets-witness-harshita/harshita/local');
  });
});
