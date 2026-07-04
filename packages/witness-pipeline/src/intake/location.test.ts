import { describe, expect, it } from 'vitest';
import { normalizeManualLocation } from './location.js';

describe('normalizeManualLocation', () => {
  it('normalizes manually supplied coordinates and timezone', () => {
    const location = normalizeManualLocation({
      displayName: 'Bengaluru, Karnataka, India',
      latitude: '12.9716',
      longitude: '77.5946',
      timezone: 'Asia/Kolkata',
    });

    expect(location).toEqual({
      display_name: 'Bengaluru, Karnataka, India',
      latitude: 12.9716,
      longitude: 77.5946,
      timezone: 'Asia/Kolkata',
      provider: 'manual',
      confidence: 'manual',
    });
  });
});
