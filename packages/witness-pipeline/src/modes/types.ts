export interface SubjectCount {
  min: number;
  max: number;
}

export interface TargetWords {
  min: number;
  max: number;
}

export interface PassSpec {
  id: string;
  title: string;
  target_words: number;
  template: string;
  model?: string;
}

export interface RegisterVariant {
  target_words?: TargetWords;
  overrides: Array<{ pass_id: string; template: string }>;
}

export interface ModeDocument {
  mode: string;
  subject_count: SubjectCount;
  roles: string[];
  target_words: TargetWords;
  architecture: 'linear' | 'hierarchical';
  pass_plan: PassSpec[];
  engine_overlay_weights: Record<string, number>;
  house_overlay: number[];
  bridge_mandates: string[];
  svg_topology: 'dyad-arc' | 'triad-triangle' | 'pentagon' | 'web-graph';
  register_variants?: {
    l1_l3?: RegisterVariant;
    l4_l5?: RegisterVariant;
  };
  templates: Record<string, string>;
  overlay_rules?: string;
  glossary?: string;
  interactions?: string;
  lessons: LessonEntry[];
}

export interface LessonEntry {
  date: string;
  title: string;
  fields: Record<string, string>;
}
