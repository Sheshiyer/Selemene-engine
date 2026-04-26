/**
 * Sigil Forge Wisdom Data
 * Contains sigil creation methods and guidance
 */

export interface SigilMethod {
  id: string
  name: string
  description: string
  steps: string[]
}

export const SIGIL_METHODS: SigilMethod[] = [
  {
    id: 'word-elimination',
    name: 'Word Elimination Method',
    description:
      'The classic chaos magick technique. Write your intention as a statement, remove all vowels and duplicate consonants, then arrange the remaining letters into a symbol.',
    steps: [
      'Write your intention as a present-tense statement of desire (e.g., "I am confident and calm")',
      'Remove all vowels from the statement',
      'Remove all duplicate consonants, keeping only the first occurrence of each',
      'Take the remaining consonants and begin sketching them together',
      'Let the letters flow, overlap, rotate, and merge into a single unified symbol',
      'Refine until the original letters are no longer recognizable as letters',
      'The resulting abstract symbol is your sigil',
    ],
  },
  {
    id: 'rose-wheel',
    name: 'Rose Wheel (Rosy Cross) Method',
    description:
      'Map letter positions onto a circular diagram and connect them with a continuous line. Creates flowing, elegant sigils with a mystical quality.',
    steps: [
      'Draw a circle and divide it into sections for each letter of the alphabet (like a clock face)',
      'Write your intention statement',
      'Locate the position of each letter on the wheel',
      'Draw a continuous line connecting the letters in order',
      'Mark the starting point with a small circle',
      'Mark the ending point with a short perpendicular line',
      'The path traced across the wheel forms your sigil',
    ],
  },
  {
    id: 'pictographic',
    name: 'Pictographic Combination',
    description:
      'Create a symbol by combining simplified pictographs that represent key concepts from your intention. More intuitive and image-based than letter methods.',
    steps: [
      'Break your intention into 2-4 core concepts or keywords',
      'For each concept, draw a simple symbolic representation (not letters)',
      'Begin combining these symbols into a unified design',
      'Allow overlap, nesting, and transformation as they merge',
      'Simplify complex areas while maintaining the essential character',
      'Continue refining until a single cohesive symbol emerges',
      'The final image should feel complete and balanced',
    ],
  },
  {
    id: 'chaos-star',
    name: 'Chaos Star Method',
    description:
      'Use the eight-pointed Chaos Star (Symbol of Chaos) as a framework. Place intention elements at each point or between points, creating a symbol with built-in magical structure.',
    steps: [
      'Draw an eight-pointed chaos star (eight arrows radiating from center)',
      'Identify eight aspects or components of your intention',
      'Assign each aspect to one of the eight points/directions',
      'Draw small symbols or modified letters at each point',
      'Connect the elements with additional lines as intuition guides',
      'The center point represents the unified intention',
      'Add a personal mark at the center to complete the binding',
    ],
  },
]

export interface ChargingMethod {
  id: string
  name: string
  description: string
}

export const CHARGING_METHODS: ChargingMethod[] = [
  {
    id: 'gnosis-meditation',
    name: 'Meditation Gnosis',
    description:
      'Enter a deep meditative state while focusing intently on the sigil. When the mind becomes still and focused, release the intention and forget the sigil.',
  },
  {
    id: 'gnosis-physical',
    name: 'Physical Gnosis',
    description:
      'Use physical exertion (dancing, exercise, breath work) to reach a peak state. At the moment of exhaustion or release, focus briefly on the sigil then let go.',
  },
  {
    id: 'destruction',
    name: 'Destruction Charging',
    description:
      'After creating the sigil, destroy it with intention — burn it, tear it up, bury it, or dissolve it in water. The act of destruction releases the energy.',
  },
  {
    id: 'display',
    name: 'Passive Display',
    description:
      'Place the sigil somewhere it will be seen regularly but not consciously focused on. The symbol works on the subconscious over time.',
  },
  {
    id: 'dreaming',
    name: 'Dream Charging',
    description:
      'Gaze at the sigil before sleep with the intention to dream it into manifestation. Place it under your pillow or beside your bed.',
  },
]

/**
 * Get a sigil method by ID
 */
export function getMethodById(id: string): SigilMethod | undefined {
  return SIGIL_METHODS.find((method) => method.id === id)
}

/**
 * Get all method IDs
 */
export function getMethodIds(): string[] {
  return SIGIL_METHODS.map((method) => method.id)
}

/**
 * Process intention using word elimination method
 * Returns the consonants that would form the sigil base
 */
export function processWordElimination(intention: string): string {
  // Remove vowels
  const noVowels = intention.toLowerCase().replace(/[aeiou\s]/g, '')
  // Remove duplicate consonants
  const seen = new Set<string>()
  let result = ''
  for (const char of noVowels) {
    if (!seen.has(char) && /[a-z]/.test(char)) {
      seen.add(char)
      result += char
    }
  }
  return result.toUpperCase()
}
