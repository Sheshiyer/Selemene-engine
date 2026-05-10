// Mayamalavagaula (#15) — canonical Carnatic ornamentation.
//
// This raga is the *first* learned in the Carnatic tradition because its
// gamakas are simple enough to teach but characteristic enough to demonstrate
// every microtonal nuance. The ornamentation below follows the standard
// Pandanallur–Tanjore conventions:
//   - Re1 (komal Re) is held with light kampita
//   - Ga3 (antara Ga) gets sphurita on attack from the previous Re
//   - Ma1 has a slow andolana toward Ga3
//   - Dha1 (komal Dha) is held with kampita matching Re1
//   - Ni3 (kakali Ni) gets a nokku from the upper Sa'

import type { GamakaAnnotation } from '../gamakas/apply';
import { GAMAKA_DEFAULTS } from '../gamakas/types';

export const MAYAMALAVAGAULA_GAMAKAS: readonly GamakaAnnotation[] = [
  // index 0 = Sa, plain
  { swaraIndex: 1, direction: 'both', gamaka: { ...GAMAKA_DEFAULTS.kampita,  cents: 15, rateHz: 5 } },  // Re1
  { swaraIndex: 2, direction: 'up',   gamaka: { ...GAMAKA_DEFAULTS.sphurita, startCents: -50, attackFraction: 0.12 } },  // Ga3 (ascending)
  { swaraIndex: 2, direction: 'down', gamaka: { ...GAMAKA_DEFAULTS.kampita,  cents: 12, rateHz: 6 } },  // Ga3 (descending)
  { swaraIndex: 3, direction: 'both', gamaka: { ...GAMAKA_DEFAULTS.andolana, cents: 40, rateHz: 1.8 } }, // Ma1
  // index 4 = Pa, plain (achala — never ornamented strongly)
  { swaraIndex: 5, direction: 'both', gamaka: { ...GAMAKA_DEFAULTS.kampita,  cents: 15, rateHz: 5 } },  // Dha1
  { swaraIndex: 6, direction: 'down', gamaka: { ...GAMAKA_DEFAULTS.nokku,    graceCents: 90, graceFraction: 0.1 } },  // Ni3 in descent
  // index 7 = Sa', plain
];
