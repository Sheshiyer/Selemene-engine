export type PatternKind = 'convergence' | 'tension' | 'guardrail-safe-framing' | 'section-structure' | 'remedy-pattern';

export interface ExtractedPattern {
  id: string;
  text: string;
  kind: PatternKind;
  source_section_id: string;
  source_rubric_score: number;
  metadata: {
    mode: string;
    report_level: string;
    systems: string[];
    source: 'post-report-extraction';
    version: string;
  };
}

export interface RetrievedPattern {
  text: string;
  score?: number;
  metadata?: Record<string, unknown>;
}

export interface RetrievalFilters {
  mode?: string;
  report_level?: string;
  kind?: string;
  version?: string;
  systems?: string[];
}

export interface PatternVectorStoreResult {
  upserted: number;
  skipped: number;
}