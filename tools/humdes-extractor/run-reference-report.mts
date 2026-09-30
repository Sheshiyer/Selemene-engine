/** Real CF-grounded, persona-sourced section execution. Old reports stay intact. */
import { promises as fs, writeFileSync } from 'node:fs';
import { resolve, join, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import { createHash } from 'node:crypto';
import { parseModeDoc } from '../../packages/witness-pipeline/src/modes/parser.js';
import { localizeReferenceTitles } from '../../packages/witness-pipeline/src/orchestrator/reference-language.js';
import { IntegratedReadingOrchestrator } from '../../packages/witness-pipeline/src/orchestrator/integrated.js';
import { VectorizeCorpusAdapter, selectCorpusIds } from '../../packages/witness-pipeline/src/grounding/index.js';
import { createOmniRouteLlmCall } from '../../packages/witness-pipeline/scripts/lib/multi-llm.js';
import { createJevClient, loadTypesafeKey } from '../../packages/witness-pipeline/src/jev/client.js';
import { createJevPassGate } from '../../packages/witness-pipeline/src/jev/pass-gate.js';
import type { WitnessPersona, SectionExecutionReceipt, PassageRetrievalReceipt } from '../../packages/witness-pipeline/src/orchestrator/reference-execution.js';
import { verifyReferenceExecution } from '../../packages/witness-pipeline/src/orchestrator/reference-verification.js';
import { validateResumeDir } from '../../packages/witness-pipeline/src/orchestrator/resume-validation.js';
import { loadEditorialPrefix, type EditorialPrefixSection, type EditorialPrefixPending } from '../../packages/witness-pipeline/src/orchestrator/editorial-prefix.js';
import { runFinalVerification } from '../../packages/witness-pipeline/src/orchestrator/final-verification.js';
import { validatePrimaryPassageAdditions } from '../../packages/witness-pipeline/src/orchestrator/editorial-passage-evidence.js';
import { buildCombinedPassages } from '../../packages/witness-pipeline/src/orchestrator/primary-passage-source.js';
import type { EditorialPassageAddition } from '../../packages/witness-pipeline/src/orchestrator/editorial-passage-evidence.js';

const args = process.argv.slice(2);
const val = (key: string, fallback?: string) => { const n = args.indexOf(key); return n < 0 ? fallback : args[n + 1]; };
const repo = resolve(dirname(fileURLToPath(import.meta.url)), '../..');
const packageArg = val('--package-dir');
if (!packageArg) throw new Error('Required: --package-dir <source package>');
const root = resolve(packageArg);
const subject = val('--subject', 'sheshnarayan')!;
if (!['gary-abitbol', 'mohan-kumar', 'sheshnarayan'].includes(subject)) throw new Error('Unknown subject slug');
const name = subject === 'gary-abitbol' ? 'Gary Abitbol' : subject === 'mohan-kumar' ? 'Mohan Kumar' : 'Sheshnarayan Cumbipuram Nateshan';
const language = val('--language', 'en')!;
if (!['en', 'fr'].includes(language)) throw new Error('Unsupported language');
const accountId = val('--account-id', process.env.CLOUDFLARE_ACCOUNT_ID);
if (!accountId || !/^[a-f0-9]{32}$/.test(accountId)) throw new Error('Pass the verified --account-id or CLOUDFLARE_ACCOUNT_ID');
const resumeFromArg = val('--resume-from');
const editorialPrefixArg = val('--editorial-prefix');
const passageRegistryArg = val('--passage-registry');
const primaryRegistryArg = val('--primary-registry');
const baselineReceiptArg = val('--baseline-receipt');
if (passageRegistryArg && !editorialPrefixArg) throw new Error('--passage-registry currently requires --editorial-prefix');
if (primaryRegistryArg && passageRegistryArg) throw new Error('Generation primary registry and editorial supplement registry cannot be combined');
if (resumeFromArg && editorialPrefixArg) throw new Error('--resume-from and --editorial-prefix are mutually exclusive');
const hash = (text: string) => createHash('sha256').update(text).digest('hex');
const inputPath = join(root, 'reports-reference/engines', `${subject}.prod.json`);
const inputText = await fs.readFile(inputPath, 'utf8');
const engines = JSON.parse(inputText);
const byId = Object.fromEntries(engines.map((e: any) => [e.engine_id, e.result]));
const hd = byId['human-design']; const gk = byId['gene-keys']; const vim = byId.vimshottari;
if (!hd || !byId['vedic-kundali']) throw new Error('Prepare reference inputs with full natal foundation first');
const selected = selectCorpusIds({
  'human-design': { type: hd.hd_type, profile: hd.profile, authority: hd.authority, defined_centers: hd.defined_centers, channels: hd.active_channels },
  'gene-keys': { gates: [...new Set((gk?.active_keys ?? []).map((k: any) => Number.parseInt(String(k.key_number ?? k.key ?? k.gene_key ?? k.number ?? k.gate), 10)).filter((n: number) => n >= 1 && n <= 64))] as number[] },
  vimshottari: { current_dasha: vim?.current_period?.mahadasha?.planet, current_antardasha: vim?.current_period?.antardasha?.planet, nakshatra: vim?.birth_nakshatra },
});
const planetaryNakshatraIds = Object.values(byId['vedic-kundali'].planets)
  .map((planet: any) => planet?.nakshatra_num)
  .filter((number: unknown): number is number => Number.isInteger(number) && Number(number) >= 1 && Number(number) <= 27)
  .map(number => `sw:vim:nakshatra:${number}:desc`);
const ids = [...new Set([...selected.ids, ...Object.keys(byId['vedic-kundali'].planets).map(p => `sw:vim:planet:${p.toLowerCase()}:desc`), ...planetaryNakshatraIds])];
const corpus = new VectorizeCorpusAdapter({ accountId, indexName: 'witness-wisdom-corpus', timeoutMs: 45000 });
async function loadPersona(name: string): Promise<WitnessPersona> {
  const sourcePath = resolve(repo, '../witness-agents/agents', name, 'IDENTITY.md');
  const identityText = await fs.readFile(sourcePath, 'utf8');
  return { name, sourcePath, identityText, sourceHash: hash(identityText) };
}
const [aletheiosPersona, pichetPersona] = await Promise.all([loadPersona('aletheios'), loadPersona('pichet')]);
const runId = `${subject}-${language}-${new Date().toISOString().replace(/[:.]/g, '-')}`;
const out = join(root, 'reports-reference/runs', runId);
await fs.mkdir(out, { recursive: true });
await fs.writeFile(join(out, 'input.snapshot.json'), inputText);
let baselineReceipt: { path: string; sha256: string } | undefined;
if (baselineReceiptArg) {
  const baselinePath = resolve(baselineReceiptArg);
  const baselineBytes = await fs.readFile(baselinePath, 'utf8');
  baselineReceipt = { path: baselinePath, sha256: hash(baselineBytes) };
  await fs.writeFile(join(out, 'baseline-part1.receipt.snapshot.json'), baselineBytes, { flag: 'wx' });
}
const preflight = await corpus.getVectors(ids);
await fs.writeFile(join(out, 'corpus-preflight.json'), JSON.stringify(preflight, null, 2));
if (preflight.state !== 'success' || !preflight.passages.length) throw new Error(`CF grounding unavailable: ${preflight.state}: ${preflight.reason ?? ''}`);
// ── Optional independent primary passage registry (fresh generation) ─
// When --primary-registry is passed the additions listed in
// registry.additionIds are appended (in order) to every retriever call
// alongside the live CF passages. Live CF is still mandatory (checked
// above). Registry bytes and hash are snapshotted so the final verifier
// can bind them without trusting the receipt's self-asserted claim.
let primaryAdditions: readonly EditorialPassageAddition[] = [];
let primaryRegistryBytes: string | undefined;
let primaryRegistrySha256: string | undefined;
let primaryRegistryPath: string | undefined;
if (primaryRegistryArg) {
  primaryRegistryPath = resolve(primaryRegistryArg);
  primaryRegistryBytes = await fs.readFile(primaryRegistryPath, 'utf8');
  primaryRegistrySha256 = hash(primaryRegistryBytes);
  const parsed = JSON.parse(primaryRegistryBytes);
  const validated = validatePrimaryPassageAdditions({
    registry: parsed,
    reservedCFIds: preflight.passages.map(p => p.id),
  });
  primaryAdditions = validated.additions;
  await fs.writeFile(join(out, 'primary-passage-registry.snapshot.json'), primaryRegistryBytes, { flag: 'wx' });
  console.log(JSON.stringify({ stage: 'primary-registry-loaded', additions: primaryAdditions.length, sha256: primaryRegistrySha256, path: primaryRegistryPath }));
}
const runtimeFiles = ['packages/witness-pipeline/src/orchestrator/reference-language.ts', 'packages/witness-pipeline/src/orchestrator/reference-source-audit.ts', 'packages/witness-pipeline/src/orchestrator/reference-verification.ts', 'packages/witness-pipeline/src/orchestrator/final-verification.ts', 'tools/humdes-extractor/run-reference-report.mts', 'packages/witness-pipeline/src/orchestrator/resume-validation.ts', 'packages/witness-pipeline/src/orchestrator/rubric.ts', 'packages/witness-pipeline/modes/integrated-kundali-reference.md', 'packages/witness-pipeline/src/orchestrator/integrated.ts', 'packages/witness-pipeline/src/orchestrator/reference-execution.ts', 'packages/witness-pipeline/src/orchestrator/engine-facts.ts', 'packages/witness-pipeline/src/grounding/vectorize-corpus-adapter.ts', 'packages/witness-pipeline/src/jev/pass-gate.ts', 'packages/witness-pipeline/scripts/lib/multi-llm.ts'];
const runtimeHashes = Object.fromEntries(await Promise.all(runtimeFiles.map(async path => [path, hash(await fs.readFile(join(repo, path), 'utf8'))])));
await fs.writeFile(join(out, 'inputs.json'), JSON.stringify({
  inputPath, inputHash: hash(inputText), subject, language, accountId,
  index: 'witness-wisdom-corpus',
  personaHashes: [aletheiosPersona.sourceHash, pichetPersona.sourceHash],
  runtimeHashes,
  serialVoices: process.argv.includes('--serial-voices'),
  requestedCombo: val('--combo', 'noesis-execute'),
  requestedAuditCombo: val('--audit-combo', 'noesis-verify'),
  ...(baselineReceipt ? { baselineReceipt } : {}),
  ...(primaryRegistrySha256 ? {
    primaryRegistry: {
      path: primaryRegistryPath,
      sha256: primaryRegistrySha256,
      additionIds: primaryAdditions.map(a => a.id),
    },
  } : {}),
}, null, 2));
console.log(JSON.stringify({ stage: 'preflight', passages: preflight.passages.length, requested: ids.length, output: out }));
if (args.includes('--preflight')) process.exit(0);
const mode = parseModeDoc(join(repo, 'packages/witness-pipeline/modes/integrated-kundali-reference.md'));
localizeReferenceTitles(mode, language);

const onlySection = val('--section');
if (onlySection) {
  if (!mode.frontmatter.pass_plan.some(p => p.id === onlySection)) throw new Error('Unknown section');
  mode.frontmatter.pass_plan = mode.frontmatter.pass_plan.filter(p => p.id === onlySection);
}

// ── Resume: validate prior run and build reuse map ─────────────────────
let reusedSections: Map<string, { output: string; receipt: SectionExecutionReceipt }> | undefined;
let resumeProvenance: string | undefined;
let editorialPrefixSections: EditorialPrefixSection[] | undefined;
let editorialPrefixPending: EditorialPrefixPending[] | undefined;
let editorialPrefixProvenance: string | undefined;

if (resumeFromArg) {
  const priorRunDir = resolve(resumeFromArg);
  const allSectionIds = mode.frontmatter.pass_plan.map(p => p.id);
  const validation = await validateResumeDir(priorRunDir, allSectionIds, {
    sourceTexts: {
      ...Object.fromEntries(preflight.passages.map(p => [p.id, p.text])),
      ...Object.fromEntries(primaryAdditions.map(a => [a.id, a.text])),
    },
    inputHash: hash(inputText),
    subject,
    language,
    accountId,
    index: 'witness-wisdom-corpus',
    aletheiosPersonaHash: aletheiosPersona.sourceHash,
    pichetPersonaHash: pichetPersona.sourceHash,
    runtimeHashes,
  });
  if (!validation.valid) {
    // Write failure to status.json before throwing so the caller can inspect it
    await fs.writeFile(join(out, 'status.json'), JSON.stringify({ status: 'resume-rejected', reason: validation.reason, finalAccepted: false }, null, 2));
    throw new Error(validation.reason);
  }
  reusedSections = new Map(validation.reusedSections.map(s => [s.section_id, { output: s.acceptedOutput, receipt: s.receipt }]));
  resumeProvenance = validation.provenance;
  console.log(JSON.stringify({ stage: 'resume-validated', reused: [...reusedSections.keys()], priorRunDir }));
}

if (editorialPrefixArg) {
  const manifestPath = resolve(editorialPrefixArg);
  const allSectionIds = mode.frontmatter.pass_plan.map(p => p.id);
  let trustedPassageSources;
  if (passageRegistryArg) {
    const registryBytes = await fs.readFile(resolve(passageRegistryArg), 'utf8');
    const registry = JSON.parse(registryBytes);
    if (!registry.records || typeof registry.records !== 'object' || Array.isArray(registry.records)) {
      throw new Error('Independent passage registry records missing');
    }
    for (const [id, record] of Object.entries(registry.records) as [string, any][]) {
      if (record?.provenance?.kind !== 'cf-vectorize') continue;
      const fresh = preflight.passages.find(p => p.id === id);
      if (record.id !== id || record.provenance.account !== accountId ||
          record.provenance.index !== 'witness-wisdom-corpus' || !fresh ||
          fresh.text !== record.text || hash(fresh.text) !== record.sha256) {
        throw new Error('Independent passage registry CF readback mismatch: ' + id);
      }
    }
    trustedPassageSources = registry.records;
    await fs.writeFile(join(out, 'editorial-source-registry.snapshot.json'), registryBytes, { flag: 'wx' });
  }
  const prefixResult = await loadEditorialPrefix(manifestPath, allSectionIds, {
    inputHash: hash(inputText),
    subject,
    language,
    accountId,
    index: 'witness-wisdom-corpus',
    aletheiosPersonaHash: aletheiosPersona.sourceHash,
    pichetPersonaHash: pichetPersona.sourceHash,
    sourceTexts: Object.fromEntries([...preflight.passages, ...primaryAdditions].map(p => [p.id, p.text])),
    primaryRegistrySha256,
    implementationSourceTexts: { 'crates/engine-transits/src/ephemeris.rs': await fs.readFile(join(repo, 'crates/engine-transits/src/ephemeris.rs'), 'utf8') },
    trustedPassageSources,
    expectedCF: { account: accountId, index: 'witness-wisdom-corpus' },
  });
  if (!prefixResult.valid) {
    // Persist rejection so the operator can inspect it before the throw unwinds the run.
    await fs.writeFile(join(out, 'status.json'), JSON.stringify({ status: 'editorial-prefix-rejected', reason: prefixResult.reason, finalAccepted: false, generationOnly: true, pendingEditorialAcceptance: true }, null, 2));
    throw new Error(prefixResult.reason);
  }
  editorialPrefixSections = prefixResult.prefixSections;
  editorialPrefixPending = prefixResult.pendingAmbiguity;
  editorialPrefixProvenance = prefixResult.provenance;

  // Copy the base receipts + separate revision/audit/jev artifacts into the
  // new run folder so the immutable evidence travels with the resumed report.
  const provenanceDir = join(out, 'editorial-prefix');
  await fs.mkdir(join(provenanceDir, 'original-receipts'), { recursive: true });
  await fs.mkdir(join(provenanceDir, 'editorial-revisions'), { recursive: true });
  await fs.mkdir(join(provenanceDir, 'audits'), { recursive: true });
  await fs.mkdir(join(provenanceDir, 'jev'), { recursive: true });
  for (const section of editorialPrefixSections) {
    await fs.copyFile(section.paths.receipt, join(provenanceDir, 'original-receipts', `${section.section_id}.receipt.json`));
    await fs.copyFile(section.paths.revision, join(provenanceDir, 'editorial-revisions', `${section.section_id}.editorial-revision.json`));
    await fs.copyFile(section.paths.audit, join(provenanceDir, 'audits', `${section.section_id}.fresh-audit.json`));
    await fs.copyFile(section.paths.jev, join(provenanceDir, 'jev', `${section.section_id}.fresh-jev.json`));
  }
  await fs.writeFile(join(out, 'editorial-prefix-provenance.json'), JSON.stringify({
    manifestPath,
    baseRunDir: prefixResult.baseRunDir,
    generationOnly: true,
    pendingEditorialAcceptance: true,
    provenance: editorialPrefixProvenance,
    pendingAmbiguity: editorialPrefixPending,
    sections: editorialPrefixSections.map(s => ({
      section_id: s.section_id,
      baseReceiptOutputHash: s.originalReceipt.output_hash,
      baseReceiptOutcome: s.originalReceipt.outcome,
      revisedOutputHash: s.revisedOutputHash,
      revisionTimestamp: s.revision.timestamp,
      revisionReviewer: s.revision.reviewer,
      patches: s.revision.patches.length,
      freshAuditInputHash: s.freshAudit.audit.input_sha256,
      freshAuditOutputHash: s.freshAudit.audit.output_sha256,
      freshJevVerdicts: s.freshJev.receipt.verdicts,
      artifactPaths: s.paths,
    })),
  }, null, 2));

  // Build reusedSections with the REVISED text as the output and the ORIGINAL
  // receipt as the checkpoint payload. integrated.ts already supports the two
  // being different — the receipt is checkpointed unchanged (raw synthesis,
  // output_hash, outcome all preserved) while the revised text drives the
  // assembled report and rubric. The strict final verifier will still detect
  // the original-vs-editorial mismatch until a separate provenance-aware
  // acceptance path ships; we intentionally do not weaken it here.
  reusedSections = new Map(editorialPrefixSections.map(s => [s.section_id, { output: s.revisedOutput, receipt: s.originalReceipt }]));
  console.log(JSON.stringify({ stage: 'editorial-prefix-loaded', reused: [...reusedSections.keys()], pendingAmbiguity: editorialPrefixPending.map(p => p.section_id), baseRunDir: prefixResult.baseRunDir }));
}

const llmReceipts: unknown[] = [];
const llm = createOmniRouteLlmCall(val('--combo', 'noesis-execute')!, { temperature: 0.4, timeout_ms: 240000, onReceipt: receipt => {
  llmReceipts.push(receipt);
  writeFileSync(join(out, 'llm-receipts.json'), JSON.stringify(llmReceipts, null, 2));
  console.log(JSON.stringify({ stage: 'llm-complete', call: llmReceipts.length, model: receipt.returned_model, elapsedMs: receipt.elapsed_ms }));
} });
// Independent audit LLM — separate combo so the auditor is not the same call
// that produced the section. Falls back to the main combo when --audit-combo
// is omitted; auditor receipts are persisted alongside llm-receipts.
const auditReceipts: unknown[] = [];
const sourceAuditLlm = createOmniRouteLlmCall(val('--audit-combo', 'noesis-verify')!, { temperature: 0, timeout_ms: 180000, onReceipt: receipt => {
  auditReceipts.push(receipt);
  writeFileSync(join(out, 'source-audit-receipts.json'), JSON.stringify(auditReceipts, null, 2));
  console.log(JSON.stringify({ stage: 'source-audit-complete', call: auditReceipts.length, model: receipt.returned_model, elapsedMs: receipt.elapsed_ms }));
} });
const key = await loadTypesafeKey();
if (!key) throw new Error('Jev key required before reference generation');
const jevGate = createJevPassGate(createJevClient(key), 'active');
const sections: SectionExecutionReceipt[] = [];
const orch = new IntegratedReadingOrchestrator({ mode, llm, jevGate,
  referenceExecution: {
    enabled: true, aletheiosPersona, pichetPersona, serialVoices: process.argv.includes('--serial-voices'),
    sourceAuditLlm: (system, user) => sourceAuditLlm(system, user, { max_tokens: 16384 }),
    requireSourceAudit: true,
    correctedMatrix: {
      register: 'L4-L5',
      language,
      subjectLabel: name,
      sectionTopic: onlySection
        ? mode.frontmatter.pass_plan.find(pass => pass.id === onlySection)?.title
        : undefined,
      interpretationLlm: (system, user, options) => llm(system, user, options),
      maxTokensPerInterpretation: 8192,
    },
    retriever: async (sectionId: string) => {
      // Query only de-identified framework IDs. Current subject facts stay out of CF queries.
      const received = await corpus.getVectors(ids);
      await fs.writeFile(join(out, `${sectionId}.corpus.json`), JSON.stringify(received, null, 2));
      const cfPassages = received.passages.map(p => ({ id: p.id, source: `cf-vectorize:witness-wisdom-corpus/${p.id}`, text: p.text }));
      // Append separately-validated primary passages (if any). CF stays
      // the source of truth for its own passages; primary additions are
      // provenance-labeled so voices see them as attributed passages.
      const combined = buildCombinedPassages({ cfPassages, primaryAdditions });
      const state = received.state === 'success' ? 'success' : received.state === 'empty' ? 'empty' : 'failure';
      const receipt: PassageRetrievalReceipt = {
        section_id: sectionId,
        state: state as 'success' | 'empty' | 'failure',
        count: combined.passages.length,
        source_ids: [...combined.passages.map(p => p.id)],
        passages_hash: combined.passagesHash,
        reason: received.reason,
        ...(combined.primarySourceIds.length > 0 ? {
          primary_source_ids: [...combined.primarySourceIds],
          cf_source_ids: [...combined.cfSourceIds],
        } : {}),
      };
      return { passages: [...combined.passages], receipt };
    },
  },
  reusedSections,
  sectionCheckpoint: async receipt => {
    sections.push(receipt);
    await fs.writeFile(join(out, `${receipt.section_id}.receipt.json`), JSON.stringify(receipt, null, 2));
    if (receipt.source_audit) {
      await fs.writeFile(join(out, `${receipt.section_id}.source-audit.json`), JSON.stringify({ current: receipt.source_audit, revisions: receipt.source_audit_revisions ?? [] }, null, 2));
    }
    await fs.writeFile(join(out, 'llm-receipts.json'), JSON.stringify(llmReceipts, null, 2));
    console.log(JSON.stringify({ section: receipt.section_id, outcome: receipt.outcome, sources: receipt.source_ids.length, reused: reusedSections?.has(receipt.section_id) ?? false }));
  },
});
try {
  const result = await orch.run({ subjectNames: [name], subjectRoles: [{ role: 'primary', name }], engineResultsBySubject: [engines], consciousnessLevel: 4, language });
  const publicReading = result.passes.map(pass => pass.output.trim()).join('\n\n') + '\n';
  await fs.writeFile(join(out, 'reading.md'), publicReading);
  await fs.writeFile(join(out, 'result.json'), JSON.stringify(result, null, 2));
  for (const pass of result.passes) {
    if (!pass.envelope) continue;
    await fs.writeFile(join(out, `${pass.id}.private-evidence-map.json`), JSON.stringify(pass.envelope.private, null, 2));
    await fs.writeFile(join(out, `${pass.id}.section-envelope.json`), JSON.stringify(pass.envelope, null, 2));
  }
  const verification = verifyReferenceExecution({ receipts: sections, passes: result.passes,
    requiredSectionIds: mode.frontmatter.pass_plan.map(p => p.id),
    expectedPersonas: { aletheios: aletheiosPersona.identityText, pichet: pichetPersona.identityText },
    subsectionMap: Object.fromEntries(mode.frontmatter.pass_plan.map(p => [p.id, [...(mode.sections[p.template] ?? '').matchAll(/^\s*-\s+(\d+\.\d+)\s/gm)].map(m => m[1])])),
    rawSynthesisInputs: Object.fromEntries(sections.map(s => [s.section_id, s.synthesis_input_text])),
    sourceTexts: {
      ...Object.fromEntries(preflight.passages.map(p => [p.id, p.text])),
      ...Object.fromEntries(primaryAdditions.map(a => [a.id, a.text])),
    },
    assembled: publicReading,
    requireSourceAudit: true,
    sectionEngineFacts: Object.fromEntries(sections.filter(s => typeof s.engine_facts === 'string').map(s => [s.section_id, s.engine_facts!])),
  });
  await fs.writeFile(join(out, 'verification.json'), JSON.stringify(verification, null, 2));
  // Final route-level verification: requires the hash-bound clean source-audit
  // per section INDEPENDENTLY of Jev. Written alongside verification.json.
  const finalReport = runFinalVerification({
    passes: result.passes,
    referenceRoute: {
      requiredSectionIds: mode.frontmatter.pass_plan.map(p => p.id),
      dyadReceipts: sections.map(receipt => ({
        section_id: receipt.section_id,
        engine_routing: 'dyad-synthesis' as const,
        aletheios_words: receipt.raw.aletheios.trim().split(/\s+/).filter(Boolean).length,
        pichet_words: receipt.raw.pichet.trim().split(/\s+/).filter(Boolean).length,
        synthesis_words: receipt.raw.synthesis.trim().split(/\s+/).filter(Boolean).length,
        aletheios_ok: Boolean(receipt.raw.aletheios.trim()) && receipt.aletheios_hash === hash(receipt.raw.aletheios),
        pichet_ok: Boolean(receipt.raw.pichet.trim()) && receipt.pichet_hash === hash(receipt.raw.pichet),
        synthesis_ok: Boolean(receipt.raw.synthesis.trim()) && receipt.output_hash === hash(receipt.raw.synthesis),
        dyad_available: Boolean(receipt.raw.aletheios.trim() && receipt.raw.pichet.trim() && receipt.raw.synthesis.trim()),
      })),
      retrievalReceipts: sections.map(receipt => ({
        state: receipt.retrieval.state,
        count: receipt.retrieval.count,
        reason: receipt.retrieval.reason,
      })),
      requireSourceAudit: true,
      sourceAuditReceipts: Object.fromEntries(sections.filter(s => s.source_audit).map(s => [s.section_id, s.source_audit!])),
      sectionEngineFacts: Object.fromEntries(sections.filter(s => typeof s.engine_facts === 'string').map(s => [s.section_id, s.engine_facts!])),
      sectionPassagesHash: Object.fromEntries(sections.map(s => [s.section_id, s.passages_hash])),
      sectionLeakageGates: Object.fromEntries(sections.filter(s => s.leakage_gate).map(s => [s.section_id, s.leakage_gate!])),
      ...(primaryAdditions.length > 0 ? { primaryPassageIds: primaryAdditions.map(a => a.id) } : {}),
    },
  });
  await fs.writeFile(join(out, 'final-verification.json'), JSON.stringify(finalReport, null, 2));
  if (primaryRegistrySha256) {
    await fs.writeFile(join(out, 'primary-passage-registry.provenance.json'), JSON.stringify({
      path: primaryRegistryPath,
      sha256: primaryRegistrySha256,
      additionIds: primaryAdditions.map(a => a.id),
      snapshot: 'primary-passage-registry.snapshot.json',
      note: 'Independent bytes for final-verification primaryPassageRegistry.',
    }, null, 2));
  }
  const judgeIssues = result.passes.filter(p => p.jev?.status !== 'judged' || Object.values(p.jev?.verdicts ?? {}).some(v => v !== 'pass'));
  // Generation checks do not prove editorial, rendered-document or bilingual
  // acceptance. Those reviews must never be silently inferred from model gates.
  const status = {
    scope: onlySection ? 'section-preview' : 'full-pilot',
    generated: result.passes.length,
    required: mode.frontmatter.pass_plan.length,
    generationAccepted: verification.passed,
    blockers: verification.blockers,
    jevIssues: judgeIssues.map(p => p.id),
    sourceAuditAccepted: sections.length > 0 && sections.every(section => section.source_audit?.status === 'clean')
      && finalReport.blockers.every(blocker => !blocker.includes('source_audit')),
    referenceRouteAccepted: finalReport.passed,
    sourceAuditBlockers: finalReport.blockers.filter(b => b.includes('source_audit')),
    review: 'pending source and rendered-artifact review',
    finalAccepted: false,
    ...(resumeProvenance ? { resumeProvenance } : {}),
    ...(primaryRegistrySha256 ? { primaryRegistry: { path: primaryRegistryPath, sha256: primaryRegistrySha256, additionIds: primaryAdditions.map(a => a.id) } } : {}),
    ...(editorialPrefixProvenance ? { editorialPrefixProvenance, generationOnly: true, pendingEditorialAcceptance: true, editorialPrefixSections: editorialPrefixSections?.map(s => s.section_id) ?? [], editorialPrefixPending: editorialPrefixPending?.map(p => ({ section_id: p.section_id, ambiguousVerdicts: p.ambiguousVerdicts })) ?? [] } : {}),
  };
  await fs.writeFile(join(out, 'status.json'), JSON.stringify(status, null, 2));
  console.log(JSON.stringify({ ...status, output: out }));
} catch (error) {
  await fs.writeFile(join(out, 'status.json'), JSON.stringify({
    status: 'failed',
    error: error instanceof Error ? error.message : String(error),
    completedSections: sections.filter(s => s.outcome !== 'failed').map(s => s.section_id),
    finalAccepted: false,
    ...(resumeProvenance ? { resumeProvenance } : {}),
    ...(primaryRegistrySha256 ? { primaryRegistry: { path: primaryRegistryPath, sha256: primaryRegistrySha256, additionIds: primaryAdditions.map(a => a.id) } } : {}),
    ...(editorialPrefixProvenance ? { editorialPrefixProvenance, generationOnly: true, pendingEditorialAcceptance: true } : {}),
  }, null, 2));
  throw error;
}
