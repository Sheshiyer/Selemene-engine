import { describe, it, expect } from 'vitest';
import { createHash } from 'node:crypto';
import {
  applyEditorialRevision,
  EditorialRevisionError,
  type ApplyEditorialRevisionInput,
  type EditorialPatch,
  type EditorialRevision,
} from './editorial-revision.js';
import type { SectionExecutionReceipt } from './reference-execution.js';

// ─── helpers ──────────────────────────────────────────────────────────

const sha256 = (s: string) => createHash('sha256').update(s, 'utf8').digest('hex');

function makeReceipt(sectionId: string, output: string): SectionExecutionReceipt {
  const outputHash = sha256(output);
  const personaA = { name: 'Aletheios', sourcePath: '/dev/null/a', sourceHash: sha256('A') };
  const personaP = { name: 'Pichet', sourcePath: '/dev/null/p', sourceHash: sha256('P') };
  return {
    section_id: sectionId,
    outcome: 'ok',
    source_ids: ['src-1'],
    source_hashes: [sha256('passage-1')],
    passages_hash: sha256('passage-1'),
    aletheios_hash: sha256('a-out'),
    pichet_hash: sha256('p-out'),
    synthesis_input_hash: sha256('syn-in'),
    synthesis_input_text: 'syn-in',
    prior_section_content_hashes: {},
    output_hash: outputHash,
    aletheios_persona: personaA,
    pichet_persona: personaP,
    raw: { aletheios: 'a-out', pichet: 'p-out', synthesis: output },
    retrieval: {
      section_id: sectionId,
      state: 'success',
      count: 1,
      source_ids: ['src-1'],
      passages_hash: sha256('passage-1'),
    },
    attempts: [],
  };
}

function makeInput(overrides: Partial<ApplyEditorialRevisionInput> = {}): ApplyEditorialRevisionInput {
  const output = 'Saturn transits the natal ascendant during 2024.';
  const receipt = makeReceipt('part3', output);
  const patch: EditorialPatch = {
    before: '2024',
    after: '2025',
    rationale: 'Ephemeris in Swiss Ephemeris 2.10 places the transit in 2025, not 2024.',
    sourceReferences: [
      { sourceId: 'swiss-ephemeris-2.10', quote: 'Saturn ingress into natal Asc window: 2025-03-14 → 2025-11-02' },
    ],
  };
  return {
    baseReceipt: receipt,
    currentOutput: output,
    expectedBaseOutputHash: receipt.output_hash,
    patches: [patch],
    reviewer: 'assistant-source-review',
    timestamp: '2026-09-30T12:00:00Z',
    ...overrides,
  };
}

// ─── happy path ───────────────────────────────────────────────────────

describe('applyEditorialRevision — valid exact correction', () => {
  it('rejects a receipt with no output hash', () => {
    const input = makeInput();
    input.baseReceipt.output_hash = '';
    expect(() => applyEditorialRevision(input)).toThrow('baseReceipt.output_hash');
  });

  it('rejects overlapping occurrences of a single before string', () => {
    const input = makeInput({ currentOutput: 'aaa', baseReceipt: makeReceipt('opening', 'aaa'), expectedBaseOutputHash: sha256('aaa') });
    input.patches = [{ ...input.patches[0], before: 'aa', after: 'b' }];
    expect(() => applyEditorialRevision(input)).toThrow('must be unique');
  });

  it('produces a new immutable revision with revised output, changed hash, and pending reviews', () => {
    const input = makeInput();
    const rev = applyEditorialRevision(input);

    expect(rev.sectionId).toBe('part3');
    expect(rev.baseOutputHash).toBe(input.baseReceipt.output_hash);
    expect(rev.revisedOutput).toBe('Saturn transits the natal ascendant during 2025.');
    expect(rev.revisedOutputHash).toBe(sha256(rev.revisedOutput));
    expect(rev.revisedOutputHash).not.toBe(rev.baseOutputHash);
    expect(rev.patches).toHaveLength(1);
    expect(rev.reviewer).toBe('assistant-source-review');
    expect(rev.timestamp).toBe('2026-09-30T12:00:00Z');
    expect(rev.pendingReviews).toEqual({
      sourceAudit: 'pending',
      jev: 'pending',
      document: 'pending',
      coherence: 'pending',
    });
  });

  it('applies multiple non-overlapping patches in deterministic order regardless of caller order', () => {
    const output = 'alpha beta gamma delta epsilon';
    const receipt = makeReceipt('part4', output);
    const p1: EditorialPatch = {
      before: 'delta',
      after: 'DELTA',
      rationale: 'must be uppercased per source style guide',
      sourceReferences: [{ sourceId: 'style-guide-v3', quote: 'proper nouns uppercased' }],
    };
    const p2: EditorialPatch = {
      before: 'alpha',
      after: 'ALPHA',
      rationale: 'must be uppercased per source style guide',
      sourceReferences: [{ sourceId: 'style-guide-v3', quote: 'proper nouns uppercased' }],
    };
    const rev = applyEditorialRevision({
      baseReceipt: receipt,
      currentOutput: output,
      expectedBaseOutputHash: receipt.output_hash,
      patches: [p1, p2], // caller order reversed
      reviewer: 'human',
      timestamp: '2026-09-30T13:00:00Z',
    });
    expect(rev.revisedOutput).toBe('ALPHA beta gamma DELTA epsilon');
    // deterministic application order: by first-occurrence index in currentOutput
    expect(rev.patches.map((p) => p.before)).toEqual(['alpha', 'delta']);
  });

  it('accepts adjacent patches (touching but not overlapping)', () => {
    const output = 'ABCD';
    const receipt = makeReceipt('part5', output);
    const rev = applyEditorialRevision({
      baseReceipt: receipt,
      currentOutput: output,
      expectedBaseOutputHash: receipt.output_hash,
      patches: [
        {
          before: 'AB',
          after: 'ab',
          rationale: 'lowercase per source',
          sourceReferences: [{ sourceId: 's', quote: 'lowercase' }],
        },
        {
          before: 'CD',
          after: 'cd',
          rationale: 'lowercase per source',
          sourceReferences: [{ sourceId: 's', quote: 'lowercase' }],
        },
      ],
      reviewer: 'human',
      timestamp: '2026-09-30T14:00:00Z',
    });
    expect(rev.revisedOutput).toBe('abcd');
  });
});

// ─── immutability + non-mutation ──────────────────────────────────────

describe('applyEditorialRevision — immutability', () => {
  it('returns a frozen artifact whose fields cannot be mutated', () => {
    const rev = applyEditorialRevision(makeInput());
    expect(Object.isFrozen(rev)).toBe(true);
    expect(Object.isFrozen(rev.pendingReviews)).toBe(true);
    expect(Object.isFrozen(rev.patches)).toBe(true);
    expect(Object.isFrozen(rev.patches[0])).toBe(true);
    expect(Object.isFrozen(rev.patches[0].sourceReferences)).toBe(true);
    expect(Object.isFrozen(rev.patches[0].sourceReferences[0])).toBe(true);

    expect(() => {
      (rev as unknown as { revisedOutput: string }).revisedOutput = 'tampered';
    }).toThrow();
    expect(() => {
      (rev.pendingReviews as unknown as { jev: string }).jev = 'accepted';
    }).toThrow();
  });

  it('does not mutate the base receipt or the input patches array', () => {
    const input = makeInput();
    const receiptSnapshot = JSON.parse(JSON.stringify(input.baseReceipt));
    const patchesSnapshot = JSON.parse(JSON.stringify(input.patches));
    const originalLength = input.patches.length;

    applyEditorialRevision(input);

    expect(input.baseReceipt).toEqual(receiptSnapshot);
    expect(input.baseReceipt.output_hash).toBe(receiptSnapshot.output_hash);
    expect(input.patches.length).toBe(originalLength);
    expect(JSON.parse(JSON.stringify(input.patches))).toEqual(patchesSnapshot);
  });

  it('preserves the base receipt hash as reference; never fabricates a re-run', () => {
    const rev = applyEditorialRevision(makeInput());
    // artifact must only carry the base HASH, not the full receipt
    expect(rev).not.toHaveProperty('baseReceipt');
    expect(rev).not.toHaveProperty('rerun');
    expect(rev).not.toHaveProperty('accepted');
    expect(rev.baseOutputHash).toMatch(/^[0-9a-f]{64}$/);
  });
});

// ─── rejection cases ──────────────────────────────────────────────────

describe('applyEditorialRevision — rejection cases', () => {
  const expectReject = (input: ApplyEditorialRevisionInput, code: string) => {
    let thrown: unknown = null;
    try {
      applyEditorialRevision(input);
    } catch (e) {
      thrown = e;
    }
    expect(thrown).toBeInstanceOf(EditorialRevisionError);
    expect((thrown as EditorialRevisionError).code).toBe(code);
  };

  it('rejects base hash mismatch (expectedBaseOutputHash vs currentOutput)', () => {
    const input = makeInput({ expectedBaseOutputHash: sha256('something else entirely') });
    expectReject(input, 'base_hash_mismatch');
  });

  it('rejects when expectedBaseOutputHash disagrees with baseReceipt.output_hash', () => {
    const output = 'Saturn transits the natal ascendant during 2024.';
    const receipt = makeReceipt('part3', output);
    const tamperedReceipt: SectionExecutionReceipt = { ...receipt, output_hash: sha256('other') };
    const input = makeInput({
      baseReceipt: tamperedReceipt,
      expectedBaseOutputHash: sha256(output),
    });
    expectReject(input, 'base_hash_mismatch');
  });

  it('rejects empty patches array', () => {
    expectReject(makeInput({ patches: [] }), 'empty_patches');
  });

  it('rejects missing occurrence of `before`', () => {
    const input = makeInput({
      patches: [
        {
          before: 'not-in-output',
          after: 'anything',
          rationale: 'because',
          sourceReferences: [{ sourceId: 's', quote: 'q' }],
        },
      ],
    });
    expectReject(input, 'missing_occurrence');
  });

  it('rejects ambiguous occurrence (before appears more than once)', () => {
    const output = 'the year 2024 followed the year 2024';
    const receipt = makeReceipt('part3', output);
    const input = makeInput({
      baseReceipt: receipt,
      currentOutput: output,
      expectedBaseOutputHash: receipt.output_hash,
      patches: [
        {
          before: '2024',
          after: '2025',
          rationale: 'ambiguity should be caught',
          sourceReferences: [{ sourceId: 's', quote: 'q' }],
        },
      ],
    });
    expectReject(input, 'ambiguous_occurrence');
  });

  it('rejects duplicate `before` strings across patches', () => {
    const output = 'alpha and alpha again';
    const receipt = makeReceipt('part3', output);
    const patch: EditorialPatch = {
      before: 'alpha',
      after: 'ALPHA',
      rationale: 'stylistic',
      sourceReferences: [{ sourceId: 's', quote: 'q' }],
    };
    const input = makeInput({
      baseReceipt: receipt,
      currentOutput: output,
      expectedBaseOutputHash: receipt.output_hash,
      patches: [patch, { ...patch, after: 'AlPhA' }],
    });
    expectReject(input, 'duplicate_before');
  });

  it('rejects overlapping patches (one before contains another)', () => {
    const output = 'the quick brown fox';
    const receipt = makeReceipt('part3', output);
    const input = makeInput({
      baseReceipt: receipt,
      currentOutput: output,
      expectedBaseOutputHash: receipt.output_hash,
      patches: [
        {
          before: 'quick brown',
          after: 'slow gray',
          rationale: 'r1',
          sourceReferences: [{ sourceId: 's', quote: 'q' }],
        },
        {
          before: 'brown fox',
          after: 'green fox',
          rationale: 'r2',
          sourceReferences: [{ sourceId: 's', quote: 'q' }],
        },
      ],
    });
    expectReject(input, 'overlapping_patches');
  });

  it('rejects empty section ID on base receipt', () => {
    const output = 'x';
    const receipt = makeReceipt('   ', output);
    const input = makeInput({
      baseReceipt: receipt,
      currentOutput: output,
      expectedBaseOutputHash: receipt.output_hash,
      patches: [
        {
          before: 'x',
          after: 'y',
          rationale: 'r',
          sourceReferences: [{ sourceId: 's', quote: 'q' }],
        },
      ],
    });
    expectReject(input, 'invalid_section_id');
  });

  it('rejects empty rationale', () => {
    const input = makeInput({
      patches: [
        {
          before: '2024',
          after: '2025',
          rationale: '   ',
          sourceReferences: [{ sourceId: 's', quote: 'q' }],
        },
      ],
    });
    expectReject(input, 'empty_rationale');
  });

  it('rejects empty sourceReferences array', () => {
    const input = makeInput({
      patches: [
        {
          before: '2024',
          after: '2025',
          rationale: 'r',
          sourceReferences: [],
        },
      ],
    });
    expectReject(input, 'empty_sources');
  });

  it('rejects source ref with empty sourceId or quote', () => {
    expectReject(
      makeInput({
        patches: [
          {
            before: '2024',
            after: '2025',
            rationale: 'r',
            sourceReferences: [{ sourceId: '', quote: 'q' }],
          },
        ],
      }),
      'invalid_source_ref',
    );
    expectReject(
      makeInput({
        patches: [
          {
            before: '2024',
            after: '2025',
            rationale: 'r',
            sourceReferences: [{ sourceId: 's', quote: '   ' }],
          },
        ],
      }),
      'invalid_source_ref',
    );
  });

  it('rejects reviewer not in the closed enumeration', () => {
    const input = {
      ...makeInput(),
      reviewer: 'ai' as unknown as ApplyEditorialRevisionInput['reviewer'],
    };
    expectReject(input, 'invalid_reviewer');
  });

  it('rejects empty timestamp', () => {
    expectReject(makeInput({ timestamp: '   ' }), 'invalid_timestamp');
  });

  it('rejects no-op patch where after equals before', () => {
    const input = makeInput({
      patches: [
        {
          before: '2024',
          after: '2024',
          rationale: 'r',
          sourceReferences: [{ sourceId: 's', quote: 'q' }],
        },
      ],
    });
    expectReject(input, 'noop_patch');
  });
});

// ─── pending review flags & no acceptance leakage ─────────────────────

describe('applyEditorialRevision — pending review flags', () => {
  it('always marks all downstream gates pending; never accepted', () => {
    const rev: EditorialRevision = applyEditorialRevision(makeInput());
    for (const key of ['sourceAudit', 'jev', 'document', 'coherence'] as const) {
      expect(rev.pendingReviews[key]).toBe('pending');
    }
  });

  it('does not carry a jev field on the revision itself (no relabel of prior Jev)', () => {
    const rev = applyEditorialRevision(makeInput());
    expect(rev).not.toHaveProperty('jev');
    expect(rev).not.toHaveProperty('source_audit');
  });
});
