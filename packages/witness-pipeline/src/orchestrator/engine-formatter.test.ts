import { describe, it, expect } from 'vitest';
import { formatEngineResultsForPrompt } from './engine-formatter.js';
import type { SelemeneEngineOutput } from '../index.js';

const sampleEngines: SelemeneEngineOutput[] = [
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

describe('formatEngineResultsForPrompt', () => {
  it('includes deterministic facts from each engine', () => {
    const out = formatEngineResultsForPrompt(sampleEngines);
    expect(out).toContain('panchanga');
    expect(out).toContain('Navami (Krishna)');
    expect(out).toContain('Pushya');
    expect(out).toContain('vimshottari');
    expect(out).toContain('Ketu');
    expect(out).toContain('Mercury');
    expect(out).toContain('Moon');
  });
});
