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
});
