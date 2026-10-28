import { describe, it, expect, afterEach, vi } from 'vitest';
import { fileURLToPath } from 'node:url';
import { loadBrandTokens, resolveBrandConfigPath } from './brand-loader.js';

const FIXTURE_BRAND_CONFIG = fileURLToPath(
  new URL('../../tests/fixtures/brand/brand-config.yaml', import.meta.url),
);

describe('loadBrandTokens', () => {
  afterEach(() => {
    vi.unstubAllEnvs();
  });

  it('loads colors and fonts from brand-config.yaml', async () => {
    const tokens = await loadBrandTokens(FIXTURE_BRAND_CONFIG);
    expect(tokens.colors.voidBlack).toBe('#101820');
    expect(tokens.colors.sacredGold).toBe('#C8A2FF');
    expect(tokens.colors.parchment).toBe('#F4F1EA');
    expect(tokens.colors.fixtureViolet).toBe('#6A4C93');
    expect(tokens.fonts.display).toBe('Fixture Display');
    expect(tokens.fonts.body).toBe('Fixture Serif');
    expect(tokens.fonts.mono).toBe('Fixture Mono');
  });

  it('falls back to BRAND_CONFIG_PATH when no path is passed', async () => {
    vi.stubEnv('BRAND_CONFIG_PATH', FIXTURE_BRAND_CONFIG);
    const tokens = await loadBrandTokens();
    expect(tokens.fonts.display).toBe('Fixture Display');
  });

  it('prefers an explicit path over BRAND_CONFIG_PATH', () => {
    vi.stubEnv('BRAND_CONFIG_PATH', '/nonexistent/brand-config.yaml');
    expect(resolveBrandConfigPath(FIXTURE_BRAND_CONFIG)).toBe(FIXTURE_BRAND_CONFIG);
  });

  it('throws when neither a path nor BRAND_CONFIG_PATH is set', () => {
    vi.stubEnv('BRAND_CONFIG_PATH', '');
    expect(() => resolveBrandConfigPath()).toThrow(/BRAND_CONFIG_PATH/);
  });
});
