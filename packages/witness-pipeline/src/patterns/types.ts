import type { ReportLevel } from '../modes/types.js';

export type PatternKind = 'convergence' | 'tension' | 'guardrail-safe-framing' | 'section-structure' | 'remedy-pattern';

export interface ExtractedPattern {
  id: string;
  text: string;
  kind: PatternKind;
  source_section_id: string;
  source_rubric_score: number;
  metadata: {
    mode: string;
    report_level: ReportLevel;
    systems: string[];
    source: 'post-report-extraction';
    version: string;
  };
}
