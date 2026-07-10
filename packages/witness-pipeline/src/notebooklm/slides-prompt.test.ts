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

  it('produces a clean solo prompt when no relationship_context is present', () => {
    const out: OrchestratorOutput = {
      mode: 'birth-blueprint',
      subject_names: ['Subject'],
      register: 'l4_l5',
      passes: [],
      assembled: 'Solo content',
      patterns: [],
    };
    const prompt = generateSlidesPrompt(out, { language: 'en' });
    expect(prompt).toContain('Solo Reading — non-predictive pattern witness');
    expect(prompt).not.toContain('Mother-Son');
  });

  it('produces usable NotebookLM prompt from a mother-son matrix-style output', () => {
    const motherSonOut: OrchestratorOutput = {
      mode: 'mother-son-lineage',
      subject_names: ['Aarav', 'Vikram'],
      register: 'l1_l3',
      relationship_header: 'Mother-Son Lineage Mapping — non-predictive pattern witness',
      passes: [
        { id: 'opening', title: 'Opening', output: 'Observable fact A.', rubric: { guardrail_gate: 'pass' } as any },
        { id: 'lineage', title: 'Lineage Field', output: 'Observable fact B.', rubric: { guardrail_gate: 'pass' } as any },
      ],
      assembled: 'Mother-Son Lineage Mapping...\n\n## Opening\n\nObservable fact A.\n\n## Lineage Field\n\nObservable fact B.',
      patterns: [],
    };

    const prompt = generateSlidesPrompt(motherSonOut, { language: 'hi' });

    expect(prompt).toContain('Language: hi');
    expect(prompt).toContain('Mother-Son Lineage Mapping — non-predictive pattern witness');
    expect(prompt).toContain('Facts only. No prediction');
    expect(prompt).toContain('Pass count: 2');
    // The prompt should be safe to paste directly into NotebookLM
    expect(prompt.length).toBeGreaterThan(200);
  });

  it('includes a bridge mandate when supplied via a minimal mode-like surface (future: pass mode doc)', () => {
    // For v1 we simulate by allowing an optional mandates array on the call
    const out: OrchestratorOutput = {
      mode: 'business-partners',
      subject_names: ['Priya', 'Rahul'],
      register: 'l1_l3',
      relationship_header: 'Business-Partners Synergy Audit — non-predictive pattern witness',
      passes: [],
      assembled: '',
      patterns: [],
    };
    const prompt = generateSlidesPrompt(out, {
      language: 'en',
      bridgeMandates: ['No investment or outcome guarantees'],
    });
    // In the first slice we accept an extension point; the test documents the intent
    expect(prompt).toContain('No investment or outcome guarantees');
  });

  /*
  Example usage (copy into a script or NotebookLM workflow):

  import { generateSlidesPrompt } from '@noesis/witness-pipeline';
  const prompt = generateSlidesPrompt(orchestratorResult, { language: 'hi' });
  // Paste `prompt` into NotebookLM "Create slides from text" or "Audio overview" source.
  */
});
