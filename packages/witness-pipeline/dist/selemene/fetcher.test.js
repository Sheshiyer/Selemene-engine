import { describe, it, expect, vi } from 'vitest';
import { fetchAllEngines } from './fetcher.js';
describe('fetchAllEngines', () => {
    it('returns engine results when all engines respond', async () => {
        const birthData = {
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
            fetchImpl: fakeFetch,
        });
        expect(results).toHaveLength(1);
        expect(results[0].engine_id).toBe('panchanga');
        expect(fakeFetch).toHaveBeenCalledWith('http://localhost:9999/api/v1/engines/panchanga/calculate', expect.objectContaining({
            method: 'POST',
            headers: expect.objectContaining({ 'X-API-Key': 'test-key', 'Content-Type': 'application/json' }),
        }));
    });
    it('returns _error for failed engine calls', async () => {
        const fakeFetch = vi.fn().mockResolvedValue({
            ok: false,
            text: async () => 'boom',
        });
        const results = await fetchAllEngines({ date: '1990-01-01', timezone: 'UTC', latitude: 0, longitude: 0 }, { api_key: 'k', engines: ['panchanga'], fetchImpl: fakeFetch });
        expect(results[0]._error).toContain('boom');
    });
});
//# sourceMappingURL=fetcher.test.js.map