// ─── Primary Passage Source — Tests ────────────────────────────────
// Unit tests for buildCombinedPassages / isPrimarySourceId. The module
// is pure — no I/O, no LLM, no network — so tests operate on frozen
// fixtures shaped after the real registry
//   tools/humdes-extractor/output/gary-abitbol-2026-09-28/
//     reports-reference/editorial-primary-only-registry.json
//
// Contract invariants under test:
//   * Combined order is `[...cfPassages, ...primaryAdditions]`.
//   * `passagesHash` = sha256(join('\n', text)) over that order.
//   * `cfSourceIds` / `primarySourceIds` bind the exact ordered subsets.
//   * CF ids MUST use the `sw:` prefix; primary ids MUST NOT use `sw:`
//     or `ed:`.
//   * A primary id colliding with a live CF id is rejected — spoofed
//     provenance is never accepted silently.
//   * Result and every passage frozen: retriever cannot mutate.
//   * `isPrimarySourceId` returns true only for non-`sw:`/non-`ed:` ids.

import { describe, it, expect } from 'vitest';
import { createHash } from 'node:crypto';
import { buildCombinedPassages, isPrimarySourceId } from './primary-passage-source.js';
import type { RetrievedPassage } from './reference-execution.js';
import type { EditorialPassageAddition } from './editorial-passage-evidence.js';

const sha256 = (t: string) => createHash('sha256').update(t, 'utf8').digest('hex');

const CF_A_TEXT = 'Framework passage A — structural notes.';
const CF_B_TEXT = 'Framework passage B — experiential notes.';

const CF_PASSAGES: readonly RetrievedPassage[] = Object.freeze([
  Object.freeze({ id: 'sw:hd:house:2:desc', source: 'cf-vectorize:acct/index', text: CF_A_TEXT }),
  Object.freeze({ id: 'sw:hd:house:10:desc', source: 'cf-vectorize:acct/index', text: CF_B_TEXT }),
]);

const PRIMARY_TEXT = 'Rao’s primary text on house theme two — cited from PDF chapter 7.';

function makePrimary(id: string, text: string = PRIMARY_TEXT): EditorialPassageAddition {
  return {
    id,
    text,
    sha256: sha256(text),
    provenance: {
      kind: 'reviewed-primary-document',
      author: 'P. V. R. Narasimha Rao',
      title: 'Vedic Astrology: An Integrated Approach',
      url: 'https://www.vedicastrologer.org/articles/vedic_astro_textbook.pdf',
      locator: 'Chapter 7.2',
      reviewer: 'assistant-source-review',
    },
  };
}

describe('buildCombinedPassages — combined ordering + hashing', () => {
  it('emits [...cf, ...primary] in registry additionIds order with matching hash', () => {
    const primary = [makePrimary('primary:rao2000:house:2', 'p1'), makePrimary('primary:rao2000:house:10', 'p2')];
    const result = buildCombinedPassages({ cfPassages: CF_PASSAGES, primaryAdditions: primary });
    expect(result.passages.map((p) => p.id)).toEqual([
      'sw:hd:house:2:desc', 'sw:hd:house:10:desc',
      'primary:rao2000:house:2', 'primary:rao2000:house:10',
    ]);
    expect(result.cfSourceIds).toEqual(['sw:hd:house:2:desc', 'sw:hd:house:10:desc']);
    expect(result.primarySourceIds).toEqual(['primary:rao2000:house:2', 'primary:rao2000:house:10']);
    const expectedHash = sha256([CF_A_TEXT, CF_B_TEXT, 'p1', 'p2'].join('\n'));
    expect(result.passagesHash).toBe(expectedHash);
    expect(result.perPassageSha256).toEqual([sha256(CF_A_TEXT), sha256(CF_B_TEXT), sha256('p1'), sha256('p2')]);
  });

  it('re-derives the same passagesHash when called twice with the same inputs', () => {
    const primary = [makePrimary('primary:rao2000:house:2')];
    const a = buildCombinedPassages({ cfPassages: CF_PASSAGES, primaryAdditions: primary });
    const b = buildCombinedPassages({ cfPassages: CF_PASSAGES, primaryAdditions: primary });
    expect(a.passagesHash).toBe(b.passagesHash);
  });

  it('returns cf-only when primary additions is empty (backwards compatible)', () => {
    const result = buildCombinedPassages({ cfPassages: CF_PASSAGES, primaryAdditions: [] });
    expect(result.primarySourceIds).toEqual([]);
    expect(result.cfSourceIds).toEqual(['sw:hd:house:2:desc', 'sw:hd:house:10:desc']);
    expect(result.passages).toHaveLength(2);
  });
});

describe('buildCombinedPassages — id prefix guards', () => {
  it('rejects a CF passage whose id lacks the sw: prefix', () => {
    expect(() => buildCombinedPassages({
      cfPassages: [{ id: 'not-cf:xyz', source: 'cf', text: 'x' }],
      primaryAdditions: [],
    })).toThrow(/cfPassages must use the sw:/);
  });

  it('rejects a primary addition using the reserved sw: prefix', () => {
    expect(() => buildCombinedPassages({
      cfPassages: CF_PASSAGES,
      primaryAdditions: [makePrimary('sw:spoof:house:2')],
    })).toThrow(/reserved id prefix/);
  });

  it('rejects a primary addition using the reserved ed: prefix', () => {
    expect(() => buildCombinedPassages({
      cfPassages: CF_PASSAGES,
      primaryAdditions: [makePrimary('ed:private:house:2')],
    })).toThrow(/reserved id prefix/);
  });
});

describe('buildCombinedPassages — provenance guards', () => {
  it('rejects a primary addition colliding with a live CF id', () => {
    expect(() => buildCombinedPassages({
      cfPassages: CF_PASSAGES,
      // Give the "primary" addition the same id as an existing CF passage.
      primaryAdditions: [{
        ...makePrimary('primary:rao2000:house:2'),
        id: 'sw:hd:house:2:desc',
      }],
    })).toThrow(/reserved id prefix|collides with a live CF id/);
  });

  it('rejects a primary addition with wrong provenance kind', () => {
    const spoof = { ...makePrimary('primary:rao2000:house:2'), provenance: { kind: 'cf-vectorize', account: 'a', index: 'b' } } as unknown as EditorialPassageAddition;
    expect(() => buildCombinedPassages({ cfPassages: CF_PASSAGES, primaryAdditions: [spoof] })).toThrow(/reviewed-primary-document/);
  });

  it('stamps the source label as reviewed-primary-document:<url>', () => {
    const primary = [makePrimary('primary:rao2000:house:2')];
    const result = buildCombinedPassages({ cfPassages: CF_PASSAGES, primaryAdditions: primary });
    const primaryPassage = result.passages.find((p) => p.id === 'primary:rao2000:house:2')!;
    expect(primaryPassage.source).toBe('reviewed-primary-document:https://www.vedicastrologer.org/articles/vedic_astro_textbook.pdf');
  });
});

describe('buildCombinedPassages — freezing', () => {
  it('freezes the outer result, passages array, and every passage', () => {
    const primary = [makePrimary('primary:rao2000:house:2')];
    const result = buildCombinedPassages({ cfPassages: CF_PASSAGES, primaryAdditions: primary });
    expect(Object.isFrozen(result)).toBe(true);
    expect(Object.isFrozen(result.passages)).toBe(true);
    for (const p of result.passages) expect(Object.isFrozen(p)).toBe(true);
  });
});

describe('isPrimarySourceId', () => {
  it('returns false for sw:-prefixed ids (CF)', () => {
    expect(isPrimarySourceId('sw:hd:house:2:desc')).toBe(false);
  });
  it('returns false for ed:-prefixed ids (private editorial)', () => {
    expect(isPrimarySourceId('ed:private:2')).toBe(false);
  });
  it('returns true for primary registry ids', () => {
    expect(isPrimarySourceId('primary:rao2000:house:2')).toBe(true);
  });
  it('returns true for an unprefixed id (fail-open shape check only; validators reject elsewhere)', () => {
    expect(isPrimarySourceId('plain-id-42')).toBe(true);
  });
});
