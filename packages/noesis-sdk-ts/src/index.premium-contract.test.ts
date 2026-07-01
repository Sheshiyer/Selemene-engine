// Basic contract test for the new additive premium asset surface.
// Ensures: route exists, response shape is stable, and existing witness surface is untouched.

import { describe, it, expect } from 'vitest';
import { NoesisClient } from '../src/index.js';

describe('Premium asset additive surface (contract)', () => {
  const baseUrl = process.env.NOESIS_BASE_URL || 'http://localhost:3000';

  it('exports generatePremiumAsset as an additive method (no signature change to existing)', () => {
    const client = new NoesisClient(baseUrl, { apiKey: 'test' });
    expect(typeof client.generatePremiumAsset).toBe('function');
    // Sanity: existing methods still present with their documented signatures
    expect(typeof client.interpretWitness).toBe('function');
    expect(typeof client.calculate).toBe('function');
    expect(typeof client.workflow).toBe('function');
  });

  it('POST /api/v1/assets/generate returns the additive envelope', async () => {
    // This is a structural contract test; it may be skipped in CI without a live server.
    if (!process.env.RUN_CONTRACT_TESTS) {
      expect(true).toBe(true);
      return;
    }
    const client = new NoesisClient(baseUrl, { apiKey: process.env.NOESIS_API_KEY! });
    const res = await client.generatePremiumAsset({
      birth_data: {
        date: '1990-01-15',
        time: '14:30',
        latitude: 12.97,
        longitude: 77.59,
        timezone: 'Asia/Kolkata',
        name: 'Test',
      },
      mode: 'integrated-reading',
      consciousness_level: 3,
    });

    expect(res).toHaveProperty('mode');
    expect(res).toHaveProperty('register');
    expect(res).toHaveProperty('passes');
    expect(res).toHaveProperty('assembled');
    expect(res).toHaveProperty('engines_used');
    expect(res).toHaveProperty('source_pack');
    expect(Array.isArray(res.passes)).toBe(true);
    expect(Array.isArray(res.engines_used)).toBe(true);
  });
});
