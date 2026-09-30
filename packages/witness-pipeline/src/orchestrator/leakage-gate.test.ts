import { describe, it, expect } from 'vitest';
import { leakageGate, blockViolations } from './leakage-gate.js';

describe('leakage gate — block severity', () => {
  it('flags Cloudflare sw: passage IDs', () => {
    const res = leakageGate('The reading draws on sw:hd:type:generator:desc for context.');
    expect(res.passed).toBe(false);
    expect(res.violations.some(v => v.pattern_id === 'cf_passage_id')).toBe(true);
  });

  it('flags hyphenated internal engine identifiers', () => {
    const res = leakageGate('Per vedic-kundali the lagna is Scorpio; human-design confirms Generator.');
    expect(res.passed).toBe(false);
    const ids = res.violations.filter(v => v.pattern_id === 'engine_id_token');
    expect(ids.length).toBeGreaterThanOrEqual(2);
  });

  it('flags receipt / audit field names', () => {
    const res = leakageGate('Verified via output_hash and passages_hash bindings.');
    expect(res.passed).toBe(false);
    expect(res.violations.some(v => v.pattern_id === 'receipt_field')).toBe(true);
  });

  it('flags backend / tool names', () => {
    const res = leakageGate('The chart was computed by swiss-ephemeris and refined in pyswisseph.');
    expect(res.passed).toBe(false);
    expect(res.violations.some(v => v.pattern_id === 'tool_name')).toBe(true);
  });

  it('flags pipeline internal identifiers', () => {
    const res = leakageGate('The witness-dyad output was blocked by a Jev verdict during pass-gate review.');
    expect(res.passed).toBe(false);
    const ids = res.violations.filter(v => v.pattern_id === 'pipeline_internal').map(v => v.matched_text);
    expect(ids.length).toBeGreaterThanOrEqual(2);
  });

  it('flags prompt ordinals', () => {
    const res = leakageGate('As shown in Quoted Passage 3 and Passage 12, the theme emerges.');
    expect(res.passed).toBe(false);
    expect(res.violations.some(v => v.pattern_id === 'prompt_ordinal')).toBe(true);
  });

  it('flags acceptance / review status labels', () => {
    const res = leakageGate('The audit returned changes-required and the run was marked malformed.');
    expect(res.passed).toBe(false);
    expect(res.violations.some(v => v.pattern_id === 'review_status')).toBe(true);
  });

  it('flags JSON-shaped fragments in prose', () => {
    const res = leakageGate('The receipt reads {"source_id": "abc", "hash": "x"} at the top of the file.');
    expect(res.passed).toBe(false);
    expect(res.violations.some(v => v.pattern_id === 'json_key_fragment')).toBe(true);
  });

  it('flags receipt-style ISO 8601 with milliseconds and Z', () => {
    const res = leakageGate('Generated at 2026-09-30T04:33:11.014Z and stitched.');
    expect(res.passed).toBe(false);
    expect(res.violations.some(v => v.pattern_id === 'receipt_timestamp')).toBe(true);
  });
});

describe('leakage gate — warn severity', () => {
  it('warns on backend / precision label commentary', () => {
    const res = leakageGate('The backend selection determined the precision achieved for this run.');
    expect(res.passed).toBe(true);
    expect(res.violations.some(v => v.pattern_id === 'backend_label' && v.severity === 'warn')).toBe(true);
  });

  it('warns on register-band vocabulary', () => {
    const res = leakageGate('This chapter sits at register L1-L3 for traditional interpretation.');
    expect(res.passed).toBe(true);
    expect(res.violations.some(v => v.pattern_id === 'register_label' && v.severity === 'warn')).toBe(true);
  });
});

describe('leakage gate — readable-vocabulary allow list', () => {
  it('accepts "Vedic Kundali" as a readable name', () => {
    const res = leakageGate('The Vedic Kundali points to a Scorpio lagna and a rich karmic axis.');
    expect(res.passed).toBe(true);
    expect(res.violations.filter(v => v.severity === 'block')).toEqual([]);
  });

  it('accepts "Human Design" as a readable name', () => {
    const res = leakageGate('Your Human Design profile is 2/4 Generator, an inviting rhythm of response.');
    expect(res.passed).toBe(true);
  });

  it('accepts "Gene Keys" and "Vimshottari" as readable names', () => {
    const res = leakageGate('Your Gene Keys sequence unfolds inside a Vimshottari Jupiter Mahadasha.');
    expect(res.passed).toBe(true);
  });

  it('accepts other readable engine and symbolic-framework names', () => {
    const res = leakageGate('Transits, Biorhythm, Enneagram, Tarot, I Ching, Biofield, Nadabrahman, Numerology, Face Reading, Sacred Geometry, Sigil Forge, and Vedic Clock are named as readable chart vocabularies.');
    expect(res.passed).toBe(true);
    expect(res.violations).toEqual([]);
  });

  it('still blocks hyphenated "human-design" even alongside "Human Design"', () => {
    const res = leakageGate('Human Design is the framework; human-design is the internal engine id.');
    expect(res.passed).toBe(false);
    expect(res.violations.some(v => v.pattern_id === 'engine_id_token')).toBe(true);
  });
});

describe('leakage gate — false positive safeguards', () => {
  it('accepts ordinary braces used narratively', () => {
    const res = leakageGate('The reading holds room for {your own observations} across the year.');
    expect(res.passed).toBe(true);
  });

  it('accepts ordinary bracketed citation markers', () => {
    const res = leakageGate('This synthesis draws on the classical commentaries [see Part IX] noted earlier.');
    expect(res.passed).toBe(true);
  });

  it('accepts narrative dates without receipt-style timestamps', () => {
    const res = leakageGate('The pivot arrives in September 2026, with a follow-on window in May 2027.');
    expect(res.passed).toBe(true);
  });

  it('accepts a clean symbolic paragraph', () => {
    const text = [
      'Your Vedic Kundali frames a Scorpio Lagna held by a Saraswati Yoga.',
      'Human Design speaks of a 2/4 Generator inviting response, and Gene Keys names',
      'the sequence that keeps opening across your life. The Vimshottari Mahadasha',
      'in Jupiter now steadies what the Rahu-Ketu axis has stirred.',
    ].join(' ');
    const res = leakageGate(text);
    expect(res.passed).toBe(true);
    expect(res.violations.filter(v => v.severity === 'block')).toEqual([]);
  });
});

describe('leakage gate — determinism and helpers', () => {
  it('produces identical results for identical inputs', () => {
    const t = 'A sample sentence with sw:hd:type:generator:desc leak.';
    const a = leakageGate(t);
    const b = leakageGate(t);
    expect(a).toEqual(b);
  });

  it('sorts violations by line then severity', () => {
    const text = [
      'Line 1: swiss-ephemeris block.',              // line 1
      'Line 2: precision_achieved warn.',            // line 2
      'Line 3: sw:hd:type:generator:desc block.',    // line 3
    ].join('\n');
    const res = leakageGate(text);
    const lines = res.violations.map(v => v.line_number);
    expect(lines).toEqual([...lines].sort((a, b) => a - b));
  });

  it('blockViolations returns only block severity entries', () => {
    const res = leakageGate('The backend chose sw:hd:type:generator:desc for this section.');
    const only = blockViolations(res);
    expect(only.every(v => v.severity === 'block')).toBe(true);
    expect(only.length).toBeGreaterThanOrEqual(1);
  });
});
