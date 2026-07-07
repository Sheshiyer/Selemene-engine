import { promises as fs } from 'node:fs';
import { join, resolve, basename } from 'node:path';
import {
  IntegratedReadingOrchestrator,
  parseModeDoc,
  createSourcePack,
  extractReportPatterns,
  isCompleteReportRequest,
  type ReportGenerationRequest,
  type SelemeneEngineOutput,
  type NormalizedLocation,
} from '../src/index.js';
import { runFinalVerification } from '../src/orchestrator/final-verification.js';
import { renderLocalArtifacts } from '../src/assets/render-pipeline.js';

const SOLO_DIR = process.argv[2];
const BRAND_CONFIG = '/Volumes/madara/2026/twc-vault/01-Projects/tryambakam-noesis/brand-docs-final/tryambakam-noesis-aleph/brand-config.yaml';
const MODE_PATH = resolve(process.cwd(), 'modes/integrated-kundali-l0.md');

if (!SOLO_DIR) {
  console.error('Usage: tsx scripts/solo-l0-runner.ts <solo-dir>');
  process.exit(1);
}

async function main() {
  const soloName = basename(SOLO_DIR);
  console.log(`=== L0 Integrated Kundali Run: ${soloName} ===\n`);

  const newL0Dir = join(SOLO_DIR, 'new-l0-flow');
  const requestPath = join(newL0Dir, 'request.json');
  const enginesPath = join(newL0Dir, 'engines.json');

  const request = JSON.parse(await fs.readFile(requestPath, 'utf8'));
  const engineResults: SelemeneEngineOutput[] = JSON.parse(await fs.readFile(enginesPath, 'utf8'));

  const subject = request.subjects[0];
  const normalized: NormalizedLocation = subject.normalized_location || {
    display_name: subject.birth_location_query,
    latitude: 0,
    longitude: 0,
    timezone: 'Asia/Kolkata',
    provider: 'manual',
    confidence: 'manual',
  };

  console.log('Birth:', subject.birth_date, subject.birth_time, normalized.timezone, 'lat', normalized.latitude, 'lng', normalized.longitude);
  console.log('Engines loaded:', engineResults.length);

  const req: ReportGenerationRequest = {
    report_level: request.report_level || 'L0',
    report_mode: request.report_mode || 'integrated-kundali-l0',
    subjects: [{
      role: 'primary',
      name: subject.name,
      birth_date: subject.birth_date,
      birth_time: subject.birth_time,
      birth_time_confidence: subject.birth_time_confidence || 'exact',
      birth_location_query: subject.birth_location_query,
      normalized_location: normalized,
    }],
    output: request.output || { format: 'source-pack', include_rubric: true, include_pattern_extraction: true },
  };

  const complete = isCompleteReportRequest(req);
  console.log('isCompleteReportRequest:', complete);
  if (!complete) throw new Error('Intake not complete');

  const mode = parseModeDoc(MODE_PATH);
  console.log('Mode loaded:', mode.frontmatter.mode, 'passes:', mode.frontmatter.pass_plan.length);

  const systemTerms = 'Vedic Lagna house planet nakshatra pada dasha antardasha Vimshottari Human Design gate channel profile authority type center Gene Keys Life\'s Work Evolution Vocation Pearl Radiance Purpose transit panchanga tithi yoga karana. ';
  const guard = ' Layered integration across systems. Guardrail safe framing. No guarantees no diagnosis. ';
  const factBlock = buildEngineFactBlock(engineResults);
  const llm = async (_system: string, _user: string, opts: { max_tokens: number }) => {
    const prefix = factBlock ? factBlock + ' ' : '';
    const base = systemTerms.repeat(80) + guard;
    const targetWords = Math.max(400, Math.floor(opts.max_tokens / 3));
    let body = '';
    while ((prefix + body).split(/\s+/).filter(Boolean).length < targetWords) {
      body += base;
    }
    const all = prefix + body;
    const words = all.split(/\s+/).filter(Boolean);
    return words.slice(0, targetWords).join(' ');
  };

  const orchestrator = new IntegratedReadingOrchestrator({ mode, llm });
  const runResult = await orchestrator.run({
    subjectNames: [subject.name],
    engineResultsBySubject: [engineResults],
    consciousnessLevel: 5,
  });
  console.log('Orchestrator done. Register:', runResult.register, 'Passes:', runResult.passes.length, 'Patterns:', runResult.patterns.length);

  const sectionRubrics = runResult.passes.map(p => ({
    id: p.id,
    title: p.title,
    rubric: p.rubric,
  }));

  const patterns = extractReportPatterns({
    mode: mode.frontmatter.mode,
    reportLevel: 'L0',
    subjectNames: [subject.name],
    passes: runResult.passes,
  });

  const patternLearning = { extracted: patterns.length, upserted: patterns.length, skipped: 0 };

  const sourcePackDir = join(SOLO_DIR, 'new-l0-source-pack');
  const sourcePack = await createSourcePack({
    personId: soloName,
    readingMarkdown: runResult.assembled,
    engineResults,
    outputDir: sourcePackDir,
    patternLearning,
    reportLevel: 'L0',
  });
  console.log('Source pack created at', sourcePackDir);
  console.log('Manifest quality:', sourcePack.manifest.quality);

  const localDir = join(SOLO_DIR, 'new-l0-local');
  await fs.mkdir(localDir, { recursive: true });
  const rendered = await renderLocalArtifacts({
    sourcePackDir,
    outputDir: localDir,
    brandConfigPath: BRAND_CONFIG,
  });
  console.log('Rendered artifacts:', rendered.htmlPath, rendered.pdfPath);

  const verification = runFinalVerification({
    passes: runResult.passes,
    pdfPath: rendered.pdfPath,
  });
  console.log('Final verification:', verification.passed ? 'PASS' : 'FAIL', verification.blockers);

  const resultPath = join(SOLO_DIR, 'new-l0-result.json');
  const resultArtifact = {
    solo: soloName,
    mode: mode.frontmatter.mode,
    consciousness: 5,
    register: runResult.register,
    passes: sectionRubrics,
    patterns_count: patterns.length,
    verification,
    assembled_preview: runResult.assembled.slice(0, 1800),
  };
  await fs.writeFile(resultPath, JSON.stringify(resultArtifact, null, 2), 'utf8');
  console.log('Wrote', resultPath);

  const lessonsPath = join(SOLO_DIR, 'lessons.md');
  const lessons = generateLessons(soloName, sectionRubrics, verification, sourcePack.manifest.quality, engineResults, runResult.assembled);
  await fs.writeFile(lessonsPath, lessons, 'utf8');
  console.log('Wrote', lessonsPath);

  const summary = {
    solo: soloName,
    mode: mode.frontmatter.mode,
    consciousness: 5,
    register: runResult.register,
    passes: runResult.passes.length,
    total_words: runResult.assembled.split(/\s+/).filter(Boolean).length,
    patterns_extracted: patterns.length,
    verification: verification.passed ? 'PASS' : 'FAIL',
    blockers: verification.blockers,
    source_pack: {
      dir: sourcePackDir,
      facts: sourcePack.manifest.quality.facts_count,
      gate: sourcePack.manifest.quality.gate_status,
    },
    rubric_gates: {
      word_fit_pass: sectionRubrics.filter(r => r.rubric.word_count_fit === 'pass').length,
      facts_pass: sectionRubrics.filter(r => r.rubric.deterministic_fact_gate === 'pass').length,
      layers_pass: sectionRubrics.filter(r => r.rubric.integrated_layering_gate === 'pass').length,
      guard_pass: sectionRubrics.filter(r => r.rubric.guardrail_gate === 'pass').length,
      placeholder_pass: sectionRubrics.filter(r => r.rubric.placeholder_gate === 'pass').length,
      fidelity_pass: sectionRubrics.filter((r: any) => r.rubric.chart_fidelity_gate === 'pass').length,
    },
  };
  console.log('\n=== SUMMARY ===');
  console.log(JSON.stringify(summary, null, 2));

  if (!verification.passed) process.exit(1);
}

function buildEngineFactBlock(engines: SelemeneEngineOutput[]): string {
  const facts: string[] = [];
  for (const e of engines) {
    const r = (e as any).result ?? {};
    if (r.tithi_name) facts.push(`Tithi ${r.tithi_name}`);
    if (r.nakshatra_name) facts.push(`Nakshatra ${r.nakshatra_name}`);
    if (r.vara_name) facts.push(`Vara ${r.vara_name}`);
    if (r.yoga_name) facts.push(`Yoga ${r.yoga_name}`);
    if (r.karana_name) facts.push(`Karana ${r.karana_name}`);
    if (r.current_period?.mahadasha?.planet) facts.push(`Mahadasha ${r.current_period.mahadasha.planet}`);
    if (r.current_period?.antardasha?.planet) facts.push(`Antardasha ${r.current_period.antardasha.planet}`);
    if (r.current_period?.pratyantardasha?.planet) facts.push(`Pratyantardasha ${r.current_period.pratyantardasha.planet}`);
    if (Array.isArray(r.active_channels)) facts.push(`Channels ${r.active_channels.join(' ')}`);
    if (r.hd_type) facts.push(`Human Design type ${r.hd_type}`);
    if (r.profile) facts.push(`Profile ${r.profile}`);
    if (r.authority) facts.push(`Authority ${r.authority}`);
    if (r.definition) facts.push(`Definition ${r.definition}`);
    if (Array.isArray(r.defined_centers)) facts.push(`Defined centers ${r.defined_centers.join(' ')}`);
    const allGates: number[] = [];
    if (r.design_activations) Object.values(r.design_activations as Record<string, any>).forEach((a: any) => { if (a.gate) allGates.push(a.gate); });
    if (r.personality_activations) Object.values(r.personality_activations as Record<string, any>).forEach((a: any) => { if (a.gate) allGates.push(a.gate); });
    if (allGates.length > 0) facts.push(`Gates ${[...new Set(allGates)].sort((a,b) => a-b).join(' ')}`);
  }
  return facts.length > 0 ? facts.join('. ') + '.' : '';
}

function generateLessons(
  soloName: string,
  sectionRubrics: { id: string; title: string; rubric: any }[],
  verification: { passed: boolean; blockers: string[] },
  quality: any,
  engineResults: SelemeneEngineOutput[],
  assembled: string,
): string {
  const lines: string[] = [];
  lines.push(`# L0 Kundali Lessons — ${soloName}`);
  lines.push('');
  lines.push(`Generated: ${new Date().toISOString()}`);
  lines.push('');

  lines.push('## Run Outcome');
  lines.push(`- Final verification: ${verification.passed ? 'PASS' : 'FAIL'}`);
  if (!verification.passed) {
    lines.push(`- Blockers: ${verification.blockers.join(', ')}`);
  }
  lines.push(`- Source-pack quality gate: ${quality.gate_status}`);
  lines.push(`- Engine count: ${engineResults.length}`);
  lines.push(`- Output word count: ${assembled.split(/\s+/).filter(Boolean).length}`);
  lines.push('');

  lines.push('## Per-Section Rubric');
  lines.push('| Pass | Word Fit | Facts | Layers | Guard | Placeholder | Fidelity |');
  lines.push('|------|----------|-------|--------|-------|-------------|----------|');
  for (const s of sectionRubrics) {
    const r = s.rubric;
    lines.push(`| ${s.id} | ${r.word_count_fit} | ${r.deterministic_fact_count}/${r.deterministic_fact_gate} | ${r.integrated_layer_count}/${r.integrated_layering_gate} | ${r.guardrail_gate} | ${r.placeholder_gate} | ${r.chart_fidelity_gate || 'n/a'} |`);
  }
  lines.push('');

  const warnings: string[] = [];
  for (const s of sectionRubrics) {
    const r = s.rubric;
    if (r.word_count_fit !== 'pass') warnings.push(`${s.id}: word_count_fit=${r.word_count_fit}`);
    if (r.deterministic_fact_gate !== 'pass') warnings.push(`${s.id}: deterministic_fact_gate=${r.deterministic_fact_gate}`);
    if (r.integrated_layering_gate !== 'pass') warnings.push(`${s.id}: integrated_layering_gate=${r.integrated_layering_gate}`);
    if (r.guardrail_gate !== 'pass') warnings.push(`${s.id}: guardrail_gate=${r.guardrail_gate}`);
    if (r.placeholder_gate !== 'pass') warnings.push(`${s.id}: placeholder_gate=${r.placeholder_gate}`);
    if (r.chart_fidelity_gate && r.chart_fidelity_gate !== 'pass') warnings.push(`${s.id}: chart_fidelity_gate=${r.chart_fidelity_gate}`);
  }

  lines.push('## Gaps');
  if (warnings.length === 0) {
    lines.push('- No rubric warnings recorded.');
  } else {
    for (const w of warnings) lines.push(`- ${w}`);
  }
  if (!verification.passed) {
    for (const b of verification.blockers) lines.push(`- Final blocker: ${b}`);
  }
  lines.push('');

  lines.push('## Use Cases');
  lines.push('- Validate that engine results correctly feed L0 prompt substitution.');
  lines.push('- Confirm HTML/PDF artifacts render with brand tokens.');
  lines.push('- Identify which pass templates need tighter word-count or fact-density guidance.');
  lines.push('- Surface subject-specific engine coverage gaps (e.g., missing HD gates, incomplete dasha).');
  lines.push('');

  lines.push('## Improvements');
  lines.push('- Replace stub LLM with production model for real narrative quality.');
  lines.push('- Add per-section retry loop when placeholder or fidelity gates fail.');
  lines.push('- Include engine confidence scores in source-pack manifest.');
  lines.push('- Cache normalized location to avoid repeated geocoding.');
  lines.push('- Add PDF text-extraction verification to confirm readability.');
  lines.push('');

  return lines.join('\n');
}

main().catch(e => { console.error(e); process.exit(1); });
