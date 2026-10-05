// ─── corpus-id-selector — unit tests ─────────────────────────────────────
// Verifies pure ID derivation from engine outputs. No I/O.

import { describe, it, expect } from 'vitest';
import { selectCorpusIds } from './corpus-id-selector.js';
import type { EngineOutputsForGrounding } from './types.js';

// ─── Gene Keys ────────────────────────────────────────────────────────────────

describe('selectCorpusIds — Gene Keys', () => {
  it('emits all four field IDs for each gate number', () => {
    const result = selectCorpusIds({
      'gene-keys': { gates: [17, 4] },
    });
    expect(result.ids).toContain('sw:gk:17:shadow_description');
    expect(result.ids).toContain('sw:gk:17:gift_description');
    expect(result.ids).toContain('sw:gk:17:siddhi_description');
    expect(result.ids).toContain('sw:gk:17:life_theme');
    expect(result.ids).toContain('sw:gk:4:shadow_description');
    expect(result.ids).toHaveLength(8);
  });

  it('handles object-form gates with number property', () => {
    const result = selectCorpusIds({
      'gene-keys': { gates: [{ number: 22 }, { number: 36 }] },
    });
    expect(result.ids).toContain('sw:gk:22:shadow_description');
    expect(result.ids).toContain('sw:gk:36:gift_description');
  });

  it('deduplicates gate IDs across gates and profile_sequence', () => {
    const result = selectCorpusIds({
      'gene-keys': { gates: [17], profile_sequence: [17, 64] },
    });
    // 17 appears in both but should only have 4 IDs total for 17
    const gate17Ids = result.ids.filter((id) => id.startsWith('sw:gk:17:'));
    expect(gate17Ids).toHaveLength(4);
    // 64 also gets picked up
    expect(result.ids).toContain('sw:gk:64:shadow_description');
  });

  it('ignores invalid gate numbers (0, 65)', () => {
    const result = selectCorpusIds({
      'gene-keys': { gates: [0, 65, 1] },
    });
    expect(result.ids.filter((id) => id.startsWith('sw:gk:0:'))).toHaveLength(0);
    expect(result.ids.filter((id) => id.startsWith('sw:gk:65:'))).toHaveLength(0);
    expect(result.ids).toContain('sw:gk:1:shadow_description');
  });

  it('provides rationale for every selected ID', () => {
    const result = selectCorpusIds({ 'gene-keys': { gates: [5] } });
    for (const id of result.ids) {
      expect(result.rationale[id]).toBeTruthy();
    }
  });
});

// ─── Human Design ─────────────────────────────────────────────────────────────

describe('selectCorpusIds — Human Design', () => {
  it('emits type, profile, authority, centers, channels', () => {
    const input: EngineOutputsForGrounding = {
      'human-design': {
        type: 'Generator',
        profile: '2/4',
        authority: 'Sacral_Authority',
        defined_centers: ['Sacral', 'Solar Plexus'],
        channels: ['1-8', '2-14'],
      },
    };
    const result = selectCorpusIds(input);
    expect(result.ids).toContain('sw:hd:type:generator:desc');
    expect(result.ids).toContain('sw:hd:prof:2-4:desc');
    expect(result.ids).toContain('sw:hd:auth:sacral-authority:desc');
    expect(result.ids).toContain('sw:hd:ctr:sacral:desc');
    expect(result.ids).toContain('sw:hd:ctr:solar plexus:desc');
    expect(result.ids).toContain('sw:hd:ch:1-8:desc');
    expect(result.ids).toContain('sw:hd:ch:2-14:desc');
  });

  it('normalizes profile separators: 2-4 and 2_4 both become 2_4', () => {
    const r1 = selectCorpusIds({ 'human-design': { profile: '2-4' } });
    const r2 = selectCorpusIds({ 'human-design': { profile: '2_4' } });
    const r3 = selectCorpusIds({ 'human-design': { profile: '2/4' } });
    expect(r1.ids).toContain('sw:hd:prof:2-4:desc');
    expect(r2.ids).toContain('sw:hd:prof:2-4:desc');
    expect(r3.ids).toContain('sw:hd:prof:2-4:desc');
  });

  it('normalizes type with spaces: Manifesting Generator → manifesting_generator', () => {
    const result = selectCorpusIds({ 'human-design': { type: 'Manifesting Generator' } });
    expect(result.ids).toContain('sw:hd:type:manifesting-generator:desc');
  });

  it('normalizes channel from gate-array form [1, 8]', () => {
    const result = selectCorpusIds({
      'human-design': {
        channels: [{ gates: [1, 8] }],
      },
    });
    expect(result.ids).toContain('sw:hd:ch:1-8:desc');
  });

  it('normalizes channel gate order: [8,1] still → 1-8', () => {
    const result = selectCorpusIds({
      'human-design': { channels: [{ gates: [8, 1] }] },
    });
    expect(result.ids).toContain('sw:hd:ch:1-8:desc');
  });

  it('normalizes center names with spaces: Solar Plexus → solar_plexus', () => {
    const result = selectCorpusIds({
      'human-design': { defined_centers: ['Solar Plexus', 'Head Center'] },
    });
    expect(result.ids).toContain('sw:hd:ctr:solar plexus:desc');
    expect(result.ids).toContain('sw:hd:ctr:head center:desc');
  });

  it('deduplicates center and channel IDs', () => {
    const result = selectCorpusIds({
      'human-design': {
        defined_centers: ['Sacral', 'Sacral'],
        channels: ['1-8', '1-8'],
      },
    });
    expect(result.ids.filter((id) => id === 'sw:hd:ctr:sacral:desc')).toHaveLength(1);
    expect(result.ids.filter((id) => id === 'sw:hd:ch:1-8:desc')).toHaveLength(1);
  });
});

// ─── Vimshottari ──────────────────────────────────────────────────────────────

describe('selectCorpusIds — Vimshottari', () => {
  it('emits nakshatra and planet IDs', () => {
    const result = selectCorpusIds({
      vimshottari: {
        nakshatra_number: 7,
        current_dasha: 'Jupiter',
        current_antardasha: 'Moon',
      },
    });
    expect(result.ids).toContain('sw:vim:nakshatra:7:desc');
    expect(result.ids).toContain('sw:vim:planet:jupiter:desc');
    expect(result.ids).toContain('sw:vim:planet:moon:desc');
  });

  it('handles object-form nakshatra', () => {
    const result = selectCorpusIds({
      vimshottari: { nakshatra: { number: 12, name: 'Uttara Phalguni' } },
    });
    expect(result.ids).toContain('sw:vim:nakshatra:12:desc');
  });

  it('deduplicates dasha and antardasha when same planet', () => {
    const result = selectCorpusIds({
      vimshottari: {
        nakshatra_number: 3,
        current_dasha: 'Ketu',
        current_antardasha: 'Ketu',
      },
    });
    expect(result.ids.filter((id) => id === 'sw:vim:planet:ketu:desc')).toHaveLength(1);
  });

  it('ignores nakshatra numbers out of range 1-27', () => {
    const result = selectCorpusIds({
      vimshottari: { nakshatra_number: 0 },
    });
    expect(result.ids.filter((id) => id.startsWith('sw:vim:nakshatra:'))).toHaveLength(0);
  });
});

// ─── Combined outputs ─────────────────────────────────────────────────────────

describe('selectCorpusIds — combined engines', () => {
  it('merges IDs from all three engines without duplicates', () => {
    const result = selectCorpusIds({
      'human-design': { type: 'Projector', profile: '1/3' },
      'gene-keys': { gates: [10] },
      vimshottari: { nakshatra_number: 1, current_dasha: 'Sun' },
    });
    expect(result.ids).toContain('sw:hd:type:projector:desc');
    expect(result.ids).toContain('sw:hd:prof:1-3:desc');
    expect(result.ids).toContain('sw:gk:10:shadow_description');
    expect(result.ids).toContain('sw:vim:nakshatra:1:desc');
    expect(result.ids).toContain('sw:vim:planet:sun:desc');
    // All unique
    expect(new Set(result.ids).size).toBe(result.ids.length);
  });

  it('returns empty when engine outputs are empty objects', () => {
    const result = selectCorpusIds({
      'human-design': {},
      'gene-keys': {},
      vimshottari: {},
    });
    expect(result.ids).toHaveLength(0);
  });

  it('returns empty when called with empty object', () => {
    const result = selectCorpusIds({});
    expect(result.ids).toHaveLength(0);
    expect(Object.keys(result.rationale)).toHaveLength(0);
  });
});
