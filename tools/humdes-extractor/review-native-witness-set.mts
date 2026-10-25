/** Full-context Jev review of exact native-authored chapters; never authors prose. */
import { readFile, writeFile, mkdir, readdir } from 'node:fs/promises';
import { resolve, join } from 'node:path';
import { createHash } from 'node:crypto';
import { createJevClient, loadTypesafeKey, noul } from '../../packages/witness-pipeline/src/jev/client.js';
import { createPartitionedJevClient } from '../../packages/witness-pipeline/src/jev/partitioned-client.js';
const root = resolve(import.meta.dir, 'output/gary-abitbol-2026-09-28');
const completion = join(root, 'completion-2026-10-04');
const directory = join(completion, 'native-final');
const execute = process.argv.includes('--execute');
const key = await loadTypesafeKey();
if (execute && !key) throw new Error('Jev key unavailable; no review or acceptance fabricated.');
const sha = (s: string) => createHash('sha256').update(s).digest('hex');
const vocabulary = JSON.parse(await readFile(join(completion, 'gene-keys-primary-vocabulary.json'), 'utf8'));
const editorial = JSON.parse(await readFile(join(completion, 'witness-reference-extract.json'), 'utf8'));
const method = { id: 'local:witness-method', source: `provided-docx:${editorial.source};sha256=${editorial.sha256}`, text: editorial.paragraphs.slice(12, 14).map((p: any) => p.text).join('\n') };
const primary = vocabulary.records.map((r: any) => ({ id: `primary:gk:${r.key}`, source: r.url, text: `Gene Key ${r.key}: ${r.labels.join(' – ')}. Author's official framework vocabulary; not a recalculated personal activation.` }));
const houseRegistry = JSON.parse(await readFile(join(root, 'reports-reference/editorial-primary-only-registry.json'), 'utf8'));
const housePassages = houseRegistry.additionIds.map((id: string) => {
  const r = houseRegistry.records[id];
  if (sha(r.text) !== r.sha256) throw new Error(`Primary house-passage drift ${id}`);
  return { id, source: `${r.provenance.url}#${r.provenance.locator}`, text: r.text };
});
const questions = {
  factual_accurate: noul('Are all personal chart claims supported by the named subject source records, allowing separately reviewed primary vocabulary to correct known generated-label errors? Missing claims must not be inferred. Judge only the record subset in this batch.', { true: 'All claims relevant to this batch are supported or explicitly tentative; no incorrect attribution', false: 'Contains an incorrect source value, invented chart fact or wrong subject attribution' }),
  guardrail_clean: noul('Is the chapter free of guaranteed outcomes, medical findings, personality certainty and chart-assigned business roles? Recorded periods and symbolic contemplation are allowed.', { true: 'Clean descriptive and reflective prose', false: 'Contains diagnosis, guarantee or chart-based role assignment' }),
  reader_clear: noul('Is this reader-facing prose free of engine field paths, schema identifiers, audit tables, review verdicts and internal pipeline explanations?', { true: 'Natural reader-facing report', false: 'Technical processing or review content leaks into report' }),
  contour_method: noul('Does the chapter retain individual agency, avoid claiming independent validation from dependent systems, and distinguish calculational discrepancies from symbolic interpretive tensions?', { true: 'Method preserved or chapter does not make these claims', false: 'Conflates source errors with meaningful contradictions or overstates convergence' }),
};
const originals: Record<string, string> = {
  sheshnarayan: 'sheshnarayan-en-2026-09-30T12-36-41-933Z',
  'mohan-kumar': 'mohan-kumar-en-2026-09-30T08-39-45-416Z',
  'gary-abitbol': 'gary-abitbol-fr-2026-09-30T08-35-03-147Z',
  'business-partners-triad': 'business-partners-triad-en-2026-09-30T09-02-48-504Z',
};
const names: Record<string, string> = { sheshnarayan: 'Sheshnarayan Cumbipuram Nateshan', 'mohan-kumar': 'Mohan Kumar', 'gary-abitbol': 'Gary Abitbol' };
const plans: any[] = [];
for (const file of (await readdir(directory)).filter(f => /-(en|fr)\.md$/.test(f)).sort()) {
  const slug = file.replace(/-(en|fr)\.md$/, '');
  const triad = slug === 'business-partners-triad';
  const subjects = triad ? Object.keys(names) : [slug];
  const sourceRecords = await Promise.all(subjects.map(async s => ({ subject: names[s], records: JSON.parse(await readFile(join(root, 'reports-reference/engines', `${s}.prod.json`), 'utf8')) })));
  if (triad) {
    const penta = JSON.parse(await readFile(join(root, 'raw/business-penta-source.json'), 'utf8'));
    sourceRecords.push({ subject: 'Provided diagram; attributed per-partner legend', records: [{ engine_id: 'business-penta-reference', result: penta }] });
  }
  const archived = JSON.parse(await readFile(join(root, 'reports-reference/runs', originals[slug], 'corpus-preflight.json'), 'utf8'));
  for (const p of archived.passages) if (sha(p.text) !== p.textHash) throw new Error(`Passage drift ${p.id}`);
  const passages = [...archived.passages.map((p: any) => ({ id: p.id, source: `archived-corpus:${p.id}`, text: p.text })), method, ...primary, ...housePassages];
  const text = await readFile(join(directory, file), 'utf8');
  const matches = [...text.matchAll(/^## (?:Opening|Ouverture|Part(?:ie)?\s+[IVX]+)\b.*$/gm)];
  if (matches.length !== 12) throw new Error(`${file}: expected Opening plus Parts I–XI; got ${matches.length}`);
  const reviewDir = join(directory, 'reviews', file.replace('.md', ''));
  await mkdir(reviewDir, { recursive: true });
  const chapters = matches.map((m, i) => ({ id: i ? `part${i}` : 'opening', title: m[0].slice(3), text: text.slice(m.index!, matches[i + 1]?.index ?? text.length).trim() }));
  const summary: any[] = [];
  for (const chapter of chapters) {
    const request = { state: {
      pass_id: chapter.id, pass_title: chapter.title, subjects: subjects.map(s => names[s]),
      engine_facts: 'Personal chart records remain immutable. Primary framework vocabulary is separately verified.\n## Full supplied source records\nComplete archived subject records:\n' + JSON.stringify(sourceRecords),
      section: chapter.text, attributed_framework_passages: passages,
    }, questions };
    await writeFile(join(reviewDir, `${chapter.id}.request.json`), JSON.stringify(request, null, 2));
    if (!execute) { summary.push({ id: chapter.id, status: 'prepared', chapterSha256: sha(chapter.text) }); continue; }
    const client = createPartitionedJevClient(createJevClient(key, { timeoutMs: 30000 })!, { onEvidence: e => { void writeFile(join(reviewDir, `${chapter.id}.partition-evidence.json`), JSON.stringify(e, null, 2)); } });
    const response = await client(request);
    await writeFile(join(reviewDir, `${chapter.id}.response.json`), JSON.stringify(response, null, 2));
    const clean = Object.keys(questions).every(q => response.answers[q]?.type === 'noul' && (response.answers[q] as any).noul >= .85);
    summary.push({ id: chapter.id, status: clean ? 'clean' : 'needs-review', chapterSha256: sha(chapter.text), answers: response.answers });
    console.log(JSON.stringify({ file, chapter: chapter.id, status: clean ? 'clean' : 'needs-review' }));
  }
  const result = { file, sha256: sha(text), route: 'native GPT authoring with archived source grounding and independent Jev chapter review', freshCloudflareRead: false, jevExecuted: execute, chapters: summary, finalAccepted: false };
  await writeFile(join(reviewDir, 'summary.json'), JSON.stringify(result, null, 2));
  plans.push(result);
}
await writeFile(join(directory, 'jev-review-manifest.json'), JSON.stringify({ preparedReports: plans.length, keyAvailable: !!key, execute, reports: plans }, null, 2));
console.log(JSON.stringify({ preparedReports: plans.length, keyAvailable: !!key, execute }));
