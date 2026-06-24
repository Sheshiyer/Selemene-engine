import { describe, it, expect } from 'vitest';
import { calculateAccuracy, compareField } from './scoring.js';

describe('compareField', () => {
  it('passes on exact match', () => {
    const outcome = compareField('Shanivara', 'Shanivara', 0.2);
    expect(outcome.pass).toBe(true);
    expect(outcome.weight).toBe(0.2);
  });

  it('fails on mismatch', () => {
    const outcome = compareField('Shanivara', 'Shukravara', 0.2);
    expect(outcome.pass).toBe(false);
  });
});

describe('calculateAccuracy', () => {
  it('returns 1.0 when all fields pass', () => {
    const fields = {
      a: { pass: true, expected: 1, actual: 1, weight: 0.5 },
      b: { pass: true, expected: 2, actual: 2, weight: 0.5 },
    };
    expect(calculateAccuracy(fields)).toBe(1.0);
  });

  it('returns weighted accuracy when some fields fail', () => {
    const fields = {
      a: { pass: true, expected: 1, actual: 1, weight: 0.8 },
      b: { pass: false, expected: 2, actual: 3, weight: 0.2 },
    };
    expect(calculateAccuracy(fields)).toBe(0.8);
  });
});
