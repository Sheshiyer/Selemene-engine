// ─── Reference Verification Tests ────────────────────────────────────
// Covers the verifier (reference-verification.ts) against:
//   1. Clean passing receipt — all checks pass
//   2. Corrupted receipt — aletheios hash tampered
//   3. Missing section — required section absent from receipts
//   4. Empty retrieval — state=empty receipt, source_ids empty
//   5. Missing subsection headings — subsectionMap provided but heading absent
//   6. Jev unknown / verdict fail — blockers produced
//   7. Persona hash mismatch — supplied identity text differs from receipt hash
//   8. Synthesis input hash mismatch — rawSynthesisInputs provided but wrong
//   9. Failed outcome receipt skips integrity checks on raw hashes
//  10. Duplicate section IDs in receipts

import { describe, it, expect } from 'vitest';
import { createHash } from 'node:crypto';
import { verifyReferenceExecution } from './reference-verification.js';
import type { ReferenceVerificationInput } from './reference-verification.js';
import type { SectionExecutionReceipt } from './reference-execution.js';
import type { PassResult, SectionRubric } from './integrated.js';

// ─── Helpers ──────────────────────────────────────────────────────────

function sha256(text: string): string {
  return createHash('sha256').update(text, 'utf8').digest('hex');
}

const ALETHEIOS_IDENTITY = '# aletheios IDENTITY\nYou are Aletheios, structural interpreter.';
const PICHET_IDENTITY = '# pichet IDENTITY\nYou are Pichet, experiential reflector.';

const ALETHEIOS_RAW = 'Aletheios structural output: the chart shows Tithi Shukla 3 with Nakshatra Rohini.';
const PICHET_RAW = 'Pichet experiential output: the rhythm is embodied and felt every day.';
const SYNTHESIS_RAW = `## Part I

This section witnesses the structural and experiential patterns together.

The chart holds Tithi Shukla 3 and Nakshatra Rohini (panchanga data).

What remains open: how does the subject orient to cyclic rhythm?`;

const PASSAGE_TEXT = 'Structural pattern notes from corpus.';
const PASSAGE_ID = 'corpus-p1';

function makePassageReceiptSuccess(sectionId: string) {
  return {
    section_id: sectionId,
    state: 'success' as const,
    count: 1,
    source_ids: [PASSAGE_ID],
    passages_hash: sha256(PASSAGE_TEXT),
  };
}

function makeCleanRubric(sectionId: string): SectionRubric {
  return {
    section_id: sectionId,
    title: 'Part I',
    target_words: 80,
    actual_words: 85,
    word_count_fit: 'pass',
    word_count_ratio: 1.0625,
    deterministic_fact_count: 5,
    deterministic_fact_gate: 'pass',
    integrated_layer_count: 3,
    integrated_layering_gate: 'pass',
    guardrail_gate: 'pass',
    guardrail_violations: [],
    model_requested: 'tier-default',
    model_used: 'tier-default',
    latency_ms: 100,
  };
}

function makeCleanReceipt(sectionId: string): SectionExecutionReceipt {
  const aletheiosHash = sha256(ALETHEIOS_RAW);
  const pichetHash = sha256(PICHET_RAW);
  const outputHash = sha256(SYNTHESIS_RAW);
  const synthesisInput = `${ALETHEIOS_RAW}\n\n${PICHET_RAW}`;
  const synthesisInputHash = sha256(synthesisInput);

  return {
    section_id: sectionId,
    outcome: 'ok',
    source_ids: [PASSAGE_ID],
    source_hashes: [sha256(PASSAGE_TEXT)],
    passages_hash: sha256(PASSAGE_TEXT),
    aletheios_hash: aletheiosHash,
    pichet_hash: pichetHash,
    synthesis_input_hash: synthesisInputHash,
    synthesis_input_text: synthesisInput,
    prior_section_content_hashes: {},
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
    raw: {
      aletheios: ALETHEIOS_RAW,
      pichet: PICHET_RAW,
      synthesis: SYNTHESIS_RAW,
    },
    jev: { pass_id: sectionId, mode: 'shadow', status: 'judged', model: 'fixture', latency_ms: 1, verdicts: { guardrail: 'pass', framing: 'pass', grounding: 'pass', register: 'pass' }, blocked: false, disagreements: [] },
    retrieval: makePassageReceiptSuccess(sectionId),
    attempts: [{ attempt: 1, outcome: 'ok', aletheios_raw: ALETHEIOS_RAW, pichet_raw: PICHET_RAW, synthesis_raw: SYNTHESIS_RAW }],
  };
}

function makeCleanPass(sectionId: string): PassResult {
  return {
    id: sectionId,
    title: 'Part I',
    output: SYNTHESIS_RAW,
    rubric: makeCleanRubric(sectionId),
  };
}

function makeBaseInput(overrides: Partial<ReferenceVerificationInput> = {}): ReferenceVerificationInput {
  const sectionId = 'part1';
  return {
    receipts: [makeCleanReceipt(sectionId)],
    passes: [makeCleanPass(sectionId)],
    requiredSectionIds: [sectionId],
    expectedPersonas: {
      aletheios: ALETHEIOS_IDENTITY,
      pichet: PICHET_IDENTITY,
    },
    sourceTexts: { [PASSAGE_ID]: PASSAGE_TEXT },
    subsectionMap: { part1: [] },
    rawSynthesisInputs: { part1: `${ALETHEIOS_RAW}\n\n${PICHET_RAW}` },
    ...overrides,
  };
}

// ─── Test 1: Clean passing receipt ────────────────────────────────────

describe('reference-verification: clean receipt', () => {
  it('passes with no blockers when all fields are consistent', () => {
    const result = verifyReferenceExecution(makeBaseInput());
    expect(result.passed).toBe(true);
    expect(result.blockers).toHaveLength(0);
    expect(result.artifactSha256).toHaveLength(64);
    expect(result.artifactSha256).toMatch(/^[0-9a-f]{64}$/);
  });

  it('returns a stable artifactSha256 for identical inputs', () => {
    const a = verifyReferenceExecution(makeBaseInput());
    const b = verifyReferenceExecution(makeBaseInput());
    expect(a.artifactSha256).toBe(b.artifactSha256);
  });

  it('accepts the canonical assembled artifact without duplicating pass titles', () => {
    const result = verifyReferenceExecution(makeBaseInput({ assembled: `${SYNTHESIS_RAW}\n` }));
    expect(result.blockers).not.toContain('artifact:assembled_content_mismatch');
  });
});

// ─── Test 2: Corrupted receipt ────────────────────────────────────────

describe('reference-verification: corrupted receipt', () => {
  it('blocks when aletheios_hash does not match sha256(raw.aletheios)', () => {
    const receipt = makeCleanReceipt('part1');
    receipt.aletheios_hash = sha256('tampered aletheios output');
    const result = verifyReferenceExecution(makeBaseInput({ receipts: [receipt] }));
    expect(result.passed).toBe(false);
    expect(result.blockers.some((b) => b.includes('aletheios_hash_mismatch'))).toBe(true);
  });

  it('blocks when pichet_hash does not match sha256(raw.pichet)', () => {
    const receipt = makeCleanReceipt('part1');
    receipt.pichet_hash = sha256('tampered pichet output');
    const result = verifyReferenceExecution(makeBaseInput({ receipts: [receipt] }));
    expect(result.passed).toBe(false);
    expect(result.blockers.some((b) => b.includes('pichet_hash_mismatch'))).toBe(true);
  });

  it('blocks when output_hash does not match sha256(raw.synthesis)', () => {
    const receipt = makeCleanReceipt('part1');
    receipt.output_hash = sha256('tampered synthesis output');
    const result = verifyReferenceExecution(makeBaseInput({ receipts: [receipt] }));
    expect(result.passed).toBe(false);
    expect(result.blockers.some((b) => b.includes('synthesis_output_hash_mismatch'))).toBe(true);
  });

  it('blocks when pass output does not match receipt output_hash', () => {
    const pass = makeCleanPass('part1');
    pass.output = 'Tampered pass output that does not match the receipt hash.';
    const result = verifyReferenceExecution(makeBaseInput({ passes: [pass] }));
    expect(result.passed).toBe(false);
    expect(result.blockers.some((b) => b.includes('pass_output_hash_mismatch'))).toBe(true);
  });

  it('blocks when raw.aletheios is empty', () => {
    const receipt = makeCleanReceipt('part1');
    receipt.raw = { ...receipt.raw, aletheios: '' };
    const result = verifyReferenceExecution(makeBaseInput({ receipts: [receipt] }));
    expect(result.passed).toBe(false);
    expect(result.blockers.some((b) => b.includes('raw_aletheios_empty'))).toBe(true);
  });

  it('blocks when raw.pichet is empty', () => {
    const receipt = makeCleanReceipt('part1');
    receipt.raw = { ...receipt.raw, pichet: '' };
    const result = verifyReferenceExecution(makeBaseInput({ receipts: [receipt] }));
    expect(result.passed).toBe(false);
    expect(result.blockers.some((b) => b.includes('raw_pichet_empty'))).toBe(true);
  });
});

// ─── Test 3: Missing section ──────────────────────────────────────────

describe('reference-verification: missing section', () => {
  it('blocks when a required section is absent from receipts', () => {
    const result = verifyReferenceExecution(
      makeBaseInput({ requiredSectionIds: ['part1', 'part2'] }),
    );
    expect(result.passed).toBe(false);
    expect(result.blockers.some((b) => b.includes('missing_required_section:part2'))).toBe(true);
  });

  it('blocks when sections are out of declared order', () => {
    const receipt1 = makeCleanReceipt('part1');
    const receipt2 = makeCleanReceipt('part2');
    // Receipts provided in wrong order
    const result = verifyReferenceExecution({
      receipts: [receipt2, receipt1],
      passes: [makeCleanPass('part1'), makeCleanPass('part2')],
      requiredSectionIds: ['part1', 'part2'],
      expectedPersonas: { aletheios: ALETHEIOS_IDENTITY, pichet: PICHET_IDENTITY },
    });
    expect(result.passed).toBe(false);
    expect(result.blockers.some((b) => b.includes('section_order_mismatch'))).toBe(true);
  });
});

// ─── Test 4: Empty retrieval ──────────────────────────────────────────

describe('reference-verification: empty retrieval', () => {
  it('does not block on empty retrieval state per se (that is an execution concern), but blocks on success with zero count', () => {
    // A receipt with retrieval.state='success' but count=0 is a structural contradiction
    const receipt = makeCleanReceipt('part1');
    receipt.retrieval = { ...receipt.retrieval, state: 'success', count: 0, source_ids: [] };
    receipt.source_ids = [];
    const result = verifyReferenceExecution(makeBaseInput({ receipts: [receipt] }));
    expect(result.passed).toBe(false);
    expect(result.blockers.some((b) => b.includes('retrieval_success_zero_count'))).toBe(true);
    expect(result.blockers.some((b) => b.includes('retrieval_success_empty_source_ids'))).toBe(true);
  });

  it('blocks when retrieval success source_hashes array is empty', () => {
    const receipt = makeCleanReceipt('part1');
    receipt.source_hashes = [];
    const result = verifyReferenceExecution(makeBaseInput({ receipts: [receipt] }));
    expect(result.passed).toBe(false);
    expect(result.blockers.some((b) => b.includes('retrieval_success_empty_source_hashes'))).toBe(true);
  });

  it('blocks when retrieval success source_hashes length mismatches source_ids length', () => {
    const receipt = makeCleanReceipt('part1');
    receipt.source_hashes = [sha256('a'), sha256('b')]; // 2 hashes but only 1 source_id
    const result = verifyReferenceExecution(makeBaseInput({ receipts: [receipt] }));
    expect(result.passed).toBe(false);
    expect(result.blockers.some((b) => b.includes('retrieval_hash_count_mismatch'))).toBe(true);
  });

  it('blocks when retrieval success passages_hash is empty', () => {
    const receipt = makeCleanReceipt('part1');
    receipt.passages_hash = '';
    const result = verifyReferenceExecution(makeBaseInput({ receipts: [receipt] }));
    expect(result.passed).toBe(false);
    expect(result.blockers.some((b) => b.includes('retrieval_success_empty_passages_hash'))).toBe(true);
  });

  it('accepts a failed receipt (outcome=failed) with empty retrieval state without hash checks', () => {
    const receipt = makeCleanReceipt('part1');
    receipt.outcome = 'failed';
    receipt.raw = { aletheios: '', pichet: '', synthesis: '' };
    receipt.aletheios_hash = '';
    receipt.pichet_hash = '';
    receipt.output_hash = '';
    receipt.synthesis_input_hash = '';
    receipt.source_ids = [];
    receipt.source_hashes = [];
    receipt.passages_hash = '';
    receipt.retrieval = { section_id: 'part1', state: 'empty', count: 0, source_ids: [], passages_hash: '' };

    const pass = makeCleanPass('part1');
    pass.output = '';

    const result = verifyReferenceExecution(makeBaseInput({ receipts: [receipt], passes: [pass] }));
    // Failed receipts skip raw/hash integrity — should NOT produce hash-related blockers
    expect(result.blockers.every((b) => !b.includes('aletheios_hash_mismatch'))).toBe(true);
    expect(result.blockers.every((b) => !b.includes('pichet_hash_mismatch'))).toBe(true);
    expect(result.blockers.every((b) => !b.includes('raw_aletheios_empty'))).toBe(true);
    expect(result.blockers.every((b) => !b.includes('raw_pichet_empty'))).toBe(true);
  });
});

// ─── Test 5: Missing subsection headings ─────────────────────────────

describe('reference-verification: subsection heading validation', () => {
  it('blocks when a required subsection is missing from accepted output', () => {
    const pass = makeCleanPass('part1');
    // Output does not contain "2.1" heading
    pass.output = '## Introduction\nContent without required subsections.';

    const result = verifyReferenceExecution(
      makeBaseInput({
        passes: [pass],
        subsectionMap: { part1: ['2.1', '2.2'] },
      }),
    );
    expect(result.passed).toBe(false);
    expect(result.blockers.some((b) => b.includes('missing_required_subsection:2.1'))).toBe(true);
    expect(result.blockers.some((b) => b.includes('missing_required_subsection:2.2'))).toBe(true);
  });

  it('passes when all required subsection headings are present', () => {
    const pass = makeCleanPass('part1');
    // Output with both required subsection headings (must match receipt output hash)
    const outputWithSubs = `## 2.1 First Subsection\n\nContent for first subsection.\n\n## 2.2 Second Subsection\n\nContent for second subsection.`;
    pass.output = outputWithSubs;
    // Rebuild receipt with consistent hashes for this output
    const receipt = makeCleanReceipt('part1');
    receipt.raw.synthesis = outputWithSubs;
    receipt.output_hash = sha256(outputWithSubs);

    const result = verifyReferenceExecution(
      makeBaseInput({
        receipts: [receipt],
        passes: [pass],
        subsectionMap: { part1: ['2.1', '2.2'] },
      }),
    );
    const subBlockers = result.blockers.filter((b) => b.includes('missing_required_subsection'));
    expect(subBlockers).toHaveLength(0);
  });

  it('does not false-positive on numbers inside larger IDs (e.g. 12.1 ≠ 2.1)', () => {
    const pass = makeCleanPass('part1');
    const outputWithWrongSub = '## 12.1 Something Else\n\nContent here.';
    pass.output = outputWithWrongSub;
    const receipt = makeCleanReceipt('part1');
    receipt.raw.synthesis = outputWithWrongSub;
    receipt.output_hash = sha256(outputWithWrongSub);

    const result = verifyReferenceExecution(
      makeBaseInput({
        receipts: [receipt],
        passes: [pass],
        subsectionMap: { part1: ['2.1'] },
      }),
    );
    expect(result.blockers.some((b) => b.includes('missing_required_subsection:2.1'))).toBe(true);
  });
});

// ─── Test 6: Jev verdict failures ────────────────────────────────────

describe('reference-verification: Jev verdict handling', () => {
  it('blocks when Jev verdict is "fail" for guardrail', () => {
    const receipt = makeCleanReceipt('part1');
    receipt.jev = {
      pass_id: 'part1',
      mode: 'active',
      status: 'judged',
      blocked: true,
      disagreements: [],
      verdicts: {
        guardrail: 'fail',
        framing: 'pass',
        grounding: 'pass',
        register: 'pass',
      },
      answers: {
        guardrail_clean: 0.1,
        relationship_framing_ok: 0.9,
        fact_grounding: { score: 3, confidence: 0.9, legend: {} },
        register_fit: { choice: 'l4_l5', confidence: 0.9 },
      },
    };
    const result = verifyReferenceExecution(makeBaseInput({ receipts: [receipt] }));
    expect(result.passed).toBe(false);
    expect(result.blockers.some((b) => b.includes('jev_verdict_fail:guardrail'))).toBe(true);
    expect(result.blockers.some((b) => b.includes('jev_blocked'))).toBe(true);
  });

  it('warns but does not block when Jev verdict is "could-not-tell"', () => {
    const receipt = makeCleanReceipt('part1');
    receipt.jev = {
      pass_id: 'part1',
      mode: 'shadow',
      status: 'judged',
      blocked: false,
      disagreements: [],
      verdicts: {
        guardrail: 'pass',
        framing: 'could-not-tell',
        grounding: 'pass',
        register: 'pass',
      },
      answers: {
        guardrail_clean: 0.9,
        relationship_framing_ok: 0.4,
        fact_grounding: { score: 3, confidence: 0.9, legend: {} },
        register_fit: { choice: 'l4_l5', confidence: 0.9 },
      },
    };
    const result = verifyReferenceExecution(makeBaseInput({ receipts: [receipt] }));
    const jevUnresolved = result.warnings.some((w) => w.includes('jev_verdict_unresolved:framing'));
    expect(jevUnresolved).toBe(true);
    expect(result.blockers.some((b) => b.includes('jev_verdict_fail'))).toBe(false);
  });

  it('blocks when Jev receipt is absent', () => {
    const receipt = makeCleanReceipt('part1');
    receipt.jev = undefined;
    const result = verifyReferenceExecution(makeBaseInput({ receipts: [receipt] }));
    expect(result.warnings.some((w) => w.includes('jev_not_configured'))).toBe(true);
    expect(result.blockers.some((b) => b.includes('jev'))).toBe(true);
  });

  it('blocks when Jev status is skipped (client not configured)', () => {
    const receipt = makeCleanReceipt('part1');
    receipt.jev = {
      pass_id: 'part1',
      mode: 'shadow',
      status: 'skipped',
      reason: 'TYPESAFE_API_KEY not set',
      blocked: false,
      disagreements: [],
    };
    const result = verifyReferenceExecution(makeBaseInput({ receipts: [receipt] }));
    expect(result.warnings.some((w) => w.includes('jev_skipped'))).toBe(true);
    expect(result.blockers.some((b) => b.includes('jev'))).toBe(true);
  });
});

// ─── Test 7: Persona hash mismatch ───────────────────────────────────

describe('reference-verification: persona hash verification', () => {
  it('blocks when supplied aletheios identity text does not match receipt sourceHash', () => {
    const result = verifyReferenceExecution(
      makeBaseInput({
        expectedPersonas: {
          aletheios: 'Wrong identity text for aletheios that was never used.',
          pichet: PICHET_IDENTITY,
        },
      }),
    );
    expect(result.passed).toBe(false);
    expect(result.blockers.some((b) => b.includes('persona_hash_mismatch:aletheios'))).toBe(true);
  });

  it('blocks when supplied pichet identity text does not match receipt sourceHash', () => {
    const result = verifyReferenceExecution(
      makeBaseInput({
        expectedPersonas: {
          aletheios: ALETHEIOS_IDENTITY,
          pichet: 'Wrong pichet identity text that was never loaded.',
        },
      }),
    );
    expect(result.passed).toBe(false);
    expect(result.blockers.some((b) => b.includes('persona_hash_mismatch:pichet'))).toBe(true);
  });

  it('warns (not blocks) when a persona is not supplied for verification', () => {
    const result = verifyReferenceExecution(
      makeBaseInput({
        expectedPersonas: {
          // aletheios is missing — should warn but not block
          pichet: PICHET_IDENTITY,
        },
      }),
    );
    expect(result.warnings.some((w) => w.includes('persona_not_supplied:aletheios'))).toBe(true);
    expect(result.blockers.some((b) => b.includes('persona_hash_mismatch:aletheios'))).toBe(false);
  });
});

// ─── Test 8: Synthesis input hash mismatch ───────────────────────────

describe('reference-verification: synthesis prior receipt integrity', () => {
  it('blocks when rawSynthesisInput hash does not match synthesis_input_hash', () => {
    const synthesisInput = `${ALETHEIOS_RAW}\n\n${PICHET_RAW}`;
    const receipt = makeCleanReceipt('part1');
    receipt.synthesis_input_hash = sha256(synthesisInput);

    const result = verifyReferenceExecution(
      makeBaseInput({
        receipts: [receipt],
        rawSynthesisInputs: { part1: 'Completely different synthesis input text that was not used.' },
      }),
    );
    expect(result.passed).toBe(false);
    expect(result.blockers.some((b) => b.includes('synthesis_input_hash_mismatch'))).toBe(true);
  });

  it('passes when rawSynthesisInput hash matches synthesis_input_hash', () => {
    const synthesisInput = `${ALETHEIOS_RAW}\n\n${PICHET_RAW}`;
    const receipt = makeCleanReceipt('part1');
    receipt.synthesis_input_hash = sha256(synthesisInput);

    const result = verifyReferenceExecution(
      makeBaseInput({
        receipts: [receipt],
        rawSynthesisInputs: { part1: synthesisInput },
      }),
    );
    expect(result.blockers.some((b) => b.includes('synthesis_input_hash_mismatch'))).toBe(false);
  });

  it('skips synthesis input hash check when rawSynthesisInputs not provided', () => {
    const result = verifyReferenceExecution(makeBaseInput());
    expect(result.blockers.some((b) => b.includes('synthesis_input_hash_mismatch'))).toBe(false);
  });
});

// ─── Test 9: Failed outcome receipt — skips hash integrity checks ─────

describe('reference-verification: failed outcome receipt', () => {
  it('does not produce hash-mismatch blockers for outcome=failed receipts', () => {
    const receipt = makeCleanReceipt('part1');
    receipt.outcome = 'failed';
    receipt.raw = { aletheios: '', pichet: '', synthesis: '' };
    receipt.aletheios_hash = '';
    receipt.pichet_hash = '';
    receipt.output_hash = '';
    receipt.synthesis_input_hash = '';

    const pass = makeCleanPass('part1');
    pass.output = '';

    const result = verifyReferenceExecution(makeBaseInput({ receipts: [receipt], passes: [pass] }));
    const hashBlockers = result.blockers.filter((b) =>
      b.includes('hash_mismatch') || b.includes('raw_aletheios_empty') || b.includes('raw_pichet_empty') || b.includes('raw_synthesis_empty'),
    );
    expect(hashBlockers).toHaveLength(0);
  });
});

// ─── Test 10: Duplicate section IDs ──────────────────────────────────

describe('reference-verification: duplicate section IDs', () => {
  it('blocks on duplicate receipt section IDs', () => {
    const receipt1 = makeCleanReceipt('part1');
    const receipt2 = makeCleanReceipt('part1'); // duplicate
    const result = verifyReferenceExecution(
      makeBaseInput({
        receipts: [receipt1, receipt2],
        passes: [makeCleanPass('part1')],
        requiredSectionIds: ['part1'],
      }),
    );
    expect(result.passed).toBe(false);
    expect(result.blockers.some((b) => b.includes('receipt:duplicate_section_id:part1'))).toBe(true);
  });
});

// ─── Test 11: Rubric gate failures ───────────────────────────────────

describe('reference-verification: rubric gate failures', () => {
  it('blocks on word_count_fit=fail', () => {
    const pass = makeCleanPass('part1');
    pass.rubric = { ...makeCleanRubric('part1'), word_count_fit: 'fail', actual_words: 10 };
    const result = verifyReferenceExecution(makeBaseInput({ passes: [pass] }));
    expect(result.blockers.some((b) => b.includes('rubric:word_count_fail'))).toBe(true);
  });

  it('warns on word_count_fit=warn', () => {
    const pass = makeCleanPass('part1');
    pass.rubric = { ...makeCleanRubric('part1'), word_count_fit: 'warn', actual_words: 60 };
    const result = verifyReferenceExecution(makeBaseInput({ passes: [pass] }));
    expect(result.warnings.some((w) => w.includes('rubric:word_count_warn'))).toBe(true);
    expect(result.blockers.some((b) => b.includes('word_count_fail'))).toBe(false);
  });

  it('blocks on guardrail_gate=fail', () => {
    const pass = makeCleanPass('part1');
    pass.rubric = { ...makeCleanRubric('part1'), guardrail_gate: 'fail', guardrail_violations: ['guarantee'] };
    const result = verifyReferenceExecution(makeBaseInput({ passes: [pass] }));
    expect(result.blockers.some((b) => b.includes('rubric:guardrail_fail'))).toBe(true);
  });

  it('blocks on deterministic_fact_gate=fail', () => {
    const pass = makeCleanPass('part1');
    pass.rubric = { ...makeCleanRubric('part1'), deterministic_fact_gate: 'fail', deterministic_fact_count: 0 };
    const result = verifyReferenceExecution(makeBaseInput({ passes: [pass] }));
    expect(result.blockers.some((b) => b.includes('rubric:deterministic_fact_fail'))).toBe(true);
  });

  it('blocks on integrated_layering_gate=fail', () => {
    const pass = makeCleanPass('part1');
    pass.rubric = { ...makeCleanRubric('part1'), integrated_layering_gate: 'fail', integrated_layer_count: 0 };
    const result = verifyReferenceExecution(makeBaseInput({ passes: [pass] }));
    expect(result.blockers.some((b) => b.includes('rubric:integrated_layering_fail'))).toBe(true);
  });
});

// ─── Test 12: Multi-section artifact SHA-256 ─────────────────────────

describe('reference-verification: artifact SHA-256', () => {
  it('produces a deterministic artifact SHA-256 over all accepted output hashes in order', () => {
    const receipt1 = makeCleanReceipt('part1');
    const receipt2 = makeCleanReceipt('part2');
    // Give part2 different content so output hashes differ
    const part2Output = 'Part 2 structural and experiential synthesis output with Nakshatra Ashwini.';
    receipt2.raw.synthesis = part2Output;
    receipt2.output_hash = sha256(part2Output);

    const pass2 = makeCleanPass('part2');
    pass2.output = part2Output;

    const result = verifyReferenceExecution({
      receipts: [receipt1, receipt2],
      passes: [makeCleanPass('part1'), pass2],
      requiredSectionIds: ['part1', 'part2'],
      expectedPersonas: { aletheios: ALETHEIOS_IDENTITY, pichet: PICHET_IDENTITY },
    });

    expect(result.artifactSha256).toHaveLength(64);
    expect(result.artifactSha256).toMatch(/^[0-9a-f]{64}$/);

    // Artifact should differ from single-section artifact
    const singleResult = verifyReferenceExecution(makeBaseInput());
    expect(result.artifactSha256).not.toBe(singleResult.artifactSha256);
  });

  it('returns empty-string sha256 when all receipts are failed (no accepted outputs)', () => {
    const receipt = makeCleanReceipt('part1');
    receipt.outcome = 'failed';
    receipt.output_hash = '';
    receipt.raw = { aletheios: '', pichet: '', synthesis: '' };
    const pass = makeCleanPass('part1');
    pass.output = '';

    const result = verifyReferenceExecution(makeBaseInput({ receipts: [receipt], passes: [pass] }));
    // sha256('') is the empty-string digest
    expect(result.artifactSha256).toBe(sha256(''));
  });
});


describe('reference-verification: strict final boundaries', () => {
  it('cannot accept an empty manifest and empty output', () => {
    expect(verifyReferenceExecution(makeBaseInput({ receipts: [], passes: [], requiredSectionIds: [] })).passed).toBe(false);
  });
  it('does not accept an artifact edited after its section receipts', () => {
    const result = verifyReferenceExecution(makeBaseInput({ assembled: 'Different final prose' }));
    expect(result.blockers).toContain('artifact:assembled_content_mismatch');
  });
  it('requires actual source text and a full synthesis input', () => {
    const result = verifyReferenceExecution(makeBaseInput({ sourceTexts: {}, rawSynthesisInputs: {} }));
    expect(result.passed).toBe(false);
    expect(result.blockers.some(x => x.includes('source_text_unverified'))).toBe(true);
    expect(result.blockers).toContain('part1:synthesis_input_missing');
  });
});


// ─── Source-audit gate (real reference route) ─────────────────────────

import { computeAuditInputHash } from './reference-source-audit.js';
import type { SourceAuditReceipt } from './reference-source-audit.js';

function makeCleanAudit(
  sectionId: string,
  overrides: Partial<SourceAuditReceipt> = {},
): SourceAuditReceipt {
  const engineFacts = 'Panchanga: Tithi Shukla 3, Nakshatra Rohini.';
  const modelRaw = JSON.stringify({
    status: 'clean',
    coverage: { total_claims_audited: 1, sources_referenced: 1 },
    claim_checks: [{ claim_quoted: SYNTHESIS_RAW, source_reference: { source_path_or_passage_id: 'panchanga.result.tithi', supporting_values: 'Shukla 3' } }],
    findings: [],
  });
  const inputHash = computeAuditInputHash({
    acceptedOutput: SYNTHESIS_RAW,
    passagesHash: sha256(PASSAGE_TEXT),
    engineFacts,
  });
  return {
    section_id: sectionId,
    status: 'clean',
    model_raw: modelRaw,
    input_sha256: inputHash,
    output_sha256: sha256(modelRaw),
    coverage: { total_claims_audited: 1, sources_referenced: 1 },
    claim_checks: [{ claim_quoted: SYNTHESIS_RAW, source_reference: { source_path_or_passage_id: 'panchanga.result.tithi', supporting_values: 'Shukla 3' } }],
    findings: [],
    ...overrides,
  };
}

function makeInputWithAudit(
  overrides: Partial<ReferenceVerificationInput> = {},
): ReferenceVerificationInput {
  const engineFacts = 'Panchanga: Tithi Shukla 3, Nakshatra Rohini.';
  const receipt = makeCleanReceipt('part1');
  receipt.source_audit = makeCleanAudit('part1');
  return makeBaseInput({
    receipts: [receipt],
    requireSourceAudit: true,
    sectionEngineFacts: { part1: engineFacts },
    ...overrides,
  });
}

describe('reference-verification: source-audit gate', () => {
  it('passes when a clean, hash-bound audit is present', () => {
    const result = verifyReferenceExecution(makeInputWithAudit());
    expect(result.passed).toBe(true);
    expect(result.blockers).toHaveLength(0);
  });

  it('blocks when audit is missing and requireSourceAudit=true', () => {
    const receipt = makeCleanReceipt('part1');
    // no source_audit attached
    const result = verifyReferenceExecution(
      makeBaseInput({
        receipts: [receipt],
        requireSourceAudit: true,
        sectionEngineFacts: { part1: 'engine facts' },
      }),
    );
    expect(result.passed).toBe(false);
    expect(result.blockers.some((b) => b.includes('source_audit_missing'))).toBe(true);
  });

  it('skips audit checks when requireSourceAudit is false or omitted', () => {
    const receipt = makeCleanReceipt('part1');
    // No source_audit and no requireSourceAudit — legacy routes must still pass
    const result = verifyReferenceExecution(makeBaseInput({ receipts: [receipt] }));
    expect(result.passed).toBe(true);
    expect(result.blockers.filter((b) => b.includes('source_audit'))).toHaveLength(0);
  });

  it('blocks when audit status is not clean', () => {
    const receipt = makeCleanReceipt('part1');
    receipt.source_audit = makeCleanAudit('part1', {
      status: 'changes-required',
      reason: 'error findings remain after repair',
      findings: [
        {
          id: 'f1',
          severity: 'error',
          claim_quoted: 'wrong grouping',
          source_reference: { source_path_or_passage_id: 'engine.json' },
          analysis: 'mismatched',
          required_correction: 'fix',
        },
      ],
    });
    const result = verifyReferenceExecution(
      makeBaseInput({
        receipts: [receipt],
        requireSourceAudit: true,
        sectionEngineFacts: { part1: 'engine facts' },
      }),
    );
    expect(result.passed).toBe(false);
    expect(
      result.blockers.some((b) => b.includes('source_audit_status_changes-required')),
    ).toBe(true);
  });

  it('blocks when the audit output hash is tampered', () => {
    const receipt = makeCleanReceipt('part1');
    const audit = makeCleanAudit('part1');
    audit.output_sha256 = sha256('tampered raw audit output');
    receipt.source_audit = audit;
    const result = verifyReferenceExecution(
      makeInputWithAudit({ receipts: [receipt] }),
    );
    expect(result.passed).toBe(false);
    expect(
      result.blockers.some((b) => b.includes('source_audit_output_hash_mismatch')),
    ).toBe(true);
  });

  it('blocks when the audit input hash disagrees with engine facts', () => {
    const receipt = makeCleanReceipt('part1');
    receipt.source_audit = makeCleanAudit('part1'); // hash bound to original engineFacts
    // Supply *different* engine facts to the verifier
    const result = verifyReferenceExecution(
      makeBaseInput({
        receipts: [receipt],
        requireSourceAudit: true,
        sectionEngineFacts: { part1: 'DIFFERENT engine facts — swapped after audit' },
      }),
    );
    expect(result.passed).toBe(false);
    expect(
      result.blockers.some((b) => b.includes('source_audit_input_hash_mismatch')),
    ).toBe(true);
  });

  it('blocks a clean audit with zero coverage', () => {
    const receipt = makeCleanReceipt('part1');
    const audit = makeCleanAudit('part1', {
      coverage: { total_claims_audited: 0, sources_referenced: 0 },
    });
    receipt.source_audit = audit;
    const result = verifyReferenceExecution(makeInputWithAudit({ receipts: [receipt] }));
    expect(result.passed).toBe(false);
    expect(result.blockers.some((b) => b.includes('source_audit_zero_coverage'))).toBe(true);
  });

  it('does not enforce audit for failed-outcome receipts even with requireSourceAudit', () => {
    const receipt = makeCleanReceipt('part1');
    receipt.outcome = 'failed';
    receipt.output_hash = '';
    receipt.raw = { aletheios: '', pichet: '', synthesis: '' };
    const pass = makeCleanPass('part1');
    pass.output = '';
    const result = verifyReferenceExecution(
      makeBaseInput({
        receipts: [receipt],
        passes: [pass],
        requireSourceAudit: true,
        sectionEngineFacts: { part1: 'facts' },
      }),
    );
    // failed receipts are handled by the earlier failure gate; audit checks
    // must not add spurious blockers here.
    expect(result.blockers.filter((b) => b.includes('source_audit'))).toHaveLength(0);
  });
});
