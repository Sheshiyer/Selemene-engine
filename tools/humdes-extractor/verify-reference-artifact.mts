/**
 * verify-reference-artifact.mts
 *
 * Bytes-first CLI that runs the reference-verifier-bound final review over
 * a saved reference-execution run.  Never mutates original receipts; writes
 * a unique verification output file next to them.  Fails closed on any
 * missing required section, missing coherence review when revised sections
 * are declared, missing render/parity evidence, or CF source snapshot
 * mismatch.  Does NOT accept a caller-supplied acceptance boolean and does
 * NOT shrink the manifest to match what happens to be on disk.
 *
 * Usage:
 *   tsx tools/humdes-extractor/verify-reference-artifact.mts \
 *     --manifest <path> [--out <path>] [--report-partial] [--dry-run] \
 *     [--allow-missing-cf]
 *
 * Manifest schema (v1) — JSON on disk:
 *
 * {
 *   "schemaVersion": 1,
 *   "mode": "full" | "generation-only" | "source-only",
 *   "runDir": "<absolute path to run directory>",
 *   "requiredSectionIds": ["opening", "part1", ..., "part11"],
 *   "sections": [
 *     {
 *       "id": "opening",
 *       "receiptFile": "opening.receipt.json",
 *       "title": "Opening — Scope and Available Systems",
 *       "subsections": ["1.1", "1.2"],
 *       "revised": {
 *         "revisionFile": "editorial-prefix/editorial-revisions/opening.editorial-revision.json",
 *         "freshAuditFile": "editorial-prefix/audits/opening.fresh-audit.json",
 *         "freshJevFile": "editorial-prefix/jev/opening.fresh-jev.json",
 *         "guardrailDispositionFile": "opening.guardrail-disposition.json",
 *         "expectedJevMode": "shadow"
 *       },
 *       "priorContext": {
 *         "opening": { "kind": "revised" }
 *       }
 *     }
 *   ],
 *   "personas": {
 *     "aletheios": "/absolute/path/to/aletheios/IDENTITY.md",
 *     "pichet":    "/absolute/path/to/pichet/IDENTITY.md"
 *   },
 *   "sourceSupplements": {
 *     "crates/engine-transits/src/ephemeris.rs": "/abs/path/to/that/file"
 *   },
 *   "cloudflare": {
 *     "accountId": "abcdef0123456789abcdef0123456789",
 *     "indexName": "witness-wisdom-corpus"
 *   },
 *   "coherenceReviewFile": "coherence-review.json",
 *   "renderReceipt":  {
 *     "path": "render-qa/report.json",
 *     "sha256": "<qa report bytes sha>",
 *     "artifact": { "path": "render-qa/report.pdf", "sha256": "<pdf bytes sha>", "kind": "application/pdf" },
 *     "bindsAssembledSha256": "<sha256 of the assembled Markdown artifact text>",
 *     "status": "completed",
 *     "reviewer": "human",
 *     "reviewedAt": "2026-09-30T12:00:00Z",
 *     "findings": [{ "id": "f1", "summary": "font ok", "unresolved": false }]
 *   },
 *   "parityReceipt":  {
 *     "path": "parity-review.json",
 *     "sha256": "<parity report bytes sha>",
 *     "languages": ["en", "fr"],
 *     "artifacts": {
 *       "en": { "path": "render-qa/report.en.pdf", "sha256": "<en pdf bytes sha>" },
 *       "fr": { "path": "render-qa/report.fr.pdf", "sha256": "<fr pdf bytes sha>" }
 *     },
 *     "status": "completed",
 *     "reviewer": "human",
 *     "reviewedAt": "2026-09-30T12:00:00Z",
 *     "findings": []
 *   },
 *   "bilingualExpected": false,
 *   "modeFile": "/abs/path/to/packages/witness-pipeline/modes/integrated-kundali-reference.md",
 *   "outputFile": "final-verification-<ISO>.json"
 * }
 *
 * Fail-closed rules:
 *   * mode !== 'full' cannot claim finalAccepted (enforced by runFinalReview).
 *   * Any missing section receipt is a blocker; the manifest's
 *     requiredSectionIds is NEVER shrunk to match disk.
 *   * `--report-partial` allows the CLI to still write an output document
 *     when receipts are missing, but finalAccepted stays false.
 *   * Real CF readback is attempted unless `--allow-missing-cf` is passed;
 *     when readback fails, the failure is preserved as a source-integrity
 *     blocker rather than silently ignored.
 */

import { promises as fs, existsSync, writeFileSync } from 'node:fs';
import { resolve, dirname, isAbsolute, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { referenceModeFile, triadEngineFactsFromSnapshots } from '../../packages/witness-pipeline/src/orchestrator/reference-mode-contract.js';
import { BUSINESS_TRIAD_SLUGS, localizeBusinessTriadTitles } from '../../packages/witness-pipeline/src/orchestrator/business-triad-support.js';
import { localizeReferenceTitles } from '../../packages/witness-pipeline/src/orchestrator/reference-language.js';
import { createHash } from 'node:crypto';
import type { SectionExecutionReceipt } from '../../packages/witness-pipeline/src/orchestrator/reference-execution.js';
import type { PassResult } from '../../packages/witness-pipeline/src/orchestrator/integrated.js';
import {
  runFinalReview,
  type FinalReviewInput,
  type CloudflareSnapshot,
  type FinalReviewResult,
  CANONICAL_FULL_MODE_SECTION_IDS,
} from '../../packages/witness-pipeline/src/orchestrator/final-verification.js';
import type {
  EditorialVerificationOptions,
  EditorialSectionEvidence,
  EditorialCoherenceReview,
  FreshAuditSidecar,
  FreshJevSidecar,
  PriorContextEdge,
} from '../../packages/witness-pipeline/src/orchestrator/editorial-verification.js';
import type {
  GuardrailDispositionArtifact,
  JevReviewArtifact,
  GuardrailDiagnosticArtifact,
  RevisionArtifact,
} from '../../packages/witness-pipeline/src/orchestrator/guardrail-disposition.js';
import { VectorizeCorpusAdapter } from '../../packages/witness-pipeline/src/grounding/index.js';
import { auditSectionOutput, countReportWords } from '../../packages/witness-pipeline/src/orchestrator/rubric.js';
import { parseModeDoc } from '../../packages/witness-pipeline/src/modes/parser.js';
import type { SectionRubric } from '../../packages/witness-pipeline/src/orchestrator/integrated.js';

// ─── Args ──────────────────────────────────────────────────────────────

const args = process.argv.slice(2);
function argVal(name: string): string | undefined {
  const i = args.indexOf(name);
  return i >= 0 && i + 1 < args.length ? args[i + 1] : undefined;
}
const has = (name: string) => args.includes(name);

const manifestArg = argVal('--manifest');
if (!manifestArg) {
  console.error('Usage: verify-reference-artifact.mts --manifest <path> [--out <path>] [--report-partial] [--allow-missing-cf] [--dry-run]');
  process.exit(2);
}

const reportPartial = has('--report-partial');
const allowMissingCf = has('--allow-missing-cf');
const dryRun = has('--dry-run');
const outOverride = argVal('--out');

// ─── Helpers ───────────────────────────────────────────────────────────

const sha256 = (bytes: string) => createHash('sha256').update(bytes, 'utf8').digest('hex');

function abs(runDir: string, p: string): string {
  return isAbsolute(p) ? p : resolve(runDir, p);
}

async function readBytes(path: string): Promise<string> {
  return fs.readFile(path, 'utf8');
}

async function readJson<T>(path: string): Promise<T> {
  return JSON.parse(await readBytes(path)) as T;
}

interface ManifestSectionRevised {
  revisionFile: string;
  freshAuditFile: string;
  freshJevFile: string;
  guardrailDispositionFile?: string;
  expectedJevMode?: string;
}
interface ManifestSection {
  id: string;
  receiptFile: string;
  title: string;
  subsections?: string[];
  revised?: ManifestSectionRevised;
  priorContext?: Record<string, PriorContextEdge>;
  /**
   * Optional path (run-relative) to a saved rubric.json for this section.
   * When absent, the CLI will look for `freshJev.rubric` for revised
   * sections and otherwise recompute deterministically via
   * `auditSectionOutput`.  Fabricated all-pass defaults are FORBIDDEN.
   */
  rubricFile?: string;
}
interface Manifest {
  schemaVersion: 1;
  mode: 'full' | 'generation-only' | 'source-only';
  runDir: string;
  requiredSectionIds: string[];
  sections: ManifestSection[];
  personas: Record<string, string>;
  sourceSupplements?: Record<string, string>;
  /** Independently approved passage registry, resolved relative to runDir. */
  passageRegistryFile?: string;
  /**
   * Optional independent primary passage registry used at fresh
   * generation. When present it is loaded and hash-bound; the CLI hands
   * `runFinalReview` the exact bytes so acceptance never trusts a
   * receipt's self-asserted primary claim.
   */
  primaryPassageRegistryFile?: string;
  cloudflare?: { accountId: string; indexName?: string };
  coherenceReviewFile?: string;
  historicalRevisions?: Record<string, ManifestSectionRevised[]>;
  /**
   * Render evidence packet — REQUIRED for full mode.
   *   * `path` / `sha256` identify the QA-report file (bytes hashed).
   *   * `artifact` MUST point at the actual rendered DOCX/PDF/HTML with
   *     both `path` and `sha256`; CLI reads its bytes and re-hashes.
   *   * `bindsAssembledSha256` MUST equal sha256 of the assembled artifact
   *     text that runFinalReview computes.
   *   * `status:'completed'`, non-empty `reviewer`, `reviewedAt`, and
   *     zero unresolved findings are enforced by runFinalReview.
   */
  renderReceipt?: {
    path: string;
    sha256: string;
    kind?: string;
    artifact: { path: string; sha256: string; kind?: string };
    bindsAssembledSha256: string;
    status: 'completed' | string;
    reviewer: string;
    reviewedAt: string;
    findings?: readonly { id: string; summary: string; unresolved: boolean }[];
  };
  /**
   * Parity evidence packet — REQUIRED whenever `bilingualExpected` is true.
   *   * Both `artifacts.<lang>` entries MUST resolve to real files with
   *     matching sha256s; CLI reads both, hashes both, and cross-checks.
   *   * `languages` MUST equal `Object.keys(artifacts)` as a set with
   *     length ≥ 2.
   *   * Same completed-review / reviewer / findings rules as render.
   */
  parityReceipt?: {
    path: string;
    sha256: string;
    kind?: string;
    languages: readonly string[];
    artifacts: Readonly<Record<string, { path: string; sha256: string; kind?: string }>>;
    status: 'completed' | string;
    reviewer: string;
    reviewedAt: string;
    findings?: readonly { id: string; summary: string; unresolved: boolean }[];
    bindsAssembledSha256?: string;
  };
  bilingualExpected?: boolean;
  outputFile?: string;
  /**
   * Optional absolute path to the canonical mode file
   * (`packages/witness-pipeline/modes/<mode>.md`).  When present, the CLI
   * loads `pass_plan` and derives titles + subsection IDs from it, using
   * the file as the source of truth over any per-section title in the
   * manifest.  Required for full mode.
   */
  modeFile?: string;
}

function validateManifestShape(m: any): asserts m is Manifest {
  if (!m || typeof m !== 'object') throw new Error('manifest not an object');
  if (m.schemaVersion !== 1) throw new Error(`unsupported schemaVersion: ${m.schemaVersion}`);
  if (!['full', 'generation-only', 'source-only'].includes(m.mode)) throw new Error(`invalid mode: ${m.mode}`);
  if (typeof m.runDir !== 'string' || !m.runDir.length) throw new Error('runDir missing');
  if (!Array.isArray(m.requiredSectionIds) || m.requiredSectionIds.length === 0) throw new Error('requiredSectionIds empty');
  if (!Array.isArray(m.sections)) throw new Error('sections not array');
  if (!m.personas || typeof m.personas !== 'object') throw new Error('personas missing');
  for (const id of m.requiredSectionIds) {
    const sec = m.sections.find((s: any) => s?.id === id);
    if (!sec) throw new Error(`section entry missing for required id: ${id}`);
    if (typeof sec.receiptFile !== 'string' || !sec.receiptFile.length) throw new Error(`receiptFile missing for section: ${id}`);
    if (typeof sec.title !== 'string' || !sec.title.length) throw new Error(`title missing for section: ${id}`);
  }
}

// ─── Load raw evidence from disk ───────────────────────────────────────

interface LoadedSection {
  id: string;
  title: string;
  receiptPath: string;
  receiptBytes: string;
  receipt: SectionExecutionReceipt;
  subsections: string[];
  revised?: {
    revision: { sectionId: string; baseOutputHash: string; revisedOutput: string; revisedOutputHash: string; patches: readonly any[]; reviewer: any; timestamp: string };
    freshAudit: FreshAuditSidecar;
    freshJev: FreshJevSidecar;
    guardrailDisposition?: {
      disposition: GuardrailDispositionArtifact;
      originalJev: JevReviewArtifact;
      diagnostic: GuardrailDiagnosticArtifact;
      revision: RevisionArtifact;
      rawEvidenceTexts: Record<string, string>;
    };
    expectedJevMode?: string;
    baseReceiptSha256: string;
  };
  priorContext?: Record<string, PriorContextEdge>;
  missing: boolean;
  loadErrors: string[];
}

async function loadSection(runDir: string, sec: ManifestSection): Promise<LoadedSection> {
  const loadErrors: string[] = [];
  const receiptPath = abs(runDir, sec.receiptFile);
  let receiptBytes = '';
  let receipt: SectionExecutionReceipt | null = null;
  let missing = false;
  if (!existsSync(receiptPath)) {
    missing = true;
    loadErrors.push(`receipt file missing: ${receiptPath}`);
  } else {
    try {
      receiptBytes = await readBytes(receiptPath);
      receipt = JSON.parse(receiptBytes) as SectionExecutionReceipt;
    } catch (e: any) {
      loadErrors.push(`receipt parse error for ${sec.id}: ${e?.message ?? e}`);
      missing = true;
    }
  }
  const out: LoadedSection = {
    id: sec.id,
    title: sec.title,
    receiptPath,
    receiptBytes,
    receipt: receipt ?? ({ section_id: sec.id, outcome: 'failed', source_ids: [], source_hashes: [], passages_hash: '', aletheios_hash: '', pichet_hash: '', synthesis_input_hash: '', synthesis_input_text: '', prior_section_content_hashes: {}, output_hash: '', aletheios_persona: { name: 'aletheios', sourcePath: '', sourceHash: '' }, pichet_persona: { name: 'pichet', sourcePath: '', sourceHash: '' }, raw: { aletheios: '', pichet: '', synthesis: '' }, retrieval: { section_id: sec.id, state: 'failure', count: 0 } as any, attempts: [] } as SectionExecutionReceipt),
    subsections: sec.subsections ?? [],
    priorContext: sec.priorContext,
    missing,
    loadErrors,
  };

  if (sec.revised && !missing && receipt) {
    try {
      const revisionPath = abs(runDir, sec.revised.revisionFile);
      const auditPath    = abs(runDir, sec.revised.freshAuditFile);
      const jevPath      = abs(runDir, sec.revised.freshJevFile);
      const revision  = JSON.parse(await readBytes(revisionPath));
      const audit     = JSON.parse(await readBytes(auditPath)) as FreshAuditSidecar;
      const freshJev  = JSON.parse(await readBytes(jevPath))   as FreshJevSidecar;
      type GuardrailDispositionBundle = NonNullable<LoadedSection['revised']>['guardrailDisposition'];
      let guardrailDisposition: GuardrailDispositionBundle = undefined;
      if (sec.revised.guardrailDispositionFile) {
        const dispPath = abs(runDir, sec.revised.guardrailDispositionFile);
        if (!existsSync(dispPath)) {
          loadErrors.push(`guardrail disposition missing: ${dispPath}`);
        } else {
          const disposition = JSON.parse(await readBytes(dispPath)) as GuardrailDispositionArtifact;
          const rawEvidenceTexts: Record<string, string> = {};
          const evidenceEntries: any[] = Array.isArray((disposition as any).evidence) ? (disposition as any).evidence : [];
          for (const ref of evidenceEntries) {
            if (typeof ref?.file !== 'string' || ref.file.includes('/') || ref.file.includes('..')) {
              loadErrors.push(`disposition evidence file unsafe: ${ref?.file}`);
              continue;
            }
            const p = join(dirname(dispPath), ref.file);
            if (!existsSync(p)) {
              loadErrors.push(`disposition evidence missing: ${p}`);
              continue;
            }
            rawEvidenceTexts[ref.file] = await readBytes(p);
          }
          const findJson = (pattern: RegExp) => {
            const ref = evidenceEntries.find((e: any) => pattern.test(e?.file ?? ''));
            if (!ref) return null;
            const raw = rawEvidenceTexts[ref.file];
            return raw ? JSON.parse(raw) : null;
          };
          const originalJev = findJson(/editorial-jev|fresh-jev/);
          const diagnostic  = findJson(/guardrail-diagnostic/);
          const revisionRef = findJson(/editorial-revision/);
          if (!originalJev || !diagnostic || !revisionRef) {
            loadErrors.push('disposition evidence incomplete — need originalJev, diagnostic, revision');
          } else {
            guardrailDisposition = { disposition, originalJev, diagnostic, revision: revisionRef, rawEvidenceTexts };
          }
        }
      }
      out.revised = {
        revision,
        freshAudit: audit,
        freshJev,
        guardrailDisposition,
        expectedJevMode: sec.revised.expectedJevMode,
        baseReceiptSha256: sha256(receiptBytes),
      };
    } catch (e: any) {
      loadErrors.push(`revised evidence load error for ${sec.id}: ${e?.message ?? e}`);
    }
  }

  return out;
}

// ─── Canonical mode loading & subsection extraction ────────────────────

/**
 * Parsed canonical mode data used by the CLI to override caller-supplied
 * titles / subsections.  This is the "canonical mode contract" the user
 * requires: full mode acceptance MUST bind to
 * `packages/witness-pipeline/modes/integrated-kundali-reference.md` (or
 * whichever mode file the manifest names), not to whatever the caller
 * put in the manifest.
 */
interface CanonicalModeContract {
  /** Ordered pass_plan section IDs. */
  sectionIds: readonly string[];
  /** id → title, taken verbatim from pass_plan[*].title. */
  titles: Readonly<Record<string, string>>;
  /** id → target_words. */
  targetWords: Readonly<Record<string, number>>;
  /**
   * id → subsection IDs (e.g. ['1.1','1.2','1.3']) extracted from the
   * matching `## <template>` block by scanning bullet lines that begin
   * with `- N.M` or `- N.M.K`.  This is the SAME structural pattern the
   * reference verifier's subsection gate uses.
   */
  subsections: Readonly<Record<string, readonly string[]>>;
}

function extractSubsectionIdsFromTemplate(templateBody: string): string[] {
  const ids: string[] = [];
  const seen = new Set<string>();
  // Matches lines that start (after optional whitespace) with `-` or `*`
  // followed by a numeric identifier like `1.2`, `2.10.3`, then any
  // separator (space, `—`, `:`).  Prose-only bullets are ignored.
  const re = /^\s*[-*]\s+(\d+(?:\.\d+){1,3})\b/gm;
  let m: RegExpExecArray | null;
  while ((m = re.exec(templateBody)) !== null) {
    const id = m[1];
    if (!seen.has(id)) { seen.add(id); ids.push(id); }
  }
  return ids;
}

async function loadCanonicalMode(modeFilePath: string, language = 'en'): Promise<CanonicalModeContract> {
  const parsed = parseModeDoc(modeFilePath);
  if (parsed.frontmatter.mode === 'business-triad-reference') localizeBusinessTriadTitles(parsed, language);
  else localizeReferenceTitles(parsed, language);
  const passPlan = parsed.frontmatter.pass_plan ?? [];
  const sectionIds = passPlan.map(p => p.id);
  const titles: Record<string, string> = {};
  const targetWords: Record<string, number> = {};
  const subsections: Record<string, string[]> = {};
  for (const p of passPlan) {
    titles[p.id] = p.title;
    targetWords[p.id] = p.target_words;
    const body = parsed.sections[p.template] ?? '';
    subsections[p.id] = extractSubsectionIdsFromTemplate(body);
  }
  return { sectionIds, titles, targetWords, subsections };
}

// ─── Rubric evidence loader ────────────────────────────────────────────
//
// Rubric acquisition order (per section):
//   1. Revised section → use the freshJev.rubric artefact bytes (that IS
//      the reviewer-produced rubric bound to the revised output hash).
//   2. `manifest.sections[i].rubricFile` present on disk → parse bytes.
//   3. Recompute deterministically via auditSectionOutput({ output,
//      targetWords, ... }).  Word count / deterministic fact count /
//      integrated layering / guardrail are all functions of the accepted
//      output text and the canonical target_words.  This is the substrate
//      the pipeline itself uses at generation time.
//   4. NONE of the above → return `{ rubric: undefined, blocker: '...' }`
//      so the caller pushes a preflight blocker.  NEVER fabricate an
//      all-pass default; that is the exact loophole the critical review
//      called out.
//
// The final-verification module additionally sweeps every rubric dimension
// for `'pass' | 'warn' | 'fail'` and rejects anything else — so a rubric
// whose fields are malformed still blocks even if it passes shape
// validation here.

interface RubricLoadResult {
  rubric?: SectionRubric;
  source: 'freshJev' | 'file' | 'recomputed' | 'missing';
  blocker?: string;
}

function validateRubricShape(
  rubric: unknown,
  sectionId: string,
): { ok: boolean; reason?: string } {
  if (!rubric || typeof rubric !== 'object') return { ok: false, reason: 'not_object' };
  const r = rubric as Record<string, unknown>;
  const requiredGates = ['word_count_fit', 'deterministic_fact_gate', 'integrated_layering_gate', 'guardrail_gate'] as const;
  for (const g of requiredGates) {
    if (!['pass', 'warn', 'fail'].includes(r[g] as string)) {
      return { ok: false, reason: `${g}_invalid` };
    }
  }
  if (typeof r.section_id !== 'string' || r.section_id !== sectionId) {
    return { ok: false, reason: 'section_id_mismatch' };
  }
  return { ok: true };
}

async function loadRubricForSection(
  runDir: string,
  loaded: LoadedSection,
  canonical: CanonicalModeContract,
  manifestSection: ManifestSection | undefined,
): Promise<RubricLoadResult> {
  const rev = loaded.revised;
  const canonicalTitle = canonical.titles[loaded.id] ?? loaded.title;
  const canonicalTargetWords = canonical.targetWords[loaded.id] ?? 0;

  // 1) Revised section → freshJev.rubric IS the reviewer-produced rubric.
  if (rev && rev.freshJev && (rev.freshJev as { rubric?: unknown }).rubric) {
    const rubric = rev.freshJev.rubric;
    const shape = validateRubricShape(rubric, loaded.id);
    if (!shape.ok) {
      return { source: 'freshJev', blocker: `section:${loaded.id}:freshJev_rubric_invalid:${shape.reason}` };
    }
    if (rubric?.target_words !== canonicalTargetWords ||
        rubric.actual_words !== countReportWords(rev.revision.revisedOutput)) {
      return { source: 'freshJev', blocker: `section:${loaded.id}:freshJev_rubric_count_or_target_mismatch` };
    }
    return { rubric: rubric as unknown as SectionRubric, source: 'freshJev' };
  }

  // 2) Explicit rubric file next to the receipt.
  if (manifestSection?.rubricFile) {
    const p = abs(runDir, manifestSection.rubricFile);
    if (!existsSync(p)) {
      return { source: 'file', blocker: `section:${loaded.id}:rubric_file_missing:${p}` };
    }
    const bytes = await readBytes(p);
    let parsed: unknown;
    try { parsed = JSON.parse(bytes); }
    catch (e: unknown) {
      const msg = e instanceof Error ? e.message : String(e);
      return { source: 'file', blocker: `section:${loaded.id}:rubric_file_parse_error:${msg}` };
    }
    const shape = validateRubricShape(parsed, loaded.id);
    if (!shape.ok) {
      return { source: 'file', blocker: `section:${loaded.id}:rubric_file_invalid:${shape.reason}` };
    }
    const checked = parsed as SectionRubric;
    const text = rev ? rev.revision.revisedOutput : loaded.receipt.raw.synthesis;
    if (checked.target_words !== canonicalTargetWords || checked.actual_words !== countReportWords(text)) {
      return { source: 'file', blocker: `section:${loaded.id}:rubric_file_count_or_target_mismatch` };
    }
    return { rubric: parsed as SectionRubric, source: 'file' };
  }

  // 3) Deterministic recompute — same fn the pipeline uses at generation.
  //    Requires a positive canonical target_words for the word-count gate
  //    to be meaningful; otherwise return a blocker so we do not silently
  //    accept a target=0 pass.
  if (canonicalTargetWords <= 0) {
    return {
      source: 'missing',
      blocker: `section:${loaded.id}:rubric_evidence_missing_no_canonical_target_words`,
    };
  }
  const output = rev ? rev.revision.revisedOutput : loaded.receipt.raw.synthesis;
  if (typeof output !== 'string' || output.length === 0) {
    return {
      source: 'missing',
      blocker: `section:${loaded.id}:rubric_recompute_impossible_no_output`,
    };
  }
  const rubric = auditSectionOutput({
    sectionId: loaded.id,
    title: canonicalTitle,
    targetWords: canonicalTargetWords,
    output,
    modelRequested: 'recomputed',
    modelUsed: 'recomputed',
    latencyMs: 0,
    engineResults: [],
  });
  return { rubric, source: 'recomputed' };
}

/** Rebuild a PassResult from a loaded receipt + explicit rubric. */
function buildPassResult(loaded: LoadedSection, canonicalTitle: string, rubric: SectionRubric): PassResult {
  const rev = loaded.revised;
  const output = rev ? rev.revision.revisedOutput : loaded.receipt.raw.synthesis;
  return { id: loaded.id, title: canonicalTitle, output, rubric };
}

// ─── Main ──────────────────────────────────────────────────────────────

async function main(): Promise<number> {
  const manifestPath = resolve(manifestArg!);
  const manifestBytes = await readBytes(manifestPath);
  const manifest = JSON.parse(manifestBytes);
  validateManifestShape(manifest);
  const runDir = resolve(manifest.runDir);

  const preflight: string[] = [];

  // Load every required section — never shrink the manifest.
  const loaded: LoadedSection[] = [];
  for (const id of manifest.requiredSectionIds) {
    const sec = manifest.sections.find(s => s.id === id)!;
    const l = await loadSection(runDir, sec);
    loaded.push(l);
    if (l.missing) preflight.push(`section:${id}:receipt_missing`);
    for (const err of l.loadErrors) preflight.push(`section:${id}:${err}`);
  }

  const hasAnyMissing = loaded.some(l => l.missing);
  if (hasAnyMissing && !reportPartial) {
    // Fail closed BEFORE running the verifier — we won't fabricate placeholder
    // receipts.  Ask the caller to fix or pass --report-partial.
    const report = { finalAccepted: false, blockers: preflight, mode: manifest.mode, requiredSectionIds: manifest.requiredSectionIds, warnings: [], scope: 'preflight — pass --report-partial to still write output' };
    console.log(JSON.stringify(report, null, 2));
    if (!dryRun) writeOutput(manifest, runDir, report);
    return 1;
  }

  // ── Canonical mode contract (source of truth for titles/subsections/target_words) ─
  //
  // For full mode we REQUIRE the manifest to name the canonical mode file
  // and enforce that:
  //   * `requiredSectionIds` equals canonical `pass_plan[*].id` in exact
  //     order (checked twice — once here for a clear preflight signal,
  //     once in runFinalReview via CANONICAL_FULL_MODE_SECTION_IDS).
  //   * Each per-section title comes from `pass_plan[*].title`, not from
  //     the caller's manifest.  A manifest title that disagrees with the
  //     canonical title is surfaced as a blocker.
  //   * Subsection IDs are extracted from the canonical pass template and
  //     used as `subsectionMap` for the reference verifier — the CLI does
  //     NOT shrink the contract to whatever subsection list the caller
  //     supplied.
  let canonical: CanonicalModeContract | undefined;
  if (manifest.mode === 'full') {
    if (!manifest.modeFile) {
      preflight.push('canonical:mode_file_missing_for_full_mode');
    } else {
      const modePath = resolve(manifest.modeFile);
      if (!existsSync(modePath)) {
        preflight.push(`canonical:mode_file_not_found:${modePath}`);
      } else {
        try {
          const savedInputs = JSON.parse(await readBytes(join(runDir, 'inputs.json')));
          const contractFile = referenceModeFile(savedInputs);
          const bundledMode = resolve(dirname(fileURLToPath(import.meta.url)), '../../packages/witness-pipeline/modes', contractFile);
          if (contractFile === 'business-triad-reference.md') {
            const snapshots = Object.fromEntries(await Promise.all(BUSINESS_TRIAD_SLUGS.map(async slug => [slug, await readBytes(join(runDir, `input.${slug}.snapshot.json`))])));
            const expectedFacts = triadEngineFactsFromSnapshots(savedInputs, snapshots);
            for (const section of loaded) {
              if (!section.missing && section.receipt.engine_facts !== expectedFacts) preflight.push(`canonical:triad_engine_facts_mismatch:${section.id}`);
            }
            const modeKey = 'packages/witness-pipeline/modes/business-triad-reference.md';
            if (savedInputs.runtimeHashes?.[modeKey] !== sha256(await readBytes(bundledMode))) preflight.push('canonical:triad_mode_runtime_hash_mismatch');
          }
          if (await readBytes(modePath) !== await readBytes(bundledMode)) {
            preflight.push('canonical:mode_file_differs_from_repository_contract');
          }
          canonical = await loadCanonicalMode(bundledMode, savedInputs.language);
        } catch (e: unknown) {
          const msg = e instanceof Error ? e.message : String(e);
          preflight.push(`canonical:mode_file_parse_error:${msg}`);
        }
      }
    }
    if (canonical) {
      // Compare against the canonical embedded constant, not against the
      // manifest — even a manifest that happens to declare 12 IDs but
      // renames one must fail loudly here.
      const expected = CANONICAL_FULL_MODE_SECTION_IDS;
      const declared = manifest.requiredSectionIds;
      if (declared.length !== expected.length) {
        preflight.push(`canonical:section_id_count_mismatch:${declared.length}!=${expected.length}`);
      }
      for (let i = 0; i < Math.min(declared.length, expected.length); i++) {
        if (declared[i] !== expected[i]) {
          preflight.push(`canonical:section_id_mismatch:index_${i}:expected_${expected[i]}:got_${declared[i]}`);
        }
      }
      // Titles: canonical wins; caller mismatches surfaced.
      for (const sec of manifest.sections) {
        const canonicalTitle = canonical.titles[sec.id];
        if (canonicalTitle && sec.title !== canonicalTitle) {
          preflight.push(`canonical:section_title_mismatch:${sec.id}:expected_${canonicalTitle}:got_${sec.title}`);
        }
      }
    }
  }

  // Fallback canonical fields when mode file was not loaded (partial /
  // non-full).  For non-full modes we still recompute the rubric using
  // whatever target_words / title the caller declared; missing values
  // fall through to the loader's blocker path.
  const canonicalOrFallback: CanonicalModeContract = canonical ?? {
    sectionIds: manifest.requiredSectionIds,
    titles: Object.fromEntries(manifest.sections.map(s => [s.id, s.title])),
    targetWords: Object.fromEntries(manifest.sections.map(s => [s.id, 0])),
    subsections: Object.fromEntries(manifest.sections.map(s => [s.id, s.subsections ?? []])),
  };

  // Load personas from disk INDEPENDENTLY of the receipts.
  const expectedPersonas: Record<string, string> = {};
  for (const [name, path] of Object.entries(manifest.personas)) {
    const p = resolve(path);
    if (!existsSync(p)) {
      preflight.push(`persona:${name}:missing:${p}`);
      continue;
    }
    expectedPersonas[name] = await readBytes(p);
  }

  // Load source supplements INDEPENDENTLY.
  const implementationSourceTexts: Record<string, string> = {};
  if (manifest.sourceSupplements) {
    for (const [scopePath, absPath] of Object.entries(manifest.sourceSupplements)) {
      const p = resolve(absPath);
      if (!existsSync(p)) {
        preflight.push(`source_supplement:${scopePath}:missing:${p}`);
        continue;
      }
      implementationSourceTexts[scopePath] = await readBytes(p);
    }
  }

  // Passes rebuilt from receipts/revisions — with rubric loaded from
  // reviewer artefacts (freshJev.rubric), an explicit rubric file, or
  // deterministically recomputed via auditSectionOutput.  A section for
  // which none of these paths yield a valid rubric is a PREFLIGHT
  // BLOCKER, never silently filled with an all-pass default.
  const passes: PassResult[] = [];
  const rubricSources: Record<string, string> = {};
  for (const l of loaded) {
    if (l.missing) continue;
    const manifestSec = manifest.sections.find(s => s.id === l.id);
    const canonicalTitle = canonicalOrFallback.titles[l.id] ?? l.title;
    const rubricResult = await loadRubricForSection(runDir, l, canonicalOrFallback, manifestSec);
    rubricSources[l.id] = rubricResult.source;
    if (!rubricResult.rubric) {
      if (rubricResult.blocker) preflight.push(rubricResult.blocker);
      else preflight.push(`section:${l.id}:rubric_missing`);
      continue;
    }
    passes.push(buildPassResult(l, canonicalTitle, rubricResult.rubric));
  }
  const assembled = passes.map(p => `## ${p.title}\n\n${p.output}`).join('\n\n');
  const assembledSha256 = sha256(assembled);

  // Reference verifier inputs — subsectionMap comes from CANONICAL mode
  // data, not from the caller manifest.
  const requiredSectionIds = manifest.requiredSectionIds;
  const receipts = loaded.filter(l => !l.missing).map(l => l.receipt);
  const subsectionMap: Record<string, string[]> = {};
  const rawSynthesisInputs: Record<string, string> = {};
  const sourceTextsById: Record<string, string> = {};
  const sectionEngineFacts: Record<string, string> = {};
  for (const l of loaded) {
    if (l.missing) continue;
    const canonicalSubs = canonicalOrFallback.subsections[l.id];
    subsectionMap[l.id] = canonicalSubs && canonicalSubs.length > 0
      ? [...canonicalSubs]
      : l.subsections;
    rawSynthesisInputs[l.id] = l.receipt.synthesis_input_text;
    sectionEngineFacts[l.id] = l.receipt.engine_facts ?? '';
  }

  // CF readback for source snapshot integrity.
  let trustedPassageSources: EditorialVerificationOptions['trustedPassageSources'];
  if (manifest.passageRegistryFile) {
    const registry = JSON.parse(await readBytes(abs(runDir, manifest.passageRegistryFile)));
    if (!registry.records || typeof registry.records !== 'object' || Array.isArray(registry.records)) {
      throw new Error('Independent passage registry records missing');
    }
    trustedPassageSources = registry.records;
  }
  const additionalCF = Object.values(trustedPassageSources ?? {}).filter(r => r?.provenance?.kind === 'cf-vectorize');
  // Independent primary registry (mixed-source generation). Loaded from
  // its OWN bytes so acceptance never trusts a receipt's self-asserted
  // primary_source_ids. When the manifest doesn't declare a primary
  // registry file, mixed-source generation cannot be accepted. Editorial
  // passageRegistryFile is separate and never changes generation provenance.
  let primaryRegistryBytes: string | undefined;
  let primaryRegistrySha256: string | undefined;
  let primaryRegistryPath: string | undefined;
  let primaryRegistryParsed: { records?: Record<string, any>; additionIds?: string[] } | undefined;
  if (manifest.primaryPassageRegistryFile) {
    primaryRegistryPath = abs(runDir, manifest.primaryPassageRegistryFile);
    primaryRegistryBytes = await readBytes(primaryRegistryPath);
    primaryRegistrySha256 = sha256(primaryRegistryBytes);
    try { primaryRegistryParsed = JSON.parse(primaryRegistryBytes); }
    catch { preflight.push('primary_registry_not_json'); }
  }
  // Determine consumed primary ids from immutable generation receipts.
  // runFinalReview independently checks the supplied registry authorises them;
  // every receipt-declared primary id is authorised.
  const primaryFromReceipts = new Set<string>();
  for (const l of loaded) for (const id of l.receipt.primary_source_ids ?? []) primaryFromReceipts.add(id);
  const declaredPrimaryIds: string[] = [];
  const seenPrimary = new Set<string>();
  for (const id of primaryFromReceipts) {
    if (typeof id !== 'string' || seenPrimary.has(id)) continue;
    seenPrimary.add(id);
    declaredPrimaryIds.push(id);
  }
  // Populate source texts for primary ids from the registry (bytes-first).
  if (primaryRegistryParsed?.records) {
    for (const id of declaredPrimaryIds) {
      const rec = primaryRegistryParsed.records[id];
      if (!rec || typeof rec.text !== 'string') {
        preflight.push(`primary_registry_missing_text:${id}`);
        continue;
      }
      if (rec.provenance?.kind !== 'reviewed-primary-document') {
        preflight.push(`primary_registry_wrong_kind:${id}`);
        continue;
      }
      sourceTextsById[id] = rec.text;
    }
  } else if (declaredPrimaryIds.length > 0) {
    preflight.push('primary_registry_records_missing');
  }
  let cloudflareSnapshot: CloudflareSnapshot | undefined;
  // CF readback must NOT try to fetch primary ids from CF — they are not there.
  const primarySet = new Set(declaredPrimaryIds);
  const cfCandidateIds = [
    ...receipts.flatMap(r => (r.source_ids ?? []).filter(id => !primarySet.has(id))),
    ...additionalCF.map(r => r.id).filter(id => !primarySet.has(id)),
  ];
  const uniqueSourceIds = [...new Set(cfCandidateIds)];
  if (manifest.cloudflare && !allowMissingCf && uniqueSourceIds.length > 0) {
    const adapter = new VectorizeCorpusAdapter({
      accountId: manifest.cloudflare.accountId,
      indexName: manifest.cloudflare.indexName ?? 'witness-wisdom-corpus',
      timeoutMs: 60_000,
    });
    const receipt = await adapter.getVectors(uniqueSourceIds);
    if (receipt.state !== 'success') {
      preflight.push(`cloudflare_readback_failed: ${receipt.state}: ${(receipt as any).reason ?? ''}`);
    } else {
      cloudflareSnapshot = {
        accountId: manifest.cloudflare.accountId,
        indexName: manifest.cloudflare.indexName ?? 'witness-wisdom-corpus',
        passages: receipt.passages.map(p => ({ id: p.id, text: p.text, sha256: sha256(p.text) })),
      };
      for (const p of receipt.passages) sourceTextsById[p.id] = p.text;
    }
  }
  for (const record of additionalCF) {
    const provenance = record.provenance;
    if (provenance.kind !== 'cf-vectorize') continue;
    const live = sourceTextsById[record.id];
    if (!live || live !== record.text || sha256(live) !== record.sha256 ||
        provenance.account !== manifest.cloudflare?.accountId ||
        provenance.index !== (manifest.cloudflare?.indexName ?? 'witness-wisdom-corpus')) {
      preflight.push(`supplement_cf_readback_mismatch:${record.id}`);
    }
  }
  // For source_ids not in the CF snapshot, we still hand the receipt-declared
  // source text through when present in the run directory; otherwise the
  // reference verifier will flag source_text_unverified.  We do NOT
  // fabricate texts — the source_hashes on the receipts stand.
  for (const l of loaded) {
    for (let i = 0; i < l.receipt.source_ids.length; i++) {
      const id = l.receipt.source_ids[i];
      if (!(id in sourceTextsById) && cloudflareSnapshot) {
        // The CF readback missed this ID — leave it out; verifier will flag it.
      }
    }
  }

  // Editorial evidence bundle.
  //
  // Doctrine: coherence review is REQUIRED for every full-mode run
  // (not only editorial runs).  We load it from disk independently and
  // hand it to runFinalReview, which enforces reviewer, timestamp, status,
  // artifactSha256 binding to sha256(assembled), and zero unresolved
  // findings.  When there are revised sections, the reference verifier
  // ALSO re-runs its stricter provenance-aware coherence checks (this is
  // additive, not a substitute).
  const revisedSections: Record<string, EditorialSectionEvidence> = {};
  for (const l of loaded) {
    if (l.missing) continue;
    if (!l.revised) continue;
    revisedSections[l.id] = {
      baseReceiptSha256: l.revised.baseReceiptSha256,
      baseReceiptRawBytes: l.receiptBytes,
      revisionArtifact: l.revised.revision as any,
      freshAudit: l.revised.freshAudit,
      freshJev: l.revised.freshJev,
      expectedJevMode: l.revised.expectedJevMode,
      guardrailDisposition: l.revised.guardrailDisposition,
    };
  }
  const historicalRevisions: Record<string, EditorialSectionEvidence[]> = {};
  for (const [id, entries] of Object.entries(manifest.historicalRevisions ?? {})) {
    const sec = manifest.sections.find(s => s.id === id);
    if (!sec || !Array.isArray(entries)) { preflight.push(`historical_revision_invalid_section:${id}`); continue; }
    for (const entry of entries) {
      const historical = await loadSection(runDir, { ...sec, revised: entry });
      preflight.push(...historical.loadErrors.map(e => `historical:${id}:${e}`));
      if (!historical.revised) { preflight.push(`historical_revision_missing:${id}`); continue; }
      const h = historical.revised;
      (historicalRevisions[id] ??= []).push({ baseReceiptSha256: h.baseReceiptSha256,
        baseReceiptRawBytes: historical.receiptBytes, revisionArtifact: h.revision as any,
        freshAudit: h.freshAudit, freshJev: h.freshJev,
        expectedJevMode: h.expectedJevMode, guardrailDisposition: h.guardrailDisposition });
    }
  }
  let coherenceReview: EditorialCoherenceReview | undefined;
  if (manifest.coherenceReviewFile) {
    const cohPath = abs(runDir, manifest.coherenceReviewFile);
    if (existsSync(cohPath)) {
      try { coherenceReview = JSON.parse(await readBytes(cohPath)); }
      catch (e: unknown) {
        const msg = e instanceof Error ? e.message : String(e);
        preflight.push(`coherence_review_parse_error:${cohPath}:${msg}`);
      }
    } else {
      preflight.push(`coherence_review_missing:${cohPath}`);
    }
  } else if (manifest.mode === 'full') {
    preflight.push('coherence_review_file_missing_for_full_mode');
  }
  const originalPasses: Record<string, string> = {};
  for (const l of loaded) if (!l.missing) originalPasses[l.id] = l.receipt.raw.synthesis;
  const priorContext: Record<string, Record<string, PriorContextEdge>> = {};
  for (const l of loaded) if (l.priorContext && !l.missing) priorContext[l.id] = l.priorContext;
  const editorial: EditorialVerificationOptions | undefined =
    (Object.keys(revisedSections).length > 0 || coherenceReview !== undefined || manifest.mode === 'full')
      ? { revisedSections, historicalRevisions, originalPasses, coherenceReview, implementationSourceTexts, priorContext,
          trustedPassageSources,
          expectedCF: manifest.cloudflare ? {account:manifest.cloudflare.accountId,index:manifest.cloudflare.indexName ?? 'witness-wisdom-corpus'} : undefined }
      : undefined;

  // ── Render / parity evidence — strict schema, bytes-first ──────────
  //
  // Doctrine:
  //   * Render: we resolve BOTH the QA-report file bytes and the actual
  //     DOCX/PDF/HTML artifact bytes.  Fabricated hash-matching text is
  //     rejected by runFinalReview because the artifact bytes must hash
  //     to `artifact.sha256` AND `bindsAssembledSha256` must equal
  //     sha256(assembled).  We surface a preflight blocker on any read
  //     failure so the CLI reports it before the verifier runs.
  //   * Parity: we resolve every language artifact declared in
  //     `artifacts` and preserve the declared `languages` list so the
  //     verifier can reject a single-file parity pointer.
  let renderReceipt: FinalReviewInput['renderReceipt'];
  if (manifest.renderReceipt) {
    const r = manifest.renderReceipt;
    const pointerPath = abs(runDir, r.path);
    const pointerRaw = existsSync(pointerPath) ? await readBytes(pointerPath) : undefined;
    if (pointerRaw === undefined) preflight.push(`render_file_missing:${pointerPath}`);
    let artifactBytes: Uint8Array | undefined;
    if (r.artifact && typeof r.artifact === 'object' && typeof r.artifact.path === 'string') {
      const artPath = abs(runDir, r.artifact.path);
      if (existsSync(artPath)) artifactBytes = await fs.readFile(artPath);
      else preflight.push(`render_artifact_file_missing:${artPath}`);
    } else {
      preflight.push('render_artifact_pointer_missing');
    }
    renderReceipt = {
      path: r.path,
      sha256: r.sha256,
      rawBytes: pointerRaw,
      kind: r.kind,
      artifact: {
        path: r.artifact?.path ?? '',
        sha256: r.artifact?.sha256 ?? '',
        kind: r.artifact?.kind,
        rawBytes: artifactBytes ?? '',
      },
      bindsAssembledSha256: r.bindsAssembledSha256,
      status: r.status,
      reviewer: r.reviewer,
      reviewedAt: r.reviewedAt,
      findings: r.findings,
    };
  }
  let parityReceipt: FinalReviewInput['parityReceipt'];
  if (manifest.parityReceipt) {
    const pr = manifest.parityReceipt;
    const pointerPath = abs(runDir, pr.path);
    const pointerRaw = existsSync(pointerPath) ? await readBytes(pointerPath) : undefined;
    if (pointerRaw === undefined) preflight.push(`parity_file_missing:${pointerPath}`);
    const artifacts: Record<string, { path: string; sha256: string; kind?: string; rawBytes: string | Uint8Array }> = {};
    if (pr.artifacts && typeof pr.artifacts === 'object') {
      for (const [lang, art] of Object.entries(pr.artifacts)) {
        const artPath = abs(runDir, art.path);
        let bytes: Uint8Array = new Uint8Array();
        if (existsSync(artPath)) bytes = await fs.readFile(artPath);
        else preflight.push(`parity_artifact_file_missing:${lang}:${artPath}`);
        artifacts[lang] = { path: art.path, sha256: art.sha256, kind: art.kind, rawBytes: bytes };
      }
    } else {
      preflight.push('parity_artifacts_pointer_missing');
    }
    parityReceipt = {
      path: pr.path,
      sha256: pr.sha256,
      rawBytes: pointerRaw,
      kind: pr.kind,
      languages: pr.languages ?? [],
      artifacts,
      status: pr.status,
      reviewer: pr.reviewer,
      reviewedAt: pr.reviewedAt,
      findings: pr.findings,
      bindsAssembledSha256: pr.bindsAssembledSha256,
    };
  }

  const input: FinalReviewInput = {
    mode: manifest.mode,
    requiredSectionIds,
    receipts,
    passes,
    assembled,
    expectedPersonas,
    subsectionMap,
    rawSynthesisInputs,
    sourceTexts: sourceTextsById,
    sectionEngineFacts,
    ...(editorial ? { editorial } : {}),
    ...(cloudflareSnapshot ? { cloudflareSnapshot } : {}),
    ...(renderReceipt ? { renderReceipt } : {}),
    ...(parityReceipt ? { parityReceipt } : {}),
    bilingualExpected: !!manifest.bilingualExpected,
    ...(declaredPrimaryIds.length > 0 ? { primaryPassageIds: declaredPrimaryIds } : {}),
    ...(primaryRegistryBytes && primaryRegistrySha256 && primaryRegistryPath
      ? { primaryPassageRegistry: { path: primaryRegistryPath, sha256: primaryRegistrySha256, rawBytes: primaryRegistryBytes } }
      : {}),
  };

  let result: FinalReviewResult;
  try {
    result = runFinalReview(input);
  } catch (e: any) {
    const err = { finalAccepted: false, error: String(e?.message ?? e), preflight, mode: manifest.mode };
    console.log(JSON.stringify(err, null, 2));
    if (!dryRun) writeOutput(manifest, runDir, err);
    return 1;
  }

  // Fold preflight blockers into the final report so partial-manifest runs
  // surface everything, and finalAccepted stays false.
  const finalReport = {
    schemaVersion: 1,
    mode: manifest.mode,
    manifestPath,
    manifestSha256: sha256(manifestBytes),
    finalAccepted: preflight.length === 0 && result.finalAccepted,
    generationAccepted: result.generationAccepted,
    sourceAccepted: result.sourceAccepted,
    referenceRoutePassed: result.referenceRoutePassed,
    packagingAccepted: result.packagingAccepted,
    artifactSha256: result.artifactSha256,
    blockers: [...preflight, ...result.blockers],
    warnings: result.warnings,
    packagingReport: result.packagingReport,
    referenceReportBlockers: result.referenceReport.blockers,
    editorialVerdicts: result.referenceReport.editorialVerdicts ?? [],
    rubricSources,
    canonicalModeLoaded: canonical !== undefined,
    assembledSha256,
    generatedAt: new Date().toISOString(),
  };

  console.log(JSON.stringify({
    finalAccepted: finalReport.finalAccepted,
    generationAccepted: finalReport.generationAccepted,
    sourceAccepted: finalReport.sourceAccepted,
    referenceRoutePassed: finalReport.referenceRoutePassed,
    packagingAccepted: finalReport.packagingAccepted,
    blockerCount: finalReport.blockers.length,
    firstBlockers: finalReport.blockers.slice(0, 8),
  }, null, 2));

  if (!dryRun) writeOutput(manifest, runDir, finalReport);
  return finalReport.finalAccepted ? 0 : 1;
}

function writeOutput(manifest: Manifest, runDir: string, report: unknown): void {
  const isoStamp = new Date().toISOString().replace(/[:.]/g, '-');
  const defaultName = `final-verification-${isoStamp}.json`;
  const target = outOverride
    ? resolve(outOverride)
    : resolve(runDir, manifest.outputFile ?? defaultName);
  // NEVER overwrite an existing receipt: use `wx` flag.  The `isoStamp`
  // guarantees uniqueness on collision.
  writeFileSync(target, JSON.stringify(report, null, 2) + '\n', { flag: 'wx' });
  console.error(`wrote ${target}`);
}

main().then(code => process.exit(code)).catch(err => {
  console.error('verify-reference-artifact fatal:', err?.stack ?? err);
  process.exit(2);
});
