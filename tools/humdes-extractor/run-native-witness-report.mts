/** Explicit native-GPT continuation using immutable archived grounding. */
import { readFile, writeFile, mkdir } from 'node:fs/promises';
import { resolve, join } from 'node:path';
import { createHash } from 'node:crypto';
import { parseModeDoc } from '../../packages/witness-pipeline/src/modes/parser.js';
import { localizeReferenceTitles } from '../../packages/witness-pipeline/src/orchestrator/reference-language.js';
import { localizeBusinessTriadTitles, loadTriadEngines } from '../../packages/witness-pipeline/src/orchestrator/business-triad-support.js';
import { IntegratedReadingOrchestrator } from '../../packages/witness-pipeline/src/orchestrator/integrated.js';
import { buildCombinedPassages } from '../../packages/witness-pipeline/src/orchestrator/primary-passage-source.js';
import { validatePrimaryPassageAdditions } from '../../packages/witness-pipeline/src/orchestrator/editorial-passage-evidence.js';
import { createJevClient, loadTypesafeKey } from '../../packages/witness-pipeline/src/jev/client.js';
import { createJevPassGate } from '../../packages/witness-pipeline/src/jev/pass-gate.js';
import { createPartitionedJevClient } from '../../packages/witness-pipeline/src/jev/partitioned-client.js';
import { nativeWitnessLlm } from './native-witness-llm.mts';
import type { PassageRetrievalReceipt, WitnessPersona } from '../../packages/witness-pipeline/src/orchestrator/reference-execution.js';
import type { SelemeneEngineOutput } from '../../packages/witness-pipeline/src/selemene/types.js';
const args = process.argv.slice(2);
const val = (flag: string, fallback?: string) => { const i = args.indexOf(flag); return i < 0 ? fallback : args[i + 1]; };
const repo = resolve(import.meta.dir, '../..');
const root = resolve(val('--package-dir', 'tools/humdes-extractor/output/gary-abitbol-2026-09-28')!);
const subject = val('--subject', 'sheshnarayan')!;
const language = val('--language', 'en')!;
if (!['sheshnarayan', 'mohan-kumar', 'gary-abitbol', 'business-partners-triad'].includes(subject) || !['en', 'fr'].includes(language)) throw new Error('Unknown subject/language');
const triad = subject === 'business-partners-triad';
const completion = join(root, 'completion-2026-10-04');
const queue = resolve(val('--queue', join(completion, 'native-queue'))!);
const out = resolve(val('--out', join(completion, 'native-runs', `${subject}-${language}`))!);
await mkdir(out, { recursive: true });
const hash = (s: string) => createHash('sha256').update(s).digest('hex');
// The user-supplied editorial reference is a local document, not a
// Cloudflare hit or an invented public primary-document URL.
const methodExtract = await readFile(join(completion, 'witness-reference-extract.json'), 'utf8');
const methodDocument = JSON.parse(methodExtract);
const methodPassage = {
  id: 'local:witness-alchemist:opening-method',
  source: `user-provided-editorial-document:${methodDocument.source}#paragraphs-12-13;sha256=${methodDocument.sha256}`,
  text: 'Editorial method reference only; not current subject facts or empirical proof.\n' +
    methodDocument.paragraphs.slice(12, 14).map((p: { text: string }) => p.text).join('\n'),
};
await writeFile(join(out, 'editorial-method-reference.snapshot.json'), JSON.stringify(methodPassage, null, 2), { flag: 'wx' });
const vocabularyBytes = await readFile(join(completion, 'gene-keys-primary-vocabulary.json'), 'utf8');
const vocabulary = JSON.parse(vocabularyBytes);
const vocabularyPassages = vocabulary.records.map((r: { key: number; labels: string[]; url: string }) => ({
  id: `primary:genekeys:vocabulary:${r.key}`,
  source: `independently-reviewed-primary-framework:${r.url}`,
  text: `Gene Key ${r.key}: Shadow ${r.labels[0]}; Gift ${r.labels[1]}; Siddhi ${r.labels[2]}. Framework vocabulary reviewed against the author's official library on ${vocabulary.reviewed_on}. Use this separately attributed vocabulary when saved generated labels conflict; do not alter source activations or infer missing lines.`,
}));
await writeFile(join(out, 'gene-keys-primary-vocabulary.snapshot.json'), vocabularyBytes, { flag: 'wx' });
const names: Record<string, string> = { sheshnarayan: 'Sheshnarayan Cumbipuram Nateshan', 'mohan-kumar': 'Mohan Kumar', 'gary-abitbol': 'Gary Abitbol' };
let bundles = triad ? await loadTriadEngines(root) : [{ subject, name: names[subject], role: 'primary', sourceBytes: await readFile(join(root, 'reports-reference/engines', `${subject}.prod.json`), 'utf8'), engines: [] as unknown[] }];
if (!triad) bundles[0].engines = JSON.parse(bundles[0].sourceBytes);
if (triad) {
  const sourceText = await readFile(join(root, 'raw/business-penta-source.json'), 'utf8');
  const source = JSON.parse(sourceText);
  const image = await readFile(join(root, 'raw/business-penta-context.png'));
  if (createHash('sha256').update(image).digest('hex') !== source.sha256) throw new Error('Penta screenshot hash mismatch');
  await writeFile(join(out, 'business-penta-source.snapshot.json'), sourceText, { flag: 'wx' });
  for (const b of bundles) {
    const legend = source.transcribed_legend[b.name];
    if (!legend) throw new Error(`Missing Penta legend: ${b.name}`);
    b.engines.push({ engine_id: 'business-penta-reference', result: {
      subject: b.name, source_kind: 'user-provided screenshot transcription, not an independent calculation',
      source_title: source.source_title, source_image_sha256: source.sha256,
      displayed_age_historical: legend.displayed_age, legend_gates: legend.gates,
      visible_function_labels: source.visible_function_labels, visible_axis_labels: source.visible_axis_labels,
      limits: source.limits,
    }, metadata: { source_record_kind: 'archived-user-supplied-diagram' } });
  }
}
const originalRuns: Record<string, string> = {
  sheshnarayan: 'sheshnarayan-en-2026-09-30T12-36-41-933Z',
  'mohan-kumar': 'mohan-kumar-en-2026-09-30T08-39-45-416Z',
  'gary-abitbol': 'gary-abitbol-fr-2026-09-30T08-35-03-147Z',
  'business-partners-triad': 'business-partners-triad-en-2026-09-30T09-02-48-504Z',
};
const archivedPath = join(root, 'reports-reference/runs', originalRuns[subject], 'corpus-preflight.json');
const archivedBytes = await readFile(archivedPath, 'utf8');
const archived = JSON.parse(archivedBytes);
if (archived.state !== 'success' || !archived.passages?.length) throw new Error('Archived corpus unavailable');
for (const p of archived.passages) if (hash(p.text) !== p.textHash) throw new Error(`Archived passage hash mismatch: ${p.id}`);
await writeFile(join(out, 'archived-corpus.snapshot.json'), archivedBytes, { flag: 'wx' });
const registryBytes = await readFile(join(root, 'reports-reference/editorial-primary-only-registry.json'), 'utf8');
const primary = validatePrimaryPassageAdditions({ registry: JSON.parse(registryBytes), reservedCFIds: archived.passages.map((p: any) => p.id) }).additions;
await writeFile(join(out, 'primary-passage-registry.snapshot.json'), registryBytes, { flag: 'wx' });
const mode = parseModeDoc(join(repo, 'packages/witness-pipeline/modes', triad ? 'business-triad-reference.md' : 'integrated-kundali-reference.md'));
if (triad) localizeBusinessTriadTitles(mode, language); else localizeReferenceTitles(mode, language);
const only = val('--section');
if (only) mode.frontmatter.pass_plan = mode.frontmatter.pass_plan.filter(p => p.id === only);
const persona = async (name: string): Promise<WitnessPersona> => {
  const sourcePath = resolve(repo, '../witness-agents/agents', name, 'IDENTITY.md');
  const identityText = await readFile(sourcePath, 'utf8');
  return { name, sourcePath, identityText, sourceHash: hash(identityText) };
};
const [aletheiosPersona, pichetPersona] = await Promise.all([persona('aletheios'), persona('pichet')]);
for (const b of bundles) await writeFile(join(out, `input.${b.subject}.original.snapshot.json`), b.sourceBytes, { flag: 'wx' });
await writeFile(join(out, 'engine-inputs.snapshot.json'), JSON.stringify(bundles.map(b => ({ subject: b.name, engines: b.engines })), null, 2), { flag: 'wx' });
const calls: unknown[] = [];
const receipt = (r: unknown) => { calls.push(r); void writeFile(join(out, 'native-call-receipts.json'), JSON.stringify(calls, null, 2)); };
const llm = nativeWitnessLlm(queue, 'generation', receipt);
const audit = nativeWitnessLlm(queue, 'audit', receipt);
const key = await loadTypesafeKey();
const partitions: unknown[] = [];
const baseClient = createJevClient(key);
const jevGate = baseClient ? createJevPassGate(triad ? createPartitionedJevClient(baseClient, { onEvidence: e => { partitions.push(e); void writeFile(join(out, 'jev-partition-receipts.json'), JSON.stringify(partitions, null, 2)); } }) : baseClient, 'active') : undefined;
await writeFile(join(out, 'route-provenance.json'), JSON.stringify({
  nativeGenerationAuthorized: 'User 2026-10-04: use native GPT models while Mac mini is offline',
  generation: 'native Codex GPT agents', sourceAudit: 'independent native Codex GPT agent',
  corpus: { freshness: 'archived', originalPath: archivedPath, snapshotSha256: hash(archivedBytes), freshCloudflareRead: false },
  jevAtStart: key ? 'active' : 'pending-key', finalAccepted: false,
}, null, 2));
const sections: unknown[] = [];
const orch = new IntegratedReadingOrchestrator({ mode, llm, jevGate, referenceExecution: {
  enabled: true, aletheiosPersona, pichetPersona, serialVoices: true,
  envelopeCheckpoint: async e => {
    await writeFile(join(out, `${e.public.section_id}.private-evidence-map.json`), JSON.stringify(e.private, null, 2));
    await writeFile(join(out, `${e.public.section_id}.section-envelope.json`), JSON.stringify(e, null, 2));
    await writeFile(join(out, `${e.public.section_id}.public.md`), e.public.content);
  },
  sourceAuditLlm: audit, requireSourceAudit: true,
  correctedMatrix: { register: 'L4-L5', language, subjectLabel: bundles.map(b => b.name).join(' × '), interpretationLlm: llm, maxTokensPerInterpretation: 8192 },
  retriever: async sectionId => {
    const combined = buildCombinedPassages({ cfPassages: archived.passages.map((p: any) => ({ id: p.id, source: `archived-cf-vectorize:witness-wisdom-corpus/${p.id}`, text: p.text })), primaryAdditions: primary });
    const passages = [...combined.passages, methodPassage, ...vocabularyPassages];
    const retrieval: PassageRetrievalReceipt = { section_id: sectionId, state: 'success', count: passages.length,
      source_ids: passages.map(p => p.id), passages_hash: hash(passages.map(p => p.text).join('\n')),
      primary_source_ids: [...combined.primarySourceIds, ...vocabularyPassages.map((p: { id: string }) => p.id)], cf_source_ids: [...combined.cfSourceIds],
      reason: 'Immutable archived corpus bytes; fresh live Cloudflare read unavailable on current host.' };
    await writeFile(join(out, `${sectionId}.corpus.json`), JSON.stringify({ retrieval, passages, freshness: 'archived', localEditorialSourceIds: [methodPassage.id] }, null, 2));
    return { passages, receipt: retrieval };
  },
}, sectionCheckpoint: async r => {
  sections.push(r);
  await writeFile(join(out, `${r.section_id}.receipt.json`), JSON.stringify(r, null, 2));
  if (r.outcome !== 'failed') await writeFile(join(out, `${r.section_id}.public.md`), r.raw.synthesis);
  console.log(JSON.stringify({ subject, language, section: r.section_id, outcome: r.outcome }));
} });
try {
  const result = await orch.run({ subjectNames: bundles.map(b => b.name), subjectRoles: bundles.map(b => ({ role: b.role, name: b.name })),
    engineResultsBySubject: bundles.map(b => b.engines as SelemeneEngineOutput[]), consciousnessLevel: 4, language,
    ...(triad ? { relationshipContext: { type: 'business-partners' as const } } : {}) });
  const reading = result.passes.map(p => `# ${p.title}\n\n${p.output.trim()}`).join('\n\n') + '\n';
  await writeFile(join(out, 'reading.md'), reading);
  await writeFile(join(out, 'result.json'), JSON.stringify(result, null, 2));
  for (const p of result.passes) if (p.envelope) {
    await writeFile(join(out, `${p.id}.private-evidence-map.json`), JSON.stringify(p.envelope.private, null, 2));
    await writeFile(join(out, `${p.id}.section-envelope.json`), JSON.stringify(p.envelope, null, 2));
  }
  await writeFile(join(out, 'status.json'), JSON.stringify({ generated: result.passes.length, required: mode.frontmatter.pass_plan.length,
    sourceAuditClean: sections.every((s: any) => s.source_audit?.status === 'clean'), finalAccepted: false,
    remaining: ['Jev if not completed', 'full source/coherence review', 'bilingual parity', 'DOCX/PDF render and all-page QA'] }, null, 2));
} catch (error) {
  await writeFile(join(out, 'status.json'), JSON.stringify({ status: 'failed', error: error instanceof Error ? error.message : String(error), finalAccepted: false }, null, 2));
  throw error;
}
