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
    if (Array.isArray(v) && v.length) out.push(`${label} ${v.join('/')}`);
  };
  pair('lifes_work', "Life's Work");
  pair('evolution', 'Evolution');
  pair('radiance', 'Radiance');
  pair('purpose', 'Purpose');
  const keys = Array.isArray(r.active_keys) ? r.active_keys.slice(0, 6) : [];
  for (const k of keys) {
    const key = asRecord(k);
    const id = key.key ?? key.gene_key ?? key.number ?? key.gate;
    const triad = [key.shadow, key.gift, key.siddhi].filter(Boolean).join(' → ');
    if (id && triad) out.push(`Key ${id}: ${triad}`);
  }
  return out;
}

function summariseNumerology(r: AnyRecord): string[] {
  const out: string[] = [];
  for (const k of ['life_path', 'expression', 'personality', 'birthday', 'soul_urge']) {
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

const SUMMARISERS: Record<string, (r: AnyRecord) => string[]> = {
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
  const fn = SUMMARISERS[e.engine_id];
  if (fn) {
    const facts = fn(r);
    return facts.length ? `- ${e.engine_id}${backend}: ${facts.join('; ')}` : `- ${e.engine_id}${backend}: no summarisable facts`;
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
