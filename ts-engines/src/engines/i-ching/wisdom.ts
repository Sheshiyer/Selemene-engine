/**
 * I-Ching Wisdom Data
 * Contains the 64 hexagrams with meanings and symbolism
 */

export interface Hexagram {
  number: number
  name: string
  chineseName: string
  meaning: string
  judgment: string
  image: string
  lines: [boolean, boolean, boolean, boolean, boolean, boolean] // bottom to top, true = yang
}

// First 8 hexagrams with full data, rest are stub data
export const HEXAGRAMS: Hexagram[] = [
  {
    number: 1,
    name: 'The Creative',
    chineseName: '乾 (Qián)',
    meaning: 'Pure yang energy, heaven, creative power, strong action',
    judgment:
      'The Creative works sublime success, furthering through perseverance.',
    image:
      'The movement of heaven is full of power. Thus the superior person makes themselves strong and untiring.',
    lines: [true, true, true, true, true, true],
  },
  {
    number: 2,
    name: 'The Receptive',
    chineseName: '坤 (Kūn)',
    meaning: 'Pure yin energy, earth, receptive power, nurturing devotion',
    judgment:
      'The Receptive brings about sublime success, furthering through the perseverance of a mare.',
    image:
      "The earth's condition is receptive devotion. Thus the superior person who has breadth of character carries the outer world.",
    lines: [false, false, false, false, false, false],
  },
  {
    number: 3,
    name: 'Difficulty at the Beginning',
    chineseName: '屯 (Zhūn)',
    meaning: 'Initial difficulties, birth pangs, gathering resources',
    judgment:
      'Difficulty at the Beginning works supreme success, furthering through perseverance.',
    image: 'Clouds and thunder: the image of Difficulty at the Beginning.',
    lines: [true, false, false, false, true, false],
  },
  {
    number: 4,
    name: 'Youthful Folly',
    chineseName: '蒙 (Méng)',
    meaning: 'Inexperience, learning, seeking guidance',
    judgment:
      'Youthful Folly has success. It is not I who seek the young fool; the young fool seeks me.',
    image: 'A spring wells up at the foot of the mountain: the image of Youth.',
    lines: [false, true, false, false, false, true],
  },
  {
    number: 5,
    name: 'Waiting',
    chineseName: '需 (Xū)',
    meaning: 'Patient waiting, nourishment, trust in timing',
    judgment:
      'Waiting. If you are sincere, you have light and success. Perseverance brings good fortune.',
    image: 'Clouds rise up to heaven: the image of Waiting.',
    lines: [true, true, true, false, true, false],
  },
  {
    number: 6,
    name: 'Conflict',
    chineseName: '訟 (Sòng)',
    meaning: 'Dispute, opposition, seeking resolution',
    judgment:
      'Conflict. You are sincere and are being obstructed. A cautious halt halfway brings good fortune.',
    image: 'Heaven and water go their opposite ways: the image of Conflict.',
    lines: [false, true, false, true, true, true],
  },
  {
    number: 7,
    name: 'The Army',
    chineseName: '師 (Shī)',
    meaning: 'Organized force, discipline, leadership',
    judgment:
      'The Army. The army needs perseverance and a strong man. Good fortune without blame.',
    image: 'In the middle of the earth is water: the image of the Army.',
    lines: [false, true, false, false, false, false],
  },
  {
    number: 8,
    name: 'Holding Together',
    chineseName: '比 (Bǐ)',
    meaning: 'Union, seeking connection, forming alliances',
    judgment:
      'Holding Together brings good fortune. Inquire of the oracle once again.',
    image: 'On the earth is water: the image of Holding Together.',
    lines: [false, false, false, false, true, false],
  },
]

// Generate remaining hexagrams (9-64) as stub data
for (let i = 9; i <= 64; i++) {
  HEXAGRAMS.push({
    number: i,
    name: `Hexagram ${i}`,
    chineseName: `卦${i}`,
    meaning: `Meaning for hexagram ${i} (stub data)`,
    judgment: `The judgment for hexagram ${i}.`,
    image: `The image for hexagram ${i}.`,
    lines: [
      (i & 1) > 0,
      (i & 2) > 0,
      (i & 4) > 0,
      (i & 8) > 0,
      (i & 16) > 0,
      (i & 32) > 0,
    ],
  })
}

/**
 * Get a hexagram by number (1-64)
 */
export function getHexagramByNumber(num: number): Hexagram | undefined {
  return HEXAGRAMS.find((h) => h.number === num)
}