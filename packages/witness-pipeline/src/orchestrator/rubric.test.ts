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
});
