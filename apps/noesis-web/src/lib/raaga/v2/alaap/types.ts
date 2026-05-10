// Alaap (आलाप) — the slow, rhythm-free improvisational opening of a
// Hindustani / Carnatic performance. The most contemplative form: free time
// (no tala), gradual exploration of the raga's swaras, deep gamakas on every
// note, pauses as important as sounds.
//
// Three classical phases:
//   1. Vilambit (विलम्बित)  — very slow. Sa-Re-Ga only at first. Each swara
//                              held many seconds with andolana.
//   2. Madhya (मध्य)        — moderate. Full octave traversed. Kampita on
//                              every swara. Phrases (tans) appear.
//   3. Drut (द्रुत)         — fast. Virtuosic palta-style runs, sharp attacks.
//
// Each phase has its own cps, gamaka density, hold duration, and pause
// behavior. The composer stitches phases sequentially with silences between.

import type { Gamaka } from '../gamakas/types';

export type AlaapPhase = 'vilambit' | 'madhya' | 'drut';

export interface AlaapPhaseConfig {
  /** Phase identifier. */
  phase: AlaapPhase;
  /** Cycles per second for this phase. Vilambit is very low. */
  cps: number;
  /** Hold duration per swara in seconds. */
  swaraSeconds: number;
  /** Default gamaka applied to every swara in this phase. */
  defaultGamaka: Gamaka;
  /** Pause (silence) duration after each phrase, in seconds. */
  pauseSeconds: number;
  /** Which swara indices to traverse. Vilambit uses just lower-octave. */
  swaraRange: 'lower' | 'middle' | 'full';
}

export interface AlaapConfig {
  /** Which phases to include in order. Default: all three. */
  phases: readonly AlaapPhase[];
  /** Override cps for a phase. */
  cpsOverrides?: Partial<Record<AlaapPhase, number>>;
  /** Override gamaka for a phase. */
  gamakaOverrides?: Partial<Record<AlaapPhase, Gamaka>>;
  /** Forces tanpura on (alaap is meaningless without drone). */
  tanpura: true;
}

export const DEFAULT_PHASE_CONFIGS: { readonly [P in AlaapPhase]: AlaapPhaseConfig } = {
  vilambit: {
    phase: 'vilambit',
    cps: 0.05,            // ~20s per cycle — extremely slow
    swaraSeconds: 6.0,
    defaultGamaka: { kind: 'andolana', cents: 60, rateHz: 0.8 },
    pauseSeconds: 4.0,
    swaraRange: 'lower',  // Sa-Re-Ga area only
  },
  madhya: {
    phase: 'madhya',
    cps: 0.1,             // ~10s per cycle
    swaraSeconds: 2.0,
    defaultGamaka: { kind: 'kampita', cents: 25, rateHz: 5 },
    pauseSeconds: 1.5,
    swaraRange: 'full',
  },
  drut: {
    phase: 'drut',
    cps: 0.3,             // faster — palta-like runs
    swaraSeconds: 0.4,
    defaultGamaka: { kind: 'sphurita', startCents: -30, attackFraction: 0.15 },
    pauseSeconds: 0.5,
    swaraRange: 'full',
  },
};
