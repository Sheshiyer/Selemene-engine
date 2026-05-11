// Six canonical Carnatic talas. Each entry is verifiable against any
// standard reference (e.g., Subbarama Dikshitar's Sangita Sampradaya
// Pradarshini); the structure must always sum to `beats`.

import type { Tala, TalaName } from './types';

export const TALAS: { readonly [K in TalaName]: Tala } = {
  'adi': {
    name: 'adi',
    display: 'Ādi',
    beats: 8,
    structure: [4, 2, 2],
    euclid: [3, 8],
    accentBeats: [0, 4, 6],
    description: 'The most common tala — 8 beats divided 4+2+2. Caturasra-jati Triputa.',
  },
  'rupakam': {
    name: 'rupakam',
    display: 'Rūpakam',
    beats: 6,
    structure: [2, 4],
    euclid: [2, 6],
    accentBeats: [0, 2],
    description: '6 beats: 2 (drutam) + 4 (laghu). Common for varnams.',
  },
  'misra-chapu': {
    name: 'misra-chapu',
    display: 'Miśra Chāpu',
    beats: 7,
    structure: [3, 4],
    euclid: [2, 7],
    accentBeats: [0, 3],
    description: '7 beats: 3 + 4. Distinctive folk-derived limp; common in padams.',
  },
  'khanda-chapu': {
    name: 'khanda-chapu',
    display: 'Khaṇḍa Chāpu',
    beats: 5,
    structure: [2, 3],
    euclid: [2, 5],
    accentBeats: [0, 2],
    description: '5 beats: 2 + 3. Faster cousin of Misra Chapu.',
  },
  'tisra-eka': {
    name: 'tisra-eka',
    display: 'Tisra Ēkam',
    beats: 3,
    structure: [3],
    euclid: [1, 3],
    accentBeats: [0],
    description: '3 beats. The simplest tala — meditative, waltz-like.',
  },
  'jhampa': {
    name: 'jhampa',
    display: 'Miśra Jhampa',
    beats: 10,
    structure: [7, 1, 2],
    euclid: [3, 10],
    accentBeats: [0, 7, 8],
    description: '10 beats: 7 (laghu) + 1 (anudrutam) + 2 (drutam). Architectural cycle.',
  },
};

/** Default tala for unknown raga contexts. */
export const DEFAULT_TALA: Tala = TALAS['adi'];

export const ALL_TALAS: readonly Tala[] = Object.values(TALAS);
