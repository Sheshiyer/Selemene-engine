import { fetchAllEngines, loadSelemeneKey, type BirthData, type SelemeneEngineId } from '@noesis/witness-pipeline';
import type { Subject } from '../types.js';

export interface SelemeneOptions {
  baseUrl?: string;
  apiKey?: string;
}

export class SelemeneApiError extends Error {
  constructor(message: string) {
    super(message);
    this.name = 'SelemeneApiError';
  }
}

export async function fetchEngineResult(subject: Subject, engineId: SelemeneEngineId, opts: SelemeneOptions = {}): Promise<unknown> {
  const apiKey = opts.apiKey ?? (await loadSelemeneKey());
  if (!apiKey) {
    throw new Error('SELEMENE_API_KEY not found. Set the SELEMENE_API_KEY environment variable.');
  }

  const birthData: BirthData = {
    date: subject.birth.date,
    time: subject.birth.time,
    timezone: subject.birth.timezone,
    latitude: subject.birth.location.latitude,
    longitude: subject.birth.location.longitude,
    name: subject.name,
  };

  const results = await fetchAllEngines(birthData, {
    api_key: apiKey,
    base_url: opts.baseUrl,
    engines: [engineId],
  });

  if (results.length !== 1) {
    throw new SelemeneApiError(`Expected exactly one result for ${engineId}, got ${results.length}`);
  }

  const [result] = results;

  if (result._error) {
    throw new SelemeneApiError(`Selemene ${engineId} failed: ${result._error}`);
  }

  return result.result;
}
