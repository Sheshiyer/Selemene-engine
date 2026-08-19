import { chromium } from 'playwright';

export interface RenderPdfInput {
  htmlPath: string;
  pdfPath: string;
}

export async function renderHtmlToPdf(input: RenderPdfInput): Promise<void> {
  const browser = await chromium.launch();
  const page = await browser.newPage();
  await page.goto(`file://${input.htmlPath}`);
  await page.pdf({ path: input.pdfPath, format: 'A4', printBackground: true });
  await browser.close();
}
