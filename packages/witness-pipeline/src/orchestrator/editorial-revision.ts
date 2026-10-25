// ─── Editorial Revision Primitive ─────────────────────────────────────
// Pure, deterministic construction of an immutable EditorialRevision artifact
// from a base SectionExecutionReceipt, its current output, and an explicit
// ordered list of exact-text patches supplied by a reviewer.
//
// Doctrine:
//   * The base receipt is NEVER mutated and NEVER re-run. The base output hash
//     is required and re-verified; any mismatch is a hard reject.
//   * Patches are exact-text edits with before/after/rationale/sources. Any
//     ambiguity, missing occurrence, duplicate `before`, or overlap is a hard
//     reject. No fuzzy matching. No model judgment. No LLM calls here.
//   * The returned artifact is a NEW, deep-frozen object. Pending review flags
//     ('sourceAudit' | 'jev' | 'document' | 'coherence') are ALL set to
//     'pending'. Nothing here is allowed to mark a downstream gate accepted,
//     relabel Jev, or fabricate reruns of the source receipt.
//
// This module is intentionally free of I/O, clock reads, and network calls.
// The caller supplies the timestamp so the primitive stays pure and testable.

import { createHash } from 'node:crypto';
import type { SectionExecutionReceipt } from './reference-execution.js';

// ─── Types ────────────────────────────────────────────────────────────

/** A single source reference cited to justify one editorial patch. */
export interface EditorialSourceRef {
  /** Stable source identifier (corpus ID, doc slug, receipt hash, etc.). Nonempty. */
  sourceId: string;
  /** Exact quotation or locator inside the source that grounds the correction. Nonempty. */
  quote: string;
  /** Optional human label (file path, section name). */
  label?: string;
}

/**
 * One exact-text patch. `before` must occur exactly once in the current
 * section output at the time of application; `after` replaces it. No regex,
 * no fuzz. `rationale` and `sourceReferences` are mandatory audit evidence.
 */
export interface EditorialPatch {
  before: string;
  after: string;
  rationale: string;
  sourceReferences: readonly EditorialSourceRef[];
}

/** Who authored this revision. */
export type EditorialReviewer = 'assistant-source-review' | 'human';

/**
 * Downstream gates whose acceptance MUST be earned separately after this
 * revision lands. This primitive can only mark them 'pending'.
 */
export interface PendingReviewFlags {
  sourceAudit: 'pending';
  jev: 'pending';
  document: 'pending';
  coherence: 'pending';
}

/**
 * Immutable revision artifact. Preserves the link to the base receipt via
 * `baseOutputHash` and `baseSectionId` — never carries the full base receipt.
 */
export interface EditorialRevision {
  /** Section ID the base receipt covered; copied verbatim after validation. */
  sectionId: string;
  /** SHA-256 of the base receipt's accepted output (matches baseReceipt.output_hash). */
  baseOutputHash: string;
  /** The revised section output after all patches are applied. */
  revisedOutput: string;
  /** SHA-256 of `revisedOutput`. Guaranteed different from `baseOutputHash`. */
  revisedOutputHash: string;
  /** Patch list in the deterministic order they were applied. */
  patches: readonly EditorialPatch[];
  /** Who authored the revision. */
  reviewer: EditorialReviewer;
  /** Caller-supplied timestamp (ISO-8601 recommended). Nonempty. */
  timestamp: string;
  /** All downstream gates start pending; this primitive never marks them met. */
  pendingReviews: PendingReviewFlags;
}

/** Input to `applyEditorialRevision`. */
export interface ApplyEditorialRevisionInput {
  baseReceipt: SectionExecutionReceipt;
  currentOutput: string;
  expectedBaseOutputHash: string;
  patches: readonly EditorialPatch[];
  reviewer: EditorialReviewer;
  timestamp: string;
}

// ─── Error type ───────────────────────────────────────────────────────

export type EditorialRevisionErrorCode =
  | 'invalid_section_id'
  | 'base_hash_mismatch'
  | 'empty_patches'
  | 'invalid_patch'
  | 'empty_rationale'
  | 'empty_sources'
  | 'invalid_source_ref'
  | 'duplicate_before'
  | 'missing_occurrence'
  | 'ambiguous_occurrence'
  | 'overlapping_patches'
  | 'noop_patch'
  | 'invalid_reviewer'
  | 'invalid_timestamp'
  | 'unchanged_output';

export class EditorialRevisionError extends Error {
  readonly code: EditorialRevisionErrorCode;
  readonly detail?: string;
  constructor(code: EditorialRevisionErrorCode, message: string, detail?: string) {
    super(message);
    this.name = 'EditorialRevisionError';
    this.code = code;
    this.detail = detail;
  }
}

// ─── Internal helpers ─────────────────────────────────────────────────

function sha256Hex(input: string): string {
  return createHash('sha256').update(input, 'utf8').digest('hex');
}

function isNonemptyTrimmed(value: unknown): value is string {
  return typeof value === 'string' && value.trim().length > 0;
}

function reject(code: EditorialRevisionErrorCode, message: string, detail?: string): never {
  throw new EditorialRevisionError(code, message, detail);
}

function countOccurrences(haystack: string, needle: string): number {
  if (needle.length === 0) return 0;
  let count = 0;
  let idx = haystack.indexOf(needle);
  while (idx !== -1) {
    count += 1;
    idx = haystack.indexOf(needle, idx + 1);
  }
  return count;
}

// ─── Public API ───────────────────────────────────────────────────────

/**
 * Build an immutable EditorialRevision from a base receipt + current output +
 * exact-text patches. Pure. Deterministic. Never mutates inputs. Never marks
 * any downstream gate accepted.
 */
export function applyEditorialRevision(input: ApplyEditorialRevisionInput): EditorialRevision {
  const { baseReceipt, currentOutput, expectedBaseOutputHash, patches, reviewer, timestamp } = input;

  // ── reviewer ─────────────────────────────────────────────────────
  if (reviewer !== 'assistant-source-review' && reviewer !== 'human') {
    reject('invalid_reviewer', 'reviewer must be "assistant-source-review" or "human"');
  }

  // ── timestamp ────────────────────────────────────────────────────
  if (!isNonemptyTrimmed(timestamp)) {
    reject('invalid_timestamp', 'timestamp must be a nonempty string supplied by the caller');
  }

  // ── section ID ───────────────────────────────────────────────────
  if (!baseReceipt || typeof baseReceipt !== 'object') {
    reject('invalid_section_id', 'baseReceipt is required');
  }
  const sectionId = baseReceipt.section_id;
  if (!isNonemptyTrimmed(sectionId)) {
    reject('invalid_section_id', 'baseReceipt.section_id must be nonempty');
  }

  // ── base hash gate ───────────────────────────────────────────────
  if (!isNonemptyTrimmed(expectedBaseOutputHash)) {
    reject('base_hash_mismatch', 'expectedBaseOutputHash is required');
  }
  const actualCurrentHash = sha256Hex(currentOutput);
  if (actualCurrentHash !== expectedBaseOutputHash) {
    reject(
      'base_hash_mismatch',
      'currentOutput SHA-256 does not match expectedBaseOutputHash',
      `expected=${expectedBaseOutputHash} actual=${actualCurrentHash}`,
    );
  }
  if (
    !isNonemptyTrimmed(baseReceipt.output_hash) ||
    baseReceipt.output_hash !== expectedBaseOutputHash
  ) {
    reject(
      'base_hash_mismatch',
      'expectedBaseOutputHash does not match baseReceipt.output_hash',
      `receipt=${baseReceipt.output_hash} expected=${expectedBaseOutputHash}`,
    );
  }

  // ── patches: shape + duplicate-before ───────────────────────────
  if (!Array.isArray(patches) || patches.length === 0) {
    reject('empty_patches', 'at least one patch is required');
  }

  const seenBefore = new Set<string>();
  for (let i = 0; i < patches.length; i += 1) {
    const p = patches[i];
    if (!p || typeof p !== 'object') {
      reject('invalid_patch', `patch[${i}] must be an object`);
    }
    if (typeof p.before !== 'string' || p.before.length === 0) {
      reject('invalid_patch', `patch[${i}].before must be a nonempty string`);
    }
    if (typeof p.after !== 'string') {
      reject('invalid_patch', `patch[${i}].after must be a string`);
    }
    if (p.after === p.before) {
      reject('noop_patch', `patch[${i}] is a no-op (before === after)`);
    }
    if (!isNonemptyTrimmed(p.rationale)) {
      reject('empty_rationale', `patch[${i}].rationale must be a nonempty string`);
    }
    if (!Array.isArray(p.sourceReferences) || p.sourceReferences.length === 0) {
      reject('empty_sources', `patch[${i}].sourceReferences must be a nonempty array`);
    }
    for (let j = 0; j < p.sourceReferences.length; j += 1) {
      const ref = p.sourceReferences[j];
      if (!ref || typeof ref !== 'object') {
        reject('invalid_source_ref', `patch[${i}].sourceReferences[${j}] must be an object`);
      }
      if (!isNonemptyTrimmed(ref.sourceId)) {
        reject('invalid_source_ref', `patch[${i}].sourceReferences[${j}].sourceId must be nonempty`);
      }
      if (!isNonemptyTrimmed(ref.quote)) {
        reject('invalid_source_ref', `patch[${i}].sourceReferences[${j}].quote must be nonempty`);
      }
    }
    if (seenBefore.has(p.before)) {
      reject('duplicate_before', `patch[${i}].before duplicates an earlier patch`);
    }
    seenBefore.add(p.before);
  }

  // ── occurrence gate against the ORIGINAL currentOutput ──────────
  interface Located {
    patch: EditorialPatch;
    start: number;
    end: number;
  }
  const located: Located[] = patches.map((p, i) => {
    const count = countOccurrences(currentOutput, p.before);
    if (count === 0) {
      reject('missing_occurrence', `patch[${i}].before does not appear in currentOutput`);
    }
    if (count > 1) {
      reject(
        'ambiguous_occurrence',
        `patch[${i}].before appears ${count} times in currentOutput; must be unique`,
      );
    }
    const start = currentOutput.indexOf(p.before);
    return { patch: p, start, end: start + p.before.length };
  });

  // ── deterministic order: ascending by start index ───────────────
  located.sort((a, b) => a.start - b.start);

  // ── overlap gate ────────────────────────────────────────────────
  for (let i = 1; i < located.length; i += 1) {
    if (located[i].start < located[i - 1].end) {
      reject(
        'overlapping_patches',
        `patch replacing "${located[i].patch.before.slice(0, 32)}…" overlaps a prior patch range`,
      );
    }
  }

  // ── apply patches once, left-to-right, no mutation of inputs ────
  const orderedPatches: EditorialPatch[] = located.map((l) => l.patch);
  const pieces: string[] = [];
  let cursor = 0;
  for (const l of located) {
    pieces.push(currentOutput.slice(cursor, l.start));
    pieces.push(l.patch.after);
    cursor = l.end;
  }
  pieces.push(currentOutput.slice(cursor));
  const revisedOutput = pieces.join('');
  const revisedOutputHash = sha256Hex(revisedOutput);

  if (revisedOutputHash === expectedBaseOutputHash) {
    reject('unchanged_output', 'revised output hash equals base output hash — no net change');
  }

  // ── freeze source refs / patches for immutability ───────────────
  const frozenPatches: readonly EditorialPatch[] = Object.freeze(
    orderedPatches.map((p) =>
      Object.freeze({
        before: p.before,
        after: p.after,
        rationale: p.rationale,
        sourceReferences: Object.freeze(
          p.sourceReferences.map((ref) =>
            Object.freeze({
              sourceId: ref.sourceId,
              quote: ref.quote,
              ...(ref.label !== undefined ? { label: ref.label } : {}),
            } as EditorialSourceRef),
          ),
        ),
      } as EditorialPatch),
    ),
  );

  const pendingReviews: PendingReviewFlags = Object.freeze({
    sourceAudit: 'pending',
    jev: 'pending',
    document: 'pending',
    coherence: 'pending',
  });

  const revision: EditorialRevision = Object.freeze({
    sectionId,
    baseOutputHash: expectedBaseOutputHash,
    revisedOutput,
    revisedOutputHash,
    patches: frozenPatches,
    reviewer,
    timestamp,
    pendingReviews,
  });

  return revision;
}
