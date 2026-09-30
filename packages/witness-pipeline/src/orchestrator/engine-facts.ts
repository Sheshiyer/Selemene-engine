// ─── Engine Facts Block ───────────────────────────────────────────────
// Renders the deterministic engine results for every subject into a compact,
// prompt-safe block so each pass is grounded in chart facts rather than names.
// The orchestrator substitutes {{engine_facts}} in a pass template, or appends
// the block when the template declares no placeholder.

import type { SelemeneEngineOutput } from '../selemene/types.js';

export interface EngineFactsInput {
  subjectNames: string[];
  subjectRoles?: Array<{ role: string; label?: string; name: string }>;
  engineResultsBySubject: SelemeneEngineOutput[][];
  /** Max characters of raw JSON kept for engines without a dedicated summariser. */
  fallbackChars?: number;
}

type AnyRecord = Record<string, any>;

function asRecord(v: unknown): AnyRecord {
  return v && typeof v === 'object' && !Array.isArray(v) ? (v as AnyRecord) : {};
}

function fmtDate(iso: unknown): string {
  return typeof iso === 'string' ? iso.slice(0, 10) : '';
}

function summarisePanchanga(r: AnyRecord): string[] {
  const out: string[] = [];
  if (r.tithi_name) out.push(`Tithi ${r.tithi_name}`);
  if (r.nakshatra_name) out.push(`Nakshatra ${r.nakshatra_name}`);
  if (r.vara_name) out.push(`Vara ${r.vara_name}`);
  if (r.yoga_name) out.push(`Yoga ${r.yoga_name}`);
  if (r.karana_name) out.push(`Karana ${r.karana_name}`);
  return out;
}

function summariseVimshottari(r: AnyRecord): string[] {
  const out: string[] = [];
  const bn = asRecord(r.birth_nakshatra);
  if (bn.name) out.push(`Birth nakshatra ${bn.name}${bn.number ? ` (#${bn.number})` : ''}`);
  const cp = asRecord(r.current_period);
  for (const level of ['mahadasha', 'antardasha', 'pratyantardasha']) {
    const p = asRecord(cp[level]);
    if (p.planet) out.push(`${level} ${p.planet}${p.start ? ` ${fmtDate(p.start)}→${fmtDate(p.end)}` : ''}`);
  }
  const ups = Array.isArray(r.upcoming_transitions) ? r.upcoming_transitions.slice(0, 2) : [];
  for (const u of ups) {
    const t = asRecord(u);
    const label = t.planet ?? t.to ?? t.name;
    if (label) out.push(`upcoming ${t.level ?? 'transition'} ${label}${t.start ? ` ${fmtDate(t.start)}` : t.date ? ` ${fmtDate(t.date)}` : ''}`);
  }
  return out;
}

function summariseHumanDesign(r: AnyRecord): string[] {
  const out: string[] = [];
  if (r.hd_type) out.push(`Type ${r.hd_type}`);
  if (r.profile) out.push(`Profile ${r.profile}`);
  if (r.authority) out.push(`Authority ${r.authority}`);
  if (r.definition) out.push(`Definition ${r.definition}`);
  if (Array.isArray(r.defined_centers) && r.defined_centers.length) out.push(`Defined centers ${r.defined_centers.join(', ')}`);
  if (Array.isArray(r.active_channels) && r.active_channels.length) out.push(`Channels ${r.active_channels.join(', ')}`);
  const pa = asRecord(r.personality_activations);
  const da = asRecord(r.design_activations);
  const gate = (a: AnyRecord) => (a.gate ? `${a.gate}.${a.line ?? '?'}` : '');
  const activationSummary = (activations: AnyRecord) => Object.entries(activations)
    .filter(([, activation]) => Number.isInteger(asRecord(activation).gate))
    .map(([point, activation]) => `${point} ${gate(asRecord(activation))}`);
  const personalityRows = activationSummary(pa);
  const designRows = activationSummary(da);
  if (personalityRows.length) out.push(`Personality activations (point gate.line) ${personalityRows.join('; ')}`);
  if (designRows.length) out.push(`Design activations (point gate.line) ${designRows.join('; ')}`);
  const cross = [gate(asRecord(pa.sun)), gate(asRecord(pa.earth)), gate(asRecord(da.sun)), gate(asRecord(da.earth))].filter(Boolean);
  if (cross.length === 4) out.push(`Incarnation cross gates (P-Sun/P-Earth/D-Sun/D-Earth) ${cross.join(' / ')}`);
  const gates = new Set<number>();
  for (const src of [pa, da]) for (const a of Object.values(src)) { const g = asRecord(a).gate; if (typeof g === 'number') gates.add(g); }
  if (gates.size) out.push(`Gates ${[...gates].sort((a, b) => a - b).join(' ')}`);
  return out;
}

function summariseGeneKeys(r: AnyRecord): string[] {
  const out: string[] = [];
  const seq = asRecord(r.activation_sequence);
  const pair = (k: string, label: string) => {
    const v = seq[k];
    if (Array.isArray(v) && v.length) out.push(`Source activation_sequence.${k} ${JSON.stringify(v)} (raw engine grouping; do not reinterpret as one conventional sphere placement)`);
  };
  pair('lifes_work', "Life's Work");
  pair('evolution', 'Evolution');
  pair('radiance', 'Radiance');
  pair('purpose', 'Purpose');
  const keys = Array.isArray(r.active_keys) ? r.active_keys.slice(0, 6) : [];
  for (const k of keys) {
    const key = asRecord(k);
    const id = key.key_number ?? key.key ?? key.gene_key ?? key.number ?? key.gate;
    const triad = [key.shadow, key.gift, key.siddhi].filter(Boolean).join(' → ');
    if (id && triad) out.push(`Key ${id}: ${triad}`);
  }
  return out;
}

function summariseNumerology(r: AnyRecord): string[] {
  const out: string[] = [];
  for (const k of ['life_path', 'expression', 'personality', 'birthday', 'soul_urge', 'chaldean_name']) {
    const v = asRecord(r[k]);
    if (v.value !== undefined) out.push(`${k.replace('_', ' ')} ${v.value}${v.is_master ? ' (master)' : ''}${v.meaning ? ` — ${v.meaning}` : ''}`);
  }
  return out;
}

function summariseTransits(r: AnyRecord): string[] {
  const out: string[] = [];
  const natal = Array.isArray(r.natal_positions) ? r.natal_positions : [];
  const natalStr = natal
    .map((p: AnyRecord) => `${p.planet} ${p.sign}${typeof p.degree_in_sign === 'number' ? ` ${p.degree_in_sign.toFixed(1)}°` : ''}${p.is_retrograde ? ' R' : ''}`)
    .join(', ');
  if (natalStr) out.push(`Natal positions ${natalStr}`);
  const ss = asRecord(r.sade_sati);
  if (ss.moon_sign) out.push(`Moon sign ${ss.moon_sign}; Sade Sati ${ss.is_active ? `active (${ss.phase ?? 'phase n/a'})` : 'not active'}; Saturn now in ${ss.saturn_sign ?? 'n/a'}`);
  if (r.period_quality) out.push(`Period quality ${typeof r.period_quality === 'string' ? r.period_quality : JSON.stringify(r.period_quality).slice(0, 160)}`);
  if (Array.isArray(r.retrograde_planets) && r.retrograde_planets.length) out.push(`Retrograde now ${r.retrograde_planets.join(', ')}`);
  return out;
}

/** Arithmetic derivation only; this does not assign house meanings or rulers. */
export function deriveWholeSignHouses(r: AnyRecord): string[] {
  const l = asRecord(r.lagna);
  if (r.house_system !== 'whole-sign' || !Number.isInteger(l.sign_index) ||
      l.sign_index < 0 || l.sign_index > 11 || typeof l.longitude !== 'number' ||
      !Number.isFinite(l.longitude) || l.longitude < 0 || l.longitude >= 360 ||
      Math.floor(l.longitude / 30) !== l.sign_index) return [];
  const signs = ['Aries', 'Taurus', 'Gemini', 'Cancer', 'Leo', 'Virgo',
    'Libra', 'Scorpio', 'Sagittarius', 'Capricorn', 'Aquarius', 'Pisces'];
  return Array.from({length:12},(_,i)=>`${i+1}=${signs[(l.sign_index+i)%12]}`).map(String);
}

function summariseVedicKundali(r: AnyRecord): string[] {
  const l = asRecord(r.lagna);
  const out = [`Ayanamsa ${r.ayanamsa}; houses ${r.house_system}; birth-time confidence ${r.birth_time_confidence}`,
    `Lagna ${l.sign} ${l.deg_str}; lord ${l.lord ?? 'not supplied'}; element ${l.element ?? 'not supplied'}; nakshatra ${l.nakshatra} pada ${l.pada}`];
  const houses = deriveWholeSignHouses(r);
  if (houses.length) out.push(`Derived whole-sign house signs (count lagna sign as house 1; arithmetic from matching sign_index and longitude, not additional engine output): ${houses.join('; ')}. Do not infer an unsupplied house ruler or meaning from this table.`);
  for (const [name, raw] of Object.entries(asRecord(r.planets))) {
    const p = asRecord(raw);
    out.push(`${name}: longitude ${p.longitude}; ${p.sign} ${p.deg_str}; house ${p.house}; nakshatra ${p.nakshatra} pada ${p.pada}; retrograde ${p.retrograde}`);
    out.push(`${name} source labels: sign lord ${p.sign_lord ?? 'not supplied'}; nakshatra lord ${p.nakshatra_lord ?? 'not supplied'}; dignity ${p.dignity ?? 'not supplied'}`);
  }
  return out;
}

const SUMMARISERS: Record<string, (r: AnyRecord) => string[]> = {
  'vedic-kundali': summariseVedicKundali,
  panchanga: summarisePanchanga,
  vimshottari: summariseVimshottari,
  'human-design': summariseHumanDesign,
  'gene-keys': summariseGeneKeys,
  numerology: summariseNumerology,
  transits: summariseTransits,
};

/** Engines whose output is moment- or questionnaire-bound; listed but not narrated as birth facts. */
const NON_FACT_ENGINES = new Set(['biorhythm', 'vedic-clock', 'enneagram', 'tarot', 'i-ching', 'sacred-geometry', 'sigil-forge', 'biofield', 'face-reading', 'nadabrahman']);

export function summariseEngine(e: SelemeneEngineOutput, fallbackChars = 400): string {
  if (e._error) return `- ${e.engine_id}: unavailable (${e._error})`;
  const r = asRecord(e.result);
  const backend = e.metadata?.backend ? ` [${e.metadata.backend}]` : '';
  const precision = e.metadata?.precision_achieved ? ` (source precision label: ${e.metadata.precision_achieved}; not an independent accuracy assessment)` : ' (source precision label: not supplied)';
  const fn = SUMMARISERS[e.engine_id];
  if (fn) {
    const facts = fn(r);
    return facts.length ? `- ${e.engine_id}${backend}${precision}: ${facts.join('; ')}` : `- ${e.engine_id}${backend}: no summarisable facts`;
  }
  if (NON_FACT_ENGINES.has(e.engine_id)) return `- ${e.engine_id}${backend}: present (moment-bound; not a birth fact)`;
  return `- ${e.engine_id}${backend}: ${JSON.stringify(r).slice(0, fallbackChars)}`;
}

/** Render one block per subject. Deterministic: same input → same text. */
export function buildEngineFactsBlock(input: EngineFactsInput): string {
  const sections: string[] = [];
  input.subjectNames.forEach((name, i) => {
    const role = input.subjectRoles?.[i];
    const heading = role ? `${name} (${role.role}${role.label ? `, ${role.label}` : ''})` : name;
    const engines = input.engineResultsBySubject[i] ?? [];
    const lines = engines.length
      ? engines.map((e) => summariseEngine(e, input.fallbackChars))
      : ['- no engine results supplied'];
    sections.push(`### ${heading}\n${lines.join('\n')}`);
  });
  return sections.join('\n\n');
}


// ─── Per-engine micro-interpretation additions ────────────────────────
// Phase B additive surface for the reader-facing witness contract repair.
// buildEngineFactsBlock() remains the historical flat block used by voice
// prompts. buildPerEngineMicroInterpretationPrompt() and buildEngineFieldsConsumed()
// produce the private-map inputs required by the corrective plan §2.1 and
// evidence-map.ts.

/** Register band for the micro-interpretation prompt. */
export type RegisterBand = 'L1-L3' | 'L4-L5';

/**
 * Machine-readable manifest of which engine fields were consumed at
 * generation time. Feeds PrivateEvidenceMap.engine_facts_consumed.
 * Deterministic: same engine output in → same manifest out.
 */
export interface EngineFieldsConsumed {
  engine_id: string;
  fields_used: string[];
  values_snapshot: Record<string, unknown>;
  source_precision_label: string | null;
  backend: string;
}

/**
 * Flatten an engine result into a dotted-path record. Kept private to
 * this module because it is a light-weight private helper: it does not
 * try to be a full JSON-Pointer walker, only enough to name the leaves
 * a micro-interpretation is likely to cite.
 */
function flattenFields(result: unknown, prefix = '', out: Record<string, unknown> = {}, depth = 0): Record<string, unknown> {
  if (depth > 4 || result === null || result === undefined) return out;
  if (typeof result !== 'object') {
    if (prefix) out[prefix] = result;
    return out;
  }
  if (Array.isArray(result)) {
    result.slice(0, 12).forEach((v, i) => flattenFields(v, prefix ? `${prefix}.${i}` : String(i), out, depth + 1));
    return out;
  }
  for (const [k, v] of Object.entries(result as Record<string, unknown>)) {
    const nextPrefix = prefix ? `${prefix}.${k}` : k;
    if (v && typeof v === 'object') {
      flattenFields(v, nextPrefix, out, depth + 1);
    } else {
      out[nextPrefix] = v;
    }
  }
  return out;
}

/**
 * Compute a compact fields-consumed manifest for one engine output. The
 * `fieldsFilter` argument, when supplied, restricts fields to those whose
 * dotted path passes the predicate (used by section overlays to keep the
 * private map focused).
 */
export function buildEngineFieldsConsumed(
  engine: SelemeneEngineOutput,
  fieldsFilter?: (path: string) => boolean,
): EngineFieldsConsumed {
  const flat = flattenFields(engine.result);
  const filtered: Record<string, unknown> = {};
  for (const [k, v] of Object.entries(flat)) {
    if (v === null || v === undefined || v === '') continue;
    if (fieldsFilter && !fieldsFilter(k)) continue;
    filtered[k] = v;
  }
  return {
    engine_id: engine.engine_id,
    fields_used: Object.keys(filtered).sort(),
    values_snapshot: filtered,
    source_precision_label: engine.metadata?.precision_achieved ?? null,
    backend: engine.metadata?.backend ?? 'unknown',
  };
}

/**
 * Build a structured per-engine micro-interpretation prompt suitable for
 * a small LLM extraction call. Output is deliberately declarative — the
 * caller renders it inside its own instruction envelope, but this function
 * decides exactly which engine fields the interpretation is anchored to
 * and at which register band.
 *
 * This function performs NO LLM calls itself. It returns a plain string
 * plus the machine-readable manifest so the caller can attach both to the
 * private evidence map without recomputation.
 */
export function buildPerEngineMicroInterpretationPrompt(args: {
  engine: SelemeneEngineOutput;
  register: RegisterBand;
  subject: string;
  sectionTopic?: string;
  fieldsFilter?: (path: string) => boolean;
}): { prompt: string; fields: EngineFieldsConsumed } {
  const fields = buildEngineFieldsConsumed(args.engine, args.fieldsFilter);
  const factsList = fields.fields_used.length
    ? fields.fields_used.map(f => `- ${f} = ${JSON.stringify(fields.values_snapshot[f])}`).join('\n')
    : '- (no non-empty fields consumed for this engine)';
  const registerHint = args.register === 'L1-L3'
    ? 'Interpret strictly within traditional lineage vocabulary. Do not extend into modern framework attribution.'
    : 'Interpret within the framework vocabulary appropriate to this engine. Do not import external traditions.';
  const topicLine = args.sectionTopic ? `Section topic: ${args.sectionTopic}` : 'Section topic: (unspecified)';
  const prompt = [
    `Subject: ${args.subject}`,
    `Engine: ${args.engine.engine_id}`,
    `Register: ${args.register}`,
    topicLine,
    '',
    'Engine facts available for interpretation:',
    factsList,
    '',
    registerHint,
    '',
    'Task: produce a JSON array containing 1–8 section-relevant micro-interpretations. Select the most consequential supplied fields for this section; do not mechanically repeat every field.',
    'Every array entry MUST use exactly these keys and MUST include every key:',
    '{"engine_id":"<exact Engine value above>","claim":"<concise supported interpretation>","source_field":"<exact dotted field path from the supplied list>","source_value":"<exact supplied value as a string>","interpretation_tradition":"<named tradition or framework>","confidence":"direct|derived|framework-attributed|interpretive-synthesis","contradictions":[],"uncertainty_note":null}',
    'Do not rename source_field to field, field_path, source, or source_path. Do not rename source_value to value. Never omit source_field, source_value, or interpretation_tradition.',
    'For each interpretation, cite the exact engine field and value it derives from, name the tradition or framework, mark confidence using the enumeration above, and list any contradicting engine claims.',
    'Do not add meanings that no engine field supports. If a factual absence is meaningful, state it explicitly as an "absence" interpretation with confidence "direct".',
    'Return raw JSON only; no prose, no markdown fences.',
  ].join('\n');
  return { prompt, fields };
}
