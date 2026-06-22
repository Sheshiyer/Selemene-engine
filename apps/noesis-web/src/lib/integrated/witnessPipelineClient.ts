import type { BirthData } from '@noesis/witness-pipeline';

export interface WitnessFormData {
  name?: string;
  birthDate: string;
  birthTime?: string;
  timezone: string;
  latitude: number;
  longitude: number;
}

export function buildIntegratedReadingPayload(form: WitnessFormData): { birth_data: BirthData } {
  const birth_data: BirthData = {
    date: form.birthDate,
    timezone: form.timezone,
    latitude: form.latitude,
    longitude: form.longitude,
  };
  if (form.birthTime) birth_data.time = form.birthTime;
  if (form.name) birth_data.name = form.name;
  return { birth_data };
}
