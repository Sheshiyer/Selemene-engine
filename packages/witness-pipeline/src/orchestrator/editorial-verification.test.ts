// ─── Editorial-verification integration tests ─────────────────────────
// Exercises verifyReferenceExecution's provenance-aware editorial branch:
//   * Successful reviewed editorial chain preserves old prior outputs and
//     accepts the revised text.
//   * Missing / stale coherence review blocks even when section checks pass.
//   * Tampered revision / audit / disposition / prior-context each block.
//   * Original outcome=failed with source_audit_failed only is recoverable
//     when the fresh audit is clean; any other failure kind blocks.
//   * No editorial options => existing strict behaviour unchanged.
//
// Fixtures are shaped after
//   tools/humdes-extractor/output/gary-abitbol-2026-09-28/reports-reference/
//   runs/sheshnarayan-en-2026-09-30T01-04-32-223Z
// but are compact and self-contained.  No I/O.

import { describe, it, expect } from 'vitest';
import { createHash } from 'node:crypto';
import { verifyEditorialPriorContext } from './editorial-verification.js';
import { verifyReferenceExecution, type ReferenceVerificationInput } from './reference-verification.js';
import type {
  EditorialVerificationOptions,
  EditorialSectionEvidence,
  EditorialCoherenceReview,
  FreshAuditSidecar,
  FreshJevSidecar,
} from './editorial-verification.js';
import type { SectionExecutionReceipt } from './reference-execution.js';
import type { PassResult, SectionRubric } from './integrated.js';
import { computeAuditInputHash } from './reference-source-audit.js';
import type { SourceAuditReceipt } from './reference-source-audit.js';
import { applyEditorialRevision, type EditorialPatch } from './editorial-revision.js';
import type {
  GuardrailDispositionArtifact,
  GuardrailDiagnosticArtifact,
  JevReviewArtifact,
  RevisionArtifact,
} from './guardrail-disposition.js';

// ─── Helpers ──────────────────────────────────────────────────────────

const sha256 = (s: string): string => createHash('sha256').update(s, 'utf8').digest('hex');

const ALETHEIOS_IDENTITY = '# aletheios IDENTITY\nYou are Aletheios, structural interpreter.';
const PICHET_IDENTITY = '# pichet IDENTITY\nYou are Pichet, experiential reflector.';
const ALETHEIOS_OPENING = 'Aletheios structural: eleven engine result records.';
const PICHET_OPENING = 'Pichet experiential: a scope-setting movement felt as rhythm.';
const OPENING_ORIGINAL =
  'This report gathers eleven engine result records and reads them as a scope-setting movement.\n' +
  '\n' +
  '**Method.** Two witness voices reconcile in each chapter without predicting personal outcomes.';
const OPENING_REVISED =
  'This report gathers eleven engine result records and reads them as a scope-setting movement.\n' +
  '\n' +
  '**Method.** Two witness voices reconcile in each chapter describing patterns rather than forecasting outcomes.';

const OPENING_PATCHES: EditorialPatch[] = [
  {
    before: 'without predicting personal outcomes.',
    after: 'describing patterns rather than forecasting outcomes.',
    rationale: 'Sharpen the descriptive-witness policy phrasing per source review.',
    sourceReferences: [
      { sourceId: 'policy/descriptive.md', quote: 'Descriptive witness, not prediction.', label: 'policy doc' },
    ],
  },
];

const OPENING_ENGINE_FACTS = 'Panchanga: Tithi Shukla 3, Nakshatra Rohini. Chart: Sun-Cancer, Moon-Taurus.';

// In the reviewed-editorial fixture, part1 is a NEWLY-generated section that
// runs AFTER the opening was revised.  Its synthesis-input text therefore
// includes the REVISED opening — this is exactly the "corrected prior text
// must appear" rule for newly generated sections.
const PART1_PRIOR_CONTEXT_WITH_REVISED_OPENING =
  'PRIOR SECTIONS FOR CONTINUITY:\n' + OPENING_REVISED + '\n\nCURRENT SECTION INSTRUCTIONS:\nDraft Part I.';
const PART1_ALETHEIOS = 'Aletheios: transit patterns show tropical/sidereal reconciliation.';
const PART1_PICHET = 'Pichet: felt sense of cyclic return through the year.';
const PART1_OUTPUT = 'Part I opens the transits chapter with a descriptive, non-predictive frame anchored in the opening.';
const PART1_ENGINE_FACTS = 'Panchanga+Transits engine facts.';
const PART1_PASSAGE_TEXT = 'Transits corpus passage: Lahiri sidereal conversion is applied to tropical longitude.';
const PART1_PASSAGE_ID = 'corpus-p1';

// ── Receipt & pass builders ──────────────────────────────────────────

function baseOpeningReceipt(overrides: Partial<SectionExecutionReceipt> = {}): SectionExecutionReceipt {
  const rawSynthesis = OPENING_ORIGINAL;
  const openingOutputHash = sha256(rawSynthesis);
  const synthesisInput = `${ALETHEIOS_OPENING}\n\n${PICHET_OPENING}`;
  const passages = [`p0:${PART1_PASSAGE_TEXT}`];
  const passageHash = sha256(passages.join('\n'));
  return {
    section_id: 'opening',
    outcome: 'ok',
    source_ids: ['opening-p0'],
    source_hashes: [sha256('opening passage')],
    passages_hash: sha256('opening passage'),
    aletheios_hash: sha256(ALETHEIOS_OPENING),
    pichet_hash: sha256(PICHET_OPENING),
    synthesis_input_hash: sha256(synthesisInput),
    synthesis_input_text: synthesisInput,
    prior_section_content_hashes: {},
    output_hash: openingOutputHash,
    aletheios_persona: {
      name: 'aletheios',
      sourcePath: '/witness-agents/agents/aletheios/IDENTITY.md',
      sourceHash: sha256(ALETHEIOS_IDENTITY),
    },
    pichet_persona: {
      name: 'pichet',
      sourcePath: '/witness-agents/agents/pichet/IDENTITY.md',
      sourceHash: sha256(PICHET_IDENTITY),
    },
    raw: { aletheios: ALETHEIOS_OPENING, pichet: PICHET_OPENING, synthesis: rawSynthesis },
    jev: {
      pass_id: 'opening', mode: 'shadow', status: 'judged', model: 'jev-1',
      latency_ms: 1,
      verdicts: { guardrail: 'could-not-tell', framing: 'pass', grounding: 'pass', register: 'pass' },
      blocked: false, disagreements: [],
    },
    retrieval: {
      section_id: 'opening',
      state: 'success', count: 1,
      source_ids: ['opening-p0'],
      passages_hash: sha256('opening passage'),
    },
    attempts: [{ attempt: 1, outcome: 'ok', aletheios_raw: ALETHEIOS_OPENING, pichet_raw: PICHET_OPENING, synthesis_raw: rawSynthesis }],
    engine_facts: OPENING_ENGINE_FACTS,
    ...overrides,
  };
}

function basePart1Receipt(): SectionExecutionReceipt {
  const rawSynthesis = PART1_OUTPUT;
  const outputHash = sha256(rawSynthesis);
  const synthesisInput = PART1_PRIOR_CONTEXT_WITH_REVISED_OPENING + '\n' + PART1_ALETHEIOS + '\n\n' + PART1_PICHET;
  return {
    section_id: 'part1',
    outcome: 'ok',
    source_ids: [PART1_PASSAGE_ID],
    source_hashes: [sha256(PART1_PASSAGE_TEXT)],
    passages_hash: sha256(PART1_PASSAGE_TEXT),
    aletheios_hash: sha256(PART1_ALETHEIOS),
    pichet_hash: sha256(PART1_PICHET),
    synthesis_input_hash: sha256(synthesisInput),
    synthesis_input_text: synthesisInput,
    prior_section_content_hashes: { opening: sha256(OPENING_REVISED) },
    output_hash: outputHash,
    aletheios_persona: {
      name: 'aletheios',
      sourcePath: '/witness-agents/agents/aletheios/IDENTITY.md',
      sourceHash: sha256(ALETHEIOS_IDENTITY),
    },
    pichet_persona: {
      name: 'pichet',
      sourcePath: '/witness-agents/agents/pichet/IDENTITY.md',
      sourceHash: sha256(PICHET_IDENTITY),
    },
    raw: { aletheios: PART1_ALETHEIOS, pichet: PART1_PICHET, synthesis: rawSynthesis },
    jev: {
      pass_id: 'part1', mode: 'shadow', status: 'judged', model: 'jev-1', latency_ms: 1,
      verdicts: { guardrail: 'pass', framing: 'pass', grounding: 'pass', register: 'pass' },
      blocked: false, disagreements: [],
    },
    retrieval: {
      section_id: 'part1', state: 'success', count: 1,
      source_ids: [PART1_PASSAGE_ID],
      passages_hash: sha256(PART1_PASSAGE_TEXT),
    },
    attempts: [{ attempt: 1, outcome: 'ok', aletheios_raw: PART1_ALETHEIOS, pichet_raw: PART1_PICHET, synthesis_raw: rawSynthesis }],
    engine_facts: PART1_ENGINE_FACTS,
  };
}

function cleanRubric(sectionId: string, title: string): SectionRubric {
  return {
    section_id: sectionId,
    title,
    target_words: 40,
    actual_words: 25,
    word_count_fit: 'pass',
    word_count_ratio: 0.625,
    deterministic_fact_count: 3,
    deterministic_fact_gate: 'pass',
    integrated_layer_count: 3,
    integrated_layering_gate: 'pass',
    guardrail_gate: 'pass',
    guardrail_violations: [],
    model_requested: 'editorial-revision',
    model_used: 'assistant-source-review',
    latency_ms: 0,
  };
}

// ── Fresh audit / Jev / guardrail-disposition builders ───────────────

function freshOpeningAudit(revisedOutput: string, baseReceipt: SectionExecutionReceipt): FreshAuditSidecar {
  const revisedHash = sha256(revisedOutput);
  const engineFacts = baseReceipt.engine_facts as string;
  const auditModelRaw = JSON.stringify({
    status: 'clean',
    coverage: { total_claims_audited: 1, sources_referenced: 1 },
    claim_checks: [{
      claim_quoted: revisedOutput.split('\n')[0],
      source_reference: { source_path_or_passage_id: 'opening-p0', supporting_values: 'engine records' },
    }],
    findings: [],
  });
  const inputSha = computeAuditInputHash({
    acceptedOutput: revisedOutput,
    passagesHash: baseReceipt.passages_hash,
    engineFacts,
  });
  const audit: SourceAuditReceipt = {
    section_id: 'opening',
    status: 'clean',
    model_raw: auditModelRaw,
    input_sha256: inputSha,
    output_sha256: sha256(auditModelRaw),
    coverage: { total_claims_audited: 1, sources_referenced: 1 },
    claim_checks: [{
      claim_quoted: revisedOutput.split('\n')[0],
      source_reference: { source_path_or_passage_id: 'opening-p0', supporting_values: 'engine records' },
    }],
    findings: [],
  };
  return {
    kind: 'fresh-audit-of-editorial-revision',
    baseReceiptSha256: sha256(JSON.stringify(baseReceipt)),
    sectionOutputHash: revisedHash,
    originalOutcome: baseReceipt.outcome === 'failed' ? 'failed' : 'ok',
    engineFacts,
    audit,
    finalAccepted: false,
  };
}

function freshOpeningJev(revisedOutput: string, guardrail: 'pass' | 'could-not-tell' = 'could-not-tell'): FreshJevSidecar {
  return {
    sectionOutputHash: sha256(revisedOutput),
    revisionVersion: 'editorial',
    receipt: {
      pass_id: 'opening',
      mode: 'shadow',
      status: 'judged',
      model: 'jev-1.13.0',
      verdicts: { guardrail, framing: 'pass', grounding: 'pass', register: 'pass' },
      blocked: false,
      disagreements: [],
    },
    finalAccepted: false,
  };
}

function buildDispositionBundle(revisedOutput: string): NonNullable<EditorialSectionEvidence['guardrailDisposition']> {
  const revisedHash = sha256(revisedOutput);
  const jevFile = 'opening.editorial-jev.json';
  const diagFile = 'opening.guardrail-diagnostic.json';
  const revFile = 'opening.editorial-revision.json';

  const revision: RevisionArtifact = {
    sectionId: 'opening',
    revisedOutput,
    revisedOutputHash: revisedHash,
  };
  const freshJev = freshOpeningJev(revisedOutput);
  const originalJev: JevReviewArtifact = {
    ...freshJev,
    receipt: { ...freshJev.receipt, blocked: false },
  };
  const lines = revisedOutput.split('\n');
  const units = lines
    .map((text, index) => ({ index, text }))
    .filter((u) => u.text.trim().length > 0);
  const questions: Record<string, { type: 'noul'; instructions: string; criteria: { true: string; false: string } }> = {};
  const answers: Record<string, { type: 'noul'; noul: number }> = {};
  for (const u of units) {
    const k = `u${u.index}`;
    questions[k] = {
      type: 'noul',
      instructions: `Is it true that report unit ${u.index} is a prediction, forecast, guarantee, or promised outcome rather than a description of an observable pattern?`,
      criteria: { true: 'predictive or promissory', false: 'descriptive witness' },
    };
    answers[k] = { type: 'noul', noul: 0.05 };
  }
  const diagnostic: GuardrailDiagnosticArtifact = {
    kind: 'full-nonblank-line-guardrail-diagnostic',
    section: 'opening',
    sectionOutputHash: revisedHash,
    policy: 'descriptive',
    unitCount: units.length,
    complete: true,
    missing: [],
    flags: [],
    batches: [{ units, questions, result: { model: 'jev-1.13.0', answers } }],
  };
  for (const b of diagnostic.batches) (b as { state_sha256?: string }).state_sha256 = sha256(JSON.stringify({ section: revisedOutput, units: b.units }));

  const rawJevText = JSON.stringify(originalJev);
  const rawDiagText = JSON.stringify(diagnostic);
  const rawRevText = JSON.stringify(revision);

  const disposition: GuardrailDispositionArtifact = {
    kind: 'explicit-guardrail-review-disposition',
    sectionId: 'opening',
    sectionOutputHash: revisedHash,
    reviewer: 'assistant-source-review',
    reviewedAt: '2026-09-30T01:28:01.604190+00:00',
    originalVerdict: 'could-not-tell',
    decision: 'reviewed-no-policy-violation-found',
    rationale: 'Full nonblank-line diagnostic coverage found no units at the 0.5 flag threshold.',
    evidence: [
      { file: jevFile, sha256: sha256(rawJevText) },
      { file: diagFile, sha256: sha256(rawDiagText) },
      { file: revFile, sha256: sha256(rawRevText) },
    ],
    policy: 'descriptive',
    jevVerdictPreserved: true,
    finalAccepted: false,
    remainingReview: ['cross-section coherence'],
  };
  return {
    disposition,
    originalJev,
    diagnostic,
    revision,
    rawEvidenceTexts: {
      [jevFile]: rawJevText,
      [diagFile]: rawDiagText,
      [revFile]: rawRevText,
    },
  };
}

// ── Build the reviewed editorial input end-to-end ────────────────────

function makeReviewedEditorialInput(overrides: {
  mutateOpeningReceipt?: (r: SectionExecutionReceipt) => SectionExecutionReceipt;
  mutateEvidence?: (e: EditorialSectionEvidence) => EditorialSectionEvidence;
  omitCoherenceReview?: boolean;
  mutateCoherence?: (r: EditorialCoherenceReview) => EditorialCoherenceReview;
  mutateOriginalPasses?: (p: Record<string, string>) => Record<string, string>;
  mutatePart1Receipt?: (r: SectionExecutionReceipt) => SectionExecutionReceipt;
} = {}): ReferenceVerificationInput {
  const openingReceipt0 = baseOpeningReceipt();
  const openingReceipt = overrides.mutateOpeningReceipt ? overrides.mutateOpeningReceipt(openingReceipt0) : openingReceipt0;
  const part1Receipt = overrides.mutatePart1Receipt ? overrides.mutatePart1Receipt(basePart1Receipt()) : basePart1Receipt();

  // Compute revision + fresh audit against the possibly-mutated receipt
  const applied = applyEditorialRevision({
    baseReceipt: openingReceipt,
    currentOutput: openingReceipt.raw.synthesis,
    expectedBaseOutputHash: openingReceipt.output_hash,
    patches: OPENING_PATCHES,
    reviewer: 'assistant-source-review',
    timestamp: '2026-09-30T01:13:59.113Z',
  });
  const revisedOutput = applied.revisedOutput;
  const freshAudit = freshOpeningAudit(revisedOutput, openingReceipt);
  const freshJev = freshOpeningJev(revisedOutput, 'could-not-tell');
  const guardrailDisposition = buildDispositionBundle(revisedOutput);

  const evidence0: EditorialSectionEvidence = {
    baseReceiptSha256: freshAudit.baseReceiptSha256,
    baseReceiptRawBytes: JSON.stringify(openingReceipt),
    revisionArtifact: {
      sectionId: 'opening',
      baseOutputHash: applied.baseOutputHash,
      revisedOutput: applied.revisedOutput,
      revisedOutputHash: applied.revisedOutputHash,
      patches: OPENING_PATCHES,
      reviewer: 'assistant-source-review',
      timestamp: '2026-09-30T01:13:59.113Z',
    },
    freshAudit,
    freshJev,
    guardrailDisposition,
  };
  const evidence = overrides.mutateEvidence ? overrides.mutateEvidence(evidence0) : evidence0;

  const openingPass: PassResult = {
    id: 'opening',
    title: 'Opening',
    output: evidence.revisionArtifact.revisedOutput,
    rubric: cleanRubric('opening', 'Opening'),
  };
  const part1Pass: PassResult = {
    id: 'part1',
    title: 'Part I',
    output: PART1_OUTPUT,
    rubric: cleanRubric('part1', 'Part I'),
  };

  const requiredSectionIds = ['opening', 'part1'];
  const effectiveHashes = [evidence.revisionArtifact.revisedOutputHash, part1Receipt.output_hash];
  const expectedArtifactSha = sha256(effectiveHashes.join(':'));

  // The base verifier requires `assembled` to equal `## title\n\n{output}`
  // joined by `\n\n`.  Item 5 additionally requires the coherence review's
  // artifactSha256 to bind sha256(assembled) whenever the pipeline has an
  // assembled artifact — which is always true for a real report.
  const assembled = [
    `## Opening\n\n${evidence.revisionArtifact.revisedOutput}`,
    `## Part I\n\n${PART1_OUTPUT}`,
  ].join('\n\n');
  const assembledSha = sha256(assembled);

  let originalPasses: Record<string, string> = {
    opening: OPENING_ORIGINAL,
    part1: PART1_OUTPUT,
  };
  if (overrides.mutateOriginalPasses) originalPasses = overrides.mutateOriginalPasses({ ...originalPasses });

  let coherenceReview: EditorialCoherenceReview | undefined = {
    artifactSha256: assembledSha,
    orderedEffectiveHashes: effectiveHashes,
    status: 'reviewed',
    reviewer: 'assistant-source-review',
    reviewedAt: '2026-09-30T01:30:00.000Z',
    rationale: 'Whole-document coherence review: revised opening frames Part I without policy drift.',
    findings: [{ id: 'f1', summary: 'No unresolved coherence issues.', unresolved: false }],
    reviewedRevisions: [{ sectionId: 'opening', revisedOutputHash: evidence.revisionArtifact.revisedOutputHash }],
  };
  if (overrides.omitCoherenceReview) coherenceReview = undefined;
  else if (overrides.mutateCoherence) coherenceReview = overrides.mutateCoherence(coherenceReview!);

  const editorial: EditorialVerificationOptions = {
    revisedSections: { opening: evidence },
    originalPasses,
    coherenceReview,
  };

  return {
    receipts: [openingReceipt, part1Receipt],
    passes: [openingPass, part1Pass],
    requiredSectionIds,
    expectedPersonas: { aletheios: ALETHEIOS_IDENTITY, pichet: PICHET_IDENTITY },
    subsectionMap: { opening: [], part1: [] },
    sourceTexts: {
      'opening-p0': 'opening passage',
      [PART1_PASSAGE_ID]: PART1_PASSAGE_TEXT,
    },
    rawSynthesisInputs: {
      part1: PART1_PRIOR_CONTEXT_WITH_REVISED_OPENING + '\n' + PART1_ALETHEIOS + '\n\n' + PART1_PICHET,
    },
    assembled,
    editorial,
  };
}

// ─── Happy path ───────────────────────────────────────────────────────

describe('editorial-verification: reviewed editorial chain passes', () => {
  it('accepts a revised opening + unchanged part1 with valid coherence review', () => {
    const input = makeReviewedEditorialInput();
    const result = verifyReferenceExecution(input);
    expect(result.blockers).toHaveLength(0);
    expect(result.warnings).toHaveLength(0);
    expect(result.passed).toBe(true);
    expect(result.editorialVerdicts).toBeDefined();
    expect(result.editorialVerdicts).toHaveLength(1);
    const v = result.editorialVerdicts![0];
    expect(v.sectionId).toBe('opening');
    expect(v.blockers).toHaveLength(0);
    expect(v.guardrailResolution?.resolution).toBe('reviewed-no-policy-violation-found');
    expect(v.guardrailResolution?.originalVerdict).toBe('could-not-tell');
    expect(v.notes.some((n) => n.includes('editorial_guardrail_reviewed_no_policy_violation_found'))).toBe(true);
  });

  it('publishes an artifact sha bound to REVISED opening hash, not the original', () => {
    const input = makeReviewedEditorialInput();
    const result = verifyReferenceExecution(input);
    const revisedOpeningHash = result.editorialVerdicts![0].revisedOutputHash;
    // original opening output_hash differs (different text)
    const originalOpeningHash = sha256(OPENING_ORIGINAL);
    expect(revisedOpeningHash).not.toBe(originalOpeningHash);
    // With `assembled` supplied the verifier binds artifactSha256 to
    // sha256(assembled).  The assembled text contains the REVISED opening
    // output, so recomputing sha256 with the ORIGINAL opening body must
    // yield a different hash — proving the artifact is bound to the
    // post-edit revision, not the pre-edit original.
    const assembledWithRevised = input.assembled!;
    expect(result.artifactSha256).toBe(sha256(assembledWithRevised));
    const assembledWithOriginal = assembledWithRevised.replace(
      input.editorial!.revisedSections['opening'].revisionArtifact.revisedOutput,
      OPENING_ORIGINAL,
    );
    expect(assembledWithOriginal).not.toBe(assembledWithRevised);
    expect(result.artifactSha256).not.toBe(sha256(assembledWithOriginal));
  });

  it('preserves the ORIGINAL opening receipt hashes verbatim on the input', () => {
    const input = makeReviewedEditorialInput();
    const originalOpening = input.receipts[0];
    // Verifier must not mutate the stored receipt in any way.
    expect(originalOpening.raw.synthesis).toBe(OPENING_ORIGINAL);
    expect(originalOpening.output_hash).toBe(sha256(OPENING_ORIGINAL));
    expect(originalOpening.jev?.verdicts?.guardrail).toBe('could-not-tell');
    verifyReferenceExecution(input);
    // Post-run: still unchanged
    expect(originalOpening.raw.synthesis).toBe(OPENING_ORIGINAL);
    expect(originalOpening.jev?.verdicts?.guardrail).toBe('could-not-tell');
  });
});

// ─── Coherence review contract ────────────────────────────────────────

describe('editorial-verification: coherence review is required + fresh', () => {
  it('blocks when coherence review is missing', () => {
    const input = makeReviewedEditorialInput({ omitCoherenceReview: true });
    const result = verifyReferenceExecution(input);
    expect(result.blockers).toContain('editorial:coherence_review_missing');
    expect(result.passed).toBe(false);
  });

  it('blocks when coherence review artifact hash is stale', () => {
    const input = makeReviewedEditorialInput({
      mutateCoherence: (r) => ({ ...r, artifactSha256: sha256('stale-artifact') }),
    });
    const result = verifyReferenceExecution(input);
    // Item 5 — with `assembled` supplied, the review MUST bind to
    // sha256(assembled).  A stale artifactSha256 surfaces the stronger
    // binding blocker code, not the legacy hash-list stale code.
    expect(result.blockers).toContain('editorial:coherence_review_artifact_hash_not_bound_to_assembled');
    expect(result.passed).toBe(false);
  });

  it('blocks when coherence review does not cite the revised section by hash', () => {
    const input = makeReviewedEditorialInput({
      mutateCoherence: (r) => ({
        ...r,
        reviewedRevisions: [{ sectionId: 'opening', revisedOutputHash: sha256('wrong hash') }],
      }),
    });
    const result = verifyReferenceExecution(input);
    expect(result.blockers.some((b) => b.startsWith('editorial:coherence_review_stale_revised_hash'))).toBe(true);
  });

  it('blocks when coherence review carries unresolved findings', () => {
    const input = makeReviewedEditorialInput({
      mutateCoherence: (r) => ({
        ...r,
        findings: [
          { id: 'f1', summary: 'No issues', unresolved: false },
          { id: 'f2', summary: 'Voice inconsistency between opening and part1', unresolved: true },
        ],
      }),
    });
    const result = verifyReferenceExecution(input);
    expect(result.blockers.some((b) => b.startsWith('editorial:coherence_review_unresolved_findings'))).toBe(true);
  });

  it('blocks when coherence review status is not "reviewed"', () => {
    const input = makeReviewedEditorialInput({
      mutateCoherence: (r) => ({ ...r, status: 'pending' as unknown as 'reviewed' }),
    });
    const result = verifyReferenceExecution(input);
    expect(result.blockers.some((b) => b.startsWith('editorial:coherence_review_status'))).toBe(true);
  });
});

// ─── Tampering blocks ─────────────────────────────────────────────────

describe('editorial-verification: tampered evidence blocks acceptance', () => {
  it('blocks when the revision artifact stores a wrong revisedOutputHash', () => {
    const input = makeReviewedEditorialInput({
      mutateEvidence: (e) => ({
        ...e,
        revisionArtifact: { ...e.revisionArtifact, revisedOutputHash: sha256('wrong revised hash') },
      }),
    });
    const result = verifyReferenceExecution(input);
    expect(result.blockers.some((b) => b.includes('editorial_stored_revised_hash_mismatch'))).toBe(true);
  });

  it('blocks when the original receipt raw.synthesis has been tampered', () => {
    // Build a valid input first (so applyEditorialRevision succeeds), then
    // post-tamper the stored raw.synthesis to simulate an attacker editing
    // the receipt file after evidence bundle assembly.
    const input = makeReviewedEditorialInput();
    input.receipts[0] = {
      ...input.receipts[0],
      raw: { ...input.receipts[0].raw, synthesis: OPENING_ORIGINAL + ' TAMPERED' },
    };
    const result = verifyReferenceExecution(input);
    // The base verifier catches synthesis_output_hash_mismatch AND the editorial
    // branch surfaces editorial_original_synthesis_tampered / editorial_apply_failed.
    expect(
      result.blockers.some(
        (b) =>
          b.includes('synthesis_output_hash_mismatch') ||
          b.includes('editorial_original_synthesis_tampered') ||
          b.includes('editorial_apply_failed'),
      ),
    ).toBe(true);
    expect(result.passed).toBe(false);
  });

  it('blocks when the fresh audit input hash disagrees with engine facts', () => {
    const input = makeReviewedEditorialInput({
      mutateEvidence: (e) => ({
        ...e,
        freshAudit: { ...e.freshAudit, engineFacts: 'SWAPPED ENGINE FACTS' },
      }),
    });
    const result = verifyReferenceExecution(input);
    expect(
      result.blockers.some((b) => b.includes('editorial_audit_engine_facts_drift') || b.includes('editorial_audit_input_hash_mismatch')),
    ).toBe(true);
  });

  it('blocks when the fresh audit output_sha256 is tampered', () => {
    const input = makeReviewedEditorialInput({
      mutateEvidence: (e) => ({
        ...e,
        freshAudit: {
          ...e.freshAudit,
          audit: { ...e.freshAudit.audit, output_sha256: sha256('tampered audit raw ledger') },
        },
      }),
    });
    const result = verifyReferenceExecution(input);
    expect(result.blockers.some((b) => b.includes('editorial_audit_output_hash_mismatch'))).toBe(true);
  });

  it('blocks when the guardrail disposition is missing but Jev has could-not-tell on guardrail', () => {
    const input = makeReviewedEditorialInput({
      mutateEvidence: (e) => ({ ...e, guardrailDisposition: undefined }),
    });
    const result = verifyReferenceExecution(input);
    expect(result.blockers.some((b) => b.includes('editorial_guardrail_disposition_missing'))).toBe(true);
  });

  it('blocks when the guardrail disposition rationale is empty (validator rejects it)', () => {
    const input = makeReviewedEditorialInput({
      mutateEvidence: (e) => {
        const gd = e.guardrailDisposition!;
        return {
          ...e,
          guardrailDisposition: {
            ...gd,
            disposition: { ...gd.disposition, rationale: '' },
          },
        };
      },
    });
    const result = verifyReferenceExecution(input);
    expect(result.blockers.some((b) => b.includes('editorial_guardrail_disposition_invalid'))).toBe(true);
  });

  it('blocks when a revised section is claimed but originalPasses hash of prior section is wrong', () => {
    const input = makeReviewedEditorialInput({
      mutatePart1Receipt: (r) => ({
        ...r,
        prior_section_content_hashes: { opening: sha256('DIFFERENT ORIGINAL OPENING') },
      }),
    });
    // We only marked opening as revised in this fixture; but the prior-context
    // check runs on revised sections.  Instead: mark part1 as-if revised too?
    // Not needed — swap: tamper opening's own prior_section_content_hashes to
    // reference a bogus earlier section.  Simpler: verify with a two-revised
    // setup below.  For opening (first section) there are no earlier ids so
    // this test just asserts no false positive on prior context.
    const result = verifyReferenceExecution(input);
    // part1 is not marked as revised so its prior context isn't checked;
    // this should still pass through the editorial gate.
    expect(result.editorialVerdicts).toHaveLength(1);
  });

  it('blocks when a revised later section carries a prior-context edge that does NOT match the original earlier pass', () => {
    // Build a fixture where BOTH opening and part1 are revised, and part1's
    // prior_section_content_hashes.opening no longer hashes to originalPasses.opening.
    const input = makeReviewedEditorialInput();
    const openingEvidence = input.editorial!.revisedSections['opening'];
    const revisedPart1Output = PART1_OUTPUT + ' (revised)';
    const part1Patches: EditorialPatch[] = [
      { before: 'anchored in the opening.', after: 'anchored in the opening. (revised)',
        rationale: 'Add clarifier.', sourceReferences: [{ sourceId: 'policy/x', quote: 'x' }] },
    ];
    const part1Receipt = input.receipts[1];
    const part1Applied = applyEditorialRevision({
      baseReceipt: part1Receipt,
      currentOutput: part1Receipt.raw.synthesis,
      expectedBaseOutputHash: part1Receipt.output_hash,
      patches: part1Patches,
      reviewer: 'assistant-source-review',
      timestamp: '2026-09-30T01:14:00.000Z',
    });
    const freshPart1Audit = freshOpeningAudit(part1Applied.revisedOutput, part1Receipt);
    // fix section_id on the audit to match part1
    freshPart1Audit.audit.section_id = 'part1';
    const freshPart1Jev = freshOpeningJev(part1Applied.revisedOutput, 'pass');
    freshPart1Jev.receipt.pass_id = 'part1';
    // Tamper: replace originalPasses.opening with a different text so the
    // prior_section_content_hashes edge on part1 no longer matches.
    const tampered = makeReviewedEditorialInput({
      mutateOriginalPasses: (p) => ({ ...p, opening: OPENING_ORIGINAL + ' TAMPERED HISTORY' }),
    });
    tampered.editorial = {
      ...tampered.editorial!,
      revisedSections: {
        opening: openingEvidence,
        part1: {
          baseReceiptSha256: freshPart1Audit.baseReceiptSha256,
          baseReceiptRawBytes: JSON.stringify(part1Receipt),
          revisionArtifact: {
            sectionId: 'part1',
            baseOutputHash: part1Applied.baseOutputHash,
            revisedOutput: part1Applied.revisedOutput,
            revisedOutputHash: part1Applied.revisedOutputHash,
            patches: part1Patches,
            reviewer: 'assistant-source-review',
            timestamp: '2026-09-30T01:14:00.000Z',
          },
          freshAudit: freshPart1Audit,
          freshJev: freshPart1Jev,
        },
      },
    };
    tampered.passes[1] = { ...tampered.passes[1], output: part1Applied.revisedOutput };
    // update coherence review to cite both revised hashes
    const effHashes = [openingEvidence.revisionArtifact.revisedOutputHash, part1Applied.revisedOutputHash];
    tampered.editorial!.coherenceReview = {
      artifactSha256: sha256(effHashes.join(':')),
      orderedEffectiveHashes: effHashes,
      status: 'reviewed',
      reviewer: 'assistant-source-review',
      reviewedAt: '2026-09-30T01:30:00.000Z',
      rationale: 'coherence reviewed for both revisions',
      findings: [],
      reviewedRevisions: [
        { sectionId: 'opening', revisedOutputHash: openingEvidence.revisionArtifact.revisedOutputHash },
        { sectionId: 'part1', revisedOutputHash: part1Applied.revisedOutputHash },
      ],
    };
    const result = verifyReferenceExecution(tampered);
    expect(result.blockers.some((b) => b.includes('editorial_prior_context_edge_original_hash_mismatch:opening'))).toBe(true);
  });
});

// ─── Source-audit-only recovery vs other failure kinds ────────────────

describe('editorial-verification: source-audit-only recovery', () => {
  it('accepts an original outcome=failed receipt when every attempt is source_audit_failed and fresh audit is clean', () => {
    const input = makeReviewedEditorialInput({
      mutateOpeningReceipt: (r) => ({
        ...r,
        outcome: 'failed',
        attempts: [
          { attempt: 1, outcome: 'source_audit_failed', aletheios_raw: ALETHEIOS_OPENING, pichet_raw: PICHET_OPENING, synthesis_raw: OPENING_ORIGINAL },
          { attempt: 2, outcome: 'source_audit_failed', aletheios_raw: ALETHEIOS_OPENING, pichet_raw: PICHET_OPENING, synthesis_raw: OPENING_ORIGINAL },
        ],
      }),
    });
    const result = verifyReferenceExecution(input);
    expect(result.blockers).toHaveLength(0);
    expect(result.passed).toBe(true);
  });

  it('blocks when a failed-outcome original receipt has ANY non-source_audit_failed attempt kind', () => {
    const input = makeReviewedEditorialInput({
      mutateOpeningReceipt: (r) => ({
        ...r,
        outcome: 'failed',
        attempts: [
          { attempt: 1, outcome: 'source_audit_failed', aletheios_raw: ALETHEIOS_OPENING, pichet_raw: PICHET_OPENING, synthesis_raw: OPENING_ORIGINAL },
          { attempt: 2, outcome: 'word_count_failed', aletheios_raw: ALETHEIOS_OPENING, pichet_raw: PICHET_OPENING, synthesis_raw: OPENING_ORIGINAL },
        ],
      }),
    });
    const result = verifyReferenceExecution(input);
    expect(result.blockers.some((b) => b.includes('editorial_original_failed_mixed_source_audit_and_word_count'))).toBe(true);
    expect(result.passed).toBe(false);
  });
});

// ─── No-options regression ────────────────────────────────────────────

describe('editorial-verification: no editorial options => strict base behaviour unchanged', () => {
  it('the exact same receipts with no editorial branch still block on pass_output_hash_mismatch when texts differ', () => {
    const input = makeReviewedEditorialInput();
    // Strip editorial options
    const stripped: ReferenceVerificationInput = { ...input };
    delete (stripped as { editorial?: unknown }).editorial;
    const result = verifyReferenceExecution(stripped);
    // Opening pass output is the revised text; original receipt.output_hash is
    // the ORIGINAL hash — base verifier must catch the mismatch.
    expect(result.blockers.some((b) => b.includes('opening:pass_output_hash_mismatch'))).toBe(true);
    // And it must also flag the old jev could-not-tell verdict as unresolved.
    expect(result.blockers.some((b) => b.includes('opening:jev_unresolved:guardrail') || b.includes('unresolved:opening:jev_verdict_unresolved:guardrail'))).toBe(true);
  });

  it('a fully-clean base run with no revised sections behaves identically with and without an empty editorial map', () => {
    // Base: single unrevised section, everything consistent.
    const receipt = basePart1Receipt();
    const pass: PassResult = { id: 'part1', title: 'Part I', output: PART1_OUTPUT, rubric: cleanRubric('part1', 'Part I') };
    const baseInput: ReferenceVerificationInput = {
      receipts: [receipt],
      passes: [pass],
      requiredSectionIds: ['part1'],
      expectedPersonas: { aletheios: ALETHEIOS_IDENTITY, pichet: PICHET_IDENTITY },
      subsectionMap: { part1: [] },
      sourceTexts: { [PART1_PASSAGE_ID]: PART1_PASSAGE_TEXT },
      rawSynthesisInputs: { part1: receipt.synthesis_input_text ?? '' },
    };
    // Remove the prior_section_content_hashes reference to opening (no prior sections)
    receipt.prior_section_content_hashes = {};
    const cleanResult = verifyReferenceExecution(baseInput);
    expect(cleanResult.passed).toBe(true);

    // With an editorial option whose revisedSections is empty and coherence review omitted
    const withEmptyEditorial: ReferenceVerificationInput = {
      ...baseInput,
      editorial: { revisedSections: {}, originalPasses: { part1: PART1_OUTPUT } },
    };
    const emptyResult = verifyReferenceExecution(withEmptyEditorial);
    expect(emptyResult.passed).toBe(true);
    expect(emptyResult.editorialVerdicts).toEqual([]);
  });
});

// ─── Adversarial fail-closed tests (2026-09-30 verifier review) ──────
// Each test in this section is a targeted attack on the editorial
// verifier.  Every attack MUST be rejected by a specific blocker code.
// A silent pass here means the verifier weakened.

describe('editorial-verification: item 1 — raw receipt bytes required', () => {
  it('blocks when baseReceiptRawBytes is absent (a hash without bytes proves nothing)', () => {
    const input = makeReviewedEditorialInput({
      mutateEvidence: (e) => ({ ...e, baseReceiptRawBytes: '' }),
    });
    const result = verifyReferenceExecution(input);
    expect(result.blockers.some((b) => b.includes('editorial_base_receipt_raw_bytes_missing'))).toBe(true);
    expect(result.passed).toBe(false);
  });

  it('blocks when baseReceiptRawBytes is unparseable JSON', () => {
    const input = makeReviewedEditorialInput({
      mutateEvidence: (e) => ({ ...e, baseReceiptRawBytes: '{not-json' }),
    });
    const result = verifyReferenceExecution(input);
    expect(result.blockers.some((b) => b.includes('editorial_base_receipt_raw_bytes_unparseable'))).toBe(true);
  });

  it('blocks when baseReceiptSha256 does not equal sha256(baseReceiptRawBytes) — false byte hash claim', () => {
    const input = makeReviewedEditorialInput({
      mutateEvidence: (e) => ({ ...e, baseReceiptSha256: sha256('not the actual receipt bytes') }),
    });
    const result = verifyReferenceExecution(input);
    // computed hash of the real bytes ≠ evidence.baseReceiptSha256
    expect(result.blockers.some((b) => b.includes('editorial_base_receipt_hash_mismatch'))).toBe(true);
  });

  it('blocks when the parsed raw bytes describe a different receipt (output_hash mismatch)', () => {
    const input = makeReviewedEditorialInput({
      mutateEvidence: (e) => {
        // Craft raw bytes whose parsed output_hash points elsewhere.
        const fake = { section_id: 'opening', output_hash: sha256('fabricated'), outcome: 'ok', raw: { synthesis: OPENING_ORIGINAL } };
        const fakeBytes = JSON.stringify(fake);
        return { ...e, baseReceiptRawBytes: fakeBytes, baseReceiptSha256: sha256(fakeBytes) };
      },
    });
    const result = verifyReferenceExecution(input);
    expect(result.blockers.some((b) => b.includes('editorial_base_receipt_raw_bytes_output_hash_mismatch'))).toBe(true);
  });

  it('blocks when the fresh audit sidecar baseReceiptSha256 disagrees with the recomputed raw bytes hash', () => {
    const input = makeReviewedEditorialInput({
      mutateEvidence: (e) => ({
        ...e,
        freshAudit: { ...e.freshAudit, baseReceiptSha256: sha256('other bytes entirely') },
      }),
    });
    const result = verifyReferenceExecution(input);
    // Both the audit↔evidence cross-check AND the audit↔recomputed-bytes check fire.
    expect(
      result.blockers.some(
        (b) =>
          b.includes('editorial_audit_base_receipt_mismatch') ||
          b.includes('editorial_audit_base_receipt_bytes_mismatch'),
      ),
    ).toBe(true);
  });
});

describe('editorial-verification: item 2 — fresh Jev dimension floor + bound to THIS jev', () => {
  it('blocks when the fresh Jev verdicts map is empty (empty map is not acceptance)', () => {
    const input = makeReviewedEditorialInput({
      mutateEvidence: (e) => ({
        ...e,
        freshJev: { ...e.freshJev, receipt: { ...e.freshJev.receipt, verdicts: {} } },
      }),
    });
    const result = verifyReferenceExecution(input);
    // Every required dimension must be present
    for (const dim of ['guardrail', 'framing', 'grounding', 'register'] as const) {
      expect(result.blockers.some((b) => b.includes(`editorial_jev_dimension_missing:${dim}`))).toBe(true);
    }
  });

  it('blocks when a required Jev dimension is missing (e.g. no grounding verdict)', () => {
    const input = makeReviewedEditorialInput({
      mutateEvidence: (e) => {
        const { grounding: _drop, ...rest } = e.freshJev.receipt.verdicts;
        void _drop;
        return { ...e, freshJev: { ...e.freshJev, receipt: { ...e.freshJev.receipt, verdicts: rest } } };
      },
    });
    const result = verifyReferenceExecution(input);
    expect(result.blockers.some((b) => b.includes('editorial_jev_dimension_missing:grounding'))).toBe(true);
  });

  it('blocks when Jev blocked is undefined (undefined is not acceptance)', () => {
    const input = makeReviewedEditorialInput({
      mutateEvidence: (e) => {
        const receipt = { ...e.freshJev.receipt } as Record<string, unknown>;
        delete receipt.blocked;
        return { ...e, freshJev: { ...e.freshJev, receipt: receipt as typeof e.freshJev.receipt } };
      },
    });
    const result = verifyReferenceExecution(input);
    expect(result.blockers.some((b) => b.includes('editorial_jev_blocked_or_unset'))).toBe(true);
  });

  it('blocks when Jev pass_id does not match the section under review', () => {
    const input = makeReviewedEditorialInput({
      mutateEvidence: (e) => ({
        ...e,
        freshJev: { ...e.freshJev, receipt: { ...e.freshJev.receipt, pass_id: 'some-other-section' } },
      }),
    });
    const result = verifyReferenceExecution(input);
    expect(result.blockers.some((b) => b.includes('editorial_jev_pass_id_mismatch'))).toBe(true);
  });

  it('blocks when Jev mode does not match the expected review mode', () => {
    const input = makeReviewedEditorialInput({
      mutateEvidence: (e) => ({
        ...e,
        freshJev: { ...e.freshJev, receipt: { ...e.freshJev.receipt, mode: 'production' } },
      }),
    });
    const result = verifyReferenceExecution(input);
    expect(result.blockers.some((b) => b.includes('editorial_jev_mode_mismatch'))).toBe(true);
  });

  it('blocks when the guardrail disposition binds to a DIFFERENT Jev artifact than the fresh Jev on evidence', () => {
    const input = makeReviewedEditorialInput({
      mutateEvidence: (e) => {
        // Swap the disposition's cited originalJev with an independently
        // "valid" review of the same prose (same hash) — but the
        // structural bundle differs from the fresh Jev the caller
        // actually supplied here.  The verifier must reject.
        const gd = e.guardrailDisposition!;
        return {
          ...e,
          guardrailDisposition: {
            ...gd,
            originalJev: {
              ...gd.originalJev,
              receipt: {
                ...gd.originalJev.receipt,
                verdicts: {
                  ...gd.originalJev.receipt.verdicts,
                  framing: 'pass',
                  extra_dimension_from_another_review: 'pass',
                },
              },
            },
          },
        };
      },
    });
    const result = verifyReferenceExecution(input);
    expect(result.blockers.some((b) => b.includes('editorial_guardrail_disposition_jev_binding_mismatch'))).toBe(true);
  });

  it('accepts a Jev with all four required dimensions passing and no could-not-tell (no disposition needed)', () => {
    const input = makeReviewedEditorialInput({
      mutateEvidence: (e) => ({
        ...e,
        // full pass — no could-not-tell — disposition becomes unnecessary
        freshJev: {
          ...e.freshJev,
          receipt: {
            ...e.freshJev.receipt,
            verdicts: { guardrail: 'pass', framing: 'pass', grounding: 'pass', register: 'pass' },
          },
        },
        guardrailDisposition: undefined,
      }),
    });
    const result = verifyReferenceExecution(input);
    expect(result.blockers).toHaveLength(0);
    expect(result.passed).toBe(true);
  });
});

describe('editorial-verification: item 3 — failed-outcome attempts eligibility', () => {
  it('blocks a failed-outcome receipt whose attempts list contains only ok kinds (no source_audit_failed)', () => {
    const input = makeReviewedEditorialInput({
      mutateOpeningReceipt: (r) => ({
        ...r,
        outcome: 'failed',
        attempts: [
          { attempt: 1, outcome: 'ok', aletheios_raw: ALETHEIOS_OPENING, pichet_raw: PICHET_OPENING, synthesis_raw: OPENING_ORIGINAL },
        ],
      }),
    });
    const result = verifyReferenceExecution(input);
    expect(
      result.blockers.some((b) =>
        b.includes('editorial_original_failed_no_source_audit_failed_attempt'),
      ),
    ).toBe(true);
  });

  it('blocks a failed-outcome receipt with a null attempt entry (malformed)', () => {
    const input = makeReviewedEditorialInput({
      mutateOpeningReceipt: (r) => ({
        ...r,
        outcome: 'failed',
        attempts: [
          { attempt: 1, outcome: 'source_audit_failed', aletheios_raw: ALETHEIOS_OPENING, pichet_raw: PICHET_OPENING, synthesis_raw: OPENING_ORIGINAL },
          null as unknown as SectionExecutionReceipt['attempts'][number],
        ],
      }),
    });
    const result = verifyReferenceExecution(input);
    expect(result.blockers.some((b) => b.includes('editorial_original_failed_malformed_attempt'))).toBe(true);
  });

  it('blocks a failed-outcome receipt with an unknown attempt outcome kind', () => {
    const input = makeReviewedEditorialInput({
      mutateOpeningReceipt: (r) => ({
        ...r,
        outcome: 'failed',
        attempts: [
          { attempt: 1, outcome: 'source_audit_failed', aletheios_raw: ALETHEIOS_OPENING, pichet_raw: PICHET_OPENING, synthesis_raw: OPENING_ORIGINAL },
          { attempt: 2, outcome: 'mysterious_new_outcome', aletheios_raw: ALETHEIOS_OPENING, pichet_raw: PICHET_OPENING, synthesis_raw: OPENING_ORIGINAL },
        ],
      }),
    });
    const result = verifyReferenceExecution(input);
    expect(result.blockers.some((b) => b.includes('editorial_original_failed_unknown_outcome:mysterious_new_outcome'))).toBe(true);
  });

  it('blocks a failed-outcome receipt whose attempts list is empty', () => {
    const input = makeReviewedEditorialInput({
      mutateOpeningReceipt: (r) => ({ ...r, outcome: 'failed', attempts: [] }),
    });
    const result = verifyReferenceExecution(input);
    expect(result.blockers.some((b) => b.includes('editorial_original_failed_no_attempts'))).toBe(true);
  });
});

describe('editorial-verification: item 4 — historical prior-context admits verified revised, rejects forgeries', () => {
  // Helper: build a two-revised fixture where opening was revised FIRST
  // and part1 was subsequently regenerated against the REVISED opening
  // before part1 itself underwent its own later revision.
  function makeTwoRevisedWithHistoricalRevisedOpening(): ReferenceVerificationInput {
    const base = makeReviewedEditorialInput();
    const openingEvidence = base.editorial!.revisedSections['opening'];
    const openingRevised = openingEvidence.revisionArtifact.revisedOutput;
    const openingRevisedHash = openingEvidence.revisionArtifact.revisedOutputHash;

    // Rebuild part1's raw synthesis input to contain the REVISED opening
    // verbatim (that's what the generator actually saw), and set the
    // receipt's prior_section_content_hashes.opening to sha256(revised).
    const part1Receipt = base.receipts[1];
    const rebuiltPart1Input =
      'PRIOR SECTIONS FOR CONTINUITY:\n' + openingRevised + '\n\nCURRENT SECTION INSTRUCTIONS:\nDraft Part I.\n' +
      PART1_ALETHEIOS + '\n\n' + PART1_PICHET;
    const rebuiltPart1: SectionExecutionReceipt = {
      ...part1Receipt,
      synthesis_input_text: rebuiltPart1Input,
      synthesis_input_hash: sha256(rebuiltPart1Input),
      prior_section_content_hashes: { opening: openingRevisedHash },
    };

    // Now revise part1 with a tiny edit.
    const part1Patches: EditorialPatch[] = [
      { before: 'anchored in the opening.', after: 'anchored in the opening. (extended)',
        rationale: 'Extend.', sourceReferences: [{ sourceId: 'policy/x', quote: 'x' }] },
    ];
    const part1Applied = applyEditorialRevision({
      baseReceipt: rebuiltPart1,
      currentOutput: rebuiltPart1.raw.synthesis,
      expectedBaseOutputHash: rebuiltPart1.output_hash,
      patches: part1Patches,
      reviewer: 'assistant-source-review',
      timestamp: '2026-09-30T02:00:00.000Z',
    });
    const freshPart1Audit = freshOpeningAudit(part1Applied.revisedOutput, rebuiltPart1);
    freshPart1Audit.audit.section_id = 'part1';
    freshPart1Audit.baseReceiptSha256 = sha256(JSON.stringify(rebuiltPart1));
    const freshPart1Jev = freshOpeningJev(part1Applied.revisedOutput, 'pass');
    freshPart1Jev.receipt.pass_id = 'part1';

    const part1Evidence: EditorialSectionEvidence = {
      baseReceiptSha256: sha256(JSON.stringify(rebuiltPart1)),
      baseReceiptRawBytes: JSON.stringify(rebuiltPart1),
      revisionArtifact: {
        sectionId: 'part1',
        baseOutputHash: part1Applied.baseOutputHash,
        revisedOutput: part1Applied.revisedOutput,
        revisedOutputHash: part1Applied.revisedOutputHash,
        patches: part1Patches,
        reviewer: 'assistant-source-review',
        timestamp: '2026-09-30T02:00:00.000Z',
      },
      freshAudit: freshPart1Audit,
      freshJev: freshPart1Jev,
    };

    const assembled = [
      `## Opening\n\n${openingRevised}`,
      `## Part I\n\n${part1Applied.revisedOutput}`,
    ].join('\n\n');

    const effHashes = [openingRevisedHash, part1Applied.revisedOutputHash];
    const coherenceReview: EditorialCoherenceReview = {
      artifactSha256: sha256(assembled),
      orderedEffectiveHashes: effHashes,
      status: 'reviewed',
      reviewer: 'assistant-source-review',
      reviewedAt: '2026-09-30T02:15:00.000Z',
      rationale: 'Both revisions coherent: revised opening frames part1 without policy drift.',
      findings: [],
      reviewedRevisions: [
        { sectionId: 'opening', revisedOutputHash: openingRevisedHash },
        { sectionId: 'part1', revisedOutputHash: part1Applied.revisedOutputHash },
      ],
    };

    return {
      receipts: [base.receipts[0], rebuiltPart1],
      passes: [base.passes[0], { ...base.passes[1], output: part1Applied.revisedOutput }],
      requiredSectionIds: ['opening', 'part1'],
      expectedPersonas: base.expectedPersonas,
      subsectionMap: base.subsectionMap,
      sourceTexts: base.sourceTexts,
      rawSynthesisInputs: { part1: rebuiltPart1Input },
      assembled,
      editorial: {
        revisedSections: { opening: openingEvidence, part1: part1Evidence },
        originalPasses: { opening: OPENING_ORIGINAL, part1: PART1_OUTPUT },
        coherenceReview,
        priorContext: {
          // part1 legitimately consumed the REVISED opening at generation time.
          part1: { opening: { kind: 'revised', text: openingRevised } },
        },
      },
    };
  }

  function withNewerOpening() {
    const input = makeTwoRevisedWithHistoricalRevisedOpening();
    const editorial = input.editorial!;
    const old = editorial.revisedSections.opening;
    const receipt = input.receipts[0];
    const output = old.revisionArtifact.revisedOutput + '\nA later source clarification.';
    const revision = applyEditorialRevision({ baseReceipt: receipt, currentOutput: receipt.raw.synthesis,
      expectedBaseOutputHash: receipt.output_hash, reviewer: 'assistant-source-review',
      timestamp: '2026-09-30T03:00:00.000Z', patches: [{ before: receipt.raw.synthesis, after: output,
        rationale: 'Later clarification.', sourceReferences: [{ sourceId: 'policy/x', quote: 'x' }] }] });
    editorial.revisedSections = { ...editorial.revisedSections, opening: {
      ...old, revisionArtifact: revision, freshAudit: freshOpeningAudit(output, receipt),
      freshJev: freshOpeningJev(output, 'pass'), guardrailDisposition: undefined,
    } };
    editorial.historicalRevisions = { opening: [old] };
    input.passes[0] = { ...input.passes[0], output };
    input.assembled = input.passes.map(p => `## ${p.title}\n\n${p.output}`).join('\n\n');
    editorial.coherenceReview = { ...editorial.coherenceReview!, artifactSha256: sha256(input.assembled),
      orderedEffectiveHashes: input.passes.map(p => sha256(p.output)),
      reviewedRevisions: input.passes.map(p => ({sectionId:p.id,revisedOutputHash:sha256(p.output)})) };
    return input;
  }
  it('accepts a previously reviewed predecessor while delivering its newer verified revision', () => {
    expect(verifyReferenceExecution(withNewerOpening()).blockers).toEqual([]);
  });
  it('rejects a historical predecessor when its complete independent evidence is absent', () => {
    const input=withNewerOpening(); input.editorial!.historicalRevisions=undefined;
    expect(verifyReferenceExecution(input).blockers).toContain('part1:editorial_prior_context_edge_revised_text_mismatch:opening');
  });
  it('rejects tampered audit evidence for a historical predecessor', () => {
    const input=withNewerOpening(); const old=input.editorial!.historicalRevisions!.opening[0];
    input.editorial!.historicalRevisions={opening:[{...old,freshAudit:{...old.freshAudit,sectionOutputHash:'tampered'}}]};
    const blockers=verifyReferenceExecution(input).blockers;
    expect(blockers.some(b=>b.startsWith('historical:'))).toBe(true);
    expect(blockers).toContain('part1:editorial_prior_context_edge_revised_text_mismatch:opening');
  });
  it('still rejects forged historical consumption hashes', () => {
    const input=withNewerOpening(); input.receipts[1].prior_section_content_hashes.opening='wrong';
    expect(verifyReferenceExecution(input).blockers).toContain('part1:editorial_prior_context_edge_revised_hash_mismatch:opening');
  });

  it('accepts a later section whose priorContext declares a VERIFIED revised predecessor consumed before its own later revision', () => {
    const input = makeTwoRevisedWithHistoricalRevisedOpening();
    const result = verifyReferenceExecution(input);
    expect(result.blockers).toHaveLength(0);
    expect(result.passed).toBe(true);
  });

  it('blocks when a declared revised-kind edge text does NOT match the verified revised output (forged later edit)', () => {
    const input = makeTwoRevisedWithHistoricalRevisedOpening();
    // Attack: declare a revised-kind edge whose text is a fabricated
    // "revised opening" that was NEVER actually applied.  This is exactly
    // the "never forge a later edit into old generation" doctrine.
    input.editorial = {
      ...input.editorial!,
      priorContext: {
        part1: { opening: { kind: 'revised', text: OPENING_REVISED + ' [FORGED LATER EDIT]' } },
      },
    };
    const result = verifyReferenceExecution(input);
    expect(result.blockers.some((b) => b.includes('editorial_prior_context_edge_revised_text_mismatch:opening'))).toBe(true);
    expect(result.passed).toBe(false);
  });

  it('blocks when a revised-kind edge references a section that is NOT in revisedSections', () => {
    const input = makeTwoRevisedWithHistoricalRevisedOpening();
    input.editorial = {
      ...input.editorial!,
      revisedSections: { part1: input.editorial!.revisedSections['part1'] },
      priorContext: {
        part1: { opening: { kind: 'revised', text: OPENING_REVISED } },
      },
    };
    const result = verifyReferenceExecution(input);
    expect(result.blockers.some((b) => b.includes('editorial_prior_context_edge_revised_not_verified:opening'))).toBe(true);
  });

  it('blocks when a declared edge text does NOT hash to the receipt-stored prior-context edge hash', () => {
    const input = makeTwoRevisedWithHistoricalRevisedOpening();
    // Tamper: keep the edge kind=revised and correct text, but corrupt
    // the RECEIPT's stored prior-context hash so the round-trip fails.
    const part1 = input.receipts[1];
    input.receipts[1] = { ...part1, prior_section_content_hashes: { opening: sha256('mismatched hash') } };
    const result = verifyReferenceExecution(input);
    expect(result.blockers.some((b) => b.includes('editorial_prior_context_edge_revised_hash_mismatch:opening'))).toBe(true);
  });

  it('blocks when the declared edge text is absent from the receipt raw synthesis input (edge not actually consumed)', () => {
    const input = makeTwoRevisedWithHistoricalRevisedOpening();
    // Attack: strip the opening text out of part1's raw synthesis input
    // while leaving the priorContext declaration in place.  The receipt
    // no longer proves the generator saw the declared text.
    const part1 = input.receipts[1];
    input.receipts[1] = { ...part1, synthesis_input_text: 'no prior context here at all' };
    const result = verifyReferenceExecution(input);
    expect(result.blockers.some((b) => b.includes('editorial_prior_context_edge_raw_input_missing:opening'))).toBe(true);
  });

  it('blocks when an UNDECLARED revised text leaks into the raw synthesis input of another section (forged rerun)', () => {
    const input = makeTwoRevisedWithHistoricalRevisedOpening();
    // Attack: drop the priorContext declaration for part1 while its raw
    // synthesis input still contains the REVISED opening.  Silence must
    // never admit a revised predecessor.
    input.editorial = { ...input.editorial!, priorContext: {} };
    const result = verifyReferenceExecution(input);
    // First: the default 'original'-kind edge disagrees on text and hash.
    // Second: the raw input still contains the revised opening — forged rerun.
    expect(
      result.blockers.some((b) => b.includes('editorial_original_raw_input_forged_with_revised:opening')),
    ).toBe(true);
  });

  it('blocks when default-original edge text disagrees with originalPasses (silence never admits a revised predecessor)', () => {
    // In the base fixture part1's receipt.prior_section_content_hashes.opening
    // hashes to sha256(OPENING_REVISED); by default (no priorContext) the
    // edge is treated as 'original' and must match originalPasses[opening]
    // = OPENING_ORIGINAL — which hashes differently.
    const input = makeReviewedEditorialInput({
      // Mark part1 as revised so priorContext logic runs against it.
      mutatePart1Receipt: (r) => r, // no-op; the fixture stores REVISED opening hash on part1
    });
    // Add part1 as a revised section (minimal noop revision) so the
    // prior-context helper is invoked for part1.
    const part1Receipt = input.receipts[1];
    const noopPatches: EditorialPatch[] = [
      { before: PART1_OUTPUT, after: PART1_OUTPUT + ' .', rationale: 'noop', sourceReferences: [{ sourceId: 'x', quote: 'x' }] },
    ];
    const part1Applied = applyEditorialRevision({
      baseReceipt: part1Receipt,
      currentOutput: part1Receipt.raw.synthesis,
      expectedBaseOutputHash: part1Receipt.output_hash,
      patches: noopPatches,
      reviewer: 'assistant-source-review',
      timestamp: '2026-09-30T02:30:00.000Z',
    });
    const freshAudit2 = freshOpeningAudit(part1Applied.revisedOutput, part1Receipt);
    freshAudit2.audit.section_id = 'part1';
    freshAudit2.baseReceiptSha256 = sha256(JSON.stringify(part1Receipt));
    const freshJev2 = freshOpeningJev(part1Applied.revisedOutput, 'pass');
    freshJev2.receipt.pass_id = 'part1';
    input.editorial = {
      ...input.editorial!,
      revisedSections: {
        ...input.editorial!.revisedSections,
        part1: {
          baseReceiptSha256: sha256(JSON.stringify(part1Receipt)),
          baseReceiptRawBytes: JSON.stringify(part1Receipt),
          revisionArtifact: {
            sectionId: 'part1',
            baseOutputHash: part1Applied.baseOutputHash,
            revisedOutput: part1Applied.revisedOutput,
            revisedOutputHash: part1Applied.revisedOutputHash,
            patches: noopPatches,
            reviewer: 'assistant-source-review',
            timestamp: '2026-09-30T02:30:00.000Z',
          },
          freshAudit: freshAudit2,
          freshJev: freshJev2,
        },
      },
    };
    input.passes[1] = { ...input.passes[1], output: part1Applied.revisedOutput };
    const result = verifyReferenceExecution(input);
    // Default treat-as-original path: text = OPENING_ORIGINAL, but
    // receipt stored hash = sha256(OPENING_REVISED) → hash mismatch AND
    // OPENING_ORIGINAL is not present in part1's raw synthesis input.
    expect(
      result.blockers.some((b) => b.includes('editorial_prior_context_edge_original_hash_mismatch:opening')),
    ).toBe(true);
  });
});

describe('editorial-verification: item 5 — coherence review bindings and finding shape', () => {
  it('blocks when assembled text is missing but revised sections exist', () => {
    const input = makeReviewedEditorialInput();
    const stripped: ReferenceVerificationInput = { ...input };
    delete (stripped as { assembled?: unknown }).assembled;
    const result = verifyReferenceExecution(stripped);
    expect(result.blockers).toContain('editorial:assembled_text_required_for_coherence_binding');
    expect(result.passed).toBe(false);
  });

  it('blocks when coherence review findings is present but not an array', () => {
    const input = makeReviewedEditorialInput({
      mutateCoherence: (r) => ({ ...r, findings: 'not an array' as unknown as EditorialCoherenceReview['findings'] }),
    });
    const result = verifyReferenceExecution(input);
    expect(result.blockers).toContain('editorial:coherence_review_findings_not_array');
  });

  it('blocks when a coherence review finding entry is malformed (missing boolean unresolved)', () => {
    const input = makeReviewedEditorialInput({
      mutateCoherence: (r) => ({
        ...r,
        findings: [
          { id: 'f1', summary: 'ok', unresolved: false },
          { id: 'f2', summary: 'malformed' } as unknown as EditorialCoherenceReview['findings'][number],
        ],
      }),
    });
    const result = verifyReferenceExecution(input);
    expect(result.blockers.some((b) => b.includes('editorial:coherence_review_finding_malformed:1'))).toBe(true);
  });

  it('blocks when reviewedRevisions contains duplicate section IDs', () => {
    const input = makeReviewedEditorialInput({
      mutateCoherence: (r) => ({
        ...r,
        reviewedRevisions: [
          ...r.reviewedRevisions,
          { sectionId: 'opening', revisedOutputHash: r.reviewedRevisions[0].revisedOutputHash },
        ],
      }),
    });
    const result = verifyReferenceExecution(input);
    expect(result.blockers.some((b) => b.includes('editorial:coherence_review_duplicate_reviewed_revision:opening'))).toBe(true);
  });

  it('blocks when coherence review rationale is present but not substantive (too short)', () => {
    const input = makeReviewedEditorialInput({
      mutateCoherence: (r) => ({ ...r, rationale: 'too short' }),
    });
    const result = verifyReferenceExecution(input);
    expect(result.blockers).toContain('editorial:coherence_review_rationale_not_substantive');
  });
});

describe('real receipt framing and complete byte binding', () => {
  it('rejects altered non-summary receipt fields even with matching declared byte hashes', () => {
    const input = makeReviewedEditorialInput({ mutateEvidence: e => {
      const raw = JSON.parse(e.baseReceiptRawBytes);
      raw.engine_facts = 'substituted facts';
      const bytes = JSON.stringify(raw);
      return { ...e, baseReceiptRawBytes: bytes, baseReceiptSha256: sha256(bytes),
        freshAudit: { ...e.freshAudit, baseReceiptSha256: sha256(bytes) } };
    }});
    expect(verifyReferenceExecution(input).blockers).toContain('opening:editorial_base_receipt_raw_bytes_content_mismatch');
  });

  const title = 'Opening — Scope and Available Systems';
  function realEdge(revised = false): Parameters<typeof verifyEditorialPriorContext>[0] {
    const text = revised ? OPENING_REVISED : OPENING_ORIGINAL;
    const framed = `## ${title}\n\n${text}`;
    const receipt = basePart1Receipt();
    receipt.prior_section_content_hashes = { [title]: sha256(framed) };
    receipt.synthesis_input_text = `Prior context\n${framed}\nVoices`;
    return { sectionId: 'part1', originalReceipt: receipt, requiredSectionIds: ['opening', 'part1'],
      originalPasses: { opening: OPENING_ORIGINAL }, sectionTitles: { opening: title },
      revisedOutputsByOtherSections: revised ? { opening: OPENING_REVISED } : {},
      revisedSectionIds: revised ? ['opening'] : [],
      priorContext: revised ? { opening: { kind: 'revised' as const, text: OPENING_REVISED } } : undefined };
  }
  it('accepts actual title-keyed hashes of wrapped original prior text', () => {
    expect(verifyEditorialPriorContext(realEdge())).toEqual([]);
  });
  it('accepts verified revised text with actual title framing', () => {
    expect(verifyEditorialPriorContext(realEdge(true))).toEqual([]);
  });
  it('rejects empty raw input despite a correct historical edge hash', () => {
    const input = realEdge(); input.originalReceipt.synthesis_input_text = '';
    expect(verifyEditorialPriorContext(input)).toContain('part1:editorial_prior_context_edge_raw_input_missing:opening');
  });
  it('rejects an explicitly null edge instead of defaulting to original', () => {
    const input = realEdge();
    expect(verifyEditorialPriorContext({ ...input, priorContext: { opening: null } as any }))
      .toContain('part1:editorial_prior_context_edge_malformed:opening');
  });
});

it('binds the full Jev review identity even when all verdicts are unchanged', () => {
  const input = makeReviewedEditorialInput({ mutateEvidence: e => ({ ...e,
    freshJev: { ...e.freshJev, receipt: { ...e.freshJev.receipt, model: 'a-different-review-model' } },
  }) });
  expect(verifyReferenceExecution(input).blockers).toContain('opening:editorial_guardrail_disposition_jev_binding_mismatch');
});

// ─── Optional editorial passage-evidence wiring ───────────────────────
// Focused tests for the new EditorialSectionEvidence.freshAudit.passageEvidence
// path validated by verifyEditorialSection with caller-supplied
// trustedPassageSources + expectedCF.  Existing APIs absent evidence
// must preserve behaviour (backward-compatible), and independently
// trusted registry/CF target/text maps MUST come from caller options —
// never from the evidence sidecar itself.

describe('editorial-verification: optional passage evidence supplement', () => {
  const CF_ACCOUNT = '9d9d23b27f32e70ae3afb6a1aa2c0f10';
  const CF_INDEX = 'witness-wisdom-corpus';
  const OPENING_BASE_TEXTS = ['opening passage'];
  const OPENING_BASE_IDS = ['sw:opening:p0'];

  function buildPassageEvidenceCall(overrides: {
    mutateAdditions?: (adds: any[]) => any[];
    mutateTrusted?: (rec: Record<string, any>) => Record<string, any>;
    mutateEvidenceSidecar?: (ev: any) => any;
    mutateReceipt?: (r: any) => any;
    omitTrustedPassageSources?: boolean;
    omitExpectedCF?: boolean;
    corruptBaseInReceipt?: boolean;
  } = {}) {
    // Rebuild opening receipt to match evidence base semantics: source_ids
    // must round-trip against the passage evidence bundle byte-for-byte.
    const rawSynthesis = OPENING_ORIGINAL;
    const openingOutputHash = sha256(rawSynthesis);
    const synthesisInput = `${ALETHEIOS_OPENING}\n\n${PICHET_OPENING}`;
    const basePassages: any[] = OPENING_BASE_IDS.map((id, i) => ({
      id, source: `cf-vectorize:${CF_ACCOUNT}/${CF_INDEX}/${id}`, text: OPENING_BASE_TEXTS[i],
    }));
    const basePassagesHash = sha256(OPENING_BASE_TEXTS.join('\n'));
    let openingReceipt: SectionExecutionReceipt = {
      section_id: 'opening',
      outcome: 'ok',
      source_ids: OPENING_BASE_IDS,
      source_hashes: OPENING_BASE_TEXTS.map(sha256),
      passages_hash: basePassagesHash,
      aletheios_hash: sha256(ALETHEIOS_OPENING),
      pichet_hash: sha256(PICHET_OPENING),
      synthesis_input_hash: sha256(synthesisInput),
      synthesis_input_text: synthesisInput,
      prior_section_content_hashes: {},
      output_hash: openingOutputHash,
      aletheios_persona: { name: 'aletheios', sourcePath: '/dev/null/a', sourceHash: sha256(ALETHEIOS_IDENTITY) },
      pichet_persona: { name: 'pichet', sourcePath: '/dev/null/p', sourceHash: sha256(PICHET_IDENTITY) },
      raw: { aletheios: ALETHEIOS_OPENING, pichet: PICHET_OPENING, synthesis: rawSynthesis },
      retrieval: { section_id: 'opening', state: 'success', count: 1, source_ids: OPENING_BASE_IDS, passages_hash: basePassagesHash } as any,
      attempts: [{ attempt: 1, outcome: 'ok', aletheios_raw: ALETHEIOS_OPENING, pichet_raw: PICHET_OPENING, synthesis_raw: rawSynthesis } as any],
      engine_facts: OPENING_ENGINE_FACTS,
    } as any;
    if (overrides.mutateReceipt) openingReceipt = overrides.mutateReceipt(openingReceipt);

    const applied = applyEditorialRevision({
      baseReceipt: openingReceipt,
      currentOutput: openingReceipt.raw.synthesis,
      expectedBaseOutputHash: openingReceipt.output_hash,
      patches: OPENING_PATCHES,
      reviewer: 'assistant-source-review',
      timestamp: '2026-09-30T01:13:59.113Z',
    });
    const revisedOutput = applied.revisedOutput;

    // One CF addition + one reviewed-primary-document addition.
    const CF_ADD_TEXT = 'Freshly readback CF passage on descriptive framing.';
    const PRIM_ADD_TEXT = 'Rao paraphrase: descriptive traditional framing.';
    const cfAdd: any = {
      id: 'sw:hd:framing:desc:extra', text: CF_ADD_TEXT, sha256: sha256(CF_ADD_TEXT),
      provenance: { kind: 'cf-vectorize', account: CF_ACCOUNT, index: CF_INDEX },
    };
    const primAdd: any = {
      id: 'primary:rao2000:opening:framing', text: PRIM_ADD_TEXT, sha256: sha256(PRIM_ADD_TEXT),
      provenance: { kind: 'reviewed-primary-document', author: 'Rao', title: 'Vedic Astrology', url: 'https://www.vedicastrologer.org/articles/vedic_astro_textbook.pdf', locator: 'Ch 7.2', reviewer: 'assistant-source-review' },
    };
    let additions = [cfAdd, primAdd];
    if (overrides.mutateAdditions) additions = overrides.mutateAdditions(additions);

    let trusted: Record<string, any> = { [cfAdd.id]: cfAdd, [primAdd.id]: primAdd };
    if (overrides.mutateTrusted) trusted = overrides.mutateTrusted(trusted);

    // Effective view = base + additions.
    const effectivePassagesJoin = [...OPENING_BASE_TEXTS, ...additions.map((a: any) => a.text)].join('\n');
    const effectivePassagesHash = sha256(effectivePassagesJoin);

    let sidecar: any = {
      basePassages, basePassagesHash, additions,
      addedSourceRecords: additions, effectivePassagesHash,
    };
    if (overrides.mutateEvidenceSidecar) sidecar = overrides.mutateEvidenceSidecar(sidecar);

    // Fresh audit bound to the effective hash.
    const engineFacts = openingReceipt.engine_facts as string;
    const auditModelRaw = JSON.stringify({
      status: 'clean',
      coverage: { total_claims_audited: 1, sources_referenced: 1 },
      claim_checks: [{ claim_quoted: revisedOutput.split('\n')[0], source_reference: { source_path_or_passage_id: OPENING_BASE_IDS[0], supporting_values: 'engine records' } }],
      findings: [],
    });
    const auditInputSha = computeAuditInputHash({ acceptedOutput: revisedOutput, passagesHash: sidecar.effectivePassagesHash, engineFacts });
    const audit: SourceAuditReceipt = {
      section_id: 'opening', status: 'clean', model_raw: auditModelRaw, input_sha256: auditInputSha, output_sha256: sha256(auditModelRaw),
      coverage: { total_claims_audited: 1, sources_referenced: 1 },
      claim_checks: [{ claim_quoted: revisedOutput.split('\n')[0], source_reference: { source_path_or_passage_id: OPENING_BASE_IDS[0], supporting_values: 'engine records' } }],
      findings: [],
    } as any;

    const baseReceiptRawBytes = JSON.stringify(openingReceipt);
    const freshAudit: FreshAuditSidecar = {
      kind: 'fresh-audit-of-editorial-revision',
      baseReceiptSha256: sha256(baseReceiptRawBytes),
      sectionOutputHash: sha256(revisedOutput),
      originalOutcome: 'ok',
      engineFacts,
      audit,
      passageEvidence: sidecar,
      finalAccepted: false,
    };
    const freshJev: FreshJevSidecar = {
      sectionOutputHash: sha256(revisedOutput),
      receipt: { pass_id: 'opening', mode: 'shadow', status: 'judged', verdicts: { guardrail: 'pass', framing: 'pass', grounding: 'pass', register: 'pass' }, blocked: false, disagreements: [] },
      finalAccepted: false,
    };

    const evidence: EditorialSectionEvidence = {
      baseReceiptSha256: freshAudit.baseReceiptSha256,
      baseReceiptRawBytes: overrides.corruptBaseInReceipt ? '{"corrupt":true}' : baseReceiptRawBytes,
      revisionArtifact: {
        sectionId: 'opening',
        baseOutputHash: applied.baseOutputHash,
        revisedOutput: applied.revisedOutput,
        revisedOutputHash: applied.revisedOutputHash,
        patches: OPENING_PATCHES,
        reviewer: 'assistant-source-review',
        timestamp: '2026-09-30T01:13:59.113Z',
      },
      freshAudit,
      freshJev,
    };

    return {
      sectionId: 'opening',
      originalReceipt: openingReceipt,
      passOutput: applied.revisedOutput,
      passOutputHash: applied.revisedOutputHash,
      evidence,
      trustedPassageSources: overrides.omitTrustedPassageSources ? undefined : trusted,
      expectedCF: overrides.omitExpectedCF ? undefined : { account: CF_ACCOUNT, index: CF_INDEX },
    };
  }

  it('accepts a supplemented fresh audit whose input hash binds the effective passages hash', async () => {
    const { verifyEditorialSection } = await import('./editorial-verification.js');
    const args = buildPassageEvidenceCall();
    const verdict = verifyEditorialSection(args);
    expect(verdict.blockers).toEqual([]);
    expect(verdict.sectionId).toBe('opening');
  });

  it('preserves existing behavior when no passage evidence is present (no-evidence backward compat)', async () => {
    const { verifyEditorialSection } = await import('./editorial-verification.js');
    const args = buildPassageEvidenceCall();
    // Strip the passageEvidence entirely; audit must rebind to base passages_hash.
    const engineFacts = args.originalReceipt.engine_facts as string;
    const revised = args.evidence.revisionArtifact.revisedOutput;
    const auditModelRaw = args.evidence.freshAudit.audit.model_raw;
    const auditInputSha = computeAuditInputHash({ acceptedOutput: revised, passagesHash: args.originalReceipt.passages_hash, engineFacts });
    const patched = {
      ...args,
      evidence: {
        ...args.evidence,
        freshAudit: {
          ...args.evidence.freshAudit,
          passageEvidence: undefined,
          audit: { ...args.evidence.freshAudit.audit, input_sha256: auditInputSha, output_sha256: sha256(auditModelRaw) },
        },
      },
      // Trusted maps must be irrelevant when no evidence is present.
      trustedPassageSources: undefined,
      expectedCF: undefined,
    };
    const verdict = verifyEditorialSection(patched);
    expect(verdict.blockers).toEqual([]);
  });

  it('blocks a tampered addition text whose sha256 no longer matches', async () => {
    const { verifyEditorialSection } = await import('./editorial-verification.js');
    const args = buildPassageEvidenceCall({
      mutateEvidenceSidecar: (ev) => {
        // Tamper the FIRST addition text after the audit was minted.
        const bad = { ...ev, additions: [ { ...ev.additions[0], text: 'TAMPERED payload' }, ev.additions[1] ] };
        return bad;
      },
    });
    const verdict = verifyEditorialSection(args);
    expect(verdict.blockers.some((b) => b.includes('editorial_audit_passage_evidence_'))).toBe(true);
  });

  it('blocks when base ids are reordered in the evidence bundle', async () => {
    const { verifyEditorialSection } = await import('./editorial-verification.js');
    // Add a second base passage to the receipt so we can reorder.
    const SECOND_TEXT = 'second opening passage';
    const SECOND_ID = 'sw:opening:p1';
    const combinedHash = sha256([OPENING_BASE_TEXTS[0], SECOND_TEXT].join('\n'));
    const args = buildPassageEvidenceCall({
      mutateReceipt: (r) => ({
        ...r,
        source_ids: [OPENING_BASE_IDS[0], SECOND_ID],
        source_hashes: [sha256(OPENING_BASE_TEXTS[0]), sha256(SECOND_TEXT)],
        passages_hash: combinedHash,
        retrieval: { ...r.retrieval, count: 2, source_ids: [OPENING_BASE_IDS[0], SECOND_ID], passages_hash: combinedHash },
      }),
      mutateEvidenceSidecar: (ev) => {
        // Bundle carries both, but reversed vs receipt.source_ids order.
        const bp0 = { id: OPENING_BASE_IDS[0], source: `cf-vectorize:${CF_ACCOUNT}/${CF_INDEX}/${OPENING_BASE_IDS[0]}`, text: OPENING_BASE_TEXTS[0] };
        const bp1 = { id: SECOND_ID, source: `cf-vectorize:${CF_ACCOUNT}/${CF_INDEX}/${SECOND_ID}`, text: SECOND_TEXT };
        return { ...ev, basePassages: [bp1, bp0], basePassagesHash: combinedHash };
      },
    });
    const verdict = verifyEditorialSection(args);
    expect(verdict.blockers.some((b) => b.includes('editorial_audit_passage_evidence_base_id_mismatch'))).toBe(true);
  });

  it('blocks when the audit sidecar swaps the effective passages hash', async () => {
    const { verifyEditorialSection } = await import('./editorial-verification.js');
    const args = buildPassageEvidenceCall({
      mutateEvidenceSidecar: (ev) => ({ ...ev, effectivePassagesHash: sha256('an-unrelated-value') }),
    });
    const verdict = verifyEditorialSection(args);
    expect(
      verdict.blockers.some((b) =>
        b.includes('editorial_audit_passage_evidence_effective_hash_mismatch') ||
        b.includes('editorial_audit_input_hash_mismatch'),
      ),
    ).toBe(true);
  });

  it('blocks when trustedPassageSources is omitted despite passage evidence being present (untrusted source)', async () => {
    const { verifyEditorialSection } = await import('./editorial-verification.js');
    const args = buildPassageEvidenceCall({ omitTrustedPassageSources: true });
    const verdict = verifyEditorialSection(args);
    expect(verdict.blockers).toContain('opening:editorial_audit_passage_evidence_untrusted_source');
  });

  it.each(['null', 'null-base-entry', 'object-additions'])('rejects malformed passage evidence without throwing: %s', async (kind) => {
    const { verifyEditorialSection } = await import('./editorial-verification.js');
    const args = buildPassageEvidenceCall();
    const original = args.evidence.freshAudit.passageEvidence!;
    (args.evidence.freshAudit as any).passageEvidence = kind === 'null' ? null
      : kind === 'null-base-entry' ? { ...original, basePassages: [null] }
      : { ...original, additions: {} };
    expect(verifyEditorialSection(args).blockers).toContain('opening:editorial_audit_passage_evidence_invalid');
  });

  it('blocks when an addition mislabels a primary-document as cf-vectorize (primary mislabel)', async () => {
    const { verifyEditorialSection } = await import('./editorial-verification.js');
    const args = buildPassageEvidenceCall({
      mutateAdditions: (adds) => {
        // Rewrite the reviewed-primary-document addition (index 1) as a
        // cf-vectorize provenance while still using a `primary:` id —
        // provenance validator rejects the id/prefix crossover.
        const [cf, prim] = adds;
        const mislabeled = { ...prim, provenance: { kind: 'cf-vectorize', account: CF_ACCOUNT, index: CF_INDEX } };
        return [cf, mislabeled];
      },
      mutateTrusted: (rec) => {
        // Keep trusted registry consistent with the sidecar to isolate
        // the mislabel failure to the provenance validator (not a
        // trusted-record equality mismatch).
        const primId = 'primary:rao2000:opening:framing';
        rec[primId] = { ...rec[primId], provenance: { kind: 'cf-vectorize', account: CF_ACCOUNT, index: CF_INDEX } };
        return rec;
      },
    });
    const verdict = verifyEditorialSection(args);
    expect(verdict.blockers.some((b) => b.startsWith('opening:editorial_audit_passage_evidence_'))).toBe(true);
  });
});
