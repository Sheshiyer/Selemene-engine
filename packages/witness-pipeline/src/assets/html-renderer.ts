import type { BrandTokens } from './brand-loader.js';

export interface RenderHtmlInput {
  title: string;
  markdown: string;
  brandTokens: BrandTokens;
}

export async function renderReadingToHtml(input: RenderHtmlInput): Promise<string> {
  const { colors, fonts } = input.brandTokens;
  const colorVars = Object.entries(colors)
    .map(([k, v]) => `  --${k.replace(/([A-Z])/g, '-$1').toLowerCase()}: ${v};`)
    .join('\n');

  return `<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <title>${escapeHtml(input.title)}</title>
  <style>
    :root {
${colorVars}
    }
    body {
      font-family: ${fonts.body ?? 'serif'}, serif;
      background: var(--parchment);
      color: var(--void-black);
      max-width: 680px;
      margin: 0 auto;
      padding: 2rem;
    }
    h1, h2, h3 { font-family: ${fonts.display ?? 'serif'}, serif; }
  </style>
</head>
<body>
  ${markdownToHtml(input.markdown)}
</body>
</html>`;
}

function escapeHtml(text: string): string {
  return text
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;');
}

function markdownToHtml(markdown: string): string {
  return markdown
    .replace(/^# (.*$)/gim, '<h1>$1</h1>')
    .replace(/^## (.*$)/gim, '<h2>$1</h2>')
    .replace(/\*\*(.*?)\*\*/g, '<strong>$1</strong>')
    .replace(/\n/g, '\n  ');
}
