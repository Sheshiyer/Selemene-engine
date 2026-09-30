// ─── Editorial Passage Evidence Validator ────────────────────────────
// Pure, leaf primitive used by a fresh editorial audit that wants to
// supplement — never replace — the original CF retrieval evidence with
// additional passages that were either
//
//   (a) freshly read back from the SAME Cloudflare Vectorize index
//       (kind: 'cf-vectorize' — proves the ID actually lives in the
//       corpus the base retrieval used), or
//
//   (b) short paraphrases quoted from an independently reviewed primary
//       document (kind: 'reviewed-primary-document' — a human or
//       assistant-source-review reviewer looked at an URL/locator).
//
// The primitive DOES NOT perform any I/O, network, LLM, Vectorize, or
// filesystem read. It is a hash + shape verifier. The caller is
// responsible for actually performing the CF readback / document review
// and handing the recovered trusted records to this function alongside
// the additions.
//
// Doctrine (must not weaken elsewhere):
//   * The ORIGINAL base retrieval receipts are immutable. This module
//     freezes the array it returns and re-verifies the caller-supplied
//     `basePassagesHash` against `sha256(base.map(p=>p.text).join('\n'))`
//     — the same join reference-source-audit uses.
//   * Original base source texts must ALSO round-trip against the
//     independently supplied trusted-record registry (or an explicit
//     `trustedOriginalTexts` map). If any base ID is missing from the
//     registry, or the trusted text hashes differently, the fresh
//     evidence is rejected as retrieval-drift.
//   * Additions must have unique non-empty ids, non-empty text, a valid
//     sha256 hex of that text, and a provenance discriminator that
//     mechanically forbids forgery:
//       - 'cf-vectorize' additions MUST use an id starting with 'sw:'
//         and MUST declare the exact account+index the caller says the
//         readback came from. That account+index MUST match the trusted
//         registry entry for the same id, byte-for-byte.
//       - 'reviewed-primary-document' additions MUST NOT use the 'sw:'
//         prefix (that surface is reserved for CF passages) and MUST
//         NOT use the 'ed:' prefix (reserved for private editorial
//         scratch — never trusted source evidence). They MUST carry a
//         https URL, non-empty locator, author, title, and a reviewer
//         of exactly 'assistant-source-review' or 'human'.
//     A trusted-record must exist for every addition, and the addition
//     record MUST fully equal (deep equal) the trusted record — the
//     caller cannot substitute their own text/provenance for an id the
//     registry knows about.
//   * Duplicate addition ids, addition ids that collide with base ids,
//     conflicting trusted-record entries, blank text, malformed arrays
//     — all fail closed.
//   * The returned `effectivePassages` is `[...base, ...additions]`
//     mapped to a `{id, source, text}` shape so downstream code that
//     already consumes RetrievedPassage[] doesn't need to know about
//     provenance. The returned array (and its inner objects) are
//     Object.frozen(). `effectivePassagesHash` uses the SAME newline
//     join reference-source-audit relies on so a fresh audit can bind
//     input_sha256 against the combined evidence.
//   * The module NEVER claims Vectorize ingestion, NEVER marks
//     acceptance, NEVER mutates any input. Callers only receive the
//     frozen combined view + a separately labeled `addedSourceRecords`
//     block preserving the exact provenance the reviewer supplied.
//
// Parent integration (editorial-verification / reference-verification)
// is a separate worker's responsibility. This module is a leaf primitive
// pending that wiring — nothing here declares itself route-connected.

import { createHash } from 'node:crypto';
import { isDeepStrictEqual } from 'node:util';
import type { RetrievedPassage } from './reference-execution.js';

// ─── Public types ─────────────────────────────────────────────────────

/**
 * Provenance discriminator carried on every fresh addition.
 *
 * The two variants are mutually exclusive — a passage is either an
 * actual CF readback from the same Vectorize index the base retrieval
 * used, OR a paraphrase drawn from a reviewed primary document with a
 * public URL. Anything else is a forgery.
 */
export type EditorialPassageProvenance =
  | {
      kind: 'cf-vectorize';
      /** Cloudflare account ID the readback was performed against. */
      account: string;
      /** Vectorize index name the readback was performed against. */
      index: string;
    }
  | {
      kind: 'reviewed-primary-document';
      author: string;
      title: string;
      /** Full document URL. Must be https. */
      url: string;
      /** Human-readable locator (e.g. chapter/page). */
      locator: string;
      /** Reviewer identity — either the assistant source-review pass or a human. */
      reviewer: 'assistant-source-review' | 'human';
    };

/** A fresh addition offered on top of the immutable base retrieval. */
export interface EditorialPassageAddition {
  /** Stable unique ID. Must not collide with any base id or other addition id. */
  id: string;
  /** Passage text. Non-empty. */
  text: string;
  /** sha256(text) hex — recomputed here; the caller-declared value must match. */
  sha256: string;
  provenance: EditorialPassageProvenance;
}

/**
 * Trusted source record independently supplied by the caller from a real
 * CF readback OR the reviewed-primary-document registry. The validator
 * requires strict deep equality between the addition and its trusted
 * record — the caller cannot fabricate a record.
 *
 * The shape is intentionally identical to `EditorialPassageAddition`.
 */
export type TrustedSourceRecord = EditorialPassageAddition;

/** Immutable per-base-id trusted original text supplied by the caller. */
export type TrustedOriginalTextMap = Readonly<Record<string, string>>;

export interface EditorialPassageEvidenceInput {
  /** Required when adding CF passages; independently selected route target. */
  expectedCF?: { account: string; index: string };
  /** Original CF retrieval passages, verbatim from the base receipt. */
  basePassages: readonly RetrievedPassage[];
  /** sha256 of `basePassages.map(p=>p.text).join('\n')`, hex. */
  basePassagesHash: string;
  /** Fresh additions the reviewer wants to append. May be empty. */
  additions: readonly EditorialPassageAddition[];
  /**
   * Independently supplied trusted records keyed by addition id.
   * Every addition id must appear here. Records not referenced by any
   * addition are permitted (registry may be broader than this audit).
   */
  trustedSourceRecords: Readonly<Record<string, TrustedSourceRecord>>;
  /**
   * Independently supplied trusted original texts keyed by base id.
   * Every base id must appear here. Enables base-drift detection
   * without the caller re-passing the original preflight object.
   */
  trustedOriginalTexts: TrustedOriginalTextMap;
}

/** The frozen combined view the caller may feed to a fresh audit. */
export interface EditorialPassageEvidenceResult {
  /**
   * Frozen `[...base, ...additions]` mapped to RetrievedPassage. Every
   * inner object is Object.frozen; the outer array is Object.frozen.
   * Callers cannot mutate; nothing here implies ingestion or acceptance.
   */
  effectivePassages: readonly RetrievedPassage[];
  /** sha256 of `effectivePassages.map(p=>p.text).join('\n')`, hex. */
  effectivePassagesHash: string;
  /** Original base hash, echoed back for provenance receipts. */
  originalBasePassagesHash: string;
  /**
   * Separately labeled additions with full provenance preserved.
   * Distinct field so downstream cannot confuse fresh additions with
   * the original CF corpus. Frozen.
   */
  addedSourceRecords: readonly EditorialPassageAddition[];
}

export class EditorialPassageEvidenceError extends Error {
  constructor(message: string) {
    super(message);
    this.name = 'EditorialPassageEvidenceError';
  }
}

// ─── Internals ────────────────────────────────────────────────────────

const SHA256_HEX = /^[a-f0-9]{64}$/;
const CF_ID_PREFIX = 'sw:';
const PRIVATE_EDITORIAL_ID_PREFIX = 'ed:';

function sha256(text: string): string {
  return createHash('sha256').update(text, 'utf8').digest('hex');
}

function isPlainObject(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null && !Array.isArray(value);
}

function isNonEmptyString(value: unknown): value is string {
  return typeof value === 'string' && value.trim().length > 0;
}

function joinPassageTexts(passages: readonly { text: string }[]): string {
  return passages.map((p) => p.text).join('\n');
}

function assertValidBasePassage(index: number, passage: RetrievedPassage): void {
  if (!isPlainObject(passage)) {
    throw new EditorialPassageEvidenceError(
      `basePassages[${index}] is not an object`,
    );
  }
  if (!isNonEmptyString(passage.id)) {
    throw new EditorialPassageEvidenceError(
      `basePassages[${index}].id must be a non-empty string`,
    );
  }
  if (!passage.id.startsWith(CF_ID_PREFIX)) {
    throw new EditorialPassageEvidenceError('base passages must use public sw: CF IDs');
  }
  if (!isNonEmptyString(passage.source)) {
    throw new EditorialPassageEvidenceError(
      `basePassages[${index}].source must be a non-empty string`,
    );
  }
  if (!isNonEmptyString(passage.text)) {
    throw new EditorialPassageEvidenceError(
      `basePassages[${index}].text must be a non-empty string`,
    );
  }
}

function assertValidProvenance(
  additionId: string,
  provenance: EditorialPassageProvenance,
): void {
  if (!isPlainObject(provenance)) {
    throw new EditorialPassageEvidenceError(
      `additions[id=${additionId}].provenance must be an object`,
    );
  }
  if (provenance.kind === 'cf-vectorize') {
    if (!isNonEmptyString(provenance.account)) {
      throw new EditorialPassageEvidenceError(
        `additions[id=${additionId}] cf-vectorize provenance requires non-empty account`,
      );
    }
    if (!isNonEmptyString(provenance.index)) {
      throw new EditorialPassageEvidenceError(
        `additions[id=${additionId}] cf-vectorize provenance requires non-empty index`,
      );
    }
    if (!additionId.startsWith(CF_ID_PREFIX)) {
      throw new EditorialPassageEvidenceError(
        `additions[id=${additionId}] cf-vectorize passages must use the "sw:" id prefix`,
      );
    }
    return;
  }
  if (provenance.kind === 'reviewed-primary-document') {
    if (additionId.startsWith(CF_ID_PREFIX)) {
      throw new EditorialPassageEvidenceError(
        `additions[id=${additionId}] reviewed-primary-document must not use the "sw:" id prefix (reserved for CF passages)`,
      );
    }
    if (additionId.startsWith(PRIVATE_EDITORIAL_ID_PREFIX)) {
      throw new EditorialPassageEvidenceError(
        `additions[id=${additionId}] reviewed-primary-document must not use the private "ed:" id prefix`,
      );
    }
    if (!isNonEmptyString(provenance.author)) {
      throw new EditorialPassageEvidenceError(
        `additions[id=${additionId}] reviewed-primary-document requires non-empty author`,
      );
    }
    if (!isNonEmptyString(provenance.title)) {
      throw new EditorialPassageEvidenceError(
        `additions[id=${additionId}] reviewed-primary-document requires non-empty title`,
      );
    }
    if (!isNonEmptyString(provenance.url) || !provenance.url.startsWith('https://')) {
      throw new EditorialPassageEvidenceError(
        `additions[id=${additionId}] reviewed-primary-document requires https URL`,
      );
    }
    try {
      const url = new URL(provenance.url);
      if (url.protocol !== 'https:' || !url.hostname || url.username || url.password) throw new Error('invalid');
    } catch {
      throw new EditorialPassageEvidenceError(`additions[id=${additionId}] requires a valid public https URL`);
    }
    if (!isNonEmptyString(provenance.locator)) {
      throw new EditorialPassageEvidenceError(
        `additions[id=${additionId}] reviewed-primary-document requires non-empty locator`,
      );
    }
    if (
      provenance.reviewer !== 'assistant-source-review' &&
      provenance.reviewer !== 'human'
    ) {
      throw new EditorialPassageEvidenceError(
        `additions[id=${additionId}] reviewed-primary-document reviewer must be "assistant-source-review" or "human"`,
      );
    }
    return;
  }
  throw new EditorialPassageEvidenceError(
    `additions[id=${additionId}] provenance.kind is unknown or missing`,
  );
}

function assertValidAdditionShape(
  index: number,
  addition: EditorialPassageAddition,
): void {
  if (!isPlainObject(addition)) {
    throw new EditorialPassageEvidenceError(
      `additions[${index}] is not an object`,
    );
  }
  if (!isNonEmptyString(addition.id)) {
    throw new EditorialPassageEvidenceError(
      `additions[${index}].id must be a non-empty string`,
    );
  }
  if (addition.id.startsWith(PRIVATE_EDITORIAL_ID_PREFIX)) {
    throw new EditorialPassageEvidenceError(
      `additions[${index}].id "${addition.id}" uses the reserved private "ed:" prefix`,
    );
  }
  if (!isNonEmptyString(addition.text)) {
    throw new EditorialPassageEvidenceError(
      `additions[${index}].text must be a non-empty string`,
    );
  }
  if (!SHA256_HEX.test(addition.sha256)) {
    throw new EditorialPassageEvidenceError(
      `additions[${index}].sha256 is not a valid lowercase hex sha256`,
    );
  }
  if (sha256(addition.text) !== addition.sha256) {
    throw new EditorialPassageEvidenceError(
      `additions[${index}] (id=${addition.id}) sha256 does not match sha256(text)`,
    );
  }
  assertValidProvenance(addition.id, addition.provenance);
}

/**
 * Structural deep-equality for a trusted record vs an addition. Rejects
 * any extra keys on either side so a caller cannot bury forged fields.
 */
function recordsAreEqual(
  addition: EditorialPassageAddition,
  trusted: TrustedSourceRecord,
): boolean {
  const keys = ['id', 'text', 'sha256', 'provenance'];
  if (Object.keys(addition).some(k => !keys.includes(k)) || Object.keys(trusted).some(k => !keys.includes(k))) return false;
  if (!isDeepStrictEqual(addition, trusted)) return false;
  if (addition.id !== trusted.id) return false;
  if (addition.text !== trusted.text) return false;
  if (addition.sha256 !== trusted.sha256) return false;
  const ap = addition.provenance;
  const tp = trusted.provenance;
  if (ap.kind !== tp.kind) return false;
  if (ap.kind === 'cf-vectorize' && tp.kind === 'cf-vectorize') {
    if (Object.keys(ap).length !== 3 || Object.keys(tp).length !== 3) return false;
    return ap.account === tp.account && ap.index === tp.index;
  }
  if (
    ap.kind === 'reviewed-primary-document' &&
    tp.kind === 'reviewed-primary-document'
  ) {
    if (Object.keys(ap).length !== 6 || Object.keys(tp).length !== 6) return false;
    return (
      ap.author === tp.author &&
      ap.title === tp.title &&
      ap.url === tp.url &&
      ap.locator === tp.locator &&
      ap.reviewer === tp.reviewer
    );
  }
  return false;
}

// ─── Public entry point ───────────────────────────────────────────────

/**
 * Validate fresh editorial passage evidence and return the frozen
 * effective combined view. Pure — no I/O, no LLM, no network.
 *
 * Throws EditorialPassageEvidenceError on any structural, hash, or
 * provenance violation. On success the returned object is safe to feed
 * into a fresh audit input as `passages` + `passagesHash`.
 */
export function validateEditorialPassageEvidence(
  input: EditorialPassageEvidenceInput,
): EditorialPassageEvidenceResult {
  if (!isPlainObject(input)) {
    throw new EditorialPassageEvidenceError('input must be an object');
  }
  if (!Array.isArray(input.basePassages)) {
    throw new EditorialPassageEvidenceError('basePassages must be an array');
  }
  if (input.basePassages.length === 0) {
    throw new EditorialPassageEvidenceError(
      'basePassages must be non-empty — original retrieval cannot be discarded',
    );
  }
  if (!Array.isArray(input.additions)) {
    throw new EditorialPassageEvidenceError('additions must be an array');
  }
  if (!SHA256_HEX.test(input.basePassagesHash)) {
    throw new EditorialPassageEvidenceError(
      'basePassagesHash is not a valid lowercase hex sha256',
    );
  }
  if (!isPlainObject(input.trustedSourceRecords)) {
    throw new EditorialPassageEvidenceError(
      'trustedSourceRecords must be an object keyed by addition id',
    );
  }
  if (!isPlainObject(input.trustedOriginalTexts)) {
    throw new EditorialPassageEvidenceError(
      'trustedOriginalTexts must be an object keyed by base id',
    );
  }

  // 1. Shape-check every base passage.
  const baseIds = new Set<string>();
  for (let i = 0; i < input.basePassages.length; i += 1) {
    const p = input.basePassages[i];
    assertValidBasePassage(i, p);
    if (baseIds.has(p.id)) {
      throw new EditorialPassageEvidenceError(
        `basePassages contains duplicate id "${p.id}"`,
      );
    }
    baseIds.add(p.id);
  }

  // 2. Recompute base hash exactly the way reference-source-audit does.
  const recomputedBaseHash = sha256(joinPassageTexts(input.basePassages));
  if (recomputedBaseHash !== input.basePassagesHash) {
    throw new EditorialPassageEvidenceError(
      'basePassagesHash does not match sha256(basePassages join) — original corpus altered',
    );
  }

  // 3. Cross-check every base text against the trusted original registry.
  for (const p of input.basePassages) {
    const trustedText = input.trustedOriginalTexts[p.id];
    if (typeof trustedText !== 'string' || trustedText.length === 0) {
      throw new EditorialPassageEvidenceError(
        `trustedOriginalTexts missing entry for base id "${p.id}"`,
      );
    }
    if (trustedText !== p.text) {
      throw new EditorialPassageEvidenceError(
        `base id "${p.id}" text differs from trusted original registry — retrieval drift`,
      );
    }
  }

  // 4. Shape-check every addition and confirm no id collisions.
  const additionIds = new Set<string>();
  for (let i = 0; i < input.additions.length; i += 1) {
    const add = input.additions[i];
    assertValidAdditionShape(i, add);
    if (add.provenance.kind === 'cf-vectorize' &&
        (!input.expectedCF || add.provenance.account !== input.expectedCF.account ||
         add.provenance.index !== input.expectedCF.index)) {
      throw new EditorialPassageEvidenceError('CF addition does not match independently selected account/index');
    }
    if (baseIds.has(add.id)) {
      throw new EditorialPassageEvidenceError(
        `additions[${i}].id "${add.id}" collides with an existing base passage id`,
      );
    }
    if (additionIds.has(add.id)) {
      throw new EditorialPassageEvidenceError(
        `additions contains duplicate id "${add.id}"`,
      );
    }
    additionIds.add(add.id);
  }

  // 5. Every addition must have a trusted-record match. Deep equality.
  for (const add of input.additions) {
    const trusted = input.trustedSourceRecords[add.id];
    if (trusted === undefined) {
      throw new EditorialPassageEvidenceError(
        `additions[id=${add.id}] has no matching trustedSourceRecords entry`,
      );
    }
    // Trusted record must itself be structurally valid so the caller
    // can't ship a garbage registry entry to whitelist a bad addition.
    if (!isPlainObject(trusted)) {
      throw new EditorialPassageEvidenceError(
        `trustedSourceRecords["${add.id}"] must be an object`,
      );
    }
    if (trusted.id !== add.id) {
      throw new EditorialPassageEvidenceError(
        `trustedSourceRecords["${add.id}"] carries mismatched id "${trusted.id}"`,
      );
    }
    if (!SHA256_HEX.test(trusted.sha256) || sha256(trusted.text) !== trusted.sha256) {
      throw new EditorialPassageEvidenceError(
        `trustedSourceRecords["${add.id}"] sha256 does not match sha256(text)`,
      );
    }
    // Re-run provenance validation on the trusted record so its shape
    // is held to the same doctrine (forbid ed:/sw: crossovers, https, etc.).
    assertValidProvenance(trusted.id, trusted.provenance);
    if (!recordsAreEqual(add, trusted)) {
      throw new EditorialPassageEvidenceError(
        `additions[id=${add.id}] does not deep-equal its trustedSourceRecords entry — forged provenance`,
      );
    }
    // 5b. For cf-vectorize, the account+index the caller declared must
    // match the trusted registry entry byte-for-byte (already covered
    // by recordsAreEqual, but re-check defensively).
    if (
      add.provenance.kind === 'cf-vectorize' &&
      trusted.provenance.kind === 'cf-vectorize' &&
      (add.provenance.account !== trusted.provenance.account ||
        add.provenance.index !== trusted.provenance.index)
    ) {
      throw new EditorialPassageEvidenceError(
        `additions[id=${add.id}] cf-vectorize account/index does not match trusted registry`,
      );
    }
  }

  // 6. Build frozen effective view + added-records ledger.
  const effectivePassages: RetrievedPassage[] = [];
  for (const p of input.basePassages) {
    effectivePassages.push(
      Object.freeze({ id: p.id, source: p.source, text: p.text }),
    );
  }
  for (const add of input.additions) {
    const source =
      add.provenance.kind === 'cf-vectorize'
        ? `cf-vectorize:${add.provenance.account}/${add.provenance.index}`
        : `reviewed-primary-document:${add.provenance.url}`;
    effectivePassages.push(
      Object.freeze({ id: add.id, source, text: add.text }),
    );
  }
  const effectivePassagesHash = sha256(joinPassageTexts(effectivePassages));

  // Freeze the added-source-records ledger separately with full provenance.
  const addedSourceRecords = input.additions.map((add) => {
    const provenance =
      add.provenance.kind === 'cf-vectorize'
        ? Object.freeze({
            kind: 'cf-vectorize' as const,
            account: add.provenance.account,
            index: add.provenance.index,
          })
        : Object.freeze({
            kind: 'reviewed-primary-document' as const,
            author: add.provenance.author,
            title: add.provenance.title,
            url: add.provenance.url,
            locator: add.provenance.locator,
            reviewer: add.provenance.reviewer,
          });
    return Object.freeze({
      id: add.id,
      text: add.text,
      sha256: add.sha256,
      provenance,
    }) as EditorialPassageAddition;
  });

  return Object.freeze({
    effectivePassages: Object.freeze(effectivePassages),
    effectivePassagesHash,
    originalBasePassagesHash: input.basePassagesHash,
    addedSourceRecords: Object.freeze(addedSourceRecords),
  });
}

// ═══════════════════════════════════════════════════════════════════════
// ─── Primary passage registry validator (fresh-generation surface) ───
// ═══════════════════════════════════════════════════════════════════════
//
// Doctrine (must not weaken elsewhere):
//   * This validator is used by the fresh-generation CLI to accept an
//     independent "primary passage registry" — the same registry shape
//     the editorial-prefix path already consumes — and return a frozen
//     view of the reviewed-primary-document additions selected via the
//     registry's `additionIds`.
//   * We deliberately re-use `assertValidAdditionShape` /
//     `assertValidProvenance` so the schema, hash, and provenance rules
//     match the editorial supplement path byte-for-byte. This module
//     never invents a weaker schema and never accepts a cf-vectorize
//     entry (fresh CF passages must come from a live adapter, never
//     from a self-asserted registry file).
//   * No I/O; the caller reads registry bytes and hands us the parsed
//     object. The caller is responsible for snapshotting bytes + hash.
//   * The returned array (and every inner record) is Object.frozen so
//     downstream code cannot mutate provenance.
//
// Failures throw `EditorialPassageEvidenceError` — same error class
// operators already know from the editorial-prefix flow.

/** Parsed registry shape produced by the CLI writers. */
export interface PrimaryPassageRegistryFile {
  /** All records in the registry, keyed by id. Both CF and primary allowed here. */
  records: Readonly<Record<string, TrustedSourceRecord>>;
  /** Ordered list of ids that fresh generation may select as primary additions. */
  additionIds: readonly string[];
  /** Free-form metadata (scope, sourceEvidence, ...) preserved unmodified. */
  [key: string]: unknown;
}

export interface ValidatePrimaryPassageAdditionsInput {
  /** The parsed registry file. Must contain a `records` object and `additionIds` array. */
  registry: PrimaryPassageRegistryFile;
  /**
   * Optional deny-list of ids reserved for the live CF retrieval. When
   * supplied, a primary addition that collides with any live CF id is
   * rejected as retrieval-drift or spoofed provenance.
   */
  reservedCFIds?: readonly string[];
}

export interface ValidatePrimaryPassageAdditionsResult {
  /** The selected primary additions, in `additionIds` order. Frozen. */
  additions: readonly EditorialPassageAddition[];
  /** Same records mapped by id for callers that need registry lookup. Frozen. */
  additionsById: Readonly<Record<string, EditorialPassageAddition>>;
  /** The exact ordered id list processed. Frozen copy of `additionIds`. */
  additionIds: readonly string[];
}

/**
 * Validate a primary passage registry and return the frozen primary
 * additions selected via `additionIds`. Pure — no I/O, no LLM, no
 * network.
 *
 * Rules:
 *   * `registry.records` must be a plain object; `registry.additionIds`
 *     must be a non-empty string array (empty registries never fresh-
 *     generate primary evidence — call sites should skip the flag).
 *   * Every id in `additionIds` must exist in `registry.records`.
 *   * Every referenced record must have provenance kind
 *     'reviewed-primary-document' — fresh generation MUST NOT trust a
 *     self-asserted cf-vectorize registry entry (live CF is the source
 *     of truth for CF passages).
 *   * Every referenced record must pass `assertValidAdditionShape`
 *     (byte-exact reuse of the editorial supplement rules).
 *   * Duplicate ids in `additionIds` are rejected.
 *   * Ids that collide with any reserved CF id are rejected.
 */
export function validatePrimaryPassageAdditions(
  input: ValidatePrimaryPassageAdditionsInput,
): ValidatePrimaryPassageAdditionsResult {
  if (!isPlainObject(input)) {
    throw new EditorialPassageEvidenceError('input must be an object');
  }
  const { registry, reservedCFIds } = input;
  if (!isPlainObject(registry)) {
    throw new EditorialPassageEvidenceError('registry must be an object');
  }
  if (!isPlainObject(registry.records)) {
    throw new EditorialPassageEvidenceError('registry.records must be an object keyed by id');
  }
  if (!Array.isArray(registry.additionIds)) {
    throw new EditorialPassageEvidenceError('registry.additionIds must be an array');
  }
  if (registry.additionIds.length === 0) {
    throw new EditorialPassageEvidenceError(
      'registry.additionIds must be non-empty — nothing to fresh-generate against',
    );
  }
  const reserved = new Set(reservedCFIds ?? []);
  const seen = new Set<string>();
  const additions: EditorialPassageAddition[] = [];
  const additionsById: Record<string, EditorialPassageAddition> = {};
  for (let i = 0; i < registry.additionIds.length; i += 1) {
    const id = registry.additionIds[i];
    if (!isNonEmptyString(id)) {
      throw new EditorialPassageEvidenceError(
        `registry.additionIds[${i}] must be a non-empty string`,
      );
    }
    if (seen.has(id)) {
      throw new EditorialPassageEvidenceError(
        `registry.additionIds contains duplicate id "${id}"`,
      );
    }
    seen.add(id);
    if (reserved.has(id)) {
      throw new EditorialPassageEvidenceError(
        `registry primary addition "${id}" collides with a live CF passage id — spoofed provenance rejected`,
      );
    }
    const record = registry.records[id];
    if (record === undefined) {
      throw new EditorialPassageEvidenceError(
        `registry.additionIds[${i}] references unknown record "${id}"`,
      );
    }
    if (!isPlainObject(record)) {
      throw new EditorialPassageEvidenceError(
        `registry.records["${id}"] must be an object`,
      );
    }
    if (record.id !== id) {
      throw new EditorialPassageEvidenceError(
        `registry.records["${id}"] carries mismatched id "${record.id}"`,
      );
    }
    // Fresh generation must never trust a self-asserted CF record.
    if (record.provenance?.kind !== 'reviewed-primary-document') {
      throw new EditorialPassageEvidenceError(
        `registry addition "${id}" must have provenance.kind === 'reviewed-primary-document' for fresh generation; got "${record.provenance?.kind ?? '<missing>'}"`,
      );
    }
    // Full shape / hash / provenance / URL / reviewer check.
    assertValidAdditionShape(i, record);
    const frozen = Object.freeze({
      id: record.id,
      text: record.text,
      sha256: record.sha256,
      provenance: Object.freeze({
        kind: 'reviewed-primary-document' as const,
        author: record.provenance.author,
        title: record.provenance.title,
        url: record.provenance.url,
        locator: record.provenance.locator,
        reviewer: record.provenance.reviewer,
      }),
    }) as EditorialPassageAddition;
    additions.push(frozen);
    additionsById[id] = frozen;
  }
  return Object.freeze({
    additions: Object.freeze(additions),
    additionsById: Object.freeze(additionsById),
    additionIds: Object.freeze([...registry.additionIds]),
  });
}
