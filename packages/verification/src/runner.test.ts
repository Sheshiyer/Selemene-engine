import { describe, it, expect, vi } from 'vitest';
import { MatrixRunner } from './runner.js';
import { extract } from './extract.js';
import type { GoldenFile, Subject } from './types.js';

const subject: Subject = {
  id: 'sahil',
  name: 'Sahil Singh Sabharwal',
  birth: {
    date: '1992-03-14',
    time: '02:22:00',
    timezone: 'Asia/Kolkata',
    location: { place: 'Bengaluru', latitude: 12.9716, longitude: 77.5946 },
  },
  added: '2026-06-24',
};

describe('MatrixRunner.verify', () => {
  it('returns 100% accuracy when all fields match', async () => {
    const golden: GoldenFile = {
      subject: 'sahil',
      engine: 'panchanga',
      source: 'drikpanchang.com',
      captured: '2026-06-24',
      fields: {
        vara: { expected: 'Shanivara (Saturday)', weight: 1.0 },
      },
    };

    const runner = new MatrixRunner({
      engine: 'panchanga',
      fetchEngine: vi.fn().mockResolvedValue({ vara_name: 'Shanivara (Saturday)' }),
    });

    const result = await runner.verify(subject, golden);

    expect(result.accuracy).toBe(1.0);
    expect(result.fields.vara.pass).toBe(true);
  });

  it('returns error and empty missingFields when fetchEngine throws', async () => {
    const golden: GoldenFile = {
      subject: 'sahil',
      engine: 'panchanga',
      source: 'drikpanchang.com',
      captured: '2026-06-24',
      fields: {
        vara: { expected: 'Shanivara (Saturday)', weight: 1.0 },
      },
    };

    const runner = new MatrixRunner({
      engine: 'panchanga',
      fetchEngine: vi.fn().mockRejectedValue(new Error('API failure')),
    });

    const result = await runner.verify(subject, golden);

    expect(result.error).toBe('API failure');
    expect(result.missingFields).toEqual([]);
    expect(result.accuracy).toBe(0);
  });

  it('records missing field in missingFields when actual is undefined', async () => {
    const golden: GoldenFile = {
      subject: 'sahil',
      engine: 'panchanga',
      source: 'drikpanchang.com',
      captured: '2026-06-24',
      fields: {
        vara: { expected: 'Shanivara (Saturday)', weight: 1.0 },
        tithi: { expected: 'Amavasya', weight: 1.0 },
      },
    };

    const runner = new MatrixRunner({
      engine: 'panchanga',
      fetchEngine: vi.fn().mockResolvedValue({ vara_name: 'Shanivara (Saturday)' }),
    });

    const result = await runner.verify(subject, golden);

    expect(result.missingFields).toEqual(['tithi']);
    expect(result.fields.vara.pass).toBe(true);
  });

  it('returns error when golden engine does not match runner engine', async () => {
    const golden: GoldenFile = {
      subject: 'sahil',
      engine: 'human-design',
      source: 'test',
      captured: '2026-06-24',
      fields: {
        type: { expected: 'Generator', weight: 1.0 },
      },
    };

    const runner = new MatrixRunner({
      engine: 'panchanga',
      fetchEngine: vi.fn().mockResolvedValue({}),
    });

    const result = await runner.verify(subject, golden);

    expect(result.error).toContain('Engine mismatch');
    expect(result.missingFields).toEqual([]);
    expect(result.accuracy).toBe(0);
    expect(result.fields).toEqual({});
    expect(runner['fetchEngine']).not.toHaveBeenCalled();
  });
});

describe('extract array path', () => {
  it('returns the value at a valid array index', () => {
    expect(extract({ items: ['a', 'b', 'c'] }, 'items.1')).toBe('b');
  });

  it('returns undefined for a negative array index', () => {
    expect(extract({ items: ['a', 'b', 'c'] }, 'items.-1')).toBeUndefined();
  });

  it('returns undefined for a non-integer array key', () => {
    expect(extract({ items: ['a', 'b', 'c'] }, 'items.foo')).toBeUndefined();
  });
});
