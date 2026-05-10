// Map a Tala onto a Strudel pattern fragment. Phase-2 swarms (T-032..T-035)
// will deepen this; Phase-1 freezes the contract.

import type { Tala } from './types';

export interface TalaPatternOptions {
  /** Number of swaras in the raga sequence. */
  swaraCount: number;
  /** Optional per-swara gain boost (linear). E.g. 1.5 = +3.5dB on samam. */
  samamGain?: number;
  /** Optional secondary accent gain. */
  edamGain?: number;
}

/**
 * Build a Strudel `gain` mini-notation string that emphasizes the tala's
 * accent beats. The pattern length must equal the tala's beat count.
 *
 * Example: Adi (8 beats, accents on 0,4,6) →
 *   "1.5 1 1 1 1.3 1 1.3 1"
 */
export const talaGainPattern = (tala: Tala, opts: TalaPatternOptions = { swaraCount: 0 }): string => {
  const samam = opts.samamGain ?? 1.5;
  const edam = opts.edamGain ?? 1.2;
  const beats = Array.from({ length: tala.beats }, (_, i) => {
    if (i === tala.accentBeats[0]) return samam.toFixed(2);
    if (tala.accentBeats.includes(i)) return edam.toFixed(2);
    return '1';
  });
  return beats.join(' ');
};

/**
 * Build a Strudel `euclid()` invocation expression for this tala. Used as a
 * structural mask: `pattern.struct(euclid(${k}, ${n}))`.
 */
export const talaEuclidExpr = (tala: Tala): string => {
  const [k, n] = tala.euclid;
  return `euclid(${k}, ${n})`;
};

/**
 * Suggested cps for a tala when no breath override is provided. The slower
 * the cycle structure, the lower the cps. Heuristic.
 */
export const defaultCpsForTala = (tala: Tala): number => {
  // 8-beat tala at 0.5 cps → one full cycle in 16 seconds, ~1 swara/2s
  return 0.5 * (8 / tala.beats);
};
