import { describe, it, expect, vi } from 'vitest';
import { IntegratedReadingOrchestrator } from './integrated.js';
const mockMode = {
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
const mockEngineResults = [
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
        expect(llm).toHaveBeenCalledWith(expect.stringContaining('Arathi'), expect.any(String), expect.objectContaining({ max_tokens: expect.any(Number) }));
        const userPrompt = llm.mock.calls[0][1];
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
});
//# sourceMappingURL=integrated.test.js.map