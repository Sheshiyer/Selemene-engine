// Gamaka renderers — each function takes a swara's central Hz + hold weight
// and emits a Strudel mini-notation fragment that approximates the ornament.
//
// Why mini-notation expansion instead of `vib()` / `penv()`?
// `@strudel/web@1.3.0` doesn't reliably expose those at the top-level scope
// we verified. Mini-notation `[h1 h2 h3]@weight` is bulletproof — every Hz
// is a discrete event at exact just-intonation, no extra DSP layer to fail.

import type {
  Gamaka,
  GamakaKampita,
  GamakaAndolana,
  GamakaKurula,
  GamakaNokku,
  GamakaSphurita,
} from './types';

const cents = (hz: number, c: number): number => hz * Math.pow(2, c / 1200);
const fmt = (hz: number): string => hz.toFixed(3);

/** Cubic Hermite ease — smoother than linear, used for kurula glides. */
const ease = (x: number): number => x * x * (3 - 2 * x);

/** kampita — light vibrato. Emits 8 micro-events oscillating ±cents. */
export const renderKampita = (hz: number, g: GamakaKampita): string => {
  // 8 events across the swara's hold: sin wave samples
  const samples = 8;
  const points: string[] = [];
  for (let i = 0; i < samples; i++) {
    const phase = (i / samples) * Math.PI * 2;
    const offset = g.cents * Math.sin(phase);
    points.push(fmt(cents(hz, offset)));
  }
  return `[${points.join(' ')}]`;
};

/** andolana — slow oscillation. Wider, fewer points, slower. */
export const renderAndolana = (hz: number, g: GamakaAndolana): string => {
  const samples = 4;
  const points: string[] = [];
  for (let i = 0; i < samples; i++) {
    const phase = (i / samples) * Math.PI * 2;
    const offset = g.cents * Math.sin(phase);
    points.push(fmt(cents(hz, offset)));
  }
  return `[${points.join(' ')}]`;
};

/** kurula — curved glide from neighbor into the target swara. */
export const renderKurula = (hz: number, g: GamakaKurula): string => {
  const startHz = cents(hz, -g.cents);  // start a "shruti" below
  const glideSteps = 5;
  const points: string[] = [];
  for (let i = 0; i < glideSteps; i++) {
    const t = ease(i / (glideSteps - 1));
    const c = -g.cents * (1 - t);
    points.push(fmt(cents(hz, c)));
  }
  // glideFraction of the beat is the slide; the rest is sustain
  const glideWeight = Math.max(0.1, Math.min(0.9, g.glideFraction));
  const sustainWeight = 1 - glideWeight;
  return `[[${points.join(' ')}]@${glideWeight} ${fmt(hz)}@${sustainWeight}]`;
};

/** nokku — grace note above target, rapidly resolved. */
export const renderNokku = (hz: number, g: GamakaNokku): string => {
  const graceHz = cents(hz, g.graceCents);
  const graceWeight = Math.max(0.03, Math.min(0.3, g.graceFraction));
  const sustainWeight = 1 - graceWeight;
  return `[${fmt(graceHz)}@${graceWeight} ${fmt(hz)}@${sustainWeight}]`;
};

/** sphurita — bent rising attack. */
export const renderSphurita = (hz: number, g: GamakaSphurita): string => {
  const attackSteps = 4;
  const points: string[] = [];
  for (let i = 0; i < attackSteps; i++) {
    const t = i / (attackSteps - 1);
    // exponential ease: faster start, slower landing
    const eased = 1 - Math.pow(1 - t, 2);
    const c = g.startCents * (1 - eased);
    points.push(fmt(cents(hz, c)));
  }
  const attackWeight = Math.max(0.05, Math.min(0.4, g.attackFraction));
  const sustainWeight = 1 - attackWeight;
  return `[[${points.join(' ')}]@${attackWeight} ${fmt(hz)}@${sustainWeight}]`;
};

/** Plain swara — no ornament, just the Hz. */
export const renderPlain = (hz: number): string => fmt(hz);

/** Dispatch by gamaka kind. Returns Strudel mini-notation fragment. */
export const renderGamaka = (hz: number, g: Gamaka): string => {
  switch (g.kind) {
    case 'none':     return renderPlain(hz);
    case 'kampita':  return renderKampita(hz, g);
    case 'andolana': return renderAndolana(hz, g);
    case 'kurula':   return renderKurula(hz, g);
    case 'nokku':    return renderNokku(hz, g);
    case 'sphurita': return renderSphurita(hz, g);
  }
};
