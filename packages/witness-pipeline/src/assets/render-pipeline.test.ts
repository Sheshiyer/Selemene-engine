import { describe, it, expect } from 'vitest';
import { readFile, stat } from 'node:fs/promises';
import { mkdtempSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { renderLocalArtifacts } from './render-pipeline.js';

describe('renderLocalArtifacts', () => {
 it('creates reading.html and reading.pdf in output dir', async () => {
 const dir = mkdtempSync(join(tmpdir(), 'render-test-'));
    writeFileSync(join(dir, 'reading.md'), '# Hello\n\nPushya');
 writeFileSync(join(dir, 'engines.json'), JSON.stringify([]));
 writeFileSync(
 join(dir, 'manifest.json'),
 JSON.stringify({ person_id: 'harshita' }),
 );

 const brandConfigPath = fileURLToPath(
 new URL('./fixtures/brand-config.yaml', import.meta.url),
 );

 const result = await renderLocalArtifacts({
 sourcePackDir: dir,
 outputDir: dir,
 brandConfigPath,
 });
 expect(result.htmlPath).toBe(join(dir, 'reading.html'));
 expect(result.pdfPath).toBe(join(dir, 'reading.pdf'));

 const htmlStats = await stat(result.htmlPath);
 const pdfStats = await stat(result.pdfPath);
 expect(htmlStats.isFile()).toBe(true);
 expect(pdfStats.isFile()).toBe(true);
 expect(htmlStats.size).toBeGreaterThan(0);
 expect(pdfStats.size).toBeGreaterThan(0);

 const htmlContents = await readFile(result.htmlPath, 'utf-8');
 expect(htmlContents).toContain('Hello');
 expect(htmlContents).toContain('Pushya');
 });
});
