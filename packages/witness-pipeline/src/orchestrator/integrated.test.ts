import { describe, it, expect, vi } from 'vitest';
import { resolve } from 'node:path';
import { IntegratedReadingOrchestrator } from './integrated.js';
import { parseModeDoc } from '../modes/parser.js';
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
    'pass-opening-template': 'Write about {{subject_names}}.\n\nEngine results:\n{{engine_results}}',
  },
  lessons: [],
  raw_path: 'mock-kundali.md',
};

const mockTriadMode: ParsedModeDoc = {
  frontmatter: {
    mode: 'triad-triangle-test',
    subject_count: { min: 3, max: 3 },
    roles: ['partner', 'partner', 'partner'],
    target_words: { min: 300, max: 500 },
    architecture: 'linear',
    pass_plan: [
      { id: 'opening', title: 'Opening', target_words: 100, template: 'opening-template' },
      { id: 'field', title: 'Field', target_words: 150, template: 'field-template' },
    ],
    engine_overlay_weights: { 'human-design': 1 },
    house_overlay: [1, 7],
    bridge_mandates: ['Use triad language'],
    svg_topology: 'triad-triangle',
    relationship_types: ['unmarried-partners'],
    report_level: 'L2',
  },
  sections: {
    'opening-template': '# {{relationship_header}}\nTriad field for {{subject_names}}.',
    'field-template': 'Witness the triangle between the three partners.',
  },
  lessons: [],
  raw_path: 'triad-triangle-test.md',
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
    'pass-alpha-template': 'Write about {{subject_names}} using {{overlay_summary}} in {{language}}.',
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

// Synthetic chart values only; not derived from any real birth data.
function makeEngine(engine_id: SelemeneEngineOutput['engine_id'], result: Record<string, unknown>): SelemeneEngineOutput {
  return { ...mockEngineResults[0], engine_id, result };
}

const sampleEngineResults: SelemeneEngineOutput[] = [
  makeEngine('panchanga', { nakshatra_name: 'Pushya', tithi_name: 'Shukla Panchami' }),
  makeEngine('vimshottari', { current_period: { mahadasha: { planet: 'Saturn' } } }),
  makeEngine('human-design', { hd_type: 'Projector', profile: '5/1' }),
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

  it('injects language into prompts when supplied', async () => {
    const llm = vi.fn().mockResolvedValue('ok');
    const orchestrator = new IntegratedReadingOrchestrator({ mode: mockMode, llm });
    await orchestrator.run({
      subjectNames: ['A'],
      engineResultsBySubject: [mockEngineResults],
      consciousnessLevel: 2,
      language: 'hi',
    });
    const user = llm.mock.calls[0][1] as string;
    expect(user).toContain('hi'); // via {{language}} or system line
  });

  it('injects deterministic engine results into the rendered prompt', async () => {
    const llm = vi.fn().mockResolvedValue('ok');
    const orchestrator = new IntegratedReadingOrchestrator({ mode: mockKundaliMode, llm });
    await orchestrator.run({
      subjectNames: ['A'],
      engineResultsBySubject: [sampleEngineResults],
      consciousnessLevel: 2,
    });
    const prompt = llm.mock.calls[0][1] as string;
    expect(prompt).toContain('## Deterministic Engine Results');
    expect(prompt).toContain('nakshatra Pushya');
    expect(prompt).toContain('dasha Saturn');
    expect(prompt).toContain('HD type Projector');
    expect(prompt).not.toContain('{{engine_results}}');
  });

  it.each([2, 5])(
    'every integrated-kundali-l0 pass prompt carries engine data (consciousness level %i)',
    async (consciousnessLevel) => {
      const mode = parseModeDoc(resolve(__dirname, '../../modes/integrated-kundali-l0.md'));
      const llm = vi.fn().mockResolvedValue('ok');
      const orchestrator = new IntegratedReadingOrchestrator({ mode, llm });
      await orchestrator.run({
        subjectNames: ['A'],
        engineResultsBySubject: [sampleEngineResults],
        consciousnessLevel,
      });
      expect(llm).toHaveBeenCalledTimes(mode.frontmatter.pass_plan.length);
      for (const [, prompt] of llm.mock.calls as [string, string][]) {
        expect(prompt).toContain('## Deterministic Engine Results');
        expect(prompt).toContain('nakshatra Pushya');
        expect(prompt).not.toMatch(/\{\{\w+\}\}/);
      }
    },
  );
});
