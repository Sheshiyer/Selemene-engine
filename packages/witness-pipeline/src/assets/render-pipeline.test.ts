import { describe, it, expect } from 'vitest';
import { renderLocalArtifacts } from './render-pipeline.js';
import { mkdtempSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';

describe('renderLocalArtifacts', () => {
  it('creates reading.html and reading.pdf in output dir', async () => {
    const dir = mkdtempSync(join(tmpdir(), 'render-test-'));
    writeFileSync(join(dir, 'reading.md'), '# Hello\n\nPushya');
    writeFileSync(join(dir, 'engines.json'), JSON.stringify([]));
    writeFileSync(
      join(dir, 'manifest.json'),
      JSON.stringify({ person_id: 'harshita' }),
    );

    const result = await renderLocalArtifacts({
      sourcePackDir: dir,
      outputDir: dir,
      brandConfigPath: '/Volumes/madara/2026/twc-vault/01-Projects/tryambakam-noesis/brand-docs-final/tryambakam-noesis-aleph/brand-config.yaml',
    });
    expect(result.htmlPath).toBe(join(dir, 'reading.html'));
    expect(result.pdfPath).toBe(join(dir, 'reading.pdf'));
  });
});
