import { describe, expect, it, vi, beforeEach, afterEach } from 'vitest';
import {
  normalizeManualLocation,
  searchBirthplace,
  confirmNormalizedLocation,
  isValidIANATimezone,
} from './location.js';
import type { LocationCandidate } from './location.js';

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

  it('throws on invalid latitude', () => {
    expect(() =>
      normalizeManualLocation({ displayName: 'x', latitude: '999', longitude: '0', timezone: 'UTC' })
    ).toThrow('Invalid latitude');
  });

  it('throws on invalid longitude', () => {
    expect(() =>
      normalizeManualLocation({ displayName: 'x', latitude: '0', longitude: '200', timezone: 'UTC' })
    ).toThrow('Invalid longitude');
  });

  it('throws on empty timezone', () => {
    expect(() =>
      normalizeManualLocation({ displayName: 'x', latitude: '0', longitude: '0', timezone: '  ' })
    ).toThrow('Timezone is required');
  });
});

describe('isValidIANATimezone', () => {
  it('accepts valid IANA timezones', () => {
    expect(isValidIANATimezone('Asia/Kolkata')).toBe(true);
    expect(isValidIANATimezone('America/New_York')).toBe(true);
    expect(isValidIANATimezone('UTC')).toBe(true);
    expect(isValidIANATimezone('Europe/London')).toBe(true);
  });

  it('rejects invalid timezones', () => {
    expect(isValidIANATimezone('')).toBe(false);
    expect(isValidIANATimezone('Asia/Nowhere')).toBe(false);
    expect(isValidIANATimezone('not-a-tz')).toBe(false);
  });
});

describe('searchBirthplace', () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it('returns empty in privacy mode', async () => {
    const res = await searchBirthplace('Bangalore, India', { privacyMode: true });
    expect(res).toEqual([]);
  });

  it('returns empty for empty query', async () => {
    const res = await searchBirthplace('   ');
    expect(res).toEqual([]);
  });

  it('returns candidates from mocked Nominatim', async () => {
    const mockFetch = vi.fn().mockResolvedValue({
      ok: true,
      json: async () => [
        { display_name: 'Bengaluru, Karnataka, India', lat: '12.9716', lon: '77.5946' },
        { display_name: 'Bangalore, India', lat: '12.97', lon: '77.59' },
      ],
    });

    const res = await searchBirthplace('Bangalore, India', { fetchImpl: mockFetch as any });
    expect(res).toEqual([
      { display_name: 'Bengaluru, Karnataka, India', latitude: 12.9716, longitude: 77.5946, provider: 'nominatim' },
      { display_name: 'Bangalore, India', latitude: 12.97, longitude: 77.59, provider: 'nominatim' },
    ]);
    expect(mockFetch).toHaveBeenCalledOnce();
  });

  it('returns empty array on network error (graceful)', async () => {
    const mockFetch = vi.fn().mockRejectedValue(new Error('network'));
    const resPromise = searchBirthplace('Bangalore', { fetchImpl: mockFetch as any });
    await vi.runAllTimersAsync();
    const res = await resPromise;
    expect(res).toEqual([]);
  });

  it('returns empty array on non-ok response', async () => {
    const mockFetch = vi.fn().mockResolvedValue({ ok: false });
    const resPromise = searchBirthplace('Bangalore', { fetchImpl: mockFetch as any });
    await vi.runAllTimersAsync();
    const res = await resPromise;
    expect(res).toEqual([]);
  });

  it('rate-limits between calls', async () => {
    const mockFetch = vi
      .fn()
      .mockResolvedValueOnce({ ok: true, json: async () => [{ display_name: 'A', lat: '1', lon: '1' }] })
      .mockResolvedValueOnce({ ok: true, json: async () => [{ display_name: 'B', lat: '2', lon: '2' }] });

    const p1 = searchBirthplace('q1', { fetchImpl: mockFetch as any });
    await vi.runAllTimersAsync();
    await p1;

    const p2 = searchBirthplace('q2', { fetchImpl: mockFetch as any });
    await vi.advanceTimersByTimeAsync(500);
    const res2Promise = p2;

    await vi.advanceTimersByTimeAsync(700);
    await res2Promise;

    expect(mockFetch).toHaveBeenCalledTimes(2);
  });
});

describe('confirmNormalizedLocation', () => {
  const candidate: LocationCandidate = {
    display_name: 'Bengaluru, Karnataka, India',
    latitude: 12.9716,
    longitude: 77.5946,
    provider: 'nominatim',
  };

  it('confirms a candidate with explicit IANA timezone', () => {
    const loc = confirmNormalizedLocation({ candidate, timezone: 'Asia/Kolkata' });
    expect(loc).toEqual({
      display_name: 'Bengaluru, Karnataka, India',
      latitude: 12.9716,
      longitude: 77.5946,
      timezone: 'Asia/Kolkata',
      provider: 'nominatim',
      confidence: 'selected',
    });
  });

  it('prefers manual input when both provided', () => {
    const loc = confirmNormalizedLocation({
      candidate,
      manual: { displayName: 'Manual Place', latitude: 1, longitude: 2, timezone: 'UTC' },
      timezone: 'Asia/Kolkata',
    });
    expect(loc.provider).toBe('manual');
    expect(loc.display_name).toBe('Manual Place');
  });

  it('throws if neither candidate nor manual provided', () => {
    expect(() => confirmNormalizedLocation({ timezone: 'UTC' })).toThrow('requires either candidate or manual');
  });

  it('throws if timezone missing for candidate', () => {
    expect(() => confirmNormalizedLocation({ candidate })).toThrow('Timezone is required');
  });

  it('throws if timezone is invalid IANA', () => {
    expect(() => confirmNormalizedLocation({ candidate, timezone: 'Not/AZone' })).toThrow('Invalid IANA timezone');
  });
});
