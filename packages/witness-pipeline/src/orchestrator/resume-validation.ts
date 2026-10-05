// ─── Resume Validation for Reference Report Runs ──────────────────────
// Validates a prior run directory as a safe, contiguous prefix of successful
// section receipts that can be reused for a resumed run.
//
// Contract:
//   - Only receipts with outcome 'ok' or 'repaired' qualify; the first 'failed'
//     receipt terminates the usable prefix (sections after it are not reused).
//   - All hashes in every qualifying receipt are verified against the stored
//     raw outputs and internal cross-references before any external state is consulted.
//   - Persona hashes in receipts are checked against the caller-supplied current
//     persona hashes; a mismatch rejects the entire prefix.
//   - The inputs.json identity block (subject, language, inputHash, runtimeHashes,
//     personaHashes) must match the current run's parameters exactly.
//   - Jev-blocked receipts (receipt.jev.blocked === true) are never reused.
//   - The full synthesis output text (raw.synthesis) of each reused receipt is
//     returned as the prior-section content for continuation; it is never re-generated.
//   - All validation is purely local (no LLM calls, no network calls, no CF queries).
//
// Usage: call validateResumeDir() before starting any LLM generation.

import { createHash } from 'node:crypto';
import { promises as fs } from 'node:fs';
import { join } from 'node:path';
import type { SectionExecutionReceipt } from './reference-execution.js';

// ─── Public types ─────────────────────────────────────────────────────

/**
 * The identity block written to inputs.json by the run script.
 * All fields must match the current run exactly for resumption to be safe.
 */
export interface RunInputsIdentity {
  inputHash: string;
  subject: string;
  language: string;
  accountId: string;
  index: string;
  /** [aletheiosHash, pichetHash] */
  personaHashes: string[];
  /** Map of relative file path → sha256 of file content at run start. */
  runtimeHashes: Record<string, string>;
}

/**
 * Parameters that describe the current run's identity.
 * Passed by the caller for comparison against the prior run's inputs.json.
 */
export interface CurrentRunIdentity {
  /** Freshly retrieved framework text, keyed by corpus ID. */
  sourceTexts: Record<string, string>;
  inputHash: string;
  subject: string;
  language: string;
  accountId: string;
  index: string;
  /** sha256 of aletheios IDENTITY.md text (loaded by caller). */
  aletheiosPersonaHash: string;
  /** sha256 of pichet IDENTITY.md text (loaded by caller). */
  pichetPersonaHash: string;
  /** Current runtime file hashes (same keys as the prior run's runtimeHashes). */
  runtimeHashes: Record<string, string>;
}

/**
 * A single reusable section from the prior run.
 * The caller uses `acceptedOutput` as the section output and
 * `synthesis_input_text` to reconstruct prior-section context for the next section.
 */
export interface ReusedSection {
  section_id: string;
  /** The accepted section output (raw.synthesis from the receipt). */
  acceptedOutput: string;
  /** The full synthesis_input_text from the receipt (for audit and context threading). */
  synthesis_input_text: string;
  /** The full receipt, immutable. */
  receipt: SectionExecutionReceipt;
}

/**
 * Outcome when validation succeeds.
 */
export interface ResumeValidationSuccess {
  valid: true;
  /** Ordered reusable sections (contiguous prefix of successful outcomes). */
  reusedSections: ReusedSection[];
  /**
   * The run directory that was validated.
   * New run MUST write all outputs to a new unique directory.
   */
  priorRunDir: string;
  /**
   * Human-readable provenance summary for status.json in the new run.
   */
  provenance: string;
}

/**
 * Outcome when validation fails — no sections can be reused.
 */
export interface ResumeValidationFailure {
  valid: false;
  reason: string;
}

export type ResumeValidationResult = ResumeValidationSuccess | ResumeValidationFailure;

// ─── Implementation ───────────────────────────────────────────────────

function sha256(text: string): string {
  return createHash('sha256').update(text, 'utf8').digest('hex');
}

/**
 * Validate a prior run directory for safe resumption.
 *
 * Steps:
 *  1. Read and parse inputs.json; verify all identity fields match current run.
 *  2. Read the ordered pass_plan section IDs provided by the caller.
 *  3. For each section (in order), attempt to read {sectionId}.receipt.json.
 *     - If file missing: stop (no more reusable sections past this point).
 *     - If outcome === 'failed': stop (never reuse; report truncation point).
 *     - If jev?.blocked === true: stop (Jev blocked — never reuse).
 *     - Otherwise: validate internal hashes, persona provenance, source consistency.
 *  4. Return the contiguous prefix of validated sections.
 *
 * @param priorRunDir  Absolute path to the prior run directory (--resume-from value).
 * @param sectionIds   Ordered section IDs from the current run's pass_plan.
 * @param current      Current run identity for comparison.
 */
export async function validateResumeDir(
  priorRunDir: string,
  sectionIds: string[],
  current: CurrentRunIdentity,
): Promise<ResumeValidationResult> {
  // ── 1. Load and verify inputs.json ───────────────────────────────────
  let prior: RunInputsIdentity;
  try {
    const raw = await fs.readFile(join(priorRunDir, 'inputs.json'), 'utf8');
    prior = JSON.parse(raw) as RunInputsIdentity;
  } catch (err) {
    return {
      valid: false,
      reason: `resume: cannot read prior inputs.json: ${err instanceof Error ? err.message : String(err)}`,
    };
  }

  const identityErrors: string[] = [];
  if (!prior || typeof prior !== 'object' || typeof prior.inputHash !== 'string' ||
      !Array.isArray(prior.personaHashes) || prior.personaHashes.some(h => typeof h !== 'string') ||
      !prior.runtimeHashes || Object.values(prior.runtimeHashes).some(h => typeof h !== 'string')) {
    return { valid: false, reason: 'resume: malformed input identity' };
  }

  if (prior.subject !== current.subject) {
    identityErrors.push(`subject mismatch: prior=${prior.subject} current=${current.subject}`);
  }
  if (prior.language !== current.language) {
    identityErrors.push(`language mismatch: prior=${prior.language} current=${current.language}`);
  }
  if (prior.inputHash !== current.inputHash) {
    identityErrors.push(`inputHash mismatch: prior=${prior.inputHash.slice(0, 12)} current=${current.inputHash.slice(0, 12)}`);
  }
  if (prior.accountId !== current.accountId) {
    identityErrors.push(`accountId mismatch: prior=${prior.accountId} current=${current.accountId}`);
  }
  if (prior.index !== current.index) {
    identityErrors.push(`corpus index mismatch: prior=${prior.index} current=${current.index}`);
  }

  // Persona hashes: prior stores [aletheiosHash, pichetHash]
  if (!Array.isArray(prior.personaHashes) || prior.personaHashes.length < 2) {
    identityErrors.push('prior inputs.json missing personaHashes array');
  } else {
    if (prior.personaHashes[0] !== current.aletheiosPersonaHash) {
      identityErrors.push(
        `aletheios persona hash mismatch: prior=${prior.personaHashes[0].slice(0, 12)} current=${current.aletheiosPersonaHash.slice(0, 12)}`,
      );
    }
    if (prior.personaHashes[1] !== current.pichetPersonaHash) {
      identityErrors.push(
        `pichet persona hash mismatch: prior=${prior.personaHashes[1].slice(0, 12)} current=${current.pichetPersonaHash.slice(0, 12)}`,
      );
    }
  }

  // Runtime contract hashes: every key in the prior run must exist and match current
  if (!prior.runtimeHashes || typeof prior.runtimeHashes !== 'object') {
    identityErrors.push('prior inputs.json missing runtimeHashes object');
  } else {
    for (const [path, priorHash] of Object.entries(prior.runtimeHashes)) {
      const currentHash = current.runtimeHashes[path];
      if (currentHash === undefined) {
        identityErrors.push(`runtimeHashes: key "${path}" present in prior but missing in current — runtime contract incompatible`);
      } else if (priorHash !== currentHash) {
        identityErrors.push(`runtimeHashes: "${path}" changed: prior=${priorHash.slice(0, 12)} current=${currentHash.slice(0, 12)}`);
      }
    }
    // If current has keys the prior lacks, those are new files — not a rejection,
    // but worth noting (new code may change behaviour for un-covered sections).
    for (const path of Object.keys(current.runtimeHashes)) {
      if (!(path in prior.runtimeHashes)) {
        identityErrors.push(`runtimeHashes: key "${path}" present in current but absent in prior — runtime contract expanded; cannot safely reuse prefix`);
      }
    }
  }

  if (identityErrors.length > 0) {
    return {
      valid: false,
      reason: `resume: identity verification failed:\n  ${identityErrors.join('\n  ')}`,
    };
  }

  // ── 2. Walk section IDs in order; collect contiguous successful prefix ─
  const reusedSections: ReusedSection[] = [];
  let truncationReason = '';

  for (const sectionId of sectionIds) {
    const receiptPath = join(priorRunDir, `${sectionId}.receipt.json`);
    let receipt: SectionExecutionReceipt;

    try {
      const raw = await fs.readFile(receiptPath, 'utf8');
      receipt = JSON.parse(raw) as SectionExecutionReceipt;
    } catch (error) {
      if ((error as NodeJS.ErrnoException).code !== 'ENOENT') return { valid: false, reason: `resume: malformed or unreadable receipt for ${sectionId}` };
      // Missing receipt — stop here (no more sections to reuse)
      truncationReason = `section "${sectionId}" receipt not found in prior run — prefix ends here`;
      break;
    }

    if (!receipt || typeof receipt !== 'object') return { valid: false, reason: `resume: malformed receipt for ${sectionId}` };
    // ── 2a. Reject failed outcomes ────────────────────────────────────
    if (receipt.outcome === 'failed') {
      truncationReason = `section "${sectionId}" has outcome=failed in prior run — prefix ends before this section`;
      break;
    }

    if (receipt.outcome !== 'ok' && receipt.outcome !== 'repaired') {
      return { valid: false, reason: `resume: unknown outcome for ${sectionId}` };
    }

    // ── 2b. Reject Jev-blocked receipts ───────────────────────────────
    if (receipt.jev?.blocked === true) {
      truncationReason = `section "${sectionId}" is Jev-blocked in prior run — prefix ends before this section`;
      break;
    }
    if (!receipt.raw || !receipt.retrieval || !Array.isArray(receipt.source_ids) ||
        !receipt.source_ids.length || !Array.isArray(receipt.source_hashes) ||
        typeof receipt.output_hash !== 'string' || typeof receipt.passages_hash !== 'string') {
      return { valid: false, reason: `resume: malformed receipt fields for ${sectionId}` };
    }

    // ── 2c. Verify section_id matches ────────────────────────────────
    if (receipt.section_id !== sectionId) {
      return {
        valid: false,
        reason: `resume: receipt file "${sectionId}.receipt.json" has section_id="${receipt.section_id}" — malformed receipt; rejecting entire prior run`,
      };
    }

    // ── 2d. Verify output hash ────────────────────────────────────────
    const rawSynthesis = receipt.raw?.synthesis;
    if (!rawSynthesis || rawSynthesis.length < 100) {
      return {
        valid: false,
        reason: `resume: section "${sectionId}" receipt raw.synthesis is absent or too short — malformed receipt; rejecting entire prior run`,
      };
    }
    const recomputedOutputHash = sha256(rawSynthesis);
    if (recomputedOutputHash !== receipt.output_hash) {
      return {
        valid: false,
        reason: `resume: section "${sectionId}" output_hash mismatch: stored=${receipt.output_hash.slice(0, 12)} recomputed=${recomputedOutputHash.slice(0, 12)} — receipt tampered or corrupted; rejecting entire prior run`,
      };
    }

    // ── 2e. Verify aletheios/pichet hash fields are non-empty ─────────
    if (!receipt.aletheios_hash || receipt.aletheios_hash.length !== 64) {
      return {
        valid: false,
        reason: `resume: section "${sectionId}" aletheios_hash is missing or malformed — rejecting entire prior run`,
      };
    }
    if (!receipt.pichet_hash || receipt.pichet_hash.length !== 64) {
      return {
        valid: false,
        reason: `resume: section "${sectionId}" pichet_hash is missing or malformed — rejecting entire prior run`,
      };
    }

    // ── 2f. Verify voice outputs hash correctly ───────────────────────
    const rawAletheios = receipt.raw?.aletheios;
    const rawPichet = receipt.raw?.pichet;
    if (!rawAletheios || !rawPichet) {
      return {
        valid: false,
        reason: `resume: section "${sectionId}" missing raw voice outputs — rejecting entire prior run`,
      };
    }
    if (sha256(rawAletheios) !== receipt.aletheios_hash) {
      return {
        valid: false,
        reason: `resume: section "${sectionId}" aletheios_hash does not match raw.aletheios — receipt tampered; rejecting entire prior run`,
      };
    }
    if (sha256(rawPichet) !== receipt.pichet_hash) {
      return {
        valid: false,
        reason: `resume: section "${sectionId}" pichet_hash does not match raw.pichet — receipt tampered; rejecting entire prior run`,
      };
    }

    // ── 2g. Verify synthesis_input_hash ──────────────────────────────
    if (!receipt.synthesis_input_text || !receipt.synthesis_input_hash) return { valid: false, reason: `resume: missing synthesis input for ${sectionId}` };
    if (receipt.synthesis_input_text && receipt.synthesis_input_hash) {
      const recomputedSynthesisHash = sha256(receipt.synthesis_input_text);
      if (recomputedSynthesisHash !== receipt.synthesis_input_hash) {
        return {
          valid: false,
          reason: `resume: section "${sectionId}" synthesis_input_hash does not match synthesis_input_text — receipt tampered; rejecting entire prior run`,
        };
      }
    }

    // ── 2h. Verify persona provenance in receipt matches current personas ─
    if (receipt.aletheios_persona?.sourceHash !== current.aletheiosPersonaHash) {
      return {
        valid: false,
        reason: `resume: section "${sectionId}" aletheios_persona.sourceHash=${receipt.aletheios_persona?.sourceHash?.slice(0, 12)} does not match current aletheios hash=${current.aletheiosPersonaHash.slice(0, 12)} — persona changed; rejecting entire prior run`,
      };
    }
    if (receipt.pichet_persona?.sourceHash !== current.pichetPersonaHash) {
      return {
        valid: false,
        reason: `resume: section "${sectionId}" pichet_persona.sourceHash=${receipt.pichet_persona?.sourceHash?.slice(0, 12)} does not match current pichet hash=${current.pichetPersonaHash.slice(0, 12)} — persona changed; rejecting entire prior run`,
      };
    }

    // ── 2i. Verify retrieval receipt fields are non-empty ─────────────
    if (receipt.retrieval.state !== 'success' || !receipt.retrieval.passages_hash || receipt.retrieval.count === 0) {
      return {
        valid: false,
        reason: `resume: section "${sectionId}" retrieval receipt state=${receipt.retrieval.state} or empty passages — malformed receipt; rejecting entire prior run`,
      };
    }

    // ── 2j. Verify source_hashes length matches source_ids ────────────
    if (!Array.isArray(receipt.source_hashes) || receipt.source_hashes.length !== receipt.source_ids.length) {
      return {
        valid: false,
        reason: `resume: section "${sectionId}" source_hashes length (${receipt.source_hashes?.length}) does not match source_ids length (${receipt.source_ids?.length}) — malformed receipt; rejecting entire prior run`,
      };
    }

    // ── 2k. Verify passages_hash consistency with source_hashes ──────
    // The passages_hash stored in the retrieval receipt is sha256 of the joined passage texts.
    // We cannot re-retrieve the passages here (no CF call), but we can verify that
    // the receipt-level passages_hash matches the top-level passages_hash field,
    // and that both are non-empty hex strings of the right length.
    if (receipt.passages_hash.length !== 64 || receipt.retrieval.passages_hash.length !== 64) {
      return {
        valid: false,
        reason: `resume: section "${sectionId}" passages_hash is not a valid sha256 hex string — malformed receipt; rejecting entire prior run`,
      };
    }
    if (receipt.passages_hash !== receipt.retrieval.passages_hash) {
      return {
        valid: false,
        reason: `resume: section "${sectionId}" receipt.passages_hash !== receipt.retrieval.passages_hash — internal inconsistency; rejecting entire prior run`,
      };
    }

    if (receipt.retrieval.section_id !== sectionId || receipt.retrieval.count !== receipt.source_ids.length || JSON.stringify(receipt.retrieval.source_ids) !== JSON.stringify(receipt.source_ids)) return { valid: false, reason: `resume: retrieval identity mismatch for ${sectionId}` };
    const priorPrimaryIds = Array.isArray(receipt.primary_source_ids) ? receipt.primary_source_ids : [];
    // A prior run with primary_source_ids can only be safely resumed
    // when the current run supplies the SAME primary registry (sourceTexts
    // for every prior primary id + text hashes match). We do not attempt
    // to auto-load the registry here — resume-validation is a pure local
    // check; the caller must pre-load and merge primary texts into
    // `current.sourceTexts` before calling us. Missing text = fail closed
    // with an actionable reason so the caller knows to pass the registry.
    const texts = receipt.source_ids.map(id => current.sourceTexts[id]);
    const missingTextIndex = texts.findIndex(t => !t);
    if (missingTextIndex >= 0) {
      const missingId = receipt.source_ids[missingTextIndex];
      const kindHint = priorPrimaryIds.includes(missingId)
        ? 'primary passage registry not supplied — pass --primary-registry with the same registry the prior run used'
        : 'CF passage text not in current preflight — corpus drift';
      return { valid: false, reason: `resume: source text missing for ${sectionId} id=${missingId} — ${kindHint}` };
    }
    if (texts.some((text, i) => sha256(text) !== receipt.source_hashes[i]) || sha256(texts.join('\n')) !== receipt.passages_hash) return { valid: false, reason: `resume: source text or source hash mismatch for ${sectionId}` };

    // ── Section passes all checks — add to reusable prefix ───────────
    reusedSections.push({
      section_id: sectionId,
      acceptedOutput: rawSynthesis,
      synthesis_input_text: receipt.synthesis_input_text ?? '',
      receipt,
    });
  }

  if (reusedSections.length === 0) {
    return {
      valid: false,
      reason: truncationReason
        ? `resume: no reusable sections found — ${truncationReason}`
        : 'resume: no section receipts found in prior run directory',
    };
  }

  const reusedIds = reusedSections.map((s) => s.section_id);
  const provenance = [
    `Resumed from prior run: ${priorRunDir}`,
    `Reused sections (${reusedIds.length}): ${reusedIds.join(', ')}`,
    truncationReason ? `Prefix ended at: ${truncationReason}` : '',
    `Prior run identity verified: subject=${prior.subject}, language=${prior.language}, inputHash=${prior.inputHash.slice(0, 12)}`,
  ].filter(Boolean).join('\n');

  return {
    valid: true,
    reusedSections,
    priorRunDir,
    provenance,
  };
}
