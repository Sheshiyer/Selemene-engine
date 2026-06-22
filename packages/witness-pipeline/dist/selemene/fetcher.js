// ─── Selemene Engine Fetcher ─────────────────────────────────────────
// Ported from witness-agents/scripts/integratedreading/selemene/fetcher.ts
// Adds dependency-injected fetchImpl for testability.
import { SELEMENE_ENGINE_IDS } from './types.js';
export { SELEMENE_ENGINE_IDS } from './types.js';
export const SELEMENE_BASE_URL = 'https://selemene.tryambakam.space';
async function callEngine(engineId, birthData, opts) {
    const url = `${opts.base_url ?? SELEMENE_BASE_URL}/api/v1/engines/${engineId}/calculate`;
    const ctrl = new AbortController();
    const t = setTimeout(() => ctrl.abort(), opts.timeout_ms ?? 30_000);
    const fetchImpl = opts.fetchImpl ?? fetch;
    try {
        const res = await fetchImpl(url, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
                'X-API-Key': opts.api_key,
            },
            body: JSON.stringify({ birth_data: birthData }),
            signal: ctrl.signal,
        });
        clearTimeout(t);
        if (!res.ok) {
            const txt = await res.text().catch(() => '');
            return { engine_id: engineId, _error: `HTTP ${res.status}: ${txt.slice(0, 200)}` };
        }
        const data = await res.json();
        return { ...data, engine_id: engineId };
    }
    catch (err) {
        clearTimeout(t);
        return { engine_id: engineId, _error: err instanceof Error ? err.message : String(err) };
    }
}
export async function fetchAllEngines(birthData, opts) {
    const engines = opts.engines ?? Array.from(SELEMENE_ENGINE_IDS);
    const results = await Promise.all(engines.map((eId) => callEngine(eId, birthData, opts)));
    return results;
}
export async function loadSelemeneKey() {
    if (process.env.SELEMENE_API_KEY)
        return process.env.SELEMENE_API_KEY;
    const { default: fs } = await import('node:fs');
    const { default: path } = await import('node:path');
    const { default: os } = await import('node:os');
    const envPath = path.join(os.homedir(), '.claude', '.env');
    if (!fs.existsSync(envPath))
        return undefined;
    const txt = fs.readFileSync(envPath, 'utf-8');
    const match = txt.match(/^SELEMENE_API_KEY=(\S+)/m);
    if (match) {
        process.env.SELEMENE_API_KEY = match[1];
        return match[1];
    }
    return undefined;
}
//# sourceMappingURL=fetcher.js.map