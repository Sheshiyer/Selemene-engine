import type { SectionRubric } from './integrated.js';

export interface AuditSectionInput {
  sectionId: string;
  title: string;
  targetWords: number;
  output: string;
  modelRequested: string;
  modelUsed: string;
  latencyMs: number;
}

const SYSTEM_PATTERNS = [
  /\b(Vedic|Lagna|house|planet|nakshatra|pada|dasha|antardasha|Sade Sati)\b/gi,
  /\b(Human Design|HD|gate|channel|profile|authority|type|center)\b/gi,
  /\b(Gene Keys|Life's Work|Evolution|Vocation|Pearl|Radiance|Purpose)\b/gi,
  /\b(transit|panchanga|tithi|yoga|karana)\b/gi,
  /\b(Vimshottari|mahadasha|pratyantardasha)\b/gi,
];

const LAYERS: Array<[string, RegExp]> = [
  ['vedic', /\b(Vedic|Lagna|nakshatra|house|planet|Rahu|Ketu)\b/i],
  ['vimshottari', /\b(Vimshottari|mahadasha|antardasha|dasha)\b/i],
  ['human-design', /\b(Human Design|HD|gate|channel|profile|authority|center)\b/i],
  ['gene-keys', /\b(Gene Keys|Life's Work|Evolution|Vocation|Pearl|Radiance|Purpose)\b/i],
  ['transits', /\b(transit|Sade Sati|Saturn|Jupiter)\b/i],
  ['panchanga', /\b(panchanga|tithi|karana|yoga)\b/i],
];

const GUARDRAILS: Record<string, RegExp[]> = {
  wealth: [/guarantee/i, /investment advice/i, /will become rich/i, /definitely inherit/i],
  'love-marriage': [/will marry/i, /inevitable/i, /must divorce/i, /spouse will be/i],
  health: [/diagnos/i, /treatment/i, /cure/i, /will develop/i, /you have .*disease/i],
  'family-lineage': [/will have children/i, /infertile/i, /parent will die/i, /child will/i],
};

const SECTION_THRESHOLDS: Record<string, { minFacts: number; minLayers: number }> = {
  opening: { minFacts: 1, minLayers: 2 },
  'convergence-map': { minFacts: 6, minLayers: 4 },
  'vedic-foundation': { minFacts: 12, minLayers: 1 },
  'karmic-architecture': { minFacts: 6, minLayers: 3 },
  'career-dharma': { minFacts: 6, minLayers: 3 },
  wealth: { minFacts: 5, minLayers: 3 },
  'love-marriage': { minFacts: 5, minLayers: 3 },
  health: { minFacts: 4, minLayers: 2 },
  'family-lineage': { minFacts: 4, minLayers: 2 },
  'master-timeline': { minFacts: 8, minLayers: 3 },
  'remedies-practices': { minFacts: 4, minLayers: 3 },
  'final-synthesis': { minFacts: 5, minLayers: 4 },
};

const DEFAULT_THRESHOLDS = { minFacts: 3, minLayers: 3 };

export function auditSectionOutput(input: AuditSectionInput): SectionRubric {
  const words = input.output.trim().split(/\s+/).filter(Boolean).length;
  const ratio = input.targetWords > 0 ? words / input.targetWords : 0;
  const deterministicFactCount = SYSTEM_PATTERNS.reduce((sum, re) => {
    return sum + (input.output.match(re)?.length ?? 0);
  }, 0);
  const integratedLayerCount = LAYERS.filter(([, re]) => re.test(input.output)).length;
  const thresholds = SECTION_THRESHOLDS[input.sectionId] ?? DEFAULT_THRESHOLDS;
  const guardrailViolations = (GUARDRAILS[input.sectionId] ?? [])
    .filter((re) => re.test(input.output))
    .map((re) => re.source);

  return {
    section_id: input.sectionId,
    title: input.title,
    target_words: input.targetWords,
    actual_words: words,
    word_count_fit: wordCountGate(ratio),
    word_count_ratio: Number(ratio.toFixed(3)),
    deterministic_fact_count: deterministicFactCount,
    deterministic_fact_gate: thresholdGate(deterministicFactCount, thresholds.minFacts),
    integrated_layer_count: integratedLayerCount,
    integrated_layering_gate: thresholdGate(integratedLayerCount, thresholds.minLayers),
    guardrail_gate: guardrailViolations.length === 0 ? 'pass' : 'fail',
    guardrail_violations: guardrailViolations,
    model_requested: input.modelRequested,
    model_used: input.modelUsed,
    latency_ms: input.latencyMs,
  };
}

function thresholdGate(value: number, min: number): 'pass' | 'warn' | 'fail' {
  if (value >= min) return 'pass';
  if (value > 0 && value >= Math.ceil(min / 2)) return 'warn';
  return 'fail';
}

function wordCountGate(ratio: number): 'pass' | 'warn' | 'fail' {
  if (ratio >= 0.8 && ratio <= 1.25) return 'pass';
  if (ratio >= 0.65 && ratio <= 1.45) return 'warn';
  return 'fail';
}
