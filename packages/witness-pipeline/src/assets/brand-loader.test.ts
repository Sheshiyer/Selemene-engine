import { describe, it, expect } from 'vitest';
import { fileURLToPath } from 'node:url';
import { loadBrandTokens } from './brand-loader.js';

describe('loadBrandTokens', () => {
 it('loads colors and fonts from brand-config.yaml', async () => {
 const brandConfigPath = fileURLToPath(
 new URL('./fixtures/brand-config.yaml', import.meta.url),
 );
 const tokens = await loadBrandTokens(brandConfigPath);
 expect(tokens.colors.voidBlack).toBe('#070B1D');
 expect(tokens.fonts.display).toBe('Panchang');
 });
});
