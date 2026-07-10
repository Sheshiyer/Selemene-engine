// packages/witness-pipeline/src/notebooklm/slides-prompt.test.ts
import { describe, it, expect } from 'vitest';
import { generateSlidesPrompt } from './slides-prompt.js';
import type { OrchestratorOutput } from '../orchestrator/integrated.js';

describe('generateSlidesPrompt', () => {
  it('exports generateSlidesPrompt as a pure function', () => {
    const out: OrchestratorOutput = {
      mode: 'test-mode',
      subject_names: ['Test'],
      register: 'l1_l3',
      passes: [],
      assembled: 'test',
      patterns: [],
    };
    const prompt = generateSlidesPrompt(out);
    expect(typeof prompt).toBe('string');
    expect(prompt.length).toBeGreaterThan(0);
  });

  it('is re-exported from package root', async () => {
    const mod = await import('@noesis/witness-pipeline');
    expect(typeof mod.generateSlidesPrompt).toBe('function');
  });

  it('injects relationship_header and language into the prompt when present', () => {
    const out: OrchestratorOutput = {
      mode: 'mother-son-lineage',
      subject_names: ['Aarav', 'Vikram'],
      register: 'l1_l3',
      relationship_header: 'Mother-Son Lineage Mapping — non-predictive pattern witness',
      passes: [
        { id: 'opening', title: 'Opening', output: 'Fact one.', rubric: { guardrail_gate: 'pass' } as any },
      ],
      assembled: 'Mother-Son...',
      patterns: [],
    };
    const prompt = generateSlidesPrompt(out, { language: 'hi' });
    expect(prompt).toContain('Language: hi');
    expect(prompt).toContain('Mother-Son Lineage Mapping — non-predictive pattern witness');
    expect(prompt).toContain('Facts only. No prediction');
  });
});
