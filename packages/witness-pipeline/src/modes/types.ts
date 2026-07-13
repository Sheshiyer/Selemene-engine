// ─── Mode document types ───────────────────────────────────────────────

export type TopologyKey = 'dyad-arc' | 'triad-triangle' | 'pentagon' | 'web-graph';
export type ArchitectureKey = 'linear' | 'hierarchical';
export type RegisterBand = 'l1_l3' | 'l4_l5';
export type ReportLevel = 'L0' | 'L1' | 'L2' | 'L3' | 'L4' | 'L5';

export interface PassSpec {
  id: string;
  title: string;
  target_words: number;
  template: string;
  model?: string;
}

export interface PassTemplateOverride {
  pass_id: string;
  template: string;
}

export interface RegisterVariantSpec {
  target_words?: { min: number; max: number };
  overrides?: PassTemplateOverride[];
}

export interface RegisterVariants {
  l1_l3?: RegisterVariantSpec;
  l4_l5?: RegisterVariantSpec;
}

export interface ModeConfig {
  mode: string;
  subject_count: { min: number; max: number };
  roles: string[];
  target_words: { min: number; max: number };
  architecture: ArchitectureKey;
  pass_plan: PassSpec[];
  engine_overlay_weights: Record<string, number>;
  house_overlay: number[];
  bridge_mandates: string[];
  svg_topology: TopologyKey;
  relationship_types?: string[];
  register_variants?: RegisterVariants;
  report_level?: ReportLevel;
}

export interface LessonsEntry {
  date: string;
  title: string;
  question?: string;
  variants?: string[];
  winner?: string;
  adopted?: string;
  reference?: string;
}

export interface ParsedModeDoc {
  frontmatter: ModeConfig;
  sections: Record<string, string>;
  lessons: LessonsEntry[];
  raw_path: string;
}
