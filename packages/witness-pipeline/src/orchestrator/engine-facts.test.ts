import { describe, it, expect } from 'vitest';
import { buildEngineFactsBlock, summariseEngine } from './engine-facts.js';
import type { SelemeneEngineOutput } from '../selemene/types.js';

const meta = { calculation_time_ms: 1, backend: 'native', precision_achieved: 'standard', cached: false, timestamp: 't', engine_version: '1' };
const hd = (): SelemeneEngineOutput => ({
  engine_id: 'human-design', witness_prompt: '', consciousness_level: 2, envelope_version: '1', metadata: { ...meta, backend: 'swiss-ephemeris' },
  result: {
    hd_type: 'Generator', profile: '2/4', authority: 'Sacral', definition: 'Split',
    defined_centers: ['Sacral', 'Throat'], active_channels: ['43-23'],
    personality_activations: { sun: { gate: 23, line: 4 }, earth: { gate: 43, line: 4 } },
    design_activations: { sun: { gate: 49, line: 2 }, earth: { gate: 4, line: 2 } },
  },
});
const vim = (): SelemeneEngineOutput => ({
  engine_id: 'vimshottari', witness_prompt: '', consciousness_level: 2, envelope_version: '1', metadata: meta,
  result: { birth_nakshatra: { name: 'Uttara Phalguni', number: 12 }, current_period: { mahadasha: { planet: 'Jupiter', start: '2026-09-09T21:39:58+00:00', end: '2042-09-09T21:39:58+00:00' } } },
});

describe('engine facts block', () => {
  it('renders one section per subject with role labels and chart facts', () => {
    const block = buildEngineFactsBlock({
      subjectNames: ['A', 'B'],
      subjectRoles: [{ role: 'business-partner', name: 'A', label: 'CEO' }, { role: 'business-partner', name: 'B' }],
      engineResultsBySubject: [[hd()], [vim()]],
    });
    expect(block).toContain('### A (business-partner, CEO)');
    expect(block).toContain('### B (business-partner)');
    expect(block).toContain('Type Generator');
    expect(block).toContain('Profile 2/4');
    expect(block).toContain('Incarnation cross gates (P-Sun/P-Earth/D-Sun/D-Earth) 23.4 / 43.4 / 49.2 / 4.2');
    expect(block).toContain('mahadasha Jupiter 2026-09-09→2042-09-09');
    expect(block).toContain('[swiss-ephemeris]');
  });

  it('is deterministic and marks errors and moment-bound engines', () => {
    const errored: SelemeneEngineOutput = { engine_id: 'panchanga', _error: 'HTTP 500', result: null, witness_prompt: '', consciousness_level: 2, envelope_version: '1', metadata: meta };
    const moment: SelemeneEngineOutput = { engine_id: 'biorhythm', result: { physical: 0.1 }, witness_prompt: '', consciousness_level: 2, envelope_version: '1', metadata: meta };
    expect(summariseEngine(errored)).toBe('- panchanga: unavailable (HTTP 500)');
    expect(summariseEngine(moment)).toContain('moment-bound');
    const a = buildEngineFactsBlock({ subjectNames: ['X'], engineResultsBySubject: [[hd(), vim()]] });
    const b = buildEngineFactsBlock({ subjectNames: ['X'], engineResultsBySubject: [[hd(), vim()]] });
    expect(a).toBe(b);
  });

  it('notes missing engine results for a subject instead of dropping the subject', () => {
    const block = buildEngineFactsBlock({ subjectNames: ['A', 'B'], engineResultsBySubject: [[hd()]] });
    expect(block).toContain('### B\n- no engine results supplied');
  });
});
