import { validatePrimaryPassageAdditions } from './editorial-passage-evidence.js';
import { validateSourcePartition } from './source-partition.js';
import type { PassResult } from './integrated.js';
import type { DyadSectionReceipt } from './witness-dyad.js';
import type { RetrievalReceipt } from './grounding-adapter.js';
import { DYAD_UNAVAILABLE_MARKER } from './witness-dyad.js';
import fs from 'node:fs';
import { createHash } from 'node:crypto';
import { buildManifestFromIds, validatePassResultsAgainstManifest } from './section-manifest.js';
import type { SourceAuditReceipt } from './reference-source-audit.js';
import { computeAuditInputHash, hasCleanAuditEvidence } from './reference-source-audit.js';
import type { LeakageGateResult } from './leakage-gate.js';

export interface FinalVerificationInput {
  passes: PassResult[];
  pdfPath?: string;
  /**
   * When set, activates the reference-aligned route gates.
   * All required subsection IDs must be present and accepted.
   */
  referenceRoute?: ReferenceRouteOptions;
}

/** Options for the reference-aligned route (Opening + Parts I-XI). */
export interface ReferenceRouteOptions {
  /** Required section IDs in order. */
  requiredSectionIds: string[];
  /** Hashes of actual accepted section text supplied to the synthesis call. */
  synthesisInputHashes?: Record<string, string>;
  /** Per-section dyad receipts (same order as requiredSectionIds). */
  dyadReceipts?: DyadSectionReceipt[];
  /** Per-section retrieval receipts (same order as requiredSectionIds). */
  retrievalReceipts?: RetrievalReceipt[];
  /**
   * Which rubric dimensions are enforced in this mode.
   * Default: all standard gates + dyad + retrieval (reference route).
   */
  requiredRubricGates?: RequiredRubricGates;
  /**
   * When true, the real reference route requires a hash-bound clean source-audit
   * receipt for every accepted section. This is enforced INDEPENDENTLY of Jev.
   */
  requireSourceAudit?: boolean;
  /**
   * Per-section source-audit receipts keyed by section ID.
   * Supplied by the runner alongside pass results.
   */
  sourceAuditReceipts?: Record<string, SourceAuditReceipt>;
  /**
   * Per-section passages_hash (from the retrieval receipt used at generation).
   * Required for recomputing the audit input hash.
   */
  sectionPassagesHash?: Record<string, string>;
  /**
   * Per-section engine facts as fed to the section prompt at generation.
   * Required for recomputing the audit input hash.
   */
  sectionEngineFacts?: Record<string, string>;
  /**
   * Ordered list of primary (reviewed-primary-document) source ids that
   * were appended to the CF retrieval at generation. When present:
   *   * `verifyCloudflareSnapshot` will NOT require these ids in the
   *     CF snapshot (they are not CF passages).
   *   * The route caller is responsible for independently binding these
   *     ids to the primary passage registry bytes (source-audit-side).
   * Absent / empty means "pure CF-only run" — full backwards compatible.
   */
  primaryPassageIds?: readonly string[];
  /**
   * Optional per-section deterministic leakage-gate results (keyed by section id).
   * When supplied for a section, any block-severity violations mark that section
   * as a reference-route blocker. Absent entries are treated as "not evaluated"
   * and do NOT retroactively pass — the rubric's own leakage_gate field is
   * still consulted independently. Absence of the whole map keeps existing
   * routes fully backwards compatible.
   */
  sectionLeakageGates?: Record<string, LeakageGateResult>;
}

export interface RequiredRubricGates {
  word_count?: boolean;
  deterministic_fact?: boolean;
  integrated_layering?: boolean;
  guardrail?: boolean;
  dyad_receipt?: boolean;
  retrieval_receipt?: boolean;
}

const REFERENCE_ROUTE_REQUIRED_GATES: RequiredRubricGates = {
  word_count: true,
  deterministic_fact: true,
  integrated_layering: true,
  guardrail: true,
  dyad_receipt: true,
  // retrieval_receipt: true — grounding is REQUIRED for the reference route.
  // A disabled or missing retrieval receipt is a blocker, not a warning.
  retrieval_receipt: true,
};

/**
 * Canonical full-mode section identifiers in strict order.
 *
 * Full mode acceptance requires the caller's `requiredSectionIds` to
 * EQUAL this list, element-by-element.  Renaming, permuting, or shrinking
 * the caller list — even to a length-12 alternative — is rejected.
 *
 * The list is derived from `packages/witness-pipeline/modes/integrated-kundali-reference.md`
 * (`pass_plan[*].id`).  Do not modify without also updating the mode file
 * and the reference-verifier tests.
 */
export const CANONICAL_FULL_MODE_SECTION_IDS: readonly string[] = Object.freeze([
  'opening',
  'part1', 'part2', 'part3', 'part4', 'part5', 'part6',
  'part7', 'part8', 'part9', 'part10', 'part11',
]);

// A Jev receipt marked `blocked` (active mode, confident safety failure) is a blocker.
// Shadow receipts never block.

export interface FinalVerificationResult {
  passed: boolean;
  blockers: string[];
  warnings: string[];
  /** Present when reference-aligned route was evaluated. */
  reference_route_report?: ReferenceRouteReport;
}

export interface ReferenceRouteReport {
  required_sections: string[];
  present_sections: string[];
  missing_sections: string[];
  coverage_gap_sections: string[];
  dyad_available_sections: string[];
  dyad_unavailable_sections: string[];
  retrieval_state_summary: Record<string, string>;
}

export function runFinalVerification(input: FinalVerificationInput): FinalVerificationResult {
  const blockers: string[] = [];
  const warnings: string[] = [];

  // ─── Standard gates (preserved from original) ──────────────────────
  for (const pass of input.passes) {
    const r = pass.rubric as any;
    if (r.placeholder_gate === 'fail') blockers.push(`${pass.id}:placeholder_gate`);
    if (r.chart_fidelity_gate === 'fail') blockers.push(`${pass.id}:chart_fidelity_gate`);
    if (pass.jev?.blocked) blockers.push(`${pass.id}:jev_blocked`);
  }

  if (input.pdfPath && !fs.existsSync(input.pdfPath)) {
    blockers.push('pdf:missing');
  }

  // ─── Reference-aligned route gates ─────────────────────────────────
  let reference_route_report: ReferenceRouteReport | undefined;

  if (input.referenceRoute) {
    const rr = input.referenceRoute;
    const gates = REFERENCE_ROUTE_REQUIRED_GATES;
    blockers.push(...validatePassResultsAgainstManifest(input.passes.map(p => p.id), buildManifestFromIds(rr.requiredSectionIds)).blockers);
    const duplicateReceipts = (ids: string[]) => ids.filter((id, i) => ids.indexOf(id) !== i);
    if (duplicateReceipts((rr.dyadReceipts ?? []).map(r => r.section_id)).length) blockers.push('reference:duplicate_dyad_receipt');

    const presentIds = new Set(input.passes.map((p) => p.id));
    const missingIds = rr.requiredSectionIds.filter((id) => !presentIds.has(id));
    const coverageGapIds: string[] = [];
    const dyadAvailableIds: string[] = [];
    const dyadUnavailableIds: string[] = [];
    const retrievalStateSummary: Record<string, string> = {};

    // Missing required sections are always blockers
    for (const id of missingIds) {
      blockers.push(`reference:missing_section:${id}`);
    }

    // Per-pass reference gates
    for (let i = 0; i < input.passes.length; i++) {
      const pass = input.passes[i];
      const r = pass.rubric;
      for (const field of ['word_count_fit', 'deterministic_fact_gate', 'integrated_layering_gate', 'guardrail_gate'] as const) {
        if (!['pass', 'warn', 'fail'].includes(r?.[field])) blockers.push(`${pass.id}:required_gate_missing:${field}`);
      }

      // Detect coverage gap marker in output
      const hasCoverageGap = /coverage.gap|not supplied|absent.*required|marked.*absent/i.test(pass.output);
      if (hasCoverageGap) coverageGapIds.push(pass.id);

      // Dyad receipt gate
      if (gates.dyad_receipt) {
        const dyadReceipt = rr.dyadReceipts?.find(receipt => receipt.section_id === pass.id);
        if (dyadReceipt === undefined) {
          // Dyad receipt not provided at all
          if (pass.output.includes(DYAD_UNAVAILABLE_MARKER)) {
            dyadUnavailableIds.push(pass.id);
            blockers.push(`${pass.id}:dyad_unavailable:no_receipt`);
          } else {
            // No receipt and no marker — dyad was never invoked for this pass
            dyadUnavailableIds.push(pass.id);
            blockers.push(`${pass.id}:dyad_receipt_missing — dyad was not invoked for required section`);
          }
        } else if (!dyadReceipt.dyad_available) {
          dyadUnavailableIds.push(pass.id);
          blockers.push(`${pass.id}:dyad_unavailable — ${dyadReceipt.unavailable_reason ?? 'unknown reason'}`);
        } else {
          dyadAvailableIds.push(pass.id);
        }
      }

      // Retrieval receipt gate — when gates.retrieval_receipt is true (reference route),
      // a missing, disabled, or failed retrieval is a BLOCKER. Empty is a warning.
      if (gates.retrieval_receipt) {
        const rReceipt = rr.retrievalReceipts?.[i];
        if (!rReceipt) {
          // No receipt at all: grounding was never wired for this pass
          blockers.push(`${pass.id}:retrieval_receipt_missing — reference route requires grounding retrieval receipt`);
        } else {
          retrievalStateSummary[pass.id] = rReceipt.state;
          if (!['success', 'empty', 'failure', 'disabled'].includes(rReceipt.state)) blockers.push(`${pass.id}:retrieval_state_invalid`);
          if (rReceipt.state === 'success' && (!Number.isInteger(rReceipt.count) || rReceipt.count <= 0)) blockers.push(`${pass.id}:retrieval_success_without_passages`);
          if (rReceipt.state === 'failure') {
            blockers.push(`${pass.id}:retrieval_failure — ${rReceipt.reason ?? 'unknown error'}`);
          } else if (rReceipt.state === 'disabled') {
            // Disabled grounding is a blocker in reference route (no opt-out boolean weakens this)
            blockers.push(`${pass.id}:retrieval_disabled — reference route requires grounding; disabled state is not allowed`);
          } else if (rReceipt.state === 'empty') {
            blockers.push(`${pass.id}:retrieval_empty — required retrieval returned 0 passages`);
          }
        }
      } else if (rr.retrievalReceipts?.[i]) {
        // Non-required mode: record state, warn on disabled/empty, block only on failure
        const rReceipt = rr.retrievalReceipts[i];
        retrievalStateSummary[pass.id] = rReceipt.state;
        if (rReceipt.state === 'failure') {
          blockers.push(`${pass.id}:retrieval_failure — ${rReceipt.reason ?? 'unknown error'}`);
        }
        if (rReceipt.state === 'disabled') {
          warnings.push(`${pass.id}:retrieval_disabled — section ran without knowledge retrieval`);
        }
        if (rReceipt.state === 'empty') {
          blockers.push(`${pass.id}:retrieval_empty — required retrieval returned 0 passages`);
        }
      }

      // Required rubric dimension gates
      if (gates.word_count && r.word_count_fit === 'fail') {
        blockers.push(`${pass.id}:word_count_fail (${r.actual_words}/${r.target_words})`);
      } else if (r.word_count_fit === 'warn') {
        warnings.push(`${pass.id}:word_count_warn (${r.actual_words}/${r.target_words})`);
      }

      if (gates.deterministic_fact && r.deterministic_fact_gate === 'fail') {
        blockers.push(`${pass.id}:deterministic_fact_fail (count=${r.deterministic_fact_count})`);
      }

      if (gates.integrated_layering && r.integrated_layering_gate === 'fail') {
        blockers.push(`${pass.id}:integrated_layering_fail (layers=${r.integrated_layer_count})`);
      }

      if (gates.guardrail && r.guardrail_gate === 'fail') {
        blockers.push(`${pass.id}:guardrail_fail (violations=${r.guardrail_violations?.join(', ') ?? 'unknown'})`);
      }

      // Deterministic leakage gate. Rubric-level fail is a blocker.
      // A caller-supplied LeakageGateResult (via sectionLeakageGates) takes
      // precedence when present — it carries the full violation list, not
      // only the compact rubric summary.
      const supplied = rr.sectionLeakageGates?.[pass.id];
      if (supplied) {
        const blocks = supplied.violations.filter(v => v.severity === 'block');
        if (blocks.length > 0) {
          blockers.push(`${pass.id}:leakage_gate_fail (${blocks.length} block violations)`);
          for (const v of blocks) {
            blockers.push(`${pass.id}:leakage:${v.pattern_id}@L${v.line_number}`);
          }
        }
      } else if (r.leakage_gate === 'fail') {
        blockers.push(`${pass.id}:leakage_gate_fail (${(r.leakage_violations ?? []).length} block violations)`);
      }
    }

    // ── Reference-route source-audit gate ─────────────────────────────
    // Hash-bound clean audit is required INDEPENDENTLY of Jev when set.
    if (rr.requireSourceAudit) {
      for (const pass of input.passes) {
        const audit = rr.sourceAuditReceipts?.[pass.id];
        if (!audit) {
          blockers.push(`${pass.id}:source_audit_missing — reference route requires hash-bound source-audit receipt`);
          continue;
        }
        if (audit.status !== 'clean') {
          blockers.push(`${pass.id}:source_audit_status_${audit.status}${audit.reason ? ` — ${audit.reason}` : ''}`);
        }
        const outputSha = createHash('sha256').update(audit.model_raw, 'utf8').digest('hex');
        if (audit.output_sha256 !== outputSha) {
          blockers.push(`${pass.id}:source_audit_output_hash_mismatch`);
        }
        const passagesHash = rr.sectionPassagesHash?.[pass.id];
        const engineFacts = rr.sectionEngineFacts?.[pass.id];
        if (!hasCleanAuditEvidence(audit, pass.output)) blockers.push(`${pass.id}:source_audit_evidence_invalid`);
        if (passagesHash === undefined || engineFacts === undefined) blockers.push(`${pass.id}:source_audit_inputs_missing`);
        if (passagesHash !== undefined && engineFacts !== undefined) {
          const expected = computeAuditInputHash({
            acceptedOutput: pass.output,
            passagesHash,
            engineFacts,
          });
          if (audit.input_sha256 !== expected) {
            blockers.push(`${pass.id}:source_audit_input_hash_mismatch — accepted text, passages, or engine facts differ from audited version`);
          }
        }
        if (audit.status === 'clean' && audit.coverage.total_claims_audited <= 0) {
          blockers.push(`${pass.id}:source_audit_zero_coverage`);
        }
      }
    }

    // Synthesis completeness: the synthesis section must receive accepted section artifacts,
    // not merely mention the section IDs in prose. We verify that each non-synthesis pass
    // was itself accepted (no DYAD_UNAVAILABLE marker and no blocking rubric failures).
    const synthSectionId = rr.requiredSectionIds[rr.requiredSectionIds.length - 1];
    const synth = input.passes.find((p) => p.id === synthSectionId);
    if (synth) {
      for (const source of input.passes.filter(p => p.id !== synthSectionId)) {
        const hash = createHash('sha256').update(source.output).digest('hex');
        if (rr.synthesisInputHashes?.[source.id] !== hash) blockers.push(`${synthSectionId}:synthesis_input_unverified:${source.id}`);
      }
      const acceptedSectionIds = input.passes
        .filter((p) => p.id !== synthSectionId && !p.output.includes(DYAD_UNAVAILABLE_MARKER))
        .map((p) => p.id);
      const missingFromAccepted = rr.requiredSectionIds
        .filter((id) => id !== synthSectionId && !acceptedSectionIds.includes(id));
      if (missingFromAccepted.length > 0) {
        // Sections that were never accepted cannot be consumed by synthesis — this is a blocker
        blockers.push(`${synthSectionId}:synthesis_missing_accepted_sections — synthesis cannot consume unaccepted sections: ${missingFromAccepted.join(', ')}`);
      }
    }

    reference_route_report = {
      required_sections: rr.requiredSectionIds,
      present_sections: input.passes.map((p) => p.id),
      missing_sections: missingIds,
      coverage_gap_sections: coverageGapIds,
      dyad_available_sections: dyadAvailableIds,
      dyad_unavailable_sections: dyadUnavailableIds,
      retrieval_state_summary: retrievalStateSummary,
    };
  }

  return {
    passed: blockers.length === 0,
    blockers,
    warnings,
    reference_route_report,
  };
}

// ═══════════════════════════════════════════════════════════════════════
// ─── Final Review — reference-verifier-bound acceptance ──────────────
// ═══════════════════════════════════════════════════════════════════════
//
// Doctrine (do NOT weaken elsewhere):
//   * `finalAccepted` is derived here, never accepted from any caller.
//   * The reference-execution verifier (verifyReferenceExecution) is
//     invoked internally with the caller-supplied raw evidence.  A fresh
//     editorial audit / Jev / guardrail-disposition is only ALLOWED to
//     matter when the reference verifier has already accepted the run
//     without them — a failing reference verifier can never be rescued
//     by editorial evidence, and editorial evidence never substitutes
//     for reference gates.
//   * Bytes rule: raw receipt bytes, revision bytes, fresh audit / Jev
//     bytes, persona bytes, and CF corpus snapshot bytes are the ONLY
//     acceptable substrate.  Parsed shapes handed in as convenience are
//     re-derived from bytes and cross-checked.
//   * `mode === 'full'` is the only mode that may produce
//     `finalAccepted: true`.  Any other declared mode reports the
//     sub-verdicts (generation, source) but fixes `finalAccepted: false`
//     regardless of individual gate outcomes.
//   * Missing full 12 sections, missing coherence review when revised
//     sections exist, missing render receipt (or hash mismatch), or
//     missing parity receipt (or hash mismatch) all block acceptance.
//   * The `assembled` text must equal the canonical
//     `passes.map(p => "## " + title + "\n\n" + output).join("\n\n")`
//     — the base verifier already enforces this and we re-enforce here
//     so a caller cannot slip a mismatched pre-assembled string past.
//   * The CF corpus snapshot check is enforced when the caller supplies
//     `cloudflareSnapshot.passages`: every source_id in every receipt
//     must appear in the snapshot AND the passage sha256 must equal the
//     receipt's stored source_hashes entry for that ID.  A snapshot
//     that fails to cover every receipt-declared ID is a blocker.

import type { SectionExecutionReceipt } from './reference-execution.js';
import {
  verifyReferenceExecution,
  type EditorialVerificationOptions,
  type ReferenceVerificationResult,
} from './reference-verification.js';

/** Reviewer-supplied readback of the exact source snapshot for one ID. */
export interface CloudflareSnapshotPassage {
  id: string;
  /** Raw passage text as read back from Cloudflare Vectorize. */
  text: string;
  /** SHA-256 of `text` (recomputed here regardless — caller-supplied value is only cross-checked). */
  sha256?: string;
}

/** Complete CF snapshot for source-ID integrity binding. */
export interface CloudflareSnapshot {
  accountId: string;
  indexName: string;
  passages: CloudflareSnapshotPassage[];
}

/** Pointer to an external evidence file with an expected hash. */
export interface EvidenceFileRef {
  path: string;
  sha256: string;
  /** Optional exact raw bytes; when supplied, we recompute sha256 and cross-check. */
  rawBytes?: string;
}

/**
 * A single finding entry on a review evidence packet.
 *
 * `unresolved` is REQUIRED and MUST be a boolean.  A finding without an
 * explicit boolean disposition blocks acceptance — silence never admits.
 */
export interface EvidenceFinding {
  id: string;
  summary: string;
  unresolved: boolean;
}

/**
 * Byte-bound artifact reference used inside render/parity evidence.
 * Distinct from `EvidenceFileRef` because render/parity require the
 * caller to supply raw bytes so we can hash and cross-check on the spot;
 * a pointer alone is never enough for a rendered document.
 */
export interface ArtifactBytesRef {
  /** Repo- or run-relative path — informational only, never trusted. */
  path: string;
  /** Claimed sha256; recomputed from `rawBytes` and cross-checked. */
  sha256: string;
  /** MIME/kind label (e.g. 'application/pdf', 'text/html'). Optional. */
  kind?: string;
  /** Raw file bytes — REQUIRED. Without bytes we cannot verify anything. */
  rawBytes: string | Uint8Array;
}

/**
 * Strict evidence packet describing a rendered artifact (DOCX/PDF/HTML).
 *
 * A hash-matching JSON pointer is NOT sufficient — the caller MUST supply
 * both the QA report file (path + sha256 + bytes) AND the actual rendered
 * artifact bytes (`artifact.rawBytes`) so this module can independently:
 *   * recompute the artifact sha256 and cross-check `artifact.sha256`
 *   * verify `bindsAssembledSha256` equals `sha256(assembled)`
 *   * verify `status === 'completed'`
 *   * verify a non-empty reviewer string
 *   * reject any unresolved finding
 *
 * A previous shape allowed arbitrary text that happened to hash-match; that
 * loophole is closed here.
 */
export interface RenderEvidence extends EvidenceFileRef {
  kind?: string;
  /** Actual rendered artifact bytes (DOCX/PDF/HTML) — REQUIRED for full mode. */
  artifact: ArtifactBytesRef;
  /** Must equal the sha256 of the pipeline's assembled artifact text. */
  bindsAssembledSha256: string;
  /** MUST equal literal 'completed'. Any other value blocks. */
  status: 'completed' | string;
  /** Non-empty reviewer identifier (e.g. 'human', 'assistant-source-review'). */
  reviewer: string;
  /** ISO-8601 timestamp of when the render review was completed. */
  reviewedAt: string;
  /** Zero or more findings; any `unresolved: true` blocks. */
  findings?: readonly EvidenceFinding[];
}

/**
 * Strict evidence packet describing a bilingual parity review.
 *
 * Parity MUST bind BOTH language artifacts.  A single arbitrary file
 * pointer is rejected.  Languages entered here must exactly cover the
 * pipeline's declared `bilingualExpected` pair (e.g. 'en' and 'fr').
 *
 * Shape:
 *   * `artifacts` is keyed by ISO 639-1 language code and each entry MUST
 *     supply raw bytes; the sha256 is recomputed and cross-checked.
 *   * `languages` is the ordered list of language codes covered — length
 *     MUST be ≥2 and MUST match `Object.keys(artifacts)` as a set.
 *   * `status`, `reviewer`, `reviewedAt`, `findings` follow the same rules
 *     as `RenderEvidence`.
 *   * `bindsAssembledSha256` is optional (parity is a comparison of two
 *     renderings, not of the assembled source), but when supplied it MUST
 *     equal `sha256(assembled)`.
 */
export interface ParityEvidence extends EvidenceFileRef {
  kind?: string;
  languages: readonly string[];
  artifacts: Readonly<Record<string, ArtifactBytesRef>>;
  status: 'completed' | string;
  reviewer: string;
  reviewedAt: string;
  findings?: readonly EvidenceFinding[];
  bindsAssembledSha256?: string;
}

/** Everything the final review needs. */
export interface FinalReviewInput {
  /** Only "full" may produce `finalAccepted: true`. */
  mode: 'full' | 'generation-only' | 'source-only';
  /** Exact ordered list of required section IDs.  Must contain 12 entries for full mode. */
  requiredSectionIds: string[];
  /** Original section execution receipts, one per required section, in order. */
  receipts: SectionExecutionReceipt[];
  /** PassResults, in the same order as receipts.  `pass.output` = accepted section text. */
  passes: PassResult[];
  /** Canonical assembled artifact text.  Must equal the canonical join of passes; blocker otherwise. */
  assembled: string;
  /** Authoritative persona texts (name → raw bytes). */
  expectedPersonas: Record<string, string>;
  /** Required numeric subsections per section_id (e.g. ["2.1","2.2",…]). */
  subsectionMap: Record<string, string[]>;
  /** Full synthesis input text per section_id (used by the reference verifier to bind hashes). */
  rawSynthesisInputs: Record<string, string>;
  /** Passage source texts keyed by source_id (bytes to bind against source_hashes). */
  sourceTexts: Record<string, string>;
  /** Per-section engine facts as fed at generation. */
  sectionEngineFacts: Record<string, string>;
  /** Editorial evidence (optional; when omitted the strict path is used). */
  editorial?: EditorialVerificationOptions;
  /** Real CF readback for source snapshot integrity. Required for full mode when any receipt has non-empty source_ids. */
  cloudflareSnapshot?: CloudflareSnapshot;
  /**
   * Ordered list of primary (reviewed-primary-document) source ids that
   * were appended to CF at generation. When present:
   *   * CF snapshot binding is not required for these ids.
   *   * The caller is responsible for independently loading the primary
   *     passage registry bytes and populating `sourceTexts` for these
   *     ids from the registry (the reference verifier still binds
   *     text→sha256 for each id).
   * Absent / empty = pure CF-only run.
   */
  primaryPassageIds?: readonly string[];
  /**
   * Bytes-first evidence pointer for the independent primary passage
   * registry snapshot the run was generated with. When any receipt
   * declares a primary source id, this pointer is REQUIRED and its
   * bytes must hash to the declared sha256.
   */
  primaryPassageRegistry?: EvidenceFileRef;
  /** Render evidence pointer — required for full mode. */
  renderReceipt?: RenderEvidence;
  /** Parity evidence pointer — required for full mode when caller declares bilingual. */
  parityReceipt?: ParityEvidence;
  /** Declares that bilingual parity is expected; when true, parityReceipt is required for full mode. */
  bilingualExpected?: boolean;
}

export interface FinalReviewResult {
  /** True iff mode='full' AND every gate below is green.  Derived here, never accepted from a caller. */
  finalAccepted: boolean;
  /** Reference-execution verifier verdict (generation + source acceptance surface). */
  generationAccepted: boolean;
  /** Source acceptance derived from the reference verifier and CF snapshot integrity. */
  sourceAccepted: boolean;
  /** True when the additional reference-route rubric/synthesis/audit gates pass. */
  referenceRoutePassed: boolean;
  /** True when render / parity / coherence evidence is present and bytes-bound. */
  packagingAccepted: boolean;
  /** SHA-256 of the assembled artifact (from reference verifier). */
  artifactSha256: string;
  /** All blocker codes gathered, deduplicated in insertion order. */
  blockers: string[];
  /** Non-blocking notices. */
  warnings: string[];
  /** Reference verifier raw result. */
  referenceReport: ReferenceVerificationResult;
  /** Base final-verification result (reference route rubric gates). */
  referenceRouteReport: FinalVerificationResult;
  /** Summary of packaging evidence checks. */
  packagingReport: PackagingReport;
}

export interface PackagingReport {
  renderPresent: boolean;
  renderHashOk: boolean;
  parityRequired: boolean;
  parityPresent: boolean;
  parityHashOk: boolean;
  coherenceRequired: boolean;
  coherencePresent: boolean;
  sectionCount: number;
  sectionCountExpected: number;
}

// ─── Internal helpers (bytes-first) ──────────────────────────────────

function sha256Hex(bytes: string): string {
  return createHash('sha256').update(bytes, 'utf8').digest('hex');
}

function canonicalAssembled(passes: PassResult[]): string {
  return passes.map(p => `## ${p.title}\n\n${p.output}`).join('\n\n');
}

/**
 * Verify a CF snapshot covers every source_id every receipt declares AND
 * the passage sha256 equals the receipt's stored source_hash for that ID.
 * Never accepts a caller-computed sha256 blindly.
 */
function verifyCloudflareSnapshot(
  snapshot: CloudflareSnapshot | undefined,
  receipts: SectionExecutionReceipt[],
  primaryIds?: readonly string[],
): { blockers: string[]; verifiedIds: string[] } {
  const blockers: string[] = [];
  const verifiedIds: string[] = [];
  if (!snapshot) return { blockers, verifiedIds };
  // Recompute every snapshot passage hash; the caller-supplied sha256 is
  // cross-checked, never trusted.
  const snapshotById = new Map<string, { text: string; hash: string }>();
  for (const p of snapshot.passages) {
    if (typeof p?.id !== 'string' || !p.id.length) {
      blockers.push('cloudflare_snapshot_passage_invalid_id');
      continue;
    }
    if (typeof p.text !== 'string') {
      blockers.push(`cloudflare_snapshot_passage_missing_text:${p.id}`);
      continue;
    }
    const hash = sha256Hex(p.text);
    if (typeof p.sha256 === 'string' && p.sha256.length && p.sha256 !== hash) {
      blockers.push(`cloudflare_snapshot_passage_hash_self_mismatch:${p.id}`);
    }
    if (snapshotById.has(p.id)) {
      blockers.push(`cloudflare_snapshot_duplicate_id:${p.id}`);
    }
    snapshotById.set(p.id, { text: p.text, hash });
  }
  const primary = new Set(primaryIds ?? []);
  // Defence in depth: a primary id MUST NOT appear in the CF snapshot.
  // If it does, either the snapshot was polluted or the caller is trying
  // to spoof a primary id as CF.
  for (const id of primary) {
    if (snapshotById.has(id)) {
      blockers.push(`cloudflare_snapshot_spoofed_primary_id:${id}`);
    }
  }
  for (const receipt of receipts) {
    for (let i = 0; i < receipt.source_ids.length; i++) {
      const sourceId = receipt.source_ids[i];
      if (primary.has(sourceId)) {
        // Primary ids are verified elsewhere (against the independent
        // registry bytes); the CF snapshot has nothing to say about them.
        continue;
      }
      const expectedHash = receipt.source_hashes[i];
      const snapshotEntry = snapshotById.get(sourceId);
      if (!snapshotEntry) {
        blockers.push(`${receipt.section_id}:cloudflare_snapshot_missing:${sourceId}`);
        continue;
      }
      if (typeof expectedHash !== 'string' || !/^[0-9a-f]{64}$/.test(expectedHash)) {
        blockers.push(`${receipt.section_id}:receipt_source_hash_invalid:${sourceId}`);
        continue;
      }
      if (snapshotEntry.hash !== expectedHash) {
        blockers.push(`${receipt.section_id}:cloudflare_snapshot_hash_mismatch:${sourceId}`);
      } else {
        verifiedIds.push(sourceId);
      }
    }
  }
  return { blockers, verifiedIds };
}

/** Verify an evidence file pointer: bytes-first when provided, else pointer-only sha binding. */
function verifyEvidenceFileRef(
  ref: EvidenceFileRef | undefined,
  label: string,
): { present: boolean; hashOk: boolean; blockers: string[] } {
  const blockers: string[] = [];
  if (!ref) return { present: false, hashOk: false, blockers };
  if (typeof ref.path !== 'string' || !ref.path.length) {
    blockers.push(`${label}_ref_path_missing`);
    return { present: true, hashOk: false, blockers };
  }
  if (typeof ref.sha256 !== 'string' || !/^[0-9a-f]{64}$/.test(ref.sha256)) {
    blockers.push(`${label}_ref_sha256_invalid`);
    return { present: true, hashOk: false, blockers };
  }
  if (typeof ref.rawBytes === 'string') {
    const actual = sha256Hex(ref.rawBytes);
    if (actual !== ref.sha256) {
      blockers.push(`${label}_ref_bytes_hash_mismatch`);
      return { present: true, hashOk: false, blockers };
    }
    return { present: true, hashOk: true, blockers };
  }
  // No bytes supplied — pointer-only. We do not read the disk here; the CLI
  // may enforce that separately.  Treat as `present` but hashOk=false so
  // callers cannot pass a hash without bytes.
  blockers.push(`${label}_ref_bytes_missing`);
  return { present: true, hashOk: false, blockers };
}

/**
 * Verify a bytes-only artifact reference (render's DOCX/PDF, each parity
 * language artifact).  Always requires `rawBytes`.  Returns whether the
 * artifact hashed cleanly plus any blockers.
 */
function verifyArtifactBytesRef(
  ref: ArtifactBytesRef | undefined,
  label: string,
): { present: boolean; hashOk: boolean; sha256: string | undefined; blockers: string[] } {
  const blockers: string[] = [];
  if (!ref || typeof ref !== 'object') {
    return { present: false, hashOk: false, sha256: undefined, blockers };
  }
  if (typeof ref.path !== 'string' || !ref.path.length) blockers.push(`${label}_path_missing`);
  if (typeof ref.sha256 !== 'string' || !/^[0-9a-f]{64}$/.test(ref.sha256)) {
    blockers.push(`${label}_sha256_invalid`);
  }
  if ((typeof ref.rawBytes !== 'string' && !(ref.rawBytes instanceof Uint8Array)) || ref.rawBytes.length === 0) {
    blockers.push(`${label}_bytes_missing`);
    return { present: true, hashOk: false, sha256: undefined, blockers };
  }
  const actual = createHash('sha256').update(ref.rawBytes).digest('hex');
  if (typeof ref.sha256 === 'string' && ref.sha256.length && actual !== ref.sha256) {
    blockers.push(`${label}_bytes_hash_mismatch`);
    return { present: true, hashOk: false, sha256: actual, blockers };
  }
  return { present: true, hashOk: blockers.length === 0, sha256: actual, blockers };
}

/** Reject any review evidence packet with unresolved findings or missing reviewer/status. */
function verifyReviewShape(
  ev: { status?: string; reviewer?: string; reviewedAt?: string; findings?: readonly EvidenceFinding[] } | undefined,
  label: string,
): string[] {
  const blockers: string[] = [];
  if (!ev) { blockers.push(`${label}_review_missing`); return blockers; }
  if (ev.status !== 'completed') blockers.push(`${label}_review_status_not_completed`);
  if (typeof ev.reviewer !== 'string' || ev.reviewer.trim().length === 0) blockers.push(`${label}_review_reviewer_missing`);
  if (typeof ev.reviewedAt !== 'string' || ev.reviewedAt.trim().length === 0) blockers.push(`${label}_review_reviewedAt_missing`);
  const findings = ev.findings ?? [];
  if (!Array.isArray(findings)) { blockers.push(`${label}_review_findings_not_array`); return blockers; }
  for (let i = 0; i < findings.length; i++) {
    const f = findings[i] as unknown as { id?: unknown; summary?: unknown; unresolved?: unknown };
    if (!f || typeof f !== 'object'
      || typeof f.id !== 'string'
      || typeof f.summary !== 'string'
      || typeof f.unresolved !== 'boolean') {
      blockers.push(`${label}_review_finding_malformed:${i}`);
      continue;
    }
  }
  const unresolved = findings
    .filter(f => f && typeof (f as EvidenceFinding).unresolved === 'boolean' && (f as EvidenceFinding).unresolved === true)
    .map(f => (f as EvidenceFinding).id);
  if (unresolved.length > 0) blockers.push(`${label}_review_unresolved_findings:${unresolved.join(',')}`);
  return blockers;
}

/**
 * Strict render evidence verifier — combines pointer bytes, artifact bytes,
 * assembled-hash binding, and review-shape checks.  Returns whether every
 * render check passed, along with all blockers.
 */
function verifyRenderEvidence(
  ev: RenderEvidence | undefined,
  assembledSha256: string,
): { present: boolean; accepted: boolean; blockers: string[] } {
  if (!ev) return { present: false, accepted: false, blockers: [] };
  const blockers: string[] = [];
  const pointer = verifyEvidenceFileRef(ev, 'render');
  for (const b of pointer.blockers) blockers.push(b);
  const art = verifyArtifactBytesRef(ev.artifact, 'render_artifact');
  for (const b of art.blockers) blockers.push(b);
  if (typeof ev.bindsAssembledSha256 !== 'string' || !/^[0-9a-f]{64}$/.test(ev.bindsAssembledSha256)) {
    blockers.push('render_bindsAssembledSha256_invalid');
  } else if (ev.bindsAssembledSha256 !== assembledSha256) {
    blockers.push('render_bindsAssembledSha256_mismatch');
  }
  for (const b of verifyReviewShape(ev, 'render')) blockers.push(b);
  return { present: true, accepted: blockers.length === 0 && pointer.hashOk && art.hashOk, blockers };
}

/**
 * Strict parity evidence verifier — requires ≥2 language artifacts, each
 * with recomputed sha256, plus the pointer file itself and the review
 * shape.  Rejects arbitrary single-file parity pointers.
 */
function verifyParityEvidence(
  ev: ParityEvidence | undefined,
  expectedLanguages: readonly string[],
  assembledSha256: string,
): { present: boolean; accepted: boolean; blockers: string[] } {
  if (!ev) return { present: false, accepted: false, blockers: [] };
  const blockers: string[] = [];
  const pointer = verifyEvidenceFileRef(ev, 'parity');
  for (const b of pointer.blockers) blockers.push(b);
  const langs = Array.isArray(ev.languages) ? ev.languages : [];
  if (langs.length < 2) blockers.push('parity_languages_insufficient');
  const artifacts = ev.artifacts && typeof ev.artifacts === 'object' ? ev.artifacts : undefined;
  if (!artifacts) blockers.push('parity_artifacts_missing');
  else {
    const artKeys = Object.keys(artifacts);
    if (artKeys.length < 2) blockers.push('parity_artifacts_insufficient');
    const declaredSet = new Set(langs);
    const artSet = new Set(artKeys);
    for (const lang of declaredSet) if (!artSet.has(lang)) blockers.push(`parity_artifact_missing_for_language:${lang}`);
    for (const lang of artSet) if (!declaredSet.has(lang)) blockers.push(`parity_artifact_extra_language:${lang}`);
    if (expectedLanguages.length >= 2) {
      const expectedSet = new Set(expectedLanguages);
      for (const lang of expectedSet) if (!declaredSet.has(lang)) blockers.push(`parity_expected_language_missing:${lang}`);
    }
    for (const [lang, art] of Object.entries(artifacts)) {
      const check = verifyArtifactBytesRef(art, `parity_artifact_${lang}`);
      for (const b of check.blockers) blockers.push(b);
    }
  }
  if (typeof ev.bindsAssembledSha256 === 'string' && ev.bindsAssembledSha256.length) {
    if (!/^[0-9a-f]{64}$/.test(ev.bindsAssembledSha256)) blockers.push('parity_bindsAssembledSha256_invalid');
    else if (ev.bindsAssembledSha256 !== assembledSha256) blockers.push('parity_bindsAssembledSha256_mismatch');
  }
  for (const b of verifyReviewShape(ev, 'parity')) blockers.push(b);
  return { present: true, accepted: blockers.length === 0 && pointer.hashOk, blockers };
}

/**
 * Reference-verifier-bound final review.
 *
 * This is the ONLY entry point that produces `finalAccepted: true`.  Every
 * other API in this module returns component gates that are consumed here.
 *
 * Failure model:
 *   * Any blocker from the reference verifier (verifyReferenceExecution)
 *     propagates directly.
 *   * The reference-route final-verification gates then run against the
 *     passes reconstructed from receipts and are ALSO required to pass.
 *   * Packaging (render / parity / coherence / 12-section count) must all
 *     be present with valid byte-bound evidence.
 *   * The mode string must be exactly `'full'`.  Any other mode surfaces
 *     the sub-verdicts but forces `finalAccepted: false`.
 */
export function runFinalReview(input: FinalReviewInput): FinalReviewResult {
  const blockers: string[] = [];
  const warnings: string[] = [];

  // ── Structural preflight (never trust caller shape) ───────────────
  if (!input || typeof input !== 'object') throw new TypeError('runFinalReview: input required');
  const mode = input.mode;
  if (mode !== 'full' && mode !== 'generation-only' && mode !== 'source-only') {
    blockers.push('mode:invalid');
  }
  if (!Array.isArray(input.requiredSectionIds) || input.requiredSectionIds.length === 0) {
    blockers.push('required_section_ids:empty');
  }
  if (!Array.isArray(input.receipts) || !Array.isArray(input.passes)) {
    blockers.push('receipts_or_passes:not_array');
  }
  if (typeof input.assembled !== 'string') {
    blockers.push('assembled:missing');
  }
  const expectedAssembled = canonicalAssembled(input.passes ?? []);
  if (typeof input.assembled === 'string' && input.assembled.trim() !== expectedAssembled.trim()) {
    blockers.push('assembled:content_mismatch_with_canonical');
  }

  // ── Reference verifier (generation + source integrity) ────────────
  const referenceReport = verifyReferenceExecution({
    receipts: input.receipts ?? [],
    passes: input.passes ?? [],
    requiredSectionIds: input.requiredSectionIds ?? [],
    expectedPersonas: input.expectedPersonas ?? {},
    subsectionMap: input.subsectionMap ?? {},
    rawSynthesisInputs: input.rawSynthesisInputs ?? {},
    sourceTexts: input.sourceTexts ?? {},
    assembled: input.assembled ?? '',
    requireSourceAudit: true,
    sectionEngineFacts: input.sectionEngineFacts ?? {},
    ...(input.editorial ? { editorial: input.editorial } : {}),
  });
  const generationAccepted = referenceReport.passed;
  for (const b of referenceReport.blockers) blockers.push(`ref:${b}`);
  for (const w of referenceReport.warnings) warnings.push(`ref:${w}`);

  // ── CF snapshot binding (source acceptance) ───────────────────────
  // Primary passage registry — independent validation of the fresh
  // primary additions. When any receipt declares a primary id, the
  // caller MUST supply the registry bytes so we can hash-bind them and
  // reject a self-asserted receipt claim. Ids known to be primary are
  // then EXCLUDED from CF snapshot binding but still bound to their
  // registry-recorded text (via `sourceTexts`) by the reference
  // verifier above.
  const sourceBlockerStart = blockers.length;
  const declaredPrimaryIds = new Set(input.primaryPassageIds ?? []);
  const receiptPrimaryIds = new Set<string>();
  for (const r of input.receipts ?? []) {
    for (const error of validateSourcePartition(r, declaredPrimaryIds)) {
      blockers.push(`${r.section_id}:source_partition:${error}`);
    }
    for (const id of r?.primary_source_ids ?? []) receiptPrimaryIds.add(id);
  }
  // Every receipt-declared primary id must be listed by the caller's
  // `primaryPassageIds`, otherwise the receipt is claiming primary
  // provenance the run inputs never authorised.
  for (const id of receiptPrimaryIds) {
    if (!declaredPrimaryIds.has(id)) {
      blockers.push(`primary:receipt_id_not_authorised:${id}`);
    }
  }
  // Caller declarations must describe consumed evidence, not waive CF checks
  // for IDs whose primary provenance no receipt declares.
  for (const id of declaredPrimaryIds) {
    if (!receiptPrimaryIds.has(id)) blockers.push(`primary:declared_id_unused:${id}`);
    if (typeof id !== 'string' || id.startsWith('sw:') || id.startsWith('ed:')) {
      blockers.push(`primary:declared_id_reserved_or_invalid:${id}`);
    }
  }
  if (receiptPrimaryIds.size > 0) {
    const reg = input.primaryPassageRegistry;
    if (!reg) {
      blockers.push('primary:registry_missing');
    } else if (typeof reg.rawBytes !== 'string' || reg.rawBytes.length === 0) {
      blockers.push('primary:registry_bytes_missing');
    } else if (!/^[0-9a-f]{64}$/.test(reg.sha256)) {
      blockers.push('primary:registry_sha256_invalid');
    } else if (sha256Hex(reg.rawBytes) !== reg.sha256) {
      blockers.push('primary:registry_bytes_hash_mismatch');
    } else {
      // Bytes hash-ok — verify the registry actually authorises every
      // receipt-declared primary id and each id's text matches the
      // receipt-declared source_hash for that id.
      let parsed: any;
      try {
        parsed = JSON.parse(reg.rawBytes);
        validatePrimaryPassageAdditions({ registry: parsed });
      } catch {
        blockers.push('primary:registry_invalid');
        parsed = undefined;
      }
      if (parsed) {
        const records = parsed && parsed.records;
        if (!records || typeof records !== 'object' || Array.isArray(records)) {
          blockers.push('primary:registry_records_missing');
        } else {
          for (const r of input.receipts ?? []) {
            for (let i = 0; i < (r?.source_ids?.length ?? 0); i += 1) {
              const id = r.source_ids[i];
              if (!receiptPrimaryIds.has(id)) continue;
              const rec = (records as Record<string, any>)[id];
              if (!rec) {
                blockers.push(`${r.section_id}:primary_id_unknown:${id}`);
                continue;
              }
              if (rec.id !== id) {
                blockers.push(`${r.section_id}:primary_id_mismatch:${id}`);
              }
              if (typeof rec.text !== 'string' || rec.text.length === 0) {
                blockers.push(`${r.section_id}:primary_text_missing:${id}`);
                continue;
              }
              const recHash = sha256Hex(rec.text);
              if (rec.sha256 !== recHash) {
                blockers.push(`${r.section_id}:primary_registry_hash_self_mismatch:${id}`);
              }
              const receiptHash = r.source_hashes?.[i];
              if (typeof receiptHash !== 'string' || receiptHash !== recHash) {
                blockers.push(`${r.section_id}:primary_text_altered:${id}`);
              }
              const prov = rec.provenance;
              if (!prov || prov.kind !== 'reviewed-primary-document') {
                blockers.push(`${r.section_id}:primary_provenance_wrong_kind:${id}`);
              } else {
                if (typeof prov.url !== 'string' || !prov.url.startsWith('https://')) {
                  blockers.push(`${r.section_id}:primary_url_invalid:${id}`);
                }
                if (typeof prov.locator !== 'string' || prov.locator.length === 0) {
                  blockers.push(`${r.section_id}:primary_locator_missing:${id}`);
                }
                if (typeof prov.author !== 'string' || prov.author.length === 0) {
                  blockers.push(`${r.section_id}:primary_author_missing:${id}`);
                }
                if (typeof prov.title !== 'string' || prov.title.length === 0) {
                  blockers.push(`${r.section_id}:primary_title_missing:${id}`);
                }
                if (prov.reviewer !== 'assistant-source-review' && prov.reviewer !== 'human') {
                  blockers.push(`${r.section_id}:primary_reviewer_invalid:${id}`);
                }
              }
              // Defence in depth: id must NOT start with sw: / ed: prefixes.
              if (typeof id === 'string' && (id.startsWith('sw:') || id.startsWith('ed:'))) {
                blockers.push(`${r.section_id}:primary_id_reserved_prefix:${id}`);
              }
            }
          }
          // additionIds sanity — when present, every receipt-declared
          // primary id must appear in additionIds.
          if (Array.isArray(parsed.additionIds)) {
            const authorised = new Set<string>();
            for (const id of parsed.additionIds) if (typeof id === 'string') authorised.add(id);
            for (const id of receiptPrimaryIds) {
              if (!authorised.has(id)) {
                blockers.push(`primary:receipt_id_not_in_addition_ids:${id}`);
              }
            }
          }
        }
      }
    }
  } else if (input.primaryPassageRegistry) {
    warnings.push('primary:registry_declared_but_unused');
  }
  const snapshotResult = verifyCloudflareSnapshot(
    input.cloudflareSnapshot,
    input.receipts ?? [],
    input.primaryPassageIds ?? [],
  );
  for (const b of snapshotResult.blockers) blockers.push(`cf:${b}`);
  const primarySet = declaredPrimaryIds;
  const anyReceiptHasCFSourceIds = (input.receipts ?? []).some(
    r => Array.isArray(r?.source_ids) && r.source_ids.some(id => !primarySet.has(id)),
  );
  // Full-mode CF is still mandatory whenever ANY non-primary source id
  // shows up in any receipt (i.e. real CF passages were consumed). Pure
  // primary-only runs are not accepted for full mode — CF stays required.
  if (mode === 'full' && anyReceiptHasCFSourceIds && !input.cloudflareSnapshot) {
    blockers.push('cf:snapshot_missing_for_full_mode');
  }
  if (mode === 'full' && receiptPrimaryIds.size > 0) {
    for (const r of input.receipts ?? []) {
      if (!r.source_ids.some(id => id.startsWith('sw:') && !declaredPrimaryIds.has(id))) {
        blockers.push(`cf:${r.section_id}:mixed_sources_require_cf`);
      }
    }
  }
  const sourceAccepted =
    blockers.length === sourceBlockerStart &&
    snapshotResult.blockers.length === 0 &&
    (!!input.cloudflareSnapshot || !anyReceiptHasCFSourceIds);

  // ── Reference-route rubric gates (folded in, no dyad/retrieval duplication) ───
  // The reference verifier already re-checks retrieval + rubric per section.
  // We add ONE additional independent rubric sweep against the passes so a
  // rubric dimension that is undefined (not 'pass'/'warn'/'fail') is caught
  // even when the reference verifier's field check happens to accept it.
  // Dyad and retrieval receipt gates are NOT re-run here — the reference
  // verifier owns those checks and doubling them would either duplicate
  // blockers or, worse, require callers to fabricate dyad receipt shapes.
  const rubricBlockers: string[] = [];
  for (const p of input.passes ?? []) {
    const r: any = p?.rubric ?? {};
    for (const field of ['word_count_fit', 'deterministic_fact_gate', 'integrated_layering_gate', 'guardrail_gate'] as const) {
      if (!['pass', 'warn', 'fail'].includes(r[field])) rubricBlockers.push(`${p.id}:required_gate_missing:${field}`);
      else if (r[field] === 'fail') rubricBlockers.push(`${p.id}:required_gate_fail:${field}`);
    }
  }
  for (const b of rubricBlockers) blockers.push(`route:${b}`);
  const referenceRoutePassed = rubricBlockers.length === 0;
  const referenceRouteReport: FinalVerificationResult = {
    passed: referenceRoutePassed,
    blockers: rubricBlockers,
    warnings: [],
  };

  // ── Packaging evidence (render / parity / coherence / 12 sections) ──
  //
  // Doctrine for this block:
  //   * Render evidence in full mode must:
  //       - carry pointer bytes (path/sha256/rawBytes cross-checked)
  //       - carry actual artifact bytes (DOCX/PDF/HTML) with sha256 recomputed
  //       - bind bindsAssembledSha256 exactly to sha256(assembled)
  //       - present a completed review with reviewer/timestamp and no
  //         unresolved findings
  //   * Parity, when bilingual is expected, must bind BOTH language
  //     artifacts with recomputed sha256, declare a matching languages
  //     list, and present the same completed-review shape.
  //   * Coherence review is REQUIRED for every full-mode run (not only
  //     editorial runs).  Its artifactSha256 must equal sha256(assembled)
  //     when the assembled text is present, and its reviewer/status/
  //     unresolved-finding rules mirror the shape above.  Editorial runs
  //     inherit the stricter reference-verifier bindings on top.
  const assembledCanonical = canonicalAssembled(input.passes ?? []);
  const assembledForBinding = typeof input.assembled === 'string' ? input.assembled : assembledCanonical;
  const assembledSha256 = sha256Hex(assembledForBinding);
  const renderResult = verifyRenderEvidence(input.renderReceipt, assembledSha256);
  const parityRequired = mode === 'full' && !!input.bilingualExpected;
  const expectedParityLanguages: readonly string[] = parityRequired
    ? (input.parityReceipt?.languages ?? [])
    : [];
  const parityResult = verifyParityEvidence(input.parityReceipt, expectedParityLanguages, assembledSha256);

  const coherenceRequired = mode === 'full';
  const coherenceReview = input.editorial?.coherenceReview;
  const coherenceBlockers: string[] = [];
  if (coherenceRequired) {
    if (!coherenceReview || typeof coherenceReview !== 'object') {
      coherenceBlockers.push('coherence:missing');
    } else {
      if (coherenceReview.status !== 'reviewed') coherenceBlockers.push('coherence:status_not_reviewed');
      if (typeof (coherenceReview as { reviewer?: unknown }).reviewer !== 'string'
        || ((coherenceReview as { reviewer: string }).reviewer !== 'assistant-source-review'
          && (coherenceReview as { reviewer: string }).reviewer !== 'human')) {
        coherenceBlockers.push('coherence:reviewer_invalid');
      }
      if (typeof coherenceReview.reviewedAt !== 'string' || coherenceReview.reviewedAt.trim().length === 0) {
        coherenceBlockers.push('coherence:reviewedAt_missing');
      }
      if (typeof coherenceReview.artifactSha256 !== 'string' || !/^[0-9a-f]{64}$/.test(coherenceReview.artifactSha256)) {
        coherenceBlockers.push('coherence:artifactSha256_invalid');
      } else if (coherenceReview.artifactSha256 !== assembledSha256) {
        coherenceBlockers.push('coherence:artifactSha256_not_bound_to_assembled');
      }
      const findings = Array.isArray(coherenceReview.findings) ? coherenceReview.findings : [];
      for (let i = 0; i < findings.length; i++) {
        const f = findings[i] as unknown as { id?: unknown; summary?: unknown; unresolved?: unknown };
        if (!f || typeof f !== 'object'
          || typeof f.id !== 'string'
          || typeof f.summary !== 'string'
          || typeof f.unresolved !== 'boolean') {
          coherenceBlockers.push(`coherence:finding_malformed:${i}`);
        }
      }
      const unresolved = findings
        .filter(f => f && (f as { unresolved?: boolean }).unresolved === true)
        .map(f => (f as { id: string }).id);
      if (unresolved.length > 0) coherenceBlockers.push(`coherence:unresolved_findings:${unresolved.join(',')}`);
    }
  }

  if (mode === 'full') {
    if (!renderResult.present) blockers.push('render:missing');
    else for (const b of renderResult.blockers) blockers.push(b);
    if (parityRequired) {
      if (!parityResult.present) blockers.push('parity:missing');
      else for (const b of parityResult.blockers) blockers.push(b);
    }
    for (const b of coherenceBlockers) blockers.push(b);
    if ((input.requiredSectionIds?.length ?? 0) !== 12) {
      blockers.push('sections:full_mode_requires_12');
    } else {
      // Strict canonical ID match — order, spelling, and identity all count.
      // A length-12 caller list with any renamed or permuted ID is rejected.
      const canonical = CANONICAL_FULL_MODE_SECTION_IDS;
      const declared = input.requiredSectionIds ?? [];
      for (let i = 0; i < canonical.length; i++) {
        if (declared[i] !== canonical[i]) {
          blockers.push(`sections:canonical_id_mismatch:index_${i}:expected_${canonical[i]}:got_${declared[i] ?? '<missing>'}`);
        }
      }
    }
  } else {
    // Non-full modes: never accept but preserve the sub-reports for the CLI.
  }

  const packagingReport: PackagingReport = {
    renderPresent: renderResult.present,
    renderHashOk: renderResult.present && renderResult.accepted,
    parityRequired,
    parityPresent: parityResult.present,
    parityHashOk: parityResult.present && parityResult.accepted,
    coherenceRequired,
    coherencePresent: coherenceRequired ? coherenceBlockers.length === 0 : true,
    sectionCount: input.requiredSectionIds?.length ?? 0,
    sectionCountExpected: 12,
  };
  const packagingAccepted = mode === 'full'
    ? renderResult.present && renderResult.accepted
      && (!parityRequired || (parityResult.present && parityResult.accepted))
      && packagingReport.coherencePresent
      && packagingReport.sectionCount === packagingReport.sectionCountExpected
    : false;

  // ── Derive finalAccepted (never accept from caller) ────────────────
  const finalAccepted = mode === 'full'
    && generationAccepted
    && sourceAccepted
    && referenceRoutePassed
    && packagingAccepted
    && blockers.length === 0;

  // Dedupe blockers preserving insertion order
  const seen = new Set<string>();
  const dedupedBlockers = blockers.filter(b => { if (seen.has(b)) return false; seen.add(b); return true; });

  return {
    finalAccepted,
    generationAccepted,
    sourceAccepted,
    referenceRoutePassed,
    packagingAccepted,
    artifactSha256: referenceReport.artifactSha256,
    blockers: dedupedBlockers,
    warnings,
    referenceReport,
    referenceRouteReport,
    packagingReport,
  };
}
