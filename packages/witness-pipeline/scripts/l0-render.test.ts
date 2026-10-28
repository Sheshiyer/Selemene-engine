import { describe, it, expect, afterEach, vi } from 'vitest';
import { join } from 'node:path';
import { buildL0ArtifactPath, buildL0SourcePackPath } from './l0-render.js';

describe('buildL0ArtifactPath', () => {
  afterEach(() => {
    vi.unstubAllEnvs();
  });

  it('returns canonical factory local path', () => {
    const path = buildL0ArtifactPath('harshita', '/archive');
    expect(path).toBe(join('/archive', '.premium-assets-witness-harshita', 'harshita', 'local'));
  });

  it('returns the sibling source-pack path', () => {
    const path = buildL0SourcePackPath('harshita', '/home/u/local/archive');
    expect(path).toBe(join('/home/u/local/archive', '.premium-assets-witness-harshita', 'harshita', 'source-pack'));
  });

  it('falls back to PREMIUM_ASSETS_ROOT', () => {
    vi.stubEnv('PREMIUM_ASSETS_ROOT', '/env-archive');
    expect(buildL0ArtifactPath('harshita')).toContain('/env-archive/.premium-assets-witness-harshita/harshita/local');
  });

  it('throws when no root is configured', () => {
    vi.stubEnv('PREMIUM_ASSETS_ROOT', '');
    expect(() => buildL0ArtifactPath('harshita')).toThrow(/PREMIUM_ASSETS_ROOT/);
  });
});
