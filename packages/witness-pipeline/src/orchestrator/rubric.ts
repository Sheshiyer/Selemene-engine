import type { SectionRubric } from './integrated.js';

export interface AuditSectionInput {
  sectionId: string;
  title: string;
  targetWords: number;
  output: string;
  modelRequested: string;
  modelUsed: string;
  latencyMs: number;
  engineResults?: any[];
}

const PLACEHOLDER_RE = /\[[^\]]+\bfrom engine results\b[^\]]*\]/gi;

function hasPlaceholder(output: string): boolean {
  return PLACEHOLDER_RE.test(output);
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

const FIDELITY_THRESHOLDS: Record<string, { pass: number; warn: number }> = {
  opening: { pass: 0.15, warn: 0.05 },
  'convergence-map': { pass: 0.35, warn: 0.2 },
  'vedic-foundation': { pass: 0.55, warn: 0.3 },
  'karmic-architecture': { pass: 0.35, warn: 0.2 },
  'career-dharma': { pass: 0.3, warn: 0.15 },
  wealth: { pass: 0.3, warn: 0.15 },
  'love-marriage': { pass: 0.3, warn: 0.15 },
  health: { pass: 0.3, warn: 0.15 },
  'family-lineage': { pass: 0.3, warn: 0.15 },
  'master-timeline': { pass: 0.4, warn: 0.2 },
  'remedies-practices': { pass: 0.3, warn: 0.15 },
  'final-synthesis': { pass: 0.35, warn: 0.2 },
};

const FIDELITY_ALWAYS_PASS = new Set(['opening', 'final-synthesis']);

function scalarLabel(value: unknown): string | undefined {
  if (value === undefined || value === null) return undefined;
  if (typeof value === 'string') {
    const trimmed = value.trim();
    return trimmed.length > 0 ? trimmed : undefined;
  }
  if (typeof value === 'number' || typeof value === 'boolean') return String(value);
  if (typeof value === 'object') {
    // Structured value objects common in Selemene engine results.
    const obj = value as Record<string, unknown>;
    return (
      scalarLabel(obj.value) ??
      scalarLabel(obj.key_number) ??
      scalarLabel(obj.key) ??
      scalarLabel(obj.name) ??
      scalarLabel(obj.planet) ??
      scalarLabel(obj.sign) ??
      scalarLabel(obj.gate) ??
      scalarLabel(obj.channel) ??
      scalarLabel(obj.event) ??
      scalarLabel(obj.year) ??
      scalarLabel(obj.practice) ??
      scalarLabel(obj.recommendation) ??
      undefined
    );
  }
  return undefined;
}

function addFact(facts: Set<string>, prefix: string, value: unknown): void {
  const token = scalarLabel(value);
  if (!token) return;
  facts.add(`${prefix}:${token}`.toLowerCase());
}

function addArrayFacts(facts: Set<string>, prefix: string, arr: unknown): void {
  if (!Array.isArray(arr)) return;
  for (const item of arr) {
    if (item === null || item === undefined) continue;
    if (typeof item === 'string') {
      addFact(facts, prefix, item);
    } else if (typeof item === 'object') {
      // For gene-key / numerology / chakra style objects, extract multiple identifying tokens.
      const obj = item as Record<string, unknown>;
      const primary =
        scalarLabel(obj.key) ??
        scalarLabel(obj.name) ??
        scalarLabel(obj.planet) ??
        scalarLabel(obj.sign) ??
        scalarLabel(obj.gate) ??
        scalarLabel(obj.channel) ??
        scalarLabel(obj.chakra) ??
        scalarLabel(obj.event) ??
        scalarLabel(obj.year) ??
        scalarLabel(obj.practice) ??
        scalarLabel(obj.recommendation);
      if (primary) addFact(facts, prefix, primary);
      // Secondary tokens increase recall without bloating the fact set too much.
      if (obj.gift) addFact(facts, prefix, obj.gift);
      if (obj.shadow) addFact(facts, prefix, obj.shadow);
      if (obj.siddhi) addFact(facts, prefix, obj.siddhi);
      if (obj.meaning) addFact(facts, prefix, obj.meaning);
    }
  }
}

function extractGenericFacts(engines: any[]): Set<string> {
  const facts = new Set<string>();
  for (const e of engines) {
    const r = e.result || {};
    // Fundamentals — expected in every pass
    if (r.nakshatra_name) addFact(facts, 'nakshatra', r.nakshatra_name);
    if (r.tithi_name) addFact(facts, 'tithi', r.tithi_name);
    if (r.current_period?.mahadasha?.planet) {
      addFact(facts, 'dasha', r.current_period.mahadasha.planet);
    }
    if (r.current_period?.antardasha?.planet) {
      addFact(facts, 'antardasha', r.current_period.antardasha.planet);
    }
    // HD identifiers — expected in most passes
    if (r.hd_type) addFact(facts, 'hdtype', r.hd_type);
    if (r.profile) addFact(facts, 'profile', r.profile);
    if (r.authority) addFact(facts, 'auth', r.authority);
    // Vedic — expected in vedic-focused passes
    if (r.lagna_sign) addFact(facts, 'lagna', r.lagna_sign);
    if (r.lagna) addFact(facts, 'lagna', r.lagna);
    if (r.vara_name) addFact(facts, 'vara', r.vara_name);
    if (r.yoga_name) addFact(facts, 'yoga', r.yoga_name);
  }
  return facts;
}

export function extractSectionFacts(engines: any[], sectionId: string): Set<string> {
  // Generic chart facts are relevant for the broad overview and Vedic passes.
  // Thematic passes get their own focused fact set so the fidelity score measures
  // whether the LLM actually grounded the section in its specific engine data.
  const usesGenericFacts = new Set(['opening', 'convergence-map', 'vedic-foundation', 'final-synthesis']);
  const facts = usesGenericFacts.has(sectionId) ? extractGenericFacts(engines) : new Set<string>();

  // Helpers that read across the union of engine results.
  const biofieldResult = engines.find((e) => e.engine_id === 'biofield' || e.result?.biofield)?.result ?? {};
  const biorhythmResult = engines.find((e) => e.engine_id === 'biorhythm' || e.result?.biorhythm)?.result ?? {};
  const numerologyResult = engines.find((e) => e.engine_id === 'numerology')?.result ?? {};
  const humanDesignResult = engines.find((e) => e.engine_id === 'human-design')?.result ?? {};
  const geneKeysResult = engines.find((e) => e.engine_id === 'gene-keys')?.result ?? {};
  const vimshottariResult = engines.find((e) => e.engine_id === 'vimshottari')?.result ?? {};
  const transitsResult = engines.find((e) => e.engine_id === 'transits')?.result ?? {};
  const nadabrahmanResult = engines.find((e) => e.engine_id === 'nadabrahman')?.result ?? {};
  const panchangaResult = engines.find((e) => e.engine_id === 'panchanga')?.result ?? {};

  const allResults = engines.map((e) => e.result ?? {});
  const anyResult = allResults.reduce((acc, r) => ({ ...acc, ...r }), {});

  switch (sectionId) {
    case 'health': {
      const areas = biofieldResult?.areas_of_attention ?? anyResult?.areas_of_attention;
      addArrayFacts(facts, 'biofield', areas);
      const chakras = biofieldResult?.chakra_readings ?? anyResult?.chakra_readings;
      if (Array.isArray(chakras)) {
        for (const c of chakras) {
          const name = typeof c === 'string' ? c : c?.chakra ?? c?.name;
          addFact(facts, 'chakra', name);
        }
      }
      const br = biorhythmResult ?? anyResult;
      addFact(facts, 'biorhythm', br?.physical);
      addFact(facts, 'biorhythm', br?.emotional);
      addFact(facts, 'biorhythm', br?.spiritual);
      addFact(facts, 'biorhythm', br?.overall_energy);
      break;
    }
    case 'family-lineage': {
      addFact(facts, 'numpath', numerologyResult?.life_path ?? anyResult?.life_path);
      addFact(facts, 'numsoul', numerologyResult?.soul_urge ?? anyResult?.soul_urge);
      addFact(facts, 'numexpr', numerologyResult?.expression ?? anyResult?.expression);
      addFact(facts, 'profile', humanDesignResult?.profile ?? anyResult?.profile);
      addArrayFacts(facts, 'genekey', geneKeysResult?.active_keys ?? anyResult?.active_keys);
      break;
    }
    case 'love-marriage': {
      addFact(facts, 'profile', humanDesignResult?.profile ?? anyResult?.profile);
      addFact(facts, 'auth', humanDesignResult?.authority ?? anyResult?.authority);
      addArrayFacts(facts, 'center', humanDesignResult?.defined_centers ?? anyResult?.defined_centers);
      addArrayFacts(facts, 'genekey', geneKeysResult?.active_keys ?? anyResult?.active_keys);
      if (transitsResult?.venus_sign) addFact(facts, 'venus', transitsResult.venus_sign);
      if (transitsResult?.mars_sign) addFact(facts, 'mars', transitsResult.mars_sign);
      if (transitsResult?.jupiter_sign) addFact(facts, 'jupiter', transitsResult.jupiter_sign);
      break;
    }
    case 'wealth': {
      addFact(facts, 'numexpr', numerologyResult?.expression ?? anyResult?.expression);
      addFact(facts, 'numpath', numerologyResult?.life_path ?? anyResult?.life_path);
      addFact(facts, 'hdtype', humanDesignResult?.hd_type ?? anyResult?.hd_type);
      if (vimshottariResult?.current_period?.mahadasha?.planet) {
        addFact(facts, 'dasha', vimshottariResult.current_period.mahadasha.planet);
      }
      if (vimshottariResult?.current_period?.antardasha?.planet) {
        addFact(facts, 'antardasha', vimshottariResult.current_period.antardasha.planet);
      }
      addArrayFacts(facts, 'seq', geneKeysResult?.activation_sequence ?? anyResult?.activation_sequence);
      break;
    }
    case 'master-timeline': {
      if (vimshottariResult?.current_period?.mahadasha?.planet) {
        addFact(facts, 'dasha', vimshottariResult.current_period.mahadasha.planet);
      }
      if (vimshottariResult?.current_period?.antardasha?.planet) {
        addFact(facts, 'antardasha', vimshottariResult.current_period.antardasha.planet);
      }
      if (vimshottariResult?.current_period?.pratyantardasha?.planet) {
        addFact(facts, 'pratyantardasha', vimshottariResult.current_period.pratyantardasha.planet);
      }
      const transitions = vimshottariResult?.upcoming_transitions ?? anyResult?.upcoming_transitions;
      if (Array.isArray(transitions)) {
        for (const t of transitions) {
          if (typeof t === 'string') {
            addFact(facts, 'transition', t);
            continue;
          }
          // Prefer structured planet endpoints when available (real engine data).
          // Fall back to the human-readable event/year label for test/legacy fixtures.
          addFact(facts, 'fromplanet', t?.from_planet);
          addFact(facts, 'toplanet', t?.to_planet);
          addFact(facts, 'transitiontype', t?.type);
          addFact(facts, 'transition', t?.event ?? t?.planet ?? t?.year);
        }
      }
      if (transitsResult?.sade_sati) {
        const ss = transitsResult.sade_sati;
        if (typeof ss === 'string' || typeof ss === 'boolean' || typeof ss === 'number') {
          addFact(facts, 'sadesati', ss);
        } else if (typeof ss === 'object') {
          addFact(facts, 'sadesati', ss.is_active);
          if (ss.is_active && ss.phase) addFact(facts, 'sadesatiphase', ss.phase);
        }
      }
      const retro = transitsResult?.retrograde_planets ?? anyResult?.retrograde_planets;
      addArrayFacts(facts, 'retro', retro);
      break;
    }
    case 'remedies-practices': {
      const areas = biofieldResult?.areas_of_attention ?? anyResult?.areas_of_attention;
      addArrayFacts(facts, 'biofield', areas);
      const recs = nadabrahmanResult?.recommendations ?? anyResult?.recommendations;
      if (Array.isArray(recs)) {
        for (const r of recs) {
          const label = typeof r === 'string' ? r : r?.practice ?? r?.name ?? r?.recommendation;
          addFact(facts, 'nada', label);
        }
      }
      break;
    }
    case 'karmic-architecture': {
      addArrayFacts(facts, 'genekey', geneKeysResult?.active_keys ?? anyResult?.active_keys);
      const channels = humanDesignResult?.active_channels ?? anyResult?.active_channels;
      if (Array.isArray(channels)) {
        for (const ch of channels) {
          const label =
            typeof ch === 'string'
              ? ch
              : ch?.channel ?? ch?.name ?? (ch?.gate1 && ch?.gate2 ? `${ch.gate1}-${ch.gate2}` : undefined);
          addFact(facts, 'channel', label);
        }
      }
      break;
    }
    case 'career-dharma': {
      addFact(facts, 'numexpr', numerologyResult?.expression ?? anyResult?.expression);
      addFact(facts, 'hdtype', humanDesignResult?.hd_type ?? anyResult?.hd_type);
      addFact(facts, 'profile', humanDesignResult?.profile ?? anyResult?.profile);
      break;
    }
    case 'vedic-foundation':
    case 'convergence-map': {
      // Keep broad facts already extracted; add lagna/gate activations if present.
      const design = humanDesignResult?.design_activations ?? anyResult?.design_activations;
      const personality = humanDesignResult?.personality_activations ?? anyResult?.personality_activations;
      if (Array.isArray(design)) {
        for (const d of design) addFact(facts, 'gate', d?.gate ?? d?.number ?? d);
      }
      if (Array.isArray(personality)) {
        for (const p of personality) addFact(facts, 'gate', p?.gate ?? p?.number ?? p);
      }
      if (panchangaResult?.karana_name) addFact(facts, 'karana', panchangaResult.karana_name);
      if (panchangaResult?.sun_sign) addFact(facts, 'sunsign', panchangaResult.sun_sign);
      if (panchangaResult?.moon_sign) addFact(facts, 'moonsign', panchangaResult.moon_sign);
      break;
    }
  }

  return facts;
}

function computeFidelity(output: string, engineFacts: Set<string>): { score: number; details: string[] } {
  if (engineFacts.size === 0) return { score: 0, details: ['no engine facts provided'] };
  const lower = output.toLowerCase();
  let hits = 0;
  const details: string[] = [];
  for (const f of engineFacts) {
    const token = f.split(':')[1] || f;
    if (lower.includes(token)) { hits++; details.push(`hit:${f}`); }
  }
  return { score: hits / engineFacts.size, details };
}

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

  const engineFacts = extractSectionFacts(input.engineResults || [], input.sectionId);
  const fid = computeFidelity(input.output, engineFacts);

  const fThresh = FIDELITY_THRESHOLDS[input.sectionId] ?? { pass: 0.35, warn: 0.2 };
  const fidelityGate = FIDELITY_ALWAYS_PASS.has(input.sectionId)
    ? 'pass'
    : fid.score >= fThresh.pass ? 'pass' : fid.score >= fThresh.warn ? 'warn' : 'fail';

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
    placeholder_gate: hasPlaceholder(input.output) ? 'fail' : 'pass',
    model_requested: input.modelRequested,
    model_used: input.modelUsed,
    latency_ms: input.latencyMs,
    chart_fidelity_score: engineFacts.size > 0 ? Number(fid.score.toFixed(3)) : undefined,
    chart_fidelity_details: engineFacts.size > 0 ? fid.details : undefined,
    chart_fidelity_gate: engineFacts.size > 0 ? fidelityGate : 'warn',
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
