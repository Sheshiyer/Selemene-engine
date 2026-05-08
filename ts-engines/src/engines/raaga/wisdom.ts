/**
 * Raaga Engine — melakarta wisdom data
 *
 * 72 Melakarta ragas derived algorithmically from the Venkatamakhin / Katapayadi
 * system (Chaturdandi Prakashika ordering). Swaras expressed as just-intonation
 * ratios over the 22-shruti scale of Bharata–Sārṅgadeva.
 *
 * Mirrors the front-end shrutis.ts + melakartas.ts libs so backend calculations
 * stay numerically identical to what the Strudel player renders.
 */

// ---------------------------------------------------------------------------
// 22 Shrutis (+ Sa' as index 22)
// ---------------------------------------------------------------------------

/** [numerator, denominator] just-intonation ratio */
type Ratio = readonly [number, number]

const SHRUTIS: readonly Ratio[] = [
  [1, 1],       // 0  Sa
  [256, 243],   // 1  R1
  [16, 15],     // 2  R2
  [10, 9],      // 3  R3
  [9, 8],       // 4  R4 / G1 (vivadi alias)
  [32, 27],     // 5  G1 / R3-alt
  [6, 5],       // 6  G2
  [5, 4],       // 7  G3
  [81, 64],     // 8  G4
  [4, 3],       // 9  M1
  [27, 20],     // 10 M2-alt
  [45, 32],     // 11 M2-alt2
  [729, 512],   // 12 M2 (tivra Ma)
  [3, 2],       // 13 Pa
  [128, 81],    // 14 D1
  [8, 5],       // 15 D2
  [5, 3],       // 16 D3 / N1-alt
  [27, 16],     // 17 D4
  [16, 9],      // 18 N1 / D3 (vivadi alias)
  [9, 5],       // 19 N2
  [15, 8],      // 20 N3
  [243, 128],   // 21 N4
  [2, 1],       // 22 Sa' (octave)
]

export function ratioOf(idx: number): number {
  const r = SHRUTIS[idx]
  if (!r) throw new Error(`Shruti index out of range: ${idx}`)
  return r[0] / r[1]
}

// Carnatic swara → shruti index mapping
const SW = {
  Sa: 0,
  R1: 1,  R2: 4,  R3: 5,
  G1: 5,  G2: 6,  G3: 7,
  M1: 9,  M2: 12,
  Pa: 13,
  D1: 14, D2: 16, D3: 18,
  N1: 18, N2: 19, N3: 20,
  Sa_: 22,
} as const

// ---------------------------------------------------------------------------
// 72 Melakarta generator (Venkatamakhin formula)
// ---------------------------------------------------------------------------

const RG_PAIRS: readonly [number, number][] = [
  [SW.R1, SW.G1], // chakra-set 0
  [SW.R1, SW.G2], // chakra-set 1
  [SW.R1, SW.G3], // chakra-set 2
  [SW.R2, SW.G2], // chakra-set 3
  [SW.R2, SW.G3], // chakra-set 4
  [SW.R3, SW.G3], // chakra-set 5
]

const DN_PAIRS: readonly [number, number][] = [
  [SW.D1, SW.N1], // pos 0
  [SW.D1, SW.N2], // pos 1
  [SW.D1, SW.N3], // pos 2
  [SW.D2, SW.N2], // pos 3
  [SW.D2, SW.N3], // pos 4
  [SW.D3, SW.N3], // pos 5
]

/** Canonical names from Venkatamakhin's Chaturdandi Prakashika ordering. */
const NAMES: readonly string[] = [
  'Kanakangi', 'Ratnangi', 'Ganamurthi', 'Vanaspathi', 'Manavathi', 'Tanarupi',
  'Senavathi', 'Hanumatodi', 'Dhenuka', 'Natakapriya', 'Kokilapriya', 'Rupavathi',
  'Gayakapriya', 'Vakulabharanam', 'Mayamalavagaula', 'Chakravakam', 'Suryakantam', 'Hatakambari',
  'Jhankaradhwani', 'Natabhairavi', 'Keeravani', 'Kharaharapriya', 'Gourimanohari', 'Varunapriya',
  'Mararanjani', 'Charukesi', 'Sarasangi', 'Harikambhoji', 'Dheerasankarabharanam', 'Naganandini',
  'Yagapriya', 'Ragavardhini', 'Gangeyabhushani', 'Vagadheeswari', 'Shulini', 'Chalanata',
  'Salagam', 'Jalarnavam', 'Jhalavarali', 'Navaneetam', 'Pavani', 'Raghupriya',
  'Gavambodhi', 'Bhavapriya', 'Shubhapantuvarali', 'Shadvidamargini', 'Suvarnangi', 'Divyamani',
  'Dhavalambari', 'Namanarayani', 'Kamavardhini', 'Ramapriya', 'Gamanashrama', 'Viswambhari',
  'Shyamalangi', 'Shanmukhapriya', 'Simhendramadhyamam', 'Hemavathi', 'Dharmavathi', 'Neetimathi',
  'Kantamani', 'Rishabhapriya', 'Latangi', 'Vachaspathi', 'Mechakalyani', 'Chitrambari',
  'Sucharitra', 'Jyotiswarupini', 'Dhatuvardhini', 'Nasikabhushani', 'Kosalam', 'Rasikapriya',
]

export interface Melakarta {
  num: number            // 1..72
  name: string
  chakra: number         // 1..12 (indu=1..aditya=12)
  arohana: readonly number[]   // shruti indices: Sa R G M Pa D N Sa'
  avarohana: readonly number[] // descending (same set, reversed Sa' N D Pa M G R Sa)
  ratios: readonly number[]    // just-intonation ratios for arohana
  ma_type: 'shuddha' | 'prati' // Ma1 vs Ma2
}

function buildMelakarta(num: number): Melakarta {
  const isPratiMa = num > 36
  const localN = isPratiMa ? num - 36 : num
  const chakraInSet = Math.floor((localN - 1) / 6)
  const posInChakra = (localN - 1) % 6

  const [rg_r, rg_g] = RG_PAIRS[chakraInSet]
  const [dn_d, dn_n] = DN_PAIRS[posInChakra]
  const ma = isPratiMa ? SW.M2 : SW.M1

  const arohana = [SW.Sa, rg_r, rg_g, ma, SW.Pa, dn_d, dn_n, SW.Sa_] as const
  // Descend: Sa' N D Pa M G R Sa (reverse, drop duplicate octave-Sa)
  const avarohana = [...arohana].reverse() as readonly number[]

  return {
    num,
    name: NAMES[num - 1],
    chakra: chakraInSet + (isPratiMa ? 7 : 1),
    arohana,
    avarohana,
    ratios: arohana.map(ratioOf),
    ma_type: isPratiMa ? 'prati' : 'shuddha',
  }
}

export const MELAKARTAS: readonly Melakarta[] = Array.from({ length: 72 }, (_, i) =>
  buildMelakarta(i + 1),
)

export function getMelakarta(num: number): Melakarta | undefined {
  if (num < 1 || num > 72) return undefined
  return MELAKARTAS[num - 1]
}

export function findMelakartaByName(query: string): Melakarta | undefined {
  const norm = query.toLowerCase().replace(/[^a-z]/g, '')
  if (!norm) return undefined
  return MELAKARTAS.find((m) => m.name.toLowerCase().replace(/[^a-z]/g, '').includes(norm))
}

// ---------------------------------------------------------------------------
// Dosha–melakarta affinity table (Ayurvedic therapeutic mapping)
// ---------------------------------------------------------------------------

export type Dosha = 'vata' | 'pitta' | 'kapha'

const DOSHA_AFFINITY: Record<Dosha, readonly number[]> = {
  // Vata: calming, grounding ragas — Ma2 / minor thirds / smooth transitions
  vata: [15, 22, 8, 29, 65, 36, 20, 28],
  // Pitta: cooling, moonlit ragas — Ma1 / soothing Ga
  pitta: [22, 28, 48, 14, 20, 27, 57, 29],
  // Kapha: stimulating, solar ragas — sharp intervals, energising
  kapha: [65, 36, 29, 63, 64, 28, 15, 22],
}

export function getRaagasForDosha(dosha: Dosha): readonly Melakarta[] {
  return DOSHA_AFFINITY[dosha].map((n) => MELAKARTAS[n - 1])
}

// ---------------------------------------------------------------------------
// Time-of-day (prahar) mapping  [8 prahars, 3h each]
// ---------------------------------------------------------------------------

export interface PraharInfo {
  prahar: number   // 1..8
  label: string
  start_hour: number
  recommended: readonly number[] // melakarta nums
}

export const PRAHARS: readonly PraharInfo[] = [
  { prahar: 1, label: 'Sunrise',    start_hour: 6,  recommended: [15, 29, 8]  },
  { prahar: 2, label: 'Morning',    start_hour: 9,  recommended: [28, 29, 22] },
  { prahar: 3, label: 'Midday',     start_hour: 12, recommended: [65, 64, 36] },
  { prahar: 4, label: 'Afternoon',  start_hour: 15, recommended: [63, 28, 22] },
  { prahar: 5, label: 'Evening',    start_hour: 18, recommended: [20, 22, 8]  },
  { prahar: 6, label: 'Dusk',       start_hour: 21, recommended: [22, 20, 48] },
  { prahar: 7, label: 'Night',      start_hour: 0,  recommended: [8, 20, 14]  },
  { prahar: 8, label: 'Pre-dawn',   start_hour: 3,  recommended: [15, 8, 14]  },
]

export function getPraharForHour(hour: number): PraharInfo {
  const order = [6, 9, 12, 15, 18, 21, 0, 3]
  for (let i = order.length - 1; i >= 0; i--) {
    const start = order[i]
    // Wrap around midnight
    if (start <= hour || (start > 21 && hour < 6)) {
      return PRAHARS[i]
    }
  }
  return PRAHARS[0]
}
