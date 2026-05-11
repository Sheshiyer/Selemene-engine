// applyGamaka — real renderer (Phase 2). Builds a Strudel `freq()` mini-notation
// string with per-swara gamaka expansions, ready to feed `evaluate()`.

import type { Gamaka } from './types';
import { renderGamaka, renderPlain } from './render';

export interface GamakaAnnotation {
  /** Index into the arohana/avarohana sequence the gamaka attaches to. */
  swaraIndex: number;
  /** Direction the swara appears in. `'up'` = arohana, `'down'` = avarohana. */
  direction: 'up' | 'down' | 'both';
  /** The ornament to apply at this swara position. */
  gamaka: Gamaka;
}

export interface ApplyGamakaInput {
  /** Absolute Hz values per swara (pre-baked just-intonation, no quantization). */
  hzs: readonly number[];
  /** Optional gamaka annotations. Empty array = pure shrutis, no ornaments. */
  annotations: readonly GamakaAnnotation[];
  /** Cycles-per-second for playback (Strudel cps). */
  cps: number;
  /** Hold duration per swara in cycles. Default 1.0 = one cps cycle per swara. */
  holdCycles?: number;
  /** Whether the input represents arohana, avarohana, or both. */
  direction?: 'up' | 'down' | 'both';
  /** Default gamaka applied when no per-swara annotation exists. */
  defaultGamaka?: Gamaka;
}

export interface ApplyGamakaOutput {
  /** The freq() mini-notation string (just the inside of the quotes). */
  miniNotation: string;
  /** Total duration in seconds. */
  durationSeconds: number;
  /** Sanity metadata: which gamakas were applied per swara. */
  appliedKinds: readonly string[];
}

export type ApplyGamaka = (input: ApplyGamakaInput) => ApplyGamakaOutput;

/** Pick the gamaka for a given swara index, considering direction filter. */
const pickAnnotation = (
  swaraIndex: number,
  direction: 'up' | 'down' | 'both',
  annotations: readonly GamakaAnnotation[],
  defaultGamaka: Gamaka,
): Gamaka => {
  for (const a of annotations) {
    if (a.swaraIndex !== swaraIndex) continue;
    if (a.direction === 'both' || a.direction === direction || direction === 'both') {
      return a.gamaka;
    }
  }
  return defaultGamaka;
};

export const applyGamaka: ApplyGamaka = ({
  hzs,
  annotations,
  cps,
  holdCycles = 1,
  direction = 'both',
  defaultGamaka = { kind: 'none' },
}) => {
  const fragments: string[] = [];
  const appliedKinds: string[] = [];

  for (let i = 0; i < hzs.length; i++) {
    const hz = hzs[i];
    // For 'both' direction sequences (arohana + avarohana stitched), figure
    // out per-step whether we're ascending or descending.
    const half = Math.ceil(hzs.length / 2);
    const stepDir: 'up' | 'down' | 'both' =
      direction === 'both' ? (i < half ? 'up' : 'down') : direction;

    const g = pickAnnotation(i, stepDir, annotations, defaultGamaka);
    fragments.push(g.kind === 'none' ? renderPlain(hz) : renderGamaka(hz, g));
    appliedKinds.push(g.kind);
  }

  const miniNotation = fragments.join(' ');
  const durationSeconds = (hzs.length * holdCycles) / cps;

  return { miniNotation, durationSeconds, appliedKinds };
};
