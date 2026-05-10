// Tala-aware pattern composition. Wraps a freq() mini-notation in a `.struct()`
// or `.gain()` chain that emphasizes samam (downbeat) and edam (mid-cycle).

import type { Tala } from './types';
import { talaGainPattern } from './strudel';

export interface TalaEmitterOptions {
  swaraCount: number;
  /** Gain on samam (downbeat). Default 1.5 (≈ +3.5 dB). */
  samamGain?: number;
  /** Gain on edam / secondary accent. Default 1.2. */
  edamGain?: number;
  /** Whether to repeat the swara sequence to fill an integer multiple of beats. */
  fitToBeats?: boolean;
}

export interface TalaEmitterResult {
  /** Strudel chain suffix to append, e.g. ".gain(\"1.5 1 1 1 1.2 1 1.2 1\")". */
  gainChain: string;
  /** How many beats the resulting pattern occupies. */
  totalBeats: number;
}

/**
 * Build a tala-aware suffix that lays accents on samam/edam beats. The caller
 * is responsible for applying `.slow(n)` to stretch the pattern across the
 * desired number of cycles; this function only emits the gain map.
 */
export const emitTala = (tala: Tala, opts: TalaEmitterOptions): TalaEmitterResult => {
  const gain = talaGainPattern(tala, {
    swaraCount: opts.swaraCount,
    samamGain: opts.samamGain ?? 1.5,
    edamGain: opts.edamGain ?? 1.2,
  });
  return {
    gainChain: `.gain("${gain}")`,
    totalBeats: tala.beats,
  };
};
