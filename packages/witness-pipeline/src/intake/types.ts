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
  output: { format: 'markdown' | 'docx' | 'pdf' | 'source-pack'; include_rubric: boolean; include_pattern_extraction: boolean };
}

export function isCompleteReportRequest(request: ReportGenerationRequest): boolean {
  return request.subjects.length > 0 && request.subjects.every((subject) => Boolean(subject.normalized_location));
}
