// ─── Private Evidence Map + Public Section + Envelope schemas ────────
// Introduces the three-layer boundary that the corrective plan requires:
//   L0  private engine facts consumed        (immutable field snapshots)
//   L1  private per-engine micro-interpretations
//   L2  public reader-facing section output  (no private data)
//
// The envelope binds all three plus the existing immutable
// SectionExecutionReceipt and the deterministic LeakageGateResult.
//
// This module is ADDITIVE. No existing type is renamed or removed.
// Historical receipt bytes remain untouched: envelope fields live on the
// companion sidecar, and the receipt only gains OPTIONAL `private_evidence_map_hash`
// and `leakage_gate` fields (see reference-execution.ts).
//
// The receipt itself is NOT redefined here — it is imported by-name so
// downstream code can build an envelope without a cyclic dependency at type
// resolution time.

import type { SectionExecutionReceipt } from './reference-execution.js';
import type { LeakageGateResult } from './leakage-gate.js';

export type MicroInterpretationConfidence =
  | 'direct'                    // exact engine value stated as-is
  | 'derived'                   // arithmetic from supplied fields (e.g. whole-sign house table)
  | 'framework-attributed'      // tradition/passage attributes meaning to the value
  | 'interpretive-synthesis';   // cross-engine reflection acknowledged as synthesis

export interface EngineFactsConsumed {
  engine_id: string;
  /** Exact field paths from the SelemeneEngineOutput.result tree (dotted). */
  fields_used: string[];
  /** Frozen snapshot of the values referenced by fields_used at generation time. */
  values_snapshot: Record<string, unknown>;
  /** metadata.precision_achieved label (or null when not supplied). */
  source_precision_label: string | null;
  /** metadata.backend identifier as reported by the engine at generation time. */
  backend: string;
}

export interface MicroInterpretation {
  engine_id: string;
  /** Exact text of the reader-facing claim this interpretation supports. */
  claim: string;
  /** Dotted field path within the engine result (e.g. 'planets.Jupiter.dignity'). */
  source_field: string;
  /** Stringified engine value (e.g. 'Exalted' or '15.25'). */
  source_value: string;
  /** Readable tradition label used INTERNALLY (e.g. 'Vedic Jyotish', 'Human Design'). */
  interpretation_tradition: string;
  confidence: MicroInterpretationConfidence;
  /** Other engine claims that disagree (private only). */
  contradictions: string[];
  /** Concise internal note; may be surfaced in reader prose as natural language, never as JSON. */
  uncertainty_note: string | null;
}

export interface PrivateEvidenceMap {
  section_id: string;
  subject: string;
  language: string;
  timestamp_iso: string;

  engine_facts_consumed: EngineFactsConsumed[];
  micro_interpretations: MicroInterpretation[];

  aletheios_raw: string;
  pichet_raw: string;

  /** Exact Cloudflare Vectorize passage IDs bound to the private map. */
  cf_passage_ids: string[];
  /** SHA-256 of joined CF passage texts, matching the retrieval receipt. */
  cf_passages_hash: string;
  /** Primary (reviewed-primary-document) passage IDs, separated from CF ids. */
  primary_passage_ids: string[];

  /** Receipt binding — never mutates the receipt bytes. */
  receipt_output_hash: string;
  /** SHA-256 of the engine_facts string that was fed to voices for this section. */
  engine_facts_hash: string;
  /** Canonical audit input hash (see reference-source-audit.computeAuditInputHash). */
  audit_input_hash: string;
}

export interface PublicSectionOutput {
  section_id: string;
  title: string;
  /** Markdown prose. No JSON keys, no tool names, no receipt fields. */
  content: string;
  word_count: number;
  /** e.g. ["1.1", "1.2", "1.3"]. */
  subsection_ids: string[];
}

/** Companion sidecar. The receipt bytes are NEVER modified by wrapping in an envelope. */
export interface SectionGenerationEnvelope {
  public: PublicSectionOutput;
  private: PrivateEvidenceMap;
  receipt: SectionExecutionReceipt;
  leakage_gate: LeakageGateResult;
}

/**
 * Validate structural coverage of the private map:
 *   - every micro-interpretation references an engine that appears in
 *     engine_facts_consumed
 *   - every micro-interpretation cites a field_used that appears in that
 *     engine's fields_used list
 *   - every distinct engine that contributed facts has at least one
 *     micro-interpretation (unless explicitly allow-empty)
 *
 * Returns a list of blocker strings; empty means valid.
 */
export function validatePrivateEvidenceMap(map: PrivateEvidenceMap): string[] {
  const blockers: string[] = [];
  const engineIndex = new Map<string, EngineFactsConsumed>();
  for (const e of map.engine_facts_consumed) engineIndex.set(e.engine_id, e);

  for (const [i, mi] of map.micro_interpretations.entries()) {
    const eng = engineIndex.get(mi.engine_id);
    if (!eng) {
      blockers.push(`micro_interpretations[${i}]:engine_not_in_facts_consumed:${mi.engine_id}`);
      continue;
    }
    if (mi.source_field && !eng.fields_used.includes(mi.source_field)) {
      blockers.push(`micro_interpretations[${i}]:orphaned_field:${mi.engine_id}.${mi.source_field}`);
    }
    if (!mi.claim || !mi.source_value || !mi.interpretation_tradition) {
      blockers.push(`micro_interpretations[${i}]:incomplete_entry`);
    }
  }

  const coveredEngines = new Set(map.micro_interpretations.map(m => m.engine_id));
  for (const e of map.engine_facts_consumed) {
    if (e.fields_used.length > 0 && !coveredEngines.has(e.engine_id)) {
      blockers.push(`engine_facts_consumed:no_micro_interpretation:${e.engine_id}`);
    }
  }

  return blockers;
}
