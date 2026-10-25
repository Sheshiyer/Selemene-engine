// ─── Editorial Verification Helper ────────────────────────────────────
// Provenance-aware verification primitives for revised sections consumed by
// verifyReferenceExecution.  This module is pure: no I/O, no network, no
// clocks, no LLM calls, no mutation of any input.  All hashes are recomputed
// from bytes supplied by the caller.
//
// Doctrine (must not weaken elsewhere):
//   * The original SectionExecutionReceipt is NEVER mutated.  Its stored raw
//     voices / synthesis / hashes remain the ORIGINAL model call receipt.
//   * A revised section's post-edit output is delivered via `pass.output`
//     and must equal applyEditorialRevision(originalReceipt.raw.synthesis,
//     patches) recomputed here — the stored revisedOutput/hash from the
//     manifest revision artifact is re-verified, never trusted.
//   * The fresh source-audit is a `fresh-audit-of-editorial-revision`
//     sidecar bound by hash to the revised output, saved receipt bytes,
//     saved passages_hash and saved engine_facts (plus a bounded,
//     text/hash/scope-verified supplemental-implementation-evidence
//     appendix mapped through implementationSourceTexts, as
//     editorial-prefix validates).
//   * The fresh Jev must be `judged`, not blocked, bound to the revised
//     output hash, and have no `fail` verdicts.  A `could-not-tell` on
//     the guardrail dimension only — never any other dimension — may be
//     resolved by a full ValidateGuardrailDispositionInput that passes
//     validateGuardrailDisposition.  The stored verdict is NEVER changed.
//   * The original receipt outcome may be `failed` only when its sole
//     failing attempt outcome kind is `source_audit_failed` and the fresh
//     audit is clean.  Any other failure kind blocks.
//   * Supplemental engine facts must be exactly `base.engine_facts` plus
//     the bounded implementation-evidence appendix (text/hash/scope
//     matching editorial-prefix serialization) — nothing else.
//   * Original model-generation prior-context edges must reference the
//     ORIGINAL historical prior outputs (validated against the caller's
//     originalPasses map).  A revised section's raw synthesis input must
//     NOT contain any other section's revised output text (guards against
//     forged reruns).  Newly generated sections that follow revised ones
//     must include the CURRENT effective prior text (the revised
//     versions) — the base verifier already checks that against
//     rawSynthesisInputs and passes[].output.
//   * A whole-document editorial coherence review is REQUIRED whenever
//     any revised sections are present.  Missing or stale coherence
//     review blocks even when every section-level check passes.
//   * When ReferenceVerificationInput.editorial is undefined the verifier
//     preserves its existing strict behaviour verbatim.
//
// v2 hardening (2026-09-30 review, docs/plans/2026-09-30-editorial-verifier-review.md):
//   * Item 1 — the ORIGINAL receipt file's raw bytes are required.  The
//     verifier recomputes their sha256 and cross-checks against both
//     `evidence.baseReceiptSha256` and `freshAudit.baseReceiptSha256`, and
//     re-parses them to confirm the parsed shape matches the caller's
//     `originalReceipt` object.  A caller-supplied hash alone is never trusted.
//   * Item 2 — fresh-Jev verdicts must EXPLICITLY carry guardrail /
//     framing / grounding / register set to `pass` (guardrail may be
//     `could-not-tell` iff resolved by a bound disposition).  Empty
//     verdict maps, missing dimensions, `blocked !== false`, mode /
//     pass_id drift, and unknown verdict labels all fail closed.  A
//     guardrail disposition is bound to THIS fresh Jev artifact by
//     structural equality of `disposition.originalJev`.
//   * Item 3 — outcome=`failed` recovery requires ≥1 explicit
//     `source_audit_failed` attempt; any `null`/malformed attempt entry
//     or unknown outcome kind blocks; an all-`ok` attempts list cannot
//     justify a `failed` outcome.
//   * Item 4 — the historical prior-context edges are validated per edge
//     against an explicit `priorContext` allowlist declaring whether the
//     `original` or a `revised` predecessor text was actually consumed at
//     generation time.  Revised-kind edges must (a) match the caller's
//     verified revised output, (b) hash to the receipt's stored edge
//     hash, and (c) appear verbatim in `receipt.synthesis_input_text`.
//     Undeclared revised predecessors that leak into the raw synthesis
//     input still fail closed as forged reruns.
//   * Item 5 — coherence-review `artifactSha256` binds the actual
//     assembled text when supplied; the hash-list fallback only stands
//     in for missing assembly.  Rationale must be substantive; findings
//     must be non-null objects with explicit boolean `unresolved`;
//     malformed findings and duplicate reviewedRevisions block acceptance.

import { createHash } from 'node:crypto';
import { countReportWords } from './rubric.js';
import {
  applyEditorialRevision,
  EditorialRevisionError,
  type EditorialPatch,
  type EditorialReviewer,
  type EditorialRevision,
} from './editorial-revision.js';
import {
  computeAuditInputHash,
  hasCleanAuditEvidence,
  type SourceAuditReceipt,
} from './reference-source-audit.js';
import {
  validateGuardrailDisposition,
  GuardrailDispositionError,
  type GuardrailDispositionArtifact,
  type GuardrailDiagnosticArtifact,
  type JevReviewArtifact,
  type RevisionArtifact,
  type GuardrailDispositionResolution,
} from './guardrail-disposition.js';
import type { SectionExecutionReceipt } from './reference-execution.js';
import { isDeepStrictEqual } from 'node:util';
import {
  validateEditorialPassageEvidence,
  EditorialPassageEvidenceError,
  type EditorialPassageAddition,
  type TrustedSourceRecord,
} from './editorial-passage-evidence.js';
import type { RetrievedPassage } from './reference-execution.js';

// ─── Types ────────────────────────────────────────────────────────────

/** Fresh source-audit sidecar carried alongside a revised section. */
export interface FreshAuditSidecar {
  kind: 'fresh-audit-of-editorial-revision';
  baseReceiptSha256: string;
  sectionOutputHash: string;
  originalOutcome: 'ok' | 'repaired' | 'failed';
  revisionVersion?: string;
  supplementalEvidence?: {
    path: string;
    sha256: string;
    text: string;
    scope: 'Current local implementation; not independent proof of the saved production binary version';
  };
  engineFacts: string;
  instruction?: string;
  audit: SourceAuditReceipt;
  /**
   * Optional editorial passage-evidence bundle recorded by the reviewer
   * CLI.  Present only when a fresh audit was supplemented with fresh
   * additions (a CF readback and/or reviewed-primary-document paraphrases).
   *
   * The bundle carries the ORIGINAL retrieved passages verbatim and the
   * fresh additions only — never rewrites the base retrieval receipt.
   * The verifier revalidates the bundle against caller-supplied trusted
   * records (options.trustedPassageSources / options.expectedCF) and
   * against the base receipt's source_ids/source_hashes/passages_hash,
   * then recomputes the audit input hash from the effective passages
   * hash.  When absent, base-only audit binding is preserved exactly as
   * before — no behaviour change for existing receipts.
   */
  passageEvidence?: {
    basePassages: readonly RetrievedPassage[];
    basePassagesHash: string;
    additions: readonly EditorialPassageAddition[];
    /**
     * The frozen `[...base, ...additions]` view + effective hash produced
     * by validateEditorialPassageEvidence at reviewer time.  The verifier
     * recomputes both and requires deep equality — the stored fields are
     * never trusted on their own.
     */
    addedSourceRecords: readonly EditorialPassageAddition[];
    effectivePassagesHash: string;
  };
  finalAccepted: boolean;
}

/** Fresh Jev sidecar carried alongside a revised section. */
export interface FreshJevSidecar {
  sectionOutputHash: string;
  revisionVersion?: string;
  rubric?: Record<string, unknown>;
  receipt: {
    pass_id: string;
    mode: string;
    status: string;
    model?: string;
    latency_ms?: number;
    answers?: Record<string, unknown>;
    verdicts: Record<string, string>;
    blocked?: boolean;
    disagreements?: string[];
  };
  finalAccepted: boolean;
}

/** Everything the verifier needs to accept ONE revised section. */
export interface EditorialSectionEvidence {
  /** SHA-256 of the ORIGINAL saved receipt file bytes on disk. */
  baseReceiptSha256: string;
  /**
   * Exact raw UTF-8 bytes of the ORIGINAL saved receipt file on disk.
   * The verifier recomputes sha256(baseReceiptRawBytes), requires it to
   * equal `baseReceiptSha256` AND `freshAudit.baseReceiptSha256`, and
   * JSON-parses the bytes to confirm they materialise to the same
   * `SectionExecutionReceipt` the verifier was handed (section_id,
   * output_hash, raw.synthesis).  A caller-supplied hash without the raw
   * bytes it purports to describe is never trusted.
   */
  baseReceiptRawBytes: string;
  /** The immutable revision artifact (as stored on disk) — recomputed and re-verified. */
  revisionArtifact: {
    sectionId: string;
    baseOutputHash: string;
    revisedOutput: string;
    revisedOutputHash: string;
    patches: readonly EditorialPatch[];
    reviewer: EditorialReviewer;
    timestamp: string;
  };
  /** Fresh source-audit-of-editorial-revision sidecar. */
  freshAudit: FreshAuditSidecar;
  /** Fresh judged Jev sidecar. */
  freshJev: FreshJevSidecar;
  /**
   * Canonical review mode the fresh Jev receipt MUST declare (e.g.
   * `'shadow'`).  Any other value blocks acceptance.  Defaults to
   * `'shadow'` when omitted — the standard editorial review mode.
   */
  expectedJevMode?: string;
  /** Full ValidateGuardrailDispositionInput. Required iff freshJev has a `guardrail: could-not-tell`. */
  guardrailDisposition?: {
    disposition: GuardrailDispositionArtifact;
    originalJev: JevReviewArtifact;
    diagnostic: GuardrailDiagnosticArtifact;
    revision: RevisionArtifact;
    rawEvidenceTexts: Readonly<Record<string, string>>;
  };
}

/**
 * Historical prior-context edge declaration.  For each earlier section a
 * later section legitimately consumed at generation time, the caller
 * declares WHICH version was consumed and supplies its exact text.
 *
 *   * `kind: 'original'` — the pre-edit original pass output.  `text` must
 *     equal `originalPasses[earlierId]`.
 *   * `kind: 'revised'` — a previously verified revised version.  `text`
 *     must equal `revisedSections[earlierId].revisionArtifact.revisedOutput`,
 *     and the earlier section MUST be present in `revisedSections`.  This
 *     is the ONLY way a revised predecessor may legitimately appear in a
 *     later section's raw synthesis input; it never forges a later edit
 *     into an older generation because the edge is bound by hash to the
 *     receipt's own `prior_section_content_hashes` and by inclusion to
 *     the receipt's own `synthesis_input_text`.
 */
export interface PriorContextEdge {
  kind: 'original' | 'revised';
  /** Exact raw text of the predecessor version consumed at generation time. */
  text: string;
}

/**
 * One item of the whole-document coherence-review finding list. `unresolved`
 * findings block the whole report even if every section check passes.
 */
export interface CoherenceReviewFinding {
  id: string;
  summary: string;
  unresolved: boolean;
}

/** Reviewed revision reference cited in the coherence review. */
export interface ReviewedRevisionRef {
  sectionId: string;
  revisedOutputHash: string;
}

/** Whole-document editorial coherence review. */
export interface EditorialCoherenceReview {
  /** SHA-256 of the assembled artifact (either passes-based, or the caller-supplied `assembled`). */
  artifactSha256: string;
  /** Effective output hash for every section in requiredSectionIds order (revised → revised hash, else original output_hash). */
  orderedEffectiveHashes: string[];
  status: 'reviewed';
  reviewer: EditorialReviewer;
  reviewedAt: string;
  rationale: string;
  findings: readonly CoherenceReviewFinding[];
  reviewedRevisions: readonly ReviewedRevisionRef[];
}

/**
 * Optional typed evidence map handed to verifyReferenceExecution to enable
 * provenance-aware editorial acceptance.  When omitted the base verifier
 * behaves exactly as before.
 */
export interface EditorialVerificationOptions {
  /** section_id → per-section editorial evidence bundle. */
  revisedSections: Readonly<Record<string, EditorialSectionEvidence>>;
  /** Previously reviewed revisions consumed by later generations; independently reverified. */
  historicalRevisions?: Readonly<Record<string, readonly EditorialSectionEvidence[]>>;
  /**
   * Original passes text keyed by section_id.  Used to validate the raw
   * receipts' `prior_section_content_hashes` and to detect a revised
   * section's raw synthesis illegitimately consuming other revised
   * outputs.  Must contain every section_id in requiredSectionIds.
   */
  originalPasses: Readonly<Record<string, string>>;
  /** Whole-document editorial coherence review; required when revisedSections is nonempty. */
  coherenceReview?: EditorialCoherenceReview;
  /**
   * Explicit current-implementation-source texts, keyed by path.  Only
   * these paths may appear as `freshAudit.supplementalEvidence.path`.
   * Their sha256 and text must match exactly, as editorial-prefix validates.
   */
  implementationSourceTexts?: Readonly<Record<string, string>>;
  /**
   * Explicit per-section historical prior-context edges.  Keyed as
   * `priorContext[sectionId][earlierId] = { kind, text }`.  For every
   * `earlierId` in the receipt's `prior_section_content_hashes`, the
   * verifier requires an edge declaration.  Undeclared edges default to
   * `original` and MUST match `originalPasses[earlierId]` — silence never
   * admits a revised predecessor.  The rules are:
   *   * Every declared edge's `sha256(text)` MUST equal the receipt's
   *     stored `prior_section_content_hashes[earlierId]`.
   *   * Every declared edge's `text` MUST appear verbatim in the
   *     receipt's `synthesis_input_text` (proves the generator saw it).
   *   * `kind: 'original'` requires `text === originalPasses[earlierId]`.
   *   * `kind: 'revised'` requires the earlier section to be in
   *     `revisedSections` and `text === revisedSections[earlierId]
   *     .revisionArtifact.revisedOutput`, or independently reverified historical evidence.
   *   * Any OTHER revised section's text found inside
   *     `synthesis_input_text` that is not declared as consumed on this
   *     edge is a forged rerun and blocks acceptance.
   */
  priorContext?: Readonly<Record<string, Readonly<Record<string, PriorContextEdge>>>>;
  /**
   * Independently trusted reviewed-source registry keyed by addition id.
   * Consumed ONLY when a section's `freshAudit.passageEvidence` is
   * present.  Callers must build this from their own source of truth
   * (approved reviewer registry, fresh CF readback ledger) — never from
   * the evidence sidecar the audit records; the sidecar can never
   * whitelist its own additions.  When omitted, sections carrying
   * passage evidence fail closed with an explicit
   * `editorial_audit_passage_evidence_untrusted_source` blocker.
   */
  trustedPassageSources?: Readonly<Record<string, TrustedSourceRecord>>;
  /**
   * Independently trusted CF target the reviewer readback targeted.
   * Required whenever any addition carries `provenance.kind ===
   * 'cf-vectorize'`.  Callers derive this from their own inputs
   * (typically `inputs.json` accountId + index), never from the
   * evidence sidecar itself.  Editorial acceptance rejects CF additions
   * whose account/index differ from this value byte-for-byte.
   */
  expectedCF?: { account: string; index: string };
}

/** Result of verifying one revised section. */
export interface EditorialSectionVerdict {
  sectionId: string;
  revisedOutputHash: string;
  blockers: string[];
  notes: string[];
  guardrailResolution?: GuardrailDispositionResolution;
}

// ─── Helpers ──────────────────────────────────────────────────────────

const sha256 = (text: string): string =>
  createHash('sha256').update(text, 'utf8').digest('hex');

const SUPPLEMENTAL_SCOPE =
  'Current local implementation; not independent proof of the saved production binary version';

/** Attempt outcome kinds the verifier recognises. Anything else is `unknown`. */
const KNOWN_ATTEMPT_OUTCOMES = new Set<string>([
  'ok',
  'source_audit_failed',
  'source_audit_reaudit_failed',
  'missing_subsections',
  'retrieval_empty',
  'retrieval_failure',
  'voice_error',
  'synthesis_error',
  'synthesis_transient_retry',
  // `word_count` is the outcome kind reference-execution.ts actually writes
  // for a word-count repair rejection.  `word_count_failed` is retained as a
  // legacy synonym for older synthetic fixtures.
  'word_count',
  'word_count_failed',
  'jev_failed',
  'guardrail_failed',
]);

const REQUIRED_JEV_DIMENSIONS = ['guardrail', 'framing', 'grounding', 'register'] as const;

// ─── Original outcome classification (shared, pure) ────────────────────
//
// A `failed` receipt is recoverable through editorial revision only under
// one of three BOUNDED classifications — never as a general "failed → accept
// anyway" escape hatch:
//
//   * `source_audit_failed_recovery` — the original policy: every attempt
//     is `ok` or `source_audit_failed` AND at least one attempt actually
//     records a `source_audit_failed`.  This does not require a nonempty
//     saved draft (the receipt may have empty voices).
//
//   * `word_count_recovery` — the new, narrower Part V recovery: the
//     synthesis reached a nonempty draft that the word-count repair loop
//     rejected, then the same-request retrieval/transport machinery
//     timed out.  The saved draft + both witness voices are intact and
//     hash-verified; the reviewer can now repair the draft content and
//     collect a fresh audit + Jev against the revised output.
//
//   * `source_audit_reaudit_recovery` — generation reached ok, then the
//     audit repair failed with hash-bound actionable findings. Both voices
//     and the terminal draft must remain intact. Fresh revision, audit, Jev
//     and word fit are mandatory.
//
// Recovery is EXCLUSIVE — a receipt cannot be classified as both.  The
// classifier fails closed on any malformed / unknown attempt, mixed
// families, missing timeout reason, or tampered raw voices.  It never
// changes the raw stored outcome.

export type OriginalOutcomeClassification =
  | { kind: 'ok' }
  | { kind: 'repaired' }
  | { kind: 'source_audit_failed_recovery' }
  | { kind: 'word_count_recovery' }
  | { kind: 'source_audit_reaudit_recovery' }
  | { kind: 'source_audit_after_word_fit_recovery' }
  | { kind: 'blocked'; blockers: string[] };

const WORD_COUNT_ATTEMPT_KINDS = new Set(['word_count', 'word_count_failed']);

function reasonHas(rec: Record<string, unknown>, needle: string): boolean {
  const reason = rec.reason;
  return typeof reason === 'string' && reason.toLowerCase().includes(needle);
}

function nonEmptyString(value: unknown): value is string {
  return typeof value === 'string' && value.length > 0;
}

/**
 * Classify an original receipt's outcome for editorial acceptance.  Pure:
 * no I/O, no clocks, no mutation.  All hashes are recomputed here from the
 * receipt's raw bytes; callers must not shortcut this by trusting stored
 * hash fields alone.
 *
 * The returned classification is used identically by both
 * `verifyEditorialSection` (editorial-verification) and the prefix loader
 * (editorial-prefix) so the two entry points cannot drift.
 */
export function classifyOriginalOutcome(
  receipt: SectionExecutionReceipt,
): OriginalOutcomeClassification {
  const outcome = receipt.outcome;
  if (outcome === 'ok') return { kind: 'ok' };
  if (outcome === 'repaired') return { kind: 'repaired' };
  if (outcome !== 'failed') {
    return { kind: 'blocked', blockers: [`editorial_original_outcome_unknown:${String(outcome)}`] };
  }

  const attempts = Array.isArray(receipt.attempts) ? receipt.attempts : null;
  if (!attempts || attempts.length === 0) {
    return { kind: 'blocked', blockers: ['editorial_original_failed_no_attempts'] };
  }

  // Well-formedness + known-kind pass.
  let sawSourceAuditFailed = false;
  let sawWordCount = false;
  let firstWordCountIdx = -1;
  const kinds: string[] = [];
  const disallowed = new Set<string>();
  for (let i = 0; i < attempts.length; i += 1) {
    const a = attempts[i] as unknown;
    if (!a || typeof a !== 'object') {
      return { kind: 'blocked', blockers: [`editorial_original_failed_malformed_attempt:${i}`] };
    }
    const rec = a as Record<string, unknown>;
    const kind = rec.outcome;
    if (typeof kind !== 'string' || kind.length === 0) {
      return { kind: 'blocked', blockers: [`editorial_original_failed_malformed_attempt:${i}`] };
    }
    if (!KNOWN_ATTEMPT_OUTCOMES.has(kind)) {
      return { kind: 'blocked', blockers: [`editorial_original_failed_unknown_outcome:${kind}`] };
    }
    kinds.push(kind);
    if (kind === 'source_audit_failed') sawSourceAuditFailed = true;
    if (WORD_COUNT_ATTEMPT_KINDS.has(kind)) {
      sawWordCount = true;
      if (firstWordCountIdx < 0) firstWordCountIdx = i;
    }
    if (kind !== 'ok' && kind !== 'source_audit_failed' && !WORD_COUNT_ATTEMPT_KINDS.has(kind) &&
        kind !== 'synthesis_error' && kind !== 'synthesis_transient_retry') {
      disallowed.add(kind);
    }
  }

  // A completed generation followed by a failed audit repair is a separate
  // bounded path. Earlier word-count attempts must have reached `ok` first.
  if (kinds.includes('source_audit_reaudit_failed')) {
    const blockers: string[] = [];
    const terminal = attempts[attempts.length - 1];
    const prior = kinds.slice(0, -1);
    if (kinds.at(-1) !== 'source_audit_reaudit_failed' || prior.at(-1) !== 'ok' ||
        prior.slice(0, -1).some(k => !WORD_COUNT_ATTEMPT_KINDS.has(k))) {
      blockers.push('editorial_original_reaudit_invalid_sequence');
    }
    const raw = receipt.raw;
    for (const [field, hash] of [
      ['synthesis', receipt.output_hash], ['aletheios', receipt.aletheios_hash],
      ['pichet', receipt.pichet_hash],
    ] as const) {
      if (!raw || !nonEmptyString(raw[field]) || sha256(raw[field]) !== hash ||
          terminal[`${field}_raw`] !== raw[field]) {
        blockers.push(`editorial_original_reaudit_raw_mismatch:${field}`);
      }
    }
    const audit = receipt.source_audit;
    if (!audit || audit.section_id !== receipt.section_id ||
        audit.status !== 'changes-required' ||
        !Array.isArray(audit.findings) || !audit.findings.some(f => f.severity === 'error') ||
        !nonEmptyString(audit.model_raw) || sha256(audit.model_raw) !== audit.output_sha256 ||
        !nonEmptyString(receipt.engine_facts) || !raw ||
        audit.input_sha256 !== computeAuditInputHash({
          acceptedOutput: raw.synthesis, passagesHash: receipt.passages_hash,
          engineFacts: receipt.engine_facts ?? '',
        })) {
      blockers.push('editorial_original_reaudit_evidence_invalid');
    }
    return blockers.length ? { kind: 'blocked', blockers } : { kind: 'source_audit_reaudit_recovery' };
  }

  // The generator can repair word fit successfully before an audit returns
  // an unusable ledger. Admit only that exact sequence with intact evidence;
  // the revised section must still earn a fresh audit, Jev and word-fit check.
  if (sawWordCount && kinds.at(-1) === 'source_audit_failed') {
    const blockers: string[] = [];
    const prior = kinds.slice(0, -1);
    if (prior.at(-1) !== 'ok' || prior.slice(0, -1).some(k => !WORD_COUNT_ATTEMPT_KINDS.has(k))) {
      blockers.push('editorial_original_post_word_fit_invalid_sequence');
    }
    const terminal = attempts[attempts.length - 1];
    for (const [field, expected] of [['synthesis', receipt.output_hash], ['aletheios', receipt.aletheios_hash], ['pichet', receipt.pichet_hash]] as const) {
      if (!receipt.raw || !nonEmptyString(receipt.raw[field]) || sha256(receipt.raw[field]) !== expected || terminal[`${field}_raw`] !== receipt.raw[field]) {
        blockers.push(`editorial_original_post_word_fit_raw_mismatch:${field}`);
      }
    }
    const audit = receipt.source_audit;
    if (!audit || audit.section_id !== receipt.section_id ||
        !['insufficient', 'malformed'].includes(audit.status) ||
        !nonEmptyString(audit.model_raw) || sha256(audit.model_raw) !== audit.output_sha256 ||
        !nonEmptyString(receipt.engine_facts) || !receipt.raw ||
        audit.input_sha256 !== computeAuditInputHash({acceptedOutput: receipt.raw.synthesis, passagesHash: receipt.passages_hash, engineFacts: receipt.engine_facts ?? ''})) {
      blockers.push('editorial_original_post_word_fit_audit_invalid');
    }
    return blockers.length ? {kind: 'blocked', blockers} : {kind: 'source_audit_after_word_fit_recovery'};
  }

  // ── Source-audit-only path (unchanged legacy policy) ──────────
  const sourceAuditOnlyDisallowed = new Set<string>();
  for (const k of kinds) {
    if (k !== 'ok' && k !== 'source_audit_failed') sourceAuditOnlyDisallowed.add(k);
  }
  if (sourceAuditOnlyDisallowed.size === 0) {
    if (!sawSourceAuditFailed) {
      return {
        kind: 'blocked',
        blockers: ['editorial_original_failed_no_source_audit_failed_attempt'],
      };
    }
    return { kind: 'source_audit_failed_recovery' };
  }

  // ── Word-count recovery path (new, bounded) ───────────────────
  //
  // Requires a word_count attempt, allows synthesis_transient_retry (transport
  // only) and synthesis_error (explicit timeout, and only AFTER a word_count
  // attempt — never a pre-witness failure), and demands intact hashed raw
  // synthesis + both witness voices.  Source-audit failure is not permitted
  // here (the two recovery classifications are mutually exclusive).
  const blockers: string[] = [];
  if (disallowed.size > 0) {
    blockers.push(
      `editorial_original_failed_disallowed_kinds:${[...disallowed].sort().join(',')}`,
    );
  }
  if (sawSourceAuditFailed) {
    // A mixed family — source-audit + word-count — is not an accepted
    // classification.  Reviewer must repair whichever gate actually failed
    // rather than combining recovery kinds.
    blockers.push('editorial_original_failed_mixed_source_audit_and_word_count');
  }
  if (!sawWordCount) {
    blockers.push('editorial_original_failed_no_word_count_attempt');
  }

  // synthesis_error / synthesis_transient_retry reason gates.
  for (let i = 0; i < attempts.length; i += 1) {
    const rec = attempts[i] as unknown as Record<string, unknown>;
    const kind = rec.outcome as string;
    if (kind === 'synthesis_transient_retry') {
      if (!reasonHas(rec, 'transient transport')) {
        blockers.push(`editorial_original_failed_transient_retry_reason_not_transport:${i}`);
      }
    } else if (kind === 'synthesis_error') {
      if (firstWordCountIdx < 0 || i < firstWordCountIdx) {
        blockers.push(`editorial_original_failed_synthesis_error_before_word_count:${i}`);
      }
      if (!reasonHas(rec, 'timeout') && !reasonHas(rec, 'aborted due to timeout')) {
        blockers.push(`editorial_original_failed_synthesis_error_reason_not_timeout:${i}`);
      }
    }
  }

  // Saved draft + both voices must be intact and hash-bound.
  const raw = receipt.raw as { synthesis?: unknown; aletheios?: unknown; pichet?: unknown } | undefined;
  if (!raw || typeof raw !== 'object') {
    blockers.push('editorial_original_failed_word_count_raw_missing');
  } else {
    if (!nonEmptyString(raw.synthesis)) {
      blockers.push('editorial_original_failed_word_count_raw_synthesis_empty');
    } else if (sha256(raw.synthesis) !== receipt.output_hash) {
      blockers.push('editorial_original_failed_word_count_raw_synthesis_hash_mismatch');
    }
    if (!nonEmptyString(raw.aletheios)) {
      blockers.push('editorial_original_failed_word_count_raw_aletheios_empty');
    } else if (sha256(raw.aletheios) !== receipt.aletheios_hash) {
      blockers.push('editorial_original_failed_word_count_raw_aletheios_hash_mismatch');
    }
    if (!nonEmptyString(raw.pichet)) {
      blockers.push('editorial_original_failed_word_count_raw_pichet_empty');
    } else if (sha256(raw.pichet) !== receipt.pichet_hash) {
      blockers.push('editorial_original_failed_word_count_raw_pichet_hash_mismatch');
    }
  }

  if (blockers.length > 0) return { kind: 'blocked', blockers };
  return { kind: 'word_count_recovery' };
}

/**
 * Parse the raw receipt bytes and confirm they materialise to the same
 * SectionExecutionReceipt the verifier was handed.  Returns a blocker code
 * on any mismatch, or null on success.  Only the fields the editorial gate
 * depends on are compared — this is a shape check, not a full deep equal
 * (the caller may have added transient annotations in memory that never
 * touched disk).
 */
function verifyBaseReceiptRawBytes(
  originalReceipt: SectionExecutionReceipt,
  rawBytes: unknown,
): { computedHash: string; blockers: string[] } {
  const blockers: string[] = [];
  if (typeof rawBytes !== 'string' || rawBytes.length === 0) {
    blockers.push('editorial_base_receipt_raw_bytes_missing');
    return { computedHash: '', blockers };
  }
  const computedHash = sha256(rawBytes);
  let parsed: unknown;
  try {
    parsed = JSON.parse(rawBytes);
  } catch {
    blockers.push('editorial_base_receipt_raw_bytes_unparseable');
    return { computedHash, blockers };
  }
  if (!parsed || typeof parsed !== 'object') {
    blockers.push('editorial_base_receipt_raw_bytes_shape_invalid');
    return { computedHash, blockers };
  }
  const p = parsed as Record<string, unknown>;
  if (!isDeepStrictEqual(parsed, originalReceipt)) {
    blockers.push('editorial_base_receipt_raw_bytes_content_mismatch');
  }
  if (p.section_id !== originalReceipt.section_id) {
    blockers.push('editorial_base_receipt_raw_bytes_section_id_mismatch');
  }
  if (p.output_hash !== originalReceipt.output_hash) {
    blockers.push('editorial_base_receipt_raw_bytes_output_hash_mismatch');
  }
  const rawField = p.raw as Record<string, unknown> | undefined;
  if (!rawField || rawField.synthesis !== originalReceipt.raw.synthesis) {
    blockers.push('editorial_base_receipt_raw_bytes_synthesis_mismatch');
  }
  if (p.outcome !== originalReceipt.outcome) {
    blockers.push('editorial_base_receipt_raw_bytes_outcome_mismatch');
  }
  return { computedHash, blockers };
}

/**
 * Verify a single revised section end-to-end.
 *
 * Pure; never mutates the original receipt or any input; never fabricates a
 * verdict; never converts a `could-not-tell` outside the guardrail dimension
 * into acceptance.  Returns a verdict list of blockers/warnings and, when
 * present, the frozen guardrail disposition resolution.
 */
export function verifyEditorialSection(args: {
  sectionId: string;
  originalReceipt: SectionExecutionReceipt;
  passOutput: string;
  passOutputHash: string;
  evidence: EditorialSectionEvidence;
  implementationSourceTexts?: Readonly<Record<string, string>>;
  /**
   * Independently trusted reviewed-source registry keyed by addition id
   * (see EditorialVerificationOptions).  Required whenever the section's
   * `evidence.freshAudit.passageEvidence` is present; absent-evidence
   * sections ignore this field so existing APIs preserve behaviour.
   */
  trustedPassageSources?: Readonly<Record<string, TrustedSourceRecord>>;
  /**
   * Independently trusted CF target the reviewer readback targeted (see
   * EditorialVerificationOptions).  Required when any addition carries
   * cf-vectorize provenance.
   */
  expectedCF?: { account: string; index: string };
}): EditorialSectionVerdict {
  const { sectionId, originalReceipt, passOutput, passOutputHash, evidence, implementationSourceTexts, trustedPassageSources, expectedCF } = args;
  const blockers: string[] = [];
  const notes: string[] = [];
  const prefix = sectionId;
  let guardrailResolution: GuardrailDispositionResolution | undefined;

  // ── 0. Original receipt outcome eligibility (item 3 hardening) ──
  // A failed-outcome receipt is recoverable via editorial revision + fresh
  // audit under EXACTLY two bounded classifications, mutually exclusive:
  //   * source_audit_failed_recovery — every attempt is `ok` or
  //     `source_audit_failed`, at least one is `source_audit_failed`.
  //   * word_count_recovery — reference-execution reached a nonempty draft
  //     that the word-count repair loop rejected, then the same-request
  //     retrieval/transport machinery timed out; saved draft + both raw
  //     voices are intact and hash-verified.
  // A shared pure classifier (`classifyOriginalOutcome`) enforces both
  // paths identically for the prefix loader and this verifier, so the two
  // entry points cannot drift.  Recovery never rewrites the raw stored
  // outcome; it only decides whether editorial repair is permitted.
  const originalClassification = classifyOriginalOutcome(originalReceipt);
  if (originalClassification.kind === 'blocked') {
    for (const code of originalClassification.blockers) {
      blockers.push(`${prefix}:${code}`);
    }
  }

  // ── 0.5 Raw receipt bytes (item 1 hardening) ────────────────────
  // The caller supplies the ORIGINAL saved receipt file's raw bytes.
  // Recompute their sha256 here and cross-check both the caller's
  // `baseReceiptSha256` and the fresh audit sidecar's
  // `baseReceiptSha256`.  Parse them and confirm the parsed content
  // materialises to the same receipt the verifier was handed.  A hash
  // without its bytes proves nothing.
  const rawCheck = verifyBaseReceiptRawBytes(originalReceipt, (evidence as { baseReceiptRawBytes?: unknown }).baseReceiptRawBytes);
  for (const b of rawCheck.blockers) blockers.push(`${prefix}:${b}`);
  const computedBaseReceiptHash = rawCheck.computedHash;
  if (computedBaseReceiptHash && evidence.baseReceiptSha256 !== computedBaseReceiptHash) {
    blockers.push(`${prefix}:editorial_base_receipt_hash_mismatch`);
  }

  // ── 1. Revision artifact re-application (never trust stored bytes) ─
  const stored = evidence.revisionArtifact;
  if (stored.sectionId !== sectionId) {
    blockers.push(`${prefix}:editorial_revision_section_id_mismatch`);
  }
  if (stored.baseOutputHash !== originalReceipt.output_hash) {
    blockers.push(`${prefix}:editorial_base_output_hash_mismatch`);
  }
  if (sha256(originalReceipt.raw.synthesis) !== originalReceipt.output_hash) {
    blockers.push(`${prefix}:editorial_original_synthesis_tampered`);
  }
  let recomputed: EditorialRevision | null = null;
  try {
    recomputed = applyEditorialRevision({
      baseReceipt: originalReceipt,
      currentOutput: originalReceipt.raw.synthesis,
      expectedBaseOutputHash: originalReceipt.output_hash,
      patches: stored.patches,
      reviewer: stored.reviewer,
      timestamp: stored.timestamp,
    });
  } catch (err) {
    const code = err instanceof EditorialRevisionError ? err.code : 'apply_failed';
    blockers.push(`${prefix}:editorial_apply_failed:${code}`);
  }
  if (recomputed) {
    if (recomputed.revisedOutputHash !== stored.revisedOutputHash) {
      blockers.push(`${prefix}:editorial_stored_revised_hash_mismatch`);
    }
    if (recomputed.revisedOutput !== stored.revisedOutput) {
      blockers.push(`${prefix}:editorial_stored_revised_text_mismatch`);
    }
    if (passOutputHash !== recomputed.revisedOutputHash) {
      blockers.push(`${prefix}:editorial_pass_output_hash_mismatch`);
    }
    if (passOutput !== recomputed.revisedOutput) {
      blockers.push(`${prefix}:editorial_pass_output_text_mismatch`);
    }
    // Word-count recovery MUST produce a revised output that actually
    // differs from the raw draft the pipeline rejected — otherwise the
    // reviewer merely re-published the rejected text, and the failed
    // outcome would be laundered as an editorial acceptance.  A
    // source-audit-only recovery may legitimately keep the text unchanged
    // (the audit was the failure, not the text).
    if ((originalClassification.kind === 'word_count_recovery' || originalClassification.kind === 'source_audit_reaudit_recovery' || originalClassification.kind === 'source_audit_after_word_fit_recovery')) {
      if (recomputed.revisedOutput === originalReceipt.raw.synthesis) {
        blockers.push(`${prefix}:editorial_word_count_recovery_no_revision`);
      }
      if (recomputed.revisedOutputHash === originalReceipt.output_hash) {
        blockers.push(`${prefix}:editorial_word_count_recovery_hash_unchanged`);
      }
    }
  }

  const revisedOutput = recomputed?.revisedOutput ?? stored.revisedOutput;
  const revisedOutputHash = recomputed?.revisedOutputHash ?? stored.revisedOutputHash;

  // ── 2. Fresh source-audit ───────────────────────────────────────
  const freshAudit = evidence.freshAudit;
  if (freshAudit.kind !== 'fresh-audit-of-editorial-revision') {
    blockers.push(`${prefix}:editorial_audit_wrong_kind`);
  }
  if (freshAudit.baseReceiptSha256 !== evidence.baseReceiptSha256) {
    blockers.push(`${prefix}:editorial_audit_base_receipt_mismatch`);
  }
  if (computedBaseReceiptHash && freshAudit.baseReceiptSha256 !== computedBaseReceiptHash) {
    blockers.push(`${prefix}:editorial_audit_base_receipt_bytes_mismatch`);
  }
  if (freshAudit.sectionOutputHash !== revisedOutputHash) {
    blockers.push(`${prefix}:editorial_audit_section_output_hash_mismatch`);
  }
  // engineFacts must equal base.engine_facts + bounded implementation appendix (if any)
  const baseEngineFacts = originalReceipt.engine_facts;
  if (typeof baseEngineFacts !== 'string' || baseEngineFacts.length === 0) {
    blockers.push(`${prefix}:editorial_original_engine_facts_missing`);
  }
  let expectedEngineFacts = baseEngineFacts ?? '';
  if (freshAudit.supplementalEvidence !== undefined) {
    const supp = freshAudit.supplementalEvidence;
    const suppOk =
      !!supp &&
      typeof supp.path === 'string' &&
      typeof supp.text === 'string' &&
      typeof supp.sha256 === 'string' &&
      supp.scope === SUPPLEMENTAL_SCOPE &&
      supp.sha256 === sha256(supp.text) &&
      implementationSourceTexts?.[supp.path] === supp.text &&
      Object.keys(supp).every((k) => ['path', 'sha256', 'text', 'scope'].includes(k));
    if (!suppOk) {
      blockers.push(`${prefix}:editorial_audit_supplemental_evidence_invalid`);
    } else {
      expectedEngineFacts += '\nAdditional implementation evidence, with limited scope:\n' + JSON.stringify(supp);
    }
  }
  if (freshAudit.engineFacts !== expectedEngineFacts) {
    blockers.push(`${prefix}:editorial_audit_engine_facts_drift`);
  }

  // ── 2.5 Optional passage evidence (supplemented fresh audit) ───
  // When the reviewer supplemented the base retrieval with fresh
  // additions the audit ran against, `freshAudit.passageEvidence`
  // carries the exact base + additions bundle.  Revalidate it against
  // the base receipt (source_ids / source_hashes / passages_hash) and
  // against the caller-supplied independently trusted registry + CF
  // target.  Recompute the effective passages hash and bind the audit
  // input hash to it.  When absent, existing behaviour is preserved
  // exactly: base-only audit binding via originalReceipt.passages_hash.
  let effectivePassagesHash: string = originalReceipt.passages_hash;
  const passageEvidence = freshAudit.passageEvidence;
  const passageShapeValid = passageEvidence != null &&
    Array.isArray(passageEvidence.basePassages) &&
    passageEvidence.basePassages.every(p => p && typeof p.id === 'string' && typeof p.text === 'string') &&
    Array.isArray(passageEvidence.additions) && Array.isArray(passageEvidence.addedSourceRecords);
  if (passageEvidence !== undefined && !passageShapeValid) {
    blockers.push(`${prefix}:editorial_audit_passage_evidence_invalid`);
  } else if (passageEvidence !== undefined) {
    // Base IDs / hash / texts must round-trip against the original
    // receipt EXACTLY, in order.  Silence never admits divergence.
    const baseIds = originalReceipt.source_ids ?? [];
    const baseHashes = originalReceipt.source_hashes ?? [];
    const bp = Array.isArray(passageEvidence.basePassages) ? passageEvidence.basePassages : [];
    if (bp.length !== baseIds.length) {
      blockers.push(`${prefix}:editorial_audit_passage_evidence_base_length_mismatch`);
    } else {
      let idsOk = true;
      for (let i = 0; i < bp.length; i += 1) {
        if (bp[i].id !== baseIds[i]) {
          blockers.push(`${prefix}:editorial_audit_passage_evidence_base_id_mismatch:${i}`);
          idsOk = false;
        } else if (sha256(bp[i].text) !== baseHashes[i]) {
          blockers.push(`${prefix}:editorial_audit_passage_evidence_base_text_mismatch:${baseIds[i]}`);
        }
      }
      if (idsOk && passageEvidence.basePassagesHash !== originalReceipt.passages_hash) {
        blockers.push(`${prefix}:editorial_audit_passage_evidence_base_hash_mismatch`);
      }
    }

    // Additions must round-trip against the caller's independently
    // supplied trusted registry — the sidecar never whitelists itself.
    if (!trustedPassageSources) {
      blockers.push(`${prefix}:editorial_audit_passage_evidence_untrusted_source`);
    } else {
      // Build trustedOriginalTexts from the base passages we just
      // proved match the base receipt.  The receipt IS the independent
      // authority for base-text drift here — its source_hashes were
      // cross-checked one-by-one above.
      const trustedOriginalTexts: Record<string, string> = {};
      for (const bpp of bp) trustedOriginalTexts[bpp.id] = bpp.text;
      const needsCF = (passageEvidence.additions ?? []).some(
        (a) => a && a.provenance && a.provenance.kind === 'cf-vectorize',
      );
      if (needsCF && !expectedCF) {
        blockers.push(`${prefix}:editorial_audit_passage_evidence_expected_cf_missing`);
      } else {
        try {
          const revalidated = validateEditorialPassageEvidence({
            expectedCF,
            basePassages: bp,
            basePassagesHash: originalReceipt.passages_hash,
            additions: passageEvidence.additions ?? [],
            trustedSourceRecords: trustedPassageSources,
            trustedOriginalTexts,
          });
          if (revalidated.effectivePassagesHash !== passageEvidence.effectivePassagesHash) {
            blockers.push(`${prefix}:editorial_audit_passage_evidence_effective_hash_mismatch`);
          }
          // Recorded added-source-records must deep-equal the
          // freshly-revalidated set — no forged provenance survives.
          if (!isDeepStrictEqual(
            (passageEvidence.addedSourceRecords ?? []).map((r) => JSON.parse(JSON.stringify(r))),
            revalidated.addedSourceRecords.map((r) => JSON.parse(JSON.stringify(r))),
          )) {
            blockers.push(`${prefix}:editorial_audit_passage_evidence_added_records_mismatch`);
          }
          effectivePassagesHash = revalidated.effectivePassagesHash;
        } catch (err) {
          const code = err instanceof EditorialPassageEvidenceError ? 'invalid' : 'error';
          blockers.push(`${prefix}:editorial_audit_passage_evidence_${code}`);
        }
      }
    }
  }

  const expectedAuditInputHash = computeAuditInputHash({
    acceptedOutput: revisedOutput,
    passagesHash: effectivePassagesHash,
    engineFacts: expectedEngineFacts,
  });
  if (freshAudit.audit?.input_sha256 !== expectedAuditInputHash) {
    blockers.push(`${prefix}:editorial_audit_input_hash_mismatch`);
  }
  if (!freshAudit.audit || freshAudit.audit.status !== 'clean') {
    blockers.push(
      `${prefix}:editorial_audit_status_${String(freshAudit.audit?.status ?? 'missing')}`,
    );
  } else {
    if (!hasCleanAuditEvidence(freshAudit.audit, revisedOutput)) {
      blockers.push(`${prefix}:editorial_audit_evidence_invalid`);
    }
    if (sha256(freshAudit.audit.model_raw) !== freshAudit.audit.output_sha256) {
      blockers.push(`${prefix}:editorial_audit_output_hash_mismatch`);
    }
    if (freshAudit.audit.coverage?.total_claims_audited <= 0) {
      blockers.push(`${prefix}:editorial_audit_zero_coverage`);
    }
  }

  // ── 3. Fresh Jev + guardrail disposition (item 2 hardening) ─────
  const freshJev = evidence.freshJev;
  if ((originalClassification.kind === 'word_count_recovery' || originalClassification.kind === 'source_audit_reaudit_recovery' || originalClassification.kind === 'source_audit_after_word_fit_recovery') && !hasRecoveredWordFit(revisedOutput, freshJev.rubric)) {
    blockers.push(`${prefix}:editorial_word_count_recovery_fresh_word_fit_missing`);
  }
  const expectedJevMode = evidence.expectedJevMode ?? 'shadow';
  if (freshJev.sectionOutputHash !== revisedOutputHash) {
    blockers.push(`${prefix}:editorial_jev_section_output_hash_mismatch`);
  }
  const jevReceipt = freshJev.receipt;
  if (!jevReceipt || typeof jevReceipt !== 'object') {
    blockers.push(`${prefix}:editorial_jev_receipt_missing`);
  } else {
    // Bindings back to this section — the Jev must speak of THIS section
    if (jevReceipt.pass_id !== sectionId) {
      blockers.push(`${prefix}:editorial_jev_pass_id_mismatch:${String(jevReceipt.pass_id)}`);
    }
    if (jevReceipt.mode !== expectedJevMode) {
      blockers.push(`${prefix}:editorial_jev_mode_mismatch:${String(jevReceipt.mode)}`);
    }
    if (jevReceipt.status !== 'judged') {
      blockers.push(`${prefix}:editorial_jev_not_judged`);
    }
    // `blocked` must be EXPLICITLY false — undefined is not acceptance.
    if (jevReceipt.blocked !== false) {
      blockers.push(`${prefix}:editorial_jev_blocked_or_unset`);
    }
    const verdicts = jevReceipt.verdicts;
    if (!verdicts || typeof verdicts !== 'object') {
      blockers.push(`${prefix}:editorial_jev_verdicts_missing`);
    } else {
      // Every required dimension must be present.
      for (const dim of REQUIRED_JEV_DIMENSIONS) {
        if (!(dim in verdicts)) {
          blockers.push(`${prefix}:editorial_jev_dimension_missing:${dim}`);
        }
      }
      // Every present verdict must be pass, or guardrail=could-not-tell
      // (resolved below via disposition).  Unknown labels fail closed.
      for (const [dim, verdict] of Object.entries(verdicts)) {
        if (verdict === 'fail') {
          blockers.push(`${prefix}:editorial_jev_verdict_fail:${dim}`);
        } else if (verdict === 'could-not-tell') {
          if (dim !== 'guardrail') {
            blockers.push(`${prefix}:editorial_jev_verdict_unresolved:${dim}`);
          }
        } else if (verdict !== 'pass') {
          blockers.push(`${prefix}:editorial_jev_verdict_non_pass:${dim}:${String(verdict)}`);
        }
      }
      if (
        jevReceipt.status === 'judged' &&
        jevReceipt.blocked === false &&
        verdicts.guardrail === 'could-not-tell'
      ) {
        const disp = evidence.guardrailDisposition;
        if (!disp) {
          blockers.push(`${prefix}:editorial_guardrail_disposition_missing`);
        } else {
          // Item 2 doctrine — the disposition MUST bind to THIS fresh Jev
          // artifact, not another independently valid Jev review of the
          // same prose.  Enforce structural equality of the originalJev
          // bundle carried by the disposition versus the fresh Jev
          // receipt the caller supplied.  Any drift blocks.
          // The disposition validator binds the complete parsed JSON sidecar
          // to its raw file hash. Compare that same complete artifact here;
          // reducing it to verdicts loses review identity and rejects real files.
          if (!isDeepStrictEqual(disp.originalJev, freshJev)) {
            blockers.push(`${prefix}:editorial_guardrail_disposition_jev_binding_mismatch`);
          } else {
            try {
              const resolution = validateGuardrailDisposition({
                disposition: disp.disposition,
                exactOutput: revisedOutput,
                originalJev: disp.originalJev,
                diagnostic: disp.diagnostic,
                revision: disp.revision,
                rawEvidenceTexts: disp.rawEvidenceTexts,
              });
              if (resolution.sectionOutputHash !== revisedOutputHash) {
                blockers.push(`${prefix}:editorial_guardrail_disposition_section_hash_mismatch`);
              } else if (resolution.sectionId !== sectionId) {
                blockers.push(`${prefix}:editorial_guardrail_disposition_section_id_mismatch`);
              } else {
                guardrailResolution = resolution;
                notes.push(
                  `${prefix}:editorial_guardrail_reviewed_no_policy_violation_found`,
                );
              }
            } catch (err) {
              const code =
                err instanceof GuardrailDispositionError ? err.code : 'disposition_failed';
              blockers.push(`${prefix}:editorial_guardrail_disposition_invalid:${code}`);
            }
          }
        }
      }
    }
  }

  return { sectionId, revisedOutputHash, blockers, notes, guardrailResolution };
}

/**
 * Verify that the original receipt's model-generation prior-context edges
 * point at either (a) the ORIGINAL historical prior outputs, or
 * (b) explicitly-declared VERIFIED-revised predecessor texts that the
 * generator legitimately consumed before this section was itself later
 * revised.  Item 4 doctrine:
 *
 *   * Silence never admits a revised predecessor.  A missing edge
 *     declaration is interpreted as `kind: 'original'`.
 *   * Every edge — original or revised — must round-trip:
 *       sha256(edge.text) === receipt.prior_section_content_hashes[earlierId]
 *       AND receipt.synthesis_input_text.includes(edge.text)
 *   * A revised-kind edge additionally requires the earlier section to
 *     appear in `revisedSections` and the declared text to equal the
 *     verified current or historical revised output of that earlier section.
 *     Historical outputs require full editorial evidence revalidation; it never forges a
 *     later edit into an older generation because the receipt's own
 *     hash + inclusion are hard binds.
 *   * Any OTHER revised text (not declared on any edge for this section)
 *     that leaks into `synthesis_input_text` is a forged rerun.
 */
export function verifyEditorialPriorContext(args: {
  sectionId: string;
  originalReceipt: SectionExecutionReceipt;
  requiredSectionIds: string[];
  originalPasses: Readonly<Record<string, string>>;
  revisedOutputsByOtherSections: Readonly<Record<string, string>>;
  /** Only outputs that passed full editorial verification may enter this map. */
  verifiedHistoricalOutputs?: Readonly<Record<string, readonly string[]>>;
  priorContext?: Readonly<Record<string, PriorContextEdge>>;
  revisedSectionIds?: readonly string[];
  /** Titles used by the real orchestrator to frame and hash prior chapters. */
  sectionTitles?: Readonly<Record<string, string>>;
}): string[] {
  const {
    sectionId,
    originalReceipt,
    requiredSectionIds,
    originalPasses,
    revisedOutputsByOtherSections,
    priorContext,
    revisedSectionIds,
    sectionTitles,
  } = args;
  const blockers: string[] = [];
  const prefix = sectionId;
  const idx = requiredSectionIds.indexOf(sectionId);
  const earlierIds = idx >= 0 ? requiredSectionIds.slice(0, idx) : [];
  const priorHashes = originalReceipt.prior_section_content_hashes ?? {};
  const rawInput = originalReceipt.synthesis_input_text ?? '';
  const revisedIdSet = new Set(revisedSectionIds ?? []);

  for (const earlierId of earlierIds) {
    const declared = priorContext?.[earlierId];
    const originalText = originalPasses[earlierId];
    // Effective edge: declared, or implicit-original when omitted.
    let edge: PriorContextEdge | null = null;
    if (priorContext && Object.hasOwn(priorContext, earlierId)) {
      if (
        !declared ||
        typeof declared !== 'object' ||
        (declared.kind !== 'original' && declared.kind !== 'revised') ||
        typeof declared.text !== 'string' ||
        declared.text.length === 0
      ) {
        blockers.push(`${prefix}:editorial_prior_context_edge_malformed:${earlierId}`);
        continue;
      }
      edge = declared;
    } else {
      if (typeof originalText !== 'string' || originalText.length === 0) {
        blockers.push(`${prefix}:editorial_original_prior_missing:${earlierId}`);
        continue;
      }
      edge = { kind: 'original', text: originalText };
    }

    // Kind-specific text binding.
    if (edge.kind === 'original') {
      if (typeof originalText !== 'string' || originalText.length === 0) {
        blockers.push(`${prefix}:editorial_original_prior_missing:${earlierId}`);
        continue;
      }
      if (edge.text !== originalText) {
        blockers.push(`${prefix}:editorial_prior_context_edge_original_text_mismatch:${earlierId}`);
      }
    } else {
      // revised
      if (!revisedIdSet.has(earlierId)) {
        blockers.push(`${prefix}:editorial_prior_context_edge_revised_not_verified:${earlierId}`);
        continue;
      }
      const verifiedRevised = revisedOutputsByOtherSections[earlierId];
      if (typeof verifiedRevised !== 'string' || verifiedRevised.length === 0) {
        blockers.push(`${prefix}:editorial_prior_context_edge_revised_missing:${earlierId}`);
        continue;
      }
      if (edge.text !== verifiedRevised && !args.verifiedHistoricalOutputs?.[earlierId]?.includes(edge.text)) {
        blockers.push(`${prefix}:editorial_prior_context_edge_revised_text_mismatch:${earlierId}`);
      }
    }

    // Hash binding: the receipt's stored edge hash MUST equal sha256(text).
    // Real execution hashes `## Title\n\noutput` under the chapter title.
    // Older synthetic/legacy receipts use section IDs and bare output.
    const title = sectionTitles?.[earlierId];
    const usesTitle = !Object.hasOwn(priorHashes, earlierId) && typeof title === 'string' && title.length > 0;
    const framedText = usesTitle ? `## ${title}\n\n${edge.text}` : edge.text;
    const expectedHash = sha256(framedText);
    const storedHash = priorHashes[usesTitle ? title! : earlierId];
    if (storedHash === undefined) {
      blockers.push(`${prefix}:editorial_prior_context_edge_missing:${earlierId}`);
    } else if (storedHash !== expectedHash) {
      const suffix = edge.kind === 'revised'
        ? 'editorial_prior_context_edge_revised_hash_mismatch'
        : 'editorial_prior_context_edge_original_hash_mismatch';
      blockers.push(`${prefix}:${suffix}:${earlierId}`);
    }

    // Raw-input inclusion binding: the exact text MUST appear in the
    // synthesis input the generator actually saw.
    if (!rawInput.includes(framedText)) {
      blockers.push(`${prefix}:editorial_prior_context_edge_raw_input_missing:${earlierId}`);
    }
  }

  // Forgery guard: no OTHER revised text may appear in this receipt's raw
  // synthesis input unless it was declared on an edge for this section.
  const declaredRevisedTexts = new Set<string>();
  if (priorContext) {
    for (const [otherId, edge] of Object.entries(priorContext)) {
      if (edge && edge.kind === 'revised' && typeof edge.text === 'string' && otherId !== sectionId) {
        declaredRevisedTexts.add(edge.text);
      }
    }
  }
  for (const [otherId, revisedText] of Object.entries(revisedOutputsByOtherSections)) {
    if (otherId === sectionId) continue;
    if (typeof revisedText !== 'string' || revisedText.length === 0) continue;
    if (declaredRevisedTexts.has(revisedText)) continue;
    if (rawInput.includes(revisedText)) {
      blockers.push(`${prefix}:editorial_original_raw_input_forged_with_revised:${otherId}`);
    }
  }
  return blockers;
}

/** Item 5 doctrine — a rationale under this length is not substantive. */
const COHERENCE_RATIONALE_MIN_LEN = 32;

/**
 * Verify the whole-document editorial coherence review.  Missing / stale
 * review blocks the report even if every per-section check passes.
 *
 * Item 5 hardening (2026-09-30):
 *   * `artifactSha256` MUST bind the actual assembled artifact text when
 *     the caller can supply it.  The verifier calls this helper with
 *     `assembledText` (the real assembled bytes) whenever the report
 *     pipeline has one; the review's `artifactSha256` must equal
 *     `sha256(assembledText)`.  The hash-list fallback is only ever a
 *     placeholder for pipelines that have no assembled text at all — it
 *     never silently substitutes for a missing assembly.
 *   * `rationale` must be substantive (non-trivial length).
 *   * `findings` must be a non-null array of well-formed objects with an
 *     explicit `boolean` `unresolved` flag; any missing/malformed entry
 *     blocks (silence never admits vacuous findings).
 *   * `reviewedRevisions` must not contain duplicate section IDs.
 */
export function verifyEditorialCoherenceReview(args: {
  review: EditorialCoherenceReview | undefined;
  requiredSectionIds: string[];
  effectiveHashesInOrder: string[];
  revisedSectionIds: string[];
  revisedHashesById: Readonly<Record<string, string>>;
  /** Fallback hash used only when the pipeline has no assembled text. */
  expectedArtifactSha256: string;
  /**
   * Exact assembled artifact text, when the pipeline produced one.  If
   * provided, the review MUST bind its `artifactSha256` to
   * `sha256(assembledText)` — the hash-list fallback never stands in.
   */
  assembledText?: string;
}): string[] {
  const { review, requiredSectionIds, effectiveHashesInOrder, revisedSectionIds, revisedHashesById, expectedArtifactSha256, assembledText } = args;
  const blockers: string[] = [];
  if (!review) {
    blockers.push('editorial:coherence_review_missing');
    return blockers;
  }
  if (review.status !== 'reviewed') {
    blockers.push(`editorial:coherence_review_status:${String(review.status)}`);
  }
  if (typeof review.reviewer !== 'string' || (review.reviewer !== 'assistant-source-review' && review.reviewer !== 'human')) {
    blockers.push('editorial:coherence_review_reviewer_invalid');
  }
  if (typeof review.rationale !== 'string' || review.rationale.trim().length === 0) {
    blockers.push('editorial:coherence_review_rationale_empty');
  } else if (review.rationale.trim().length < COHERENCE_RATIONALE_MIN_LEN) {
    blockers.push('editorial:coherence_review_rationale_not_substantive');
  }
  if (typeof review.reviewedAt !== 'string' || review.reviewedAt.trim().length === 0) {
    blockers.push('editorial:coherence_review_timestamp_empty');
  }
  if (
    !Array.isArray(review.orderedEffectiveHashes) ||
    review.orderedEffectiveHashes.length !== effectiveHashesInOrder.length ||
    review.orderedEffectiveHashes.some((h, i) => h !== effectiveHashesInOrder[i])
  ) {
    blockers.push('editorial:coherence_review_ordered_hashes_mismatch');
  }
  // Item 5 — bind to actual assembled bytes when available, never silently
  // fall back to a hash-list string.
  if (typeof assembledText === 'string') {
    const assembledHash = sha256(assembledText);
    if (review.artifactSha256 !== assembledHash) {
      blockers.push('editorial:coherence_review_artifact_hash_not_bound_to_assembled');
    }
  } else if (review.artifactSha256 !== expectedArtifactSha256) {
    blockers.push('editorial:coherence_review_artifact_hash_stale');
  }
  const reviewedRevisions = Array.isArray(review.reviewedRevisions) ? review.reviewedRevisions : [];
  const reviewedIds = new Set(reviewedRevisions.map((r) => r?.sectionId));
  for (const id of revisedSectionIds) {
    if (!reviewedIds.has(id)) {
      blockers.push(`editorial:coherence_review_missing_revised_section:${id}`);
    } else {
      const ref = reviewedRevisions.find((r) => r?.sectionId === id);
      if (!ref || ref.revisedOutputHash !== revisedHashesById[id]) {
        blockers.push(`editorial:coherence_review_stale_revised_hash:${id}`);
      }
    }
  }
  // Duplicate reviewedRevisions section IDs — reject.
  const seenReviewedIds = new Set<string>();
  for (const ref of reviewedRevisions) {
    const rid = ref?.sectionId;
    if (typeof rid === 'string' && rid.length > 0) {
      if (seenReviewedIds.has(rid)) {
        blockers.push(`editorial:coherence_review_duplicate_reviewed_revision:${rid}`);
      } else {
        seenReviewedIds.add(rid);
      }
    }
  }
  for (const ref of reviewedRevisions) {
    if (!ref || typeof ref.sectionId !== 'string' || !revisedSectionIds.includes(ref.sectionId)) {
      blockers.push(`editorial:coherence_review_extra_revised_reference:${ref?.sectionId ?? 'unknown'}`);
    }
  }
  // Section coverage must match requiredSectionIds order — already enforced via effectiveHashesInOrder.
  // Findings must be well-formed.  Missing key is treated as [] but a
  // non-array present value blocks; each entry must be an object with
  // string id/summary and an explicit boolean `unresolved`.
  const findingsRaw = (review as { findings?: unknown }).findings;
  let findings: CoherenceReviewFinding[] = [];
  if (findingsRaw === undefined) {
    findings = [];
  } else if (!Array.isArray(findingsRaw)) {
    blockers.push('editorial:coherence_review_findings_not_array');
  } else {
    for (let i = 0; i < findingsRaw.length; i++) {
      const f = findingsRaw[i] as unknown;
      if (
        !f ||
        typeof f !== 'object' ||
        typeof (f as { id?: unknown }).id !== 'string' ||
        typeof (f as { summary?: unknown }).summary !== 'string' ||
        typeof (f as { unresolved?: unknown }).unresolved !== 'boolean'
      ) {
        blockers.push(`editorial:coherence_review_finding_malformed:${i}`);
      } else {
        findings.push(f as CoherenceReviewFinding);
      }
    }
  }
  const unresolved = findings.filter((f) => f && f.unresolved === true);
  if (unresolved.length > 0) {
    blockers.push(
      `editorial:coherence_review_unresolved_findings:${unresolved.map((f) => f.id).join(',')}`,
    );
  }
  if (requiredSectionIds.length === 0) {
    blockers.push('editorial:coherence_review_no_sections');
  }
  return blockers;
}

/** Recompute the repaired draft's word fit; never trust only a pass label. */
export function hasRecoveredWordFit(text: string, rubric: Record<string, unknown> | undefined): boolean {
  const target = rubric?.target_words;
  const actual = countReportWords(text);
  return typeof target === 'number' && Number.isFinite(target) && target > 0 &&
    rubric?.word_count_fit === 'pass' && rubric.actual_words === actual &&
    actual >= target * 0.8 && actual <= target * 1.2;
}
