import { describe, it, expect } from 'vitest';
import { runChainAudit, verifyL0FinalArtifacts, CANONICAL_L0_FIVE } from './audit.js';
import type { SelemeneEngineOutput } from '../index.js';

function makeEngine(id: string, deterministic = true): SelemeneEngineOutput {
  return {
    engine_id: id as SelemeneEngineOutput['engine_id'],
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

describe('verifyL0FinalArtifacts', () => {
  it('passes when PDF + reading.md exist and all 5 canonical engines are present', () => {
    const engines: SelemeneEngineOutput[] = [
      makeEngine('panchanga'),
      makeEngine('vimshottari'),
      makeEngine('human-design'),
      makeEngine('gene-keys'),
      makeEngine('transits'),
      makeEngine('numerology'),
    ];
    const res = verifyL0FinalArtifacts({
      engines,
      readingMarkdown: 'Vedic panchanga, Vimshottari dasha, Human Design gates, Gene Keys, transits.',
      pdfExists: true,
      readingMdExists: true,
    });
    expect(res.passed).toBe(true);
    expect(res.blockers).toHaveLength(0);
    expect(res.details.canonical_five_present.length).toBe(5);
    expect(res.details.missing_canonical).toHaveLength(0);
  });

  it('blocks when PDF is missing', () => {
    const engines: SelemeneEngineOutput[] = [
      makeEngine('panchanga'),
      makeEngine('vimshottari'),
      makeEngine('human-design'),
      makeEngine('gene-keys'),
      makeEngine('transits'),
    ];
    const res = verifyL0FinalArtifacts({
      engines,
      readingMarkdown: 'ok',
      pdfExists: false,
      readingMdExists: true,
    });
    expect(res.passed).toBe(false);
    expect(res.blockers.some((b) => /pdf/i.test(b))).toBe(true);
  });

  it('blocks when any of the 5 canonical engines are missing or errored', () => {
    const engines: SelemeneEngineOutput[] = [
      makeEngine('panchanga'),
      makeEngine('vimshottari'),
      makeEngine('human-design'),
      // gene-keys and transits missing
    ];
    const res = verifyL0FinalArtifacts({
      engines,
      readingMarkdown: 'ok',
      pdfExists: true,
      readingMdExists: true,
    });
    expect(res.passed).toBe(false);
    expect(res.details.missing_canonical.sort()).toEqual(['gene-keys', 'transits']);
  });

  it('exposes citation counts when requireCitations is true (counts only, not a gate)', () => {
    const engines: SelemeneEngineOutput[] = [
      makeEngine('panchanga'),
      makeEngine('vimshottari'),
      makeEngine('human-design'),
      makeEngine('gene-keys'),
      makeEngine('transits'),
    ];
    const res = verifyL0FinalArtifacts({
      engines,
      readingMarkdown: 'panchanga vimshottari human design gene keys transits transits',
      pdfExists: true,
      readingMdExists: true,
      requireCitations: true,
    });
    expect(res.passed).toBe(true);
    expect(res.details.citation_counts).toBeDefined();
    expect(res.details.citation_counts?.['transits']).toBeGreaterThanOrEqual(2);
  });

  it('exposes CANONICAL_L0_FIVE as the exact five-system contract', () => {
    expect(CANONICAL_L0_FIVE).toEqual(['panchanga', 'vimshottari', 'human-design', 'gene-keys', 'transits']);
  });
});
