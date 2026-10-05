import { describe, it, expect } from 'vitest';
import {
  validateDyadReceipts,
  createDyadReceiptStore,
  DYAD_UNAVAILABLE_MARKER,
} from './witness-dyad.js';
import type { DyadSectionReceipt } from './witness-dyad.js';

function makeReceipt(sectionId: string, available: boolean, reason?: string): DyadSectionReceipt {
  return {
    section_id: sectionId,
    engine_routing: 'dyad-synthesis',
    aletheios_words: available ? 100 : 0,
    pichet_words: available ? 80 : 0,
    synthesis_words: available ? 150 : 0,
    aletheios_ok: available,
    pichet_ok: available,
    synthesis_ok: available,
    dyad_available: available,
    unavailable_reason: reason,
  };
}

describe('validateDyadReceipts — section_id-based validation', () => {
  it('passes when all required sections have available receipts', () => {
    const receipts = [
      makeReceipt('opening', true),
      makeReceipt('part1', true),
      makeReceipt('part11', true),
    ];
    const result = validateDyadReceipts(receipts, ['opening', 'part1', 'part11']);
    expect(result.passed).toBe(true);
    expect(result.blockers).toHaveLength(0);
  });

  it('blocks when a required section has no receipt (ID not in store)', () => {
    const receipts = [
      makeReceipt('opening', true),
      // part1 missing intentionally
      makeReceipt('part11', true),
    ];
    const result = validateDyadReceipts(receipts, ['opening', 'part1', 'part11']);
    expect(result.passed).toBe(false);
    expect(result.blockers.some((b) => b.includes('part1') && b.includes('dyad_receipt_missing'))).toBe(true);
  });

  it('blocks when a receipt exists but dyad_available=false', () => {
    const receipts = [
      makeReceipt('opening', true),
      makeReceipt('part1', false, 'dyad stage error: connection refused'),
    ];
    const result = validateDyadReceipts(receipts, ['opening', 'part1']);
    expect(result.passed).toBe(false);
    expect(result.blockers.some((b) => b.includes('part1') && b.includes('dyad_unavailable'))).toBe(true);
    expect(result.blockers.some((b) => b.includes('connection refused'))).toBe(true);
  });

  it('blocks when receipts list is empty and required sections are present', () => {
    const result = validateDyadReceipts([], ['opening', 'part1']);
    expect(result.passed).toBe(false);
    expect(result.blockers.some((b) => b.includes('no_receipts'))).toBe(true);
  });

  it('does not confuse receipts by array position (ID-based, not index-based)', () => {
    // Receipts in different order from requiredSectionIds — should still validate by ID
    const receipts = [
      makeReceipt('part11', true),
      makeReceipt('opening', true),
      makeReceipt('part1', false, 'timeout'),
    ];
    const result = validateDyadReceipts(receipts, ['opening', 'part1', 'part11']);
    expect(result.passed).toBe(false);
    // The blocker should reference part1, not opening (index 0 would have been opening)
    expect(result.blockers.some((b) => b.includes('part1'))).toBe(true);
    expect(result.blockers.some((b) => b.includes('opening'))).toBe(false);
    expect(result.blockers.some((b) => b.includes('part11'))).toBe(false);
  });
});

describe('createDyadReceiptStore', () => {
  it('returns an empty mutable array', () => {
    const store = createDyadReceiptStore();
    expect(store).toEqual([]);
    store.push(makeReceipt('opening', true));
    expect(store).toHaveLength(1);
  });
});

describe('DYAD_UNAVAILABLE_MARKER', () => {
  it('is a non-empty HTML comment string', () => {
    expect(DYAD_UNAVAILABLE_MARKER).toMatch(/^<!--/);
    expect(DYAD_UNAVAILABLE_MARKER.length).toBeGreaterThan(5);
  });
});


describe('structured voice-prompt builders (additive)', () => {
  it('buildAletheiosStructuredPrompt appends the structured claim requirement', async () => {
    const mod = await import('./witness-dyad.js');
    const prompt = mod.buildAletheiosStructuredPrompt('Convergence Map');
    expect(prompt).toContain('You are Aletheios');
    expect(prompt).toContain('Section topic: Convergence Map');
    expect(prompt).toContain('aletheios-claims');
    expect(prompt).toContain('"engine_id"');
  });

  it('buildPichetGroundedPrompt inlines Aletheios claims as quoted data', async () => {
    const mod = await import('./witness-dyad.js');
    const structured = '[{"engine_id":"vedic-kundali","field_path":"lagna.sign","value":"Scorpio","claim":"lagna is Scorpio"}]';
    const prompt = mod.buildPichetGroundedPrompt(structured);
    expect(prompt).toContain('You are Pichet');
    expect(prompt).toContain('CLAIM-GROUNDED REFLECTION REQUIREMENT');
    expect(prompt).toContain(structured);
  });

  it('buildSynthesisReaderFacingPrompt appends leakage prohibitions', async () => {
    const mod = await import('./witness-dyad.js');
    const prompt = mod.buildSynthesisReaderFacingPrompt();
    expect(prompt).toContain('reconciled synthesis');
    expect(prompt).toContain('READER-FACING CONTRACT');
    expect(prompt).toContain('Vedic Kundali');
    expect(prompt).toContain('sw:');
    expect(prompt).toContain('output_hash');
    expect(prompt).toContain('Recount every numerical group');
    expect(prompt).toContain('without converting them into directional forecasts');
  });

  it('SYNTHESIS_LEAKAGE_PROHIBITION lists key blocked patterns', async () => {
    const mod = await import('./witness-dyad.js');
    const text = mod.SYNTHESIS_LEAKAGE_PROHIBITION;
    for (const needle of ['swiss-ephemeris', 'witness-dyad', 'passages_hash', 'Quoted Passage', 'ISO 8601']) {
      expect(text).toContain(needle);
    }
  });
});
