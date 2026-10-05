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
  it('retains all nine natal planet rows beyond the generic truncation boundary', () => {
    const names = ['Sun', 'Moon', 'Mars', 'Mercury', 'Jupiter', 'Venus', 'Saturn', 'Rahu', 'Ketu'];
    const planets = Object.fromEntries(names.map((name, i) => [name, {
      longitude: i * 30 + 1.25, sign: `Sign ${i}`, deg_str: '1°15', house: i + 1,
      nakshatra: `Nakshatra ${i}`, pada: 2, retrograde: false,
    }]));
    const output = summariseEngine({ ...hd(), engine_id: 'vedic-kundali', result: {
      ayanamsa: 'Lahiri', house_system: 'whole-sign', birth_time_confidence: 'approximate',
      lagna: { sign: 'Scorpio', deg_str: '12°06', nakshatra: 'Anuradha', pada: 3 }, planets,
    }});
    expect(output.length).toBeGreaterThan(400);
    expect(output).toContain('birth-time confidence approximate');
    for (const [i, name] of names.entries()) {
      expect(output).toContain(`${name}: longitude ${i * 30 + 1.25}`);
      expect(output).toContain(`house ${i + 1}; nakshatra Nakshatra ${i} pada 2`);
    }
  });
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
    expect(block).toContain('Personality activations (point gate.line) sun 23.4; earth 43.4');
    expect(block).toContain('Design activations (point gate.line) sun 49.2; earth 4.2');
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


describe('per-engine micro-interpretation additions', () => {
  const engine: SelemeneEngineOutput = {
    engine_id: 'vedic-kundali', witness_prompt: '', consciousness_level: 3, envelope_version: '1',
    metadata: { calculation_time_ms: 1, backend: 'swiss-ephemeris', precision_achieved: 'high', cached: false, timestamp: 't', engine_version: '1' },
    result: {
      ayanamsa: 'Lahiri',
      lagna: { sign: 'Scorpio', deg_str: '12°06', nakshatra: 'Anuradha' },
      planets: { Jupiter: { longitude: 15.25, sign: 'Cancer', dignity: 'Exalted', house: 9 } },
    },
  } as any;

  it('flattens engine fields with dotted paths and preserves values', async () => {
    const { buildEngineFieldsConsumed } = await import('./engine-facts.js');
    const fields = buildEngineFieldsConsumed(engine);
    expect(fields.engine_id).toBe('vedic-kundali');
    expect(fields.fields_used).toContain('ayanamsa');
    expect(fields.fields_used).toContain('lagna.sign');
    expect(fields.fields_used).toContain('planets.Jupiter.dignity');
    expect(fields.values_snapshot['planets.Jupiter.dignity']).toBe('Exalted');
    expect(fields.source_precision_label).toBe('high');
    expect(fields.backend).toBe('swiss-ephemeris');
  });

  it('respects a fieldsFilter predicate', async () => {
    const { buildEngineFieldsConsumed } = await import('./engine-facts.js');
    const fields = buildEngineFieldsConsumed(engine, p => p.startsWith('planets.Jupiter'));
    expect(fields.fields_used.every(f => f.startsWith('planets.Jupiter'))).toBe(true);
    expect(fields.fields_used).toContain('planets.Jupiter.dignity');
  });

  it('buildPerEngineMicroInterpretationPrompt returns prompt plus fields manifest', async () => {
    const { buildPerEngineMicroInterpretationPrompt } = await import('./engine-facts.js');
    const { prompt, fields } = buildPerEngineMicroInterpretationPrompt({
      engine, register: 'L4-L5', subject: 'Sheshnarayan', sectionTopic: 'Convergence Map',
    });
    expect(prompt).toContain('Subject: Sheshnarayan');
    expect(prompt).toContain('Engine: vedic-kundali');
    expect(prompt).toContain('Register: L4-L5');
    expect(prompt).toContain('Section topic: Convergence Map');
    expect(prompt).toContain('planets.Jupiter.dignity');
    expect(prompt).toContain('1–8 section-relevant micro-interpretations');
    expect(prompt).toContain('"source_field"');
    expect(prompt).toContain('"source_value"');
    expect(prompt).toContain('Never omit source_field, source_value, or interpretation_tradition');
    expect(fields.fields_used).toContain('planets.Jupiter.dignity');
  });

  it('is deterministic: same engine input → identical prompt and manifest', async () => {
    const { buildPerEngineMicroInterpretationPrompt } = await import('./engine-facts.js');
    const a = buildPerEngineMicroInterpretationPrompt({ engine, register: 'L1-L3', subject: 'S' });
    const b = buildPerEngineMicroInterpretationPrompt({ engine, register: 'L1-L3', subject: 'S' });
    expect(a).toEqual(b);
  });

  it('emits an explicit "no non-empty fields" line when the engine has no leaf values', async () => {
    const { buildPerEngineMicroInterpretationPrompt } = await import('./engine-facts.js');
    const empty: SelemeneEngineOutput = { ...engine, result: {} } as any;
    const { prompt, fields } = buildPerEngineMicroInterpretationPrompt({ engine: empty, register: 'L1-L3', subject: 'S' });
    expect(fields.fields_used).toEqual([]);
    expect(prompt).toContain('no non-empty fields consumed');
  });
});
