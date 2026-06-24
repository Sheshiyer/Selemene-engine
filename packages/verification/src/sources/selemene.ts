import { fetchAllEngines, loadSelemeneKey, type BirthData, type SelemeneEngineId } from '@noesis/witness-pipeline';
import type { Subject } from '../types.js';

export interface SelemeneOptions {
  baseUrl?: string;
  apiKey?: string;
}

export async function fetchEngineResult(subject: Subject, engineId: SelemeneEngineId, opts: SelemeneOptions = {}): Promise<unknown> {
  const apiKey = opts.apiKey ?? (await loadSelemeneKey());
  if (!apiKey) {
    throw new Error('SELEMENE_API_KEY not found. Set env var or add to ~/.claude/.env');
  }

  const birthData: BirthData = {
    date: subject.birth.date,
    time: subject.birth.time,
    timezone: subject.birth.timezone,
    latitude: subject.birth.location.latitude,
    longitude: subject.birth.location.longitude,
    name: subject.name,
  };

  const [result] = await fetchAllEngines(birthData, {
    api_key: apiKey,
    base_url: opts.baseUrl,
    engines: [engineId],
  });

  if (result._error) {
    throw new Error(`Selemene ${engineId} failed: ${result._error}`);
  }

  return result.result;
}
