# Witness Report Corrective Implementation Plan

**Date:** 2026-09-30
**Status:** Plan only — no edits, no generation until chapter gate passes
**Scope:** Recover engine-fact → per-engine micro-interpretation → Aletheios/Pichet → symbolic synthesis matrix; separate private evidence from public reader prose; gate on Sheshnarayan EN Part I Convergence Map; preserve CF Vectorize binding and immutable receipts.
**Blocking rule:** Reject bulk generation until this chapter and its private evidence map both pass all acceptance probes.

---

## 1. Three-Layer Matrix Recovery

### 1.1 Current state

The existing pipeline conflates layers. `engine-facts.ts:buildEngineFactsBlock()` produces a single flat text block of summarised engine data that is injected wholesale into every pass template via `{{engine_facts}}`. The voice prompts (Aletheios + Pichet in `witness-dyad.ts`) receive this same undifferentiated block. The synthesis prompt merges both voices but never produces a separable interpretation map. The source-audit (`reference-source-audit.ts`) checks claims against the flat engine-facts string — it cannot distinguish which engine sourced which interpretation.

The handoff identifies three layers that must be structurally separated:
1. **Private engine facts** — immutable saved subject data (dates, longitudes, calculations, CF retrieval IDs, source hashes)
2. **Private interpretation** — traceable fact→tradition/interpretation mapping, contradictions, uncertainty; reviewed by witness pair + grounding
3. **Public reading** — rich symbolic meaning in natural reader language, no JSON keys, tool names, receipt timestamps, audit rubrics, or backend status

### 1.2 Recovery: the existing three-layer matrix

The three-layer matrix IS NOT a new invention. It maps to existing architecture across two repos:

| Layer | witness-agents origin | Selemene-engine current location |
|---|---|---|
| Engine facts (L0) | `engine-lexicons.md` L1-L3 vocabulary tables + `consciousness-level-register-design.md` | `engine-facts.ts` summarisers, `selemene/types.ts` engine outputs |
| Interpretation (L1) | `system-prompts.ts` Aletheios/Pichet voice split + `engine-lexicons.md` L4-L5 voice rules | `witness-dyad.ts` ALETHEIOS_VOICE / PICHET_VOICE + `reference-execution.ts` per-section pipeline |
| Public reading (L2) | `system-prompts.ts` ANATOMIST_PERSONA + SYNTHESIS_RULES | `integrated.ts` VOICE_RULES + mode doc `integrated-kundali-reference.md` bridge mandates |

The recovery task: make these structurally separate at the schema level, not just conceptually separate in prompts.

### 1.3 New schemas

#### PrivateEvidenceMap (per section, never in reader output)

```typescript
interface PrivateEvidenceMap {
  section_id: string;
  subject: string;
  language: string;
  timestamp_iso: string;

  // Layer 0: immutable engine facts consumed
  engine_facts_consumed: {
    engine_id: string;
    fields_used: string[];        // exact field paths from SelemeneEngineOutput
    values_snapshot: Record<string, unknown>;  // frozen values at generation time
    source_precision_label: string | null;
    backend: string;
  }[];

  // Layer 1: per-engine micro-interpretations
  micro_interpretations: {
    engine_id: string;
    claim: string;                // exact claim text as it appears in the public section
    source_field: string;         // e.g. "vedic-kundali.planets.Jupiter.dignity"
    source_value: string;         // e.g. "Exalted"
    interpretation_tradition: string; // e.g. "Vedic Jyotish" / "Human Design" / "Gene Keys"
    confidence: 'direct' | 'derived' | 'framework-attributed' | 'interpretive-synthesis';
    contradictions: string[];     // other engine claims that disagree
    uncertainty_note: string | null;
  }[];

  // Witness voices raw
  aletheios_raw: string;
  pichet_raw: string;

  // Grounding provenance
  cf_passage_ids: string[];
  cf_passages_hash: string;
  primary_passage_ids: string[];

  // Receipt binding
  receipt_output_hash: string;
  engine_facts_hash: string;      // sha256 of the engine_facts string
  audit_input_hash: string;       // from computeAuditInputHash
}
```

#### PublicSectionOutput (reader-facing, no private data)

```typescript
interface PublicSectionOutput {
  section_id: string;
  title: string;
  content: string;               // markdown prose — NO JSON keys, NO tool names, NO receipt data
  word_count: number;
  subsection_ids: string[];      // e.g. ["1.1", "1.2", "1.3"]
}
```

#### SectionGenerationEnvelope (binds both)

```typescript
interface SectionGenerationEnvelope {
  public: PublicSectionOutput;
  private: PrivateEvidenceMap;
  receipt: SectionExecutionReceipt;  // existing immutable receipt
  leakage_gate: LeakageGateResult;
}
```

---

## 2. Prompt and Grader Changes

### 2.1 Engine-facts prompt reform (engine-facts.ts)

**Current:** `buildEngineFactsBlock()` produces a single string block injected into the pass template. Both voices and synthesis see the same undifferentiated facts.

**Change:** Add `buildPerEngineMicroInterpretationPrompt(engine_id, register, subject)` that:
- Extracts only the fields relevant to the current section's topic (from the mode doc's `engine_overlay_weights` + section-specific engine list)
- Renders at the register band determined by consciousness level (L1-L3 traditional / L4-L5 framework)
- Produces a structured mini-prompt that Aletheios can map structurally and Pichet can reflect experientially
- Outputs a companion `FieldsConsumed` manifest for the private evidence map

The existing `buildEngineFactsBlock()` remains for backwards compatibility. New reference-mode sections call the per-engine variant.

### 2.2 Voice prompt separation (witness-dyad.ts)

**Current:** Both voices get identical system prompts with the full engine facts block appended.

**Change:**
- Aletheios prompt receives: structural facts + micro-interpretation extraction instruction. Must output a structured claim list alongside prose. Each claim must cite `engine_id.field_path = value`.
- Pichet prompt receives: Aletheios's structural output + the same facts. Must ground experiential reflections in Aletheios's cited facts only.
- Synthesis prompt receives: both outputs + instruction to produce ONLY reader-facing prose. Explicit prohibition: no `engine_id`, no `field_path`, no `sw:` prefixes, no `receipt`, no `source_audit`, no `backend`, no `precision_achieved`, no `Jev`.

### 2.3 Source-audit prompt reform (reference-source-audit.ts)

**Current:** `buildAuditorSystemPrompt()` asks the auditor to check claims against engine facts and passages.

**Change:** The auditor additionally receives the `PrivateEvidenceMap.micro_interpretations` array and must verify:
1. Every public claim traces to at least one micro-interpretation entry
2. Every micro-interpretation entry traces to an engine fact that was actually supplied
3. No public claim adds information not present in any micro-interpretation
4. The confidence label is appropriate (direct engine value vs. framework attribution vs. interpretive synthesis)

The auditor JSON schema gains a `micro_interpretation_coverage` field.

### 2.4 Leakage gate (NEW: leakage-gate.ts)

A pure, deterministic, no-LLM function that scans public section output for leaked private data:

```typescript
interface LeakageGateResult {
  passed: boolean;
  violations: LeakageViolation[];
}

interface LeakageViolation {
  pattern_id: string;           // e.g. 'json_key', 'receipt_field', 'tool_name'
  matched_text: string;
  line_number: number;
  severity: 'block' | 'warn';
}
```

**Detection patterns (block severity):**
- JSON object/array syntax in prose: `/{[^}]{20,}}/`, `/\[[^\]]{20,}\]/`
- Receipt/audit field names: `output_hash`, `source_audit`, `passages_hash`, `engine_facts_hash`, `synthesis_input_hash`, `receipt`, `attempt`, `checkpoint`
- Tool/backend names: `sw:`, `native-rust`, `swiss-ephemeris`, `pyswisseph`, `humdes-authenticated-source`
- Pipeline internal references: `Jev`, `Aletheios`, `Pichet`, `witness-dyad`, `pass-gate`, `rubric`, `mode_matrix`, `partitioned-client`
- Prompt ordinals: `Quoted Passage 1`, `Passage 2`, `Retrieved passage`
- Timestamps in receipt format: ISO 8601 with milliseconds in non-timeline context
- Acceptance/review status labels: `finalAccepted`, `blocked`, `judged`, `shadow`

**Detection patterns (warn severity):**
- `backend` / `precision_achieved` / `precision label` in prose
- Bare `engine_id` hyphenated identifiers: `vedic-kundali`, `gene-keys`, `human-design` (allowed as readable names: "Vedic Kundali", "Gene Keys", "Human Design")
- `consciousness_level` / `register` / `L1-L3` / `L4-L5`

### 2.5 Rubric extension (rubric.ts)

Add a `leakage_gate` dimension to `SectionRubric`:

```typescript
leakage_gate: 'pass' | 'fail';
leakage_violations: string[];
```

`final-verification.ts` enforces `leakage_gate === 'pass'` for all reference-route sections.

---

## 3. Representative Chapter Gate: Sheshnarayan EN Part I Convergence Map

### 3.1 Why this chapter

- Part I is the structural foundation — it establishes the identity stack, cross-system agreements, and disagreements
- The existing Shesh EN Part I output (in `coherent-candidate-v2`) demonstrates both strengths (rigorous provenance, explicit gap marking) and the core leakage problem (engine IDs like `sw:hd:type:generator:desc` in reader text, "per sw:hd:ctr:sacral:desc" citations, raw field paths)
- Receipt shows `outcome: failed`, `source_audit: insufficient` — the audit couldn't validate claims against the flat facts block
- Part I receipt has 18 CF passage IDs — the richest retrieval set, good for testing CF binding preservation
- All three subjects share the same Part I template, so fixes proven here apply to Gary and Mohan

### 3.2 Gate criteria

The chapter gate passes ONLY when ALL of the following are true:

| # | Criterion | Probe |
|---|---|---|
| G1 | Public output contains zero `block`-severity leakage violations | `leakageGate(publicOutput).passed === true` |
| G2 | Public output contains zero `warn`-severity leakage violations (or each is explicitly acknowledged) | `leakageGate(publicOutput).violations.filter(v => v.severity === 'warn').length === 0` |
| G3 | Private evidence map has ≥1 micro-interpretation per engine that contributed facts to this section | `privateMap.micro_interpretations.map(m => m.engine_id)` covers all engines in `engine_facts_consumed` |
| G4 | Every public factual claim traces to a micro-interpretation entry | Source audit `micro_interpretation_coverage.uncovered_claims === 0` |
| G5 | No micro-interpretation cites an engine field not present in the consumed snapshot | Source audit `micro_interpretation_coverage.orphaned_interpretations === 0` |
| G6 | CF passage binding preserved: all `sw:` IDs from original retrieval present in evidence map | `privateMap.cf_passage_ids` matches receipt retrieval `source_ids` |
| G7 | Immutable receipt unchanged: original `part1.receipt.json` byte-hash matches | `sha256(receiptFileBytes) === evidence.baseReceiptSha256` |
| G8 | Jev judgment not blocked, no `fail` verdicts | `jev.blocked === false && Object.values(jev.verdicts).every(v => v !== 'fail')` |
| G9 | Source audit status `clean` with `total_claims_audited > 0` | `audit.status === 'clean' && audit.coverage.total_claims_audited > 0` |
| G10 | Word count within mode target (Part I: 1200 ± 25%) | `rubric.word_count_fit !== 'fail'` |
| G11 | Reader can understand the text without any knowledge of pipeline internals | Manual human review criterion (not automatable) |
| G12 | Source corrections survive: no Rao house-meaning arithmetic, no unsupported Darakaraka, no invented sensitivity window | Specific claim checks in source audit findings |

### 3.3 Acceptance sequence

1. Generate Part I with reformed prompts → produces `SectionGenerationEnvelope`
2. Run leakage gate → must pass (G1, G2)
3. Run source audit with micro-interpretation coverage → must pass (G4, G5, G9)
4. Run Jev judgment → must pass (G8)
5. Validate CF binding preservation → must pass (G6)
6. Validate receipt immutability → must pass (G7)
7. Run rubric → must pass (G10)
8. Verify private evidence map completeness → must pass (G3)
9. Human review of public output → must pass (G11, G12)

**Only after this sequence passes do we proceed to any other section or subject.**

---

## 4. CF Vectorize Passage Binding Preservation

### 4.1 Current contract

- Index: `witness-wisdom-corpus`
- Retrieval: exact-ID read via `sw:` prefixed corpus document IDs
- No deployment or CF mutation in this work
- Primary passage registry: `editorial-primary-only-registry.json`
- Source partition: `source-partition.ts` validates CF vs primary splits

### 4.2 Required invariants

- All existing `sw:` passage IDs from prior receipts remain valid CF index references
- No new CF insert, update, or delete operations
- `PassageRetrievalReceipt.source_ids` continues to carry exact `sw:` IDs
- `primary_source_ids` partition remains separately tracked
- `passages_hash` continues to be sha256 of joined passage texts
- `source-partition.ts:validateSourcePartition()` passes on every new receipt
- CF readback for editorial evidence uses the same index and account binding

### 4.3 Changes required

None to CF infrastructure. The only change is that `sw:` IDs move from public output prose into `PrivateEvidenceMap.cf_passage_ids`. The reader never sees `sw:hd:type:generator:desc`; the private map traces exactly which passage supported which claim.

---

## 5. Immutable Receipt Preservation

### 5.1 Contract

- Existing `SectionExecutionReceipt` files on disk are byte-for-byte immutable
- `editorial-verification.ts` requires `baseReceiptRawBytes` + recomputed sha256
- `editorial-prefix.ts` re-applies patches against original receipt, never mutates
- `final-verification.ts` recomputes audit input hash from receipt + passages + engine facts

### 5.2 Changes required

- New fields added to `SectionExecutionReceipt` interface (additive, non-breaking):
  - `private_evidence_map_hash?: string` — sha256 of the companion PrivateEvidenceMap JSON
  - `leakage_gate?: LeakageGateResult` — deterministic gate result
- Existing saved receipts with `undefined` for these fields remain valid
- The `SectionGenerationEnvelope` wraps the receipt without modifying it

---

## 6. Jev Triad Partitioned Review Repair

### 6.1 Current state

- `partitioned-client.ts` exists but is UNVERIFIED (handoff note: "interrupted worker, no completion or integration claim")
- Full-payload triad judge hit `max_tokens_exceeded` with 93,175-char engine facts
- Conservative aggregation: worst-case wins across batches

### 6.2 Required repair (deferred until chapter gate passes)

- Verify `partitioned-client.ts` unit tests pass with current codebase
- Integration test: feed the actual Shesh+Gary+Mohan triad engine facts through the partitioner
- Verify batch sizes stay under `maxRequestChars` (default 60,000)
- Verify conservative aggregation produces the expected worst-case result
- Verify coverage index accounts for all three subjects × all engines
- Verify `PartitionEvidence` sidecar is complete and hashable

**This work is explicitly deferred. Do not attempt triad generation until the individual chapter gate passes.**

---

## 7. Test Plan

### 7.1 New unit tests

| File | Tests |
|---|---|
| `leakage-gate.test.ts` | Block patterns: JSON in prose, receipt fields, tool names, `sw:` prefixes, prompt ordinals, pipeline internals. Warn patterns: bare engine IDs, register labels. Clean prose passes. Edge cases: "Human Design" (allowed) vs `human-design` (warn). |
| `engine-facts.test.ts` (extend) | `buildPerEngineMicroInterpretationPrompt()` produces per-engine output. `FieldsConsumed` manifest matches input. Empty engine → empty output. |
| `witness-dyad.test.ts` (extend) | Aletheios output contains structured claim list. Pichet output references Aletheios claims. Synthesis output passes leakage gate. |
| `reference-source-audit.test.ts` (extend) | Auditor receives micro-interpretation array. Coverage check validates trace-to-source. Orphaned interpretation → `changes-required`. |
| `private-evidence-map.test.ts` (new) | Schema validation. Hash binding to receipt. CF ID preservation. Micro-interpretation completeness against engine facts consumed. |

### 7.2 Integration tests

| Scope | What it validates |
|---|---|
| Single section (Part I, Shesh EN) | Full envelope generation → leakage gate → source audit → Jev → CF binding → receipt immutability |
| Editorial prefix continuation | Revised Part I loads correctly as prefix for Part II generation |
| Source partition | `validateSourcePartition()` passes on new receipts with PrivateEvidenceMap companion |

### 7.3 Acceptance probes (manual + scripted)

| Probe | Method |
|---|---|
| Reader comprehension | Human reads public Part I output without access to private map; confirms understanding |
| Evidence traceability | Human reads private map; confirms every public claim has a traceable source |
| Leakage spot-check | `grep -E 'sw:|output_hash|source_audit|Jev|Aletheios|Pichet|partitioned-client' public_output.md` returns empty |
| Source correction survival | Verify no Rao house-meaning arithmetic, no unsupported Darakaraka, no invented birth-time sensitivity window in output |
| CF binding verification | All `sw:` IDs in private map exist in the CF index (read-only check, no mutation) |

---

## 8. Implementation Sequence

### Phase A: Schema + Leakage Gate (no generation)

1. Define `PrivateEvidenceMap`, `PublicSectionOutput`, `SectionGenerationEnvelope`, `LeakageGateResult` types in a new `packages/witness-pipeline/src/orchestrator/evidence-map.ts`
2. Implement `leakageGate()` in new `packages/witness-pipeline/src/orchestrator/leakage-gate.ts`
3. Write `leakage-gate.test.ts` — all pattern tests must pass
4. Add `leakage_gate` dimension to `SectionRubric` in `rubric.ts`
5. Add `leakage_gate` enforcement to `final-verification.ts`

### Phase B: Prompt Reform (no generation)

6. Add `buildPerEngineMicroInterpretationPrompt()` to `engine-facts.ts`
7. Extend `witness-dyad.ts` voice prompts for structured claim extraction (Aletheios) and claim-grounded reflection (Pichet)
8. Update synthesis voice to prohibit private data in output
9. Extend source-audit prompt to accept micro-interpretation array
10. Write/extend unit tests for all prompt changes

### Phase C: Chapter Gate — Sheshnarayan EN Part I (first generation)

11. Generate Part I with reformed pipeline → `SectionGenerationEnvelope`
12. Run acceptance sequence (§3.3 above): leakage → audit → Jev → CF → receipt → rubric → evidence map
13. Human review of public output
14. If any gate fails: diagnose, patch prompts/graders, re-run from step 11
15. Gate passes → proceed

### Phase D: Private Evidence Map Binding (after gate)

16. Implement evidence map generation in `reference-execution.ts` alongside existing receipt
17. Implement evidence map validation in `final-verification.ts`
18. Write `private-evidence-map.test.ts`

### Phase E: Triad Repair (after gate)

19. Verify `partitioned-client.ts` tests
20. Integration test with actual triad payload
21. Prove conservative aggregation + exact receipt binding

### Phase F: Bulk Generation (after gate + triad repair)

22. Generate remaining 11 sections for Shesh EN
23. Generate Shesh FR (same prompts, language parameter)
24. Generate Gary EN/FR, Mohan EN/FR
25. Generate triad EN/FR
26. Verify language parity, dates, grounding, witness participation, coherence, DOCX/PDF visual quality
27. Keep provenance sidecars separate from deliverables

---

## 9. Explicit Rejections

- **No bulk generation before the Shesh EN Part I gate passes.** The corrective handoff is clear: previous technical acceptance was insufficient. All eight readings require the corrected contract.
- **No GSD phase fabrication.** This is freeform Algorithm continuity per the handoff instruction.
- **No commit, push, deploy, merge, or live infrastructure activation.** Per handoff.
- **No CF mutation.** Read-only retrieval against existing `witness-wisdom-corpus` index.
- **No Cortex/pattern-memory/event-listener connection.** Disclose internally, not in reader frontmatter.
- **No resume of old generation jobs.** PIDs 560, 95340, 8296 and descendants were terminated.

---

## 10. Source Corrections That Must Survive

These are carried verbatim from the handoff and must be enforced in the source-audit grader:

1. Rao house 11 includes older siblings — not a general "siblings" house
2. House 5 support does not automatically authorize creativity; house 6 does not automatically authorize illness/debt; house 9 does not automatically authorize father
3. Traditional rulership must have textual support, not arithmetic
4. Mohan Ketu PD duration 24.857 days is total duration, not days remaining; end 2026-10-20T15:32:40Z, Venus AD end one second later
5. Gary Sun PD start 2027-06-05 has no supplied end; do not borrow Mars AD end 2027-07-31
6. Gary birth time confidence approximate — no invented sensitivity window
7. No unsupported Darakaraka ranking, bodily metrics, or literal psychological certainty from symbolic labels
8. Legacy reference facts/dates/health claims are not authoritative sources for current readings

These become explicit `source_audit_finding` rules in the auditor system prompt.

---

## 11. Risk Register

| Risk | Mitigation |
|---|---|
| Micro-interpretation extraction adds latency per section | Per-engine extraction is one additional structured LLM call per section; budget ~20s. Measure on Part I. |
| Leakage gate false positives on legitimate chart vocabulary | Allowlist: "Human Design", "Gene Keys", "Vedic", "Vimshottari" as readable names. Reject hyphenated engine IDs. |
| Source audit with micro-interpretation array exceeds context window | Keep micro-interpretation array to ≤30 entries per section; summarise if needed. Measure on Part I. |
| Partitioned Jev client fails on actual triad payload | Deferred until Phase E. Part I gate doesn't depend on triad. |
| Human review of Part I blocks on subjective quality | Define G11 criteria upfront: "reader can understand without pipeline knowledge" is binary. |

