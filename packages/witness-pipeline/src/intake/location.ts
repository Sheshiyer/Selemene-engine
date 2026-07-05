import type { NormalizedLocation } from './types.js';

export interface ManualLocationInput {
  displayName: string;
  latitude: string | number;
  longitude: string | number;
  timezone: string;
}

export function normalizeManualLocation(input: ManualLocationInput): NormalizedLocation {
  const latitude = Number(input.latitude);
  const longitude = Number(input.longitude);
  if (!Number.isFinite(latitude) || latitude < -90 || latitude > 90) throw new Error('Invalid latitude');
  if (!Number.isFinite(longitude) || longitude < -180 || longitude > 180) throw new Error('Invalid longitude');
  if (!input.timezone.trim()) throw new Error('Timezone is required');
  return {
    display_name: input.displayName.trim(),
    latitude,
    longitude,
    timezone: input.timezone.trim(),
    provider: 'manual',
    confidence: 'manual',
  };
}

export interface LocationCandidate {
  display_name: string;
  latitude: number;
  longitude: number;
  provider: 'nominatim' | 'google-places' | 'mapbox' | 'geonames';
}

export interface SearchBirthplaceOptions {
  limit?: number;
  privacyMode?: boolean;
  fetchImpl?: typeof fetch;
}

const NOMINATIM_URL = 'https://nominatim.openstreetmap.org/search';
const NOMINATIM_USER_AGENT = 'selemene-witness-pipeline/1.0 (report-intake; +https://github.com/anomalyco/opencode)';
let lastNominatimCallMs = 0;

function delay(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

function isValidIANATimezone(tz: string): boolean {
  if (!tz || typeof tz !== 'string') return false;
  try {
    new Intl.DateTimeFormat('en-US', { timeZone: tz });
    return true;
  } catch {
    return false;
  }
}

export async function searchBirthplace(
  query: string,
  opts: SearchBirthplaceOptions = {}
): Promise<LocationCandidate[]> {
  if (opts.privacyMode) {
    return [];
  }

  const q = (query || '').trim();
  if (!q) return [];

  const now = Date.now();
  const since = now - lastNominatimCallMs;
  if (since < 1100) {
    await delay(1100 - since);
  }

  const limit = Math.max(1, Math.min(10, opts.limit ?? 5));
  const url = new URL(NOMINATIM_URL);
  url.searchParams.set('q', q);
  url.searchParams.set('format', 'json');
  url.searchParams.set('limit', String(limit));

  const doFetch = opts.fetchImpl ?? (globalThis.fetch as typeof fetch | undefined);
  if (!doFetch) {
    throw new Error('No fetch implementation available for geocoding');
  }

  let resp: Response;
  try {
    resp = await doFetch(url.toString(), {
      method: 'GET',
      headers: {
        'User-Agent': NOMINATIM_USER_AGENT,
        Accept: 'application/json',
      },
    });
  } catch {
    lastNominatimCallMs = Date.now();
    return [];
  }

  lastNominatimCallMs = Date.now();

  if (!resp.ok) {
    return [];
  }

  let items: unknown;
  try {
    items = await resp.json();
  } catch {
    return [];
  }

  if (!Array.isArray(items)) return [];

  const out: LocationCandidate[] = [];
  for (const item of items) {
    const lat = parseFloat(item?.lat);
    const lon = parseFloat(item?.lon);
    if (!Number.isFinite(lat) || !Number.isFinite(lon)) continue;
    const name = (item?.display_name || q).toString();
    out.push({
      display_name: name,
      latitude: lat,
      longitude: lon,
      provider: 'nominatim',
    });
  }
  return out;
}

export interface ConfirmLocationInput {
  candidate?: LocationCandidate;
  manual?: ManualLocationInput;
  timezone?: string;
}

export function confirmNormalizedLocation(input: ConfirmLocationInput): NormalizedLocation {
  if (input.manual) {
    return normalizeManualLocation(input.manual);
  }

  if (!input.candidate) {
    throw new Error('confirmNormalizedLocation requires either candidate or manual');
  }

  const tz = (input.timezone || '').trim();
  if (!tz) {
    throw new Error('Timezone is required to confirm a location');
  }
  if (!isValidIANATimezone(tz)) {
    throw new Error(`Invalid IANA timezone: ${tz}`);
  }

  return {
    display_name: input.candidate.display_name,
    latitude: input.candidate.latitude,
    longitude: input.candidate.longitude,
    timezone: tz,
    provider: input.candidate.provider,
    confidence: 'selected',
  };
}

export { isValidIANATimezone };
