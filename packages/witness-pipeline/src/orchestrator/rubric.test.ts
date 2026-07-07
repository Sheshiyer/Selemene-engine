import { describe, expect, it } from 'vitest';
import { auditSectionOutput } from './rubric.js';

describe('auditSectionOutput', () => {
  it('passes word count when output is within 80-125 percent of target', () => {
    const output = Array.from({ length: 90 }, (_, i) => `word${i}`).join(' ');
    const rubric = auditSectionOutput({
      sectionId: 'opening',
      title: 'Opening',
      targetWords: 100,
      output,
      modelRequested: 'tier-default',
      modelUsed: 'tier-default',
      latencyMs: 10,
    });

    expect(rubric.word_count_fit).toBe('pass');
    expect(rubric.actual_words).toBe(90);
  });

  it('counts deterministic fact references and integrated layers', () => {
    const output = 'Vedic Lagna, Vimshottari dasha, Human Design profile, Gene Keys Pearl, and transits converge.';
    const rubric = auditSectionOutput({
      sectionId: 'convergence-map',
      title: 'Part I',
      targetWords: 20,
      output,
      modelRequested: 'tier-default',
      modelUsed: 'tier-default',
      latencyMs: 10,
    });

    expect(rubric.deterministic_fact_count).toBeGreaterThanOrEqual(5);
    expect(rubric.integrated_layer_count).toBeGreaterThanOrEqual(4);
  });

  it('fails health guardrail on diagnostic language', () => {
    const rubric = auditSectionOutput({
      sectionId: 'health',
      title: 'Health',
      targetWords: 20,
      output: 'This is a diagnosis and treatment plan for disease.',
      modelRequested: 'tier-default',
      modelUsed: 'tier-default',
      latencyMs: 10,
    });

    expect(rubric.guardrail_gate).toBe('fail');
    expect(rubric.guardrail_violations.length).toBeGreaterThan(0);
  });

  it('uses stricter deterministic thresholds for kundali timeline sections', () => {
    const rubric = auditSectionOutput({
      sectionId: 'master-timeline',
      title: 'Part IX',
      targetWords: 100,
      output: 'Vimshottari dasha and Sade Sati are mentioned once.',
      modelRequested: 'tier-default',
      modelUsed: 'tier-default',
      latencyMs: 1,
    });

    expect(rubric.deterministic_fact_gate).toBe('fail');
  });

  // Minimal helper factories for fidelity test (as specified in plan)
  function makeEngineWithLagna(lagna: string): any {
    return { result: { lagna_sign: lagna } };
  }
  function makeEngineWithGate(gate: number): any {
    return { result: { gates: [gate] } };
  }

  it('computes chart_fidelity_score when engine results are supplied', () => {
    const engines = [makeEngineWithLagna('aries'), makeEngineWithGate(34)];
    const rubric = auditSectionOutput({
      sectionId: 'opening',
      title: 'Opening',
      targetWords: 100,
      output: 'Lagna is Aries and Human Design gate 34 is active.',
      modelRequested: 'tier-default',
      modelUsed: 'tier-default',
      latencyMs: 10,
      engineResults: engines,
    });

    expect(rubric.chart_fidelity_score).toBeGreaterThanOrEqual(0.5);
    expect(rubric.chart_fidelity_score).toBeLessThanOrEqual(1.0);
  });

  const sampleEngineResults = [
    {
      engine_id: 'panchanga',
      result: { tithi_name: 'Navami (Krishna)', nakshatra_name: 'Pushya' },
    },
    {
      engine_id: 'vimshottari',
      result: {
        current_period: {
          mahadasha: { planet: 'Ketu' },
          antardasha: { planet: 'Mercury' },
          pratyantardasha: { planet: 'Moon' },
        },
      },
    },
  ];

  it('flags unsubstituted placeholders', () => {
    const rubric = auditSectionOutput({
      sectionId: 'opening',
      title: 'Opening',
      targetWords: 450,
      output: 'Your Lagna is [exact sign from engine results].',
      modelRequested: 'test',
      modelUsed: 'test',
      latencyMs: 0,
      engineResults: sampleEngineResults,
    });
    expect(rubric.placeholder_gate).toBe('fail');
  });
});
