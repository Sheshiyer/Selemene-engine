// ─── Selemene Engine Fetcher ─────────────────────────────────────────
// Ported from witness-agents/scripts/integratedreading/selemene/fetcher.ts
// Adds dependency-injected fetchImpl for testability.

import { SELEMENE_ENGINE_IDS, type SelemeneEngineId, type SelemeneEngineOutput, type BirthData } from './types.js';
export { SELEMENE_ENGINE_IDS, type SelemeneEngineId, type SelemeneEngineOutput, type BirthData } from './types.js';
import { decodeEngineCapabilityList, type ContractEngineCapabilityList } from '@selemene/engine-sdk';
import { readFileSync } from 'node:fs';

export const SELEMENE_BASE_URL = 'https://selemene.tryambakam.space';

export interface FetchOptions {
  api_key: string;
  base_url?: string;
  timeout_ms?: number;
  engines?: SelemeneEngineId[];
  /** Fixture injection for deterministic tests; production uses the protected route. */
  capabilities?: unknown;
  fetchImpl?: typeof fetch;
}

async function callEngine(engineId: SelemeneEngineId, birthData: BirthData, opts: FetchOptions): Promise<SelemeneEngineOutput> {
  const url = `${opts.base_url ?? SELEMENE_BASE_URL}/api/v1/engines/${encodeURIComponent(engineId)}/calculate`;
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
      return { engine_id: engineId, _error: `HTTP ${res.status}` } as SelemeneEngineOutput;
    }
    const data: unknown = await readJson(res);
    return { ...(data as object), engine_id: engineId } as SelemeneEngineOutput;
  } catch (err: unknown) {
    clearTimeout(t);
    return { engine_id: engineId, _error: err instanceof Error && err.name === 'AbortError' ? 'TIMEOUT' : 'REQUEST_FAILED' } as SelemeneEngineOutput;
  }
}

export async function fetchAllEngines(birthData: BirthData, opts: FetchOptions): Promise<SelemeneEngineOutput[]> {
  if (!opts.api_key?.trim()) throw new Error('SELEMENE_API_KEY is required');
  const canonical = await preflightCapabilities(opts);
  const eligible = eligibleWitnessIds(canonical);
  const requested = opts.engines ?? eligible;
  for (const engineId of requested) {
    if (!eligible.includes(engineId)) {
      throw new Error(`engine ${engineId} is not an eligible Witness capability`);
    }
  }
  const engines = requested;
  const results = await Promise.all(engines.map((eId) => callEngine(eId, birthData, opts)));
  return results;
}

const PUBLIC_MIRROR_IDS = new Set([
  'biofield', 'biorhythm', 'enneagram', 'face-reading', 'gene-keys', 'human-design',
  'i-ching', 'nadabrahman', 'numerology', 'panchanga', 'raaga', 'sacred-geometry',
  'sigil-forge', 'tarot', 'transits', 'vedic-clock', 'vimshottari',
]);

function loadCanonicalFixture(): unknown {
  const path = new URL('../../../../contracts/v1/fixtures/engine-capability-list.json', import.meta.url);
  return JSON.parse(readFileSync(path, 'utf8')) as unknown;
}

function eligibleWitnessIds(list: ContractEngineCapabilityList): SelemeneEngineId[] {
  return list.capabilities
    .filter((row) => PUBLIC_MIRROR_IDS.has(row.engine_id) && row.operations?.calculate === 'supported' && row.operations.witness_eligible)
    .map((row) => row.engine_id)
    .filter((id): id is SelemeneEngineId => (SELEMENE_ENGINE_IDS as readonly string[]).includes(id))
    .sort((a, b) => (SELEMENE_ENGINE_IDS as readonly string[]).indexOf(a) - (SELEMENE_ENGINE_IDS as readonly string[]).indexOf(b));
}

async function readJson(res: Response): Promise<unknown> {
  if (typeof res.text === 'function') {
    const text = await res.text();
    if (!text) throw new Error('empty response');
    return JSON.parse(text) as unknown;
  }
  return res.json();
}

async function preflightCapabilities(opts: FetchOptions): Promise<ContractEngineCapabilityList> {
  const fixture = decodeEngineCapabilityList(loadCanonicalFixture());
  const payload = opts.capabilities ?? await fetchCapabilityEnvelope(opts);
  const list = decodeEngineCapabilityList(payload);
  if (list.count !== fixture.count || list.public_mirror_count !== fixture.public_mirror_count ||
      list.capabilities.map((row) => row.engine_id).join(',') !== fixture.capabilities.map((row) => row.engine_id).join(',')) {
    throw new Error('canonical capability preflight mismatch');
  }
  return list;
}

async function fetchCapabilityEnvelope(opts: FetchOptions): Promise<unknown> {
  const url = `${opts.base_url ?? SELEMENE_BASE_URL}/api/v1/engines/capabilities`;
  const fetchImpl = opts.fetchImpl ?? fetch;
  const res = await fetchImpl(url, {
    method: 'GET',
    headers: { 'X-API-Key': opts.api_key },
  });
  if (!res.ok) throw new Error('capability preflight failed');
  return readJson(res);
}

export async function loadSelemeneKey(): Promise<string | undefined> {
  if (process.env.SELEMENE_API_KEY) return process.env.SELEMENE_API_KEY;
  const { default: fs } = await import('node:fs');
  const { default: path } = await import('node:path');
  const { default: os } = await import('node:os');
  const envPath = path.join(os.homedir(), '.claude', '.env');
  if (!fs.existsSync(envPath)) return undefined;
  const txt = fs.readFileSync(envPath, 'utf-8');
  const match = txt.match(/^SELEMENE_API_KEY=(\S+)/m);
  if (match) {
    process.env.SELEMENE_API_KEY = match[1];
    return match[1];
  }
  return undefined;
}
