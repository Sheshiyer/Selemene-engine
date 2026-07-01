import { describe, it, expect, vi } from 'vitest';
import { IntegratedReadingOrchestrator } from './integrated.js';
import { parseModeDocument } from '../modes/parser.js';
const sampleModeDoc = `---
mode: composite-dyad
subject_count:
  min: 2
  max: 2
roles:
  - subject-a
  - subject-b
target_words:
  min: 200
  max: 300
architecture: linear
pass_plan:
  - id: alpha
    title: Structural Field
    target_words: 150
    template: pass-alpha-template
engine_overlay_weights:
  panchanga: 1.0
house_overlay: [1, 4, 7, 10]
bridge_mandates:
  - "Braid Vedic and HD data"
svg_topology: dyad-arc
---

## pass-alpha-template
Write about {{subject_names}} using {{overlay_summary}} and {{bridge_mandates}}.
`;
const mockEngines = [
    {
        engine_id: 'panchanga',
        result: { tithi_name: 'Test' },
        witness_prompt: 'Observe.',
        consciousness_level: 2,
        metadata: { calculation_time_ms: 1, backend: 'native', precision_achieved: 'standard', cached: false, timestamp: '2026-06-22T00:00:00Z', engine_version: '1' },
        envelope_version: '1',
    },
];
describe('IntegratedReadingOrchestrator end-to-end', () => {
    it('executes full multi-pass flow with real mode doc', async () => {
        const mode = parseModeDocument(sampleModeDoc, 'e2e-test.md');
        const llm = vi.fn().mockResolvedValue('PASS OUTPUT');
        const orchestrator = new IntegratedReadingOrchestrator({ mode, llm });
        const result = await orchestrator.run({
            subjectNames: ['Arathi', 'Rohan'],
            engineResultsBySubject: [mockEngines, mockEngines],
            consciousnessLevel: 2,
        });
        expect(result.passes).toHaveLength(1);
        expect(result.assembled).toContain('PASS OUTPUT');
        expect(result.register).toBe('l1_l3');
        expect(llm).toHaveBeenCalled();
    });
});
//# sourceMappingURL=integrated.e2e.test.js.map