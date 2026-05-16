// Algorithmic gamaka derivation for all 72 melakartas.
//
// Rather than requiring 72 hand-crafted presets (only Mayamalavagaula exists),
// this derives a sensible per-swara GamakaAnnotation[] from properties that
// *are* in the melakarta data: mood, rasa, dosha_affinity, and swara identity.
//
// Rules applied (in priority order):
//
//   1. Sa (idx 0) and Pa (idx 4) — achala swaras. Never ornamented. → none
//   2. Komal intervals (R1, G1, G2, D1, D2, N1) → kampita by default
//      (the "settling" vibrato characteristic of minor-leaning swaras)
//   3. Prati Madhyama (M2) → andolana, gently oscillating toward M1
//   4. Rasa overrides gamaka character:
//        shanta / bhakti          → andolana (deep, slow sway)
//        karuna                   → andolana (gentle, longing)
//        shringara / madhura      → kurula (flowing slides)
//        raudra / vira            → sphurita (aggressive bent attack)
//        adbhuta / hasya          → nokku (quick, surprised grace)
//   5. Dosha modifies intensity:
//        vata  → softer andolana (wider, slower — steadying)
//        pitta → kampita intensity (precise, rhythmic)
//        kapha → sphurita (activating, wake-up)
//
// This is not a substitute for hand-crafted presets but is far better than
// applying a single uniform gamaka to all swaras.

import type { GamakaAnnotation } from './apply';
import type { Gamaka } from './types';
import type { Melakarta } from '../../melakartas';
import { CARNATIC_SWARA_TO_SHRUTI as SW } from '../../shrutis';

// Shruti indices that are "komal" (minor-flavour) in the Carnatic system.
const KOMAL_INDICES: Set<number> = new Set([
  SW.R1,  // Shuddha Rishabha  256/243
  SW.G1,  // Shuddha Gandhara  32/27
  SW.G2,  // Sadharana Gandhara 6/5
  SW.D1,  // Shuddha Dhaivata  128/81
  SW.D2,  // Chatushruti Dhaivata 8/5
  SW.N1,  // Shuddha Nishada   16/9
  SW.N2,  // Kaisiki Nishada   9/5
]);

const ACHALA_INDICES: Set<number> = new Set([SW.Sa, SW.Pa, SW.Sa_]);

type Rasa =
  | 'shanta' | 'bhakti' | 'karuna' | 'shringara' | 'madhura'
  | 'raudra' | 'vira' | 'adbhuta' | 'hasya' | string;

type Dosha = 'vata' | 'pitta' | 'kapha' | string;

/** Map rasa → base gamaka for non-achala swaras. */
function rasaGamaka(rasa: Rasa): Gamaka {
  switch (rasa) {
    case 'shanta':
    case 'bhakti':
    case 'karuna':
      return { kind: 'andolana', cents: 55, rateHz: 1.4 };
    case 'shringara':
    case 'madhura':
      return { kind: 'kurula', cents: 80, glideFraction: 0.35 };
    case 'raudra':
    case 'vira':
      return { kind: 'sphurita', startCents: -55, attackFraction: 0.12 };
    case 'adbhuta':
    case 'hasya':
      return { kind: 'nokku', graceCents: 90, graceFraction: 0.09 };
    default:
      return { kind: 'kampita', cents: 20, rateHz: 5.5 };
  }
}

/** Dosha-modifies the base gamaka — returns an adjusted copy. */
function applyDoshaModifier(base: Gamaka, dosha: Dosha): Gamaka {
  if (base.kind === 'andolana') {
    if (dosha === 'vata') return { ...base, cents: 70, rateHz: 1.0 };  // wider, slower
    if (dosha === 'kapha') return { kind: 'sphurita', startCents: -45, attackFraction: 0.1 };
  }
  if (base.kind === 'kampita') {
    if (dosha === 'pitta') return { ...base, cents: 22, rateHz: 6.5 };  // crisper
    if (dosha === 'vata')   return { kind: 'andolana', cents: 50, rateHz: 1.2 };
  }
  return base;
}

/**
 * Derive per-swara GamakaAnnotation[] for a melakarta.
 *
 * @param melakarta - The melakarta (from melakartas.ts; has arohana shruti indices).
 * @param rasa      - Rasa string from melakarta_ragas.json.
 * @param dosha     - Primary dosha from melakarta_ragas.json (first element).
 */
export function deriveGamakas(
  melakarta: Melakarta,
  rasa: Rasa = 'shanta',
  dosha: Dosha = 'vata',
): readonly GamakaAnnotation[] {
  const base = rasaGamaka(rasa);
  const modded = applyDoshaModifier(base, dosha);
  const annotations: GamakaAnnotation[] = [];

  // arohana has 8 shruti indices: [Sa, R, G, M, Pa, D, N, Sa']
  melakarta.arohana.forEach((shrutiIdx, swaraIdx) => {
    // Rule 1 — achala swaras: never ornamented
    if (ACHALA_INDICES.has(shrutiIdx)) return;

    // Rule 2 — komal swaras always get kampita (lighter than the rasa base)
    if (KOMAL_INDICES.has(shrutiIdx)) {
      annotations.push({
        swaraIndex: swaraIdx,
        direction: 'both',
        gamaka: { kind: 'kampita', cents: 18, rateHz: 5 },
      });
      return;
    }

    // Rule 3 — prati madhyama (M2) gets andolana
    if (shrutiIdx === SW.M2) {
      annotations.push({
        swaraIndex: swaraIdx,
        direction: 'both',
        gamaka: { kind: 'andolana', cents: 40, rateHz: 1.8 },
      });
      return;
    }

    // Rule 4+5 — rasa+dosha derived gamaka
    annotations.push({
      swaraIndex: swaraIdx,
      direction: 'both',
      gamaka: modded,
    });
  });

  return annotations;
}
