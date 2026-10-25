// ─── Guardrail Uncertainty Disposition Validator ──────────────────────
// Pure, deterministic validator for an explicit reviewer disposition of a
// guardrail Jev "could-not-tell" verdict. Fails closed on any inconsistency.
//
// Doctrine:
//   * A "could-not-tell" guardrail Jev verdict is NEVER silently relabeled
//     `pass`. An explicit reviewer disposition (with cited full-nonblank-line
//     diagnostic + revision + Jev receipt) can only ever return the resolution
//     label 'reviewed-no-policy-violation-found', while the original Jev
//     verdict `could-not-tell` is preserved verbatim on the artifact.
//   * The validator only *checks* the artifact and its evidence bundle.
//     It performs NO I/O, calls NO LLMs, reads NO clocks, mutates NOTHING,
//     and never fabricates a disposition — the caller supplies the artifact.
//   * Any hash disagreement, missing/mistyped/out-of-range noul answer,
//     extra answer, missing coverage of a nonblank output line, invented
//     unit line, blocked or errored Jev receipt, or fail verdict on any
//     Jev sub-verdict is a hard reject.
//   * The validator does not change thresholds, does not re-run any verifier,
//     and does not touch other pipeline artifacts.
//
// This module is intentionally free of imports from the rest of the pipeline
// so it can be reused as a leaf guard.

import { createHash } from 'node:crypto';
import { isDeepStrictEqual } from 'node:util';

// ─── Types ────────────────────────────────────────────────────────────

/** Who authored the disposition. Same two-value contract as editorial revisions. */
export type DispositionReviewer = 'assistant-source-review' | 'human';

/** The one and only decision this validator accepts. */
export type DispositionDecision = 'reviewed-no-policy-violation-found';

/** The one and only original verdict this validator accepts. */
export type OriginalGuardrailVerdict = 'could-not-tell';

/** One evidence pointer inside the disposition. */
export interface DispositionEvidenceRef {
  /** Filename (basename) of the referenced artifact. Nonempty. */
  file: string;
  /** SHA-256 of the referenced artifact's raw file bytes. Nonempty lowercase hex. */
  sha256: string;
}

/**
 * The explicit reviewer disposition artifact under validation.
 * All fields are required. The caller supplies this verbatim.
 */
export interface GuardrailDispositionArtifact {
  kind: 'explicit-guardrail-review-disposition';
  sectionId: string;
  /** SHA-256 of the *current* (revised) section output the disposition speaks to. */
  sectionOutputHash: string;
  reviewer: DispositionReviewer;
  reviewedAt: string;
  originalVerdict: OriginalGuardrailVerdict;
  decision: DispositionDecision;
  rationale: string;
  evidence: readonly DispositionEvidenceRef[];
  policy: 'descriptive';
  jevVerdictPreserved: true;
  finalAccepted: boolean;
  remainingReview: readonly string[];
}

/** Minimal Jev shape this validator inspects. */
export interface JevReviewArtifact {
  sectionOutputHash: string;
  receipt: {
    status: string;
    blocked: boolean;
    verdicts: Record<string, string>;
    [key: string]: unknown;
  };
  [key: string]: unknown;
}

/** One unit line inside a diagnostic batch. */
export interface DiagnosticUnit {
  /** Zero-based line index into the split-by-'\n' revision output. */
  index: number;
  /** Exact nonblank line text at `index`. */
  text: string;
}

/** One noul-typed question row inside a diagnostic batch. */
export interface DiagnosticQuestion {
  type: 'noul';
  instructions: string;
  criteria: { true: string; false: string };
}

/** One noul-typed answer row inside a diagnostic batch. */
export interface DiagnosticAnswer {
  type: 'noul';
  noul: number;
}

/** One batch of the full-nonblank-line diagnostic. */
export interface DiagnosticBatch {
  units: readonly DiagnosticUnit[];
  questions: Record<string, DiagnosticQuestion>;
  result: {
    model?: string;
    answers: Record<string, DiagnosticAnswer>;
    [key: string]: unknown;
  };
  [key: string]: unknown;
}

/** The full-nonblank-line guardrail diagnostic artifact. */
export interface GuardrailDiagnosticArtifact {
  kind: 'full-nonblank-line-guardrail-diagnostic';
  section: string;
  sectionOutputHash: string;
  policy: string;
  unitCount: number;
  complete: boolean;
  missing: readonly unknown[];
  flags: readonly unknown[];
  batches: readonly DiagnosticBatch[];
  [key: string]: unknown;
}

/** Minimal revision shape this validator inspects. */
export interface RevisionArtifact {
  sectionId?: string;
  revisedOutput: string;
  revisedOutputHash: string;
  [key: string]: unknown;
}

/**
 * The single input to {@link validateGuardrailDisposition}. `rawEvidenceTexts`
 * keys are filenames (matching the `evidence[].file` fields) and values are the
 * exact raw file text (UTF-8) that the caller loaded from disk.
 */
export interface ValidateGuardrailDispositionInput {
  disposition: GuardrailDispositionArtifact;
  /** Exact current section output text (post-revision) the disposition rules on. */
  exactOutput: string;
  originalJev: JevReviewArtifact;
  diagnostic: GuardrailDiagnosticArtifact;
  revision: RevisionArtifact;
  /**
   * Raw UTF-8 text of every file listed in `disposition.evidence[].file`, keyed
   * by that same filename. Extra keys are ignored; missing keys are a hard
   * reject.
   */
  rawEvidenceTexts: Readonly<Record<string, string>>;
}

/** The validated, frozen resolution the caller may propagate. */
export interface GuardrailDispositionResolution {
  sectionId: string;
  originalVerdict: OriginalGuardrailVerdict;
  /** Explicit resolution label. Never 'pass'. */
  resolution: 'reviewed-no-policy-violation-found';
  reviewer: DispositionReviewer;
  reviewedAt: string;
  sectionOutputHash: string;
  policy: 'descriptive';
  jevVerdictPreserved: true;
  finalAccepted: boolean;
  evidence: readonly DispositionEvidenceRef[];
}

// ─── Error type ───────────────────────────────────────────────────────

export type GuardrailDispositionErrorCode =
  | 'invalid_disposition'
  | 'invalid_kind'
  | 'invalid_section_id'
  | 'invalid_reviewer'
  | 'invalid_reviewed_at'
  | 'invalid_original_verdict'
  | 'invalid_decision'
  | 'empty_rationale'
  | 'invalid_policy'
  | 'jev_verdict_not_preserved'
  | 'invalid_evidence'
  | 'missing_evidence_text'
  | 'evidence_hash_mismatch'
  | 'section_output_hash_mismatch'
  | 'revision_hash_mismatch'
  | 'jev_hash_mismatch'
  | 'jev_blocked_or_errored'
  | 'jev_fail_verdict'
  | 'jev_original_verdict_mismatch'
  | 'diagnostic_incomplete'
  | 'diagnostic_hash_mismatch'
  | 'diagnostic_policy_mismatch'
  | 'diagnostic_unit_count_mismatch'
  | 'diagnostic_coverage_gap'
  | 'diagnostic_duplicate_unit'
  | 'diagnostic_invented_unit'
  | 'diagnostic_unit_text_mismatch'
  | 'diagnostic_question_type'
  | 'diagnostic_missing_answer'
  | 'diagnostic_extra_answer'
  | 'diagnostic_answer_shape'
  | 'diagnostic_answer_out_of_range'
  | 'diagnostic_flag_present'
  | 'missing_evidence_ref';

export class GuardrailDispositionError extends Error {
  readonly code: GuardrailDispositionErrorCode;
  readonly detail?: string;
  constructor(code: GuardrailDispositionErrorCode, message: string, detail?: string) {
    super(message);
    this.name = 'GuardrailDispositionError';
    this.code = code;
    this.detail = detail;
  }
}

// ─── Internal helpers ─────────────────────────────────────────────────

function sha256Hex(input: string): string {
  return createHash('sha256').update(input, 'utf8').digest('hex');
}

function sha256HexBytes(input: string): string {
  // Evidence files were hashed as raw bytes on disk. Vitest fixtures pass
  // UTF-8 text; Node's createHash treats a string arg as UTF-8, which matches
  // shasum output for UTF-8 files. This helper is spelled separately so the
  // intent (bytes of the file) is visible at call sites.
  return createHash('sha256').update(input, 'utf8').digest('hex');
}

function isNonemptyTrimmed(value: unknown): value is string {
  return typeof value === 'string' && value.trim().length > 0;
}

function isLowerHex64(value: unknown): value is string {
  return typeof value === 'string' && /^[0-9a-f]{64}$/.test(value);
}

function reject(code: GuardrailDispositionErrorCode, message: string, detail?: string): never {
  throw new GuardrailDispositionError(code, message, detail);
}

// ─── Public API ───────────────────────────────────────────────────────

/**
 * Validate an explicit guardrail uncertainty disposition against its cited
 * Jev review, full-nonblank-line diagnostic, revision artifact, and raw
 * evidence file texts. Returns a frozen resolution record on success. Throws
 * {@link GuardrailDispositionError} on any inconsistency.
 *
 * The returned resolution preserves the original `could-not-tell` verdict and
 * exposes the explicit label `reviewed-no-policy-violation-found`. It is
 * never `pass`.
 */
export function validateGuardrailDisposition(
  input: ValidateGuardrailDispositionInput,
): GuardrailDispositionResolution {
  const { disposition, exactOutput, originalJev, diagnostic, revision, rawEvidenceTexts } = input;

  // ── disposition shape ────────────────────────────────────────────
  if (!disposition || typeof disposition !== 'object') {
    reject('invalid_disposition', 'disposition artifact is required');
  }
  if (disposition.kind !== 'explicit-guardrail-review-disposition') {
    reject('invalid_kind', `disposition.kind must be "explicit-guardrail-review-disposition"`);
  }
  if (!isNonemptyTrimmed(disposition.sectionId)) {
    reject('invalid_section_id', 'disposition.sectionId must be nonempty');
  }
  if (!isLowerHex64(disposition.sectionOutputHash)) {
    reject('section_output_hash_mismatch', 'disposition.sectionOutputHash must be a 64-hex sha256');
  }
  if (disposition.reviewer !== 'assistant-source-review' && disposition.reviewer !== 'human') {
    reject('invalid_reviewer', 'disposition.reviewer must be "assistant-source-review" or "human"');
  }
  if (!isNonemptyTrimmed(disposition.reviewedAt)) {
    reject('invalid_reviewed_at', 'disposition.reviewedAt must be a nonempty string');
  }
  if (disposition.originalVerdict !== 'could-not-tell') {
    reject(
      'invalid_original_verdict',
      'disposition.originalVerdict must be "could-not-tell" (this validator only dispositions uncertainty)',
    );
  }
  if (disposition.decision !== 'reviewed-no-policy-violation-found') {
    reject(
      'invalid_decision',
      'disposition.decision must be exactly "reviewed-no-policy-violation-found" — never "pass"',
    );
  }
  if (!isNonemptyTrimmed(disposition.rationale)) {
    reject('empty_rationale', 'disposition.rationale must be a nonempty string');
  }
  if (disposition.policy !== 'descriptive') {
    reject('invalid_policy', 'disposition.policy must be "descriptive"');
  }
  if (disposition.jevVerdictPreserved !== true) {
    reject(
      'jev_verdict_not_preserved',
      'disposition.jevVerdictPreserved must be literal true — the original Jev verdict is never rewritten',
    );
  }

  // ── exactOutput binds sectionOutputHash ─────────────────────────
  const exactOutputHash = sha256Hex(exactOutput);
  if (exactOutputHash !== disposition.sectionOutputHash) {
    reject(
      'section_output_hash_mismatch',
      'sha256(exactOutput) does not match disposition.sectionOutputHash',
      `expected=${disposition.sectionOutputHash} actual=${exactOutputHash}`,
    );
  }

  // ── revision binds the same output ──────────────────────────────
  if (!revision || typeof revision !== 'object') {
    reject('revision_hash_mismatch', 'revision artifact is required');
  }
  if (typeof revision.revisedOutput !== 'string') {
    reject('revision_hash_mismatch', 'revision.revisedOutput must be a string');
  }
  if (!isLowerHex64(revision.revisedOutputHash)) {
    reject('revision_hash_mismatch', 'revision.revisedOutputHash must be a 64-hex sha256');
  }
  const revisedOutputHashActual = sha256Hex(revision.revisedOutput);
  if (revisedOutputHashActual !== revision.revisedOutputHash) {
    reject(
      'revision_hash_mismatch',
      'sha256(revision.revisedOutput) does not match revision.revisedOutputHash',
      `expected=${revision.revisedOutputHash} actual=${revisedOutputHashActual}`,
    );
  }
  if (revision.revisedOutput !== exactOutput) {
    reject('revision_hash_mismatch', 'revision.revisedOutput must equal exactOutput byte-for-byte');
  }
  if (revision.revisedOutputHash !== disposition.sectionOutputHash) {
    reject(
      'revision_hash_mismatch',
      'revision.revisedOutputHash does not bind the disposition sectionOutputHash',
    );
  }

  // ── Jev binding + verdict discipline ────────────────────────────
  if (!originalJev || typeof originalJev !== 'object') {
    reject('jev_hash_mismatch', 'originalJev artifact is required');
  }
  if (originalJev.sectionOutputHash !== disposition.sectionOutputHash) {
    reject(
      'jev_hash_mismatch',
      'originalJev.sectionOutputHash does not bind the disposition sectionOutputHash',
      `jev=${originalJev.sectionOutputHash} disposition=${disposition.sectionOutputHash}`,
    );
  }
  const jevReceipt = originalJev.receipt;
  if (!jevReceipt || typeof jevReceipt !== 'object') {
    reject('jev_hash_mismatch', 'originalJev.receipt is required');
  }
  if (jevReceipt.blocked !== false) {
    reject('jev_blocked_or_errored', 'originalJev.receipt.blocked must be false');
  }
  if (jevReceipt.status !== 'judged') {
    reject('jev_blocked_or_errored', 'originalJev.receipt.status must be "judged"');
  }
  const verdicts = jevReceipt.verdicts;
  if (!verdicts || typeof verdicts !== 'object') {
    reject('jev_fail_verdict', 'originalJev.receipt.verdicts is required');
  }
  for (const [name, verdict] of Object.entries(verdicts)) {
    if (verdict === 'fail') {
      reject(
        'jev_fail_verdict',
        `originalJev.receipt.verdicts.${name} is "fail" — dispositions cannot rescue a failed verdict`,
      );
    }
  }
  if (verdicts.guardrail !== 'could-not-tell') {
    reject(
      'jev_original_verdict_mismatch',
      `originalJev.receipt.verdicts.guardrail must be "could-not-tell"; got "${String(verdicts.guardrail)}"`,
    );
  }
  for (const dimension of ['framing', 'grounding', 'register']) {
    if (verdicts[dimension] !== 'pass') reject('jev_fail_verdict', `guardrail disposition cannot resolve missing or uncertain ${dimension}`);
  }

  // ── evidence: shape, presence, hash ─────────────────────────────
  if (!Array.isArray(disposition.evidence) || disposition.evidence.length === 0) {
    reject('invalid_evidence', 'disposition.evidence must be a nonempty array');
  }
  const seenFiles = new Set<string>();
  for (let i = 0; i < disposition.evidence.length; i += 1) {
    const ref = disposition.evidence[i];
    if (!ref || typeof ref !== 'object') {
      reject('invalid_evidence', `disposition.evidence[${i}] must be an object`);
    }
    if (!isNonemptyTrimmed(ref.file)) {
      reject('invalid_evidence', `disposition.evidence[${i}].file must be nonempty`);
    }
    if (!isLowerHex64(ref.sha256)) {
      reject('invalid_evidence', `disposition.evidence[${i}].sha256 must be a 64-hex sha256`);
    }
    if (seenFiles.has(ref.file)) {
      reject('invalid_evidence', `disposition.evidence[${i}].file "${ref.file}" is duplicated`);
    }
    seenFiles.add(ref.file);
    const rawText = rawEvidenceTexts?.[ref.file];
    if (typeof rawText !== 'string') {
      reject(
        'missing_evidence_text',
        `rawEvidenceTexts is missing an entry for "${ref.file}"`,
      );
    }
    const actual = sha256HexBytes(rawText);
    if (actual !== ref.sha256) {
      reject(
        'evidence_hash_mismatch',
        `evidence "${ref.file}" sha256 mismatch`,
        `expected=${ref.sha256} actual=${actual}`,
      );
    }
  }

  // ── evidence must at minimum cover jev + diagnostic + revision ──
  const evidenceFiles = new Set(disposition.evidence.map((e) => e.file));
  const jevListed = disposition.evidence.some((e) => /editorial-jev.*\.json$/.test(e.file));
  const diagListed = disposition.evidence.some((e) =>
    /guardrail-diagnostic.*\.json$/.test(e.file),
  );
  const revisionListed = disposition.evidence.some((e) =>
    /editorial-revision(?:-v\d+)?\.json$/.test(e.file),
  );
  if (!jevListed) {
    reject('missing_evidence_ref', 'disposition.evidence must list the editorial-jev artifact');
  }
  if (!diagListed) {
    reject(
      'missing_evidence_ref',
      'disposition.evidence must list the guardrail-diagnostic artifact',
    );
  }
  if (!revisionListed) {
    reject('missing_evidence_ref', 'disposition.evidence must list the editorial-revision artifact');
  }
  void evidenceFiles;

  // ── diagnostic: shape, section, hash, policy, completeness ─────
  if (!diagnostic || typeof diagnostic !== 'object') {
    reject('diagnostic_incomplete', 'diagnostic artifact is required');
  }
  if (diagnostic.kind !== 'full-nonblank-line-guardrail-diagnostic') {
    reject('diagnostic_incomplete', 'diagnostic.kind must be "full-nonblank-line-guardrail-diagnostic"');
  }
  if (diagnostic.section !== disposition.sectionId) {
    reject(
      'diagnostic_incomplete',
      `diagnostic.section "${diagnostic.section}" does not match disposition.sectionId "${disposition.sectionId}"`,
    );
  }
  if (diagnostic.sectionOutputHash !== disposition.sectionOutputHash) {
    reject(
      'diagnostic_hash_mismatch',
      'diagnostic.sectionOutputHash does not bind the disposition sectionOutputHash',
    );
  }
  if (diagnostic.policy !== disposition.policy) {
    reject(
      'diagnostic_policy_mismatch',
      `diagnostic.policy "${diagnostic.policy}" does not match disposition.policy "${disposition.policy}"`,
    );
  }
  if (diagnostic.complete !== true) {
    reject('diagnostic_incomplete', 'diagnostic.complete must be literal true');
  }
  if (!Array.isArray(diagnostic.missing) || diagnostic.missing.length !== 0) {
    reject('diagnostic_incomplete', 'diagnostic.missing must be an empty array');
  }
  if (!Array.isArray(diagnostic.flags) || diagnostic.flags.length !== 0) {
    reject('diagnostic_flag_present', 'diagnostic.flags must be an empty array');
  }
  if (!Array.isArray(diagnostic.batches) || diagnostic.batches.length === 0) {
    reject('diagnostic_incomplete', 'diagnostic.batches must be a nonempty array');
  }

  // ── coverage: every nonblank line indexed exactly once ─────────
  const outputLines = exactOutput.split('\n');
  const nonblankIndexes: number[] = [];
  for (let i = 0; i < outputLines.length; i += 1) {
    if (outputLines[i].trim().length > 0) nonblankIndexes.push(i);
  }
  const expectedIndexSet = new Set(nonblankIndexes);

  if (typeof diagnostic.unitCount !== 'number' || diagnostic.unitCount !== nonblankIndexes.length) {
    reject(
      'diagnostic_unit_count_mismatch',
      `diagnostic.unitCount ${diagnostic.unitCount} does not equal nonblank line count ${nonblankIndexes.length}`,
    );
  }

  const seenUnitIndexes = new Set<number>();
  let totalUnits = 0;
  for (let b = 0; b < diagnostic.batches.length; b += 1) {
    const batch = diagnostic.batches[b];
    if (!batch || typeof batch !== 'object') {
      reject('diagnostic_incomplete', `diagnostic.batches[${b}] must be an object`);
    }
    if (!Array.isArray(batch.units) || batch.units.length === 0) {
      reject('diagnostic_incomplete', `diagnostic.batches[${b}].units must be a nonempty array`);
    }
    if (!batch.questions || typeof batch.questions !== 'object') {
      reject('diagnostic_incomplete', `diagnostic.batches[${b}].questions must be an object`);
    }
    if (!batch.result || typeof batch.result !== 'object' || !batch.result.answers) {
      reject('diagnostic_incomplete', `diagnostic.batches[${b}].result.answers must be present`);
    }

    const questionKeys = Object.keys(batch.questions);
    const answerKeys = Object.keys(batch.result.answers);

    // Every unit must map to a nonblank line and have matching text.
    const batchUnitKeys: string[] = [];
    for (let u = 0; u < batch.units.length; u += 1) {
      const unit = batch.units[u];
      if (!unit || typeof unit !== 'object') {
        reject('diagnostic_incomplete', `diagnostic.batches[${b}].units[${u}] must be an object`);
      }
      if (typeof unit.index !== 'number' || !Number.isInteger(unit.index) || unit.index < 0) {
        reject(
          'diagnostic_incomplete',
          `diagnostic.batches[${b}].units[${u}].index must be a nonnegative integer`,
        );
      }
      if (typeof unit.text !== 'string') {
        reject(
          'diagnostic_incomplete',
          `diagnostic.batches[${b}].units[${u}].text must be a string`,
        );
      }
      if (!expectedIndexSet.has(unit.index)) {
        reject(
          'diagnostic_invented_unit',
          `diagnostic.batches[${b}].units[${u}].index ${unit.index} does not point at a nonblank line of exactOutput`,
        );
      }
      if (outputLines[unit.index] !== unit.text) {
        reject(
          'diagnostic_unit_text_mismatch',
          `diagnostic.batches[${b}].units[${u}].text does not match exactOutput line ${unit.index}`,
        );
      }
      if (seenUnitIndexes.has(unit.index)) {
        reject(
          'diagnostic_duplicate_unit',
          `diagnostic unit for line index ${unit.index} appears more than once`,
        );
      }
      seenUnitIndexes.add(unit.index);
      batchUnitKeys.push(`u${unit.index}`);
      totalUnits += 1;
    }

    // Questions must match unit keys exactly, be noul-typed.
    const batchUnitKeySet = new Set(batchUnitKeys);
    if (
      questionKeys.length !== batchUnitKeys.length ||
      !batchUnitKeys.every((k) => batchUnitKeySet.has(k) && k in batch.questions)
    ) {
      reject(
        'diagnostic_incomplete',
        `diagnostic.batches[${b}].questions keys must equal the batch unit keys`,
      );
    }
    for (const qk of questionKeys) {
      if (!batchUnitKeySet.has(qk)) {
        reject(
          'diagnostic_incomplete',
          `diagnostic.batches[${b}].questions has stray key "${qk}"`,
        );
      }
      const q = batch.questions[qk];
      if (!q || q.type !== 'noul') {
        reject(
          'diagnostic_question_type',
          `diagnostic.batches[${b}].questions.${qk}.type must be "noul"`,
        );
      }
    }

    // Answers: every question must be answered; no extra answers.
    for (const qk of batchUnitKeys) {
      if (!(qk in batch.result.answers)) {
        reject(
          'diagnostic_missing_answer',
          `diagnostic.batches[${b}].result.answers is missing "${qk}"`,
        );
      }
    }
    for (const ak of answerKeys) {
      if (!batchUnitKeySet.has(ak)) {
        reject(
          'diagnostic_extra_answer',
          `diagnostic.batches[${b}].result.answers has extra key "${ak}"`,
        );
      }
      const ans = batch.result.answers[ak];
      if (!ans || typeof ans !== 'object' || ans.type !== 'noul') {
        reject(
          'diagnostic_answer_shape',
          `diagnostic.batches[${b}].result.answers.${ak}.type must be "noul"`,
        );
      }
      const noul = ans.noul;
      if (typeof noul !== 'number' || !Number.isFinite(noul)) {
        reject(
          'diagnostic_answer_shape',
          `diagnostic.batches[${b}].result.answers.${ak}.noul must be a finite number`,
        );
      }
      if (noul < 0 || noul > 1) {
        reject(
          'diagnostic_answer_out_of_range',
          `diagnostic.batches[${b}].result.answers.${ak}.noul ${noul} is outside [0,1]`,
        );
      }
      if (!(noul < 0.5)) {
        reject(
          'diagnostic_answer_out_of_range',
          `diagnostic.batches[${b}].result.answers.${ak}.noul ${noul} is not strictly < 0.5 (would trip the 0.5 flag threshold)`,
        );
      }
    }
  }

  if (totalUnits !== nonblankIndexes.length) {
    reject(
      'diagnostic_coverage_gap',
      `diagnostic covered ${totalUnits} units but exactOutput has ${nonblankIndexes.length} nonblank lines`,
    );
  }
  for (const idx of nonblankIndexes) {
    if (!seenUnitIndexes.has(idx)) {
      reject('diagnostic_coverage_gap', `no diagnostic unit covers nonblank line index ${idx}`);
    }
  }

  // Bind the inspected objects to the hashed files, not merely similarly
  // named evidence. Verify the exact diagnostic policy and request state.
  for (const batch of diagnostic.batches) {
    for (const unit of batch.units) {
      const question = batch.questions[`u${unit.index}`];
      const expected = `Is it true that report unit ${unit.index} is a prediction, forecast, guarantee, or promised outcome rather than a description of an observable pattern?`;
      if (question.instructions !== expected || question.criteria?.true !== 'predictive or promissory' || question.criteria?.false !== 'descriptive witness') {
        reject('diagnostic_question_type', 'diagnostic question does not match the descriptive guardrail policy');
      }
    }
    if (batch.state_sha256 !== sha256Hex(JSON.stringify({ section: exactOutput, units: batch.units }))) {
      reject('diagnostic_hash_mismatch', 'diagnostic request state hash mismatch');
    }
  }
  const boundArtifacts: Array<[RegExp, unknown]> = [
    [/editorial-jev.*\.json$/, originalJev],
    [/guardrail-diagnostic.*\.json$/, diagnostic],
    [/editorial-revision(?:-v\d+)?\.json$/, revision],
  ];
  for (const [pattern, artifact] of boundArtifacts) {
    const refs = disposition.evidence.filter(ref => pattern.test(ref.file));
    if (refs.length !== 1) reject('invalid_evidence', 'each evidence role requires exactly one file');
    let parsed: unknown;
    try { parsed = JSON.parse(rawEvidenceTexts[refs[0].file]); }
    catch { reject('invalid_evidence', 'evidence file must contain JSON'); }
    if (!isDeepStrictEqual(parsed, artifact)) reject('evidence_hash_mismatch', 'inspected artifact differs from its hashed evidence file');
  }
  if (disposition.finalAccepted !== false) reject('invalid_disposition', 'a guardrail disposition cannot declare final acceptance');

  // ── build frozen resolution ─────────────────────────────────────
  const frozenEvidence: readonly DispositionEvidenceRef[] = Object.freeze(
    disposition.evidence.map((e) =>
      Object.freeze({ file: e.file, sha256: e.sha256 } as DispositionEvidenceRef),
    ),
  );

  const resolution: GuardrailDispositionResolution = Object.freeze({
    sectionId: disposition.sectionId,
    originalVerdict: 'could-not-tell',
    resolution: 'reviewed-no-policy-violation-found',
    reviewer: disposition.reviewer,
    reviewedAt: disposition.reviewedAt,
    sectionOutputHash: disposition.sectionOutputHash,
    policy: 'descriptive',
    jevVerdictPreserved: true,
    finalAccepted: false,
    evidence: frozenEvidence,
  });

  return resolution;
}
