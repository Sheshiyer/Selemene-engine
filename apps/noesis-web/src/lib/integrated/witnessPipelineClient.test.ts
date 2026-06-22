import { describe, it, expect } from 'vitest';
import { buildIntegratedReadingPayload } from './witnessPipelineClient.js';

describe('buildIntegratedReadingPayload', () => {
  it('serializes birth data', () => {
    const payload = buildIntegratedReadingPayload({
      name: 'Arathi',
      birthDate: '1980-03-15',
      birthTime: '06:30',
      timezone: 'Asia/Kolkata',
      latitude: 12.9716,
      longitude: 77.5946,
    });
    expect(payload.birth_data.name).toBe('Arathi');
    expect(payload.birth_data.date).toBe('1980-03-15');
    expect(payload.birth_data.time).toBe('06:30');
    expect(payload.birth_data.timezone).toBe('Asia/Kolkata');
  });

  it('omits optional fields when not provided', () => {
    const payload = buildIntegratedReadingPayload({
      birthDate: '1980-03-15',
      timezone: 'Asia/Kolkata',
      latitude: 12.9716,
      longitude: 77.5946,
    });
    expect(payload.birth_data.name).toBeUndefined();
    expect(payload.birth_data.time).toBeUndefined();
  });
});
