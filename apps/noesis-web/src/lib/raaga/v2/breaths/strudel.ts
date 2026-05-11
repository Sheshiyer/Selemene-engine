// Map a Breath onto Strudel cps + ADSR articulation.
//
// Heuristic: faster breath = faster cps, sharper attack. Slower / sustained
// breath = slower cps, longer release. The articulation envelope shapes
// each swara so Bhastrika *feels* like bellows and Brahmari *feels* like a hum.

import type { Breath } from './data';

export interface BreathArticulation {
  /** Cycles per second for Strudel `setcps`. */
  cps: number;
  /** Attack in seconds. */
  attack: number;
  /** Decay in seconds. */
  decay: number;
  /** Sustain level [0..1]. */
  sustain: number;
  /** Release in seconds. */
  release: number;
  /** Optional gain multiplier (LFO-style breath modulation). */
  gainMul: number;
}

/**
 * Map a breath cycle to Strudel articulation.
 *
 * cps ≈ 1 / (cycleSeconds × 4)   — one swara takes a quarter of the breath
 * attack scales with inhale (pattern[0]); release scales with exhale (pattern[2]).
 */
export const breathToArticulation = (b: Breath): BreathArticulation => {
  const [inhale, holdIn, exhale, holdOut] = b.pattern;
  const cycleS = Math.max(0.5, b.cycleSeconds);
  const cps = +(1 / (cycleS * 2)).toFixed(4);  // one swara every half-breath
  const attack = +Math.min(1.5, Math.max(0.01, inhale * 0.05)).toFixed(3);
  const release = +Math.min(2.5, Math.max(0.05, exhale * 0.08)).toFixed(3);
  // Bhastrika & Kapalabhati: very sharp staccato
  const isStaccato = b.name === 'bhastrika' || b.name === 'kapalabhati';
  const isSustain = b.name === 'brahmari' || b.name === 'shitali' || b.name === 'dirgha';
  return {
    cps: isStaccato ? Math.max(cps * 4, 1.0) : cps,
    attack: isStaccato ? 0.005 : attack,
    decay: isStaccato ? 0.02 : 0.1,
    sustain: isStaccato ? 0.3 : isSustain ? 0.95 : 0.7,
    release: isStaccato ? 0.05 : isSustain ? Math.max(release, 1.0) : release,
    gainMul: 1.0,
  };
};

/**
 * Build a Strudel articulation suffix that can be appended to a freq() chain:
 *
 *   freq("...").s("sitar").attack(0.04).decay(0.1).sustain(0.7).release(0.4)
 */
export const articulationStrudelSuffix = (a: BreathArticulation): string =>
  `.attack(${a.attack}).decay(${a.decay}).sustain(${a.sustain}).release(${a.release}).gain(${a.gainMul})`;
