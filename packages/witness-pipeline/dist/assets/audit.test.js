import { describe, it, expect } from 'vitest';
import { runChainAudit } from './audit.js';
function makeEngine(id, deterministic = true) {
    return {
        engine_id: id,
        result: {},
        witness_prompt: 'x',
        consciousness_level: 2,
        metadata: {
            calculation_time_ms: 1,
            backend: deterministic ? 'native' : 'ts',
            precision_achieved: 'standard',
            cached: false,
            timestamp: '2026-06-22T00:00:00Z',
            engine_version: '1',
        },
        envelope_version: '1',
    };
}
describe('runChainAudit', () => {
    it('passes a pack with deterministic engine data', () => {
        const result = runChainAudit({
            personId: 'test',
            readingMarkdown: 'Lagna in Aries.',
            engineResults: [
                makeEngine('panchanga'),
                makeEngine('vimshottari'),
                makeEngine('human-design'),
            ],
        });
        expect(result.blockers).toHaveLength(0);
        expect(result.passed).toBe(true);
    });
    it('blocks a pack with oracle engines when deterministic-only is required', () => {
        const result = runChainAudit({
            personId: 'test',
            readingMarkdown: 'Lagna in Aries.',
            engineResults: [
                makeEngine('panchanga'),
                makeEngine('tarot', false),
            ],
            deterministicOnly: true,
        });
        expect(result.blockers.length).toBeGreaterThan(0);
        expect(result.passed).toBe(false);
    });
    it('warns for short readings', () => {
        const result = runChainAudit({
            personId: 'test',
            readingMarkdown: 'Hi.',
            engineResults: [
                makeEngine('panchanga'),
                makeEngine('vimshottari'),
                makeEngine('human-design'),
            ],
        });
        expect(result.warnings.length).toBeGreaterThan(0);
        expect(result.passed).toBe(true);
    });
});
//# sourceMappingURL=audit.test.js.map