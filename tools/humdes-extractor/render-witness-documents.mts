import { resolve, dirname, basename, join } from 'node:path';
import { mkdir, readdir, unlink } from 'node:fs/promises';
import { pathToFileURL } from 'node:url';

// Supply paths from load_workspace_dependencies. No repository or global packages.
const [input, outputDirectory] = process.argv.slice(2);
const packageRoot = process.env.WITNESS_NODE_PACKAGES;
const soffice = process.env.WITNESS_SOFFICE;
const pdftoppm = process.env.WITNESS_PDFTOPPM;
if (!input || !outputDirectory || !packageRoot || !soffice || !pdftoppm)
  throw new Error('Usage: WITNESS_NODE_PACKAGES=... WITNESS_SOFFICE=... WITNESS_PDFTOPPM=... bun render-witness-documents.mts reading.md output-dir');
const d = await import(pathToFileURL(join(packageRoot, 'docx', 'dist', 'index.mjs')).href);
const { Document, Packer, Paragraph, TextRun, ExternalHyperlink, HeadingLevel, AlignmentType, Table, TableRow, TableCell, WidthType, BorderStyle, Footer, PageNumber } = d;
const md = await Bun.file(input).text();
const lines = md.replaceAll('\r\n', '\n').split('\n');
const children: any[] = [];
const inline = (s: string) => s.split(/(\[[^\]]+\]\([^)]+\)|\*\*[^*]+\*\*|\*[^*]+\*)/g).filter(Boolean).map(t => {
  const link = t.match(/^\[([^\]]+)\]\(([^)]+)\)$/);
  if (link) return new ExternalHyperlink({ link: link[2], children: [new TextRun({ text: link[1], style: 'Hyperlink' })] });
  return new TextRun({
  text: t.replace(/^\*\*|\*\*$/g, '').replace(/^\*|\*$/g, '').replace(/\[([^\]]+)\]\([^)]+\)/g, '$1').replaceAll('`', ''),
  bold: t.startsWith('**'), italics: t.startsWith('*') && !t.startsWith('**') });
});
const cell = (text: string, header: boolean, width: number) => new TableCell({
  width: { size: width, type: WidthType.DXA }, margins: { top: 100, bottom: 100, left: 110, right: 110 },
  shading: { fill: header ? 'E9E3DC' : 'FFFFFF' },
  children: [new Paragraph({ children: inline(text.trim()), spacing: { after: 40, line: 260 },
    style: header ? 'TableHeader' : 'TableBody' })] });
let firstTitle = true;
let chapterStarted = false;
for (let i = 0; i < lines.length; i++) {
  const line = lines[i].trim();
  if (!line || /^-{3,}$/.test(line)) continue;
  if (line.startsWith('|') && lines[i + 1]?.match(/^\s*\|?[\s:|-]+\|\s*$/)) {
    const records = [line]; i += 2;
    while (i < lines.length && lines[i].trim().startsWith('|')) records.push(lines[i++].trim());
    i--;
    const parsed = records.map(r => r.replace(/^\||\|$/g, '').split('|'));
    // Give long prose columns room without letting a single long cell
    // squeeze all other columns into unusable narrow strips.
    const scores = parsed[0].map((_: string, c: number) => {
      const lengths = parsed.map(r => (r[c] ?? '').trim().length).sort((a, b) => a - b);
      return Math.max(8, Math.min(40, Math.sqrt(lengths[Math.floor(lengths.length * .75)] || 8) * 5));
    });
    const total = scores.reduce((a: number, b: number) => a + b, 0);
    const widths = scores.map((s: number) => Math.floor(9648 * s / total));
    widths[widths.length - 1] += 9648 - widths.reduce((a: number, b: number) => a + b, 0);
    const border = { style: BorderStyle.SINGLE, size: 4, color: 'D9D9D9' };
    children.push(new Table({ width: { size: 9648, type: WidthType.DXA },
      columnWidths: widths, borders: { top: border, bottom: border, left: border, right: border, insideHorizontal: border, insideVertical: border },
      rows: parsed.map((r, n) => new TableRow({ tableHeader: n === 0, cantSplit: true, children: r.map((t, c) => cell(t, n === 0, widths[c])) })) }));
    children.push(new Paragraph({ text: '', spacing: { after: 100 } }));
    continue;
  }
  const heading = line.match(/^(#{1,4})\s+(.+)$/);
  if (heading) {
    const isTitle = heading[1].length === 1 && firstTitle;
    const level = heading[1].length;
    const isChapter = /^(?:Opening|Ouverture|Part(?:ie)?\s+[IVX]+|Introduction)\b/i.test(heading[2]);
    const isSubtitle = !firstTitle && !chapterStarted && !isChapter;
    if (isChapter) chapterStarted = true;
    children.push(new Paragraph({ children: inline(heading[2]),
      heading: isTitle ? HeadingLevel.TITLE : isSubtitle ? undefined : level <= 2 ? HeadingLevel.HEADING_1 : HeadingLevel.HEADING_2,
      alignment: isTitle || isSubtitle ? AlignmentType.CENTER : AlignmentType.LEFT,
      spacing: { before: isTitle ? 320 : 260, after: isTitle ? 240 : 120 },
      keepNext: true, pageBreakBefore: isChapter && children.length > 4 }));
    firstTitle = false; continue;
  }
  const list = line.match(/^(?:[-*]|\d+\.)\s+(.+)$/);
  children.push(new Paragraph({ children: inline(list ? list[1] : line.replace(/^>\s*/, '')),
    ...(list ? { bullet: { level: 0 } } : {}),
    spacing: { after: 140, line: 320 }, widowControl: true }));
}
const doc = new Document({ creator: 'Selemene', title: lines.find(l => l.startsWith('# '))?.slice(2) ?? basename(input),
  styles: { default: { document: { run: { font: 'Calibri', size: 22, color: '1A1A1A' } } },
    paragraphStyles: [
      { id: 'Title', name: 'Title', basedOn: 'Normal', run: { size: 56, bold: true, color: '000000' } },
      { id: 'Heading1', name: 'Heading 1', basedOn: 'Normal', run: { size: 36, bold: true, color: '000000' } },
      { id: 'Heading2', name: 'Heading 2', basedOn: 'Normal', run: { size: 28, bold: true, color: '000000' } },
      { id: 'TableHeader', name: 'Table Header', basedOn: 'Normal', run: { size: 19, bold: true } },
      { id: 'TableBody', name: 'Table Body', basedOn: 'Normal', run: { size: 19 } },
    ] },
  sections: [{ properties: { page: { size: { width: 12240, height: 15840 },
    margin: { top: 1440, bottom: 1440, left: 1296, right: 1296, header: 708, footer: 708 } } },
    footers: { default: new Footer({ children: [new Paragraph({ alignment: AlignmentType.CENTER,
      children: [new TextRun({ children: [PageNumber.CURRENT], size: 18, color: '666666' })] })] }) }, children }] });
const out = resolve(outputDirectory);
await mkdir(out, { recursive: true });
const stem = basename(input).replace(/\.md$/, '');
const docx = join(out, `${stem}.docx`);
await Bun.write(docx, await Packer.toBuffer(doc));
const profile = pathToFileURL(join(out, '.lo-profile')).href;
const conversion = Bun.spawnSync([soffice, `-env:UserInstallation=${profile}`, '--headless', '--convert-to', 'pdf', '--outdir', out, docx]);
if (conversion.exitCode) throw new Error(conversion.stderr.toString());
const pdf = join(out, `${stem}.pdf`);
if (!(await Bun.file(pdf).exists())) throw new Error(`PDF missing: ${conversion.stdout.toString()} ${conversion.stderr.toString()}`);
const qa = join(out, `${stem}-qa`); await mkdir(qa, { recursive: true });
for (const name of await readdir(qa)) if (/^page-\d+\.png$/.test(name)) await unlink(join(qa, name));
const raster = Bun.spawnSync([pdftoppm, '-r', '110', '-png', pdf, join(qa, 'page')]);
if (raster.exitCode) throw new Error(raster.stderr.toString());
console.log(JSON.stringify({ docx, pdf, qa, paragraphs: children.length }));
