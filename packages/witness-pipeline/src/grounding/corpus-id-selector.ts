// ─── Corpus ID Selector ───────────────────────────────────────────────────
// Pure function — derives exact witness-wisdom-corpus IDs from Selemene engine
// outputs. No subject names, birth data, or personal identifiers ever passed.
//
// ID schemes (mirrors ingest-wisdom-corpus.ts exactly):
//   Gene Keys:    sw:gk:<number>:shadow_description | gift_description | siddhi_description | life_theme
//   HD type:      sw:hd:type:<slug>:desc
//   HD profile:   sw:hd:prof:<slug>:desc
//   HD authority: sw:hd:auth:<slug>:desc
//   HD center:    sw:hd:ctr:<slug>:desc
//   HD channel:   sw:hd:ch:<slug>:desc
//   Vimshottari:  sw:vim:nakshatra:<number>:desc
//                 sw:vim:planet:<slug>:desc
//
// Normalisation rules:
//   - All slugs lowercased, spaces → '_', special chars stripped
//   - Profile '2/4' or '2-4' → '2_4'
//   - Authority 'Sacral_Authority' → 'sacral_authority'
//   - Center 'Solar Plexus' → 'solar_plexus'
//   - Channel '1-8' or [1,8] → '1-8' (always hyphen-joined, lower)
//   - Planet 'Ketu' → 'ketu'

import type {
  EngineOutputsForGrounding,
  HumanDesignEngineOutput,
  GeneKeysEngineOutput,
  VimshottariEngineOutput,
  SelectedCorpusIds,
} from './types.js';

// ─── ID normalisation helpers ─────────────────────────────────────────────────

function normalizeSlug(raw: string): string {
  return raw
    .toLowerCase()
    .replace(/[\s/]+/g, '_')
    .replace(/[^a-z0-9_-]/g, '')
    .replace(/_+/g, '_')
    .replace(/^_+|_+$/g, '');
}

function normalizeProfile(raw: string): string {
  // '2/4', '2-4', '2_4' all → '2_4'
  return raw.replace(/[^a-z0-9]/gi, '-').toLowerCase();
}

function normalizeChannelId(raw: string | [number, number] | number[]): string {
  if (typeof raw === 'string') {
    const trimmed = raw.trim();
    const parts = trimmed.split(/[-_/]/);
    if (parts.length === 2) {
      const a = parseInt(parts[0], 10);
      const b = parseInt(parts[1], 10);
      if (!isNaN(a) && !isNaN(b)) {
        return `${Math.min(a, b)}-${Math.max(a, b)}`;
      }
    }
    return trimmed.toLowerCase();
  }
  const nums = raw as number[];
  if (nums.length >= 2) {
    const a = nums[0];
    const b = nums[1];
    return `${Math.min(a, b)}-${Math.max(a, b)}`;
  }
  return String(nums[0] ?? 0);
}

function normalizeGateNumber(raw: number | { number: number }): number {
  return typeof raw === 'number' ? raw : raw.number;
}

// ─── Gene Keys IDs ────────────────────────────────────────────────────────────

const GK_FIELDS = ['shadow_description', 'gift_description', 'siddhi_description', 'life_theme'] as const;

function geneKeyIds(
  gates: Array<number | { number: number }>,
  rationale: Record<string, string>,
): string[] {
  const ids: string[] = [];
  for (const gate of gates) {
    const num = normalizeGateNumber(gate);
    if (num < 1 || num > 64) continue;
    for (const field of GK_FIELDS) {
      const id = `sw:gk:${num}:${field}`;
      ids.push(id);
      rationale[id] = `Gene Key ${num} ${field.replace('_', ' ')}`;
    }
  }
  return ids;
}

function selectGeneKeysIds(
  gk: GeneKeysEngineOutput,
  rationale: Record<string, string>,
): string[] {
  const ids: string[] = [];
  if (gk.gates && Array.isArray(gk.gates) && gk.gates.length > 0) {
    ids.push(...geneKeyIds(gk.gates as Array<number | { number: number }>, rationale));
  }
  // Profile sequence gates (if distinct from regular gates)
  if (gk.profile_sequence && Array.isArray(gk.profile_sequence)) {
    for (const num of gk.profile_sequence) {
      if (typeof num !== 'number' || num < 1 || num > 64) continue;
      for (const field of GK_FIELDS) {
        const id = `sw:gk:${num}:${field}`;
        if (!ids.includes(id)) {
          ids.push(id);
          rationale[id] = `Gene Key ${num} ${field.replace('_', ' ')} (profile sequence)`;
        }
      }
    }
  }
  return ids;
}

// ─── Human Design IDs ────────────────────────────────────────────────────────

function selectHumanDesignIds(
  hd: HumanDesignEngineOutput,
  rationale: Record<string, string>,
): string[] {
  const ids: string[] = [];

  if (hd.type) {
    const slug = hd.type.replace(/([a-z])([A-Z])/g, '$1-$2').toLowerCase().replace(/[ _]/g, '-');
    const id = `sw:hd:type:${slug}:desc`;
    ids.push(id);
    rationale[id] = `HD type: ${hd.type}`;
  }

  if (hd.profile) {
    const slug = normalizeProfile(hd.profile);
    const id = `sw:hd:prof:${slug}:desc`;
    ids.push(id);
    rationale[id] = `HD profile: ${hd.profile}`;
  }

  if (hd.authority) {
    const base = hd.authority.replace(/([a-z])([A-Z])/g, '$1-$2').toLowerCase().replace(/[ _]/g, '-');
    const slug = base.endsWith('-authority') ? base : `${base}-authority`;
    const id = `sw:hd:auth:${slug}:desc`;
    ids.push(id);
    rationale[id] = `HD authority: ${hd.authority}`;
  }

  if (hd.defined_centers && Array.isArray(hd.defined_centers)) {
    for (const center of hd.defined_centers) {
      if (typeof center !== 'string') continue;
      const slug = center.replace('SolarPlexus', 'Solar Plexus').toLowerCase().replace(/_/g, '-');
      const id = `sw:hd:ctr:${slug}:desc`;
      if (!ids.includes(id)) {
        ids.push(id);
        rationale[id] = `HD defined center: ${center}`;
      }
    }
  }

  if (hd.channels && Array.isArray(hd.channels)) {
    for (const ch of hd.channels) {
      let channelKey: string;
      if (typeof ch === 'string') {
        channelKey = normalizeChannelId(ch);
      } else if (
        typeof ch === 'object' &&
        ch !== null &&
        'gates' in ch &&
        Array.isArray((ch as { gates: unknown }).gates)
      ) {
        const gates = (ch as { gates: unknown }).gates as number[];
        channelKey = normalizeChannelId(gates);
      } else if (
        typeof ch === 'object' &&
        ch !== null &&
        'channel' in ch &&
        typeof (ch as { channel: unknown }).channel === 'string'
      ) {
        channelKey = normalizeChannelId((ch as { channel: string }).channel);
      } else {
        continue;
      }
      const id = `sw:hd:ch:${channelKey}:desc`;
      if (!ids.includes(id)) {
        ids.push(id);
        rationale[id] = `HD channel: ${channelKey}`;
      }
    }
  }

  return ids;
}

// ─── Vimshottari IDs ──────────────────────────────────────────────────────────

function selectVimshottariIds(
  vim: VimshottariEngineOutput,
  rationale: Record<string, string>,
): string[] {
  const ids: string[] = [];

  if (vim.nakshatra_number !== undefined) {
    const num = vim.nakshatra_number;
    if (num >= 1 && num <= 27) {
      const id = `sw:vim:nakshatra:${num}:desc`;
      ids.push(id);
      rationale[id] = `Vimshottari nakshatra #${num}`;
    }
  } else if (vim.nakshatra) {
    if (typeof vim.nakshatra === 'object' && vim.nakshatra.number) {
      const num = vim.nakshatra.number;
      if (num >= 1 && num <= 27) {
        const id = `sw:vim:nakshatra:${num}:desc`;
        ids.push(id);
        rationale[id] = `Vimshottari nakshatra #${num}`;
      }
    }
    // If nakshatra is a string name, we can't derive the number without a lookup table
    // — so we skip it here. Callers should pass nakshatra_number when available.
  }

  if (vim.current_dasha) {
    const slug = normalizeSlug(vim.current_dasha);
    const id = `sw:vim:planet:${slug}:desc`;
    ids.push(id);
    rationale[id] = `Vimshottari current dasha: ${vim.current_dasha}`;
  }

  if (vim.current_antardasha) {
    const slug = normalizeSlug(vim.current_antardasha);
    const id = `sw:vim:planet:${slug}:desc`;
    if (!ids.includes(id)) {
      ids.push(id);
      rationale[id] = `Vimshottari antardasha: ${vim.current_antardasha}`;
    }
  }

  return ids;
}

// ─── Main selector ────────────────────────────────────────────────────────────

/**
 * Derive exact corpus IDs from Selemene engine outputs.
 * Pure function — no side effects, no network, no birth data.
 *
 * @param engineOutputs  Typed outputs from Selemene engines (HD, Gene Keys, Vimshottari).
 * @returns SelectedCorpusIds with deduped ID list and per-ID rationale for audit.
 */
export function selectCorpusIds(engineOutputs: EngineOutputsForGrounding): SelectedCorpusIds {
  const rationale: Record<string, string> = {};
  const seen = new Set<string>();
  const ids: string[] = [];

  const addIds = (newIds: string[]): void => {
    for (const id of newIds) {
      if (!seen.has(id)) {
        seen.add(id);
        ids.push(id);
      }
    }
  };

  const hd = engineOutputs['human-design'];
  if (hd && typeof hd === 'object') {
    addIds(selectHumanDesignIds(hd as HumanDesignEngineOutput, rationale));
  }

  const gk = engineOutputs['gene-keys'];
  if (gk && typeof gk === 'object') {
    addIds(selectGeneKeysIds(gk as GeneKeysEngineOutput, rationale));
  }

  const vim = engineOutputs['vimshottari'];
  if (vim && typeof vim === 'object') {
    addIds(selectVimshottariIds(vim as VimshottariEngineOutput, rationale));
  }

  return { ids, rationale };
}
