import { describe, it, expect } from 'vitest';
import { validatePrivateEvidenceMap, type PrivateEvidenceMap } from './evidence-map.js';

function baseMap(overrides: Partial<PrivateEvidenceMap> = {}): PrivateEvidenceMap {
  return {
    section_id: 'part1',
    subject: 'Sheshnarayan',
    language: 'en',
    timestamp_iso: '2026-09-30T00:00:00Z',
    engine_facts_consumed: [
      {
        engine_id: 'vedic-kundali',
        fields_used: ['lagna.sign', 'planets.Jupiter.dignity'],
        values_snapshot: { 'lagna.sign': 'Scorpio', 'planets.Jupiter.dignity': 'Exalted' },
        source_precision_label: 'standard',
        backend: 'swiss-ephemeris',
      },
    ],
    micro_interpretations: [
      {
        engine_id: 'vedic-kundali',
        claim: 'Jupiter is exalted, softening the Scorpio Lagna.',
        source_field: 'planets.Jupiter.dignity',
        source_value: 'Exalted',
        interpretation_tradition: 'Vedic Jyotish',
        confidence: 'framework-attributed',
        contradictions: [],
        uncertainty_note: null,
      },
    ],
    aletheios_raw: 'structural output',
    pichet_raw: 'experiential output',
    cf_passage_ids: ['sw:vk:planet:jupiter:exalted'],
    cf_passages_hash: 'a'.repeat(64),
    primary_passage_ids: [],
    receipt_output_hash: 'b'.repeat(64),
    engine_facts_hash: 'c'.repeat(64),
    audit_input_hash: 'd'.repeat(64),
    ...overrides,
  };
}

describe('validatePrivateEvidenceMap', () => {
  it('returns no blockers for a well-formed map', () => {
    expect(validatePrivateEvidenceMap(baseMap())).toEqual([]);
  });

  it('rejects a micro-interpretation whose engine is not in engine_facts_consumed', () => {
    const map = baseMap({
      micro_interpretations: [
        {
          engine_id: 'gene-keys',
          claim: 'x',
          source_field: 'active_keys.0.key_number',
          source_value: '1',
          interpretation_tradition: 'Gene Keys',
          confidence: 'framework-attributed',
          contradictions: [],
          uncertainty_note: null,
        },
      ],
    });
    const blockers = validatePrivateEvidenceMap(map);
    expect(blockers.some(b => b.includes('engine_not_in_facts_consumed:gene-keys'))).toBe(true);
  });

  it('rejects orphaned fields not present in fields_used', () => {
    const map = baseMap({
      micro_interpretations: [
        {
          engine_id: 'vedic-kundali',
          claim: 'x',
          source_field: 'planets.Sun.dignity', // not in fields_used
          source_value: 'Neutral',
          interpretation_tradition: 'Vedic Jyotish',
          confidence: 'framework-attributed',
          contradictions: [],
          uncertainty_note: null,
        },
      ],
    });
    const blockers = validatePrivateEvidenceMap(map);
    expect(blockers.some(b => b.startsWith('micro_interpretations[0]:orphaned_field:vedic-kundali.planets.Sun.dignity'))).toBe(true);
  });

  it('rejects an engine that contributed facts but has no micro-interpretation', () => {
    const map = baseMap({
      engine_facts_consumed: [
        {
          engine_id: 'vedic-kundali',
          fields_used: ['lagna.sign'],
          values_snapshot: { 'lagna.sign': 'Scorpio' },
          source_precision_label: null,
          backend: 'swiss-ephemeris',
        },
        {
          engine_id: 'human-design',
          fields_used: ['hd_type'],
          values_snapshot: { hd_type: 'Generator' },
          source_precision_label: null,
          backend: 'humdes',
        },
      ],
    });
    const blockers = validatePrivateEvidenceMap(map);
    expect(blockers).toContain('engine_facts_consumed:no_micro_interpretation:human-design');
  });

  it('rejects incomplete micro-interpretation entries', () => {
    const map = baseMap({
      micro_interpretations: [
        {
          engine_id: 'vedic-kundali',
          claim: '',
          source_field: 'planets.Jupiter.dignity',
          source_value: 'Exalted',
          interpretation_tradition: 'Vedic Jyotish',
          confidence: 'framework-attributed',
          contradictions: [],
          uncertainty_note: null,
        },
      ],
    });
    const blockers = validatePrivateEvidenceMap(map);
    expect(blockers.some(b => b.endsWith(':incomplete_entry'))).toBe(true);
  });
});
