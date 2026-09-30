// ─── Primary Passage Source (fresh-generation combined evidence) ────
//
// Doctrine (do NOT weaken elsewhere):
//   * Fresh generation may append separately-validated primary passages
//     to the live Cloudflare Vectorize retrieval. This module is the
//     canonical helper that turns primary registry additions into the
//     RetrievedPassage shape used by reference-execution and stamps a
//     stable, provenance-truthful `source` label so downstream code
//     cannot mistake a reviewed-primary-document passage for CF.
//   * Live CF retrieval is still mandatory — this module never
//     substitutes for it. It only appends after CF has already been
//     retrieved and cross-checked.
//   * The combined ordering is `[...cfPassages, ...primaryPassages]`
//     with primaries emitted in registry `additionIds` order. Callers
//     rely on this exact order to build a stable `passages_hash`.
//   * Primary passages MUST NOT use the `sw:` id prefix (reserved for
//     CF) and MUST NOT use the `ed:` id prefix (private editorial).
//     `validatePrimaryPassageAdditions` already enforces this at load
//     time via `assertValidAdditionShape`; we re-check the source label
//     here for defence in depth.
//
// This module is pure. No I/O, no LLM, no network.
import { createHash } from 'node:crypto';
import type { RetrievedPassage } from './reference-execution.js';
import type { EditorialPassageAddition } from './editorial-passage-evidence.js';

const PRIMARY_SOURCE_PREFIX = 'reviewed-primary-document:';

export interface CombinedPassagesInput {
  /** Live CF passages, in the order returned by the adapter. */
  cfPassages: readonly RetrievedPassage[];
  /** Validated primary additions, in registry `additionIds` order. */
  primaryAdditions: readonly EditorialPassageAddition[];
}

export interface CombinedPassagesResult {
  /** Frozen `[...cf, ...primary]` view for retriever consumption. */
  passages: readonly RetrievedPassage[];
  /** SHA-256 of the joined combined texts (`.map(p => p.text).join('\n')`). */
  passagesHash: string;
  /** Ordered CF-only ids (for the retrieval receipt's `cf_source_ids` field). */
  cfSourceIds: readonly string[];
  /** Ordered primary-only ids (registry `additionIds` subset actually appended). */
  primarySourceIds: readonly string[];
  /** Per-passage sha256(text) hex, same order as `passages`. */
  perPassageSha256: readonly string[];
}

function sha256(text: string): string {
  return createHash('sha256').update(text, 'utf8').digest('hex');
}

/**
 * Build the combined evidence view. Every primary addition becomes a
 * RetrievedPassage with:
 *   * `id` = the primary registry id (already validated as non-`sw:`,
 *     non-`ed:`),
 *   * `source` = `reviewed-primary-document:<https URL>` — the exact
 *     label pattern reference-execution already uses when it renders a
 *     primary-document evidence block.
 *   * `text` = the registry text (immutable).
 *
 * The returned view is frozen so the retriever cannot silently mutate
 * provenance downstream.
 */
export function buildCombinedPassages(
  input: CombinedPassagesInput,
): CombinedPassagesResult {
  const cfIds: string[] = [];
  const primaryIds: string[] = [];
  const passages: RetrievedPassage[] = [];
  for (const p of input.cfPassages) {
    if (typeof p?.id !== 'string' || !p.id.startsWith('sw:')) {
      throw new Error('cfPassages must use the sw: id prefix (live CF only)');
    }
    cfIds.push(p.id);
    passages.push(Object.freeze({ id: p.id, source: p.source, text: p.text }));
  }
  const cfSet = new Set(cfIds);
  for (const add of input.primaryAdditions) {
    if (add.provenance.kind !== 'reviewed-primary-document') {
      throw new Error('primaryAdditions must carry reviewed-primary-document provenance');
    }
    if (add.id.startsWith('sw:') || add.id.startsWith('ed:')) {
      throw new Error(`primary addition "${add.id}" uses a reserved id prefix`);
    }
    if (cfSet.has(add.id)) {
      throw new Error(`primary addition "${add.id}" collides with a live CF id`);
    }
    primaryIds.push(add.id);
    const source = `${PRIMARY_SOURCE_PREFIX}${add.provenance.url}`;
    passages.push(Object.freeze({ id: add.id, source, text: add.text }));
  }
  const passagesHash = passages.length > 0
    ? sha256(passages.map((p) => p.text).join('\n'))
    : '';
  const perPassageSha256 = passages.map((p) => sha256(p.text));
  return Object.freeze({
    passages: Object.freeze(passages),
    passagesHash,
    cfSourceIds: Object.freeze(cfIds),
    primarySourceIds: Object.freeze(primaryIds),
    perPassageSha256: Object.freeze(perPassageSha256),
  });
}

/**
 * Return true when `sourceId` is a primary registry id — i.e. does NOT
 * start with the reserved CF `sw:` prefix. Callers use this to skip
 * CF-only checks (snapshot binding) for primary ids.
 */
export function isPrimarySourceId(sourceId: string): boolean {
  return !sourceId.startsWith('sw:') && !sourceId.startsWith('ed:');
}
