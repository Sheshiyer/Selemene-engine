// ─── Editorial Passage Evidence Validator — Tests ─────────────────────
// Focused unit tests for the leaf primitive in editorial-passage-evidence.ts.
// Fixtures are shaped after
//   tools/humdes-extractor/output/gary-abitbol-2026-09-28/reports-reference/
//     house-meaning-source-supplement.json
// but are compact, self-contained, and use only shapes the CF preflight
// contract already exposes (RetrievedPassage {id, source, text} + 'sw:' IDs).
// No I/O, no network, no LLM.

import { describe, it, expect } from 'vitest';
import { createHash } from 'node:crypto';
import type { RetrievedPassage } from './reference-execution.js';
import {
  validateEditorialPassageEvidence,
  EditorialPassageEvidenceError,
  type EditorialPassageAddition,
  type EditorialPassageEvidenceInput,
  validatePrimaryPassageAdditions,
} from './editorial-passage-evidence.js';

const sha256 = (t: string): string =>
  createHash('sha256').update(t, 'utf8').digest('hex');

// ─── Fixtures ─────────────────────────────────────────────────────────

const BASE_A_TEXT = 'The 2nd house theme covers material resources and speech.';
const BASE_B_TEXT = 'The 10th house theme covers vocation and public conduct.';

const BASE_PASSAGES: readonly RetrievedPassage[] = Object.freeze([
  Object.freeze({
    id: 'sw:hd:house:2:desc',
    source: 'cf-vectorize:acct-x/witness-corpus',
    text: BASE_A_TEXT,
  }),
  Object.freeze({
    id: 'sw:hd:house:10:desc',
    source: 'cf-vectorize:acct-x/witness-corpus',
    text: BASE_B_TEXT,
  }),
]);

const BASE_HASH = sha256([BASE_A_TEXT, BASE_B_TEXT].join('\n'));

const TRUSTED_ORIGINAL_TEXTS = Object.freeze({
  'sw:hd:house:2:desc': BASE_A_TEXT,
  'sw:hd:house:10:desc': BASE_B_TEXT,
});

const CF_ADD_TEXT = 'The 5th house theme covers creative self-expression and progeny.';
const CF_ADDITION: EditorialPassageAddition = {
  id: 'sw:hd:house:5:desc',
  text: CF_ADD_TEXT,
  sha256: sha256(CF_ADD_TEXT),
  provenance: {
    kind: 'cf-vectorize',
    account: 'acct-x',
    index: 'witness-corpus',
  },
};

const PRIMARY_ADD_TEXT =
  'In Rao’s traditional house scheme the second house includes family and speech.';
const PRIMARY_ADDITION: EditorialPassageAddition = {
  id: 'primary:rao2000:house:2',
  text: PRIMARY_ADD_TEXT,
  sha256: sha256(PRIMARY_ADD_TEXT),
  provenance: {
    kind: 'reviewed-primary-document',
    author: 'P. V. R. Narasimha Rao',
    title: 'Vedic Astrology: An Integrated Approach',
    url: 'https://www.vedicastrologer.org/articles/vedic_astro_textbook.pdf',
    locator: 'Chapter 7.2, printed pages 68–69; PDF pages 80–81',
    reviewer: 'assistant-source-review',
  },
};

const TRUSTED_RECORDS = Object.freeze({
  'sw:hd:house:5:desc': CF_ADDITION,
  'primary:rao2000:house:2': PRIMARY_ADDITION,
});

function baseInput(
  overrides: Partial<EditorialPassageEvidenceInput> = {},
): EditorialPassageEvidenceInput {
  return {
    expectedCF: { account: 'acct-x', index: 'witness-corpus' },
    basePassages: BASE_PASSAGES,
    basePassagesHash: BASE_HASH,
    additions: [CF_ADDITION, PRIMARY_ADDITION],
    trustedSourceRecords: TRUSTED_RECORDS,
    trustedOriginalTexts: TRUSTED_ORIGINAL_TEXTS,
    ...overrides,
  };
}

// ─── Success ──────────────────────────────────────────────────────────

describe('validateEditorialPassageEvidence — happy path', () => {
  it('returns frozen mixed CF + primary-document effective view with matching hash', () => {
    const result = validateEditorialPassageEvidence(baseInput());
    expect(result.originalBasePassagesHash).toBe(BASE_HASH);
    expect(result.effectivePassages).toHaveLength(4);
    expect(result.effectivePassages.map((p) => p.id)).toEqual([
      'sw:hd:house:2:desc',
      'sw:hd:house:10:desc',
      'sw:hd:house:5:desc',
      'primary:rao2000:house:2',
    ]);
    const expectedHash = sha256(
      [BASE_A_TEXT, BASE_B_TEXT, CF_ADD_TEXT, PRIMARY_ADD_TEXT].join('\n'),
    );
    expect(result.effectivePassagesHash).toBe(expectedHash);
    expect(result.addedSourceRecords).toHaveLength(2);
    expect(result.addedSourceRecords[0].provenance.kind).toBe('cf-vectorize');
    expect(result.addedSourceRecords[1].provenance.kind).toBe(
      'reviewed-primary-document',
    );
    // Frozen contract
    expect(Object.isFrozen(result)).toBe(true);
    expect(Object.isFrozen(result.effectivePassages)).toBe(true);
    for (const p of result.effectivePassages) expect(Object.isFrozen(p)).toBe(true);
    for (const r of result.addedSourceRecords) expect(Object.isFrozen(r)).toBe(true);
  });

  it('accepts an empty additions array and echoes the base hash unchanged', () => {
    const result = validateEditorialPassageEvidence(
      baseInput({ additions: [], trustedSourceRecords: {} }),
    );
    expect(result.addedSourceRecords).toHaveLength(0);
    expect(result.effectivePassagesHash).toBe(BASE_HASH);
    expect(result.originalBasePassagesHash).toBe(BASE_HASH);
  });
});

// ─── Base retrieval integrity ─────────────────────────────────────────

describe('validateEditorialPassageEvidence — base retrieval integrity', () => {
  it('rejects when basePassagesHash does not match the base join', () => {
    expect(() =>
      validateEditorialPassageEvidence(
        baseInput({ basePassagesHash: sha256('tampered') }),
      ),
    ).toThrow(/original corpus altered/);
  });

  it('rejects when a base text differs from the trusted original registry', () => {
    const badBase = [
      { id: 'sw:hd:house:2:desc', source: 'cf', text: 'edited base text' },
      BASE_PASSAGES[1],
    ] as RetrievedPassage[];
    expect(() =>
      validateEditorialPassageEvidence(
        baseInput({
          basePassages: badBase,
          basePassagesHash: sha256(
            ['edited base text', BASE_B_TEXT].join('\n'),
          ),
        }),
      ),
    ).toThrow(/retrieval drift/);
  });

  it('rejects when the trusted original registry omits a base id', () => {
    expect(() =>
      validateEditorialPassageEvidence(
        baseInput({
          trustedOriginalTexts: {
            'sw:hd:house:2:desc': BASE_A_TEXT,
          },
        }),
      ),
    ).toThrow(/trustedOriginalTexts missing entry/);
  });

  it('rejects an empty base retrieval', () => {
    expect(() =>
      validateEditorialPassageEvidence(
        baseInput({
          basePassages: [],
          basePassagesHash: sha256(''),
        }),
      ),
    ).toThrow(/basePassages must be non-empty/);
  });

  it('rejects a duplicate id inside base retrieval', () => {
    const dup = [
      BASE_PASSAGES[0],
      { ...BASE_PASSAGES[0] },
    ] as RetrievedPassage[];
    expect(() =>
      validateEditorialPassageEvidence(
        baseInput({
          basePassages: dup,
          basePassagesHash: sha256([BASE_A_TEXT, BASE_A_TEXT].join('\n')),
          trustedOriginalTexts: { 'sw:hd:house:2:desc': BASE_A_TEXT },
        }),
      ),
    ).toThrow(/duplicate id/);
  });
});

// ─── Addition forgery / mismatch ──────────────────────────────────────

describe('validateEditorialPassageEvidence — addition forgery', () => {
  it('rejects a cf-vectorize passage without the "sw:" id prefix', () => {
    const forged: EditorialPassageAddition = {
      ...CF_ADDITION,
      id: 'other:5',
    };
    expect(() =>
      validateEditorialPassageEvidence(
        baseInput({
          additions: [forged],
          trustedSourceRecords: { 'other:5': forged },
        }),
      ),
    ).toThrow(/must use the "sw:" id prefix/);
  });

  it('rejects a reviewed-primary-document masquerading with a "sw:" id', () => {
    const forged: EditorialPassageAddition = {
      ...PRIMARY_ADDITION,
      id: 'sw:hd:pretend:1',
    };
    expect(() =>
      validateEditorialPassageEvidence(
        baseInput({
          additions: [forged],
          trustedSourceRecords: { 'sw:hd:pretend:1': forged },
        }),
      ),
    ).toThrow(/must not use the "sw:" id prefix/);
  });

  it('rejects any addition that uses the private "ed:" id prefix', () => {
    const forged: EditorialPassageAddition = {
      ...PRIMARY_ADDITION,
      id: 'ed:scratch:1',
    };
    expect(() =>
      validateEditorialPassageEvidence(
        baseInput({
          additions: [forged],
          trustedSourceRecords: { 'ed:scratch:1': forged },
        }),
      ),
    ).toThrow(/reserved private "ed:" prefix/);
  });

  it('rejects a reviewed-primary-document without an https URL', () => {
    const forged: EditorialPassageAddition = {
      ...PRIMARY_ADDITION,
      provenance: {
        ...PRIMARY_ADDITION.provenance,
        kind: 'reviewed-primary-document',
        url: 'http://insecure.example/book.pdf',
      } as EditorialPassageAddition['provenance'],
    };
    expect(() =>
      validateEditorialPassageEvidence(
        baseInput({
          additions: [forged],
          trustedSourceRecords: { [forged.id]: forged },
        }),
      ),
    ).toThrow(/requires https URL/);
  });

  it('rejects when addition text differs but sha256 is caller-declared to match', () => {
    // Text was mutated but sha256 field left in place → sha256(text) must diverge.
    const forged: EditorialPassageAddition = {
      ...CF_ADDITION,
      text: CF_ADD_TEXT + ' EXTRA',
    };
    expect(() =>
      validateEditorialPassageEvidence(
        baseInput({
          additions: [forged],
          trustedSourceRecords: { [forged.id]: forged },
        }),
      ),
    ).toThrow(/sha256 does not match sha256\(text\)/);
  });

  it('rejects when addition provenance metadata differs from the trusted record', () => {
    const forged: EditorialPassageAddition = {
      ...CF_ADDITION,
      provenance: {
        kind: 'cf-vectorize',
        account: 'acct-x',
        index: 'different-index',
      },
    };
    // sha256/text still correct; trusted record still has the original index.
    expect(() =>
      validateEditorialPassageEvidence(
        baseInput({
          additions: [forged, PRIMARY_ADDITION],
          expectedCF: { account: 'acct-x', index: 'different-index' },
        }),
      ),
    ).toThrow(/does not deep-equal its trustedSourceRecords entry/);
  });

  it('rejects when no trustedSourceRecords entry exists for an addition', () => {
    expect(() =>
      validateEditorialPassageEvidence(
        baseInput({
          trustedSourceRecords: {
            'primary:rao2000:house:2': PRIMARY_ADDITION,
          },
        }),
      ),
    ).toThrow(/no matching trustedSourceRecords entry/);
  });

  it('rejects duplicate addition ids', () => {
    expect(() =>
      validateEditorialPassageEvidence(
        baseInput({
          additions: [CF_ADDITION, CF_ADDITION],
          trustedSourceRecords: { [CF_ADDITION.id]: CF_ADDITION },
        }),
      ),
    ).toThrow(/duplicate id/);
  });

  it('rejects addition id colliding with an existing base passage id', () => {
    const colliding: EditorialPassageAddition = {
      ...CF_ADDITION,
      id: 'sw:hd:house:2:desc',
    };
    expect(() =>
      validateEditorialPassageEvidence(
        baseInput({
          additions: [colliding],
          trustedSourceRecords: { [colliding.id]: colliding },
        }),
      ),
    ).toThrow(/collides with an existing base passage id/);
  });

  it('rejects blank addition text', () => {
    const blank: EditorialPassageAddition = {
      ...CF_ADDITION,
      text: '',
      sha256: sha256(''),
    };
    expect(() =>
      validateEditorialPassageEvidence(
        baseInput({
          additions: [blank],
          trustedSourceRecords: { [blank.id]: blank },
        }),
      ),
    ).toThrow(/text must be a non-empty string/);
  });

  it('rejects an unknown provenance kind', () => {
    const alien = {
      ...CF_ADDITION,
      provenance: { kind: 'made-up', account: 'x', index: 'y' } as unknown,
    } as EditorialPassageAddition;
    expect(() =>
      validateEditorialPassageEvidence(
        baseInput({
          additions: [alien],
          trustedSourceRecords: { [alien.id]: alien },
        }),
      ),
    ).toThrow(/provenance\.kind is unknown or missing/);
  });
});

// ─── Malformed inputs ─────────────────────────────────────────────────

describe('validateEditorialPassageEvidence — malformed inputs', () => {
  it('rejects non-array basePassages', () => {
    expect(() =>
      validateEditorialPassageEvidence({
        ...baseInput(),
        basePassages: 'nope' as unknown as readonly RetrievedPassage[],
      }),
    ).toThrow(/basePassages must be an array/);
  });

  it('rejects non-array additions', () => {
    expect(() =>
      validateEditorialPassageEvidence({
        ...baseInput(),
        additions: 'nope' as unknown as readonly EditorialPassageAddition[],
      }),
    ).toThrow(/additions must be an array/);
  });

  it('rejects malformed basePassagesHash', () => {
    expect(() =>
      validateEditorialPassageEvidence(
        baseInput({ basePassagesHash: 'not-hex' }),
      ),
    ).toThrow(/basePassagesHash is not a valid lowercase hex sha256/);
  });

  it('is an EditorialPassageEvidenceError on failure', () => {
    try {
      validateEditorialPassageEvidence(
        baseInput({ basePassagesHash: sha256('other') }),
      );
      throw new Error('should have thrown');
    } catch (err) {
      expect(err).toBeInstanceOf(EditorialPassageEvidenceError);
    }
  });
});

// ─── Immutability ─────────────────────────────────────────────────────

describe('validateEditorialPassageEvidence — never mutates inputs', () => {
  it('does not mutate base passages or additions', () => {
    const snapshot = JSON.stringify({
      base: BASE_PASSAGES,
      additions: [CF_ADDITION, PRIMARY_ADDITION],
      trusted: TRUSTED_RECORDS,
      originals: TRUSTED_ORIGINAL_TEXTS,
    });
    const result = validateEditorialPassageEvidence(baseInput());
    expect(result.effectivePassages.length).toBe(4);
    const after = JSON.stringify({
      base: BASE_PASSAGES,
      additions: [CF_ADDITION, PRIMARY_ADDITION],
      trusted: TRUSTED_RECORDS,
      originals: TRUSTED_ORIGINAL_TEXTS,
    });
    expect(after).toBe(snapshot);
  });
});

describe('route target and strict record checks', () => {
  it('rejects a matching registry from a different CF target', () => {
    expect(() => validateEditorialPassageEvidence(baseInput({ expectedCF: { account: 'other', index: 'witness-corpus' } }))).toThrow(/account\/index/);
  });
  it('requires an independently selected CF target', () => {
    expect(() => validateEditorialPassageEvidence(baseInput({ expectedCF: undefined }))).toThrow(/account\/index/);
  });
  it('rejects whitespace-only text even with its correct hash', () => {
    const a = { ...PRIMARY_ADDITION, text: ' \n ', sha256: sha256(' \n ') };
    expect(() => validateEditorialPassageEvidence(baseInput({ additions: [a], trustedSourceRecords: { [a.id]: a } }))).toThrow(/non-empty/);
  });
  it('rejects an invalid HTTPS URL even when trusted metadata agrees', () => {
    const a = { ...PRIMARY_ADDITION, provenance: { ...PRIMARY_ADDITION.provenance, url: 'https://' } } as EditorialPassageAddition;
    expect(() => validateEditorialPassageEvidence(baseInput({ additions: [a], trustedSourceRecords: { [a.id]: a } }))).toThrow(/https URL/);
  });
  it('rejects extra fields on an addition rather than dropping them', () => {
    const a = { ...PRIMARY_ADDITION, falselyIngested: true };
    expect(() => validateEditorialPassageEvidence(baseInput({ additions: [a], trustedSourceRecords: { [a.id]: a } }))).toThrow(/forged provenance/);
  });
  it('rejects private IDs in the base corpus', () => {
    const base = [{ ...BASE_PASSAGES[0], id: 'ed:private' }];
    expect(() => validateEditorialPassageEvidence(baseInput({ basePassages: base, basePassagesHash: sha256(base[0].text), additions: [], trustedOriginalTexts: { 'ed:private': base[0].text } }))).toThrow(/public sw:/);
  });
});

// ─── validatePrimaryPassageAdditions ─────────────────────────────────
// The primary-passage-additions validator authorises a fresh-generation
// primary registry against the same shape rules the editorial supplement
// uses (reviewed-primary-document only). No CF fabrication is allowed.

const RAO_URL = 'https://www.vedicastrologer.org/articles/vedic_astro_textbook.pdf';
function primaryRecord(id: string, text: string): EditorialPassageAddition {
  return {
    id,
    text,
    sha256: sha256(text),
    provenance: {
      kind: 'reviewed-primary-document',
      author: 'P. V. R. Narasimha Rao',
      title: 'Vedic Astrology: An Integrated Approach',
      url: RAO_URL,
      locator: 'Chapter 7.2',
      reviewer: 'assistant-source-review',
    },
  };
}

describe('validatePrimaryPassageAdditions', () => {
  it('accepts a happy-path 2-record registry and returns frozen ordered additions', () => {
    const a = primaryRecord('primary:rao2000:house:2', 'text a');
    const b = primaryRecord('primary:rao2000:house:10', 'text b');
    const result = validatePrimaryPassageAdditions({
      registry: { records: { [a.id]: a, [b.id]: b }, additionIds: [a.id, b.id] },
    });
    expect(result.additionIds).toEqual([a.id, b.id]);
    expect(result.additions.map((r) => r.id)).toEqual([a.id, b.id]);
    expect(Object.isFrozen(result)).toBe(true);
    expect(Object.isFrozen(result.additions)).toBe(true);
    for (const r of result.additions) expect(Object.isFrozen(r)).toBe(true);
  });

  it('rejects an empty additionIds array — nothing to fresh-generate against', () => {
    expect(() => validatePrimaryPassageAdditions({
      registry: { records: {}, additionIds: [] },
    })).toThrow(/non-empty/);
  });

  it('rejects an unknown id (not present in records)', () => {
    const a = primaryRecord('primary:rao2000:house:2', 'text a');
    expect(() => validatePrimaryPassageAdditions({
      registry: { records: { [a.id]: a }, additionIds: ['primary:rao2000:house:99'] },
    })).toThrow(/unknown record/);
  });

  it('rejects a cf-vectorize record in the primary additions (live CF is source of truth)', () => {
    const spoof = {
      id: 'primary:rao2000:house:2',
      text: 'text a',
      sha256: sha256('text a'),
      provenance: { kind: 'cf-vectorize', account: 'a'.repeat(32), index: 'witness-corpus' },
    } as unknown as EditorialPassageAddition;
    expect(() => validatePrimaryPassageAdditions({
      registry: { records: { [spoof.id]: spoof }, additionIds: [spoof.id] },
    })).toThrow(/reviewed-primary-document/);
  });

  it('rejects a duplicate id in additionIds', () => {
    const a = primaryRecord('primary:rao2000:house:2', 'text a');
    expect(() => validatePrimaryPassageAdditions({
      registry: { records: { [a.id]: a }, additionIds: [a.id, a.id] },
    })).toThrow(/duplicate/);
  });

  it('rejects an id colliding with a reserved live CF id', () => {
    const a = primaryRecord('primary:rao2000:house:2', 'text a');
    expect(() => validatePrimaryPassageAdditions({
      registry: { records: { [a.id]: a }, additionIds: [a.id] },
      reservedCFIds: [a.id],
    })).toThrow(/spoofed provenance|collides with a live CF passage id/);
  });

  it('rejects an id whose record has a mismatched inner id', () => {
    const a = primaryRecord('primary:rao2000:house:2', 'text a');
    const forged = { ...a, id: 'primary:rao2000:house:99' };
    expect(() => validatePrimaryPassageAdditions({
      registry: { records: { 'primary:rao2000:house:2': forged }, additionIds: ['primary:rao2000:house:2'] },
    })).toThrow(/mismatched id/);
  });

  it('rejects a record whose sha256 does not match its text', () => {
    const a = primaryRecord('primary:rao2000:house:2', 'text a');
    const tampered = { ...a, text: 'DRIFTED text', };
    expect(() => validatePrimaryPassageAdditions({
      registry: { records: { [a.id]: tampered }, additionIds: [a.id] },
    })).toThrow();
  });

  it('rejects a primary record using the reserved sw: id prefix (masquerading as CF)', () => {
    const a = primaryRecord('sw:hd:house:2:desc', 'text a');
    expect(() => validatePrimaryPassageAdditions({
      registry: { records: { [a.id]: a }, additionIds: [a.id] },
    })).toThrow();
  });
});
