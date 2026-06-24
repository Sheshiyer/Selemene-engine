import type { SelemeneEngineId } from '@noesis/witness-pipeline';

export interface Location {
  place: string;
  latitude: number;
  longitude: number;
}

export interface Subject {
  id: string;
  name: string;
  birth: {
    date: string;
    time: string;
    timezone: string;
    location: Location;
  };
  ayanamsa?: string;
  added: string;
  source?: string;
}

export type Severity = 'P0' | 'P1' | 'P2' | 'P3' | 'P4';

export interface GoldenField {
  expected: unknown;
  weight: number;
  severity?: Severity;
  notes?: string;
}

export interface GoldenFile {
  subject: string;
  engine: SelemeneEngineId;
  source: string;
  captured: string;
  minAccuracy?: number;
  fields: Record<string, GoldenField>;
}

export interface FieldOutcome {
  pass: boolean;
  expected: unknown;
  actual: unknown;
  weight: number;
}

export interface VerificationResult {
  subject: string;
  engine: SelemeneEngineId;
  accuracy: number;
  fields: Record<string, FieldOutcome>;
  missingFields: string[];
  error?: string;
}
