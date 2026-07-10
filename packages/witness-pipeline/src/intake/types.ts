import type { ReportLevel } from '../modes/types.js';

export type ReportMode = 'kundali' | 'birth-blueprint' | 'integrated-reading' | 'synastry' | string;
export type Gender = 'female' | 'male' | 'nonbinary' | 'other' | 'prefer_not_to_say' | 'unknown';
export type ExternalChartSex = 'female' | 'male' | 'unknown';
export type SubjectRole = 'primary' | 'partner' | 'family_member' | 'friend' | 'business_partner' | 'custom';

export interface NormalizedLocation {
  display_name: string;
  latitude: number;
  longitude: number;
  timezone: string;
  provider: 'manual' | 'nominatim' | 'google-places' | 'mapbox' | 'geonames';
  confidence: 'exact' | 'selected' | 'ambiguous' | 'manual';
}

export interface ReportSubjectInput {
  role: SubjectRole;
  relationship_label?: string;
  name: string;
  gender?: Gender;
  sex_for_external_chart_source?: ExternalChartSex;
  birth_date: string;
  birth_time?: string;
  birth_time_confidence: 'exact' | 'approximate' | 'unknown';
  birth_location_query: string;
  normalized_location?: NormalizedLocation;
}

export interface ReportGenerationRequest {
  report_level: ReportLevel;
  report_mode: ReportMode;
  subjects: ReportSubjectInput[];
  relationship_context?: {
    type: 'family' | 'friends' | 'business-partners' | 'unmarried-partners' | 'married-partners' | 'custom';
    mapping_goal: string;
    sensitivity_level: 'low' | 'medium' | 'high';
  };
  language?: string; // e.g. 'en', 'hi', 'es' — injected into prompts and metadata
  output: { format: 'markdown' | 'docx' | 'pdf' | 'source-pack'; include_rubric: boolean; include_pattern_extraction: boolean };
}

export function isCompleteReportRequest(request: ReportGenerationRequest): boolean {
  return request.subjects.length > 0 && request.subjects.every((subject) => Boolean(subject.normalized_location));
}

// Convenience builders for explicit non-presumptive requests (used in tests + callers)
export function createMotherSonRequest(): ReportGenerationRequest {
  return {
    report_level: 'L2',
    report_mode: 'synastry',
    subjects: [
      { role: 'mother', name: 'Aarav', birth_date: '1970-01-01', birth_time_confidence: 'exact', birth_location_query: 'Bengaluru, India' },
      { role: 'son', name: 'Vikram', birth_date: '1995-01-01', birth_time_confidence: 'exact', birth_location_query: 'Bengaluru, India' },
    ],
    relationship_context: {
      type: 'family',
      mapping_goal: 'understand lineage transmission patterns without outcome prediction',
      sensitivity_level: 'high',
    },
    language: 'en',
    output: { format: 'markdown', include_rubric: true, include_pattern_extraction: true },
  };
}

export function createBusinessDyadRequest(): ReportGenerationRequest {
  return {
    report_level: 'L2',
    report_mode: 'synastry',
    subjects: [
      { role: 'business-partner', name: 'Priya', relationship_label: 'co-founder & CEO', birth_date: '1985-03-01', birth_time_confidence: 'exact', birth_location_query: 'Mumbai, India' },
      { role: 'business-partner', name: 'Rahul', relationship_label: 'co-founder & CTO', birth_date: '1986-07-01', birth_time_confidence: 'exact', birth_location_query: 'Mumbai, India' },
    ],
    relationship_context: {
      type: 'business-partners',
      mapping_goal: 'map decision dynamics and complementary patterns in the partnership',
      sensitivity_level: 'medium',
    },
    language: 'en',
    output: { format: 'markdown', include_rubric: true, include_pattern_extraction: true },
  };
}

export function createFamilyPentaRequest(): ReportGenerationRequest {
  return {
    report_level: 'L3',
    report_mode: 'synastry',
    subjects: [
      { role: 'mother', name: 'Lakshmi' },
      { role: 'father', name: 'Ramesh' },
      { role: 'child1', name: 'Anika' },
      { role: 'child2', name: 'Arjun' },
      { role: 'child3', name: 'Meera' },
    ].map((s) => ({
      ...s,
      birth_date: '1990-01-01',
      birth_time_confidence: 'exact' as const,
      birth_location_query: 'Chennai, India',
    })),
    relationship_context: {
      type: 'family',
      mapping_goal: 'witness family field dynamics and transmission patterns',
      sensitivity_level: 'high',
    },
    language: 'en',
    output: { format: 'markdown', include_rubric: true, include_pattern_extraction: true },
  };
}
