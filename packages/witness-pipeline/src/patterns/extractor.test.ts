import { describe, expect, it } from 'vitest';
import { extractReportPatterns } from './extractor.js';

describe('extractReportPatterns', () => {
  it('extracts anonymized reusable patterns from passing sections', () => {
    const patterns = extractReportPatterns({
      mode: 'integrated-reading',
      reportLevel: 'L3',
      subjectNames: ['Private Name'],
      passes: [
        {
          id: 'synthesis',
          title: 'Synthesis',
          output: 'Private Name has Vedic Saturn pressure and Human Design Projector pacing, forming a pattern of delayed recognition.',
          rubric: {
            section_id: 'synthesis', title: 'Synthesis', target_words: 20, actual_words: 18,
            word_count_fit: 'pass', word_count_ratio: 0.9,
            deterministic_fact_count: 4, deterministic_fact_gate: 'pass',
            integrated_layer_count: 2, integrated_layering_gate: 'pass',
            guardrail_gate: 'pass', guardrail_violations: [],
            model_requested: 'tier-default', model_used: 'tier-default', latency_ms: 1,
          },
        },
      ],
    });

    expect(patterns).toHaveLength(1);
    expect(patterns[0].text).not.toContain('Private Name');
    expect(patterns[0].metadata.mode).toBe('integrated-reading');
    expect(patterns[0].metadata.report_level).toBe('L3');
  });
});
