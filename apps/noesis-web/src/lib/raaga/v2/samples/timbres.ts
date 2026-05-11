// Per-timbre ADSR + filter defaults. Captures how each instrument *attacks*
// the swara — sitar plucks, bansuri breathes, sarangi bows, mridangam strikes,
// synth pads swell, leads sing, plucks pop.
//
// Sample-based timbres need the manifest loaded (see loader.ts).
// Synth-based timbres use Strudel's built-in oscillators — no CDN required.

export type Timbre =
  // Sample-based (require manifest)
  | 'sine' | 'sitar' | 'tanpura' | 'mridangam' | 'bansuri' | 'sarangi'
  // Synth-based (built into Strudel — no sample loading)
  | 'sawlead' | 'pad' | 'supersawpad' | 'squarepluck' | 'dronesynth';

export type TimbreCategory = 'sample' | 'synth';

export interface TimbreProfile {
  /** Strudel sound name passed to `.s()`. */
  sound: string;
  /** 'sample' = manifest required; 'synth' = built-in oscillator. */
  category: TimbreCategory;
  attack: number;
  decay: number;
  sustain: number;
  release: number;
  /** Optional low-pass filter cutoff in Hz. Undefined = no filter. */
  lpf?: number;
  /** Optional resonance Q for the LPF. */
  lpq?: number;
  /** Optional high-pass cutoff. */
  hpf?: number;
  /** Linear gain multiplier. */
  gain: number;
  /** Optional pitch detune in cents (slight thickness). */
  detune?: number;
  /** Optional human-readable description. */
  notes?: string;
}

export const TIMBRES: { readonly [K in Timbre]: TimbreProfile } = {
  // ── Sample-based ──────────────────────────────────────────────────────
  'sine': {
    sound: 'sine', category: 'synth',
    attack: 0.01, decay: 0.05, sustain: 0.85, release: 0.15,
    gain: 0.8,
    notes: 'V1 baseline — pure sine. Used for shruti precision verification.',
  },
  'sitar': {
    sound: 'sitar', category: 'sample',
    attack: 0.005, decay: 0.3, sustain: 0.4, release: 0.8,
    lpf: 4500, lpq: 2,
    gain: 1.0,
    notes: 'Sharp pluck attack; ringing decay; mild low-pass for warmth.',
  },
  'tanpura': {
    sound: 'tanpura', category: 'sample',
    attack: 0.5, decay: 1.5, sustain: 0.95, release: 2.5,
    gain: 0.6,
    notes: 'Slow swell, long sustain. Used as drone bed under main raga.',
  },
  'mridangam': {
    sound: 'mridangam', category: 'sample',
    attack: 0.001, decay: 0.08, sustain: 0.0, release: 0.3,
    hpf: 100, lpf: 6000,
    gain: 1.1,
    notes: 'Percussion — no sustain, sharp transient.',
  },
  'bansuri': {
    sound: 'bansuri', category: 'sample',
    attack: 0.08, decay: 0.1, sustain: 0.85, release: 0.4,
    lpf: 5000, lpq: 1,
    gain: 0.9,
    notes: 'Breath-driven attack; smooth sustain; airy release.',
  },
  'sarangi': {
    sound: 'sarangi', category: 'sample',
    attack: 0.15, decay: 0.2, sustain: 0.9, release: 0.5,
    lpf: 3500, lpq: 1.5,
    gain: 0.95,
    notes: 'Bowed swell; dark filter for the characteristic sarangi warmth.',
  },

  // ── Synth-based (no sample manifest required) ─────────────────────────
  'sawlead': {
    sound: 'sawtooth', category: 'synth',
    attack: 0.02, decay: 0.1, sustain: 0.7, release: 0.3,
    lpf: 3500, lpq: 4,
    gain: 0.6,
    detune: 0,
    notes: 'Bright lead synth — sawtooth + resonant LPF. Cuts through tanpura.',
  },
  'pad': {
    sound: 'sawtooth', category: 'synth',
    attack: 0.6, decay: 0.5, sustain: 0.9, release: 1.5,
    lpf: 1800, lpq: 2,
    gain: 0.45,
    detune: 7,
    notes: 'Slow swelling pad — long attack, subtle detune for chorus thickness.',
  },
  'supersawpad': {
    sound: 'supersaw', category: 'synth',
    attack: 0.8, decay: 0.6, sustain: 0.95, release: 2.0,
    lpf: 2500, lpq: 1.5,
    gain: 0.4,
    detune: 12,
    notes: 'Lush supersaw pad — 7-voice unison, wide stereo feel for ambient passages.',
  },
  'squarepluck': {
    sound: 'square', category: 'synth',
    attack: 0.001, decay: 0.15, sustain: 0.0, release: 0.2,
    lpf: 5000, lpq: 3,
    gain: 0.5,
    notes: 'Plucky square — chiptune-meets-veena. Good for fast paltas (drut).',
  },
  'dronesynth': {
    sound: 'sawtooth', category: 'synth',
    attack: 1.5, decay: 1.0, sustain: 1.0, release: 4.0,
    lpf: 1200, lpq: 1,
    gain: 0.35,
    detune: 5,
    notes: 'Synthetic tanpura alternative — endless drone with very slow swell.',
  },
};

/** Build a Strudel chain suffix expressing this timbre's articulation. */
export const timbreStrudelSuffix = (t: TimbreProfile): string => {
  const parts: string[] = [
    `.s("${t.sound}")`,
    `.attack(${t.attack})`,
    `.decay(${t.decay})`,
    `.sustain(${t.sustain})`,
    `.release(${t.release})`,
    `.gain(${t.gain})`,
  ];
  if (t.lpf != null) parts.push(`.lpf(${t.lpf})`);
  if (t.lpq != null) parts.push(`.lpq(${t.lpq})`);
  if (t.hpf != null) parts.push(`.hpf(${t.hpf})`);
  if (t.detune != null && t.detune !== 0) parts.push(`.detune(${t.detune})`);
  return parts.join('');
};

/** Filter to only synth-based timbres (no manifest needed). */
export const SYNTH_TIMBRES: readonly Timbre[] =
  (Object.entries(TIMBRES).filter(([, p]) => p.category === 'synth').map(([k]) => k)) as readonly Timbre[];
