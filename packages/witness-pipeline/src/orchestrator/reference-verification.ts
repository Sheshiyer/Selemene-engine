// ─── Reference Verification ────────────────────────────────────────────
// Verifies generation acceptance for SectionExecutionReceipt + PassResult.
// Does NOT claim rendered review or bilingual parity.
// No network calls.

import { createHash } from 'node:crypto';
import type { SectionExecutionReceipt } from './reference-execution.js';
import type { PassResult } from './integrated.js';
import { validateSubsectionHeadings } from './reference-execution.js';
import { computeAuditInputHash, hasCleanAuditEvidence } from './reference-source-audit.js';
import {
  verifyEditorialSection,
  verifyEditorialPriorContext,
  verifyEditorialCoherenceReview,
  type EditorialVerificationOptions,
} from './editorial-verification.js';

export type {
  EditorialVerificationOptions,
  EditorialSectionEvidence,
  EditorialCoherenceReview,
  CoherenceReviewFinding,
  ReviewedRevisionRef,
  FreshAuditSidecar,
  FreshJevSidecar,
  EditorialSectionVerdict,
  PriorContextEdge,
} from './editorial-verification.js';

// ─── Public API types ──────────────────────────────────────────────────

/**
 * Input for verifying a complete reference-execution run.
 *
 * @param receipts        - One SectionExecutionReceipt per section, in declared order.
 * @param passes          - PassResult array from OrchestratorOutput (same order as receipts).
 * @param requiredSectionIds - Exact ordered list of expected section IDs.
 * @param expectedPersonas   - Map of persona name → authoritative identity text.
 *                            Supplied separately from the receipt to avoid trusting the receipt's own copy.
 * @param subsectionMap      - Optional map of section_id → required numeric subsection IDs (e.g. ["2.1", "2.2"]).
 * @param rawSynthesisInputs - Optional map of section_id → the full synthesis input string that was
 *                            hashed to produce synthesis_input_hash. When provided, the hash is verified.
 */
export interface ReferenceVerificationInput {
  receipts: SectionExecutionReceipt[];
  passes: PassResult[];
  requiredSectionIds: string[];
  expectedPersonas: Record<string, string>;
  subsectionMap?: Record<string, string[]>;
  rawSynthesisInputs?: Record<string, string>;
  sourceTexts?: Record<string, string>;
  assembled?: string;
  /**
   * When true, every non-failed section receipt MUST carry a clean, hash-bound
   * source_audit. Real reference route sets this to true. Legacy paths omit it.
   */
  requireSourceAudit?: boolean;
  /**
   * Per-section engine facts used at generation time. When supplied together
   * with the receipt's passages_hash, the verifier recomputes the audit input
   * hash and blocks on mismatch (hash tampering / stale audit / swapped
   * sources).
   */
  sectionEngineFacts?: Record<string, string>;
  /**
   * Optional provenance-aware editorial evidence.  When present the verifier
   * accepts revised sections whose accepted pass output differs from the
   * ORIGINAL model receipt's raw synthesis, provided every editorial gate
   * (recomputed revision, fresh source-audit, fresh Jev, any needed
   * guardrail disposition, whole-document coherence review) is satisfied.
   *
   * When omitted the verifier's strict, non-editorial behavior is preserved
   * exactly.  It never mutates the stored receipt or converts a
   * fail/blocked/skipped/missing dimension into acceptance.
   */
  editorial?: EditorialVerificationOptions;
}

export interface ReferenceVerificationResult {
  passed: boolean;
  blockers: string[];
  warnings: string[];
  /** SHA-256 of the entire canonical artifact (all accepted output hashes joined in order). */
  artifactSha256: string;
  /**
   * Per-revised-section editorial verdicts.  Only populated when
   * ReferenceVerificationInput.editorial was supplied and the verifier
   * exercised the provenance-aware editorial gates.  Empty otherwise.
   */
  editorialVerdicts?: import('./editorial-verification.js').EditorialSectionVerdict[];
}

// ─── Implementation ────────────────────────────────────────────────────

function sha256(text: string): string {
  return createHash('sha256').update(text, 'utf8').digest('hex');
}

/**
 * Verify a complete reference-execution run.
 *
 * Checks performed (all blocking unless noted):
 *  1. Section order exact, no duplicates, no missing.
 *  2. Each receipt: raw.aletheios and raw.pichet non-empty; hashes match recomputed sha256.
 *  3. Each receipt: sha256(raw.synthesis) === output_hash (synthesis equals accepted pass output hash).
 *  4. Each receipt: retrieval success → count > 0, source_ids non-empty, source_hashes length matches source_ids, passages_hash non-empty.
 *  5. Persona hashes match supplied authoritative identity texts (both voices).
 *  6. Required numeric subsection headings present in accepted output (from subsectionMap).
 *  7. Rubric gates: word_count_fit / deterministic_fact_gate / integrated_layering_gate / guardrail_gate
 *     — every required gate must pass; missing, warn, and fail block acceptance.
 *  8. Jev must be judged with all verdicts pass. Missing or ambiguous judgments
 *     remain blockers; this verifier never converts shadow review into approval.
 *  9. Synthesis prior integrity: when rawSynthesisInputs[section_id] provided,
 *     sha256(rawSynthesisInput) === receipt.synthesis_input_hash.
 * 10. Pass output matches receipt: sha256(pass.output) === receipt.output_hash.
 */
export function verifyReferenceExecution(
  input: ReferenceVerificationInput,
): ReferenceVerificationResult {
  const blockers: string[] = [];
  const warnings: string[] = [];

  const { receipts, passes, requiredSectionIds, expectedPersonas, subsectionMap, rawSynthesisInputs } = input;
  const editorial = input.editorial;
  const revisedIds = editorial ? Object.keys(editorial.revisedSections) : [];
  const revisedIdSet = new Set(revisedIds);
  if (!requiredSectionIds.length || new Set(requiredSectionIds).size !== requiredSectionIds.length) blockers.push('manifest:empty_or_duplicate');
  if (JSON.stringify(passes.map(p => p.id)) !== JSON.stringify(requiredSectionIds)) blockers.push('pass:exact_order_mismatch');
  if (JSON.stringify(receipts.map(r => r.section_id)) !== JSON.stringify(requiredSectionIds)) blockers.push('receipt:exact_order_mismatch');
  if (input.assembled !== undefined) {
    // Corrected reference outputs already contain their canonical section
    // heading, while legacy passes contain body prose and still require the
    // wrapper. Select per pass from the first heading instead of globally
    // duplicating titles or breaking the legacy assembly contract.
    const expected = passes.map(p => {
      const output = p.output.trim();
      const firstLine = output.split('\n', 1)[0]?.replace(/^#{1,6}\s+/, '').trim();
      return firstLine === p.title.trim() ? output : `## ${p.title}\n\n${output}`;
    }).join('\n\n');
    if (input.assembled.trim() !== expected.trim()) blockers.push('artifact:assembled_content_mismatch');
  }

  // ── Editorial pre-gates (only when editorial evidence supplied) ─────
  if (editorial) {
    if (!editorial.originalPasses || typeof editorial.originalPasses !== 'object') {
      blockers.push('editorial:original_passes_missing');
    } else {
      for (const id of requiredSectionIds) {
        if (typeof editorial.originalPasses[id] !== 'string' || editorial.originalPasses[id].length === 0) {
          blockers.push(`editorial:original_pass_missing:${id}`);
        }
      }
    }
    for (const id of revisedIds) {
      if (!requiredSectionIds.includes(id)) {
        blockers.push(`editorial:revised_section_not_in_manifest:${id}`);
      }
    }
    // Item 5 doctrine — a coherence review MUST bind the actual
    // assembled artifact text.  When revised sections exist, the caller
    // must supply `assembled`; the verifier never lets a hash-list
    // string silently stand in for a real assembly.
    if (revisedIds.length > 0 && typeof input.assembled !== 'string') {
      blockers.push('editorial:assembled_text_required_for_coherence_binding');
    }
  }

  // ── 1. Section order, duplicates, missing ────────────────────────────
  const receiptIds = receipts.map((r) => r.section_id);
  const passIds = passes.map((p) => p.id);

  // Duplicate receipt IDs
  const seenReceiptIds = new Set<string>();
  for (const id of receiptIds) {
    if (seenReceiptIds.has(id)) {
      blockers.push(`receipt:duplicate_section_id:${id}`);
    } else {
      seenReceiptIds.add(id);
    }
  }

  // Duplicate pass IDs
  const seenPassIds = new Set<string>();
  for (const id of passIds) {
    if (seenPassIds.has(id)) {
      blockers.push(`pass:duplicate_section_id:${id}`);
    } else {
      seenPassIds.add(id);
    }
  }

  // Missing required sections in receipts
  for (const requiredId of requiredSectionIds) {
    if (!seenReceiptIds.has(requiredId)) {
      blockers.push(`receipt:missing_required_section:${requiredId}`);
    }
  }

  // Missing required sections in passes
  for (const requiredId of requiredSectionIds) {
    if (!seenPassIds.has(requiredId)) {
      blockers.push(`pass:missing_required_section:${requiredId}`);
    }
  }

  // Exact order: receipt order must match requiredSectionIds order
  const orderedReceiptIds = receiptIds.filter((id) => requiredSectionIds.includes(id));
  const expectedOrder = requiredSectionIds.filter((id) => seenReceiptIds.has(id));
  for (let i = 0; i < expectedOrder.length; i++) {
    if (orderedReceiptIds[i] !== expectedOrder[i]) {
      blockers.push(
        `receipt:section_order_mismatch — expected "${expectedOrder[i]}" at position ${i}, got "${orderedReceiptIds[i] ?? 'nothing'}"`,
      );
    }
  }

  // Build lookup maps for per-section checks
  const receiptBySection = new Map<string, SectionExecutionReceipt>();
  for (const receipt of receipts) {
    receiptBySection.set(receipt.section_id, receipt);
  }
  const passBySection = new Map<string, PassResult>();
  for (const pass of passes) {
    passBySection.set(pass.id, pass);
  }

  // ── Per-section checks ───────────────────────────────────────────────
  for (const sectionId of requiredSectionIds) {
    const receipt = receiptBySection.get(sectionId);
    const pass = passBySection.get(sectionId);

    if (!receipt || !pass) {
      // Already captured by missing_required_section above
      continue;
    }

    const prefix = sectionId;

    // Skip per-field integrity checks on failed receipts (they are structurally empty by design)
    const isFailed = receipt.outcome === 'failed';
    const isRevised = revisedIdSet.has(sectionId);
    // For revised sections, editorial gate decides whether a `failed` outcome is
    // narrowly permissible (shared failed-draft classification + fresh reviews). Base
    // gate still blocks unknown outcome kinds; verifyEditorialSection re-checks.
    if (!['ok', 'repaired'].includes(receipt.outcome) && !(isRevised && receipt.outcome === 'failed')) {
      blockers.push(`${prefix}:execution_not_accepted`);
    }
    if (receipt.retrieval.state !== 'success') blockers.push(`${prefix}:retrieval_not_success`);
    if (receipt.retrieval.section_id !== sectionId || JSON.stringify(receipt.retrieval.source_ids) !== JSON.stringify(receipt.source_ids) || receipt.retrieval.passages_hash !== receipt.passages_hash) blockers.push(`${prefix}:retrieval_integrity_mismatch`);
    for (const field of ['word_count_fit', 'deterministic_fact_gate', 'integrated_layering_gate', 'guardrail_gate'] as const) {
      if (pass.rubric?.[field] !== 'pass') blockers.push(`${prefix}:required_gate_unresolved:${field}`);
    }
    if (!subsectionMap || !(sectionId in subsectionMap)) blockers.push(`${prefix}:subsection_contract_missing`);
    if (!isRevised && !rawSynthesisInputs?.[sectionId]) blockers.push(`${prefix}:synthesis_input_missing`);
    if (!isRevised) {
      const synthesisText = rawSynthesisInputs?.[sectionId] ?? '';
      for (const earlier of passes.slice(0, requiredSectionIds.indexOf(sectionId))) {
        if (!synthesisText.includes(earlier.output)) blockers.push(`${prefix}:prior_section_missing:${earlier.id}`);
      }
    }
    for (const [i, sourceId] of receipt.source_ids.entries()) {
      const text = input.sourceTexts?.[sourceId];
      if (!text || sha256(text) !== receipt.source_hashes[i]) blockers.push(`${prefix}:source_text_unverified:${sourceId}`);
    }
    if (!isRevised) {
      if (receipt.jev?.status !== 'judged') blockers.push(`${prefix}:jev_not_judged`);
      for (const field of ['guardrail', 'framing', 'grounding', 'register'] as const) {
        if (receipt.jev?.verdicts?.[field] !== 'pass') blockers.push(`${prefix}:jev_unresolved:${field}`);
      }
    }

    // ── 2. Raw voices non-empty + hash integrity ─────────────────────
    // For revised sections we ALWAYS enforce raw hash integrity, even when the
    // original outcome is `failed` under the editorial recovery policy — the fresh audit
    // is bound to the ORIGINAL raw voices and their hashes must be intact.
    if (!isFailed || isRevised) {
      if (!receipt.raw.aletheios || receipt.raw.aletheios.trim().length === 0) {
        blockers.push(`${prefix}:raw_aletheios_empty`);
      } else {
        const expectedHash = sha256(receipt.raw.aletheios);
        if (receipt.aletheios_hash !== expectedHash) {
          blockers.push(
            `${prefix}:aletheios_hash_mismatch — receipt.aletheios_hash "${receipt.aletheios_hash.slice(0, 16)}…" ≠ sha256(raw.aletheios) "${expectedHash.slice(0, 16)}…"`,
          );
        }
      }

      if (!receipt.raw.pichet || receipt.raw.pichet.trim().length === 0) {
        blockers.push(`${prefix}:raw_pichet_empty`);
      } else {
        const expectedHash = sha256(receipt.raw.pichet);
        if (receipt.pichet_hash !== expectedHash) {
          blockers.push(
            `${prefix}:pichet_hash_mismatch — receipt.pichet_hash "${receipt.pichet_hash.slice(0, 16)}…" ≠ sha256(raw.pichet) "${expectedHash.slice(0, 16)}…"`,
          );
        }
      }

      // ── 3. Raw synthesis hash equals output_hash ──────────────────
      if (!receipt.raw.synthesis || receipt.raw.synthesis.trim().length === 0) {
        blockers.push(`${prefix}:raw_synthesis_empty`);
      } else {
        const expectedOutputHash = sha256(receipt.raw.synthesis);
        if (receipt.output_hash !== expectedOutputHash) {
          blockers.push(
            `${prefix}:synthesis_output_hash_mismatch — receipt.output_hash "${receipt.output_hash.slice(0, 16)}…" ≠ sha256(raw.synthesis) "${expectedOutputHash.slice(0, 16)}…"`,
          );
        }
      }
    }

    // ── 4. Retrieval success: count, IDs, hashes, passages_hash ──────
    if (receipt.retrieval.state === 'success') {
      if (receipt.retrieval.count <= 0) {
        blockers.push(`${prefix}:retrieval_success_zero_count`);
      }
      if (!receipt.source_ids || receipt.source_ids.length === 0) {
        blockers.push(`${prefix}:retrieval_success_empty_source_ids`);
      } else {
        if (receipt.source_ids.length !== receipt.retrieval.count) {
          blockers.push(
            `${prefix}:retrieval_source_id_count_mismatch — source_ids.length ${receipt.source_ids.length} ≠ retrieval.count ${receipt.retrieval.count}`,
          );
        }
      }
      if (!receipt.source_hashes || receipt.source_hashes.length === 0) {
        blockers.push(`${prefix}:retrieval_success_empty_source_hashes`);
      } else if (receipt.source_hashes.length !== receipt.source_ids.length) {
        blockers.push(
          `${prefix}:retrieval_hash_count_mismatch — source_hashes.length ${receipt.source_hashes.length} ≠ source_ids.length ${receipt.source_ids.length}`,
        );
      }
      if (!receipt.passages_hash || receipt.passages_hash.length === 0) {
        blockers.push(`${prefix}:retrieval_success_empty_passages_hash`);
      }
      // Verify each source hash is plausibly a sha256 (64 hex chars)
      for (let i = 0; i < receipt.source_hashes.length; i++) {
        if (!/^[0-9a-f]{64}$/.test(receipt.source_hashes[i])) {
          blockers.push(`${prefix}:source_hash_invalid_format — source_hashes[${i}] "${receipt.source_hashes[i].slice(0, 16)}…" is not a valid sha256 hex`);
        }
      }
    }

    // ── 5. Persona hashes match supplied authoritative identity texts ─
    for (const [personaKey, personaField] of [
      ['aletheios', receipt.aletheios_persona],
      ['pichet', receipt.pichet_persona],
    ] as const) {
      if (personaField.name !== personaKey) blockers.push(`${prefix}:wrong_persona:${personaKey}`);
      const authoritative = expectedPersonas[personaField.name];
      if (authoritative === undefined) {
        warnings.push(`${prefix}:persona_not_supplied:${personaField.name} — expected persona text not provided for verification`);
      } else {
        const expectedHash = sha256(authoritative);
        if (personaField.sourceHash !== expectedHash) {
          blockers.push(
            `${prefix}:persona_hash_mismatch:${personaField.name} — receipt.sourceHash "${personaField.sourceHash.slice(0, 16)}…" ≠ sha256(supplied identity text) "${expectedHash.slice(0, 16)}…"`,
          );
        }
      }
    }

    // ── 6. Required numeric subsection headings ───────────────────────
    if (!isFailed && subsectionMap?.[sectionId]?.length) {
      const requiredSubsections = subsectionMap[sectionId];
      const acceptedOutput = pass.output;
      const missing = validateSubsectionHeadings(acceptedOutput, requiredSubsections);
      for (const missingId of missing) {
        blockers.push(`${prefix}:missing_required_subsection:${missingId}`);
      }
    }

    // ── 7. Rubric gates ───────────────────────────────────────────────
    if (pass.rubric) {
      const rubric = pass.rubric;
      if (rubric.word_count_fit === 'fail') {
        blockers.push(`${prefix}:rubric:word_count_fail (${rubric.actual_words}/${rubric.target_words})`);
      } else if (rubric.word_count_fit === 'warn') {
        warnings.push(`${prefix}:rubric:word_count_warn (${rubric.actual_words}/${rubric.target_words})`);
      }
      if (rubric.deterministic_fact_gate === 'fail') {
        blockers.push(`${prefix}:rubric:deterministic_fact_fail (count=${rubric.deterministic_fact_count})`);
      } else if (rubric.deterministic_fact_gate === 'warn') {
        warnings.push(`${prefix}:rubric:deterministic_fact_warn (count=${rubric.deterministic_fact_count})`);
      }
      if (rubric.integrated_layering_gate === 'fail') {
        blockers.push(`${prefix}:rubric:integrated_layering_fail (layers=${rubric.integrated_layer_count})`);
      } else if (rubric.integrated_layering_gate === 'warn') {
        warnings.push(`${prefix}:rubric:integrated_layering_warn (layers=${rubric.integrated_layer_count})`);
      }
      if (rubric.guardrail_gate === 'fail') {
        blockers.push(
          `${prefix}:rubric:guardrail_fail${rubric.guardrail_violations.length ? ` (${rubric.guardrail_violations.join(', ')})` : ''}`,
        );
      }
    }

    // ── 8. Jev: all verdicts pass or unresolved (could-not-tell = warn) ─
    if (receipt.jev && !isRevised) {
      const jev = receipt.jev;
      if (jev.status === 'error') {
        warnings.push(`${prefix}:jev_error — ${jev.reason ?? 'unknown error'}`);
      } else if (jev.status === 'skipped') {
        warnings.push(`${prefix}:jev_skipped — ${jev.reason ?? 'not configured'}`);
      } else if (jev.status === 'judged' && jev.verdicts) {
        for (const [dimension, verdict] of Object.entries(jev.verdicts) as [string, string][]) {
          if (verdict === 'fail') {
            blockers.push(`${prefix}:jev_verdict_fail:${dimension}`);
          } else if (verdict === 'could-not-tell') {
            warnings.push(`${prefix}:jev_verdict_unresolved:${dimension}`);
          }
        }
        if (jev.blocked) {
          blockers.push(`${prefix}:jev_blocked`);
        }
      }
    } else if (!isRevised) {
      // No Jev receipt — warn but do not block (Jev is optional)
      warnings.push(`${prefix}:jev_not_configured — no Jev receipt; generation acceptance does not include Jev judgment`);
    }

    // ── 9. Synthesis prior integrity ──────────────────────────────────
    if (!isFailed && rawSynthesisInputs?.[sectionId]) {
      const rawInput = rawSynthesisInputs[sectionId];
      const expectedHash = sha256(rawInput);
      if (receipt.synthesis_input_hash !== expectedHash) {
        blockers.push(
          `${prefix}:synthesis_input_hash_mismatch — receipt.synthesis_input_hash "${receipt.synthesis_input_hash.slice(0, 16)}…" ≠ sha256(rawSynthesisInput) "${expectedHash.slice(0, 16)}…"`,
        );
      }
    }

    // ── 10. Pass output matches receipt output_hash ───────────────────
    if (!isFailed && !isRevised) {
      if (!pass.output || pass.output.trim().length === 0) {
        blockers.push(`${prefix}:pass_output_empty`);
      } else {
        const expectedOutputHash = sha256(pass.output);
        if (receipt.output_hash !== expectedOutputHash) {
          blockers.push(
            `${prefix}:pass_output_hash_mismatch — receipt.output_hash "${receipt.output_hash.slice(0, 16)}…" ≠ sha256(pass.output) "${expectedOutputHash.slice(0, 16)}…"`,
          );
        }
      }
    }

    // ── 11. Source-audit (independent claim-to-source review) ─────────
    // Only enforced for non-failed receipts when requireSourceAudit=true.
    // A missing / malformed / non-clean / hash-inconsistent audit is a blocker.
    if (!isFailed && input.requireSourceAudit && !isRevised) {
      const audit = receipt.source_audit;
      if (!audit) {
        blockers.push(`${prefix}:source_audit_missing — reference route requires a hash-bound source-audit receipt`);
      } else {
        if (audit.status !== 'clean') {
          blockers.push(`${prefix}:source_audit_status_${audit.status}${audit.reason ? ` — ${audit.reason}` : ''}`);
        }
        if (!hasCleanAuditEvidence(audit, pass.output)) blockers.push(`${prefix}:source_audit_evidence_invalid`);
        // Output-hash binding: the audit must have judged this exact text.
        const expectedOutputSha = sha256(audit.model_raw);
        if (audit.output_sha256 !== expectedOutputSha) {
          blockers.push(`${prefix}:source_audit_output_hash_mismatch`);
        }
        // Input-hash binding: recompute from accepted output + passages_hash + engine facts.
        const engineFacts = input.sectionEngineFacts?.[sectionId];
        if (engineFacts === undefined) blockers.push(`${prefix}:source_audit_engine_facts_missing`);
        if (engineFacts !== undefined) {
          const expectedInput = computeAuditInputHash({
            acceptedOutput: pass.output,
            passagesHash: receipt.passages_hash,
            engineFacts,
          });
          if (audit.input_sha256 !== expectedInput) {
            blockers.push(`${prefix}:source_audit_input_hash_mismatch — audit judged different text, passages, or engine facts than were accepted`);
          }
        }
        // A "clean" audit still fails when coverage is zero.
        if (audit.status === 'clean' && audit.coverage.total_claims_audited <= 0) {
          blockers.push(`${prefix}:source_audit_zero_coverage`);
        }
      }
    }
  }

  // ── Editorial branch (provenance-aware acceptance) ────────────────────
  let editorialVerdicts: import('./editorial-verification.js').EditorialSectionVerdict[] | undefined;
  if (editorial) {
    editorialVerdicts = [];
    const revisedHashesById: Record<string, string> = {};
    const revisedOutputsById: Record<string, string> = {};

    for (const sectionId of requiredSectionIds) {
      if (!revisedIdSet.has(sectionId)) continue;
      const receipt = receiptBySection.get(sectionId);
      const pass = passBySection.get(sectionId);
      if (!receipt || !pass) continue;
      const evidence = editorial.revisedSections[sectionId];
      const passOutputHash = pass.output ? sha256(pass.output) : '';
      const verdict = verifyEditorialSection({
        sectionId,
        originalReceipt: receipt,
        passOutput: pass.output ?? '',
        passOutputHash,
        evidence,
        implementationSourceTexts: editorial.implementationSourceTexts,
        trustedPassageSources: editorial.trustedPassageSources,
        expectedCF: editorial.expectedCF,
      });
      editorialVerdicts.push(verdict);
      blockers.push(...verdict.blockers);
      revisedHashesById[sectionId] = verdict.revisedOutputHash;
      revisedOutputsById[sectionId] = evidence.revisionArtifact.revisedOutput;
    }

    // Reverify every supplied historical revision against the SAME immutable
    // original receipt. Historical acceptance proves prior consumption only;
    // it never replaces the current output or the whole-document review.
    const verifiedHistoricalOutputs: Record<string, string[]> = {};
    for (const [id, histories] of Object.entries(editorial.historicalRevisions ?? {})) {
      const originalReceipt = receiptBySection.get(id);
      if (!originalReceipt || !revisedIdSet.has(id) || !Array.isArray(histories)) {
        blockers.push(`editorial:historical_revision_invalid_section:${id}`);
        continue;
      }
      for (const [index, evidence] of histories.entries()) {
        if (!evidence?.revisionArtifact || typeof evidence.revisionArtifact.revisedOutput !== 'string') {
          blockers.push(`editorial:historical_revision_malformed:${id}:${index}`);
          continue;
        }
        const output = evidence.revisionArtifact.revisedOutput;
        const verdict = verifyEditorialSection({ sectionId: id, originalReceipt,
          passOutput: output, passOutputHash: sha256(output), evidence,
          implementationSourceTexts: editorial.implementationSourceTexts,
          trustedPassageSources: editorial.trustedPassageSources, expectedCF: editorial.expectedCF });
        blockers.push(...verdict.blockers.map(b => `historical:${index}:${b}`));
        if (!verdict.blockers.length) (verifiedHistoricalOutputs[id] ??= []).push(output);
      }
    }

    // Original prior-context edges + no-forged-consumption for revised sections
    for (const sectionId of requiredSectionIds) {
      if (!revisedIdSet.has(sectionId)) continue;
      const receipt = receiptBySection.get(sectionId);
      if (!receipt) continue;
      const priorBlockers = verifyEditorialPriorContext({
        sectionId,
        originalReceipt: receipt,
        requiredSectionIds,
        originalPasses: editorial.originalPasses ?? {},
        sectionTitles: Object.fromEntries(passes.map(p => [p.id, p.title])),
        revisedOutputsByOtherSections: revisedOutputsById,
        verifiedHistoricalOutputs,
        priorContext: editorial.priorContext?.[sectionId],
        revisedSectionIds: revisedIds,
      });
      blockers.push(...priorBlockers);
    }

    // Whole-document coherence review (required whenever revised sections exist)
    if (revisedIds.length > 0) {
      const effectiveHashesInOrder = requiredSectionIds.map((id) => {
        if (revisedIdSet.has(id)) return revisedHashesById[id] ?? '';
        const r = receiptBySection.get(id);
        return r ? r.output_hash : '';
      });
      const expectedArtifactSha = sha256(
        effectiveHashesInOrder.filter((h) => h.length > 0).join(':'),
      );
      const coherenceBlockers = verifyEditorialCoherenceReview({
        review: editorial.coherenceReview,
        requiredSectionIds,
        effectiveHashesInOrder,
        revisedSectionIds: revisedIds,
        revisedHashesById,
        expectedArtifactSha256: expectedArtifactSha,
        assembledText: typeof input.assembled === 'string' ? input.assembled : undefined,
      });
      blockers.push(...coherenceBlockers);
    }
  }

  // ── Artifact SHA-256 ─────────────────────────────────────────────────
  // Computed over all accepted output hashes in required order.
  // Includes only non-failed sections with valid output_hash.
  //
  // For revised sections the ORIGINAL receipt's output_hash is superseded by
  // the recomputed revised output hash from the editorial verdict — we never
  // publish an artifact hash bound to the pre-edit text when the accepted
  // output was the post-edit revision.
  const revisedHashOverrides: Record<string, string> = {};
  if (editorialVerdicts) {
    for (const v of editorialVerdicts) revisedHashOverrides[v.sectionId] = v.revisedOutputHash;
  }
  const artifactParts = requiredSectionIds
    .map((id) => {
      const receipt = receiptBySection.get(id);
      if (!receipt) return null;
      if (revisedHashOverrides[id]) {
        return revisedHashOverrides[id];
      }
      if (receipt.outcome === 'failed' || receipt.output_hash.length === 0) return null;
      return receipt.output_hash;
    })
    .filter((h): h is string => !!h);

  const artifactSha256 = sha256(input.assembled ?? artifactParts.join(':'));

  return {
    passed: blockers.length === 0 && warnings.length === 0,
    blockers: [...new Set([...blockers, ...warnings.map(w => `unresolved:${w}`)])],
    warnings,
    artifactSha256,
    ...(editorialVerdicts ? { editorialVerdicts } : {}),
  };
}
