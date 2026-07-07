import { describe, it, expect } from 'vitest';
import { loadBrandTokens } from './brand-loader.js';

describe('loadBrandTokens', () => {
  it('loads colors and fonts from brand-config.yaml', async () => {
    const tokens = await loadBrandTokens('/Volumes/madara/2026/twc-vault/01-Projects/tryambakam-noesis/brand-docs-final/tryambakam-noesis-aleph/brand-config.yaml');
    expect(tokens.colors.voidBlack).toBe('#070B1D');
    expect(tokens.fonts.display).toBe('Panchang');
  });
});
