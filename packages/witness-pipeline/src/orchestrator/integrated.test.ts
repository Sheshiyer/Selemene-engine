import { describe, it, expect, vi } from 'vitest';
import { IntegratedReadingOrchestrator } from './integrated.js';
import type { ParsedModeDoc, SelemeneEngineOutput } from '../index.js';
import type { PatternVectorRetriever, RetrievedPattern } from '../patterns/retrieval.js';

function makeEngineWithLagnaAndGate(lagna: string, gate: number): SelemeneEngineOutput {
  return {
    engine_id: 'panchanga',
    result: { lagna_sign: lagna, gates: [gate] },
    witness_prompt: '',
    consciousness_level: 5,
    metadata: {
      calculation_time_ms: 0,
      backend: 'native',
      precision_achieved: 'standard',
      cached: false,
      timestamp: '',
      engine_version: '1',
    },
    envelope_version: '1',
  };
}

const mockKundaliMode: ParsedModeDoc = {
  frontmatter: {
    mode: 'integrated-kundali',
    subject_count: { min: 1, max: 1 },
    roles: ['subject'],
    target_words: { min: 50, max: 200 },
    architecture: 'linear',
    pass_plan: [
      { id: 'opening', title: 'Opening', target_words: 80, template: 'pass-opening-template' },
    ],
    engine_overlay_weights: {},
    house_overlay: [],
    bridge_mandates: [],
    svg_topology: 'web-graph',
  },
  sections: {
    'pass-opening-template': 'Write about {{subject_names}}.',
  },
  lessons: [],
  raw_path: 'mock-kundali.md',
};

const mockMode: ParsedModeDoc = {
  frontmatter: {
    mode: 'composite-dyad',
    subject_count: { min: 2, max: 2 },
    roles: ['subject-a', 'subject-b'],
    target_words: { min: 100, max: 500 },
    architecture: 'linear',
    pass_plan: [
      { id: 'alpha', title: 'Alpha', target_words: 100, template: 'pass-alpha-template' },
    ],
    engine_overlay_weights: { panchanga: 1 },
    house_overlay: [1],
    bridge_mandates: [],
    svg_topology: 'dyad-arc',
  },
  sections: {
    'pass-alpha-template': 'Write about {{subject_names}} using {{overlay_summary}}.',
  },
  lessons: [],
  raw_path: 'composite-dyad.md',
};

const mockEngineResults: SelemeneEngineOutput[] = [
  {
    engine_id: 'panchanga',
    result: {},
    witness_prompt: 'Observe the moment.',
    consciousness_level: 2,
    metadata: {
      calculation_time_ms: 10,
      backend: 'native',
      precision_achieved: 'standard',
      cached: false,
      timestamp: '2026-06-22T00:00:00Z',
      engine_version: '1',
    },
    envelope_version: '1',
  },
];

describe('IntegratedReadingOrchestrator', () => {
  it('renders the first pass prompt with interpolations', async () => {
    const llm = vi.fn().mockResolvedValue('alpha output');
    const orchestrator = new IntegratedReadingOrchestrator({ mode: mockMode, llm });
    const result = await orchestrator.run({
      subjectNames: ['Arathi', 'Rohan'],
      engineResultsBySubject: [mockEngineResults, mockEngineResults],
      consciousnessLevel: 2,
    });
    expect(result.passes).toHaveLength(1);
    expect(result.passes[0].output).toBe('alpha output');
    expect(result.register).toBe('l1_l3');
    expect(llm).toHaveBeenCalledWith(
      expect.stringContaining('Arathi'),
      expect.any(String),
      expect.objectContaining({ max_tokens: expect.any(Number) }),
    );
    const userPrompt = llm.mock.calls[0][1] as string;
    expect(userPrompt).toContain('Arathi');
    expect(userPrompt).toContain('Engine weights: panchanga: 1');
  });

  it('selects l4_l5 register for high consciousness level', async () => {
    const llm = vi.fn().mockResolvedValue('out');
    const orchestrator = new IntegratedReadingOrchestrator({ mode: mockMode, llm });
    const result = await orchestrator.run({
      subjectNames: ['A'],
      engineResultsBySubject: [mockEngineResults],
      consciousnessLevel: 5,
    });
    expect(result.register).toBe('l4_l5');
  });

  it('populates chart_fidelity_score when engines are passed', async () => {
    const llm = vi.fn().mockResolvedValue('Lagna is Aries with gate 34 and Vimshottari dasha active.');
    const orchestrator = new IntegratedReadingOrchestrator({ mode: mockKundaliMode, llm });
    const result = await orchestrator.run({
      subjectNames: ['A'],
      engineResultsBySubject: [[makeEngineWithLagnaAndGate('aries', 34)]],
      consciousnessLevel: 5,
    });
    expect(result.passes[0].rubric.chart_fidelity_score).toBeGreaterThan(0);
  });

  it('attaches retrieved patterns to prompt and output when retriever provided', async () => {
    const fakeRetriever: PatternVectorRetriever = {
      async retrieveSimilar() {
        const r: RetrievedPattern[] = [{ text: 'Use analogy of the projector pacing authority', score: 0.9 }];
        return r;
      },
    };
    const llm = vi.fn().mockResolvedValue('output using analogy');
    const orchestrator = new IntegratedReadingOrchestrator({ mode: mockMode, llm });
    const result = await orchestrator.run({
      subjectNames: ['A', 'B'],
      engineResultsBySubject: [mockEngineResults],
      consciousnessLevel: 2,
      retriever: fakeRetriever,
      retrievalQuery: 'projector pacing authority',
    });
    expect(result.retrieved_patterns?.length).toBeGreaterThan(0);
    const prompt = llm.mock.calls[0][1] as string;
    expect(prompt).toContain('Retrieved synthesis patterns are not deterministic facts');
    expect(prompt).toContain('projector pacing authority');
  });
});
