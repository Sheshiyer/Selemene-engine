import { describe, it, expect, beforeAll } from 'vitest';
import { mkdtempSync, writeFileSync, mkdirSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import {
  BUSINESS_TRIAD_SLUGS,
  loadTriadEngines,
  unionCorpusIdsForTriad,
  type TriadEngineBundle,
} from './business-triad-support.js';

function mockEngineBundle(overrides: {
  hdType: string;
  profile: string;
  authority: string;
  channels: string[];
  gates: number[];
  dasha: string;
  antardasha: string;
  planets: Record<string, { nakshatra_num?: number }>;
}) {
  return [
    {
      engine_id: 'human-design',
      result: {
        hd_type: overrides.hdType,
        profile: overrides.profile,
        authority: overrides.authority,
        defined_centers: ['ajna', 'throat'],
        active_channels: overrides.channels,
      },
    },
    {
      engine_id: 'gene-keys',
      result: {
        active_keys: overrides.gates.map((g) => ({ key_number: g })),
      },
    },
    {
      engine_id: 'vimshottari',
      result: {
        current_period: {
          mahadasha: { planet: overrides.dasha },
          antardasha: { planet: overrides.antardasha },
        },
        birth_nakshatra: { number: 5 },
      },
    },
    {
      engine_id: 'vedic-kundali',
      result: { planets: overrides.planets },
    },
  ];
}

describe('loadTriadEngines', () => {
  let root: string;
  beforeAll(() => {
    root = mkdtempSync(join(tmpdir(), 'triad-engines-'));
    const dir = join(root, 'reports-reference/engines');
    mkdirSync(dir, { recursive: true });
    for (const slug of BUSINESS_TRIAD_SLUGS) {
      writeFileSync(
        join(dir, `${slug}.prod.json`),
        JSON.stringify(
          mockEngineBundle({
            hdType: slug === 'gary-abitbol' ? 'Projector' : slug === 'mohan-kumar' ? 'Manifestor' : 'Generator',
            profile: slug === 'sheshnarayan' ? '5/1' : '2/4',
            authority: 'Sacral_Authority',
            channels: slug === 'gary-abitbol' ? ['1-8'] : ['20-34'],
            gates: slug === 'gary-abitbol' ? [1, 8, 11] : slug === 'mohan-kumar' ? [11, 22] : [22, 33],
            dasha: slug === 'gary-abitbol' ? 'Venus' : slug === 'mohan-kumar' ? 'Mercury' : 'Jupiter',
            antardasha: 'Ketu',
            planets: slug === 'gary-abitbol'
              ? { Sun: { nakshatra_num: 5 }, Moon: { nakshatra_num: 12 } }
              : slug === 'mohan-kumar'
                ? { Sun: { nakshatra_num: 12 }, Ketu: { nakshatra_num: 7 } }
                : { Sun: { nakshatra_num: 18 }, Jupiter: { nakshatra_num: 5 } },
          }),
          null,
          2,
        ),
      );
    }
  });

  it('returns three partners in canonical order with role=partner', async () => {
    const bundles = await loadTriadEngines(root);
    expect(bundles).toHaveLength(3);
    expect(bundles.map((b) => b.subject)).toEqual([...BUSINESS_TRIAD_SLUGS]);
    for (const bundle of bundles) {
      expect(bundle.role).toBe('partner');
      expect(bundle.name.length).toBeGreaterThan(0);
      expect(bundle.engines.length).toBe(4);
    }
  });

  it('never collapses engines by engine_id across subjects', async () => {
    const bundles = await loadTriadEngines(root);
    const perSubjectHd = bundles.map((b) => b.engines.find((e) => e.engine_id === 'human-design'));
    const seenTypes = perSubjectHd.map((hd: any) => hd?.result?.hd_type);
    expect(new Set(seenTypes).size).toBe(3);
  });

  it('preserves source envelope fields instead of silently projecting them away', async () => {
    const path = join(root, 'reports-reference/engines/gary-abitbol.prod.json');
    const { readFileSync } = await import('node:fs');
    const original = readFileSync(path, 'utf8');
    try {
      const records = JSON.parse(original);
      records[0].calculated_at = '2026-09-28T12:34:56Z';
      records[0].source_provenance = { version: 'saved' };
      writeFileSync(path, JSON.stringify(records));
      const bundles = await loadTriadEngines(root);
      expect(bundles[0].engines[0]).toEqual(records[0]);
    } finally { writeFileSync(path, original); }
  });

  it.each(['missing', 'failed', 'empty'])('rejects %s required partner data', async (kind) => {
    const path = join(root, 'reports-reference/engines/mohan-kumar.prod.json');
    const { readFileSync } = await import('node:fs');
    const original = readFileSync(path, 'utf8');
    try {
      let records = JSON.parse(original);
      if (kind === 'missing') records = records.filter((r: any) => r.engine_id !== 'human-design');
      else if (kind === 'failed') records[0]._error = 'unavailable';
      else records[0].result = {};
      writeFileSync(path, JSON.stringify(records));
      await expect(loadTriadEngines(root)).rejects.toThrow(/required engine.*mohan-kumar/);
    } finally { writeFileSync(path, original); }
  });

  it('rejects a duplicate engine_id inside one bundle', async () => {
    const badRoot = mkdtempSync(join(tmpdir(), 'triad-bad-'));
    const dir = join(badRoot, 'reports-reference/engines');
    mkdirSync(dir, { recursive: true });
    for (const slug of BUSINESS_TRIAD_SLUGS) {
      const good = mockEngineBundle({
        hdType: 'Generator',
        profile: '2/4',
        authority: 'Sacral_Authority',
        channels: ['1-8'],
        gates: [1],
        dasha: 'Venus',
        antardasha: 'Ketu',
        planets: { Sun: { nakshatra_num: 5 } },
      });
      const bad = slug === 'sheshnarayan' ? [...good, good[0]] : good;
      writeFileSync(join(dir, `${slug}.prod.json`), JSON.stringify(bad));
    }
    await expect(loadTriadEngines(badRoot)).rejects.toThrow(/Duplicate engine_id/);
    rmSync(badRoot, { recursive: true, force: true });
  });
});

describe('unionCorpusIdsForTriad', () => {
  function bundleForSlug(slug: (typeof BUSINESS_TRIAD_SLUGS)[number], seed: number): TriadEngineBundle {
    return {
      subject: slug,
      name: slug,
      role: 'partner',
      sourcePath: '/tmp/mock',
      sourceBytes: '[]',
      engines: mockEngineBundle({
        hdType: 'Generator',
        profile: '2/4',
        authority: 'Sacral_Authority',
        channels: ['1-8'],
        gates: [seed, seed + 1],
        dasha: 'Venus',
        antardasha: 'Ketu',
        planets: { Sun: { nakshatra_num: seed }, Moon: { nakshatra_num: seed + 1 } },
      }),
    };
  }

  it('deduplicates deidentified IDs across three partners preserving first-seen order', () => {
    const bundles = [
      bundleForSlug('gary-abitbol', 5),
      bundleForSlug('mohan-kumar', 6),
      bundleForSlug('sheshnarayan', 7),
    ];
    const { ids, perSubjectCounts } = unionCorpusIdsForTriad(bundles);
    expect(ids.length).toBeGreaterThan(0);
    // no duplicates
    expect(new Set(ids).size).toBe(ids.length);
    // every id must be deidentified (sw:*)
    for (const id of ids) {
      expect(id.startsWith('sw:')).toBe(true);
      expect(id).not.toMatch(/gary|mohan|shesh/i);
    }
    // three per-subject counts, each > 0
    expect(perSubjectCounts.map((c) => c.subject)).toEqual([
      'gary-abitbol',
      'mohan-kumar',
      'sheshnarayan',
    ]);
    for (const c of perSubjectCounts) {
      expect(c.count).toBeGreaterThan(0);
    }
  });

  it('never leaks subject names into the union', () => {
    const bundles = [
      bundleForSlug('gary-abitbol', 3),
      bundleForSlug('mohan-kumar', 4),
      bundleForSlug('sheshnarayan', 5),
    ];
    const { ids } = unionCorpusIdsForTriad(bundles);
    const blob = ids.join(' ');
    for (const forbidden of ['gary', 'mohan', 'sheshnarayan', 'abitbol', 'kumar']) {
      expect(blob.toLowerCase()).not.toContain(forbidden);
    }
  });

  it('produces per-subject planet IDs distinct for the two Sun-nakshatra values', () => {
    const bundles = [
      bundleForSlug('gary-abitbol', 5),
      bundleForSlug('mohan-kumar', 12),
      bundleForSlug('sheshnarayan', 18),
    ];
    const { ids } = unionCorpusIdsForTriad(bundles);
    expect(ids).toContain('sw:vim:nakshatra:5:desc');
    expect(ids).toContain('sw:vim:nakshatra:12:desc');
    expect(ids).toContain('sw:vim:nakshatra:18:desc');
  });
});
