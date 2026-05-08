// 72 Melakartas: arohana / avarohana as shruti-index sequences.
// Swaras derived algorithmically from the canonical Venkatamakhin formula:
//   - n in [1..36]  → Ma1 (4/3)        ; n in [37..72] → Ma2 (729/512)
//   - chakra-in-ma-set = floor((localN-1)/6)  → (R, G) pair
//   - position-in-chakra = (localN-1) % 6     → (D, N) pair
// 6 R-G pairs × 6 D-N pairs × 2 Ma-types = 72.

import { CARNATIC_SWARA_TO_SHRUTI as SW } from './shrutis';

export interface Melakarta {
  num: number;
  name: string;
  /** Shruti indices (8 entries: Sa, R, G, M, Pa, D, N, Sa'). */
  arohana: readonly number[];
  /** Descending; same set, reverse order. */
  avarohana: readonly number[];
  /** 1..12. Maps to body chakra (Indu..Aditya). */
  chakra: number;
}

const RG_PAIRS = [
  [SW.R1, SW.G1], // chakra 0
  [SW.R1, SW.G2], // chakra 1
  [SW.R1, SW.G3], // chakra 2
  [SW.R2, SW.G2], // chakra 3
  [SW.R2, SW.G3], // chakra 4
  [SW.R3, SW.G3], // chakra 5
] as const;

const DN_PAIRS = [
  [SW.D1, SW.N1], // pos 0
  [SW.D1, SW.N2], // pos 1
  [SW.D1, SW.N3], // pos 2
  [SW.D2, SW.N2], // pos 3
  [SW.D2, SW.N3], // pos 4
  [SW.D3, SW.N3], // pos 5
] as const;

// Canonical names from Venkatamakhin's Chaturdandi Prakashika ordering.
const NAMES: readonly string[] = [
  // Shuddha Ma (1–36)
  "Kanakangi", "Ratnangi", "Ganamurti", "Vanaspati", "Manavati", "Tanarupi",
  "Senapati", "Hanumatodi", "Dhenuka", "Natakapriya", "Kokilapriya", "Rupavati",
  "Gayakapriya", "Vakulabharanam", "Mayamalavagaula", "Chakravakam", "Suryakantam", "Hatakambari",
  "Jhankaradhvani", "Natabhairavi", "Keeravani", "Kharaharapriya", "Gourimanohari", "Varunapriya",
  "Mararanjani", "Charukesi", "Sarasangi", "Harikambhoji", "Dheerasankarabharanam", "Naganandini",
  "Yagapriya", "Ragavardhini", "Gangeyabhushani", "Vagadhishvari", "Shulini", "Chalanata",
  // Prati Ma (37–72)
  "Salagam", "Jalarnavam", "Jhalavarali", "Navaneetam", "Pavani", "Raghupriya",
  "Gavambodhi", "Bhavapriya", "Subhapantuvarali", "Shadvidhamargini", "Suvarnangi", "Divyamani",
  "Dhavalambari", "Namanarayani", "Kamavardhini", "Ramapriya", "Gamanasrama", "Vishvambhari",
  "Shyamalangi", "Shanmukhapriya", "Simhendramadhyamam", "Hemavati", "Dharmavati", "Neetimati",
  "Kantamani", "Rishabhapriya", "Latangi", "Vachaspati", "Mechakalyani", "Chitrambari",
  "Sucharitra", "Jyotisvarupini", "Dhatuvardhini", "Nasikabhushani", "Kosalam", "Rasikapriya",
];

export function generateMelakarta(num: number): Melakarta {
  if (num < 1 || num > 72) throw new RangeError(`Melakarta number must be 1..72 (got ${num})`);
  const isPratiMa = num > 36;
  const localN = isPratiMa ? num - 36 : num;
  const chakraInSet = Math.floor((localN - 1) / 6);  // 0..5
  const posInChakra = (localN - 1) % 6;              // 0..5

  const [r, g] = RG_PAIRS[chakraInSet];
  const [d, n] = DN_PAIRS[posInChakra];
  const m = isPratiMa ? SW.M2 : SW.M1;

  const arohana = [SW.Sa, r, g, m, SW.Pa, d, n, SW.Sa_];
  return {
    num,
    name: NAMES[num - 1],
    arohana,
    avarohana: [...arohana].reverse(),
    chakra: Math.floor((num - 1) / 6) + 1,  // 1..12 mapping to body chakra
  };
}

export const MELAKARTAS: readonly Melakarta[] = Array.from({ length: 72 }, (_, i) => generateMelakarta(i + 1));

export const getMelakarta = (num: number): Melakarta | undefined =>
  MELAKARTAS.find((m) => m.num === num);
