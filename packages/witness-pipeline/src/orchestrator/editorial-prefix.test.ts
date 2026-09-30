import { describe, it, expect, beforeEach, afterAll } from 'vitest';
import { promises as fs } from 'node:fs';
import { createHash } from 'node:crypto';
import { join } from 'node:path';
import { tmpdir } from 'node:os';
import {
  applyEditorialRevision,
  type EditorialPatch,
} from './editorial-revision.js';
import type { SectionExecutionReceipt, SectionAttemptRecord } from './reference-execution.js';
import { computeAuditInputHash } from './reference-source-audit.js';
import { loadEditorialPrefix, type EditorialPrefixIdentity } from './editorial-prefix.js';

// ─── helpers ──────────────────────────────────────────────────────────

const sha256 = (s: string) => createHash('sha256').update(s, 'utf8').digest('hex');

const PASSAGE_A = 'Passage A: framework text on generator type immediate response.';
const PASSAGE_B = 'Passage B: framework text on sacral authority somatic signal.';
const SOURCE_IDS = ['sw:hd:type:generator:desc', 'sw:hd:auth:sacral-authority:desc'];
const SOURCE_TEXTS: Record<string, string> = {
  [SOURCE_IDS[0]]: PASSAGE_A,
  [SOURCE_IDS[1]]: PASSAGE_B,
};
const PASSAGES_HASH = sha256([PASSAGE_A, PASSAGE_B].join('\n'));
const ALETHEIOS_PERSONA_HASH = sha256('aletheios-identity');
const PICHET_PERSONA_HASH = sha256('pichet-identity');
const INPUT_SNAPSHOT = '{"subject":"sheshnarayan","engines":[]}';
const INPUT_HASH = sha256(INPUT_SNAPSHOT);
const ENGINE_FACTS_OPENING = 'ENGINE FACTS OPENING\n- human-design: Type Generator; Authority Sacral';
const ENGINE_FACTS_PART1 = 'ENGINE FACTS PART I\n- vedic-kundali: Lagna Vrishchika';

const OPENING_ORIGINAL = 'Opening draft: eleven engines are listed. Framing differences are contradictions between systems. Method: two voices reconcile in each chapter to produce the reading.';
const OPENING_REVISED_EXPECTED = 'Opening draft: eleven engines are listed. Framing differences are conventions, not contradictions. Method: two voices reconcile in each chapter to produce the reading.';

const PART1_ORIGINAL = 'Part I: The convergence map begins with the ascendant. All planets are exalted in this chart. Vimshottari and vedic-kundali agree on the birth nakshatra.';
const PART1_REVISED_EXPECTED = 'Part I: The convergence map begins with the ascendant. No planets are marked as exalted in the supplied record. Vimshottari and vedic-kundali agree on the birth nakshatra.';

function makeAletheios(sectionId: string): string {
  return `Aletheios raw for ${sectionId}: precise reading of the engine record. This text is long enough to satisfy the receipt shape hash-consistency gate applied by resume validation and editorial prefix loaders. Deterministic corpus content.`;
}
function makePichet(sectionId: string): string {
  return `Pichet raw for ${sectionId}: felt-sense reflection on the passages. This text is long enough to satisfy the receipt shape hash-consistency gate applied by resume validation and editorial prefix loaders. Deterministic voice output.`;
}

function makeReceipt(opts: {
  sectionId: string;
  synthesis: string;
  engineFacts: string;
  outcome?: 'ok' | 'repaired' | 'failed';
  attempts?: SectionAttemptRecord[];
}): SectionExecutionReceipt {
  const aletheios = makeAletheios(opts.sectionId);
  const pichet = makePichet(opts.sectionId);
  const synthesisInput = `SYNTHESIS INPUT FOR ${opts.sectionId}\n${opts.engineFacts}\nA: ${aletheios}\nP: ${pichet}`;
  const receipt: SectionExecutionReceipt = {
    section_id: opts.sectionId,
    outcome: opts.outcome ?? 'ok',
    source_ids: SOURCE_IDS,
    source_hashes: [sha256(PASSAGE_A), sha256(PASSAGE_B)],
    passages_hash: PASSAGES_HASH,
    aletheios_hash: sha256(aletheios),
    pichet_hash: sha256(pichet),
    synthesis_input_hash: sha256(synthesisInput),
    synthesis_input_text: synthesisInput,
    prior_section_content_hashes: {},
    output_hash: sha256(opts.synthesis),
    aletheios_persona: { name: 'aletheios', sourcePath: '/dev/null/a', sourceHash: ALETHEIOS_PERSONA_HASH },
    pichet_persona: { name: 'pichet', sourcePath: '/dev/null/p', sourceHash: PICHET_PERSONA_HASH },
    raw: { aletheios, pichet, synthesis: opts.synthesis },
    retrieval: {
      section_id: opts.sectionId,
      state: 'success',
      count: SOURCE_IDS.length,
      source_ids: SOURCE_IDS,
      passages_hash: PASSAGES_HASH,
    } as SectionExecutionReceipt['retrieval'],
    engine_facts: opts.engineFacts,
    attempts: opts.attempts ?? [{ attempt: 1, outcome: 'ok', aletheios_raw: aletheios, pichet_raw: pichet, synthesis_raw: opts.synthesis }],
  };
  return receipt;
}

const OPENING_PATCHES: EditorialPatch[] = [
  {
    before: 'Framing differences are contradictions between systems.',
    after: 'Framing differences are conventions, not contradictions.',
    rationale: 'Original conflated conventions with contradictions; source review shows sidereal is used throughout.',
    sourceReferences: [{ sourceId: 'crates/engine-transits/src/ephemeris.rs:42', quote: 'to_sidereal_longitude', label: 'Engine implementation inspected during source review' }],
  },
];

const PART1_PATCHES: EditorialPatch[] = [
  {
    before: 'All planets are exalted in this chart.',
    after: 'No planets are marked as exalted in the supplied record.',
    rationale: 'Original overstated dignities; audit of vedic-kundali record shows no exaltation flag set for any planet.',
    sourceReferences: [{ sourceId: 'reports-reference/engines/sheshnarayan.prod.json#vedic-kundali.planets', quote: 'dignity: none', label: 'Engine result JSON' }],
  },
];

function buildRevisionArtifact(receipt: SectionExecutionReceipt, patches: EditorialPatch[]) {
  const revision = applyEditorialRevision({
    baseReceipt: receipt,
    currentOutput: receipt.raw.synthesis,
    expectedBaseOutputHash: receipt.output_hash,
    patches,
    reviewer: 'assistant-source-review',
    timestamp: '2026-09-30T01:13:59.113Z',
  });
  return { revision, patches };
}

function buildCleanAudit(sectionId: string, revisedOutput: string, engineFacts: string, claimQuoted: string, sourceId: string) {
  const inputSha = computeAuditInputHash({ acceptedOutput: revisedOutput, passagesHash: PASSAGES_HASH, engineFacts });
  const modelRaw = JSON.stringify({
    status: 'clean',
    coverage: { total_claims_audited: 1, sources_referenced: 1 },
    findings: [],
    claim_checks: [
      {
        claim_quoted: claimQuoted,
        source_reference: { source_path_or_passage_id: sourceId, supporting_values: 'verbatim substring of the revised output' },
      },
    ],
  });
  const audit = {
    section_id: sectionId,
    status: 'clean',
    model_raw: modelRaw,
    input_sha256: inputSha,
    output_sha256: sha256(modelRaw),
    coverage: { total_claims_audited: 1, sources_referenced: 1 },
    findings: [],
    claim_checks: [
      {
        claim_quoted: claimQuoted,
        source_reference: { source_path_or_passage_id: sourceId, supporting_values: 'verbatim substring of the revised output' },
      },
    ],
    latency_ms: 12,
  };
  return audit;
}

function buildFreshAuditSidecar(receiptJsonText: string, sectionId: string, revisedOutputHash: string, engineFacts: string, revisedOutput: string, claimQuoted: string, sourceId: string) {
  return {
    kind: 'fresh-audit-of-editorial-revision',
    baseReceiptSha256: sha256(receiptJsonText),
    sectionOutputHash: revisedOutputHash,
    originalOutcome: 'ok',
    revisionVersion: 'v1',
    supplementalEvidence: undefined,
    engineFacts,
    instruction: 'audit only',
    audit: buildCleanAudit(sectionId, revisedOutput, engineFacts, claimQuoted, sourceId),
    llmReceipts: [],
    finalAccepted: false,
  };
}

function buildFreshJevSidecar(sectionId: string, revisedOutputHash: string, verdicts: Record<string, string> = { guardrail: 'pass', framing: 'pass', grounding: 'pass', register: 'pass' }) {
  return {
    sectionOutputHash: revisedOutputHash,
    revisionVersion: 'v1',
    rubric: { section_id: sectionId, target_words: 500, actual_words: 500 },
    receipt: {
      pass_id: sectionId,
      mode: 'shadow',
      status: 'judged',
      verdicts,
      blocked: false,
      disagreements: [],
    },
    finalAccepted: false,
  };
}

// ─── fixture builder ──────────────────────────────────────────────────

interface Fixture {
  baseRunDir: string;
  manifestPath: string;
  identity: EditorialPrefixIdentity;
  openingReceipt: SectionExecutionReceipt;
  openingReceiptText: string;
  openingRevisedOutput: string;
  openingRevisedHash: string;
  openingAudit: ReturnType<typeof buildFreshAuditSidecar>;
  openingJev: ReturnType<typeof buildFreshJevSidecar>;
  part1Receipt: SectionExecutionReceipt;
  part1ReceiptText: string;
  part1RevisedOutput: string;
  part1RevisedHash: string;
  part1Audit: ReturnType<typeof buildFreshAuditSidecar>;
  part1Jev: ReturnType<typeof buildFreshJevSidecar>;
}

const createdDirs: string[] = [];

async function buildFixture(): Promise<Fixture> {
  const dir = await fs.mkdtemp(join(tmpdir(), 'editorial-prefix-'));
  createdDirs.push(dir);
  const baseRunDir = join(dir, 'base-run');
  await fs.mkdir(baseRunDir, { recursive: true });

  await fs.writeFile(join(baseRunDir, 'input.snapshot.json'), INPUT_SNAPSHOT);
  await fs.writeFile(join(baseRunDir, 'inputs.json'), JSON.stringify({
    inputHash: INPUT_HASH,
    subject: 'sheshnarayan',
    language: 'en',
    accountId: 'a'.repeat(32),
    index: 'witness-wisdom-corpus',
    personaHashes: [ALETHEIOS_PERSONA_HASH, PICHET_PERSONA_HASH],
    runtimeHashes: {},
  }));

  const openingReceipt = makeReceipt({ sectionId: 'opening', synthesis: OPENING_ORIGINAL, engineFacts: ENGINE_FACTS_OPENING });
  const openingReceiptText = JSON.stringify(openingReceipt, null, 2);
  await fs.writeFile(join(baseRunDir, 'opening.receipt.json'), openingReceiptText);
  const { revision: openingRevision } = buildRevisionArtifact(openingReceipt, OPENING_PATCHES);
  expect(openingRevision.revisedOutput).toBe(OPENING_REVISED_EXPECTED);
  await fs.writeFile(join(baseRunDir, 'opening.editorial-revision.json'), JSON.stringify(openingRevision, null, 2));
  const openingAudit = buildFreshAuditSidecar(openingReceiptText, 'opening', openingRevision.revisedOutputHash, ENGINE_FACTS_OPENING, openingRevision.revisedOutput, 'Framing differences are conventions, not contradictions.', 'crates/engine-transits/src/ephemeris.rs:42');
  await fs.writeFile(join(baseRunDir, 'opening.audit-recovery.json'), JSON.stringify(openingAudit, null, 2));
  const openingJev = buildFreshJevSidecar('opening', openingRevision.revisedOutputHash);
  await fs.writeFile(join(baseRunDir, 'opening.editorial-jev.json'), JSON.stringify(openingJev, null, 2));

  const part1Receipt = makeReceipt({ sectionId: 'part1', synthesis: PART1_ORIGINAL, engineFacts: ENGINE_FACTS_PART1 });
  const part1ReceiptText = JSON.stringify(part1Receipt, null, 2);
  await fs.writeFile(join(baseRunDir, 'part1.receipt.json'), part1ReceiptText);
  const { revision: part1Revision } = buildRevisionArtifact(part1Receipt, PART1_PATCHES);
  expect(part1Revision.revisedOutput).toBe(PART1_REVISED_EXPECTED);
  await fs.writeFile(join(baseRunDir, 'part1.editorial-revision-v3.json'), JSON.stringify(part1Revision, null, 2));
  const part1Audit = buildFreshAuditSidecar(part1ReceiptText, 'part1', part1Revision.revisedOutputHash, ENGINE_FACTS_PART1, part1Revision.revisedOutput, 'No planets are marked as exalted in the supplied record.', 'reports-reference/engines/sheshnarayan.prod.json#vedic-kundali.planets');
  await fs.writeFile(join(baseRunDir, 'part1.audit-recovery-v3.json'), JSON.stringify(part1Audit, null, 2));
  const part1Jev = buildFreshJevSidecar('part1', part1Revision.revisedOutputHash);
  await fs.writeFile(join(baseRunDir, 'part1.editorial-jev-v3.json'), JSON.stringify(part1Jev, null, 2));

  const manifest = {
    baseRunDir,
    sections: [
      { id: 'opening', revisionFile: 'opening.editorial-revision.json', auditFile: 'opening.audit-recovery.json', jevFile: 'opening.editorial-jev.json' },
      { id: 'part1', revisionFile: 'part1.editorial-revision-v3.json', auditFile: 'part1.audit-recovery-v3.json', jevFile: 'part1.editorial-jev-v3.json' },
    ],
  };
  const manifestPath = join(dir, 'editorial-prefix.manifest.json');
  await fs.writeFile(manifestPath, JSON.stringify(manifest, null, 2));

  const identity: EditorialPrefixIdentity = {
    inputHash: INPUT_HASH,
    subject: 'sheshnarayan',
    language: 'en',
    accountId: 'a'.repeat(32),
    index: 'witness-wisdom-corpus',
    aletheiosPersonaHash: ALETHEIOS_PERSONA_HASH,
    pichetPersonaHash: PICHET_PERSONA_HASH,
    sourceTexts: SOURCE_TEXTS,
  };

  return {
    baseRunDir,
    manifestPath,
    identity,
    openingReceipt,
    openingReceiptText,
    openingRevisedOutput: openingRevision.revisedOutput,
    openingRevisedHash: openingRevision.revisedOutputHash,
    openingAudit,
    openingJev,
    part1Receipt,
    part1ReceiptText,
    part1RevisedOutput: part1Revision.revisedOutput,
    part1RevisedHash: part1Revision.revisedOutputHash,
    part1Audit,
    part1Jev,
  };
}

afterAll(async () => {
  for (const d of createdDirs) {
    await fs.rm(d, { recursive: true, force: true });
  }
});

// ─── tests ────────────────────────────────────────────────────────────

const SECTION_IDS = ['opening', 'part1', 'part2', 'part3'];

describe('loadEditorialPrefix — happy path', () => {
  it('requires the independently validated generation registry hash when the base binds one', async () => {
    const fx = await buildFixture();
    const file = join(fx.baseRunDir, 'inputs.json');
    const input = JSON.parse(await fs.readFile(file, 'utf8'));
    input.primaryRegistry = {sha256: sha256('approved registry bytes')};
    await fs.writeFile(file, JSON.stringify(input));
    for (const value of [undefined, sha256('changed registry')]) {
      const result = await loadEditorialPrefix(fx.manifestPath, SECTION_IDS, {...fx.identity, primaryRegistrySha256: value});
      expect(result.valid).toBe(false);
      if (!result.valid) expect(result.reason).toContain('primary registry hash mismatch');
    }
    const result = await loadEditorialPrefix(fx.manifestPath, SECTION_IDS, {...fx.identity, primaryRegistrySha256: input.primaryRegistry.sha256});
    expect(result.valid).toBe(true);
  });
  it('accepts a valid contiguous editorial prefix and supplies the revised output', async () => {
    const fx = await buildFixture();
    const res = await loadEditorialPrefix(fx.manifestPath, SECTION_IDS, fx.identity);
    expect(res.valid).toBe(true);
    if (!res.valid) return;
    expect(res.prefixSections.map(s => s.section_id)).toEqual(['opening', 'part1']);
    expect(res.prefixSections[0].revisedOutput).toBe(OPENING_REVISED_EXPECTED);
    expect(res.prefixSections[1].revisedOutput).toBe(PART1_REVISED_EXPECTED);
    // Original receipt is not mutated: raw synthesis stays as the original text,
    // output_hash still binds the original text, and outcome is preserved.
    expect(res.prefixSections[0].originalReceipt.raw.synthesis).toBe(OPENING_ORIGINAL);
    expect(res.prefixSections[0].originalReceipt.output_hash).toBe(sha256(OPENING_ORIGINAL));
    expect(res.prefixSections[0].originalReceipt.outcome).toBe('ok');
    expect(res.prefixSections[1].originalReceipt.raw.synthesis).toBe(PART1_ORIGINAL);
    expect(res.prefixSections[1].originalReceipt.output_hash).toBe(sha256(PART1_ORIGINAL));
    expect(res.pendingAmbiguity).toEqual([]);
    expect(res.provenance).toContain('Editorial-prefix continuation from:');
    expect(res.provenance).toContain('generation-only');
  });

  it('records ambiguity when Jev returns could-not-tell on a dimension', async () => {
    const fx = await buildFixture();
    const ambiguousJev = buildFreshJevSidecar('opening', fx.openingRevisedHash, {
      guardrail: 'could-not-tell', framing: 'pass', grounding: 'pass', register: 'pass',
    });
    await fs.writeFile(join(fx.baseRunDir, 'opening.editorial-jev.json'), JSON.stringify(ambiguousJev, null, 2));
    const res = await loadEditorialPrefix(fx.manifestPath, SECTION_IDS, fx.identity);
    expect(res.valid).toBe(true);
    if (!res.valid) return;
    expect(res.pendingAmbiguity).toHaveLength(1);
    expect(res.pendingAmbiguity[0]).toMatchObject({ section_id: 'opening', ambiguousVerdicts: ['guardrail'] });
    expect(res.provenance).toContain('Pending editorial acceptance');
  });
});

describe('loadEditorialPrefix — tampering / staleness rejection', () => {
  it('rejects a tampered revisedOutputHash (recompute vs stored mismatch)', async () => {
    const fx = await buildFixture();
    const badRevision = JSON.parse(await fs.readFile(join(fx.baseRunDir, 'opening.editorial-revision.json'), 'utf8'));
    badRevision.revisedOutputHash = sha256('someone-tampered');
    await fs.writeFile(join(fx.baseRunDir, 'opening.editorial-revision.json'), JSON.stringify(badRevision));
    const res = await loadEditorialPrefix(fx.manifestPath, SECTION_IDS, fx.identity);
    expect(res.valid).toBe(false);
    if (res.valid) return;
    expect(res.reason).toMatch(/recomputed revisedOutputHash/);
  });

  it('rejects a tampered stored revisedOutput text (recompute mismatch)', async () => {
    const fx = await buildFixture();
    const badRevision = JSON.parse(await fs.readFile(join(fx.baseRunDir, 'opening.editorial-revision.json'), 'utf8'));
    badRevision.revisedOutput = 'a completely different edited draft that was never really produced by the patches';
    await fs.writeFile(join(fx.baseRunDir, 'opening.editorial-revision.json'), JSON.stringify(badRevision));
    const res = await loadEditorialPrefix(fx.manifestPath, SECTION_IDS, fx.identity);
    expect(res.valid).toBe(false);
    if (res.valid) return;
    expect(res.reason).toMatch(/recomputed revisedOutput|never trust arbitrary edited artifact/);
  });

  it('rejects a stale audit sectionOutputHash (does not match revised output)', async () => {
    const fx = await buildFixture();
    const staleAudit = { ...fx.openingAudit, sectionOutputHash: sha256('some-other-text') };
    await fs.writeFile(join(fx.baseRunDir, 'opening.audit-recovery.json'), JSON.stringify(staleAudit));
    const res = await loadEditorialPrefix(fx.manifestPath, SECTION_IDS, fx.identity);
    expect(res.valid).toBe(false);
    if (res.valid) return;
    expect(res.reason).toMatch(/audit\.sectionOutputHash.*stale audit/);
  });

  it('rejects an audit whose engineFacts differ from the original receipt', async () => {
    const fx = await buildFixture();
    const drifted = { ...fx.openingAudit, engineFacts: 'DIFFERENT ENGINE FACTS' };
    await fs.writeFile(join(fx.baseRunDir, 'opening.audit-recovery.json'), JSON.stringify(drifted));
    const res = await loadEditorialPrefix(fx.manifestPath, SECTION_IDS, fx.identity);
    expect(res.valid).toBe(false);
    if (res.valid) return;
    expect(res.reason).toMatch(/engineFacts.*differs from receipt/);
  });

  it('rejects an audit whose input_sha256 does not match computeAuditInputHash', async () => {
    const fx = await buildFixture();
    const bad = JSON.parse(JSON.stringify(fx.openingAudit));
    bad.audit.input_sha256 = sha256('wrong input hash');
    await fs.writeFile(join(fx.baseRunDir, 'opening.audit-recovery.json'), JSON.stringify(bad));
    const res = await loadEditorialPrefix(fx.manifestPath, SECTION_IDS, fx.identity);
    expect(res.valid).toBe(false);
    if (res.valid) return;
    expect(res.reason).toMatch(/audit\.input_sha256.*canonical input drift/);
  });

  it('rejects a non-clean fresh audit (findings present / status not clean)', async () => {
    const fx = await buildFixture();
    const badAudit = JSON.parse(JSON.stringify(fx.openingAudit));
    badAudit.audit.status = 'issues';
    badAudit.audit.findings = [{ id: 'f1', severity: 'medium', claim_quoted: 'x', source_reference: { source_path_or_passage_id: 'y' }, analysis: 'z', required_correction: 'w' }];
    // Rehash the model_raw to still match output_sha256 so we hit the status/findings gate deterministically.
    badAudit.audit.model_raw = JSON.stringify({ status: 'issues', coverage: badAudit.audit.coverage, findings: badAudit.audit.findings, claim_checks: [] });
    badAudit.audit.output_sha256 = sha256(badAudit.audit.model_raw);
    await fs.writeFile(join(fx.baseRunDir, 'opening.audit-recovery.json'), JSON.stringify(badAudit));
    const res = await loadEditorialPrefix(fx.manifestPath, SECTION_IDS, fx.identity);
    expect(res.valid).toBe(false);
    if (res.valid) return;
    expect(res.reason).toMatch(/audit\.status is "issues"/);
  });

  it('rejects when audit.output_sha256 does not match sha256(model_raw) — raw ledger tampered', async () => {
    const fx = await buildFixture();
    const badAudit = JSON.parse(JSON.stringify(fx.openingAudit));
    badAudit.audit.model_raw = badAudit.audit.model_raw + ' TAMPERED';
    // leave output_sha256 stale
    await fs.writeFile(join(fx.baseRunDir, 'opening.audit-recovery.json'), JSON.stringify(badAudit));
    const res = await loadEditorialPrefix(fx.manifestPath, SECTION_IDS, fx.identity);
    expect(res.valid).toBe(false);
    if (res.valid) return;
    expect(res.reason).toMatch(/raw ledger tampered/);
  });

  it('rejects a stale Jev sectionOutputHash', async () => {
    const fx = await buildFixture();
    const stale = { ...fx.openingJev, sectionOutputHash: sha256('other') };
    await fs.writeFile(join(fx.baseRunDir, 'opening.editorial-jev.json'), JSON.stringify(stale));
    const res = await loadEditorialPrefix(fx.manifestPath, SECTION_IDS, fx.identity);
    expect(res.valid).toBe(false);
    if (res.valid) return;
    expect(res.reason).toMatch(/jev\.sectionOutputHash.*stale jev/);
  });

  it('rejects a Jev with any fail verdict', async () => {
    const fx = await buildFixture();
    const failed = buildFreshJevSidecar('opening', fx.openingRevisedHash, { guardrail: 'fail', framing: 'pass', grounding: 'pass', register: 'pass' });
    await fs.writeFile(join(fx.baseRunDir, 'opening.editorial-jev.json'), JSON.stringify(failed));
    const res = await loadEditorialPrefix(fx.manifestPath, SECTION_IDS, fx.identity);
    expect(res.valid).toBe(false);
    if (res.valid) return;
    expect(res.reason).toMatch(/fail verdict\(s\): guardrail/);
  });

  it('rejects a Jev that is blocked or not judged', async () => {
    const fx = await buildFixture();
    const bad = { ...fx.openingJev, receipt: { ...fx.openingJev.receipt, blocked: true } };
    await fs.writeFile(join(fx.baseRunDir, 'opening.editorial-jev.json'), JSON.stringify(bad));
    const res = await loadEditorialPrefix(fx.manifestPath, SECTION_IDS, fx.identity);
    expect(res.valid).toBe(false);
    if (res.valid) return;
    expect(res.reason).toMatch(/receipt\.blocked === true/);
  });

  it('rejects when fresh preflight source text differs from the receipt hash', async () => {
    const fx = await buildFixture();
    const drifted: EditorialPrefixIdentity = {
      ...fx.identity,
      sourceTexts: { ...SOURCE_TEXTS, [SOURCE_IDS[0]]: PASSAGE_A + ' (drifted)' },
    };
    const res = await loadEditorialPrefix(fx.manifestPath, SECTION_IDS, drifted);
    expect(res.valid).toBe(false);
    if (res.valid) return;
    expect(res.reason).toMatch(/hash differs from fresh preflight/);
  });

  it('rejects when baseRunDir input.snapshot no longer matches current inputHash', async () => {
    const fx = await buildFixture();
    await fs.writeFile(join(fx.baseRunDir, 'input.snapshot.json'), INPUT_SNAPSHOT + ' drift');
    const res = await loadEditorialPrefix(fx.manifestPath, SECTION_IDS, fx.identity);
    expect(res.valid).toBe(false);
    if (res.valid) return;
    expect(res.reason).toMatch(/baseRunDir input\.snapshot\.json hash does not match/);
  });

  it('rejects when subject/language/persona identity in inputs.json diverges', async () => {
    const fx = await buildFixture();
    const drifted = JSON.parse(await fs.readFile(join(fx.baseRunDir, 'inputs.json'), 'utf8'));
    drifted.subject = 'someone-else';
    await fs.writeFile(join(fx.baseRunDir, 'inputs.json'), JSON.stringify(drifted));
    const res = await loadEditorialPrefix(fx.manifestPath, SECTION_IDS, fx.identity);
    expect(res.valid).toBe(false);
    if (res.valid) return;
    expect(res.reason).toMatch(/identity mismatch/);
  });

  it('rejects noncontiguous manifest ids (skips part1 to reach part2)', async () => {
    const fx = await buildFixture();
    const badManifest = JSON.parse(await fs.readFile(fx.manifestPath, 'utf8'));
    badManifest.sections = [
      { id: 'opening', revisionFile: 'opening.editorial-revision.json', auditFile: 'opening.audit-recovery.json', jevFile: 'opening.editorial-jev.json' },
      { id: 'part2', revisionFile: 'part1.editorial-revision-v3.json', auditFile: 'part1.audit-recovery-v3.json', jevFile: 'part1.editorial-jev-v3.json' },
    ];
    await fs.writeFile(fx.manifestPath, JSON.stringify(badManifest));
    const res = await loadEditorialPrefix(fx.manifestPath, SECTION_IDS, fx.identity);
    expect(res.valid).toBe(false);
    if (res.valid) return;
    expect(res.reason).toMatch(/contiguous ordered prefix/);
  });
});

describe('loadEditorialPrefix — base outcome policy', () => {
  it('rejects a failed base receipt whose attempts include a non-source_audit failure', async () => {
    const fx = await buildFixture();
    const badAttempts: SectionAttemptRecord[] = [
      { attempt: 1, outcome: 'voice_error', aletheios_raw: '', pichet_raw: '', synthesis_raw: '' },
    ];
    const badReceipt = makeReceipt({ sectionId: 'opening', synthesis: OPENING_ORIGINAL, engineFacts: ENGINE_FACTS_OPENING, outcome: 'failed', attempts: badAttempts });
    const badText = JSON.stringify(badReceipt, null, 2);
    await fs.writeFile(join(fx.baseRunDir, 'opening.receipt.json'), badText);
    // Rebuild opening audit sidecar to point at the new receipt bytes so we're only testing the outcome policy.
    const audit = buildFreshAuditSidecar(badText, 'opening', fx.openingRevisedHash, ENGINE_FACTS_OPENING, fx.openingRevisedOutput, 'Framing differences are conventions, not contradictions.', 'crates/engine-transits/src/ephemeris.rs:42');
    await fs.writeFile(join(fx.baseRunDir, 'opening.audit-recovery.json'), JSON.stringify(audit));
    const res = await loadEditorialPrefix(fx.manifestPath, SECTION_IDS, fx.identity);
    expect(res.valid).toBe(false);
    if (res.valid) return;
    expect(res.reason).toMatch(/editorial_original_failed_disallowed_kinds:voice_error/);
  });

  it('accepts a failed base receipt whose only failed attempts were source_audit_failed, with a clean fresh audit', async () => {
    const fx = await buildFixture();
    const attempts: SectionAttemptRecord[] = [
      { attempt: 1, outcome: 'source_audit_failed', aletheios_raw: makeAletheios('part1'), pichet_raw: makePichet('part1'), synthesis_raw: PART1_ORIGINAL },
      { attempt: 2, outcome: 'source_audit_failed', aletheios_raw: makeAletheios('part1'), pichet_raw: makePichet('part1'), synthesis_raw: PART1_ORIGINAL },
    ];
    const failedButRecoverable = makeReceipt({ sectionId: 'part1', synthesis: PART1_ORIGINAL, engineFacts: ENGINE_FACTS_PART1, outcome: 'failed', attempts });
    const text = JSON.stringify(failedButRecoverable, null, 2);
    await fs.writeFile(join(fx.baseRunDir, 'part1.receipt.json'), text);
    // Rebuild part1 audit sidecar to bind to the new receipt bytes.
    const audit = buildFreshAuditSidecar(text, 'part1', fx.part1RevisedHash, ENGINE_FACTS_PART1, fx.part1RevisedOutput, 'No planets are marked as exalted in the supplied record.', 'reports-reference/engines/sheshnarayan.prod.json#vedic-kundali.planets');
    await fs.writeFile(join(fx.baseRunDir, 'part1.audit-recovery-v3.json'), JSON.stringify(audit));
    const res = await loadEditorialPrefix(fx.manifestPath, SECTION_IDS, fx.identity);
    expect(res.valid).toBe(true);
    if (!res.valid) return;
    expect(res.prefixSections.map(s => s.section_id)).toEqual(['opening', 'part1']);
    expect(res.prefixSections[1].originalReceipt.outcome).toBe('failed');
    expect(res.prefixSections[1].revisedOutput).toBe(PART1_REVISED_EXPECTED);
  });
});


describe('editorial prefix implementation evidence', () => {
  async function withEvidence() {
    const fx = await buildFixture();
    const evidence = { path: 'engine.rs', sha256: sha256('verified code'), text: 'verified code', scope: 'Current local implementation; not independent proof of the saved production binary version' };
    const engineFacts = ENGINE_FACTS_OPENING + '\nAdditional implementation evidence, with limited scope:\n' + JSON.stringify(evidence);
    const audit = { ...fx.openingAudit, supplementalEvidence: evidence, engineFacts, audit: { ...fx.openingAudit.audit, input_sha256: computeAuditInputHash({ acceptedOutput: fx.openingRevisedOutput, passagesHash: PASSAGES_HASH, engineFacts }) } };
    await fs.writeFile(join(fx.baseRunDir, 'opening.audit-recovery.json'), JSON.stringify(audit));
    fx.identity.implementationSourceTexts = { 'engine.rs': 'verified code' };
    return fx;
  }
  it('accepts exact explicitly supplied current code with bounded scope', async () => { const fx=await withEvidence(); expect((await loadEditorialPrefix(fx.manifestPath, SECTION_IDS, fx.identity)).valid).toBe(true); });
  it('rejects changed implementation bytes', async () => { const fx=await withEvidence(); fx.identity.implementationSourceTexts={'engine.rs':'changed code'}; const r=await loadEditorialPrefix(fx.manifestPath, SECTION_IDS, fx.identity); expect(r.valid).toBe(false); if(!r.valid)expect(r.reason).toContain('unverified or stale'); });
  it('rejects an unapproved implementation source', async () => { const fx=await withEvidence(); delete fx.identity.implementationSourceTexts; expect((await loadEditorialPrefix(fx.manifestPath, SECTION_IDS, fx.identity)).valid).toBe(false); });
});

// ─── Optional editorial passage-evidence wiring (editorial-prefix) ───
// Focused tests exercising the loader's freshAudit.passageEvidence path
// with identity.trustedPassageSources + identity.expectedCF.  Absent
// evidence must preserve legacy behaviour (backward compatibility);
// independently trusted registry/CF target MUST come from the identity
// (caller options), never from the sidecar itself.

describe('editorial-prefix: optional passage evidence supplement', () => {
  const CF_ACCOUNT = 'a'.repeat(32);
  const CF_INDEX = 'witness-wisdom-corpus';

  async function withPassageEvidence(overrides: {
    mutateAdditions?: (adds: any[]) => any[];
    mutateTrusted?: (rec: Record<string, any>) => Record<string, any>;
    mutateEvidenceSidecar?: (ev: any) => any;
    omitTrustedPassageSources?: boolean;
    omitExpectedCF?: boolean;
  } = {}) {
    const fx = await buildFixture();
    // Base passages must round-trip against the receipt's source_ids +
    // source_hashes + passages_hash.  We reuse SOURCE_IDS from the top
    // of the file (sw:hd:type:generator:desc, sw:hd:auth:sacral-authority:desc).
    const basePassages = SOURCE_IDS.map((id) => ({ id, source: `cf-vectorize:${CF_ACCOUNT}/${CF_INDEX}/${id}`, text: SOURCE_TEXTS[id] }));
    const basePassagesHash = PASSAGES_HASH;

    const CF_ADD_TEXT = 'Fresh CF readback: descriptive framework insight for opening.';
    const PRIM_ADD_TEXT = 'Rao paraphrase: descriptive traditional framing.';
    const cfAdd: any = {
      id: 'sw:hd:framing:desc:extra', text: CF_ADD_TEXT, sha256: sha256(CF_ADD_TEXT),
      provenance: { kind: 'cf-vectorize', account: CF_ACCOUNT, index: CF_INDEX },
    };
    const primAdd: any = {
      id: 'primary:rao2000:opening:framing', text: PRIM_ADD_TEXT, sha256: sha256(PRIM_ADD_TEXT),
      provenance: { kind: 'reviewed-primary-document', author: 'Rao', title: 'Vedic Astrology', url: 'https://www.vedicastrologer.org/articles/vedic_astro_textbook.pdf', locator: 'Ch 7.2', reviewer: 'assistant-source-review' },
    };
    let additions: any[] = [cfAdd, primAdd];
    if (overrides.mutateAdditions) additions = overrides.mutateAdditions(additions);

    let trusted: Record<string, any> = { [cfAdd.id]: cfAdd, [primAdd.id]: primAdd };
    if (overrides.mutateTrusted) trusted = overrides.mutateTrusted(trusted);

    const effectiveJoin = [...SOURCE_IDS.map((id) => SOURCE_TEXTS[id]), ...additions.map((a: any) => a.text)].join('\n');
    const effectivePassagesHash = sha256(effectiveJoin);
    let sidecar: any = {
      basePassages, basePassagesHash, additions,
      addedSourceRecords: additions, effectivePassagesHash,
    };
    if (overrides.mutateEvidenceSidecar) sidecar = overrides.mutateEvidenceSidecar(sidecar);

    // Recompute the opening audit so input_sha256 binds the effective hash.
    const engineFacts = ENGINE_FACTS_OPENING;
    const inputSha = computeAuditInputHash({ acceptedOutput: fx.openingRevisedOutput, passagesHash: sidecar.effectivePassagesHash, engineFacts });
    const patchedAudit = {
      ...fx.openingAudit,
      audit: { ...fx.openingAudit.audit, input_sha256: inputSha },
      passageEvidence: sidecar,
    };
    await fs.writeFile(join(fx.baseRunDir, 'opening.audit-recovery.json'), JSON.stringify(patchedAudit, null, 2));

    const identity: EditorialPrefixIdentity = {
      ...fx.identity,
      trustedPassageSources: overrides.omitTrustedPassageSources ? undefined : trusted,
      expectedCF: overrides.omitExpectedCF ? undefined : { account: CF_ACCOUNT, index: CF_INDEX },
    };
    return { fx, identity };
  }

  it('accepts a supplemented fresh audit bound to the effective passages hash', async () => {
    const { fx, identity } = await withPassageEvidence();
    const res = await loadEditorialPrefix(fx.manifestPath, SECTION_IDS, identity);
    expect(res.valid).toBe(true);
    if (!res.valid) return;
    expect(res.prefixSections[0].section_id).toBe('opening');
  });

  it('preserves existing behavior when no passage evidence is present', async () => {
    // Straight buildFixture path (no sidecar, no trustedPassageSources) must
    // continue to accept the run — backward-compatible.
    const fx = await buildFixture();
    const res = await loadEditorialPrefix(fx.manifestPath, SECTION_IDS, fx.identity);
    expect(res.valid).toBe(true);
  });

  it('blocks a tampered addition text', async () => {
    const { fx, identity } = await withPassageEvidence({
      mutateEvidenceSidecar: (ev) => ({
        ...ev,
        additions: [{ ...ev.additions[0], text: 'TAMPERED payload' }, ev.additions[1]],
      }),
    });
    const res = await loadEditorialPrefix(fx.manifestPath, SECTION_IDS, identity);
    expect(res.valid).toBe(false);
    if (!res.valid) expect(res.reason).toMatch(/passage evidence/);
  });

  it('blocks when base ids are reordered in the sidecar', async () => {
    const { fx, identity } = await withPassageEvidence({
      mutateEvidenceSidecar: (ev) => ({
        ...ev,
        basePassages: [ev.basePassages[1], ev.basePassages[0]],
      }),
    });
    const res = await loadEditorialPrefix(fx.manifestPath, SECTION_IDS, identity);
    expect(res.valid).toBe(false);
    if (!res.valid) expect(res.reason).toContain('base id');
  });

  it('blocks when the sidecar swaps the effective passages hash', async () => {
    const { fx, identity } = await withPassageEvidence({
      mutateEvidenceSidecar: (ev) => ({ ...ev, effectivePassagesHash: sha256('unrelated-value') }),
    });
    const res = await loadEditorialPrefix(fx.manifestPath, SECTION_IDS, identity);
    expect(res.valid).toBe(false);
    if (!res.valid) expect(res.reason).toMatch(/effectivePassagesHash|canonical input drift/);
  });

  it('blocks when identity.trustedPassageSources is omitted (untrusted source)', async () => {
    const { fx, identity } = await withPassageEvidence({ omitTrustedPassageSources: true });
    const res = await loadEditorialPrefix(fx.manifestPath, SECTION_IDS, identity);
    expect(res.valid).toBe(false);
    if (!res.valid) expect(res.reason).toContain('trustedPassageSources');
  });

  it('blocks when a reviewed-primary-document is mislabeled as cf-vectorize', async () => {
    const { fx, identity } = await withPassageEvidence({
      mutateAdditions: (adds) => {
        const [cf, prim] = adds;
        const mislabeled = { ...prim, provenance: { kind: 'cf-vectorize', account: CF_ACCOUNT, index: CF_INDEX } };
        return [cf, mislabeled];
      },
      mutateTrusted: (rec) => {
        const primId = 'primary:rao2000:opening:framing';
        rec[primId] = { ...rec[primId], provenance: { kind: 'cf-vectorize', account: CF_ACCOUNT, index: CF_INDEX } };
        return rec;
      },
    });
    const res = await loadEditorialPrefix(fx.manifestPath, SECTION_IDS, identity);
    expect(res.valid).toBe(false);
    if (!res.valid) expect(res.reason).toMatch(/passage evidence/);
  });
});
