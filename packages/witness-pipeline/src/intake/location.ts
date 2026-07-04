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
