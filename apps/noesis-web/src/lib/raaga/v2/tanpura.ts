// Tanpura drone — Sa-Pa-Sa-Sa loop sustained under the main raga. The
// classical tanpura's four strings are tuned: Pa (lower 5th) - Sa - Sa - Sa
// (lower octave). We emit one Strudel pattern that loops at a slow cps so
// the four notes cycle continuously while the raga plays on top.
//
// IMPORTANT: defaults to a SYNTH oscillator (sawtooth) so it produces sound
// without requiring the sample manifest to be loaded. Pass `useSample: true`
// to switch to the `tanpura` sample (silent until samples('...') is called).

export interface TanpuraOptions {
  /** Sa pitch in Hz. The drone tunes to this. */
  rootHz: number;
  /** Cycles per second for the drone. Default 0.08 — one full Pa-Sa-Sa-Sa
   *  cycle every ~12.5 seconds, traditional pace. */
  cps?: number;
  /** Gain (0..1). Default 0.4 — sits under main raga without masking. */
  gain?: number;
  /** Use the sample-based tanpura instead of the synth fallback. Requires
   *  the sample manifest to be loaded first via `loadRaagaSamples()`. */
  useSample?: boolean;
}

export interface TanpuraResult {
  code: string;
}

/**
 * Build the tanpura Strudel code. Caller dispatches via `evaluate(code)` on
 * a separate "lane" or via `stack(rg, tan)` to layer with a main raga.
 */
export const buildTanpuraCode = (opts: TanpuraOptions): TanpuraResult => {
  const { rootHz, cps = 0.08, gain = 0.4, useSample = false } = opts;
  const pa = +(rootHz * (3 / 2) / 2).toFixed(3);  // lower Pa (3/2 down an octave)
  const sa = +rootHz.toFixed(3);
  const saLow = +(rootHz / 2).toFixed(3);          // lower Sa

  let chain: string;
  if (useSample) {
    // Sample-based tanpura — only audible after `samples()` is loaded.
    chain = `freq("${pa} ${sa} ${sa} ${saLow}").s("tanpura").attack(0.5).decay(1.5).sustain(0.95).release(2.5).gain(${gain})`;
  } else {
    // Synth tanpura — works immediately. Sawtooth + LPF + slight detune
    // for stereo width; long ADSR for sustained drone character.
    chain = `freq("${pa} ${sa} ${sa} ${saLow}").s("sawtooth").attack(1.5).decay(1.0).sustain(1.0).release(4.0).gain(${gain}).lpf(1200).detune(5)`;
  }
  const code = `setcps(${cps}); ${chain}`;
  return { code };
};
