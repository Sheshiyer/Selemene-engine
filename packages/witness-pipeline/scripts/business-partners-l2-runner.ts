/**
 * L2 business-partners synastry runner (two subjects, relationship-aware).
 *
 * Usage:
 *   tsx scripts/business-partners-l2-runner.ts <run-dir> [--live] [--source prod|local] [--jev off|shadow|active]
 *
 * Expects <run-dir>/request.json (ReportGenerationRequest with exactly two subjects and
 * relationship_context.type = "business-partners") and <run-dir>/engines/<slug>.<source>.json
 * per subject, where <slug> is request.subjects[i].slug (or a slugified name).
 *
 * Writes: reading source pack, local HTML/PDF, result.json, summary.json, lessons.md.
 */
import { promises as fs } from 'node:fs';
import { existsSync } from 'node:fs';
import { join, resolve } from 'node:path';
import {
  IntegratedReadingOrchestrator,
  parseModeDoc,
  createSourcePack,
  extractReportPatterns,
  isCompleteReportRequest,
  normalizeManualLocation,
  type ReportGenerationRequest,
  type SelemeneEngineOutput,
} from '../src/index.js';
import { buildEngineFactsBlock } from '../src/orchestrator/engine-facts.js';
import { runFinalVerification } from '../src/orchestrator/final-verification.js';
import { renderLocalArtifacts } from '../src/assets/render-pipeline.js';
import { createLlmCall } from './lib/multi-llm.js';
import { createJevClient, loadTypesafeKey } from '../src/jev/client.js';
import { createJevPassGate, type JevGateMode } from '../src/jev/pass-gate.js';

const RUN_DIR = process.argv[2];
const LIVE = process.argv.includes('--live');
const srcIdx = process.argv.indexOf('--source');
const SOURCE = srcIdx > -1 ? process.argv[srcIdx + 1] : 'prod';
const jevIdx = process.argv.indexOf('--jev');
const JEV_MODE = (jevIdx > -1 ? process.argv[jevIdx + 1] : 'shadow') as JevGateMode;
const MODE_PATH = resolve(process.cwd(), 'modes/business-partners.md');
const BRAND_CONFIG = process.env.BRAND_CONFIG
  ?? '/Volumes/madara/2026/twc-vault/01-Projects/tryambakam-noesis/brand-docs-final/tryambakam-noesis-aleph/brand-config.yaml';

if (!RUN_DIR) {
  console.error('Usage: tsx scripts/business-partners-l2-runner.ts <run-dir> [--live] [--source prod|local]');
  process.exit(1);
}

const slugify = (s: string) => s.toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/^-|-$/g, '');

async function main() {
  const raw = JSON.parse(await fs.readFile(join(RUN_DIR, 'request.json'), 'utf8'));
  if (!Array.isArray(raw.subjects) || raw.subjects.length !== 2) throw new Error('business-partners mode needs exactly two subjects');
  if (raw.relationship_context?.type !== 'business-partners') throw new Error('relationship_context.type must be business-partners');

  const req: ReportGenerationRequest = {
    report_level: raw.report_level ?? 'L2',
    report_mode: raw.report_mode ?? 'business-partners',
    language: raw.language ?? 'en',
    relationship_context: raw.relationship_context,
    subjects: raw.subjects.map((s: any) => ({
      role: s.role ?? 'business-partner',
      relationship_label: s.relationship_label,
      name: s.name,
      birth_date: s.birth_date,
      birth_time: s.birth_time,
      birth_time_confidence: s.birth_time_confidence ?? 'exact',
      birth_location_query: s.birth_location_query,
      normalized_location: s.normalized_location ?? normalizeManualLocation({
        displayName: s.birth_location_query,
        latitude: s.latitude,
        longitude: s.longitude,
        timezone: s.timezone ?? 'Asia/Kolkata',
      }),
    })),
    output: raw.output ?? { format: 'source-pack', include_rubric: true, include_pattern_extraction: true },
  };
  if (!isCompleteReportRequest(req)) throw new Error('Intake not complete (normalized_location missing)');

  const engineResultsBySubject: SelemeneEngineOutput[][] = [];
  for (const s of raw.subjects) {
    const slug = s.slug ?? slugify(s.name);
    const path = join(RUN_DIR, 'engines', `${slug}.${SOURCE}.json`);
    const engines: SelemeneEngineOutput[] = JSON.parse(await fs.readFile(path, 'utf8'));
    const ok = engines.filter((e) => !e._error);
    console.log(`Engines for ${s.name}: ${ok.length}/${engines.length} ok (${SOURCE}) from ${path}`);
    engineResultsBySubject.push(engines);
  }

  const mode = parseModeDoc(MODE_PATH);
  console.log('Mode:', mode.frontmatter.mode, 'level', (mode.frontmatter as any).report_level, 'passes', mode.frontmatter.pass_plan.length);

  const subjectNames = req.subjects.map((s) => s.name);
  const subjectRoles = req.subjects.map((s) => ({ role: s.role, name: s.name, label: s.relationship_label }));
  const consciousnessLevel = raw.consciousness_level ?? 2;

  const factsBlock = buildEngineFactsBlock({ subjectNames, subjectRoles, engineResultsBySubject });
  await fs.writeFile(join(RUN_DIR, 'engine-facts.md'), `# Engine facts\n\n${factsBlock}\n`, 'utf8');

  const llm = LIVE ? createLlmCall({ temperature: 0.7, timeout_ms: 240_000 }) : createStubLLM(factsBlock);
  const jevClient = JEV_MODE === 'off' ? null : createJevClient(await loadTypesafeKey());
  const jevGate = createJevPassGate(jevClient, JEV_MODE);
  console.log('Jev gate:', JEV_MODE, jevClient ? '(client ready)' : '(no TYPESAFE_API_KEY: receipts will be skipped, never counterfeited)');
  const orchestrator = new IntegratedReadingOrchestrator({ mode, llm, jevGate });
  const started = Date.now();
  const run = await orchestrator.run({
    subjectNames,
    subjectRoles,
    relationshipContext: req.relationship_context!,
    language: req.language,
    consciousnessLevel,
    engineResultsBySubject,
  });
  console.log('Orchestrator done in', Date.now() - started, 'ms. Register:', run.register, 'passes:', run.passes.length, 'patterns:', run.patterns.length);
  if (!run.relationship_header || !run.assembled.startsWith(run.relationship_header)) throw new Error('relationship_header missing from top of assembled reading');

  const patterns = extractReportPatterns({
    mode: mode.frontmatter.mode,
    reportLevel: req.report_level,
    subjectNames,
    passes: run.passes,
    language: req.language,
    relationship_type: req.relationship_context!.type,
  });

  const personId = `${req.subjects.map((s) => slugify(s.name.split(' ')[0])).join('-')}-business-partners-l2`;
  const sourcePackDir = join(RUN_DIR, 'source-pack');
  const sourcePack = await createSourcePack({
    personId,
    readingMarkdown: run.assembled,
    engineResults: engineResultsBySubject.flat(),
    outputDir: sourcePackDir,
    patternLearning: { extracted: patterns.length, upserted: 0, skipped: patterns.length },
    reportLevel: req.report_level,
  } as any);
  console.log('Source pack:', sourcePackDir, 'quality:', JSON.stringify(sourcePack.manifest.quality));

  let rendered: { htmlPath: string; pdfPath: string } | null = null;
  let renderError: string | null = null;
  if (existsSync(BRAND_CONFIG)) {
    try {
      const localDir = join(RUN_DIR, 'local');
      await fs.mkdir(localDir, { recursive: true });
      rendered = await renderLocalArtifacts({
        sourcePackDir,
        outputDir: localDir,
        brandConfigPath: BRAND_CONFIG,
        title: run.relationship_header.split('\n')[0].replace(/^#\s*/, ''),
      });
      console.log('Rendered:', rendered.htmlPath, rendered.pdfPath);
    } catch (e: any) {
      renderError = e.message;
      console.warn('Render skipped:', renderError);
    }
  } else {
    renderError = `brand config not found at ${BRAND_CONFIG}`;
    console.warn('Render skipped:', renderError);
  }

  const verification = runFinalVerification({ passes: run.passes, pdfPath: rendered?.pdfPath });
  const romanticLeak = /\b(romantic|romance|lover|marriage|spouse|soulmate)\b/i.test(run.assembled);
  const predictiveLeak = /\b(will (succeed|fail|profit|become rich)|guarantee[ds]?|definitely)\b/i.test(run.assembled);

  const result = {
    run: personId,
    source: SOURCE,
    live: LIVE,
    mode: mode.frontmatter.mode,
    report_level: req.report_level,
    consciousness_level: consciousnessLevel,
    register: run.register,
    relationship_header: run.relationship_header,
    passes: run.passes.map((p) => ({ id: p.id, title: p.title, rubric: p.rubric })),
    patterns_count: patterns.length,
    verification,
    guardrail_scan: { romantic_language: romanticLeak, predictive_language: predictiveLeak },
    total_words: run.assembled.split(/\s+/).filter(Boolean).length,
    source_pack: { dir: sourcePackDir, quality: sourcePack.manifest.quality },
    render: rendered ?? { skipped: renderError },
  };
  await fs.writeFile(join(RUN_DIR, 'result.json'), JSON.stringify(result, null, 2), 'utf8');
  await fs.writeFile(join(RUN_DIR, 'reading.md'), run.assembled, 'utf8');
  if (run.jev_receipts) await fs.writeFile(join(RUN_DIR, 'jev-receipts.json'), JSON.stringify(run.jev_receipts, null, 2), 'utf8');

  const gates = {
    word_fit_pass: result.passes.filter((p) => p.rubric.word_count_fit === 'pass').length,
    facts_pass: result.passes.filter((p) => p.rubric.deterministic_fact_gate === 'pass').length,
    layers_pass: result.passes.filter((p) => p.rubric.integrated_layering_gate === 'pass').length,
    guard_pass: result.passes.filter((p) => p.rubric.guardrail_gate === 'pass').length,
    fidelity_scores: result.passes.map((p) => p.rubric.chart_fidelity_score ?? null),
  };
  const jevSummary = run.jev_receipts
    ? { mode: JEV_MODE, judged: run.jev_receipts.filter((r) => r.status === 'judged').length, skipped: run.jev_receipts.filter((r) => r.status === 'skipped').length, errors: run.jev_receipts.filter((r) => r.status === 'error').length, blocked: run.jev_receipts.filter((r) => r.blocked).length, disagreements: run.jev_receipts.flatMap((r) => r.disagreements) }
    : { mode: 'off' };
  const summary = { run: personId, live: LIVE, source: SOURCE, register: run.register, jev: jevSummary, passes: run.passes.length, total_words: result.total_words, verification: verification.passed ? 'PASS' : 'FAIL', blockers: verification.blockers, guardrail_scan: result.guardrail_scan, gates, quality_gate: sourcePack.manifest.quality.gate_status };
  await fs.writeFile(join(RUN_DIR, 'summary.json'), JSON.stringify(summary, null, 2), 'utf8');
  console.log('\n=== SUMMARY ===\n' + JSON.stringify(summary, null, 2));
  if (!verification.passed || romanticLeak || predictiveLeak) process.exit(1);
}

function createStubLLM(factsBlock: string) {
  const filler = 'Vedic Lagna nakshatra dasha Human Design gate channel profile authority Gene Keys Life\'s Work transit panchanga tithi. Descriptive business-partner pattern witness. No prediction. No diagnosis. ';
  return async (_system: string, _user: string, opts: { max_tokens: number }) => {
    const targetWords = Math.max(200, Math.floor((opts.max_tokens || 600) / 2.2));
    let body = factsBlock.replace(/[#\-]/g, ' ') + ' ';
    while (body.split(/\s+/).length < targetWords) body += filler;
    return body.split(/\s+/).slice(0, targetWords).join(' ');
  };
}

main().catch((e) => { console.error(e); process.exit(1); });
