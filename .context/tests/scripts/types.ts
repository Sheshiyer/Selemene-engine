export interface TestConfig {
  model: string;
  temperature: number;
  max_tokens: number;
  runs_per_test: number;
  groups: {
    control: GroupConfig;
    treatment: GroupConfig;
  };
  scoring: {
    dimensions: string[];
    scale: { min: number; max: number };
  };
  analysis: {
    significance_level: number;
    effect_size_thresholds: {
      small: number;
      medium: number;
      large: number;
    };
  };
}

export interface GroupConfig {
  name: string;
  context_path: string;
}

export interface TestCase {
  id: string;
  category: 'simple' | 'cross-domain' | 'ambiguous' | 'security-critical' | 'refactoring';
  complexity: 'low' | 'medium' | 'high';
  domains: string[];
  prompt: string;
  expectedBehaviors: string[];
}

export interface TestRun {
  id: string;
  timestamp: string;
  case_id: string;
  group: 'control' | 'treatment';
  run_number: number;
  output: string;
  tokens_used: number;
}

export interface Score {
  run_id: string;
  dimension: string;
  score: number;
  notes: string;
  deductions: Array<{ reason: string; amount: number }>;
}

export interface TestResult {
  case_id: string;
  group: 'control' | 'treatment';
  scores: {
    pattern_adherence: number[];
    completeness: number[];
    correctness: number[];
    security: number[];
    consistency: number[];
  };
  average_scores: {
    pattern_adherence: number;
    completeness: number;
    correctness: number;
    security: number;
    consistency: number;
    total: number;
  };
}

export interface AnalysisResult {
  case_id: string;
  control_mean: number;
  treatment_mean: number;
  difference: number;
  p_value: number;
  effect_size: number;
  significant: boolean;
  effect_interpretation: 'small' | 'medium' | 'large' | 'negligible';
}

export interface SuiteReport {
  timestamp: string;
  config: TestConfig;
  results: TestResult[];
  analysis: AnalysisResult[];
  summary: {
    total_tests: number;
    significant_improvements: number;
    control_average: number;
    treatment_average: number;
    overall_effect_size: number;
  };
}
