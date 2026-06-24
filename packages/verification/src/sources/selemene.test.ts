import { describe, it, expect, vi } from 'vitest';
import { fetchEngineResult, SelemeneApiError } from './selemene.js';
import type { Subject } from '../types.js';

const mockFetchAllEngines = vi.fn();
const mockLoadSelemeneKey = vi.fn();

vi.mock('@noesis/witness-pipeline', () => ({
  fetchAllEngines: (...args: unknown[]) => mockFetchAllEngines(...args),
  loadSelemeneKey: () => mockLoadSelemeneKey(),
}));

const subject: Subject = {
  id: 'test-subject',
  name: 'Test Subject',
  birth: {
    date: '1990-01-01',
    time: '12:00',
    timezone: 'UTC',
    location: {
      place: 'Test City',
      latitude: 0,
      longitude: 0,
    },
  },
  added: '2024-01-01',
};

describe('fetchEngineResult', () => {
  it('throws a clear error when the API key is missing', async () => {
    mockLoadSelemeneKey.mockResolvedValue(undefined);

    await expect(fetchEngineResult(subject, 'panchanga')).rejects.toThrow(
      'SELEMENE_API_KEY not found. Set the SELEMENE_API_KEY environment variable.'
    );
  });

  it('throws SelemeneApiError when the engine returns an _error response', async () => {
    mockLoadSelemeneKey.mockResolvedValue('test-api-key');
    mockFetchAllEngines.mockResolvedValue([
      {
        engine_id: 'panchanga',
        result: null,
        witness_prompt: '',
        consciousness_level: 0,
        metadata: {
          calculation_time_ms: 0,
          backend: 'test',
          precision_achieved: 'none',
          cached: false,
          timestamp: new Date().toISOString(),
          engine_version: '0.0.0',
        },
        envelope_version: '1.0.0',
        _error: 'calculation failed',
      },
    ]);

    await expect(fetchEngineResult(subject, 'panchanga')).rejects.toThrow(SelemeneApiError);
    await expect(fetchEngineResult(subject, 'panchanga')).rejects.toThrow(
      'Selemene panchanga failed: calculation failed'
    );
  });

  it('returns the result for a successful response', async () => {
    const expectedResult = { tithi: 'Pratipada' };
    mockLoadSelemeneKey.mockResolvedValue('test-api-key');
    mockFetchAllEngines.mockResolvedValue([
      {
        engine_id: 'panchanga',
        result: expectedResult,
        witness_prompt: '',
        consciousness_level: 1,
        metadata: {
          calculation_time_ms: 100,
          backend: 'test',
          precision_achieved: 'high',
          cached: false,
          timestamp: new Date().toISOString(),
          engine_version: '1.0.0',
        },
        envelope_version: '1.0.0',
      },
    ]);

    const result = await fetchEngineResult(subject, 'panchanga');
    expect(result).toEqual(expectedResult);
  });

  it('throws SelemeneApiError when the result array length is unexpected', async () => {
    mockLoadSelemeneKey.mockResolvedValue('test-api-key');
    mockFetchAllEngines.mockResolvedValue([]);

    await expect(fetchEngineResult(subject, 'panchanga')).rejects.toThrow(SelemeneApiError);
    await expect(fetchEngineResult(subject, 'panchanga')).rejects.toThrow(
      'Expected exactly one result for panchanga, got 0'
    );
  });
});
