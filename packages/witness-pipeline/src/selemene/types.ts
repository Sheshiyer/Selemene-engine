// ─── Engine identifiers, routing, and typed result map ───────────────
// Ported from witness-agents/src/types/engine.ts

export const SELEMENE_ENGINE_IDS = [
  'panchanga', 'vimshottari', 'human-design', 'gene-keys', 'numerology',
  'biorhythm', 'vedic-clock', 'biofield', 'face-reading', 'nadabrahman',
  'transits', 'tarot', 'i-ching', 'enneagram', 'sacred-geometry', 'sigil-forge',
] as const;

export type SelemeneEngineId = (typeof SELEMENE_ENGINE_IDS)[number];

export type WitnessEngineAlias =
  | 'temporal-grammar' | 'chronofield' | 'energetic-authority' | 'gift-shadow-spectrum'
  | 'numeric-architecture' | 'three-wave-cycle' | 'circadian-cartography' | 'bioelectric-field'
  | 'physiognomic-mapping' | 'resonance-architecture' | 'active-planetary-weather'
  | 'archetypal-mirror' | 'hexagram-navigation' | 'nine-point-architecture'
  | 'geometric-resonance' | 'sigil-forge';

export const ENGINE_ID_MAP: Record<SelemeneEngineId, WitnessEngineAlias> = {
  'panchanga': 'temporal-grammar',
  'vimshottari': 'chronofield',
  'human-design': 'energetic-authority',
  'gene-keys': 'gift-shadow-spectrum',
  'numerology': 'numeric-architecture',
  'biorhythm': 'three-wave-cycle',
  'vedic-clock': 'circadian-cartography',
  'biofield': 'bioelectric-field',
  'face-reading': 'physiognomic-mapping',
  'nadabrahman': 'resonance-architecture',
  'transits': 'active-planetary-weather',
  'tarot': 'archetypal-mirror',
  'i-ching': 'hexagram-navigation',
  'enneagram': 'nine-point-architecture',
  'sacred-geometry': 'geometric-resonance',
  'sigil-forge': 'sigil-forge',
};

export const REVERSE_ENGINE_MAP: Record<WitnessEngineAlias, SelemeneEngineId> =
  Object.fromEntries(Object.entries(ENGINE_ID_MAP).map(([k, v]) => [v, k])) as Record<WitnessEngineAlias, SelemeneEngineId>;

export type RoutingMode = 'aletheios-primary' | 'pichet-primary' | 'dyad-synthesis';

export const ENGINE_ROUTING: Record<SelemeneEngineId, RoutingMode> = {
  'vimshottari': 'aletheios-primary',
  'human-design': 'aletheios-primary',
  'enneagram': 'aletheios-primary',
  'i-ching': 'aletheios-primary',
  'numerology': 'aletheios-primary',
  'biorhythm': 'pichet-primary',
  'vedic-clock': 'pichet-primary',
  'biofield': 'pichet-primary',
  'face-reading': 'pichet-primary',
  'nadabrahman': 'pichet-primary',
  'panchanga': 'dyad-synthesis',
  'gene-keys': 'dyad-synthesis',
  'tarot': 'dyad-synthesis',
  'sacred-geometry': 'dyad-synthesis',
  'sigil-forge': 'dyad-synthesis',
  'transits': 'dyad-synthesis',
};

export interface CalculationMetadata {
  calculation_time_ms: number;
  backend: string;
  precision_achieved: string;
  cached: boolean;
  timestamp: string;
  engine_version: string;
}

export interface SelemeneEngineOutput {
  engine_id: SelemeneEngineId;
  result: unknown;
  witness_prompt: string;
  consciousness_level: number;
  metadata: CalculationMetadata;
  envelope_version: string;
  _error?: string;
}

export interface BirthData {
  date: string;
  time?: string;
  timezone?: string;
  latitude?: number;
  longitude?: number;
  name?: string;
}
