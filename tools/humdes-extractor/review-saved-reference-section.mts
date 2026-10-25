import {loadMixedReviewPassages} from './load-mixed-review-passages.mjs';
import { readFileSync, writeFileSync, existsSync } from 'node:fs';
import { join, dirname, resolve } from 'node:path';
import { createHash } from 'node:crypto';
import { runSourceAudit } from '../../packages/witness-pipeline/src/orchestrator/reference-source-audit.js';
import { createOmniRouteLlmCall } from '../../packages/witness-pipeline/scripts/lib/multi-llm.js';
import { VectorizeCorpusAdapter } from '../../packages/witness-pipeline/src/grounding/index.js';
import {
  validateEditorialPassageEvidence,
  type EditorialPassageAddition,
  type TrustedSourceRecord,
} from '../../packages/witness-pipeline/src/orchestrator/editorial-passage-evidence.js';

// ─── Args ──────────────────────────────────────────────────────────
const argv = process.argv.slice(2);
const positional: string[] = [];
const flags: Record<string, string> = {};
for (let i = 0; i < argv.length; i += 1) {
  const a = argv[i];
  if (a.startsWith('--')) { flags[a.replace(/^--/, '')] = argv[i + 1] ?? ''; i += 1; }
  else positional.push(a);
}
const [dir, section, revisionVersion] = positional;
if (!dir || !/^(opening|part(?:[1-9]|10|11))$/.test(section ?? '')) throw new Error('run directory and valid section required');
if (revisionVersion && !/^(editorial|v[1-9][0-9]*)$/.test(revisionVersion)) throw new Error('invalid revision version');
const passageRegistryPath = flags['passage-registry'] || undefined;

// ─── Fixed inputs from run dir ─────────────────────────────────────
const base = JSON.parse(readFileSync(join(dir, section + '.receipt.json'), 'utf8'));
const revision = revisionVersion ? JSON.parse(readFileSync(join(dir, section + '.editorial-revision' + (revisionVersion === 'editorial' ? '' : '-' + revisionVersion) + '.json'), 'utf8')) : undefined;
const hash = (text: string) => createHash('sha256').update(text).digest('hex');
if (revision && (revision.baseOutputHash !== base.output_hash || revision.revisedOutputHash !== hash(revision.revisedOutput))) throw new Error('revision hash mismatch');
const output = revision?.revisedOutput ?? base.raw.synthesis;

const implementationPath = 'crates/engine-transits/src/ephemeris.rs';
const implementation = revision ? readFileSync(implementationPath, 'utf8') : '';
const supplementalEvidence = revision ? { path: implementationPath, sha256: hash(implementation), text: implementation, scope: 'Current local implementation; not independent proof of the saved production binary version' } : undefined;
const engineFactsBase = base.engine_facts + (supplementalEvidence ? '\nAdditional implementation evidence, with limited scope:\n' + JSON.stringify(supplementalEvidence) : '');

const corpus = JSON.parse(readFileSync(join(dir, section + '.corpus.json'), 'utf8'));
if (base.primary_source_ids?.length && passageRegistryPath) throw new Error('Mixed generation already includes its primary sources; additional editorial supplements require separate support');
const basePassages = base.primary_source_ids?.length ? await loadMixedReviewPassages(dir,base) : corpus.passages.map((p: any) => ({ id: p.id, text: p.text, source: 'cf-vectorize:witness-wisdom-corpus/' + p.id }));

// ─── Optional passage-registry supplement ──────────────────────────
// Reviewer supplies an INDEPENDENT approved reviewed-source registry
// from a separate file — never trust the audit sidecar to whitelist
// itself.  The registry shape is `{ records: {id: TrustedSourceRecord},
// additionIds: string[] }`.  When provided, we:
//   * Derive expectedCF from savedinputs (inputs.json accountId/index)
//     — never from the registry file itself.
//   * Take base texts from corpus.json (the run's fresh CF readback);
//     supply an independent trustedOriginalTexts mirror so
//     validateEditorialPassageEvidence enforces per-id byte equality.
//   * Feed the effective (base + additions) view to runSourceAudit and
//     record a passageEvidence sidecar next to the audit artifact so
//     the parent verifier can rebind the audit input hash.
let augmentedPassages = basePassages;
let augmentedPassagesHash: string = base.passages_hash;
let passageEvidenceSidecar: unknown = undefined;

if (passageRegistryPath) {
  const registryAbs = resolve(passageRegistryPath);
  if (!existsSync(registryAbs)) throw new Error(`passage-registry not found: ${registryAbs}`);
  const registry = JSON.parse(readFileSync(registryAbs, 'utf8')) as {
    records: Record<string, TrustedSourceRecord>;
    additionIds: string[];
  };
  if (!registry || typeof registry !== 'object' || typeof registry.records !== 'object' || !Array.isArray(registry.additionIds)) {
    throw new Error('passage-registry must be `{records: {id: TrustedSourceRecord}, additionIds: string[]}`');
  }

  // Derive expectedCF from savedinputs — never from the registry itself.
  const savedInputsPath = join(dir, 'inputs.json');
  if (!existsSync(savedInputsPath)) throw new Error('inputs.json missing from run dir; cannot derive expectedCF');
  const savedInputs = JSON.parse(readFileSync(savedInputsPath, 'utf8')) as { accountId?: string; index?: string };
  if (!savedInputs.accountId || !savedInputs.index) throw new Error('inputs.json missing accountId/index');
  const expectedCF = { account: savedInputs.accountId, index: savedInputs.index };

  // Build the additions list from the registry's approved add-list; the
  // registry itself is the caller-trusted independent source — the CLI
  // never invents records.
  const additions: EditorialPassageAddition[] = registry.additionIds.map((id) => {
    const rec = registry.records[id];
    if (!rec) throw new Error(`passage-registry additionIds refers to unknown id "${id}"`);
    return rec;
  });

  // Re-read CF now: a saved snapshot alone cannot establish fresh grounding.
  const cfAdditions = additions.filter(a => a.provenance.kind === 'cf-vectorize');
  const fresh = await new VectorizeCorpusAdapter({
    accountId: expectedCF.account, indexName: expectedCF.index, timeoutMs: 45000,
  }).getVectors([...new Set([...base.source_ids, ...cfAdditions.map(a => a.id)])]);
  if (fresh.state !== 'success') throw new Error(`fresh CF readback failed: ${fresh.state}`);
  for (let i = 0; i < base.source_ids.length; i += 1) {
    const saved = basePassages[i];
    const live = fresh.passages.find(p => p.id === base.source_ids[i]);
    if (!saved || saved.id !== base.source_ids[i] || !live ||
        hash(live.text) !== base.source_hashes[i] || saved.text !== live.text) {
      throw new Error(`base CF source mismatch: ${base.source_ids[i]}`);
    }
  }
  for (const added of cfAdditions) {
    const live = fresh.passages.find(p => p.id === added.id);
    if (!live || live.text !== added.text || hash(live.text) !== added.sha256) {
      throw new Error(`added CF source mismatch: ${added.id}`);
    }
  }

  // Independent trusted originals from the run's actual CF readback.
  // corpus.json IS the fresh CF readback the base retrieval performed;
  // the registry never re-declares base texts.
  const trustedOriginalTexts: Record<string, string> = {};
  for (const bp of basePassages as { id: string; text: string }[]) trustedOriginalTexts[bp.id] = bp.text;

  const validated = validateEditorialPassageEvidence({
    expectedCF,
    basePassages,
    basePassagesHash: base.passages_hash,
    additions,
    trustedSourceRecords: registry.records,
    trustedOriginalTexts,
  });

  augmentedPassages = validated.effectivePassages.map((p) => ({ id: p.id, text: p.text, source: p.source }));
  augmentedPassagesHash = validated.effectivePassagesHash;
  passageEvidenceSidecar = {
    basePassages,
    basePassagesHash: base.passages_hash,
    additions,
    addedSourceRecords: validated.addedSourceRecords,
    effectivePassagesHash: validated.effectivePassagesHash,
    passageRegistryPath: registryAbs,
    expectedCF,
    cfReadback: fresh,
    cfReadAt: new Date().toISOString(),
  };
}

const engineFacts = engineFactsBase;
const receipts: unknown[] = [];
const llm = createOmniRouteLlmCall('noesis-verify', { temperature: 0, timeout_ms: 180000, onReceipt: (r: unknown) => receipts.push(r) });
const instruction = 'Audit the exact supplied section afresh. Every claim_quoted must be an exact contiguous quotation of the section, never a paraphrase, combination of separated cells, or quotation from the engine facts. Preserve capitalization, accents, punctuation and Markdown characters exactly; do not lowercase the first word of a quoted sentence. If a statement is unsupported or contradictory, return changes-required with findings. Do not aim for clean. Preserve all findings. Return one strictly valid JSON object. Use a plain JSON string for supporting_values, not nested objects. Escape every embedded quotation mark, including degree-minute-second notation, and check commas between fields. Do not insert annotations outside JSON strings.';
const audit = await runSourceAudit(
  { sectionId: section, sectionTitle: section, acceptedOutput: output, engineFacts, passages: augmentedPassages, passagesHash: augmentedPassagesHash },
  (s: string, u: string, o: { max_tokens: number }) => llm(s + '\n' + instruction, u, o),
  { maxTokens: 16384 },
);
const artifact = {
  kind: revision ? 'fresh-audit-of-editorial-revision' : 'fresh-audit-of-saved-draft',
  baseReceiptSha256: hash(readFileSync(join(dir, section + '.receipt.json'), 'utf8')),
  sectionOutputHash: hash(output),
  originalOutcome: base.outcome,
  revisionVersion,
  supplementalEvidence,
  engineFacts,
  instruction,
  audit,
  passageEvidence: passageEvidenceSidecar,
  llmReceipts: receipts,
  finalAccepted: false,
};
const path = join(dir, section + '.audit-recovery-' + new Date().toISOString().replace(/:/g, '-') + '.json');
writeFileSync(path, JSON.stringify(artifact, null, 2) + '\n', { flag: 'wx' });
console.log(JSON.stringify({ path, status: audit.status, reason: audit.reason, findings: audit.findings, checks: audit.claim_checks?.length, effectivePassagesHash: augmentedPassagesHash }));
