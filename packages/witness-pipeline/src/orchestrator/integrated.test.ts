import { describe, it, expect, vi } from 'vitest';
import { IntegratedReadingOrchestrator } from './integrated.js';
import { parseModeDocument } from '../modes/parser.js';
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

  const sampleEngineResults: SelemeneEngineOutput[] = [
    {
      engine_id: 'panchanga',
      result: { tithi_name: 'Navami (Krishna)', nakshatra_name: 'Pushya' },
      witness_prompt: '',
      consciousness_level: 1,
      metadata: {
        calculation_time_ms: 0,
        backend: 'test',
        precision_achieved: 'test',
        cached: false,
        timestamp: new Date().toISOString(),
        engine_version: 'test',
      },
      envelope_version: '1.0',
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
      witness_prompt: '',
      consciousness_level: 1,
      metadata: {
        calculation_time_ms: 0,
        backend: 'test',
        precision_achieved: 'test',
        cached: false,
        timestamp: new Date().toISOString(),
        engine_version: 'test',
      },
      envelope_version: '1.0',
    },
  ];

  const engineResultsModeDoc = `---
mode: integrated-kundali-l0
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
    target_words: 80
    template: pass-opening-template
engine_overlay_weights: {}
house_overlay: []
bridge_mandates: []
svg_topology: dyad-arc
---

## pass-opening-template
Engine results:
{{engine_results}}
`;

  it('includes engine results in the rendered prompt', async () => {
    const prompts: string[] = [];
    const llm = async (_system: string, prompt: string) => {
      prompts.push(prompt);
      return 'ok';
    };
    const parsedModeDoc = parseModeDocument(engineResultsModeDoc, 'task2-test.md');
    const orchestrator = new IntegratedReadingOrchestrator({ mode: parsedModeDoc, llm });
    await orchestrator.run({
      subjectNames: ['Harshita'],
      engineResultsBySubject: [sampleEngineResults],
      consciousnessLevel: 2,
    });
    expect(prompts.length).toBeGreaterThan(0);
    expect(prompts[0]).toContain('## Deterministic Engine Results');
    expect(prompts[0]).toContain('Pushya');
  });

  it('produces a long pass at >= 70% of target word count', async () => {
    const longPassModeDoc = parseModeDocument(`---
mode: integrated-kundali-l0
subject_count:
  min: 1
  max: 1
roles:
  - subject
target_words:
  min: 50
  max: 500
architecture: linear
pass_plan:
  - id: long-pass
    title: Long Pass
    target_words: 600
    template: long-pass-template
engine_overlay_weights: {}
house_overlay: []
bridge_mandates: []
svg_topology: dyad-arc
---
## long-pass-template
Write about {{subject_names}}. Target ~{{target_words}} words. Pass: {{pass_id}}.
`, 'long-pass-test.md');

    const systemTerms = 'Vedic Lagna house planet nakshatra pada dasha antardasha Vimshottari Human Design gate channel profile authority type center Gene Keys Life\'s Work Evolution Vocation Pearl Radiance Purpose transit panchanga tithi yoga karana. ';
    const guard = ' Layered integration across systems. Guardrail safe framing. No guarantees no diagnosis. ';
    const llm = async (_system: string, _user: string, opts: { max_tokens: number }) => {
      const targetWords = Math.max(200, Math.floor((opts.max_tokens ?? 600) / 1.3));
      let words = 0;
      let body = '';
      while (words < targetWords) {
        const chunk = systemTerms.repeat(20) + guard;
        body += chunk;
        words = body.split(/\s+/).filter(Boolean).length;
      }
      const allWords = body.split(/\s+/).filter(Boolean);
      return allWords.slice(0, targetWords).join(' ');
    };

    const orchestrator = new IntegratedReadingOrchestrator({ mode: longPassModeDoc, llm });
    const result = await orchestrator.run({
      subjectNames: ['Prashanth'],
      engineResultsBySubject: [sampleEngineResults],
      consciousnessLevel: 5,
    });

    expect(result.passes).toHaveLength(1);
    const actualWords = result.passes[0].rubric.actual_words;
    const targetWords = result.passes[0].rubric.target_words;
    expect(actualWords).toBeGreaterThanOrEqual(targetWords * 0.7);
  });
});
