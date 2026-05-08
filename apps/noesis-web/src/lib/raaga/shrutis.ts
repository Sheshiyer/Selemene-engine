// Canonical 22 Shrutis (Bharata–Sārṅgadeva system).
// Just-intonation ratios. Sa and Pa are achala (immovable).
// SHRUTIS[22] is Sa' (octave) — included for arohana convenience.

export interface Shruti {
  /** 0..22. Indices 0..21 are the canonical 22; index 22 is Sa' (octave). */
  idx: number;
  /** Sanskrit name (Sa, R1..R4, G1..G4, M1..M4, Pa, D1..D4, N1..N4, Sa'). */
  name: string;
  /** Just-intonation ratio as [numerator, denominator]. */
  ratio: readonly [number, number];
  /** Cents from Sa (1200 * log2(ratio)). */
  cents: number;
  /** Carnatic swara family (Sa, Re, Ga, Ma, Pa, Dha, Ni). */
  swara: 'Sa' | 'Re' | 'Ga' | 'Ma' | 'Pa' | 'Dha' | 'Ni';
}

const c = (n: number, d: number): number => 1200 * Math.log2(n / d);

export const SHRUTIS: readonly Shruti[] = [
  { idx: 0,  name: "Sa",   ratio: [1, 1],       cents: c(1, 1),       swara: 'Sa'  },
  { idx: 1,  name: "R1",   ratio: [256, 243],   cents: c(256, 243),   swara: 'Re'  },
  { idx: 2,  name: "R2",   ratio: [16, 15],     cents: c(16, 15),     swara: 'Re'  },
  { idx: 3,  name: "R3",   ratio: [10, 9],      cents: c(10, 9),      swara: 'Re'  },
  { idx: 4,  name: "R4",   ratio: [9, 8],       cents: c(9, 8),       swara: 'Re'  },
  { idx: 5,  name: "G1",   ratio: [32, 27],     cents: c(32, 27),     swara: 'Ga'  },
  { idx: 6,  name: "G2",   ratio: [6, 5],       cents: c(6, 5),       swara: 'Ga'  },
  { idx: 7,  name: "G3",   ratio: [5, 4],       cents: c(5, 4),       swara: 'Ga'  },
  { idx: 8,  name: "G4",   ratio: [81, 64],     cents: c(81, 64),     swara: 'Ga'  },
  { idx: 9,  name: "M1",   ratio: [4, 3],       cents: c(4, 3),       swara: 'Ma'  },
  { idx: 10, name: "M2",   ratio: [27, 20],     cents: c(27, 20),     swara: 'Ma'  },
  { idx: 11, name: "M3",   ratio: [45, 32],     cents: c(45, 32),     swara: 'Ma'  },
  { idx: 12, name: "M4",   ratio: [729, 512],   cents: c(729, 512),   swara: 'Ma'  },
  { idx: 13, name: "Pa",   ratio: [3, 2],       cents: c(3, 2),       swara: 'Pa'  },
  { idx: 14, name: "D1",   ratio: [128, 81],    cents: c(128, 81),    swara: 'Dha' },
  { idx: 15, name: "D2",   ratio: [8, 5],       cents: c(8, 5),       swara: 'Dha' },
  { idx: 16, name: "D3",   ratio: [5, 3],       cents: c(5, 3),       swara: 'Dha' },
  { idx: 17, name: "D4",   ratio: [27, 16],     cents: c(27, 16),     swara: 'Dha' },
  { idx: 18, name: "N1",   ratio: [16, 9],      cents: c(16, 9),      swara: 'Ni'  },
  { idx: 19, name: "N2",   ratio: [9, 5],       cents: c(9, 5),       swara: 'Ni'  },
  { idx: 20, name: "N3",   ratio: [15, 8],      cents: c(15, 8),      swara: 'Ni'  },
  { idx: 21, name: "N4",   ratio: [243, 128],   cents: c(243, 128),   swara: 'Ni'  },
  { idx: 22, name: "Sa'",  ratio: [2, 1],       cents: c(2, 1),       swara: 'Sa'  },
];

export const ratioOf = (idx: number): number => {
  const s = SHRUTIS[idx];
  return s.ratio[0] / s.ratio[1];
};

/** The 16 Carnatic swara variants → shruti index. Vivadi pairs collapse: R3≡G1, D3≡N1. */
export const CARNATIC_SWARA_TO_SHRUTI = {
  Sa: 0,
  R1: 1, R2: 4, R3: 5,
  G1: 5, G2: 6, G3: 7,
  M1: 9, M2: 12,
  Pa: 13,
  D1: 14, D2: 16, D3: 18,
  N1: 18, N2: 19, N3: 20,
  Sa_: 22,
} as const;
