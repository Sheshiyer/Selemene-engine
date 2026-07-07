import { readFile, writeFile } from 'node:fs/promises';
import { join } from 'node:path';
import { loadBrandTokens } from './brand-loader.js';
import { renderReadingToHtml } from './html-renderer.js';
import { renderHtmlToPdf } from './pdf-renderer.js';

export interface RenderPipelineInput {
  sourcePackDir: string;
  outputDir: string;
  brandConfigPath: string;
}

export interface RenderPipelineOutput {
  htmlPath: string;
  pdfPath: string;
}

export async function renderLocalArtifacts(input: RenderPipelineInput): Promise<RenderPipelineOutput> {
  const readingMd = await readFile(join(input.sourcePackDir, 'reading.md'), 'utf-8');
  const manifest = JSON.parse(await readFile(join(input.sourcePackDir, 'manifest.json'), 'utf-8'));
  const brandTokens = await loadBrandTokens(input.brandConfigPath);
  const title = `L0 Integrated Kundali — ${manifest.person_id}`;

  const html = await renderReadingToHtml({ title, markdown: readingMd, brandTokens });
  const htmlPath = join(input.outputDir, 'reading.html');
  const pdfPath = join(input.outputDir, 'reading.pdf');

  await writeFile(htmlPath, html, 'utf-8');
  await renderHtmlToPdf({ htmlPath, pdfPath });

  return { htmlPath, pdfPath };
}
