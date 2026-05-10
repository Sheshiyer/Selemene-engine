// Alaap renderer — produces a long Strudel `freq()` mini-notation string
// covering vilambit → madhya → drut phases with pauses (silences) between
// phrases. Pure function; no Strudel imports.
//
// The output is meant to play unbroken for ~30-90 seconds depending on which
// phases are enabled, with a tanpura layer running underneath.

import type { AlaapConfig, AlaapPhase, AlaapPhaseConfig } from './types';
import { DEFAULT_PHASE_CONFIGS } from './types';
import { renderGamaka } from '../gamakas/render';

export interface AlaapRenderInput {
  /** Per-swara absolute Hz (length 8: Sa..Sa'). */
  hzs: readonly number[];
  /** Alaap configuration. */
  config: AlaapConfig;
}

export interface AlaapRenderOutput {
  /** Strudel freq() mini-notation for the whole alaap. */
  miniNotation: string;
  /** Total duration in seconds (sum of all phases + pauses). */
  durationSeconds: number;
  /** Suggested cps to drive the whole composition. We use the slowest phase
   *  as the cps; other phases adjust their hold via mini-notation weights. */
  cps: number;
  /** Per-phase metadata for caller diagnostics. */
  phases: ReadonlyArray<{ phase: AlaapPhase; durationSeconds: number; phraseCount: number }>;
}

/**
 * Generate phrase shapes for a phase. Phrases are short ascending fragments
 * starting at progressively higher swaras.
 *
 * Vilambit: 1-2 swara phrases — Sa, then Sa-Re, then Sa-Re-Ga.
 * Madhya: 3-5 swara ascending phrases.
 * Drut: 5-7 swara fast tans.
 */
const phrasesForPhase = (phase: AlaapPhase, hzCount: number): readonly (readonly number[])[] => {
  if (phase === 'vilambit') {
    // Build up: Sa alone, then Sa-Re, Sa-Re-Ga, settle on Re
    return [
      [0],                  // Sa held long
      [0, 1],               // Sa-Re
      [0, 1, 2, 1, 0],      // Sa-Re-Ga-Re-Sa
      [0, 1, 2, 1],         // settle
    ];
  }
  if (phase === 'madhya') {
    // Full octave exploration in 3-5 swara phrases
    return [
      [0, 1, 2, 3, 2, 1, 0],          // Sa-Re-Ga-Ma-Ga-Re-Sa
      [2, 3, 4, 3, 2],                 // Ga-Ma-Pa-Ma-Ga
      [0, 2, 4, 5, 4, 2, 0],          // Sa-Ga-Pa-Dha-Pa-Ga-Sa
      [3, 4, 5, 6, 5, 4, 3],          // Ma-Pa-Dha-Ni-Dha-Pa-Ma
      [0, 4, 5, 6, 7],                 // reach Sa'
    ];
  }
  // drut — fast cascading runs
  return [
    [0, 1, 2, 3, 4, 5, 6, 7],         // straight ascent
    [7, 6, 5, 4, 3, 2, 1, 0],         // straight descent
    [0, 2, 1, 3, 2, 4, 3, 5, 4, 6, 5, 7],  // oscillating ascent (sangati)
    [7, 5, 6, 4, 5, 3, 4, 2, 3, 1, 2, 0],  // oscillating descent
  ];
};

const fmt = (hz: number): string => hz.toFixed(3);

export const renderAlaap = ({ hzs, config }: AlaapRenderInput): AlaapRenderOutput => {
  if (hzs.length < 8) throw new Error('alaap needs 8 hzs (Sa..Sa\')');

  // Pick the slowest active phase as the global cps so mini-notation weights
  // can stretch other phases proportionally.
  const phaseConfigs = config.phases.map((p) => ({
    ...DEFAULT_PHASE_CONFIGS[p],
    cps: config.cpsOverrides?.[p] ?? DEFAULT_PHASE_CONFIGS[p].cps,
    defaultGamaka: config.gamakaOverrides?.[p] ?? DEFAULT_PHASE_CONFIGS[p].defaultGamaka,
  }));

  const globalCps = Math.min(...phaseConfigs.map((c) => c.cps));
  const fragments: string[] = [];
  const phaseMeta: { phase: AlaapPhase; durationSeconds: number; phraseCount: number }[] = [];
  let totalSeconds = 0;

  for (const cfg of phaseConfigs) {
    const phrases = phrasesForPhase(cfg.phase, hzs.length);
    let phaseSeconds = 0;

    // Mini-notation weight to convert this phase's per-swara duration to
    // the global cps' beat space. weight = swaraSeconds * globalCps.
    const swaraWeight = cfg.swaraSeconds * globalCps;
    const pauseWeight = cfg.pauseSeconds * globalCps;

    for (const phrase of phrases) {
      // Each swara in the phrase: render with phase's default gamaka, weighted.
      for (const swaraIdx of phrase) {
        const hz = hzs[swaraIdx];
        const ornamented = renderGamaka(hz, cfg.defaultGamaka);
        // If ornamented is a bracket expression, wrap with weight; otherwise just append weight.
        fragments.push(`${ornamented}@${swaraWeight.toFixed(4)}`);
        phaseSeconds += cfg.swaraSeconds;
      }
      // Pause after each phrase
      if (pauseWeight > 0) {
        fragments.push(`~@${pauseWeight.toFixed(4)}`);
        phaseSeconds += cfg.pauseSeconds;
      }
    }

    phaseMeta.push({ phase: cfg.phase, durationSeconds: phaseSeconds, phraseCount: phrases.length });
    totalSeconds += phaseSeconds;
  }

  return {
    miniNotation: fragments.join(' '),
    durationSeconds: totalSeconds,
    cps: globalCps,
    phases: phaseMeta,
  };
};
