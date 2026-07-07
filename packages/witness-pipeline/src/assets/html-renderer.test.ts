import { describe, it, expect } from 'vitest';
import { renderReadingToHtml } from './html-renderer.js';
import type { BrandTokens } from './brand-loader.js';

const tokens: BrandTokens = {
  colors: { voidBlack: '#070B1D', sacredGold: '#C5A017', parchment: '#F0EDE3' },
  fonts: { display: 'Panchang', body: 'Satoshi', mono: 'SF Mono' },
};

describe('renderReadingToHtml', () => {
  it('wraps markdown in brand-styled HTML', async () => {
    const html = await renderReadingToHtml({
      title: 'L0 Integrated Kundali — Harshita',
      markdown: '# Hello\n\nYour nakshatra is Pushya.',
      brandTokens: tokens,
    });
    expect(html).toContain('<html');
    expect(html).toContain('Pushya');
    expect(html).toContain('--void-black: #070B1D');
  });
});
