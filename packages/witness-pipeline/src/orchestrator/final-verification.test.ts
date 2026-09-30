import { describe, it, expect } from 'vitest';
import { runFinalVerification } from './final-verification.js';
import type { PassResult } from './integrated.js';
import type { DyadSectionReceipt } from './witness-dyad.js';
import { DYAD_UNAVAILABLE_MARKER } from './witness-dyad.js';
import type { RetrievalReceipt } from './grounding-adapter.js';
import fs from 'node:fs';
import { createHash } from 'node:crypto';
import os from 'node:os';
import path from 'node:path';

const failingPass = {
  id: 'opening',
  rubric: { placeholder_gate: 'fail', chart_fidelity_gate: 'pass' },
} as unknown as PassResult;

const passingPass = {
  id: 'opening',
  rubric: { placeholder_gate: 'pass', chart_fidelity_gate: 'pass' },
} as unknown as PassResult;

describe('runFinalVerification', () => {
  it('fails when any section has placeholder_gate fail', () => {
    const result = runFinalVerification({ passes: [failingPass], pdfPath: 'does-not-exist.pdf' });
    expect(result.passed).toBe(false);
    expect(result.blockers).toContain('opening:placeholder_gate');
  });

  it('passes when all gates pass and pdf exists', () => {
    const tmpPdf = path.join(os.tmpdir(), 'selemene-test.pdf');
    fs.writeFileSync(tmpPdf, '%PDF-1.4 test');
    const result = runFinalVerification({ passes: [passingPass], pdfPath: tmpPdf });
    fs.unlinkSync(tmpPdf);
    expect(result.passed).toBe(true);
  });
});

// ─── Helpers for reference route tests ───────────────────────────────

function makePass(id: string, output = `Section ${id} content with Vedic Lagna Human Design gate dasha transit.`): PassResult {
  return {
    id,
    title: `Title ${id}`,
    output,
    rubric: {
      section_id: id,
      title: `Title ${id}`,
      target_words: 100,
      actual_words: 90,
      word_count_fit: 'pass',
      word_count_ratio: 0.9,
      deterministic_fact_count: 5,
      deterministic_fact_gate: 'pass',
      integrated_layer_count: 3,
      integrated_layering_gate: 'pass',
      guardrail_gate: 'pass',
      guardrail_violations: [],
      model_requested: 'test',
      model_used: 'test',
      latency_ms: 10,
    },
  };
}

function makeDyadReceipt(sectionId: string, available: boolean): DyadSectionReceipt {
  return {
    section_id: sectionId,
    engine_routing: 'dyad-synthesis',
    aletheios_words: available ? 100 : 0,
    pichet_words: available ? 80 : 0,
    synthesis_words: available ? 200 : 0,
    aletheios_ok: available,
    pichet_ok: available,
    synthesis_ok: available,
    dyad_available: available,
  };
}

// ─── Retrieval required for reference route ───────────────────────────

describe('runFinalVerification — reference route retrieval required', () => {
  it('blocks when retrieval receipt is missing for a required section', () => {
    const passes = [makePass('opening'), makePass('part1')];
    const result = runFinalVerification({
      passes,
      referenceRoute: {
        requiredSectionIds: ['opening', 'part1'],
        dyadReceipts: [makeDyadReceipt('opening', true), makeDyadReceipt('part1', true)],
        // No retrievalReceipts provided
        requiredRubricGates: { dyad_receipt: true, retrieval_receipt: true },
      },
    });
    expect(result.passed).toBe(false);
    expect(result.blockers.some((b) => b.includes('retrieval_receipt_missing'))).toBe(true);
  });

  it('blocks when retrieval is disabled (no opt-out allowed for reference route)', () => {
    const passes = [makePass('opening')];
    const disabledReceipt: RetrievalReceipt = { state: 'disabled', count: 0, reason: 'provider=null' };
    const result = runFinalVerification({
      passes,
      referenceRoute: {
        requiredSectionIds: ['opening'],
        dyadReceipts: [makeDyadReceipt('opening', true)],
        retrievalReceipts: [disabledReceipt],
        requiredRubricGates: { retrieval_receipt: true },
      },
    });
    expect(result.passed).toBe(false);
    expect(result.blockers.some((b) => b.includes('retrieval_disabled') && b.includes('opening'))).toBe(true);
  });

  it('blocks on retrieval failure', () => {
    const passes = [makePass('opening')];
    const failedReceipt: RetrievalReceipt = { state: 'failure', count: 0, reason: 'HTTP 500' };
    const result = runFinalVerification({
      passes,
      referenceRoute: {
        requiredSectionIds: ['opening'],
        dyadReceipts: [makeDyadReceipt('opening', true)],
        retrievalReceipts: [failedReceipt],
        requiredRubricGates: { retrieval_receipt: true },
      },
    });
    expect(result.passed).toBe(false);
    expect(result.blockers.some((b) => b.includes('retrieval_failure'))).toBe(true);
  });

  it('blocks when required retrieval is empty', () => {
    const passes = [makePass('opening')];
    const emptyReceipt: RetrievalReceipt = { state: 'empty', count: 0 };
    const result = runFinalVerification({
      passes,
      referenceRoute: {
        requiredSectionIds: ['opening'],
        dyadReceipts: [makeDyadReceipt('opening', true)],
        retrievalReceipts: [emptyReceipt],
        requiredRubricGates: { retrieval_receipt: true },
      },
    });
    expect(result.blockers.some((b) => b.includes('retrieval_empty'))).toBe(true);
  });
});

// ─── Synthesis completeness (structural, not prose regex) ─────────────

describe('runFinalVerification — synthesis completeness structural check', () => {
  it('blocks synthesis when a required non-synthesis section has DYAD_UNAVAILABLE in output', () => {
    const synthSection = makePass('part11');
    const unavailableSection = {
      ...makePass('part1'),
      output: DYAD_UNAVAILABLE_MARKER + ' reason=timeout',
    };
    const result = runFinalVerification({
      passes: [makePass('opening'), unavailableSection, synthSection],
      referenceRoute: {
        requiredSectionIds: ['opening', 'part1', 'part11'],
        dyadReceipts: [
          makeDyadReceipt('opening', true),
          makeDyadReceipt('part1', false),
          makeDyadReceipt('part11', true),
        ],
        retrievalReceipts: [
          { state: 'success', count: 3 },
          { state: 'failure', count: 0, reason: 'timeout' },
          { state: 'success', count: 3 },
        ],
        requiredRubricGates: { dyad_receipt: true, retrieval_receipt: true },
      },
    });
    expect(result.passed).toBe(false);
    expect(result.blockers.some((b) => b.includes('synthesis_missing_accepted_sections') && b.includes('part1'))).toBe(true);
  });

  it('does not block synthesis when all sections accepted', () => {
    const passes = ['opening', 'part1', 'part11'].map((id) => makePass(id));
    const result = runFinalVerification({
      passes,
      referenceRoute: {
        requiredSectionIds: ['opening', 'part1', 'part11'],
        dyadReceipts: passes.map((p) => makeDyadReceipt(p.id, true)),
        retrievalReceipts: passes.map(() => ({ state: 'success' as const, count: 2 })),
        requiredRubricGates: { dyad_receipt: true, retrieval_receipt: true },
      },
    });
    // No synthesis blocker
    expect(result.blockers.some((b) => b.includes('synthesis_missing'))).toBe(false);
  });
});


describe('reference acceptance cannot be weakened', () => {
  function validInput() {
    const passes = [makePass('opening'), makePass('part11')];
    return { passes, referenceRoute: {
      requiredSectionIds: ['opening', 'part11'],
      dyadReceipts: passes.map(p => makeDyadReceipt(p.id, true)),
      retrievalReceipts: passes.map(() => ({ state: 'success' as const, count: 2 })),
      synthesisInputHashes: { opening: createHash('sha256').update(passes[0].output).digest('hex') },
    } };
  }
  it('accepts a complete synthetic contract', () => {
    expect(runFinalVerification(validInput()).passed).toBe(true);
  });
  it('ignores attempts to disable required dependency gates', () => {
    const input = validInput();
    input.referenceRoute.retrievalReceipts = [];
    const result = runFinalVerification({ ...input, referenceRoute: { ...input.referenceRoute, requiredRubricGates: { retrieval_receipt: false, dyad_receipt: false } } });
    expect(result.blockers.some(b => b.includes('retrieval_receipt_missing'))).toBe(true);
  });
  it('rejects a missing required rubric field', () => {
    const input = validInput();
    delete (input.passes[0].rubric as any).guardrail_gate;
    expect(runFinalVerification(input).blockers).toContain('opening:required_gate_missing:guardrail_gate');
  });
  it('rejects synthesis receipts after source text changes', () => {
    const input = validInput();
    input.passes[0].output += ' An unreviewed edit.';
    expect(runFinalVerification(input).blockers).toContain('part11:synthesis_input_unverified:opening');
  });
  it('rejects duplicate generated sections', () => {
    const input = validInput();
    input.passes.push(input.passes[0]);
    expect(runFinalVerification(input).blockers).toContain('manifest:duplicate_result_id');
  });
  it('rejects success receipts with no retrieved passages', () => {
    const input = validInput();
    input.referenceRoute.retrievalReceipts[0].count = 0;
    expect(runFinalVerification(input).blockers).toContain('opening:retrieval_success_without_passages');
  });
});


// ─── Source-audit gate (real reference route) ─────────────────────────

import { computeAuditInputHash } from './reference-source-audit.js';
import type { SourceAuditReceipt } from './reference-source-audit.js';

function makeAudit(
  sectionId: string,
  passOutput: string,
  passagesHash: string,
  engineFacts: string,
  overrides: Partial<SourceAuditReceipt> = {},
): SourceAuditReceipt {
  const modelRaw = JSON.stringify({
    status: 'clean',
    coverage: { total_claims_audited: 1, sources_referenced: 1 },
    claim_checks: [{ claim_quoted: passOutput, source_reference: { source_path_or_passage_id: 'panchanga.result.tithi', supporting_values: 'Shukla 3' } }],
    findings: [],
  });
  return {
    section_id: sectionId,
    status: 'clean',
    model_raw: modelRaw,
    input_sha256: computeAuditInputHash({ acceptedOutput: passOutput, passagesHash, engineFacts }),
    output_sha256: createHash('sha256').update(modelRaw).digest('hex'),
    coverage: { total_claims_audited: 1, sources_referenced: 1 },
    claim_checks: [{ claim_quoted: passOutput, source_reference: { source_path_or_passage_id: 'panchanga.result.tithi', supporting_values: 'Shukla 3' } }],
    findings: [],
    ...overrides,
  };
}

describe('runFinalVerification — source-audit gate', () => {
  function buildInput() {
    const passes = [makePass('opening'), makePass('part11')];
    const passagesHash = createHash('sha256').update('passage-text').digest('hex');
    const engineFacts = 'Panchanga: Tithi Shukla 3.';
    const sourceAuditReceipts: Record<string, SourceAuditReceipt> = {};
    for (const p of passes) {
      sourceAuditReceipts[p.id] = makeAudit(p.id, p.output, passagesHash, engineFacts);
    }
    return {
      passes,
      passagesHash,
      engineFacts,
      sourceAuditReceipts,
      referenceRoute: {
        requiredSectionIds: ['opening', 'part11'],
        dyadReceipts: passes.map((p) => makeDyadReceipt(p.id, true)),
        retrievalReceipts: passes.map(() => ({ state: 'success' as const, count: 2 })),
        synthesisInputHashes: { opening: createHash('sha256').update(passes[0].output).digest('hex') },
        requireSourceAudit: true,
        sourceAuditReceipts,
        sectionPassagesHash: { opening: passagesHash, part11: passagesHash },
        sectionEngineFacts: { opening: engineFacts, part11: engineFacts },
      },
    };
  }

  it('accepts when all sections have clean hash-bound audits', () => {
    const input = buildInput();
    expect(runFinalVerification(input).passed).toBe(true);
  });

  it('blocks when a source-audit receipt is missing for a required section', () => {
    const input = buildInput();
    delete input.referenceRoute.sourceAuditReceipts.part11;
    const result = runFinalVerification(input);
    expect(result.passed).toBe(false);
    expect(result.blockers.some((b) => b.includes('part11:source_audit_missing'))).toBe(true);
  });

  it('blocks when audit status is not clean', () => {
    const input = buildInput();
    input.referenceRoute.sourceAuditReceipts.opening = {
      ...input.referenceRoute.sourceAuditReceipts.opening,
      status: 'changes-required',
      reason: 'wrong numerology grouping',
    };
    const result = runFinalVerification(input);
    expect(result.passed).toBe(false);
    expect(
      result.blockers.some((b) => b.includes('opening:source_audit_status_changes-required')),
    ).toBe(true);
  });

  it('blocks a tampered audit output hash', () => {
    const input = buildInput();
    input.referenceRoute.sourceAuditReceipts.opening = {
      ...input.referenceRoute.sourceAuditReceipts.opening,
      output_sha256: createHash('sha256').update('tampered').digest('hex'),
    };
    const result = runFinalVerification(input);
    expect(result.passed).toBe(false);
    expect(
      result.blockers.some((b) => b.includes('opening:source_audit_output_hash_mismatch')),
    ).toBe(true);
  });

  it('blocks when accepted pass output diverges from audited text', () => {
    const input = buildInput();
    input.passes[0].output += ' Extra edit added after audit.';
    const result = runFinalVerification(input);
    expect(result.passed).toBe(false);
    expect(
      result.blockers.some((b) => b.includes('opening:source_audit_input_hash_mismatch')),
    ).toBe(true);
  });

  it('blocks a clean audit with zero coverage', () => {
    const input = buildInput();
    input.referenceRoute.sourceAuditReceipts.opening = {
      ...input.referenceRoute.sourceAuditReceipts.opening,
      coverage: { total_claims_audited: 0, sources_referenced: 0 },
    };
    const result = runFinalVerification(input);
    expect(result.passed).toBe(false);
    expect(
      result.blockers.some((b) => b.includes('opening:source_audit_zero_coverage')),
    ).toBe(true);
  });

  it('does not enforce source-audit gate when requireSourceAudit is false', () => {
    const input = buildInput();
    input.referenceRoute.requireSourceAudit = false;
    input.referenceRoute.sourceAuditReceipts = {};
    const result = runFinalVerification(input);
    // no source_audit blockers even though receipts are absent
    expect(result.blockers.filter((b) => b.includes('source_audit'))).toHaveLength(0);
  });
});

// ═══════════════════════════════════════════════════════════════════════
// ─── Additional imports for runFinalReview tests below ────────────────
// ═══════════════════════════════════════════════════════════════════════

// ═══════════════════════════════════════════════════════════════════════
// ─── runFinalReview: reference-verifier-bound acceptance ─────────────
// ═══════════════════════════════════════════════════════════════════════
//
// Adversarial coverage:
//   * Mismatched pass output vs. receipt output_hash blocks acceptance.
//   * Mismatched assembled text (canonical formula tampered) blocks.
//   * Forged editorial acceptance (fresh audit dirty / fresh Jev fail /
//     missing coherence review) blocks.
//   * Missing rubric dimension blocks.
//   * CF snapshot missing / hash-mismatched / duplicate blocks.
//   * `mode !== 'full'` can never claim finalAccepted, even if every
//     component gate is green.
//   * Missing render / parity / coherence evidence blocks full mode.
//   * Fewer than 12 sections blocks full mode.
//   * Base verifier failures propagate under `ref:` prefix.
//   * `finalAccepted` cannot be forced by any caller-supplied field —
//     the input has no such field.

import { runFinalReview, type FinalReviewInput, type CloudflareSnapshot } from './final-verification.js';
import type { SectionExecutionReceipt } from './reference-execution.js';
import type { EditorialCoherenceReview } from './editorial-verification.js';

// ─── Fixtures ─────────────────────────────────────────────────────────

const SECTION_IDS_12 = [
  'opening', 'part1', 'part2', 'part3', 'part4', 'part5',
  'part6', 'part7', 'part8', 'part9', 'part10', 'part11',
];

const PERSONA_ALETHEIOS = '# aletheios IDENTITY\nStructural voice.';
const PERSONA_PICHET = '# pichet IDENTITY\nExperiential voice.';

function h(text: string): string {
  return createHash('sha256').update(text, 'utf8').digest('hex');
}

function makeSourceEntry(sectionId: string): { id: string; text: string; hash: string } {
  const id = `sw:test:${sectionId}:desc`;
  const text = `Passage bytes for section ${sectionId} — structural pattern notes from corpus.`;
  return { id, text, hash: h(text) };
}

function makeReviewReceipt(sectionId: string, seq: number, allPassOutputs: Record<string, string> = {}): SectionExecutionReceipt {
  const engineFacts = `Engine facts for ${sectionId}: Tithi Shukla 3, Nakshatra Rohini.`;
  const aletheiosRaw = `Aletheios(${sectionId}) structural output referencing engine facts.`;
  const pichetRaw = `Pichet(${sectionId}) experiential reflection output.`;
  const output = `## ${sectionId}\n\nSection ${sectionId} witness content with Vedic Lagna Human Design gate dasha transit passage notes.`;
  const source = makeSourceEntry(sectionId);
  // Prior section content hashes: only for later sections
  const priorHashes: Record<string, string> = {};
  const priorTexts: string[] = [];
  for (let i = 0; i < seq; i++) {
    const priorId = SECTION_IDS_12[i];
    const priorOutput = allPassOutputs[priorId] ?? '';
    priorHashes[`Title ${priorId}`] = h(priorOutput);
    priorTexts.push(priorOutput);
  }
  const synthesisInputText = [engineFacts, aletheiosRaw, pichetRaw, ...priorTexts].join('\n\n');
  const modelRawAudit = JSON.stringify({
    status: 'clean',
    coverage: { total_claims_audited: 1, sources_referenced: 1 },
    claim_checks: [{ claim_quoted: output, source_reference: { source_path_or_passage_id: source.id, supporting_values: 'Shukla 3' } }],
    findings: [],
  });
  return {
    section_id: sectionId,
    outcome: 'ok',
    source_ids: [source.id],
    source_hashes: [source.hash],
    passages_hash: source.hash,
    aletheios_hash: h(aletheiosRaw),
    pichet_hash: h(pichetRaw),
    synthesis_input_hash: h(synthesisInputText),
    synthesis_input_text: synthesisInputText,
    prior_section_content_hashes: priorHashes,
    output_hash: h(output),
    aletheios_persona: { name: 'aletheios', sourcePath: '/tmp/aletheios/IDENTITY.md', sourceHash: h(PERSONA_ALETHEIOS) },
    pichet_persona: { name: 'pichet', sourcePath: '/tmp/pichet/IDENTITY.md', sourceHash: h(PERSONA_PICHET) },
    raw: { aletheios: aletheiosRaw, pichet: pichetRaw, synthesis: output },
    retrieval: { section_id: sectionId, state: 'success', count: 1, source_ids: [source.id], passages_hash: source.hash },
    engine_facts: engineFacts,
    jev: {
      pass_id: sectionId, mode: 'shadow', status: 'judged', model: 'fixture', latency_ms: 1,
      verdicts: { guardrail: 'pass', framing: 'pass', grounding: 'pass', register: 'pass' },
      blocked: false, disagreements: [],
    },
    source_audit: {
      section_id: sectionId,
      status: 'clean',
      model_raw: modelRawAudit,
      input_sha256: computeAuditInputHash({ acceptedOutput: output, passagesHash: source.hash, engineFacts }),
      output_sha256: h(modelRawAudit),
      coverage: { total_claims_audited: 1, sources_referenced: 1 },
      claim_checks: [{ claim_quoted: output, source_reference: { source_path_or_passage_id: source.id, supporting_values: 'Shukla 3' } }],
      findings: [],
    },
    attempts: [{ attempt: 1, outcome: 'ok', aletheios_raw: aletheiosRaw, pichet_raw: pichetRaw, synthesis_raw: output }],
  };
}

function buildFullValidInput(): FinalReviewInput {
  const outputs: Record<string, string> = {};
  for (let i = 0; i < SECTION_IDS_12.length; i++) {
    const id = SECTION_IDS_12[i];
    outputs[id] = `## ${id}\n\nSection ${id} witness content with Vedic Lagna Human Design gate dasha transit passage notes.`;
  }
  const receipts = SECTION_IDS_12.map((id, seq) => makeReviewReceipt(id, seq, outputs));
  const passes: PassResult[] = SECTION_IDS_12.map(id => ({
    id,
    title: `Title ${id}`,
    output: outputs[id],
    rubric: {
      section_id: id, title: `Title ${id}`, target_words: 80, actual_words: 82,
      word_count_fit: 'pass', word_count_ratio: 1.025,
      deterministic_fact_count: 5, deterministic_fact_gate: 'pass',
      integrated_layer_count: 3, integrated_layering_gate: 'pass',
      guardrail_gate: 'pass', guardrail_violations: [],
      model_requested: 'test', model_used: 'test', latency_ms: 1,
    },
  }));
  const assembled = passes.map(p => `## ${p.title}\n\n${p.output}`).join('\n\n');
  const rawSynthesisInputs: Record<string, string> = {};
  const sectionEngineFacts: Record<string, string> = {};
  const sourceTexts: Record<string, string> = {};
  const subsectionMap: Record<string, string[]> = {};
  const cfPassages: Array<{ id: string; text: string; sha256: string }> = [];
  for (const r of receipts) {
    rawSynthesisInputs[r.section_id] = r.synthesis_input_text;
    sectionEngineFacts[r.section_id] = r.engine_facts ?? '';
    subsectionMap[r.section_id] = [];
    for (let i = 0; i < r.source_ids.length; i++) {
      const id = r.source_ids[i];
      const text = `Passage bytes for section ${r.section_id} — structural pattern notes from corpus.`;
      sourceTexts[id] = text;
      cfPassages.push({ id, text, sha256: h(text) });
    }
  }
  const cloudflareSnapshot: CloudflareSnapshot = {
    accountId: 'a'.repeat(32),
    indexName: 'witness-wisdom-corpus',
    passages: cfPassages,
  };
  const assembledSha256 = h(assembled);
  const renderBytes = JSON.stringify({ pages: 43, ok: true });
  const renderArtifactBytes = '%PDF-1.4 assembled artifact payload bytes';
  const renderReceipt = {
    path: 'render-qa/report.json',
    sha256: h(renderBytes),
    rawBytes: renderBytes,
    artifact: {
      path: 'render-qa/report.pdf',
      sha256: h(renderArtifactBytes),
      rawBytes: renderArtifactBytes,
      kind: 'application/pdf',
    },
    bindsAssembledSha256: assembledSha256,
    status: 'completed' as const,
    reviewer: 'human',
    reviewedAt: '2026-09-30T12:00:00Z',
    findings: [],
  };
  const coherenceReview: EditorialCoherenceReview = {
    artifactSha256: assembledSha256,
    orderedEffectiveHashes: receipts.map(r => r.output_hash),
    status: 'reviewed',
    reviewer: 'human',
    reviewedAt: '2026-09-30T12:00:00Z',
    rationale: 'Whole-document review completed; every section reads coherently against every other, with no cross-section contradiction or gap detected. This rationale is intentionally substantive to clear the minimum-length gate enforced by verifyEditorialCoherenceReview.',
    findings: [],
    reviewedRevisions: [],
  };
  return {
    mode: 'full',
    requiredSectionIds: SECTION_IDS_12,
    receipts,
    passes,
    assembled,
    expectedPersonas: { aletheios: PERSONA_ALETHEIOS, pichet: PERSONA_PICHET },
    subsectionMap,
    rawSynthesisInputs,
    sourceTexts,
    sectionEngineFacts,
    cloudflareSnapshot,
    renderReceipt,
    editorial: {
      revisedSections: {},
      originalPasses: Object.fromEntries(passes.map(p => [p.id, p.output])),
      coherenceReview,
    },
    bilingualExpected: false,
  };
}

// ─── Happy path ──────────────────────────────────────────────────────

describe('runFinalReview — happy path', () => {
  it('accepts a clean full-mode 12-section run', () => {
    const input = buildFullValidInput();
    const result = runFinalReview(input);
    if (!result.finalAccepted) {
      // Surface blockers on failure to make debugging fast.
      console.error('unexpected blockers:', result.blockers);
    }
    expect(result.finalAccepted).toBe(true);
    expect(result.generationAccepted).toBe(true);
    expect(result.sourceAccepted).toBe(true);
    expect(result.referenceRoutePassed).toBe(true);
    expect(result.packagingAccepted).toBe(true);
    expect(result.blockers).toEqual([]);
    expect(result.artifactSha256).toMatch(/^[0-9a-f]{64}$/);
    expect(result.packagingReport.sectionCount).toBe(12);
  });
});

// ─── Mode gate ────────────────────────────────────────────────────────

describe('runFinalReview — mode gate', () => {
  it('never claims finalAccepted for generation-only mode even when every gate is green', () => {
    const input = buildFullValidInput();
    input.mode = 'generation-only';
    const result = runFinalReview(input);
    expect(result.finalAccepted).toBe(false);
    // Sub-verdicts still surface so the CLI may report them independently.
    expect(result.generationAccepted).toBe(true);
    expect(result.sourceAccepted).toBe(true);
  });

  it('never claims finalAccepted for source-only mode', () => {
    const input = buildFullValidInput();
    input.mode = 'source-only';
    const result = runFinalReview(input);
    expect(result.finalAccepted).toBe(false);
  });
});

// ─── Adversarial: pass ↔ receipt mismatch ─────────────────────────────

describe('runFinalReview — pass output mismatch', () => {
  it('blocks when a pass output diverges from its receipt output_hash', () => {
    const input = buildFullValidInput();
    input.passes[0] = { ...input.passes[0], output: input.passes[0].output + ' TAMPERED APPENDAGE' };
    // Rebuild assembled to still match canonical formula so the caller cannot
    // pass by silently changing pass.output while re-assembling matching bytes.
    input.assembled = input.passes.map(p => `## ${p.title}\n\n${p.output}`).join('\n\n');
    const result = runFinalReview(input);
    expect(result.finalAccepted).toBe(false);
    expect(result.blockers.some(b => b.startsWith('ref:') && b.includes('pass_output_hash_mismatch'))).toBe(true);
  });

  it('blocks when the caller-supplied assembled text does not match the canonical join', () => {
    const input = buildFullValidInput();
    input.assembled = input.assembled + '\n\n<!-- injected extra prose -->';
    const result = runFinalReview(input);
    expect(result.finalAccepted).toBe(false);
    expect(result.blockers).toContain('assembled:content_mismatch_with_canonical');
  });
});

// ─── Adversarial: forged editorial acceptance ─────────────────────────

describe('runFinalReview — forged editorial acceptance', () => {
  function makeInputWithEditorial(): FinalReviewInput {
    const input = buildFullValidInput();
    const openingReceipt = input.receipts[0];
    const revisedOutput = openingReceipt.raw.synthesis + '\n\nEDITORIAL ADDITION';
    const revisedOutputHash = h(revisedOutput);
    const cleanAuditRaw = JSON.stringify({
      status: 'clean',
      coverage: { total_claims_audited: 1, sources_referenced: 1 },
      claim_checks: [{ claim_quoted: revisedOutput, source_reference: { source_path_or_passage_id: openingReceipt.source_ids[0], supporting_values: 'Shukla 3' } }],
      findings: [],
    });
    input.editorial = {
      revisedSections: {
        opening: {
          baseReceiptSha256: 'x'.repeat(64),
          baseReceiptRawBytes: JSON.stringify(openingReceipt),
          revisionArtifact: {
            sectionId: 'opening',
            baseOutputHash: openingReceipt.output_hash,
            revisedOutput,
            revisedOutputHash,
            patches: [{ before: 'gate', after: 'GATE', rationale: 'test' } as any],
            reviewer: 'assistant-source-review',
            timestamp: new Date().toISOString(),
          },
          freshAudit: {
            kind: 'fresh-audit-of-editorial-revision',
            baseReceiptSha256: 'x'.repeat(64),
            sectionOutputHash: revisedOutputHash,
            originalOutcome: 'ok',
            engineFacts: openingReceipt.engine_facts ?? '',
            audit: {
              section_id: 'opening',
              status: 'clean',
              model_raw: cleanAuditRaw,
              input_sha256: 'y'.repeat(64),
              output_sha256: h(cleanAuditRaw),
              coverage: { total_claims_audited: 1, sources_referenced: 1 },
              claim_checks: [],
              findings: [],
            } as any,
            finalAccepted: true,   // <— forged flag: the verifier must ignore it
          },
          freshJev: {
            sectionOutputHash: revisedOutputHash,
            receipt: {
              pass_id: 'opening', mode: 'shadow', status: 'judged',
              verdicts: { guardrail: 'pass', framing: 'pass', grounding: 'pass', register: 'pass' },
              blocked: false,
            },
            finalAccepted: true,   // <— forged flag
          },
        },
      },
      originalPasses: Object.fromEntries(input.passes.map(p => [p.id, p.output])),
      // NOTE: coherenceReview intentionally missing → must block
    };
    // Pass output for opening reflects revised text
    input.passes[0] = { ...input.passes[0], output: revisedOutput };
    input.assembled = input.passes.map(p => `## ${p.title}\n\n${p.output}`).join('\n\n');
    return input;
  }

  it('blocks even when forged finalAccepted flags claim acceptance', () => {
    const input = makeInputWithEditorial();
    const result = runFinalReview(input);
    // Cannot be accepted: coherence review is missing, hashes will diverge,
    // and no caller-supplied boolean is trusted.
    expect(result.finalAccepted).toBe(false);
    expect(result.blockers.length).toBeGreaterThan(0);
  });

  it('surfaces coherence:missing when editorial evidence is present without coherence review', () => {
    const input = makeInputWithEditorial();
    const result = runFinalReview(input);
    // Coherence gate lives in both surfaces; either must fire.
    const anyCoherence = result.blockers.some(b => b.includes('coherence'));
    expect(anyCoherence).toBe(true);
  });
});

// ─── Adversarial: rubric dimension missing ────────────────────────────

describe('runFinalReview — missing rubric dimension', () => {
  it('blocks when a required rubric gate is missing on any pass', () => {
    const input = buildFullValidInput();
    (input.passes[3] as any).rubric = { ...input.passes[3].rubric, deterministic_fact_gate: undefined };
    const result = runFinalReview(input);
    expect(result.finalAccepted).toBe(false);
    expect(
      result.blockers.some(b => b.includes('deterministic_fact')),
    ).toBe(true);
  });
});

// ─── Adversarial: CF snapshot integrity ───────────────────────────────

describe('runFinalReview — CF snapshot integrity', () => {
  it('blocks when a receipt source_id is missing from the snapshot', () => {
    const input = buildFullValidInput();
    input.cloudflareSnapshot!.passages.pop();     // drop last passage
    const result = runFinalReview(input);
    expect(result.finalAccepted).toBe(false);
    expect(result.blockers.some(b => b.startsWith('cf:') && b.includes('cloudflare_snapshot_missing'))).toBe(true);
  });

  it('blocks when a snapshot passage text does not hash to the receipt source_hash', () => {
    const input = buildFullValidInput();
    const first = input.cloudflareSnapshot!.passages[0];
    input.cloudflareSnapshot!.passages[0] = { ...first, text: first.text + ' TAMPERED' };
    const result = runFinalReview(input);
    expect(result.finalAccepted).toBe(false);
    expect(result.blockers.some(b => b.startsWith('cf:') && b.includes('hash_mismatch'))).toBe(true);
  });

  it('blocks full mode when snapshot is entirely missing but receipts declare source_ids', () => {
    const input = buildFullValidInput();
    delete input.cloudflareSnapshot;
    const result = runFinalReview(input);
    expect(result.finalAccepted).toBe(false);
    expect(result.blockers).toContain('cf:snapshot_missing_for_full_mode');
  });

  it('blocks on duplicate snapshot IDs', () => {
    const input = buildFullValidInput();
    const dup = input.cloudflareSnapshot!.passages[0];
    input.cloudflareSnapshot!.passages.push({ ...dup });
    const result = runFinalReview(input);
    expect(result.finalAccepted).toBe(false);
    expect(result.blockers.some(b => b.includes('cloudflare_snapshot_duplicate_id'))).toBe(true);
  });
});

// ─── Adversarial: packaging gates ─────────────────────────────────────

describe('runFinalReview — packaging gates', () => {
  it('blocks when render receipt is missing in full mode', () => {
    const input = buildFullValidInput();
    delete input.renderReceipt;
    const result = runFinalReview(input);
    expect(result.finalAccepted).toBe(false);
    expect(result.blockers).toContain('render:missing');
  });

  it('blocks when parity is required but missing', () => {
    const input = buildFullValidInput();
    input.bilingualExpected = true;
    const result = runFinalReview(input);
    expect(result.finalAccepted).toBe(false);
    expect(result.blockers).toContain('parity:missing');
  });

  it('blocks when fewer than 12 sections are declared for full mode', () => {
    const input = buildFullValidInput();
    input.requiredSectionIds = input.requiredSectionIds.slice(0, 8);
    input.receipts = input.receipts.slice(0, 8);
    input.passes = input.passes.slice(0, 8);
    input.assembled = input.passes.map(p => `## ${p.title}\n\n${p.output}`).join('\n\n');
    const result = runFinalReview(input);
    expect(result.finalAccepted).toBe(false);
    expect(result.blockers).toContain('sections:full_mode_requires_12');
  });

  it('blocks when render receipt bytes do not hash to the declared sha256', () => {
    const input = buildFullValidInput();
    input.renderReceipt = {
      ...input.renderReceipt!,
      path: 'render-qa/report.json',
      sha256: '0'.repeat(64),
      rawBytes: input.renderReceipt!.rawBytes!,
    };
    const result = runFinalReview(input);
    expect(result.finalAccepted).toBe(false);
    expect(result.blockers.some(b => b.includes('render_ref_bytes_hash_mismatch'))).toBe(true);
  });
});

// ═══════════════════════════════════════════════════════════════════════
// ─── Adversarial: strict-evidence loopholes closed ───────────────────
// ═══════════════════════════════════════════════════════════════════════
//
// The critical review named five classes of loophole that used to slip
// through: fabricated all-pass rubric defaults, arbitrary hash-matching
// render text, artifact-hash tampering, partial or renamed 12-ID caller
// manifests, and unresolved-review-finding evidence.  Each block below
// binds one class as a runtime blocker so a regression is caught here
// rather than at a real run boundary.

import { CANONICAL_FULL_MODE_SECTION_IDS } from './final-verification.js';

describe('runFinalReview — canonical full-mode section IDs', () => {
  it('exports the frozen canonical section-ID list in exact order', () => {
    expect(CANONICAL_FULL_MODE_SECTION_IDS).toEqual([
      'opening',
      'part1', 'part2', 'part3', 'part4', 'part5', 'part6',
      'part7', 'part8', 'part9', 'part10', 'part11',
    ]);
    // Frozen so nothing can hot-patch it at runtime.
    expect(Object.isFrozen(CANONICAL_FULL_MODE_SECTION_IDS)).toBe(true);
  });

  it('blocks a length-12 caller manifest that renames a canonical ID', () => {
    const input = buildFullValidInput();
    // Rename 'part5' to 'part5-renamed' but keep every other position.
    const renamedIds = [...input.requiredSectionIds];
    renamedIds[5] = 'part5-renamed';
    input.requiredSectionIds = renamedIds;
    // Rename corresponding pass/receipt so shape stays consistent.
    input.passes[5] = { ...input.passes[5], id: 'part5-renamed' };
    input.receipts[5] = { ...input.receipts[5], section_id: 'part5-renamed' };
    input.assembled = input.passes.map(p => `## ${p.title}\n\n${p.output}`).join('\n\n');
    const result = runFinalReview(input);
    expect(result.finalAccepted).toBe(false);
    expect(result.blockers.some(b => b.startsWith('sections:canonical_id_mismatch'))).toBe(true);
  });

  it('blocks a length-12 caller manifest that permutes canonical order', () => {
    const input = buildFullValidInput();
    // Swap positions 1 and 2 (part1 <-> part2).
    const permuted = [...input.requiredSectionIds];
    [permuted[1], permuted[2]] = [permuted[2], permuted[1]];
    input.requiredSectionIds = permuted;
    const result = runFinalReview(input);
    expect(result.finalAccepted).toBe(false);
    expect(result.blockers.some(b => b.startsWith('sections:canonical_id_mismatch'))).toBe(true);
  });

  it('blocks a partial 11-section caller manifest even if all present sections are perfect', () => {
    const input = buildFullValidInput();
    input.requiredSectionIds = input.requiredSectionIds.slice(0, 11);
    input.receipts = input.receipts.slice(0, 11);
    input.passes = input.passes.slice(0, 11);
    input.assembled = input.passes.map(p => `## ${p.title}\n\n${p.output}`).join('\n\n');
    const result = runFinalReview(input);
    expect(result.finalAccepted).toBe(false);
    expect(result.blockers).toContain('sections:full_mode_requires_12');
  });
});

describe('runFinalReview — fake rubric adversaries', () => {
  it('blocks a fabricated all-pass rubric that lacks required gate fields', () => {
    const input = buildFullValidInput();
    // Strip every enforced gate on the first pass but keep other fields.
    (input.passes[0] as any).rubric = {
      section_id: 'opening',
      title: 'Title opening',
      target_words: 0, actual_words: 0,
      word_count_fit: 'yes',              // invalid enum
      deterministic_fact_gate: 'yes',     // invalid enum
      integrated_layering_gate: 'ok',     // invalid enum
      guardrail_gate: 'ok',               // invalid enum
      model_requested: 'saved', model_used: 'saved', latency_ms: 0,
    };
    const result = runFinalReview(input);
    expect(result.finalAccepted).toBe(false);
    expect(result.blockers.some(b => b.includes('opening:required_gate_missing:word_count_fit'))).toBe(true);
    expect(result.blockers.some(b => b.includes('opening:required_gate_missing:guardrail_gate'))).toBe(true);
  });

  it('blocks when the rubric fields are absent (fabricated blank default)', () => {
    const input = buildFullValidInput();
    (input.passes[2] as any).rubric = { section_id: 'part2', title: 'Title part2' };
    const result = runFinalReview(input);
    expect(result.finalAccepted).toBe(false);
    expect(result.blockers.some(b => b.includes('part2:required_gate_missing'))).toBe(true);
  });
});

describe('runFinalReview — render evidence tampering', () => {
  it('blocks arbitrary hash-matching text that lacks the strict evidence shape', () => {
    const input = buildFullValidInput();
    // Replace with just a pointer that hashes correctly but omits artifact
    // bytes, bindsAssembledSha256, and the review shape.
    const arbitraryBytes = 'arbitrary text that happens to hash-match this pointer';
    (input.renderReceipt as any) = {
      path: 'render-qa/report.json',
      sha256: h(arbitraryBytes),
      rawBytes: arbitraryBytes,
    };
    const result = runFinalReview(input);
    expect(result.finalAccepted).toBe(false);
    // Strict shape violations must fire.
    expect(result.blockers.some(b => b === 'render_bindsAssembledSha256_invalid')).toBe(true);
    expect(result.blockers.some(b => b === 'render_review_status_not_completed')).toBe(true);
    expect(result.blockers.some(b => b === 'render_review_reviewer_missing')).toBe(true);
  });

  it('verifies binary PDF bytes without UTF-8 decoding', () => {
    const input = buildFullValidInput();
    const bytes = Buffer.from([0x25,0x50,0x44,0x46,0,0xff,0x80,0xfe]);
    input.renderReceipt!.artifact = {...input.renderReceipt!.artifact,
      rawBytes: bytes, sha256:createHash('sha256').update(bytes).digest('hex')};
    expect(runFinalReview(input).finalAccepted).toBe(true);
  });
  it('rejects binary bytes corrupted by UTF-8 replacement characters', () => {
    const input = buildFullValidInput();
    const bytes = Buffer.from([0x25,0x50,0x44,0x46,0,0xff,0x80,0xfe]);
    input.renderReceipt!.artifact = {...input.renderReceipt!.artifact,
      rawBytes: bytes.toString('utf8'), sha256:createHash('sha256').update(bytes).digest('hex')};
    expect(runFinalReview(input).blockers).toContain('render_artifact_bytes_hash_mismatch');
  });

  it('blocks when render artifact bytes hash does not match declared artifact.sha256', () => {
    const input = buildFullValidInput();
    const tampered = 'TAMPERED ARTIFACT BODY';
    input.renderReceipt = {
      ...input.renderReceipt!,
      artifact: {
        ...input.renderReceipt!.artifact,
        rawBytes: tampered, // sha256 still points at the original bytes
      },
    };
    const result = runFinalReview(input);
    expect(result.finalAccepted).toBe(false);
    expect(result.blockers.some(b => b === 'render_artifact_bytes_hash_mismatch')).toBe(true);
  });

  it('blocks when bindsAssembledSha256 does not match sha256(assembled)', () => {
    const input = buildFullValidInput();
    input.renderReceipt = {
      ...input.renderReceipt!,
      bindsAssembledSha256: '0'.repeat(64),
    };
    const result = runFinalReview(input);
    expect(result.finalAccepted).toBe(false);
    expect(result.blockers.some(b => b === 'render_bindsAssembledSha256_mismatch')).toBe(true);
  });

  it('blocks when a render finding is marked unresolved', () => {
    const input = buildFullValidInput();
    input.renderReceipt = {
      ...input.renderReceipt!,
      findings: [
        { id: 'r1', summary: 'font substitution not verified', unresolved: true },
      ],
    };
    const result = runFinalReview(input);
    expect(result.finalAccepted).toBe(false);
    expect(result.blockers.some(b => b.startsWith('render_review_unresolved_findings:r1'))).toBe(true);
  });

  it('blocks when render review status is anything other than "completed"', () => {
    const input = buildFullValidInput();
    input.renderReceipt = { ...input.renderReceipt!, status: 'in-progress' };
    const result = runFinalReview(input);
    expect(result.finalAccepted).toBe(false);
    expect(result.blockers).toContain('render_review_status_not_completed');
  });
});

describe('runFinalReview — parity evidence must bind both languages', () => {
  function withParity(): FinalReviewInput {
    const input = buildFullValidInput();
    input.bilingualExpected = true;
    const enBytes = '%PDF-1.4 EN body';
    const frBytes = '%PDF-1.4 FR body';
    const pointerBytes = JSON.stringify({ compared: 'en+fr' });
    input.parityReceipt = {
      path: 'parity-review.json',
      sha256: h(pointerBytes),
      rawBytes: pointerBytes,
      languages: ['en', 'fr'],
      artifacts: {
        en: { path: 'render-qa/report.en.pdf', sha256: h(enBytes), rawBytes: enBytes, kind: 'application/pdf' },
        fr: { path: 'render-qa/report.fr.pdf', sha256: h(frBytes), rawBytes: frBytes, kind: 'application/pdf' },
      },
      status: 'completed',
      reviewer: 'human',
      reviewedAt: '2026-09-30T12:00:00Z',
      findings: [],
    } as any;
    return input;
  }

  it('accepts when both language artifacts are bytes-bound', () => {
    const input = withParity();
    const result = runFinalReview(input);
    if (!result.finalAccepted) console.error('parity happy path blockers:', result.blockers);
    expect(result.finalAccepted).toBe(true);
  });

  it('blocks when only one language artifact is supplied', () => {
    const input = withParity();
    delete (input.parityReceipt!.artifacts as any).fr;
    input.parityReceipt = { ...input.parityReceipt!, languages: ['en'] };
    const result = runFinalReview(input);
    expect(result.finalAccepted).toBe(false);
    expect(result.blockers).toContain('parity_languages_insufficient');
    expect(result.blockers).toContain('parity_artifacts_insufficient');
  });

  it('blocks when a parity artifact sha does not match its bytes', () => {
    const input = withParity();
    (input.parityReceipt!.artifacts as any).en = {
      ...input.parityReceipt!.artifacts.en,
      sha256: '0'.repeat(64),
    };
    const result = runFinalReview(input);
    expect(result.finalAccepted).toBe(false);
    expect(result.blockers.some(b => b === 'parity_artifact_en_bytes_hash_mismatch')).toBe(true);
  });

  it('blocks when a parity finding is unresolved', () => {
    const input = withParity();
    input.parityReceipt = {
      ...input.parityReceipt!,
      findings: [{ id: 'p1', summary: 'FR section 3.4 shorter than EN', unresolved: true }],
    };
    const result = runFinalReview(input);
    expect(result.finalAccepted).toBe(false);
    expect(result.blockers.some(b => b.startsWith('parity_review_unresolved_findings:p1'))).toBe(true);
  });
});

describe('runFinalReview — coherence review required for every full report', () => {
  it('blocks a clean non-editorial full report when coherence review is absent', () => {
    const input = buildFullValidInput();
    // Strip the coherence review but leave `editorial` present with no revisions.
    input.editorial = { ...input.editorial!, coherenceReview: undefined };
    const result = runFinalReview(input);
    expect(result.finalAccepted).toBe(false);
    expect(result.blockers).toContain('coherence:missing');
  });

  it('blocks when the entire editorial bundle is omitted in full mode', () => {
    const input = buildFullValidInput();
    delete (input as any).editorial;
    const result = runFinalReview(input);
    expect(result.finalAccepted).toBe(false);
    expect(result.blockers).toContain('coherence:missing');
  });

  it('blocks when the coherence review artifactSha256 does not bind assembled bytes', () => {
    const input = buildFullValidInput();
    input.editorial = {
      ...input.editorial!,
      coherenceReview: {
        ...input.editorial!.coherenceReview!,
        artifactSha256: '0'.repeat(64),
      },
    };
    const result = runFinalReview(input);
    expect(result.finalAccepted).toBe(false);
    expect(result.blockers).toContain('coherence:artifactSha256_not_bound_to_assembled');
  });

  it('blocks when the coherence review carries an unresolved finding', () => {
    const input = buildFullValidInput();
    input.editorial = {
      ...input.editorial!,
      coherenceReview: {
        ...input.editorial!.coherenceReview!,
        findings: [
          { id: 'c1', summary: 'section 7 contradicts section 4', unresolved: true },
        ],
      },
    };
    const result = runFinalReview(input);
    expect(result.finalAccepted).toBe(false);
    expect(result.blockers.some(b => b.startsWith('coherence:unresolved_findings:c1'))).toBe(true);
  });

  it('blocks when the coherence review reviewer is neither human nor assistant-source-review', () => {
    const input = buildFullValidInput();
    input.editorial = {
      ...input.editorial!,
      coherenceReview: {
        ...input.editorial!.coherenceReview!,
        reviewer: 'llm-loopback' as any,
      },
    };
    const result = runFinalReview(input);
    expect(result.finalAccepted).toBe(false);
    expect(result.blockers).toContain('coherence:reviewer_invalid');
  });
});

describe('runFinalReview — non-full modes remain nonaccepted', () => {
  it('keeps source-only mode nonaccepted even when every packaging gate is green', () => {
    const input = buildFullValidInput();
    input.mode = 'source-only';
    const result = runFinalReview(input);
    expect(result.finalAccepted).toBe(false);
  });

  it('keeps generation-only mode nonaccepted even when every packaging gate is green', () => {
    const input = buildFullValidInput();
    input.mode = 'generation-only';
    const result = runFinalReview(input);
    expect(result.finalAccepted).toBe(false);
  });
});

describe('primary-source caller exclusion regression', () => {
  it('cannot waive all Cloudflare evidence through caller-only primary IDs', () => {
    const input = buildFullValidInput();
    input.primaryPassageIds = input.receipts.flatMap(r => r.source_ids);
    input.cloudflareSnapshot = { ...input.cloudflareSnapshot!, passages: [] };
    const result = runFinalReview(input);
    expect(result.finalAccepted).toBe(false);
    expect(result.sourceAccepted).toBe(false);
    expect(result.blockers.some(b => b.startsWith('primary:declared_id_'))).toBe(true);
  });
});

// ─── Mixed-source: primary passage registry integration ──────────────
// End-to-end coverage for the new primary_source_ids / primaryPassageIds
// / primaryPassageRegistry contract. Every test mutates the shared
// buildFullValidInput() happy-path fixture in a single, well-scoped way
// so downstream regressions surface immediately.

const PRIMARY_URL = 'https://www.vedicastrologer.org/articles/vedic_astro_textbook.pdf';
const PRIMARY_ID = 'primary:rao2000:house:2';
const PRIMARY_TEXT = 'Rao 2000 primary text on house theme two — cited from PDF chapter 7.2 pages 68–69.';
const PRIMARY_TEXT_SHA = h(PRIMARY_TEXT);

function primaryRegistryBytes(): string {
  const record = {
    id: PRIMARY_ID,
    text: PRIMARY_TEXT,
    sha256: PRIMARY_TEXT_SHA,
    provenance: {
      kind: 'reviewed-primary-document',
      author: 'P. V. R. Narasimha Rao',
      title: 'Vedic Astrology: An Integrated Approach',
      url: PRIMARY_URL,
      locator: 'Chapter 7.2, pages 68–69',
      reviewer: 'assistant-source-review',
    },
  };
  return JSON.stringify({ records: { [PRIMARY_ID]: record }, additionIds: [PRIMARY_ID] });
}

/** Take the happy-path fixture and augment section `opening` with a primary passage. */
function buildValidInputWithPrimary(): FinalReviewInput {
  const input = buildFullValidInput();
  const opening = input.receipts[0];
  // Append the primary id/hash to the opening receipt.
  const combinedIds = [...opening.source_ids, PRIMARY_ID];
  const combinedHashes = [...opening.source_hashes, PRIMARY_TEXT_SHA];
  const combinedTexts = combinedIds.map((id) => id === PRIMARY_ID ? PRIMARY_TEXT : input.sourceTexts![id]);
  const combinedHash = h(combinedTexts.join('\n'));
  // Recompute the source-audit input hash to reflect the new passages_hash
  // (audit binds to acceptedOutput + passagesHash + engineFacts).
  const openingOutput = input.passes[0].output;
  const newAuditInputSha = computeAuditInputHash({
    acceptedOutput: openingOutput,
    passagesHash: combinedHash,
    engineFacts: opening.engine_facts ?? '',
  });
  input.receipts[0] = {
    ...opening,
    source_ids: combinedIds,
    source_hashes: combinedHashes,
    passages_hash: combinedHash,
    primary_source_ids: [PRIMARY_ID],
    cf_source_ids: [...opening.source_ids],
    retrieval: {
      ...opening.retrieval,
      count: combinedIds.length,
      source_ids: combinedIds,
      passages_hash: combinedHash,
    },
    source_audit: opening.source_audit ? {
      ...opening.source_audit,
      input_sha256: newAuditInputSha,
    } : opening.source_audit,
  };
  // Merge the primary text into sourceTexts so the source-audit sourceTexts map covers it too.
  input.sourceTexts = { ...(input.sourceTexts ?? {}), [PRIMARY_ID]: PRIMARY_TEXT };
  // Declare authorised primary ids + registry bytes.
  const regBytes = primaryRegistryBytes();
  input.primaryPassageIds = [PRIMARY_ID];
  input.primaryPassageRegistry = {
    path: 'primary-passage-registry.snapshot.json',
    sha256: h(regBytes),
    rawBytes: regBytes,
  };
  return input;
}

describe('runFinalReview — mixed-source primary passage registry', () => {
  it('accepts a full-mode run when a primary id is appended and the registry bytes hash-bind', () => {
    const input = buildValidInputWithPrimary();
    const result = runFinalReview(input);
    if (!result.finalAccepted) console.error('unexpected primary blockers:', result.blockers);
    expect(result.finalAccepted).toBe(true);
    expect(result.blockers).toEqual([]);
  });

  it('blocks when a receipt declares a primary id that is not in primaryPassageIds (self-asserted claim)', () => {
    const input = buildValidInputWithPrimary();
    input.primaryPassageIds = []; // caller does NOT authorise the id the receipt cites
    const result = runFinalReview(input);
    expect(result.finalAccepted).toBe(false);
    expect(result.blockers.some(b => b === `primary:receipt_id_not_authorised:${PRIMARY_ID}`)).toBe(true);
  });

  it('blocks when the primary registry bytes hash does not match its declared sha256', () => {
    const input = buildValidInputWithPrimary();
    input.primaryPassageRegistry = { ...input.primaryPassageRegistry!, sha256: 'a'.repeat(64) };
    const result = runFinalReview(input);
    expect(result.finalAccepted).toBe(false);
    expect(result.blockers).toContain('primary:registry_bytes_hash_mismatch');
  });

  it('blocks when the primary registry is missing entirely for a receipt-declared primary id', () => {
    const input = buildValidInputWithPrimary();
    delete input.primaryPassageRegistry;
    const result = runFinalReview(input);
    expect(result.finalAccepted).toBe(false);
    expect(result.blockers).toContain('primary:registry_missing');
  });

  it('blocks when the primary text stored in the registry differs from the receipt source_hash', () => {
    const input = buildValidInputWithPrimary();
    const forgedText = PRIMARY_TEXT + ' TAMPERED';
    // The record's inner sha256 must still match its (forged) text for the
    // registry to parse cleanly; otherwise a different blocker fires first.
    const forgedRecord = {
      id: PRIMARY_ID,
      text: forgedText,
      sha256: h(forgedText),
      provenance: {
        kind: 'reviewed-primary-document',
        author: 'P. V. R. Narasimha Rao',
        title: 'Vedic Astrology: An Integrated Approach',
        url: PRIMARY_URL,
        locator: 'Chapter 7.2',
        reviewer: 'assistant-source-review',
      },
    };
    const forgedBytes = JSON.stringify({ records: { [PRIMARY_ID]: forgedRecord }, additionIds: [PRIMARY_ID] });
    input.primaryPassageRegistry = {
      path: 'primary-passage-registry.snapshot.json',
      sha256: h(forgedBytes),
      rawBytes: forgedBytes,
    };
    const result = runFinalReview(input);
    expect(result.finalAccepted).toBe(false);
    // Either the receipt-vs-registry text hash mismatch or a similar drift blocker must fire.
    const opening = input.receipts[0].section_id;
    expect(result.blockers.some(b => b.startsWith(`${opening}:primary_`) && b.includes(PRIMARY_ID))).toBe(true);
  });

  it('blocks when the primary id appears in the Cloudflare snapshot (spoofed CF)', () => {
    const input = buildValidInputWithPrimary();
    input.cloudflareSnapshot!.passages.push({
      id: PRIMARY_ID, text: PRIMARY_TEXT, sha256: PRIMARY_TEXT_SHA,
    });
    const result = runFinalReview(input);
    expect(result.finalAccepted).toBe(false);
    // runFinalReview prefixes CF-side blockers with `cf:`; assert the
    // spoofed-primary-in-CF signal reaches the surface regardless of the
    // exact prefix so downstream string reformatting won't silently break
    // this contract.
    expect(
      result.blockers.some(b => b.includes(`cloudflare_snapshot_spoofed_primary_id:${PRIMARY_ID}`)),
    ).toBe(true);
  });

  it('skips CF-snapshot binding for primary ids: dropping the primary from the snapshot must NOT trip the CF-missing gate', () => {
    const input = buildValidInputWithPrimary();
    // No mutation needed: the primary id was NEVER added to the CF snapshot in
    // buildValidInputWithPrimary(). If verifyCloudflareSnapshot forgot to skip
    // primary ids we would get "cloudflare_snapshot_missing:primary:..." here.
    const result = runFinalReview(input);
    expect(result.blockers.every(b => !b.includes(`cloudflare_snapshot_missing:${PRIMARY_ID}`))).toBe(true);
    expect(result.finalAccepted).toBe(true);
  });

  it('backwards compatibility: CF-only run (no primary fields anywhere) still passes cleanly', () => {
    const input = buildFullValidInput();
    // Sanity: the fixture must not carry primary fields.
    for (const r of input.receipts) {
      expect(r.primary_source_ids).toBeUndefined();
    }
    expect(input.primaryPassageIds).toBeUndefined();
    expect(input.primaryPassageRegistry).toBeUndefined();
    const result = runFinalReview(input);
    if (!result.finalAccepted) console.error('unexpected CF-only blockers:', result.blockers);
    expect(result.finalAccepted).toBe(true);
  });

  it('CF stays mandatory when any non-primary source_id exists: removing the snapshot still blocks', () => {
    const input = buildValidInputWithPrimary();
    delete input.cloudflareSnapshot;
    const result = runFinalReview(input);
    expect(result.finalAccepted).toBe(false);
    expect(result.blockers).toContain('cf:snapshot_missing_for_full_mode');
  });
});
