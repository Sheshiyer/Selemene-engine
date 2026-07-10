import { describe, it, expect, vi } from 'vitest';
import { IntegratedReadingOrchestrator } from './integrated.js';
import { parseModeDocument, parseModeDoc } from '../modes/parser.js';
import { runChainAudit } from '../assets/audit.js';
import { createSourcePack } from '../assets/factory.js';
import type { SelemeneEngineOutput } from '../index.js';
import { mkdtempSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';

const sampleModeDoc = `---
mode: composite-dyad
subject_count:
  min: 2
  max: 2
roles:
  - subject-a
  - subject-b
target_words:
  min: 200
  max: 300
architecture: linear
pass_plan:
  - id: alpha
    title: Structural Field
    target_words: 150
    template: pass-alpha-template
engine_overlay_weights:
  panchanga: 1.0
house_overlay: [1, 4, 7, 10]
bridge_mandates:
  - "Braid Vedic and HD data"
svg_topology: dyad-arc
---

## pass-alpha-template
Write about {{subject_names}} using {{overlay_summary}} and {{bridge_mandates}}.
`;

const fidelityModeDoc = `---
mode: composite-dyad
subject_count:
  min: 1
  max: 1
roles:
  - subject
target_words:
  min: 50
  max: 100
architecture: linear
pass_plan:
  - id: opening
    title: Opening
    target_words: 50
    template: pass-opening-template
engine_overlay_weights:
  panchanga: 1.0
house_overlay: [1]
bridge_mandates:
  - "Use facts"
svg_topology: dyad-arc
---

## pass-opening-template
Lagna {{lagna}} with gate {{gate}} and dasha {{dasha}}.
`;

const mockEngines: SelemeneEngineOutput[] = [
  {
    engine_id: 'panchanga',
    result: { tithi_name: 'Test' },
    witness_prompt: 'Observe.',
    consciousness_level: 2,
    metadata: { calculation_time_ms: 1, backend: 'native', precision_achieved: 'standard', cached: false, timestamp: '2026-06-22T00:00:00Z', engine_version: '1' },
    envelope_version: '1',
  },
  {
    engine_id: 'vimshottari',
    result: { dasha: 'Test' },
    witness_prompt: 'Observe.',
    consciousness_level: 2,
    metadata: { calculation_time_ms: 1, backend: 'native', precision_achieved: 'standard', cached: false, timestamp: '2026-06-22T00:00:00Z', engine_version: '1' },
    envelope_version: '1',
  },
  {
    engine_id: 'human-design',
    result: { type: 'Generator' },
    witness_prompt: 'Observe.',
    consciousness_level: 2,
    metadata: { calculation_time_ms: 1, backend: 'native', precision_achieved: 'standard', cached: false, timestamp: '2026-06-22T00:00:00Z', engine_version: '1' },
    envelope_version: '1',
  },
];

describe('IntegratedReadingOrchestrator end-to-end', () => {
  it('executes full multi-pass flow with real mode doc', async () => {
    const mode = parseModeDocument(sampleModeDoc, 'e2e-test.md');
    const llm = vi.fn().mockResolvedValue('PASS OUTPUT');
    const orchestrator = new IntegratedReadingOrchestrator({ mode, llm });
    const result = await orchestrator.run({
      subjectNames: ['Arathi', 'Rohan'],
      engineResultsBySubject: [mockEngines, mockEngines],
      consciousnessLevel: 2,
      language: 'en',
    });
    expect(result.passes).toHaveLength(1);
    expect(result.assembled).toContain('PASS OUTPUT');
    expect(result.register).toBe('l1_l3');
    expect(llm).toHaveBeenCalled();
  });

  it('loads integrated-reading mode from disk and runs full flow with factory+audit', async () => {
    const modePath = resolve(__dirname, '../../modes/integrated-reading.md');
    const mode = parseModeDoc(modePath);
    expect(mode.frontmatter.mode).toBe('integrated-reading');
    expect(mode.frontmatter.pass_plan.length).toBe(3);
    expect(mode.frontmatter.register_variants?.l1_l3).toBeDefined();

    const llm = vi.fn().mockResolvedValue('SYNTHESIZED PASS OUTPUT FOR INTEGRATED READING');
    const orchestrator = new IntegratedReadingOrchestrator({ mode, llm });
    const result = await orchestrator.run({
      subjectNames: ['Arathi'],
      engineResultsBySubject: [mockEngines],
      consciousnessLevel: 2,
    });

    expect(result.mode).toBe('integrated-reading');
    expect(result.passes).toHaveLength(3);
    expect(result.passes.map(p => p.id)).toEqual(['structural', 'somatic', 'synthesis']);
    expect(result.assembled).toContain('SYNTHESIZED PASS OUTPUT');
    expect(result.register).toBe('l1_l3');

    // Verify factory + audit accept the assembled output
    const dir = mkdtempSync(join(tmpdir(), 'witness-e2e-'));
    try {
      const pack = await createSourcePack({
        personId: 'e2e-arathi',
        readingMarkdown: result.assembled,
        engineResults: mockEngines,
        outputDir: dir,
      });
      const audit = runChainAudit({
        personId: 'e2e-arathi',
        readingMarkdown: result.assembled,
        engineResults: mockEngines,
      });
      expect(pack.manifest.quality.gate_status).toBe('ready');
      expect(audit.passed).toBe(true);
      expect(audit.facts_count).toBe(3);
    } finally {
      rmSync(dir, { recursive: true, force: true });
    }
  });

  it('loads birth-blueprint mode from disk and runs lighter flow', async () => {
    const modePath = resolve(__dirname, '../../modes/birth-blueprint.md');
    const mode = parseModeDoc(modePath);
    expect(mode.frontmatter.mode).toBe('birth-blueprint');
    expect(mode.frontmatter.pass_plan.length).toBe(2);

    const llm = vi.fn().mockResolvedValue('BIRTH BLUEPRINT OUTPUT');
    const orchestrator = new IntegratedReadingOrchestrator({ mode, llm });
    const result = await orchestrator.run({
      subjectNames: ['Rohan'],
      engineResultsBySubject: [mockEngines.slice(0, 2)],
      consciousnessLevel: 4,
    });

    expect(result.mode).toBe('birth-blueprint');
    expect(result.passes).toHaveLength(2);
    expect(result.register).toBe('l4_l5');
    expect(result.assembled).toContain('BIRTH BLUEPRINT OUTPUT');
  });

  it('records non-zero chart fidelity when output cites real engine facts', async () => {
    const mode = parseModeDocument(fidelityModeDoc, 'fidelity-e2e.md');
    const llm = vi.fn().mockResolvedValue('Lagna is Aries with gate 34 in dasha Rahu.');
    const orchestrator = new IntegratedReadingOrchestrator({ mode, llm });
    const engineWithFacts: SelemeneEngineOutput[] = [
      {
        engine_id: 'panchanga',
        result: { lagna: 'Aries' },
        witness_prompt: 'Observe.',
        consciousness_level: 2,
        metadata: { calculation_time_ms: 1, backend: 'native', precision_achieved: 'standard', cached: false, timestamp: '2026-06-22T00:00:00Z', engine_version: '1' },
        envelope_version: '1',
      },
      {
        engine_id: 'human-design',
        result: { gate: 34 },
        witness_prompt: 'Observe.',
        consciousness_level: 2,
        metadata: { calculation_time_ms: 1, backend: 'native', precision_achieved: 'standard', cached: false, timestamp: '2026-06-22T00:00:00Z', engine_version: '1' },
        envelope_version: '1',
      },
      {
        engine_id: 'vimshottari',
        result: { dasha: 'Rahu' },
        witness_prompt: 'Observe.',
        consciousness_level: 2,
        metadata: { calculation_time_ms: 1, backend: 'native', precision_achieved: 'standard', cached: false, timestamp: '2026-06-22T00:00:00Z', engine_version: '1' },
        envelope_version: '1',
      },
    ];
    const result = await orchestrator.run({
      subjectNames: ['TestSubject'],
      engineResultsBySubject: [engineWithFacts],
      consciousnessLevel: 2,
    });
    expect(result.passes.find(p => p.id === 'opening')?.rubric?.chart_fidelity_score).toBeGreaterThan(0);
  });

  it('drops patterns that contained private birth data even after scrub', async () => {
    const modePath = resolve(__dirname, '../../modes/integrated-reading.md');
    const mode = parseModeDoc(modePath);
    const llm = vi.fn().mockResolvedValue('Born 1960-06-01 at 12.97,77.59. Lagna Aries.');
    const orchestrator = new IntegratedReadingOrchestrator({ mode, llm });
    const result = await orchestrator.run({
      subjectNames: ['PrivateTest'],
      engineResultsBySubject: [mockEngines],
      consciousnessLevel: 2,
    });
    expect(result.patterns.filter(p => p.text.includes('1960') || p.text.includes('12.97')).length).toBe(0);
  });
});
