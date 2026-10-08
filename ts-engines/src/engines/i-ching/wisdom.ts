/**
 * I-Ching Wisdom Data
 * King Wen line mapping, with explicitly identified legacy text coverage.
 */

import leggeWisdom from './legge-wisdom.json'

export type HexagramLines = [boolean, boolean, boolean, boolean, boolean, boolean]

export interface Hexagram {
  number: number
  name: string
  chineseName: string | null
  meaning: string | null
  judgment: string | null
  image: string | null
  lines: HexagramLines // bottom to top, true = yang
  wisdomStatus: 'legacy_unverified' | 'unavailable' | 'source_transcribed'
  fieldStatus: Record<string, string>
  provenance: Record<string, unknown>
}

// Preserve legacy labels/summaries without claiming a verified edition or full corpus.
const LEGACY_HEXAGRAMS: Pick<Hexagram, 'number' | 'name' | 'chineseName' | 'meaning'>[] = [
  {
    number: 1,
    name: 'The Creative',
    chineseName: '乾 (Qián)',
    meaning: 'Pure yang energy, heaven, creative power, strong action',
  },
  {
    number: 2,
    name: 'The Receptive',
    chineseName: '坤 (Kūn)',
    meaning: 'Pure yin energy, earth, receptive power, nurturing devotion',
  },
  {
    number: 3,
    name: 'Difficulty at the Beginning',
    chineseName: '屯 (Zhūn)',
    meaning: 'Initial difficulties, birth pangs, gathering resources',
  },
  {
    number: 4,
    name: 'Youthful Folly',
    chineseName: '蒙 (Méng)',
    meaning: 'Inexperience, learning, seeking guidance',
  },
  {
    number: 5,
    name: 'Waiting',
    chineseName: '需 (Xū)',
    meaning: 'Patient waiting, nourishment, trust in timing',
  },
  {
    number: 6,
    name: 'Conflict',
    chineseName: '訟 (Sòng)',
    meaning: 'Dispute, opposition, seeking resolution',
  },
  {
    number: 7,
    name: 'The Army',
    chineseName: '師 (Shī)',
    meaning: 'Organized force, discipline, leadership',
  },
  {
    number: 8,
    name: 'Holding Together',
    chineseName: '比 (Bǐ)',
    meaning: 'Union, seeking connection, forming alliances',
  },
]

// Factual glyph mapping independently checked for all 64 entries against:
// https://www.unicode.org/charts/PDF/U4DC0.pdf (glyph chart, U+4DC0..U+4DFF).
// King Wen ordering is also shown by Legge's Plate I. No chart/font assets copied.
// Trigrams and six-line strings are BOTTOM TO TOP. Matrix rows = upper trigram,
// columns = lower trigram: heaven, lake, fire, thunder, wind, water, mountain, earth.
const TRIGRAMS = ['111', '110', '101', '100', '011', '010', '001', '000'] as const
const KING_WEN_MATRIX = [
  [1, 10, 13, 25, 44, 6, 33, 12],
  [43, 58, 49, 17, 28, 47, 31, 45],
  [14, 38, 30, 21, 50, 64, 56, 35],
  [34, 54, 55, 51, 32, 40, 62, 16],
  [9, 61, 37, 42, 57, 59, 53, 20],
  [5, 60, 63, 3, 48, 29, 39, 8],
  [26, 41, 22, 27, 18, 4, 52, 23],
  [11, 19, 36, 24, 46, 7, 15, 2],
] as const

const NUMBER_BY_PATTERN = new Map<string, number>()
const LINES_BY_NUMBER = new Map<number, HexagramLines>()
for (let upper = 0; upper < 8; upper++) {
  for (let lower = 0; lower < 8; lower++) {
    const number = KING_WEN_MATRIX[upper][lower]
    const pattern = TRIGRAMS[lower] + TRIGRAMS[upper]
    NUMBER_BY_PATTERN.set(pattern, number)
    LINES_BY_NUMBER.set(number, [...pattern].map((line) => line === '1') as HexagramLines)
  }
}

// The source artifact is scan-reviewed normalized transcription, not a
// character-perfect edition. Preserve its passage pages, notes and uncertainty.
const LEGGE_BY_NUMBER = new Map(leggeWisdom.hexagrams.map((entry) => [entry.number, entry]))
if (
  leggeWisdom.hexagrams.length !== 64 ||
  LEGGE_BY_NUMBER.size !== 64 ||
  leggeWisdom.hexagrams.some(
    (entry) =>
      !Number.isInteger(entry.number) ||
      entry.number < 1 ||
      entry.number > 64 ||
      !entry.judgment.trim() ||
      !entry.image.trim() ||
      !Number.isInteger(entry.provenance.judgmentPage) ||
      !Number.isInteger(entry.provenance.imagePage),
  )
) {
  throw new Error('Bundled Legge transcription must contain 64 distinct, sourced hexagrams.')
}

export function linesToHexagramNumber(lines: readonly boolean[]): number {
  if (lines.length !== 6 || lines.some((line) => typeof line !== 'boolean')) {
    throw new RangeError('A hexagram requires six boolean lines, ordered bottom to top.')
  }
  const number = NUMBER_BY_PATTERN.get(lines.map((line) => (line ? '1' : '0')).join(''))
  if (number === undefined) throw new RangeError('Unknown hexagram line pattern.')
  return number
}

export const HEXAGRAMS: Hexagram[] = Array.from({ length: 64 }, (_, index) => {
  const number = index + 1
  const legacy = LEGACY_HEXAGRAMS.find((hexagram) => hexagram.number === number)
  const source = LEGGE_BY_NUMBER.get(number)
  if (!source) throw new Error('Bundled Legge transcription is missing a hexagram.')
  const status = legacy ? 'legacy_unverified' : 'unavailable'
  return {
    number,
    name: legacy?.name ?? `Hexagram ${number}`,
    chineseName: legacy?.chineseName ?? null,
    meaning: legacy?.meaning ?? null,
    judgment: source.judgment,
    image: source.image,
    lines: [...(LINES_BY_NUMBER.get(number) as HexagramLines)],
    wisdomStatus: 'source_transcribed',
    fieldStatus: {
      name: legacy ? 'legacy_unverified' : 'generic_identifier',
      chinese_name: status,
      meaning: status,
      judgment: 'source_transcribed',
      image: 'source_transcribed',
      lines: 'verified_mapping',
    },
    provenance: {
      line_mapping: {
        sequence: 'King Wen',
        line_order: 'bottom_to_top',
        source: 'https://www.unicode.org/charts/PDF/U4DC0.pdf',
      },
      legacy_labels_and_meanings: legacy
        ? 'Edition and attribution not verified.'
        : 'Unavailable; the generic name is a display identifier only.',
      text: {
        edition: leggeWisdom.metadata,
        passages: source.provenance,
        transcription_status: source.transcriptionStatus,
        transcription_notes: [...source.transcriptionNotes],
      },
      changing_line_texts: 'unavailable',
    },
  }
})

/**
 * Get a hexagram by number (1-64)
 */
export function getHexagramByNumber(num: number): Hexagram | undefined {
  return HEXAGRAMS.find((h) => h.number === num)
}
