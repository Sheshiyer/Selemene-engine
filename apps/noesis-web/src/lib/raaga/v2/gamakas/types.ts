// Gamaka — microtonal pitch ornamentation. Frozen contract for v2 swarms.
//
// Five canonical gamakas from Sangita Ratnakara IV (Sārṅgadeva):
//   - kampita    : light vibrato around the swara
//   - andolana   : slow oscillation (swing) between the swara and a neighbor
//   - kurula     : curved slide between adjacent shrutis
//   - nokku      : grace note one shruti above the target, rapidly resolved
//   - sphurita   : bent attack, rising from below into the swara
//
// Each gamaka is parameterized by amplitude (cents) and rate (Hz / cycles).
// A "none" variant is included so call sites can carry the field without
// branching at every layer.

export type GamakaKind = 'none' | 'kampita' | 'andolana' | 'kurula' | 'nokku' | 'sphurita';

interface GamakaBase {
  kind: GamakaKind;
}

export interface GamakaNone extends GamakaBase { kind: 'none'; }

export interface GamakaKampita extends GamakaBase {
  kind: 'kampita';
  /** Vibrato width in cents. Canonical range 10–30. */
  cents: number;
  /** Vibrato rate in Hz. Canonical range 4–8. */
  rateHz: number;
}

export interface GamakaAndolana extends GamakaBase {
  kind: 'andolana';
  /** Oscillation width in cents. Canonical range 30–80. */
  cents: number;
  /** Slow oscillation rate. Canonical range 0.8–2.5 Hz. */
  rateHz: number;
}

export interface GamakaKurula extends GamakaBase {
  kind: 'kurula';
  /** Width of the slide in cents. Defaults to ~100 (one semitone-equivalent). */
  cents: number;
  /** Glide duration as fraction of the swara's hold. 0.0–1.0 */
  glideFraction: number;
}

export interface GamakaNokku extends GamakaBase {
  kind: 'nokku';
  /** Grace pitch in cents above the main swara. Canonical 80–120. */
  graceCents: number;
  /** Grace duration as fraction of the swara's hold. Canonical 0.05–0.12. */
  graceFraction: number;
}

export interface GamakaSphurita extends GamakaBase {
  kind: 'sphurita';
  /** Starting offset in cents below the swara. Canonical -40 to -80. */
  startCents: number;
  /** Attack duration to reach the swara, fraction of hold. Canonical 0.05–0.15. */
  attackFraction: number;
}

export type Gamaka =
  | GamakaNone
  | GamakaKampita
  | GamakaAndolana
  | GamakaKurula
  | GamakaNokku
  | GamakaSphurita;

/** Exhaustiveness check helper for switch statements over Gamaka.kind. */
export const assertNeverGamaka = (g: never): never => {
  throw new Error(`Unhandled gamaka kind: ${JSON.stringify(g)}`);
};

/** Default presets — the "neutral" choice for each gamaka kind. */
export const GAMAKA_DEFAULTS: { readonly [K in GamakaKind]: Extract<Gamaka, { kind: K }> } = {
  none:     { kind: 'none' },
  kampita:  { kind: 'kampita',  cents: 20, rateHz: 6 },
  andolana: { kind: 'andolana', cents: 50, rateHz: 1.5 },
  kurula:   { kind: 'kurula',   cents: 100, glideFraction: 0.4 },
  nokku:    { kind: 'nokku',    graceCents: 100, graceFraction: 0.08 },
  sphurita: { kind: 'sphurita', startCents: -60, attackFraction: 0.1 },
};
