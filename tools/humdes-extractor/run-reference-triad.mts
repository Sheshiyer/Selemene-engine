/**
 * Reference-format BUSINESS TRIAD runner.
 *
 * Independent adapter for the three-partner reference route. NEVER imports
 * from `run-reference-report.mts` (the individual runner) and NEVER mutates
 * live hash-bound files. Preflight-first, per-subject snapshots, no engine
 * collapsing across subjects.
 *
 * Scope:
 *   - Loads three saved engine bundles (Gary, Mohan, Shesh) from
 *     `<packageDir>/reports-reference/engines/<slug>.prod.json` in canonical
 *     order via loadTriadEngines().
 *   - Unions de-identified corpus IDs across the triad via
 *     unionCorpusIdsForTriad(). No subject-derived tokens ever leave.
 *   - Fresh CF getVectors read-only for grounding preflight; live model
 *     runs still require operator opt-out of --preflight.
 *   - Parses packages/witness-pipeline/modes/business-triad-reference.md
 *     (12 canonical sections) and localizes via localizeBusinessTriadTitles.
 *   - Dual persona load from ../witness-agents/agents/{aletheios,pichet}.
 *   - Wires IntegratedReadingOrchestrator with referenceExecution.enabled
 *     so the reference execution path activates.
 *   - Per-subject snapshots, per-section receipts, runtime hashes, and a
 *     `status.json` that always records `finalAccepted: false` at
 *     generation time. Verifier acceptance is a separate downstream step.
 */

import { promises as fs, writeFileSync } from 'node:fs';
import { resolve, join, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import { createHash } from 'node:crypto';
import { parseModeDoc } from '../../packages/witness-pipeline/src/modes/parser.js';
import { IntegratedReadingOrchestrator } from '../../packages/witness-pipeline/src/orchestrator/integrated.js';
import { VectorizeCorpusAdapter } from '../../packages/witness-pipeline/src/grounding/index.js';
import { createOmniRouteLlmCall } from '../../packages/witness-pipeline/scripts/lib/multi-llm.js';
import { createJevClient, loadTypesafeKey } from '../../packages/witness-pipeline/src/jev/client.js';
import { createJevPassGate } from '../../packages/witness-pipeline/src/jev/pass-gate.js';
import { createPartitionedJevClient, type PartitionEvidence } from '../../packages/witness-pipeline/src/jev/partitioned-client.js';
import type {
  WitnessPersona,
  SectionExecutionReceipt,
  PassageRetrievalReceipt,
} from '../../packages/witness-pipeline/src/orchestrator/reference-execution.js';
import { verifyReferenceExecution } from '../../packages/witness-pipeline/src/orchestrator/reference-verification.js';
import { runFinalVerification } from '../../packages/witness-pipeline/src/orchestrator/final-verification.js';
import { validatePrimaryPassageAdditions } from '../../packages/witness-pipeline/src/orchestrator/editorial-passage-evidence.js';
import { buildCombinedPassages } from '../../packages/witness-pipeline/src/orchestrator/primary-passage-source.js';
import type { EditorialPassageAddition } from '../../packages/witness-pipeline/src/orchestrator/editorial-passage-evidence.js';
import {
  BUSINESS_TRIAD_SLUGS,
  loadTriadEngines,
  unionCorpusIdsForTriad,
  localizeBusinessTriadTitles,
  type TriadEngineBundle,
} from '../../packages/witness-pipeline/src/orchestrator/business-triad-support.js';
import type { SelemeneEngineOutput } from '../../packages/witness-pipeline/src/selemene/types.js';

// ─── CLI parsing ────────────────────────────────────────────────────────
const args = process.argv.slice(2);
const val = (key: string, fallback?: string) => {
  const n = args.indexOf(key);
  return n < 0 ? fallback : args[n + 1];
};
const repo = resolve(dirname(fileURLToPath(import.meta.url)), '../..');

const packageArg = val('--package-dir');
if (!packageArg) throw new Error('Required: --package-dir <source package>');
const root = resolve(packageArg);

const language = val('--language', 'en')!;
if (!['en', 'fr'].includes(language)) throw new Error('Unsupported language');

const accountId = val('--account-id', process.env.CLOUDFLARE_ACCOUNT_ID);
if (!accountId || !/^[a-f0-9]{32}$/.test(accountId)) {
  throw new Error('Pass the verified --account-id or CLOUDFLARE_ACCOUNT_ID');
}

const primaryRegistryArg = val('--primary-registry');
const onlySection = val('--section');
const requestedCombo = val('--combo', 'noesis-execute')!;
const requestedAuditCombo = val('--audit-combo', 'noesis-verify')!;

const hash = (text: string) => createHash('sha256').update(text).digest('hex');

// ─── Load three engine bundles (no collapsing across subjects) ──────────
const bundles: TriadEngineBundle[] = await loadTriadEngines(root);
if (bundles.length !== BUSINESS_TRIAD_SLUGS.length) {
  throw new Error(`Expected ${BUSINESS_TRIAD_SLUGS.length} triad bundles, got ${bundles.length}`);
}

// ─── Union de-identified corpus IDs across the triad ────────────────────
const unionResult = unionCorpusIdsForTriad(bundles);
const ids = unionResult.ids;
if (ids.length === 0) {
  throw new Error('Triad corpus union is empty; verify saved engine bundles have HD/GK/Vim/Vedic outputs');
}

// ─── Output dir + per-subject snapshots ─────────────────────────────────
const runId = `business-partners-triad-${language}-${new Date().toISOString().replace(/[:.]/g, '-')}`;
const out = join(root, 'reports-reference/runs', runId);
await fs.mkdir(out, { recursive: true });

for (const bundle of bundles) {
  await fs.writeFile(join(out, `input.${bundle.subject}.snapshot.json`), bundle.sourceBytes);
}

// ─── Fresh CF preflight read (no writes, no mutation) ───────────────────
const corpus = new VectorizeCorpusAdapter({ accountId, indexName: 'witness-wisdom-corpus', timeoutMs: 45000 });
const preflight = await corpus.getVectors(ids);
await fs.writeFile(join(out, 'corpus-preflight.json'), JSON.stringify(preflight, null, 2));
if (preflight.state !== 'success' || !preflight.passages.length) {
  throw new Error(`CF grounding unavailable: ${preflight.state}: ${preflight.reason ?? ''}`);
}

// ─── Personas (dual voices) ─────────────────────────────────────────────
async function loadPersona(name: string): Promise<WitnessPersona> {
  const sourcePath = resolve(repo, '../witness-agents/agents', name, 'IDENTITY.md');
  const identityText = await fs.readFile(sourcePath, 'utf8');
  return { name, sourcePath, identityText, sourceHash: hash(identityText) };
}
const [aletheiosPersona, pichetPersona] = await Promise.all([
  loadPersona('aletheios'),
  loadPersona('pichet'),
]);

// ─── Optional independent primary passage registry ──────────────────────
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
    reservedCFIds: preflight.passages.map((p) => p.id),
  });
  primaryAdditions = validated.additions;
  await fs.writeFile(
    join(out, 'primary-passage-registry.snapshot.json'),
    primaryRegistryBytes,
    { flag: 'wx' },
  );
  console.log(JSON.stringify({
    stage: 'primary-registry-loaded',
    additions: primaryAdditions.length,
    sha256: primaryRegistrySha256,
    path: primaryRegistryPath,
  }));
}

// ─── Runtime hashes (bind new files that steer this run) ────────────────
const runtimeFiles = [
  // Reference-route runtime bound by hash — unchanged by this build.
  'packages/witness-pipeline/src/orchestrator/reference-language.ts',
  'packages/witness-pipeline/src/orchestrator/reference-source-audit.ts',
  'packages/witness-pipeline/src/orchestrator/reference-verification.ts',
  'packages/witness-pipeline/src/orchestrator/final-verification.ts',
  'packages/witness-pipeline/src/orchestrator/resume-validation.ts',
  'packages/witness-pipeline/src/orchestrator/rubric.ts',
  'packages/witness-pipeline/src/orchestrator/integrated.ts',
  'packages/witness-pipeline/src/orchestrator/reference-execution.ts',
  'packages/witness-pipeline/src/orchestrator/engine-facts.ts',
  'packages/witness-pipeline/src/grounding/vectorize-corpus-adapter.ts',
  'packages/witness-pipeline/src/jev/pass-gate.ts',
  'packages/witness-pipeline/src/jev/partitioned-client.ts',
  'packages/witness-pipeline/scripts/lib/multi-llm.ts',
  // NEW artifacts that own this triad route.
  'packages/witness-pipeline/modes/business-triad-reference.md',
  'packages/witness-pipeline/src/orchestrator/business-triad-support.ts',
  'tools/humdes-extractor/run-reference-triad.mts',
];
const runtimeHashes = Object.fromEntries(
  await Promise.all(
    runtimeFiles.map(async (path) => [path, hash(await fs.readFile(join(repo, path), 'utf8'))]),
  ),
);

await fs.writeFile(
  join(out, 'inputs.json'),
  JSON.stringify(
    {
      route: 'business-triad-reference',
      subjects: bundles.map((b) => ({
        slug: b.subject,
        name: b.name,
        role: b.role,
        inputPath: b.sourcePath,
        inputHash: hash(b.sourceBytes),
        engineCount: b.engines.length,
      })),
      language,
      accountId,
      index: 'witness-wisdom-corpus',
      personaHashes: [aletheiosPersona.sourceHash, pichetPersona.sourceHash],
      corpusIdCount: ids.length,
      corpusPerSubjectCounts: unionResult.perSubjectCounts,
      runtimeHashes,
      serialVoices: process.argv.includes('--serial-voices'),
      requestedCombo,
      requestedAuditCombo,
      ...(primaryRegistrySha256
        ? {
            primaryRegistry: {
              path: primaryRegistryPath,
              sha256: primaryRegistrySha256,
              additionIds: primaryAdditions.map((a) => a.id),
            },
          }
        : {}),
    },
    null,
    2,
  ),
);

console.log(JSON.stringify({
  stage: 'preflight',
  passages: preflight.passages.length,
  requested: ids.length,
  subjects: bundles.map((b) => b.subject),
  output: out,
}));

// Preflight-first: bail before any LLM calls when requested.
if (args.includes('--preflight')) process.exit(0);

// ─── Parse mode + localize ──────────────────────────────────────────────
const modePath = join(repo, 'packages/witness-pipeline/modes/business-triad-reference.md');
const mode = parseModeDoc(modePath);
localizeBusinessTriadTitles(mode, language);

if (onlySection) {
  if (!mode.frontmatter.pass_plan.some((p) => p.id === onlySection)) {
    throw new Error('Unknown section: ' + onlySection);
  }
  mode.frontmatter.pass_plan = mode.frontmatter.pass_plan.filter((p) => p.id === onlySection);
}

// ─── LLM wiring (generation + independent audit) ────────────────────────
const llmReceipts: unknown[] = [];
const llm = createOmniRouteLlmCall(requestedCombo, {
  temperature: 0.4,
  timeout_ms: 240000,
  onReceipt: (receipt) => {
    llmReceipts.push(receipt);
    writeFileSync(join(out, 'llm-receipts.json'), JSON.stringify(llmReceipts, null, 2));
    console.log(JSON.stringify({
      stage: 'llm-complete',
      call: llmReceipts.length,
      model: receipt.returned_model,
      elapsedMs: receipt.elapsed_ms,
    }));
  },
});

const auditReceipts: unknown[] = [];
const sourceAuditLlm = createOmniRouteLlmCall(requestedAuditCombo, {
  temperature: 0,
  timeout_ms: 180000,
  onReceipt: (receipt) => {
    auditReceipts.push(receipt);
    writeFileSync(join(out, 'source-audit-receipts.json'), JSON.stringify(auditReceipts, null, 2));
    console.log(JSON.stringify({
      stage: 'source-audit-complete',
      call: auditReceipts.length,
      model: receipt.returned_model,
      elapsedMs: receipt.elapsed_ms,
    }));
  },
});

const key = await loadTypesafeKey();
if (!key) throw new Error('Jev key required before reference generation');
const partitionReceipts: PartitionEvidence[] = [];
const jevGate = createJevPassGate(createPartitionedJevClient(createJevClient(key)!, {
  onEvidence: (evidence) => {
    partitionReceipts.push(evidence);
    writeFileSync(join(out, 'jev-partition-receipts.json'), JSON.stringify(partitionReceipts, null, 2));
  },
}), 'shadow');

// ─── Orchestrator (reference execution enabled) ─────────────────────────
const sections: SectionExecutionReceipt[] = [];
const orch = new IntegratedReadingOrchestrator({
  mode,
  llm,
  jevGate,
  referenceExecution: {
    enabled: true,
    correctedMatrix: { register: 'L4-L5', language, subjectLabel: subjectNames.join(' × ') },
    aletheiosPersona,
    pichetPersona,
    serialVoices: process.argv.includes('--serial-voices'),
    sourceAuditLlm: (system, user) => sourceAuditLlm(system, user, { max_tokens: 16384 }),
    requireSourceAudit: true,
    retriever: async (sectionId: string) => {
      // Query only de-identified framework IDs. Current partners' facts stay
      // out of CF queries; only framework-scoped `sw:*` IDs may leave.
      const received = await corpus.getVectors(ids);
      await fs.writeFile(join(out, `${sectionId}.corpus.json`), JSON.stringify(received, null, 2));
      const cfPassages = received.passages.map((p) => ({
        id: p.id,
        source: `cf-vectorize:witness-wisdom-corpus/${p.id}`,
        text: p.text,
      }));
      const combined = buildCombinedPassages({ cfPassages, primaryAdditions });
      const state =
        received.state === 'success' ? 'success' : received.state === 'empty' ? 'empty' : 'failure';
      const receipt: PassageRetrievalReceipt = {
        section_id: sectionId,
        state: state as 'success' | 'empty' | 'failure',
        count: combined.passages.length,
        source_ids: [...combined.passages.map((p) => p.id)],
        passages_hash: combined.passagesHash,
        reason: received.reason,
        ...(combined.primarySourceIds.length > 0
          ? {
              primary_source_ids: [...combined.primarySourceIds],
              cf_source_ids: [...combined.cfSourceIds],
            }
          : {}),
      };
      return { passages: [...combined.passages], receipt };
    },
  },
  sectionCheckpoint: async (receipt) => {
    sections.push(receipt);
    await fs.writeFile(
      join(out, `${receipt.section_id}.receipt.json`),
      JSON.stringify(receipt, null, 2),
    );
    if (receipt.source_audit) {
      await fs.writeFile(
        join(out, `${receipt.section_id}.source-audit.json`),
        JSON.stringify(
          { current: receipt.source_audit, revisions: receipt.source_audit_revisions ?? [] },
          null,
          2,
        ),
      );
    }
    await fs.writeFile(join(out, 'llm-receipts.json'), JSON.stringify(llmReceipts, null, 2));
    console.log(JSON.stringify({
      section: receipt.section_id,
      outcome: receipt.outcome,
      sources: receipt.source_ids.length,
    }));
  },
});

// ─── Assemble per-subject arrays; NEVER flatten across subjects ─────────
const subjectNames = bundles.map((b) => b.name);
const subjectRoles = bundles.map((b) => ({ role: b.role as string, name: b.name }));
// Cast each bundle's engines to SelemeneEngineOutput[]; the individual runner
// does the same because saved bundles carry the same partial shape. We never
// merge these arrays — one entry per partner, in canonical order.
const engineResultsBySubject = bundles.map(
  (b) => b.engines as unknown as SelemeneEngineOutput[],
);

try {
  const result = await orch.run({
    subjectNames,
    subjectRoles,
    engineResultsBySubject,
    relationshipContext: {
      type: 'business-partners',
      mapping_goal: 'Describe the three partners’ symbolic patterns, preserving each person’s agency and avoiding business outcome predictions.',
      sensitivity_level: 'medium',
    },
    consciousnessLevel: 4,
    language,
  });
  await fs.writeFile(join(out, 'reading.md'), result.assembled + '\n');
  await fs.writeFile(join(out, 'result.json'), JSON.stringify(result, null, 2));
  for (const pass of result.passes) {
    if (!pass.envelope) continue;
    await fs.writeFile(join(out, pass.id + '.envelope.json'), JSON.stringify(pass.envelope, null, 2));
    await fs.writeFile(join(out, pass.id + '.private-evidence-map.json'), JSON.stringify(pass.envelope.private, null, 2));
  }

  const verification = verifyReferenceExecution({
    receipts: sections,
    passes: result.passes,
    requiredSectionIds: mode.frontmatter.pass_plan.map((p) => p.id),
    expectedPersonas: {
      aletheios: aletheiosPersona.identityText,
      pichet: pichetPersona.identityText,
    },
    subsectionMap: Object.fromEntries(
      mode.frontmatter.pass_plan.map((p) => [
        p.id,
        [...(mode.sections[p.template] ?? '').matchAll(/^\s*-\s+(\d+\.\d+)\s/gm)].map((m) => m[1]),
      ]),
    ),
    rawSynthesisInputs: Object.fromEntries(
      sections.map((s) => [s.section_id, s.synthesis_input_text]),
    ),
    sourceTexts: {
      ...Object.fromEntries(preflight.passages.map((p) => [p.id, p.text])),
      ...Object.fromEntries(primaryAdditions.map((a) => [a.id, a.text])),
    },
    assembled: result.assembled + '\n',
    requireSourceAudit: true,
    sectionEngineFacts: Object.fromEntries(
      sections.filter((s) => typeof s.engine_facts === 'string').map((s) => [s.section_id, s.engine_facts!]),
    ),
  });
  await fs.writeFile(join(out, 'verification.json'), JSON.stringify(verification, null, 2));

  const finalReport = runFinalVerification({
    passes: result.passes,
    referenceRoute: {
      requiredSectionIds: mode.frontmatter.pass_plan.map((p) => p.id),
      requireSourceAudit: true,
      sourceAuditReceipts: Object.fromEntries(
        sections.filter((s) => s.source_audit).map((s) => [s.section_id, s.source_audit!]),
      ),
      sectionEngineFacts: Object.fromEntries(
        sections.filter((s) => typeof s.engine_facts === 'string').map((s) => [s.section_id, s.engine_facts!]),
      ),
      sectionPassagesHash: Object.fromEntries(sections.map((s) => [s.section_id, s.passages_hash])),
      ...(primaryAdditions.length > 0
        ? { primaryPassageIds: primaryAdditions.map((a) => a.id) }
        : {}),
    },
  });
  await fs.writeFile(join(out, 'final-verification.json'), JSON.stringify(finalReport, null, 2));

  if (primaryRegistrySha256) {
    await fs.writeFile(
      join(out, 'primary-passage-registry.provenance.json'),
      JSON.stringify(
        {
          path: primaryRegistryPath,
          sha256: primaryRegistrySha256,
          additionIds: primaryAdditions.map((a) => a.id),
          snapshot: 'primary-passage-registry.snapshot.json',
          note: 'Independent bytes for final-verification primaryPassageRegistry.',
        },
        null,
        2,
      ),
    );
  }

  const judgeIssues = result.passes.filter(
    (p) =>
      p.jev?.status !== 'judged' ||
      Object.values(p.jev?.verdicts ?? {}).some((v) => v !== 'pass'),
  );

  // Generation-time checks do NOT prove editorial, rendered-document, or
  // bilingual acceptance. finalAccepted stays false at generation time; a
  // separate verifier/render/parity/coherence review lifts it.
  const status = {
    route: 'business-triad-reference',
    scope: onlySection ? 'section-preview' : 'full-triad',
    subjects: bundles.map((b) => ({ slug: b.subject, name: b.name })),
    generated: result.passes.length,
    required: mode.frontmatter.pass_plan.length,
    generationAccepted: verification.passed,
    blockers: verification.blockers,
    jevIssues: judgeIssues.map((p) => p.id),
    sourceAuditAccepted: finalReport.passed,
    sourceAuditBlockers: finalReport.blockers.filter((b) => b.includes('source_audit')),
    review: 'pending source and rendered-artifact review',
    finalAccepted: false,
    ...(primaryRegistrySha256
      ? {
          primaryRegistry: {
            path: primaryRegistryPath,
            sha256: primaryRegistrySha256,
            additionIds: primaryAdditions.map((a) => a.id),
          },
        }
      : {}),
  };
  await fs.writeFile(join(out, 'status.json'), JSON.stringify(status, null, 2));
  console.log(JSON.stringify({ ...status, output: out }));
} catch (error) {
  await fs.writeFile(
    join(out, 'status.json'),
    JSON.stringify(
      {
        route: 'business-triad-reference',
        status: 'failed',
        error: error instanceof Error ? error.message : String(error),
        completedSections: sections.filter((s) => s.outcome !== 'failed').map((s) => s.section_id),
        finalAccepted: false,
        ...(primaryRegistrySha256
          ? {
              primaryRegistry: {
                path: primaryRegistryPath,
                sha256: primaryRegistrySha256,
                additionIds: primaryAdditions.map((a) => a.id),
              },
            }
          : {}),
      },
      null,
      2,
    ),
  );
  throw error;
}
