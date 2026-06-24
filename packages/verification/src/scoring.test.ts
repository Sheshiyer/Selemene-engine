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

  it('accepts a zero weight', () => {
    const outcome = compareField('a', 'b', 0);
    expect(outcome.weight).toBe(0);
    expect(outcome.pass).toBe(false);
  });

  it('throws on negative weight', () => {
    expect(() => compareField('a', 'b', -1)).toThrow(/finite non-negative/);
  });

  it('throws on NaN weight', () => {
    expect(() => compareField('a', 'b', NaN)).toThrow(/finite non-negative/);
  });

  it('throws on Infinity weight', () => {
    expect(() => compareField('a', 'b', Infinity)).toThrow(/finite non-negative/);
  });

  it('throws on -Infinity weight', () => {
    expect(() => compareField('a', 'b', -Infinity)).toThrow(/finite non-negative/);
  });

  it('distinguishes Date objects with different timestamps', () => {
    const outcome = compareField(new Date('2024-01-01'), new Date('2024-01-02'), 1);
    expect(outcome.pass).toBe(false);
  });

  it('matches Date objects with identical timestamps', () => {
    const ts = new Date('2024-01-01').getTime();
    const outcome = compareField(new Date(ts), new Date(ts), 1);
    expect(outcome.pass).toBe(true);
  });

  it('treats Date and plain object as not equal', () => {
    const outcome = compareField(new Date('2024-01-01'), {}, 1);
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

  it('returns 0 when total weight is 0', () => {
    const fields = {
      a: { pass: true, expected: 1, actual: 1, weight: 0 },
      b: { pass: false, expected: 2, actual: 3, weight: 0 },
    };
    expect(calculateAccuracy(fields)).toBe(0);
  });
});

describe('deepEqual edge cases', () => {
  it('matches nested objects', () => {
    expect(compareField({ a: { b: 1 } }, { a: { b: 1 } }, 1).pass).toBe(true);
  });

  it('distinguishes nested objects', () => {
    expect(compareField({ a: { b: 1 } }, { a: { b: 2 } }, 1).pass).toBe(false);
  });

  it('matches arrays', () => {
    expect(compareField([1, 2, 3], [1, 2, 3], 1).pass).toBe(true);
  });

  it('distinguishes arrays of different length', () => {
    expect(compareField([1, 2], [1, 2, 3], 1).pass).toBe(false);
  });

  it('distinguishes arrays with different values', () => {
    expect(compareField([1, 2, 3], [1, 2, 4], 1).pass).toBe(false);
  });

  it('handles null against object', () => {
    expect(compareField(null, { a: 1 }, 1).pass).toBe(false);
  });

  it('handles undefined against value', () => {
    expect(compareField(undefined, 'value', 1).pass).toBe(false);
  });

  it('handles null against null', () => {
    expect(compareField(null, null, 1).pass).toBe(true);
  });

  it('returns false on type mismatch', () => {
    expect(compareField('1', 1, 1).pass).toBe(false);
  });
});
