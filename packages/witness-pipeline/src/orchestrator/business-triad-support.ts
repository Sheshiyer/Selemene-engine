// ─── Business Triad Reference — Support Helpers ───────────────────────────
// New, independent module for the business-triad-reference route. Never
// edits the individual reference route. Intentionally narrow surface:
//   - loadTriadEngines(rootDir): read the three saved per-partner engine
//     bundles without merging or renaming.
//   - unionCorpusIdsForTriad(enginesBySubject): union deidentified corpus
//     IDs across the three partners, preserving first-seen order.
//   - localizeBusinessTriadTitles(mode, language): apply French titles to
//     the twelve canonical business-triad sections (opening + part1..part11).
//
// Nothing here touches individual-mode helpers, nothing collapses per-
// subject engine records, and nothing extracts subject names, birth data,
// or personal identifiers into corpus IDs.

import { readFile } from 'node:fs/promises';
import { join } from 'node:path';
import { selectCorpusIds } from '../grounding/index.js';

/** Canonical partner slug set for the business triad reference route. */
export const BUSINESS_TRIAD_SLUGS = [
  'gary-abitbol',
  'mohan-kumar',
  'sheshnarayan',
] as const;

/** Display names paired 1:1 with BUSINESS_TRIAD_SLUGS order. */
export const BUSINESS_TRIAD_DISPLAY_NAMES: Readonly<Record<TriadSlug, string>> = {
  'gary-abitbol': 'Gary Abitbol',
  'mohan-kumar': 'Mohan Kumar',
  sheshnarayan: 'Sheshnarayan Cumbipuram Nateshan',
};

export type TriadSlug = (typeof BUSINESS_TRIAD_SLUGS)[number];

/** One partner's saved engine bundle. Roles are always 'partner'. */
export interface TriadEngineBundle {
  subject: TriadSlug;
  name: string;
  role: 'partner';
  engines: Array<{ engine_id: string; result: unknown; metadata?: unknown; _error?: unknown; [key: string]: unknown }>;
  sourcePath: string;
  sourceBytes: string;
}

/**
 * Read the three per-partner engine bundles from
 *   <rootDir>/reports-reference/engines/<slug>.prod.json
 * Returns them in canonical BUSINESS_TRIAD_SLUGS order. Never collapses,
 * renames, or merges engines across subjects.
 */
export async function loadTriadEngines(rootDir: string): Promise<TriadEngineBundle[]> {
  const bundles: TriadEngineBundle[] = [];
  for (const slug of BUSINESS_TRIAD_SLUGS) {
    const sourcePath = join(rootDir, 'reports-reference/engines', `${slug}.prod.json`);
    const bytes = await readFile(sourcePath, 'utf8');
    let parsed: unknown;
    try {
      parsed = JSON.parse(bytes);
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      throw new Error(`Malformed engine bundle for ${slug}: ${message}`);
    }
    if (!Array.isArray(parsed)) {
      throw new Error(`Engine bundle for ${slug} must be an array of {engine_id,result} records`);
    }
    const engineIds = new Set<string>();
    const engines: TriadEngineBundle['engines'] = [];
    for (const record of parsed as Array<Record<string, unknown>>) {
      const engineId = typeof record?.engine_id === 'string' ? record.engine_id : '';
      if (!engineId) {
        throw new Error(`Engine record without engine_id in bundle for ${slug}`);
      }
      if (engineIds.has(engineId)) {
        throw new Error(`Duplicate engine_id '${engineId}' in bundle for ${slug}`);
      }
      engineIds.add(engineId);
      engines.push({ ...record, engine_id: engineId, result: record.result });
    }
    for (const required of ['human-design', 'vedic-kundali']) {
      const record = engines.find(e => e.engine_id === required);
      if (!record || record._error || !record.result || typeof record.result !== 'object' || Array.isArray(record.result) || Object.keys(record.result).length === 0) {
        throw new Error(`Missing or failed required engine '${required}' for ${slug}`);
      }
    }
    bundles.push({
      subject: slug,
      name: BUSINESS_TRIAD_DISPLAY_NAMES[slug],
      role: 'partner',
      engines,
      sourcePath,
      sourceBytes: bytes,
    });
  }
  return bundles;
}

function firstEngineByIds(
  bundle: TriadEngineBundle,
  ids: readonly string[],
): unknown | undefined {
  for (const id of ids) {
    const found = bundle.engines.find((e) => e.engine_id === id);
    if (found) return found.result;
  }
  return undefined;
}

function extractGeneKeyGates(gk: any): number[] {
  if (!gk) return [];
  const raw = Array.isArray(gk.active_keys) ? gk.active_keys : Array.isArray(gk.gates) ? gk.gates : [];
  const nums: number[] = [];
  for (const item of raw) {
    if (typeof item === 'number') {
      if (item >= 1 && item <= 64) nums.push(item);
      continue;
    }
    if (item && typeof item === 'object') {
      const cand =
        (item as any).key_number ??
        (item as any).key ??
        (item as any).gene_key ??
        (item as any).number ??
        (item as any).gate;
      const n = Number.parseInt(String(cand), 10);
      if (Number.isFinite(n) && n >= 1 && n <= 64) nums.push(n);
    }
  }
  return [...new Set(nums)];
}

function planetIdsForVedic(vedic: any): string[] {
  if (!vedic || typeof vedic !== 'object') return [];
  const planets = (vedic.planets && typeof vedic.planets === 'object') ? vedic.planets : {};
  const ids: string[] = [];
  for (const name of Object.keys(planets)) {
    const slug = String(name).toLowerCase().replace(/[^a-z0-9_-]/g, '');
    if (slug) ids.push(`sw:vim:planet:${slug}:desc`);
  }
  return ids;
}

function planetaryNakshatraIds(vedic: any): string[] {
  if (!vedic || typeof vedic !== 'object' || !vedic.planets) return [];
  const ids: string[] = [];
  for (const planet of Object.values(vedic.planets as Record<string, any>)) {
    const num = planet && typeof planet === 'object' ? (planet as any).nakshatra_num : undefined;
    const n = Number(num);
    if (Number.isInteger(n) && n >= 1 && n <= 27) ids.push(`sw:vim:nakshatra:${n}:desc`);
  }
  return ids;
}

/**
 * Union of deidentified corpus IDs across a triad. For every partner:
 *   1) `selectCorpusIds` with that partner's (hd, gk, vim) — pure, no PII;
 *   2) planetary IDs derived from that partner's vedic-kundali planet map;
 *   3) planetary-nakshatra IDs from each supplied vedic planet.
 * The three lists are unioned in canonical partner order, dedup preserves
 * first-seen order across the triad, and NO subject names, coordinates,
 * or birth-time fields are transmitted or embedded.
 */
export function unionCorpusIdsForTriad(
  enginesBySubject: readonly TriadEngineBundle[],
): { ids: string[]; perSubjectCounts: Array<{ subject: TriadSlug; count: number }> } {
  const seen = new Set<string>();
  const ordered: string[] = [];
  const perSubjectCounts: Array<{ subject: TriadSlug; count: number }> = [];
  for (const bundle of enginesBySubject) {
    const hd = firstEngineByIds(bundle, ['human-design']);
    const gk = firstEngineByIds(bundle, ['gene-keys']);
    const vim = firstEngineByIds(bundle, ['vimshottari']);
    const vedic = firstEngineByIds(bundle, ['vedic-kundali']);
    const perSubjectIds: string[] = [];
    const selected = selectCorpusIds({
      'human-design': hd
        ? {
            type: (hd as any).hd_type ?? (hd as any).type,
            profile: (hd as any).profile,
            authority: (hd as any).authority,
            defined_centers: (hd as any).defined_centers,
            channels: (hd as any).active_channels ?? (hd as any).channels,
          }
        : undefined,
      'gene-keys': gk ? { gates: extractGeneKeyGates(gk) } : undefined,
      vimshottari: vim
        ? {
            current_dasha:
              (vim as any).current_period?.mahadasha?.planet ?? (vim as any).current_dasha,
            current_antardasha:
              (vim as any).current_period?.antardasha?.planet ?? (vim as any).current_antardasha,
            nakshatra: (vim as any).birth_nakshatra ?? (vim as any).nakshatra,
          }
        : undefined,
    });
    perSubjectIds.push(...selected.ids);
    perSubjectIds.push(...planetIdsForVedic(vedic));
    perSubjectIds.push(...planetaryNakshatraIds(vedic));
    let count = 0;
    for (const id of perSubjectIds) {
      if (typeof id !== 'string' || id.length === 0) continue;
      // Deidentification guard — no subject-derived tokens should ever
      // reach the corpus query. The known safe prefixes are:
      if (!id.startsWith('sw:')) continue;
      count++;
      if (!seen.has(id)) {
        seen.add(id);
        ordered.push(id);
      }
    }
    perSubjectCounts.push({ subject: bundle.subject, count });
  }
  return { ids: ordered, perSubjectCounts };
}

// ─── Localization ───────────────────────────────────────────────────────

const BUSINESS_TRIAD_FRENCH_TITLES: Readonly<Record<string, string>> = {
  opening: 'Ouverture — Cadre, partenaires et systèmes disponibles',
  part1: 'Partie I — Identités des partenaires et carte de convergence',
  part2: 'Partie II — Fondements individuels (trois cartes)',
  part3: 'Partie III — Relations par paires (trois paires)',
  part4: 'Partie IV — Rôles et styles de décision',
  part5: 'Partie V — Ressources, argent et échange de valeur',
  part6: 'Partie VI — Collaboration et communication',
  part7: 'Partie VII — Capacité, rythme et charge soutenable',
  part8: 'Partie VIII — Contexte d’équipe et champ des parties prenantes',
  part9: 'Partie IX — Instantanés de coordination datés',
  part10: 'Partie X — Pratiques et anti-dépendance',
  part11: 'Partie XI — Synthèse triadique',
};

export const BUSINESS_TRIAD_SECTION_IDS: readonly string[] = Object.keys(
  BUSINESS_TRIAD_FRENCH_TITLES,
);

/**
 * Apply French titles to the twelve canonical business-triad sections.
 * English is a no-op. Never reuses the individual reference route's
 * `localizeReferenceTitles` because that map is domain-specific to the
 * individual mode (career/health/family). Section IDs and target_words
 * are never mutated.
 */
export function localizeBusinessTriadTitles(
  mode: { frontmatter: { pass_plan: Array<{ id: string; title: string }> } },
  language: string,
): void {
  if (language !== 'en' && language !== 'fr') {
    throw new Error('Unsupported business-triad language: ' + language);
  }
  if (language === 'en') return;
  for (const pass of mode.frontmatter.pass_plan) {
    const localized = BUSINESS_TRIAD_FRENCH_TITLES[pass.id];
    if (!localized) {
      throw new Error('Unknown business-triad section: ' + pass.id);
    }
    pass.title = localized;
  }
}
