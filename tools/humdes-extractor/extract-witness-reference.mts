import { resolve } from 'node:path';
import { createHash } from 'node:crypto';

// Read-only OOXML extraction. Paragraphs include table cells in document order.
const [input, output] = process.argv.slice(2);
if (!input || !output) throw new Error('Usage: bun extract-witness-reference.mts input.docx output.json');
const result = Bun.spawnSync(['unzip', '-p', resolve(input), 'word/document.xml']);
if (result.exitCode) throw new Error(result.stderr.toString());
const xml = result.stdout.toString();
const decode = (s: string) => s.replace(/&#(\d+);/g, (_, n) => String.fromCodePoint(Number(n)))
  .replaceAll('&apos;', "'").replaceAll('&quot;', '"').replaceAll('&lt;', '<').replaceAll('&gt;', '>').replaceAll('&amp;', '&');
const paragraphs = [...xml.matchAll(/<w:p(?:\s[^>]*)?>[\s\S]*?<\/w:p>/g)].map(match => {
  const p = match[0];
  return { style: p.match(/<w:pStyle w:val="([^"]+)"/)?.[1] ?? 'Normal',
    text: decode([...p.matchAll(/<w:t(?:\s[^>]*)?>([\s\S]*?)<\/w:t>/g)].map(m => m[1]).join('')) };
}).filter(p => p.text.trim());
const bytes = await Bun.file(input).arrayBuffer();
await Bun.write(output, JSON.stringify({ source: resolve(input), sha256: createHash('sha256').update(Buffer.from(bytes)).digest('hex'),
  words: paragraphs.map(p => p.text).join(' ').split(/\s+/).length, paragraphs,
  sections: [...xml.matchAll(/<w:sectPr[\s\S]*?<\/w:sectPr>/g)].map(m => m[0]) }, null, 2));
console.log(JSON.stringify({ output: resolve(output), paragraphs: paragraphs.length }));
