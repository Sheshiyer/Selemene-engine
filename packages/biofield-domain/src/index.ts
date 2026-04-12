export const BIOFIELD_ENGINE_ID = "biofield-capture";

export const BIOFIELD_SESSION_STATUSES = [
  "active",
  "closed",
  "abandoned",
] as const;

export const BIOFIELD_CAPTURE_STATES = [
  "requested",
  "uploaded",
  "analyzed",
  "persisted",
  "rejected",
  "reprocessed",
] as const;

export type BiofieldSessionStatus = (typeof BIOFIELD_SESSION_STATUSES)[number];
export type BiofieldCaptureState = (typeof BIOFIELD_CAPTURE_STATES)[number];

export interface BiofieldSession {
  id: string;
  status: BiofieldSessionStatus;
  started_at: string;
  closed_at: string | null;
  client_device_id?: string | null;
  viewer_version?: string | null;
}

export interface BiofieldSessionContext {
  platform?: string;
  viewport?: {
    width: number;
    height: number;
  };
}

export interface CreateBiofieldSessionRequest {
  client_device_id?: string;
  viewer_version?: string;
  context?: BiofieldSessionContext;
}

export interface CloseBiofieldSessionRequest {
  reason?: string;
}

export interface EnergyBands {
  low: number;
  medium: number;
  high: number;
  total: number;
}

export interface QualityAssessment {
  sharpness: number;
  contrast: number;
  noise_level: number;
  exposure: number;
  sufficient_quality: boolean;
}

export interface BiofieldMetrics {
  light_quanta_density: number;
  normalized_area: number;
  average_intensity: number;
  inner_noise: number;
  energy_analysis: EnergyBands;
  entropy_form_coefficient: number;
  fractal_dimension: number;
  correlation_dimension: number;
  body_symmetry: number;
  contour_complexity: number;
  pattern_regularity: number;
}

export interface BiofieldArtifactSummary {
  id?: string;
  kind: string;
  mime_type: string;
  storage_path?: string;
  byte_size?: number;
}

export interface BiofieldReadingSummary {
  reading_id: string;
  session_id: string;
  engine_id: typeof BIOFIELD_ENGINE_ID;
  created_at: string;
  quality: Pick<QualityAssessment, "sufficient_quality">;
  artifact: BiofieldArtifactSummary;
}

export interface ListBiofieldReadingsResponse {
  items: BiofieldReadingSummary[];
  limit: number;
  offset: number;
}

export interface BiofieldReadingDetail {
  reading_id: string;
  session_id: string;
  engine_id: typeof BIOFIELD_ENGINE_ID;
  created_at: string;
  input: Record<string, unknown>;
  result: Record<string, unknown>;
  quality: QualityAssessment;
  artifacts: BiofieldArtifactSummary[];
}

export interface BiofieldCaptureResult {
  reading_id: string;
  session_id: string;
  analysis_version: string;
  metrics: BiofieldMetrics;
  quality_assessment: QualityAssessment;
  artifacts: BiofieldArtifactSummary[];
}


export interface ReprocessBiofieldReadingRequest {
  algorithms?: string[];
  options?: Record<string, unknown>;
}

export interface BiofieldReprocessResult extends BiofieldCaptureResult {
  source_reading_id: string;
}

export interface CreateBiofieldBaselineRequest {
  name: string;
  notes?: string;
  reading_ids: string[];
}

export interface BiofieldBaselineSummary {
  baseline_id: string;
  name: string;
  notes?: string | null;
  reading_count: number;
  created_at: string;
  updated_at: string;
}

export interface ListBiofieldBaselinesResponse {
  items: BiofieldBaselineSummary[];
}
