import { describe, it, expect, vi } from 'vitest';
import { fetchAllEngines, SELEMENE_BASE_URL, type BirthData } from './fetcher.js';
import capabilityFixture from '../../../../contracts/v1/fixtures/engine-capability-list.json';
import { decodeEngineCapabilityList } from '@selemene/engine-sdk';

describe('fetchAllEngines', () => {
  it('preflights the canonical envelope and fans out exactly sixteen eligible calls', async () => {
    const calls: Array<{ url: string; init?: RequestInit }> = [];
    const fakeFetch = vi.fn(async (url: string, init?: RequestInit) => {
      calls.push({ url, init });
      if (url.endsWith('/engines/capabilities')) {
        return { ok: true, text: async () => JSON.stringify(capabilityFixture) } as unknown as Response;
      }
      return { ok: true, text: async () => JSON.stringify({ engine_id: url.split('/').at(-2), result: {} }) } as unknown as Response;
    });

    const decoded = decodeEngineCapabilityList(capabilityFixture);
    const results = await fetchAllEngines(
      { date: '1990-01-01', timezone: 'UTC' },
      { api_key: 'test-key', base_url: 'http://localhost:9999', fetchImpl: fakeFetch as unknown as typeof fetch },
    );

    expect(decoded.capabilities).toHaveLength(19);
    expect(decoded.public_mirror_count).toBe(17);
    expect(results).toHaveLength(16);
    expect(calls).toHaveLength(17);
    expect(calls[0].url).toBe('http://localhost:9999/api/v1/engines/capabilities');
    expect((calls[0].init?.headers as Record<string, string>)['X-API-Key']).toBe('test-key');
    expect(calls.slice(1).map((call) => call.url.replace(/.*\/engines\/(.*)\/calculate$/, '$1'))).toEqual([
      'panchanga', 'vimshottari', 'human-design', 'gene-keys', 'numerology', 'biorhythm', 'vedic-clock',
      'biofield', 'face-reading', 'nadabrahman', 'transits', 'tarot', 'i-ching', 'enneagram', 'sacred-geometry', 'sigil-forge',
    ]);
  });

  it('rejects malformed preflight before making any calculate call', async () => {
    const calls: string[] = [];
    const fakeFetch = vi.fn(async (url: string) => {
      calls.push(url);
      return { ok: true, text: async () => JSON.stringify({ ...capabilityFixture, count: 18 }) } as unknown as Response;
    });
    await expect(fetchAllEngines(
      { date: '1990-01-01', timezone: 'UTC' },
      { api_key: 'test-key', base_url: 'http://localhost:9999', fetchImpl: fakeFetch as unknown as typeof fetch },
    )).rejects.toThrow();
    expect(calls).toHaveLength(1);
    expect(calls[0]).toContain('/engines/capabilities');
  });

  it('rejects raaga, runtime-only identities and unknown IDs before calculation', async () => {
    const fakeFetch = vi.fn();
    await expect(fetchAllEngines(
      { date: '1990-01-01', timezone: 'UTC' },
      { api_key: 'test-key', engines: ['raaga'], capabilities: capabilityFixture, fetchImpl: fakeFetch as unknown as typeof fetch },
    )).rejects.toThrow(/not an eligible/);
    expect(fakeFetch).not.toHaveBeenCalled();
  });

  it('returns engine results when all engines respond', async () => {
    const birthData: BirthData = {
      date: '1990-01-01',
      time: '12:00',
      timezone: 'Asia/Kolkata',
      latitude: 12.9716,
      longitude: 77.5946,
      name: 'Test',
    };

    const fakeFetch = vi.fn().mockResolvedValue({
      ok: true,
      json: async () => ({ engine_id: 'panchanga', result: { tithi_name: 'Test' } }),
    });

    const results = await fetchAllEngines(birthData, {
      api_key: 'test-key',
      base_url: 'http://localhost:9999',
      timeout_ms: 1000,
      engines: ['panchanga'],
      capabilities: capabilityFixture,
      fetchImpl: fakeFetch as unknown as typeof fetch,
    });

    expect(results).toHaveLength(1);
    expect(results[0].engine_id).toBe('panchanga');
    expect(fakeFetch).toHaveBeenCalledWith(
      'http://localhost:9999/api/v1/engines/panchanga/calculate',
      expect.objectContaining({
        method: 'POST',
        headers: expect.objectContaining({ 'X-API-Key': 'test-key', 'Content-Type': 'application/json' }),
      })
    );
  });

  it('returns _error for failed engine calls', async () => {
    const fakeFetch = vi.fn().mockResolvedValue({
      ok: false,
      status: 502,
      text: async () => 'boom',
    });

    const results = await fetchAllEngines(
      { date: '1990-01-01', timezone: 'UTC', latitude: 0, longitude: 0 },
      { api_key: 'k', engines: ['panchanga'], capabilities: capabilityFixture, fetchImpl: fakeFetch as unknown as typeof fetch }
    );

    expect(results[0]._error).toBe('HTTP 502');
  });
});
