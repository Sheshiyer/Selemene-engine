import { describe, it, expect, vi } from 'vitest';
import { MatrixRunner } from './runner.js';
import type { GoldenFile, Subject } from './types.js';

describe('MatrixRunner.verify', () => {
  it('returns 100% accuracy when all fields match', async () => {
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
});
