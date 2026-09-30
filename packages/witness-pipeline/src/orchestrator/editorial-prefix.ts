// ─── Editorial Draft-Prefix Loader ────────────────────────────────────
// Load a bounded, contiguous prefix of manually-corrected sections into a
// resumed generation run WITHOUT re-generating them and WITHOUT forging any
// receipt evidence. This is the "editorial revision" continuation path used
// when the opening / early parts have been repaired by human review + fresh
// independent audit + fresh Jev, and later parts must be drafted against
// the corrected prior text.
//
// Doctrine (must not weaken elsewhere):
//   * The base receipt is NEVER mutated and NEVER re-run. Its output_hash,
//     outcome, raw synthesis, and attempts stay verbatim in the reused
//     receipt handed to the orchestrator. The revised text is passed as a
//     SEPARATE `output` string alongside the receipt (integrated.ts already
//     supports this — the reused pass's `output` is written into the
//     assembled report; the receipt is checkpointed unchanged).
//   * Downstream final verification is expected to still block the
//     original-vs-editorial mismatch until a separate provenance-aware
//     acceptance path is implemented. This loader must not relabel Jev,
//     rewrite raw synthesis, or mark finalAccepted true anywhere.
//   * Revised text is recomputed here by re-running applyEditorialRevision
//     against the original receipt and the manifest patches. The stored
//     revised output and its hash in the manifest revision artifact are
//     re-verified — never trusted.
//   * The fresh audit artifact must be `kind: fresh-audit-of-editorial-revision`
//     with:
//       - sectionOutputHash === recomputed revised output hash
//       - audit.status === 'clean'
//       - audit.input_sha256 === computeAuditInputHash(revised, saved passages_hash,
//                                                     saved engineFacts from receipt)
//       - hasCleanAuditEvidence(audit, revised) === true
//       - audit.output_sha256 === sha256(audit.model_raw)  (raw clean ledger)
//     and its saved engineFacts must equal receipt.engine_facts.
//   * The fresh Jev artifact must judge the revised output (sectionOutputHash
//     match), be `status: 'judged'`, not blocked, contain no 'fail' verdicts.
//     'could-not-tell' is permitted ONLY for continued-draft generation and
//     is surfaced prominently in the returned pending ledger.
//   * Failed originals require the shared bounded classifier: source-audit
//     failure or an intact saved draft rejected during word-count repair.
//     Word-count recovery additionally needs changed text and recomputed
//     passing fresh word fit. Voice, retrieval and heading failures block.
//   * All hash checks are recomputed from bytes on disk. No network. No LLM.

import { createHash } from 'node:crypto';
import { classifyOriginalOutcome, hasRecoveredWordFit } from './editorial-verification.js';
import { promises as fs } from 'node:fs';
import { isAbsolute, join, resolve } from 'node:path';
import {
  applyEditorialRevision,
  EditorialRevisionError,
  type EditorialPatch,
  type EditorialRevision,
} from './editorial-revision.js';
import type { SectionExecutionReceipt } from './reference-execution.js';
import {
  computeAuditInputHash,
  hasCleanAuditEvidence,
  type SourceAuditReceipt,
} from './reference-source-audit.js';
import {
  validateEditorialPassageEvidence,
  EditorialPassageEvidenceError,
  type EditorialPassageAddition,
  type TrustedSourceRecord,
} from './editorial-passage-evidence.js';
import type { RetrievedPassage } from './reference-execution.js';
import { isDeepStrictEqual } from 'node:util';

// ─── Public types ─────────────────────────────────────────────────────

export interface EditorialPrefixManifestSection {
  id: string;
  /** Revision artifact path, relative to manifest.baseRunDir. */
  revisionFile: string;
  /** Fresh source-audit-of-editorial-revision sidecar path, relative to baseRunDir. */
  auditFile: string;
  /** Fresh Jev sidecar path (rubric + judged receipt), relative to baseRunDir. */
  jevFile: string;
}

export interface EditorialPrefixManifest {
  /** Absolute or manifest-relative directory holding original receipts + sidecars. */
  baseRunDir: string;
  /** Contiguous prefix of section IDs, ordered against mode pass_plan. */
  sections: EditorialPrefixManifestSection[];
}

export interface EditorialPrefixIdentity {
  /** SHA-256 of the current input.snapshot text. */
  inputHash: string;
  subject: string;
  language: string;
  accountId: string;
  index: string;
  aletheiosPersonaHash: string;
  pichetPersonaHash: string;
  /** Freshly retrieved framework text, keyed by corpus ID. */
  sourceTexts: Record<string, string>;
  /** Hash of independently validated generation primary registry bytes. */
  primaryRegistrySha256?: string;
  /** Explicit current local implementation evidence, never arbitrary paths from a review. */
  implementationSourceTexts?: Record<string, string>;
  /**
   * Independently trusted reviewed-source registry, keyed by addition id.
   * Consumed ONLY when a section's `freshAudit.passageEvidence` is
   * present.  Callers derive this from an approved reviewer registry or
   * fresh CF readback ledger — NEVER from the evidence sidecar the
   * audit records; the sidecar can never whitelist its own additions.
   * When absent, sections carrying passage evidence are rejected with
   * an explicit untrusted-source failure — silence never admits drift.
   */
  trustedPassageSources?: Record<string, TrustedSourceRecord>;
  /**
   * Independently trusted CF target the reviewer readback targeted
   * (typically inputs.json accountId + index).  Required whenever any
   * addition carries cf-vectorize provenance.  Editorial-prefix
   * validation rejects cf additions whose account/index differ.
   */
  expectedCF?: { account: string; index: string };
}

/**
 * Ambiguity note surfaced per section when Jev returned any 'could-not-tell'
 * verdict. Permitted only because this route is generation-only and the
 * separate provenance-aware acceptance path will re-evaluate them.
 */
export interface EditorialPrefixPending {
  section_id: string;
  ambiguousVerdicts: string[];
  jevArtifactPath: string;
}

export interface EditorialPrefixSection {
  section_id: string;
  /** Original receipt, byte-for-byte from disk. Never mutated. */
  originalReceipt: SectionExecutionReceipt;
  /** Immutable revision artifact re-verified via re-application. */
  revision: EditorialRevision;
  /** Recomputed revised text (used as the section output for continuation). */
  revisedOutput: string;
  /** Recomputed sha256 of revisedOutput. */
  revisedOutputHash: string;
  /** Fresh independent audit sidecar (kind: fresh-audit-of-editorial-revision). */
  freshAudit: FreshAuditSidecar;
  /** Fresh independent Jev sidecar. */
  freshJev: FreshJevSidecar;
  /** Resolved absolute paths for provenance. */
  paths: {
    receipt: string;
    revision: string;
    audit: string;
    jev: string;
  };
}

export interface FreshAuditSidecar {
  kind: 'fresh-audit-of-editorial-revision';
  baseReceiptSha256: string;
  sectionOutputHash: string;
  originalOutcome: 'ok' | 'repaired' | 'failed';
  revisionVersion?: string;
  supplementalEvidence?: unknown;
  engineFacts: string;
  instruction?: string;
  audit: SourceAuditReceipt;
  llmReceipts?: unknown;
  /**
   * Optional editorial passage-evidence bundle recorded by the reviewer
   * CLI.  Present only when the fresh audit was supplemented with fresh
   * additions.  Semantics identical to
   * FreshAuditSidecar.passageEvidence in editorial-verification.ts:
   * loader revalidates it against caller-supplied trusted records + CF
   * target and against the base receipt.  Absent-evidence receipts
   * preserve existing base-only behaviour.
   */
  passageEvidence?: {
    basePassages: RetrievedPassage[];
    basePassagesHash: string;
    additions: EditorialPassageAddition[];
    addedSourceRecords: EditorialPassageAddition[];
    effectivePassagesHash: string;
  };
  finalAccepted: boolean;
}

export interface FreshJevSidecar {
  sectionOutputHash: string;
  revisionVersion?: string;
  rubric: Record<string, unknown>;
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

export interface EditorialPrefixSuccess {
  valid: true;
  manifestPath: string;
  baseRunDir: string;
  prefixSections: EditorialPrefixSection[];
  /** Sections with permitted ambiguity flags that a downstream acceptance path must judge. */
  pendingAmbiguity: EditorialPrefixPending[];
  provenance: string;
}

export interface EditorialPrefixFailure {
  valid: false;
  reason: string;
}

export type EditorialPrefixResult = EditorialPrefixSuccess | EditorialPrefixFailure;

// ─── Implementation ───────────────────────────────────────────────────

const sha256 = (text: string) => createHash('sha256').update(text, 'utf8').digest('hex');

function isHex64(s: unknown): s is string {
  return typeof s === 'string' && /^[a-f0-9]{64}$/.test(s);
}

async function readJson(path: string): Promise<unknown> {
  const raw = await fs.readFile(path, 'utf8');
  return JSON.parse(raw);
}

function resolveUnder(baseRunDir: string, relative: string): string {
  if (typeof relative !== 'string' || relative.length === 0) {
    throw new Error(`editorial-prefix: expected relative path, got ${JSON.stringify(relative)}`);
  }
  if (isAbsolute(relative) || relative.includes('..')) {
    throw new Error(`editorial-prefix: manifest paths must be baseRunDir-relative and inside it (got "${relative}")`);
  }
  return resolve(baseRunDir, relative);
}

function verifyReceiptShape(sectionId: string, receipt: SectionExecutionReceipt): string | null {
  if (!receipt || typeof receipt !== 'object') return `receipt for "${sectionId}" is malformed`;
  if (receipt.section_id !== sectionId) return `receipt.section_id "${receipt.section_id}" !== "${sectionId}"`;
  if (!receipt.raw || typeof receipt.raw.synthesis !== 'string' || receipt.raw.synthesis.length < 100) {
    return `receipt "${sectionId}" raw.synthesis missing or too short`;
  }
  if (!isHex64(receipt.output_hash)) return `receipt "${sectionId}" output_hash malformed`;
  if (sha256(receipt.raw.synthesis) !== receipt.output_hash) {
    return `receipt "${sectionId}" output_hash mismatch — receipt tampered`;
  }
  if (typeof receipt.raw.aletheios !== 'string' || sha256(receipt.raw.aletheios) !== receipt.aletheios_hash) {
    return `receipt "${sectionId}" aletheios_hash mismatch — receipt tampered`;
  }
  if (typeof receipt.raw.pichet !== 'string' || sha256(receipt.raw.pichet) !== receipt.pichet_hash) {
    return `receipt "${sectionId}" pichet_hash mismatch — receipt tampered`;
  }
  if (typeof receipt.synthesis_input_text !== 'string' || sha256(receipt.synthesis_input_text) !== receipt.synthesis_input_hash) {
    return `receipt "${sectionId}" synthesis_input_hash mismatch — receipt tampered`;
  }
  if (!Array.isArray(receipt.source_ids) || receipt.source_ids.length === 0) {
    return `receipt "${sectionId}" has no source_ids`;
  }
  if (!Array.isArray(receipt.source_hashes) || receipt.source_hashes.length !== receipt.source_ids.length) {
    return `receipt "${sectionId}" source_hashes length mismatch`;
  }
  if (!receipt.retrieval || receipt.retrieval.state !== 'success' || receipt.retrieval.count !== receipt.source_ids.length) {
    return `receipt "${sectionId}" retrieval receipt state/count invalid`;
  }
  if (receipt.passages_hash !== receipt.retrieval.passages_hash) {
    return `receipt "${sectionId}" passages_hash mismatch between top-level and retrieval`;
  }
  if (typeof receipt.engine_facts !== 'string' || receipt.engine_facts.length === 0) {
    return `receipt "${sectionId}" engine_facts absent — cannot rebind audit input hash`;
  }
  return null;
}

function verifyReceiptPersonas(
  sectionId: string,
  receipt: SectionExecutionReceipt,
  identity: EditorialPrefixIdentity,
): string | null {
  if (receipt.aletheios_persona?.sourceHash !== identity.aletheiosPersonaHash) {
    return `receipt "${sectionId}" aletheios persona hash differs from current`;
  }
  if (receipt.pichet_persona?.sourceHash !== identity.pichetPersonaHash) {
    return `receipt "${sectionId}" pichet persona hash differs from current`;
  }
  return null;
}

function verifyReceiptSources(
  sectionId: string,
  receipt: SectionExecutionReceipt,
  identity: EditorialPrefixIdentity,
): string | null {
  for (let i = 0; i < receipt.source_ids.length; i += 1) {
    const id = receipt.source_ids[i];
    const text = identity.sourceTexts[id];
    if (typeof text !== 'string' || text.length === 0) {
      return `receipt "${sectionId}" source "${id}" missing from fresh preflight`;
    }
    if (sha256(text) !== receipt.source_hashes[i]) {
      return `receipt "${sectionId}" source "${id}" hash differs from fresh preflight — retrieval drift`;
    }
  }
  const joined = receipt.source_ids.map((id) => identity.sourceTexts[id]).join('\n');
  if (sha256(joined) !== receipt.passages_hash) {
    return `receipt "${sectionId}" passages_hash differs from fresh preflight join`;
  }
  return null;
}

function verifyReceiptOutcome(sectionId: string, receipt: SectionExecutionReceipt): string | null {
  const classification = classifyOriginalOutcome(receipt);
  return classification.kind === 'blocked' ? `receipt "${sectionId}": ${classification.blockers.join(', ')}` : null;
}

// A very light structural check for the revision artifact on disk — we do not
// trust its stored revisedOutput/revisedOutputHash; those are recomputed.
function coerceRevisionPatches(raw: unknown, sectionId: string): {
  patches: EditorialPatch[];
  reviewer: EditorialRevision['reviewer'];
  timestamp: string;
  storedRevisedOutput: string;
  storedRevisedOutputHash: string;
  storedBaseOutputHash: string;
  storedSectionId: string;
} {
  if (!raw || typeof raw !== 'object') throw new Error(`revision artifact for "${sectionId}" is not an object`);
  const r = raw as Record<string, unknown>;
  if (typeof r.sectionId !== 'string') throw new Error(`revision "${sectionId}" missing sectionId`);
  if (typeof r.baseOutputHash !== 'string') throw new Error(`revision "${sectionId}" missing baseOutputHash`);
  if (typeof r.revisedOutput !== 'string') throw new Error(`revision "${sectionId}" missing revisedOutput`);
  if (typeof r.revisedOutputHash !== 'string') throw new Error(`revision "${sectionId}" missing revisedOutputHash`);
  if (!Array.isArray(r.patches) || r.patches.length === 0) throw new Error(`revision "${sectionId}" missing patches`);
  if (r.reviewer !== 'assistant-source-review' && r.reviewer !== 'human') {
    throw new Error(`revision "${sectionId}" reviewer must be assistant-source-review|human`);
  }
  if (typeof r.timestamp !== 'string' || r.timestamp.length === 0) {
    throw new Error(`revision "${sectionId}" missing timestamp`);
  }
  const patches: EditorialPatch[] = r.patches.map((p, i) => {
    if (!p || typeof p !== 'object') throw new Error(`revision "${sectionId}" patch[${i}] malformed`);
    const pp = p as Record<string, unknown>;
    if (typeof pp.before !== 'string' || typeof pp.after !== 'string' || typeof pp.rationale !== 'string') {
      throw new Error(`revision "${sectionId}" patch[${i}] missing before/after/rationale`);
    }
    if (!Array.isArray(pp.sourceReferences) || pp.sourceReferences.length === 0) {
      throw new Error(`revision "${sectionId}" patch[${i}] missing sourceReferences`);
    }
    const sourceReferences = pp.sourceReferences.map((s, j) => {
      if (!s || typeof s !== 'object') throw new Error(`revision "${sectionId}" patch[${i}] source[${j}] malformed`);
      const ss = s as Record<string, unknown>;
      if (typeof ss.sourceId !== 'string' || typeof ss.quote !== 'string') {
        throw new Error(`revision "${sectionId}" patch[${i}] source[${j}] missing sourceId/quote`);
      }
      return {
        sourceId: ss.sourceId,
        quote: ss.quote,
        ...(typeof ss.label === 'string' ? { label: ss.label } : {}),
      };
    });
    return {
      before: pp.before,
      after: pp.after,
      rationale: pp.rationale,
      sourceReferences,
    };
  });
  return {
    patches,
    reviewer: r.reviewer,
    timestamp: r.timestamp,
    storedRevisedOutput: r.revisedOutput,
    storedRevisedOutputHash: r.revisedOutputHash,
    storedBaseOutputHash: r.baseOutputHash,
    storedSectionId: r.sectionId,
  };
}

function verifyFreshAuditShape(sectionId: string, raw: unknown): FreshAuditSidecar | string {
  if (!raw || typeof raw !== 'object') return `audit "${sectionId}" is not an object`;
  const r = raw as Record<string, unknown>;
  if (r.kind !== 'fresh-audit-of-editorial-revision') return `audit "${sectionId}" kind must be fresh-audit-of-editorial-revision`;
  if (!isHex64(r.sectionOutputHash)) return `audit "${sectionId}" sectionOutputHash malformed`;
  if (!isHex64(r.baseReceiptSha256)) return `audit "${sectionId}" baseReceiptSha256 malformed`;
  if (typeof r.engineFacts !== 'string' || r.engineFacts.length === 0) return `audit "${sectionId}" engineFacts missing`;
  if (!r.audit || typeof r.audit !== 'object') return `audit "${sectionId}" audit block missing`;
  const a = r.audit as Record<string, unknown>;
  if (a.section_id !== sectionId) return `audit "${sectionId}" audit.section_id mismatch`;
  if (typeof a.model_raw !== 'string' || a.model_raw.length === 0) return `audit "${sectionId}" audit.model_raw missing`;
  if (!isHex64(a.input_sha256) || !isHex64(a.output_sha256)) return `audit "${sectionId}" audit input/output hashes malformed`;
  if (sha256(a.model_raw) !== a.output_sha256) return `audit "${sectionId}" audit.output_sha256 !== sha256(model_raw) — raw ledger tampered`;
  if (a.status !== 'clean') return `audit "${sectionId}" audit.status is "${a.status}" — must be clean for editorial acceptance evidence`;
  return {
    kind: 'fresh-audit-of-editorial-revision',
    baseReceiptSha256: r.baseReceiptSha256,
    sectionOutputHash: r.sectionOutputHash,
    originalOutcome: (r.originalOutcome as FreshAuditSidecar['originalOutcome']) ?? 'ok',
    revisionVersion: typeof r.revisionVersion === 'string' ? r.revisionVersion : undefined,
    supplementalEvidence: r.supplementalEvidence,
    engineFacts: r.engineFacts,
    instruction: typeof r.instruction === 'string' ? r.instruction : undefined,
    audit: r.audit as unknown as SourceAuditReceipt,
    llmReceipts: r.llmReceipts,
    passageEvidence: r.passageEvidence as FreshAuditSidecar['passageEvidence'],
    finalAccepted: Boolean(r.finalAccepted),
  };
}

function verifyFreshJevShape(sectionId: string, raw: unknown): FreshJevSidecar | string {
  if (!raw || typeof raw !== 'object') return `jev "${sectionId}" is not an object`;
  const r = raw as Record<string, unknown>;
  if (!isHex64(r.sectionOutputHash)) return `jev "${sectionId}" sectionOutputHash malformed`;
  if (!r.receipt || typeof r.receipt !== 'object') return `jev "${sectionId}" receipt missing`;
  const rec = r.receipt as Record<string, unknown>;
  if (rec.status !== 'judged') return `jev "${sectionId}" receipt.status must be "judged" (got "${rec.status}")`;
  if (rec.blocked === true) return `jev "${sectionId}" receipt.blocked === true — editorial prefix forbids blocked Jev`;
  if (!rec.verdicts || typeof rec.verdicts !== 'object') return `jev "${sectionId}" receipt.verdicts missing`;
  return {
    sectionOutputHash: r.sectionOutputHash,
    revisionVersion: typeof r.revisionVersion === 'string' ? r.revisionVersion : undefined,
    rubric: (r.rubric ?? {}) as Record<string, unknown>,
    receipt: {
      pass_id: (rec.pass_id as string) ?? sectionId,
      mode: (rec.mode as string) ?? 'unknown',
      status: rec.status as string,
      model: rec.model as string | undefined,
      latency_ms: rec.latency_ms as number | undefined,
      answers: rec.answers as Record<string, unknown> | undefined,
      verdicts: rec.verdicts as Record<string, string>,
      blocked: rec.blocked as boolean | undefined,
      disagreements: rec.disagreements as string[] | undefined,
    },
    finalAccepted: Boolean(r.finalAccepted),
  };
}

// ─── Public entry point ───────────────────────────────────────────────

/**
 * Load and validate an editorial-prefix manifest. Pure disk I/O + hash math.
 *
 * @param manifestPath  Path to the manifest JSON.
 * @param sectionIds    Ordered pass_plan section IDs for the current run.
 * @param identity      Current run identity (freshly retrieved preflight, personas, subject...).
 */
export async function loadEditorialPrefix(
  manifestPath: string,
  sectionIds: string[],
  identity: EditorialPrefixIdentity,
): Promise<EditorialPrefixResult> {
  // ── 1. Load manifest ─────────────────────────────────────────────
  let manifest: EditorialPrefixManifest;
  try {
    const raw = await fs.readFile(manifestPath, 'utf8');
    manifest = JSON.parse(raw) as EditorialPrefixManifest;
  } catch (err) {
    return { valid: false, reason: `editorial-prefix: cannot read manifest: ${err instanceof Error ? err.message : String(err)}` };
  }
  if (!manifest || typeof manifest !== 'object' || typeof manifest.baseRunDir !== 'string' || !Array.isArray(manifest.sections) || manifest.sections.length === 0) {
    return { valid: false, reason: 'editorial-prefix: manifest missing baseRunDir or sections' };
  }
  const baseRunDir = isAbsolute(manifest.baseRunDir)
    ? manifest.baseRunDir
    : resolve(manifestPath, '..', manifest.baseRunDir);

  // ── 2. Contiguous-prefix / ordering gate ─────────────────────────
  const expectedPrefix = sectionIds.slice(0, manifest.sections.length);
  for (let i = 0; i < manifest.sections.length; i += 1) {
    const s = manifest.sections[i];
    if (!s || typeof s !== 'object' || typeof s.id !== 'string') {
      return { valid: false, reason: `editorial-prefix: manifest.sections[${i}] malformed` };
    }
    if (s.id !== expectedPrefix[i]) {
      return {
        valid: false,
        reason: `editorial-prefix: manifest.sections[${i}].id="${s.id}" is not the next expected pass_plan id "${expectedPrefix[i]}" — must be a contiguous ordered prefix`,
      };
    }
  }

  // ── 3. Verify baseRunDir input.snapshot identity matches current ─
  let baseInputText: string;
  try {
    baseInputText = await fs.readFile(join(baseRunDir, 'input.snapshot.json'), 'utf8');
  } catch (err) {
    return { valid: false, reason: `editorial-prefix: baseRunDir missing input.snapshot.json: ${err instanceof Error ? err.message : String(err)}` };
  }
  if (sha256(baseInputText) !== identity.inputHash) {
    return { valid: false, reason: 'editorial-prefix: baseRunDir input.snapshot.json hash does not match current input — subject inputs changed' };
  }
  let baseInputs: Record<string, unknown>;
  try {
    baseInputs = (await readJson(join(baseRunDir, 'inputs.json'))) as Record<string, unknown>;
  } catch (err) {
    return { valid: false, reason: `editorial-prefix: baseRunDir missing inputs.json: ${err instanceof Error ? err.message : String(err)}` };
  }
  const identityErrors: string[] = [];
  if (baseInputs.subject !== identity.subject) identityErrors.push(`subject: prior=${String(baseInputs.subject)} current=${identity.subject}`);
  if (baseInputs.language !== identity.language) identityErrors.push(`language: prior=${String(baseInputs.language)} current=${identity.language}`);
  if (baseInputs.accountId !== identity.accountId) identityErrors.push(`accountId: prior=${String(baseInputs.accountId)} current=${identity.accountId}`);
  if (baseInputs.index !== identity.index) identityErrors.push(`index: prior=${String(baseInputs.index)} current=${identity.index}`);
  if (baseInputs.inputHash !== identity.inputHash) identityErrors.push(`inputHash: prior=${String(baseInputs.inputHash)} current=${identity.inputHash}`);
  const priorPrimary = baseInputs.primaryRegistry as {sha256?: unknown} | undefined;
  if (priorPrimary && (typeof priorPrimary.sha256 !== 'string' || priorPrimary.sha256 !== identity.primaryRegistrySha256)) {
    identityErrors.push('generation primary registry hash mismatch');
  }
  const personaHashes = Array.isArray(baseInputs.personaHashes) ? (baseInputs.personaHashes as string[]) : [];
  if (personaHashes[0] !== identity.aletheiosPersonaHash) identityErrors.push('aletheios persona hash mismatch');
  if (personaHashes[1] !== identity.pichetPersonaHash) identityErrors.push('pichet persona hash mismatch');
  if (identityErrors.length > 0) {
    return { valid: false, reason: `editorial-prefix: baseRunDir identity mismatch: ${identityErrors.join('; ')}` };
  }

  // ── 4. Per-section validation ────────────────────────────────────
  const prefixSections: EditorialPrefixSection[] = [];
  const pendingAmbiguity: EditorialPrefixPending[] = [];

  for (const entry of manifest.sections) {
    const sectionId = entry.id;

    let receiptPath: string;
    let revisionPath: string;
    let auditPath: string;
    let jevPath: string;
    try {
      receiptPath = join(baseRunDir, `${sectionId}.receipt.json`);
      revisionPath = resolveUnder(baseRunDir, entry.revisionFile);
      auditPath = resolveUnder(baseRunDir, entry.auditFile);
      jevPath = resolveUnder(baseRunDir, entry.jevFile);
    } catch (err) {
      return { valid: false, reason: `editorial-prefix: ${err instanceof Error ? err.message : String(err)}` };
    }

    // 4a. Original receipt
    let receiptText: string;
    try {
      receiptText = await fs.readFile(receiptPath, 'utf8');
    } catch (err) {
      return { valid: false, reason: `editorial-prefix: cannot read ${sectionId}.receipt.json: ${err instanceof Error ? err.message : String(err)}` };
    }
    const baseReceiptSha = sha256(receiptText);
    let originalReceipt: SectionExecutionReceipt;
    try {
      originalReceipt = JSON.parse(receiptText) as SectionExecutionReceipt;
    } catch (err) {
      return { valid: false, reason: `editorial-prefix: ${sectionId}.receipt.json is not valid JSON: ${err instanceof Error ? err.message : String(err)}` };
    }
    const shapeErr = verifyReceiptShape(sectionId, originalReceipt);
    if (shapeErr) return { valid: false, reason: `editorial-prefix: ${shapeErr}` };
    const personaErr = verifyReceiptPersonas(sectionId, originalReceipt, identity);
    if (personaErr) return { valid: false, reason: `editorial-prefix: ${personaErr}` };
    const sourceErr = verifyReceiptSources(sectionId, originalReceipt, identity);
    if (sourceErr) return { valid: false, reason: `editorial-prefix: ${sourceErr}` };
    const outcomeErr = verifyReceiptOutcome(sectionId, originalReceipt);
    if (outcomeErr) return { valid: false, reason: `editorial-prefix: ${outcomeErr}` };

    // 4b. Revision artifact — parse patches, then re-apply
    let revisionRaw: unknown;
    try {
      revisionRaw = await readJson(revisionPath);
    } catch (err) {
      return { valid: false, reason: `editorial-prefix: cannot read revision for ${sectionId}: ${err instanceof Error ? err.message : String(err)}` };
    }
    let coerced;
    try {
      coerced = coerceRevisionPatches(revisionRaw, sectionId);
    } catch (err) {
      return { valid: false, reason: `editorial-prefix: ${err instanceof Error ? err.message : String(err)}` };
    }
    if (coerced.storedSectionId !== sectionId) {
      return { valid: false, reason: `editorial-prefix: revision sectionId "${coerced.storedSectionId}" !== manifest id "${sectionId}"` };
    }
    if (coerced.storedBaseOutputHash !== originalReceipt.output_hash) {
      return { valid: false, reason: `editorial-prefix: revision baseOutputHash does not match original receipt output_hash for ${sectionId}` };
    }
    let revision: EditorialRevision;
    try {
      revision = applyEditorialRevision({
        baseReceipt: originalReceipt,
        currentOutput: originalReceipt.raw.synthesis,
        expectedBaseOutputHash: originalReceipt.output_hash,
        patches: coerced.patches,
        reviewer: coerced.reviewer,
        timestamp: coerced.timestamp,
      });
    } catch (err) {
      const detail = err instanceof EditorialRevisionError ? `${err.code}: ${err.message}` : err instanceof Error ? err.message : String(err);
      return { valid: false, reason: `editorial-prefix: re-application of patches for ${sectionId} failed — ${detail}` };
    }
    if (revision.revisedOutput !== coerced.storedRevisedOutput) {
      return { valid: false, reason: `editorial-prefix: recomputed revisedOutput for ${sectionId} does not equal stored artifact — never trust arbitrary edited artifact JSON` };
    }
    if (revision.revisedOutputHash !== coerced.storedRevisedOutputHash) {
      return { valid: false, reason: `editorial-prefix: recomputed revisedOutputHash for ${sectionId} does not equal stored artifact hash — tampering detected` };
    }

    // 4c. Fresh audit sidecar
    let auditRaw: unknown;
    try {
      auditRaw = await readJson(auditPath);
    } catch (err) {
      return { valid: false, reason: `editorial-prefix: cannot read audit for ${sectionId}: ${err instanceof Error ? err.message : String(err)}` };
    }
    const auditOrErr = verifyFreshAuditShape(sectionId, auditRaw);
    if (typeof auditOrErr === 'string') return { valid: false, reason: `editorial-prefix: ${auditOrErr}` };
    const freshAudit = auditOrErr;
    if (freshAudit.sectionOutputHash !== revision.revisedOutputHash) {
      return { valid: false, reason: `editorial-prefix: audit.sectionOutputHash for ${sectionId} does not match recomputed revised output — stale audit` };
    }
    if (freshAudit.baseReceiptSha256 !== baseReceiptSha) {
      return { valid: false, reason: `editorial-prefix: audit.baseReceiptSha256 for ${sectionId} does not match on-disk receipt bytes — stale audit` };
    }
    let expectedEngineFacts = originalReceipt.engine_facts as string;
    if (freshAudit.supplementalEvidence !== undefined) {
      const evidence = freshAudit.supplementalEvidence as Record<string, unknown>;
      if (!evidence || typeof evidence !== 'object' || typeof evidence.path !== 'string' || typeof evidence.text !== 'string' ||
        evidence.sha256 !== sha256(evidence.text) || identity.implementationSourceTexts?.[evidence.path] !== evidence.text ||
        evidence.scope !== 'Current local implementation; not independent proof of the saved production binary version' ||
        Object.keys(evidence).some(key => !['path', 'sha256', 'text', 'scope'].includes(key))) {
        return { valid: false, reason: `editorial-prefix: supplemental implementation evidence for ${sectionId} is unverified or stale` };
      }
      expectedEngineFacts += '\nAdditional implementation evidence, with limited scope:\n' + JSON.stringify(evidence);
    }
    if (freshAudit.engineFacts !== expectedEngineFacts) {
      return { valid: false, reason: `editorial-prefix: audit.engineFacts for ${sectionId} differs from receipt.engine_facts — engine facts drift` };
    }

    // 4c.5 Optional passage evidence: revalidate additions against the
    // caller's independently trusted registry + CF target and against
    // this receipt's own source_ids / source_hashes / passages_hash.
    // On success, bind the audit input hash to the effective passages
    // hash.  When absent, base-only binding is preserved verbatim.
    let effectivePassagesHash: string = originalReceipt.passages_hash;
    const passageEvidence = freshAudit.passageEvidence;
    if (passageEvidence !== undefined) {
      if (!passageEvidence || !Array.isArray(passageEvidence.basePassages) ||
          !passageEvidence.basePassages.every(p => p && typeof p.id === 'string' && typeof p.text === 'string') ||
          !Array.isArray(passageEvidence.additions) || !Array.isArray(passageEvidence.addedSourceRecords)) {
        return { valid: false, reason: `editorial-prefix: malformed passage evidence for ${sectionId}` };
      }
      const baseIds = originalReceipt.source_ids ?? [];
      const baseHashes = originalReceipt.source_hashes ?? [];
      const bp = Array.isArray(passageEvidence.basePassages) ? passageEvidence.basePassages : [];
      if (bp.length !== baseIds.length) {
        return { valid: false, reason: `editorial-prefix: passage evidence for ${sectionId} base length differs from receipt.source_ids` };
      }
      for (let i = 0; i < bp.length; i += 1) {
        if (bp[i].id !== baseIds[i]) {
          return { valid: false, reason: `editorial-prefix: passage evidence for ${sectionId} base id[${i}] "${bp[i].id}" !== receipt.source_ids[${i}] "${baseIds[i]}"` };
        }
        if (sha256(bp[i].text) !== baseHashes[i]) {
          return { valid: false, reason: `editorial-prefix: passage evidence for ${sectionId} base text hash mismatch for "${baseIds[i]}"` };
        }
      }
      if (passageEvidence.basePassagesHash !== originalReceipt.passages_hash) {
        return { valid: false, reason: `editorial-prefix: passage evidence for ${sectionId} basePassagesHash differs from receipt.passages_hash` };
      }
      if (!identity.trustedPassageSources) {
        return { valid: false, reason: `editorial-prefix: passage evidence for ${sectionId} requires identity.trustedPassageSources — silence never admits additions` };
      }
      const needsCF = (passageEvidence.additions ?? []).some(
        (a) => a && a.provenance && a.provenance.kind === 'cf-vectorize',
      );
      if (needsCF && !identity.expectedCF) {
        return { valid: false, reason: `editorial-prefix: passage evidence for ${sectionId} carries CF additions but identity.expectedCF is unset` };
      }
      const trustedOriginalTexts: Record<string, string> = {};
      for (const bpp of bp) trustedOriginalTexts[bpp.id] = bpp.text;
      try {
        const revalidated = validateEditorialPassageEvidence({
          expectedCF: identity.expectedCF,
          basePassages: bp,
          basePassagesHash: originalReceipt.passages_hash,
          additions: passageEvidence.additions ?? [],
          trustedSourceRecords: identity.trustedPassageSources,
          trustedOriginalTexts,
        });
        if (revalidated.effectivePassagesHash !== passageEvidence.effectivePassagesHash) {
          return { valid: false, reason: `editorial-prefix: passage evidence for ${sectionId} effectivePassagesHash does not round-trip` };
        }
        if (!isDeepStrictEqual(
          (passageEvidence.addedSourceRecords ?? []).map((r) => JSON.parse(JSON.stringify(r))),
          revalidated.addedSourceRecords.map((r) => JSON.parse(JSON.stringify(r))),
        )) {
          return { valid: false, reason: `editorial-prefix: passage evidence for ${sectionId} addedSourceRecords do not deep-equal the trusted registry — forged provenance` };
        }
        effectivePassagesHash = revalidated.effectivePassagesHash;
      } catch (err) {
        const msg = err instanceof EditorialPassageEvidenceError ? err.message : (err instanceof Error ? err.message : String(err));
        return { valid: false, reason: `editorial-prefix: passage evidence for ${sectionId} rejected: ${msg}` };
      }
    }

    const expectedAuditInputHash = computeAuditInputHash({
      acceptedOutput: revision.revisedOutput,
      passagesHash: effectivePassagesHash,
      engineFacts: expectedEngineFacts,
    });
    if (freshAudit.audit.input_sha256 !== expectedAuditInputHash) {
      return { valid: false, reason: `editorial-prefix: audit.input_sha256 for ${sectionId} does not match computeAuditInputHash(revised, effective passages_hash, saved engineFacts) — canonical input drift` };
    }
    if (!hasCleanAuditEvidence(freshAudit.audit, revision.revisedOutput)) {
      return { valid: false, reason: `editorial-prefix: hasCleanAuditEvidence rejected fresh audit for ${sectionId} — raw clean ledger invalid` };
    }
    if (freshAudit.audit.coverage.total_claims_audited <= 0) {
      return { valid: false, reason: `editorial-prefix: fresh audit for ${sectionId} has zero coverage` };
    }

    // 4d. Fresh Jev sidecar
    let jevRaw: unknown;
    try {
      jevRaw = await readJson(jevPath);
    } catch (err) {
      return { valid: false, reason: `editorial-prefix: cannot read jev for ${sectionId}: ${err instanceof Error ? err.message : String(err)}` };
    }
    const jevOrErr = verifyFreshJevShape(sectionId, jevRaw);
    if (typeof jevOrErr === 'string') return { valid: false, reason: `editorial-prefix: ${jevOrErr}` };
    const freshJev = jevOrErr;
    if (['word_count_recovery', 'source_audit_reaudit_recovery', 'source_audit_after_word_fit_recovery'].includes(classifyOriginalOutcome(originalReceipt).kind) &&
        (revision.revisedOutputHash === originalReceipt.output_hash ||
         !hasRecoveredWordFit(revision.revisedOutput, freshJev.rubric))) {
      return { valid: false, reason: `editorial-prefix: ${sectionId} word-count recovery requires a changed draft and fresh recomputed passing word fit` };
    }
    if (freshJev.sectionOutputHash !== revision.revisedOutputHash) {
      return { valid: false, reason: `editorial-prefix: jev.sectionOutputHash for ${sectionId} does not match recomputed revised output — stale jev` };
    }
    const verdicts = freshJev.receipt.verdicts;
    const failedDims = Object.entries(verdicts).filter(([, v]) => v === 'fail').map(([k]) => k);
    if (failedDims.length > 0) {
      return { valid: false, reason: `editorial-prefix: jev for ${sectionId} has fail verdict(s): ${failedDims.join(', ')}` };
    }
    const ambiguousDims = Object.entries(verdicts).filter(([, v]) => v === 'could-not-tell').map(([k]) => k);
    if (ambiguousDims.length > 0) {
      pendingAmbiguity.push({ section_id: sectionId, ambiguousVerdicts: ambiguousDims, jevArtifactPath: jevPath });
    }

    prefixSections.push({
      section_id: sectionId,
      originalReceipt,
      revision,
      revisedOutput: revision.revisedOutput,
      revisedOutputHash: revision.revisedOutputHash,
      freshAudit,
      freshJev,
      paths: { receipt: receiptPath, revision: revisionPath, audit: auditPath, jev: jevPath },
    });
  }

  const provenance = [
    `Editorial-prefix continuation from: ${baseRunDir}`,
    `Sections carried forward (${prefixSections.length}): ${prefixSections.map((s) => s.section_id).join(', ')}`,
    pendingAmbiguity.length > 0
      ? `Pending editorial acceptance — ambiguous Jev verdicts: ${pendingAmbiguity.map((p) => `${p.section_id}[${p.ambiguousVerdicts.join('/')}]`).join(', ')}`
      : 'No ambiguous Jev verdicts recorded',
    'generation-only: downstream final verification still blocks original-vs-editorial mismatch until provenance-aware acceptance ships',
  ].join('\n');

  return { valid: true, manifestPath, baseRunDir, prefixSections, pendingAmbiguity, provenance };
}
