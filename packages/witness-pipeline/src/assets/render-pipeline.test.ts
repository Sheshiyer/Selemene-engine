import { describe, it, expect } from 'vitest';
import { renderLocalArtifacts } from './render-pipeline.js';
import { mkdtempSync, readFileSync, statSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';

const FIXTURE_BRAND_CONFIG = fileURLToPath(
  new URL('../../tests/fixtures/brand/brand-config.yaml', import.meta.url),
);

describe('renderLocalArtifacts', () => {
  it('creates a non-empty, brand-styled reading.html and reading.pdf in output dir', async () => {
    const dir = mkdtempSync(join(tmpdir(), 'render-test-'));
    writeFileSync(join(dir, 'reading.md'), '# Synthetic Heading\n\nSynthetic body text.');
    writeFileSync(join(dir, 'engines.json'), JSON.stringify([]));
    writeFileSync(
      join(dir, 'manifest.json'),
      JSON.stringify({ person_id: 'synthetic-person' }),
    );

    const result = await renderLocalArtifacts({
      sourcePackDir: dir,
      outputDir: dir,
      brandConfigPath: FIXTURE_BRAND_CONFIG,
    });
    expect(result.htmlPath).toBe(join(dir, 'reading.html'));
    expect(result.pdfPath).toBe(join(dir, 'reading.pdf'));

    const html = readFileSync(result.htmlPath, 'utf-8');
    expect(html).toContain('--void-black: #101820;');
    expect(html).toContain('--parchment: #F4F1EA;');
    expect(html).toContain('font-family: Fixture Serif, serif;');
    expect(html).toContain('font-family: Fixture Display, serif;');

    const pdf = readFileSync(result.pdfPath);
    expect(statSync(result.pdfPath).size).toBeGreaterThan(0);
    expect(pdf.subarray(0, 5).toString('latin1')).toBe('%PDF-');
  }, 30_000);
});
