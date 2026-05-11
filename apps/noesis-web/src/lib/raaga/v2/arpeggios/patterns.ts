// Six canonical paltas. Each generates a sequence of swara indices given the
// raga's 7 swaras (+ upper Sa as index 7).
//
// Convention: swaras = [Sa, R, G, M, Pa, D, N, Sa']  → indices 0..7

import type { Palta } from './types';

/**
 * Sarali Varisai — the first set every Carnatic student learns.
 * Sequence (over a 7-swara raga): SRGM RGMP GMPD MPDN PDNS PDNSpa-down ...
 * We render the canonical 5 ascending phrases of 4 notes each:
 *   S R G M | R G M P | G M P D | M P D N | P D N Sa'
 * Then a graceful descending unwind:
 *   Sa' N D P | N D P M | D P M G | P M G R | M G R S
 * Total: 40 notes.
 */
const sarali: Palta = {
  name: 'sarali',
  display: 'Sarali Varisai',
  description: '4-note ascending paired patterns. The first lesson in Carnatic music.',
  generate: (s) => {
    if (s.length < 8) throw new Error('sarali needs 8 swaras [Sa..Sa\']');
    const phrasesUp = [];
    for (let start = 0; start <= 4; start++) {
      phrasesUp.push(s[start], s[start + 1], s[start + 2], s[start + 3]);
    }
    const phrasesDown = [];
    for (let start = 7; start >= 3; start--) {
      phrasesDown.push(s[start], s[start - 1], s[start - 2], s[start - 3]);
    }
    return [...phrasesUp, ...phrasesDown];
  },
};

/**
 * Jantai Varisai — paired-repeat patterns. Each swara struck twice.
 *   S S R R G G M M | R R G G M M P P | ...
 * Renders 5 ascending paired phrases:
 *   2(S R G M) | 2(R G M P) | ... | 2(P D N S')
 * Total: 40 notes.
 */
const jantai: Palta = {
  name: 'jantai',
  display: 'Jantai Varisai',
  description: 'Paired-repeat patterns — each swara struck twice for emphasis.',
  generate: (s) => {
    const out: number[] = [];
    for (let start = 0; start <= 4; start++) {
      for (let i = 0; i < 4; i++) {
        out.push(s[start + i], s[start + i]);  // pair-repeat
      }
    }
    return out;
  },
};

/**
 * Dattu Varisai — alternating-skip patterns. Two-note skipping figures.
 *   S G | R M | G P | M D | P N | D S' (ascending pairs separated by skips)
 * Then descending:
 *   S' D | N P | D M | P G | M R | G S
 * Total: 24 notes.
 */
const dattu: Palta = {
  name: 'dattu',
  display: 'Dattu Varisai',
  description: 'Alternating-skip patterns — two-note figures with characteristic gaps.',
  generate: (s) => {
    const out: number[] = [];
    for (let start = 0; start <= 5; start++) {
      out.push(s[start], s[start + 2]);  // skip 1
    }
    for (let start = 7; start >= 2; start--) {
      out.push(s[start], s[start - 2]);
    }
    return out;
  },
};

/**
 * Tara Sthayi Varisai — patterns reaching the upper Sa'.
 * Each swara paired with the upper Sa':
 *   S Sa' | R Sa' | G Sa' | M Sa' | P Sa' | D Sa' | N Sa'
 * Then unwound: Sa' D N P D M ... back to Sa.
 * Total: 21 notes.
 */
const tara: Palta = {
  name: 'tara',
  display: 'Tara Sthayi Varisai',
  description: 'Each swara paired with the upper Sa — reaches for the high tonic.',
  generate: (s) => {
    const out: number[] = [];
    for (let i = 0; i < 7; i++) {
      out.push(s[i], s[7]);  // pair every swara with Sa'
    }
    // Graceful descent to anchor
    out.push(s[7], s[6], s[5], s[4], s[3], s[2], s[1], s[0]);
    return out;
  },
};

/**
 * Mel-Sthayi — patterns oscillating between the middle and upper registers.
 * Climbs in 3-note groups using the upper Sa' as anchor.
 *   S G P | R M D | G P N | M D S' | P N S' | D S' Sa' (octaved)
 */
const melSthayi: Palta = {
  name: 'mel-sthayi',
  display: 'Mel-Sthayi',
  description: 'Three-note climbing groups with the upper Sa as anchor.',
  generate: (s) => {
    const out: number[] = [];
    // 3-note ascending triples with skip of 2
    for (let start = 0; start <= 4; start++) {
      out.push(s[start], s[start + 2], s[start + 4]);
    }
    // Resolve to upper Sa' and back down
    out.push(s[7], s[6], s[5], s[4], s[3], s[2], s[1], s[0]);
    return out;
  },
};

/**
 * Alankara (Adi tala) — 8-beat decorative figure.
 *   S R G | R G M | G M P | M P D | ... (3-note ascending with overlap)
 */
const alankara: Palta = {
  name: 'alankara',
  display: 'Alankāra',
  description: '3-note ascending overlapping figures — decorative practice.',
  generate: (s) => {
    const out: number[] = [];
    for (let start = 0; start <= 5; start++) {
      out.push(s[start], s[start + 1], s[start + 2]);
    }
    // Mirror descent
    for (let start = 7; start >= 2; start--) {
      out.push(s[start], s[start - 1], s[start - 2]);
    }
    return out;
  },
};

export const PALTAS: { readonly [K in import('./types').PaltaName]: Palta } = {
  'sarali': sarali,
  'jantai': jantai,
  'dattu': dattu,
  'tara': tara,
  'mel-sthayi': melSthayi,
  'alankara': alankara,
};

export const ALL_PALTAS: readonly Palta[] = Object.values(PALTAS);
