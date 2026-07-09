import { describe, expect, it } from 'vitest';
import { auditSectionOutput, extractSectionFacts } from './rubric.js';

describe('auditSectionOutput', () => {
  it('passes word count when output is within 80-125 percent of target', () => {
    const output = Array.from({ length: 90 }, (_, i) => `word${i}`).join(' ');
    const rubric = auditSectionOutput({
      sectionId: 'opening',
      title: 'Opening',
      targetWords: 100,
      output,
      modelRequested: 'tier-default',
      modelUsed: 'tier-default',
      latencyMs: 10,
    });

    expect(rubric.word_count_fit).toBe('pass');
    expect(rubric.actual_words).toBe(90);
  });

  it('counts deterministic fact references and integrated layers', () => {
    const output = 'Vedic Lagna, Vimshottari dasha, Human Design profile, Gene Keys Pearl, and transits converge.';
    const rubric = auditSectionOutput({
      sectionId: 'convergence-map',
      title: 'Part I',
      targetWords: 20,
      output,
      modelRequested: 'tier-default',
      modelUsed: 'tier-default',
      latencyMs: 10,
    });

    expect(rubric.deterministic_fact_count).toBeGreaterThanOrEqual(5);
    expect(rubric.integrated_layer_count).toBeGreaterThanOrEqual(4);
  });

  it('fails health guardrail on diagnostic language', () => {
    const rubric = auditSectionOutput({
      sectionId: 'health',
      title: 'Health',
      targetWords: 20,
      output: 'This is a diagnosis and treatment plan for disease.',
      modelRequested: 'tier-default',
      modelUsed: 'tier-default',
      latencyMs: 10,
    });

    expect(rubric.guardrail_gate).toBe('fail');
    expect(rubric.guardrail_violations.length).toBeGreaterThan(0);
  });

  it('uses stricter deterministic thresholds for kundali timeline sections', () => {
    const rubric = auditSectionOutput({
      sectionId: 'master-timeline',
      title: 'Part IX',
      targetWords: 100,
      output: 'Vimshottari dasha and Sade Sati are mentioned once.',
      modelRequested: 'tier-default',
      modelUsed: 'tier-default',
      latencyMs: 1,
    });

    expect(rubric.deterministic_fact_gate).toBe('fail');
  });

  // Minimal helper factories for fidelity test (as specified in plan)
  function makeEngineWithLagna(lagna: string): any {
    return { result: { lagna_sign: lagna } };
  }
  function makeEngineWithGate(gate: number): any {
    return { result: { gates: [gate] } };
  }

  it('computes chart_fidelity_score when engine results are supplied', () => {
    const engines = [makeEngineWithLagna('aries'), makeEngineWithGate(34)];
    const rubric = auditSectionOutput({
      sectionId: 'opening',
      title: 'Opening',
      targetWords: 100,
      output: 'Lagna is Aries and Human Design gate 34 is active.',
      modelRequested: 'tier-default',
      modelUsed: 'tier-default',
      latencyMs: 10,
      engineResults: engines,
    });

    expect(rubric.chart_fidelity_score).toBeGreaterThanOrEqual(0.5);
    expect(rubric.chart_fidelity_score).toBeLessThanOrEqual(1.0);
  });

  const sampleEngineResults = [
    {
      engine_id: 'panchanga',
      result: { tithi_name: 'Navami (Krishna)', nakshatra_name: 'Pushya' },
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
    },
  ];

  it('flags unsubstituted placeholders', () => {
    const rubric = auditSectionOutput({
      sectionId: 'opening',
      title: 'Opening',
      targetWords: 450,
      output: 'Your Lagna is [exact sign from engine results].',
      modelRequested: 'test',
      modelUsed: 'test',
      latencyMs: 0,
      engineResults: sampleEngineResults,
    });
    expect(rubric.placeholder_gate).toBe('fail');
  });

  it('fails chart fidelity when score is below threshold', () => {
    const rubric = auditSectionOutput({
      sectionId: 'vedic-foundation',
      title: 'Vedic Foundation',
      targetWords: 3800,
      output: 'Generic text with no specific facts.',
      modelRequested: 'test',
      modelUsed: 'test',
      latencyMs: 0,
      engineResults: sampleEngineResults,
    });
    expect(rubric.chart_fidelity_gate).toBe('fail');
  });

  it('extracts section-specific health facts from biofield and biorhythm engines', () => {
    const engines = [
      {
        engine_id: 'biofield',
        result: {
          areas_of_attention: ['liver', 'adrenal rhythm'],
          chakra_readings: [{ chakra: 'manipura' }, { chakra: 'anahata' }],
        },
      },
      {
        engine_id: 'biorhythm',
        result: { physical: 'high', emotional: 'low', spiritual: 'neutral', overall_energy: 'recalibrating' },
      },
    ];
    const facts = extractSectionFacts(engines, 'health');
    expect(facts.size).toBeGreaterThan(0);
    const factValues = Array.from(facts).map((f) => f.split(':')[1]);
    expect(factValues).toContain('liver');
    expect(factValues).toContain('manipura');
    expect(factValues).toContain('high');

    const rubric = auditSectionOutput({
      sectionId: 'health',
      title: 'Health',
      targetWords: 200,
      output: 'The biofield flags liver and adrenal rhythm, with manipura and anahata active. Biorhythm physical is high while emotional is low and overall energy is recalibrating.',
      modelRequested: 'test',
      modelUsed: 'test',
      latencyMs: 0,
      engineResults: engines,
    });
    expect(rubric.chart_fidelity_score).toBeGreaterThan(0.5);
  });

  it('extracts section-specific timeline facts from vimshottari and transits engines', () => {
    const engines = [
      {
        engine_id: 'vimshottari',
        result: {
          current_period: { mahadasha: { planet: 'Saturn' }, antardasha: { planet: 'Venus' } },
          upcoming_transitions: [{ event: 'Jupiter antardasha', year: 2027 }, { event: 'Saturn return', year: 2031 }],
        },
      },
      {
        engine_id: 'transits',
        result: { sade_sati: 'peak', retrograde_planets: ['Mercury', 'Saturn'] },
      },
    ];
    const facts = extractSectionFacts(engines, 'master-timeline');
    const factValues = Array.from(facts).map((f) => f.split(':')[1]);
    expect(factValues).toContain('saturn');
    expect(factValues).toContain('venus');
    expect(factValues).toContain('jupiter antardasha');
    expect(factValues).toContain('peak');

    const rubric = auditSectionOutput({
      sectionId: 'master-timeline',
      title: 'Master Timeline',
      targetWords: 300,
      output: 'You are in Saturn mahadasha with Venus antardasha. The next major transition is Jupiter antardasha in 2027. Sade Sati is at peak and Mercury is retrograde.',
      modelRequested: 'test',
      modelUsed: 'test',
      latencyMs: 0,
      engineResults: engines,
    });
    expect(rubric.chart_fidelity_score).toBeGreaterThan(0.5);
  });

  it('extracts short keyword tokens from biofield areas_of_attention for remedies-practices', () => {
    const engines = [
      {
        engine_id: 'nadabrahman',
        result: {
          recommendations: [
            {
              raga_name: 'Harikambhoji',
              mood: 'bhakti',
              reason: 'Balances pitta energy in afternoon hours',
              therapeutic_qualities: ['calming', 'devotional', 'centering'],
            },
          ],
        },
      },
      {
        engine_id: 'biofield',
        result: {
          areas_of_attention: [
            'Vishuddha (Throat) (Throat area) - lower activity linked to Mercury placement',
            'Anahata (Heart) - balanced',
          ],
        },
      },
    ];
    const facts = extractSectionFacts(engines, 'remedies-practices');
    const values = Array.from(facts).map((f) => f.split(':')[1]);

    expect(values).toContain('vishuddha');
    expect(values).toContain('throat');
    expect(values).toContain('mercury');
    expect(values).toContain('lower');
    expect(values).toContain('anahata');
    expect(values).toContain('heart');
    expect(values).toContain('balanced');
    expect(values).toContain('harikambhoji');
    expect(values).toContain('bhakti');
    expect(values).toContain('calming');

    const rubric = auditSectionOutput({
      sectionId: 'remedies-practices',
      title: 'Remedies & Practices',
      targetWords: 200,
      output: 'Your Throat chakra (Vishuddha) shows lower activity and relates to Mercury. The Heart center is balanced. Listen to Harikambhoji in the afternoon for its calming bhakti qualities.',
      modelRequested: 'test',
      modelUsed: 'test',
      latencyMs: 0,
      engineResults: engines,
    });
    expect(rubric.chart_fidelity_score).toBeGreaterThan(0.5);
    expect(rubric.chart_fidelity_gate).toBe('pass');
  });

  it('extracts section-specific family-lineage facts from engines', () => {
    const engines = [
      {
        engine_id: 'numerology',
        result: { life_path: 7, soul_urge: 3, expression: 9 },
      },
      {
        engine_id: 'human-design',
        result: { profile: '5/1', hd_type: 'Generator', authority: 'Sacral', defined_centers: ['Root', 'Sacral', 'G', 'Spleen'] },
      },
      {
        engine_id: 'gene-keys',
        result: { active_keys: ['genekey-1', 'genekey-2'] },
      },
    ];
    const facts = extractSectionFacts(engines, 'family-lineage');
    const factValues = Array.from(facts).map((f) => f.split(':')[1]);
    expect(factValues).toContain('7');
    expect(factValues).toContain('5/1');
    expect(factValues).toContain('genekey-1');
    expect(factValues).toContain('generator');
    expect(factValues).toContain('sacral');
    expect(factValues).toContain('root');
    expect(factValues).toContain('spleen');
  });
});
