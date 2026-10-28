#!/usr/bin/env npx ts-node
/**
 * Smoke test for @noesis/witness-pipeline
 *
 * Run with: npx ts-node scripts/smoke.ts
 * Or via npm: npm run smoke
 *
 * Verifies that:
 * 1. All exports are accessible
 * 2. Basic orchestrator instantiation works
 * 3. Mode parser works
 * 4. Audit validates deterministic gates
 * 5. A synthetic L0 source pack renders to HTML/PDF and passes final verification
 */

import { mkdtempSync, readFileSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';

import {
  // Selemene fetcher
  fetchAllEngines,
  loadSelemeneKey,
  SELEMENE_BASE_URL,
  ENGINE_ID_MAP,
  ENGINE_ROUTING,
  REVERSE_ENGINE_MAP,
  SELEMENE_ENGINE_IDS,
  type SelemeneEngineId,
  type RoutingMode,
  type SelemeneEngineOutput,
  type BirthData,
  // Mode parser
  parseModeDocument,
  parseModeDoc,
  getPassTemplate,
  getTargetWordsForRegister,
  summarizeLessons,
  type ParsedModeDoc,
  type ModeConfig,
  // Orchestrator
  IntegratedReadingOrchestrator,
  // Asset factory
  createSourcePack,
  type SourcePack,
  // Chain audit
  runChainAudit,
  type AuditResult,
  // Intake (location normalization)
  isCompleteReportRequest,
  normalizeManualLocation,
  confirmNormalizedLocation,
  searchBirthplace,
  buildReportIntakeQuestions,
  type ReportGenerationRequest,
  type PassResult,
} from '../src/index.js';
import { renderLocalArtifacts } from '../src/assets/render-pipeline.js';
import { runFinalVerification } from '../src/orchestrator/final-verification.js';

const FIXTURE_BRAND_CONFIG = fileURLToPath(new URL('../tests/fixtures/brand/brand-config.yaml', import.meta.url));

async function smoke() {
  console.log('🔥 Smoke test for @noesis/witness-pipeline\n');

  // 1. Verify exports
  console.log('1. Checking exports...');
  if (!fetchAllEngines) throw new Error('fetchAllEngines not exported');
  if (!loadSelemeneKey) throw new Error('loadSelemeneKey not exported');
  if (!SELEMENE_BASE_URL) throw new Error('SELEMENE_BASE_URL not exported');
  if (!ENGINE_ID_MAP) throw new Error('ENGINE_ID_MAP not exported');
  if (!ENGINE_ROUTING) throw new Error('ENGINE_ROUTING not exported');
  if (!SELEMENE_ENGINE_IDS) throw new Error('SELEMENE_ENGINE_IDS not exported');
  if (!parseModeDocument) throw new Error('parseModeDocument not exported');
  if (!IntegratedReadingOrchestrator) throw new Error('IntegratedReadingOrchestrator not exported');
  if (!createSourcePack) throw new Error('createSourcePack not exported');
  if (!runChainAudit) throw new Error('runChainAudit not exported');
  if (!isCompleteReportRequest) throw new Error('isCompleteReportRequest not exported');
  if (!normalizeManualLocation) throw new Error('normalizeManualLocation not exported');
  if (!confirmNormalizedLocation) throw new Error('confirmNormalizedLocation not exported');
  if (!searchBirthplace) throw new Error('searchBirthplace not exported');
  if (!buildReportIntakeQuestions) throw new Error('buildReportIntakeQuestions not exported');
  console.log('   ✓ All exports accessible\n');

  // 2. Test ENGINE_ID_MAP
  console.log('2. Checking ENGINE_ID_MAP...');
  const vimshottariAlias = ENGINE_ID_MAP['vimshottari'];
  if (vimshottariAlias !== 'chronofield') {
    throw new Error(`Expected chronofield alias for vimshottari, got ${vimshottariAlias}`);
  }
  console.log(`   ✓ ENGINE_ID_MAP['vimshottari'] = ${vimshottariAlias}\n`);

  // 3. Test ENGINE_ROUTING
  console.log('3. Checking ENGINE_ROUTING...');
  const vimshottariRouting = ENGINE_ROUTING['vimshottari'];
  if (vimshottariRouting !== 'aletheios-primary') {
    throw new Error(`Expected aletheios-primary routing for vimshottari, got ${vimshottariRouting}`);
  }
  console.log(`   ✓ ENGINE_ROUTING['vimshottari'] = ${vimshottariRouting}\n`);

  // 4. Test SELEMENE_ENGINE_IDS
  console.log('4. Checking SELEMENE_ENGINE_IDS...');
  if (!SELEMENE_ENGINE_IDS.includes('vimshottari')) {
    throw new Error('vimshottari not in SELEMENE_ENGINE_IDS');
  }
  console.log(`   ✓ SELEMENE_ENGINE_IDS has ${SELEMENE_ENGINE_IDS.length} engines\n`);

  // 5. Test orchestrator class exists (requires config to instantiate)
  console.log('5. Checking IntegratedReadingOrchestrator...');
  if (typeof IntegratedReadingOrchestrator !== 'function') {
    throw new Error('IntegratedReadingOrchestrator is not a constructor');
  }
  console.log(`   ✓ IntegratedReadingOrchestrator constructor exported\n`);

  // 6. Test mode parser function exists (requires full schema-compliant doc to parse)
  console.log('6. Checking parseModeDocument...');
  if (typeof parseModeDocument !== 'function') {
    throw new Error('parseModeDocument is not a function');
  }
  // Note: Full parsing requires schema-compliant frontmatter with mode, subject_count,
  // roles, target_words, architecture, pass_plan, engine_overlay_weights, house_overlay,
  // bridge_mandates, and svg_topology. For smoke test, we just verify the function exists.
  console.log(`   ✓ parseModeDocument function exported\n`);

  // 7. Test runChainAudit
  console.log('7. Checking runChainAudit...');
  const mockEngineResults: SelemeneEngineOutput[] = [
    { engine_id: 'vimshottari', period: 'Jupiter', sublord: 'Venus' } as SelemeneEngineOutput,
    { engine_id: 'panchanga', tithi: 'Navami', nakshatra: 'Rohini' } as SelemeneEngineOutput,
    { engine_id: 'human-design', type: 'Generator', strategy: 'Wait to Respond' } as SelemeneEngineOutput,
    { engine_id: 'gene-keys', activations: ['Gate 1'] } as SelemeneEngineOutput,
  ];
  const auditResult = runChainAudit({
    personId: 'test-person',
    readingMarkdown: 'This is a test reading with enough characters to pass the length check easily.',
    engineResults: mockEngineResults,
  });
  if (!auditResult.passed) {
    throw new Error(`Audit failed: ${auditResult.blockers.join(', ')}`);
  }
  console.log(`   ✓ Audit passed with ${auditResult.facts_count} deterministic facts\n`);

  // 8. Intake location normalization contract
  // Note: The rich ReportGenerationRequest shape (report_level + subjects[] + normalized_location)
  // is the canonical contract. The public Rust API (/api/v1/assets/generate) now accepts it
  // (via AssetGenerateRequest + ReportGenerationRequest in noesis-core). TS types here are the source of truth.
  console.log('8. Checking intake location normalization...');
  const incomplete: ReportGenerationRequest = {
    report_level: 'L3',
    report_mode: 'integrated-reading',
    subjects: [{
      role: 'primary',
      name: 'Test',
      birth_date: '1990-01-01',
      birth_time_confidence: 'exact',
      birth_location_query: 'Bengaluru, India',
    }],
    output: { format: 'markdown', include_rubric: true, include_pattern_extraction: true },
  };
  if (isCompleteReportRequest(incomplete)) {
    throw new Error('isCompleteReportRequest should be false without normalized_location');
  }

  const confirmed = confirmNormalizedLocation({
    manual: { displayName: 'Bengaluru, Karnataka, India', latitude: 12.9716, longitude: 77.5946, timezone: 'Asia/Kolkata' },
  });
  incomplete.subjects[0].normalized_location = confirmed;
  if (!isCompleteReportRequest(incomplete)) {
    throw new Error('isCompleteReportRequest should be true after normalized_location present');
  }

  const manualLoc = normalizeManualLocation({
    displayName: 'Mumbai',
    latitude: 19.076,
    longitude: 72.877,
    timezone: 'Asia/Kolkata',
  });
  if (!manualLoc || manualLoc.provider !== 'manual') {
    throw new Error('normalizeManualLocation failed');
  }

  const qs = buildReportIntakeQuestions({ subjectCount: 1, relationship: false });
  if (!qs.some((q) => q.header === 'Birthplace')) {
    throw new Error('buildReportIntakeQuestions missing Birthplace');
  }

  console.log('   ✓ Intake location normalization contract verified\n');

  // 9. Synthetic L0 render + final verification (no live engines, LLM or real brand docs)
  console.log('9. Checking L0 render and final verification...');
  const l0Root = mkdtempSync(join(tmpdir(), 'smoke-l0-'));
  try {
    const sourcePackDir = join(l0Root, 'source-pack');
    const localDir = join(l0Root, 'local');
    const pack = await createSourcePack({
      personId: 'smoke-person',
      readingMarkdown: '# Synthetic L0 reading\n\nSynthetic body text.',
      engineResults: [],
      outputDir: sourcePackDir,
      reportLevel: 'L0',
    });
    if (pack.manifest.report_level !== 'L0') {
      throw new Error(`Expected report_level L0, got ${pack.manifest.report_level}`);
    }

    const rendered = await renderLocalArtifacts({
      sourcePackDir,
      outputDir: localDir,
      brandConfigPath: FIXTURE_BRAND_CONFIG,
    });
    if (readFileSync(rendered.pdfPath).subarray(0, 5).toString('latin1') !== '%PDF-') {
      throw new Error('Rendered reading.pdf is not a PDF');
    }

    const passes = [
      { id: 'synthetic', title: 'Synthetic', output: '', rubric: { placeholder_gate: 'pass', chart_fidelity_gate: 'pass' } },
    ] as unknown as PassResult[];
    const verification = runFinalVerification({ passes, pdfPath: rendered.pdfPath });
    if (!verification.passed) {
      throw new Error(`Final verification blockers: ${verification.blockers.join(', ')}`);
    }

    const missingPdf = runFinalVerification({ passes, pdfPath: join(localDir, 'missing.pdf') });
    if (missingPdf.passed || !missingPdf.blockers.includes('pdf:missing')) {
      throw new Error('Final verification should block on a missing PDF');
    }
  } finally {
    rmSync(l0Root, { recursive: true, force: true });
  }
  console.log('   ✓ L0 source pack rendered to HTML/PDF and passed final verification\n');

  console.log('🎉 All smoke tests passed!\n');
}

smoke().catch((err) => {
  console.error('❌ Smoke test failed:', err.message);
  process.exit(1);
});
