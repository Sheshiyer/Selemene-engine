/** Build numbered contact sheets covering every rasterized PDF page. */
import { readdir, mkdir, writeFile } from 'node:fs/promises';
import { join, resolve } from 'node:path';
import { pathToFileURL } from 'node:url';
const [directory] = process.argv.slice(2);
if (!directory || !process.env.WITNESS_NODE_PACKAGES) throw new Error('Expected raster directory and bundled node package path');
const sharpRoot = join(process.env.WITNESS_NODE_PACKAGES, 'sharp');
const sharpPackage = await Bun.file(join(sharpRoot, 'package.json')).json();
const sharp = (await import(pathToFileURL(join(sharpRoot, sharpPackage.main)).href)).default;
const root = resolve(directory);
const pages = (await readdir(root)).filter(f => /^page-\d+\.png$/.test(f)).sort((a, b) => Number(a.match(/\d+/)![0]) - Number(b.match(/\d+/)![0]));
await mkdir(join(root, 'contacts'), { recursive: true });
const width = 420, height = 566, columns = 3, rows = 4;
const contacts: string[] = [];
for (let offset = 0; offset < pages.length; offset += columns * rows) {
  const composites: any[] = [];
  for (const [i, file] of pages.slice(offset, offset + columns * rows).entries()) {
    const thumb = await sharp(join(root, file)).resize(width - 12, height - 26, { fit: 'inside' }).png().toBuffer();
    const x = i % columns * width, y = Math.floor(i / columns) * height;
    composites.push({ input: thumb, left: x + 6, top: y + 22 });
    // Label generated QA thumbnails; this is not part of a delivered report.
    const label = Buffer.from(`<svg width="${width}" height="20"><rect width="100%" height="100%" fill="#eeeeee"/><text x="8" y="15" font-size="14" font-family="Arial">Page ${offset + i + 1}</text></svg>`);
    composites.push({ input: label, left: x, top: y });
  }
  const target = join(root, 'contacts', `pages-${offset + 1}-${Math.min(offset + columns * rows, pages.length)}.png`);
  await sharp({ create: { width: width * columns, height: height * rows, channels: 3, background: '#cccccc' } }).composite(composites).png().toFile(target);
  contacts.push(target);
}
await writeFile(join(root, 'layout-qa-manifest.json'), JSON.stringify({ pageCount: pages.length, pages, contacts, visualReviewStatus: 'pending', method: 'Every PDF page rasterized; numbered contact sheets plus detailed page inspection' }, null, 2));
console.log(JSON.stringify({ pageCount: pages.length, contacts }));
