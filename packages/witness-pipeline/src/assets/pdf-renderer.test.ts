import { describe, it, expect } from 'vitest';
import { renderHtmlToPdf } from './pdf-renderer.js';
import { writeFile, unlink, stat } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';

describe('renderHtmlToPdf', () => {
  it('writes a PDF file', async () => {
    const htmlPath = join(tmpdir(), 'test.html');
    const pdfPath = join(tmpdir(), 'test.pdf');
    await writeFile(htmlPath, '<html><body>Hello</body></html>');
    await renderHtmlToPdf({ htmlPath, pdfPath });
    const stats = await stat(pdfPath);
    expect(stats.size).toBeGreaterThan(0);
    await unlink(htmlPath);
    await unlink(pdfPath);
  });
});
