/**
 * Sacred Geometry Wisdom Data
 * Contains sacred geometric forms with symbolism and meditation guidance
 */

export interface SacredForm {
  id: string
  name: string
  description: string
  symbolism: string
  meditationPrompt: string
  elements: string[]
  numerology: number
}

export const SACRED_FORMS: SacredForm[] = [
  {
    id: 'flower-of-life',
    name: 'Flower of Life',
    description:
      'An ancient symbol composed of multiple evenly-spaced, overlapping circles arranged in a flower-like pattern with six-fold symmetry. It is considered to contain the patterns of creation.',
    symbolism:
      'Unity, creation, and the interconnectedness of all life. Contains the blueprint for all existence - from the structure of atoms to galaxies.',
    meditationPrompt:
      'Allow your awareness to rest on the center of the Flower of Life. Notice how each circle supports and is supported by others. Let the pattern breathe.',
    elements: ['circle', 'hexagon', 'vesica piscis'],
    numerology: 6,
  },
  {
    id: 'seed-of-life',
    name: 'Seed of Life',
    description:
      'Seven overlapping circles forming a rose-like pattern. It represents the seven days of creation and is the core pattern within the Flower of Life.',
    symbolism:
      'Creation, fertility, and blessing. Represents the first seven days of creation and the fundamental patterns of existence.',
    meditationPrompt:
      'Focus on the central circle, then allow your gaze to soften as you take in all seven circles simultaneously. What emerges in the spaces between?',
    elements: ['circle', 'hexagon'],
    numerology: 7,
  },
  {
    id: 'metatrons-cube',
    name: "Metatron's Cube",
    description:
      'A complex geometric figure derived from the Flower of Life, containing all five Platonic solids. Named after the archangel Metatron.',
    symbolism:
      'Divine knowledge, transformation, and the architecture of the universe. Contains the fundamental forms that define matter and energy.',
    meditationPrompt:
      "Trace the lines of Metatron's Cube with your awareness. Notice how complexity arises from simplicity. What hidden structures reveal themselves?",
    elements: [
      'circle',
      'line',
      'tetrahedron',
      'cube',
      'octahedron',
      'dodecahedron',
      'icosahedron',
    ],
    numerology: 13,
  },
  {
    id: 'sri-yantra',
    name: 'Sri Yantra',
    description:
      'An ancient Hindu symbol consisting of nine interlocking triangles radiating from a central point (bindu), surrounded by lotus petals and a gated frame.',
    symbolism:
      'Cosmic unity, the union of masculine (Shiva) and feminine (Shakti), and the journey from the infinite to the finite and back.',
    meditationPrompt:
      'Begin at the outer gate and let your gaze slowly journey inward through the lotus petals and triangles. Rest at the bindu. What stillness resides there?',
    elements: ['triangle', 'circle', 'lotus', 'square'],
    numerology: 9,
  },
  {
    id: 'vesica-piscis',
    name: 'Vesica Piscis',
    description:
      'The intersection of two circles with the same radius, where each circle passes through the center of the other. Creates an almond or fish-like shape.',
    symbolism:
      'The womb of creation, duality, and the birth of form from formlessness. Represents the first act of creation - the division of unity.',
    meditationPrompt:
      'Contemplate the space where two become one. The vesica is neither circle, yet emerges from both. What new possibility lives in your own intersections?',
    elements: ['circle', 'lens'],
    numerology: 2,
  },
  {
    id: 'tetrahedron',
    name: 'Tetrahedron',
    description:
      'The simplest Platonic solid, consisting of four equilateral triangular faces. Associated with the element of Fire.',
    symbolism:
      'Fire, transformation, and the spark of life. Represents the first physical form, the most stable structure, and the foundation of all geometry.',
    meditationPrompt:
      'Feel the stability of three points resting on a surface while one point reaches upward. Where does the fire element move within you?',
    elements: ['triangle', 'point'],
    numerology: 4,
  },
  {
    id: 'cube',
    name: 'Cube (Hexahedron)',
    description:
      'A Platonic solid with six square faces. Associated with the element of Earth and the foundation of physical reality.',
    symbolism:
      'Earth, stability, and grounding. Represents solidity, structure, and the manifest world.',
    meditationPrompt:
      'Imagine yourself seated within a perfect cube. Feel the equal distance to all six faces. Where do you need more grounding in your life?',
    elements: ['square', 'point'],
    numerology: 6,
  },
  {
    id: 'octahedron',
    name: 'Octahedron',
    description:
      'A Platonic solid with eight equilateral triangular faces. Associated with the element of Air and the heart chakra.',
    symbolism:
      'Air, integration, and compassion. A bridge between earthly and spiritual realms, representing balance and healing.',
    meditationPrompt:
      'Visualize the octahedron suspended at your heart center, spinning gently. What rises and what settles as it turns?',
    elements: ['triangle', 'point'],
    numerology: 8,
  },
  {
    id: 'dodecahedron',
    name: 'Dodecahedron',
    description:
      'A Platonic solid with twelve pentagonal faces. Associated with the element of Ether/Spirit and the cosmos.',
    symbolism:
      'Ether, universe, and divine expression. Plato associated it with the cosmos itself. Represents the heavens and transcendence.',
    meditationPrompt:
      'Each pentagon contains the golden ratio. Hold the dodecahedron in your mind as the shape of the universe itself. What does this perspective reveal?',
    elements: ['pentagon', 'point'],
    numerology: 12,
  },
  {
    id: 'icosahedron',
    name: 'Icosahedron',
    description:
      'A Platonic solid with twenty equilateral triangular faces. Associated with the element of Water and emotional flow.',
    symbolism:
      'Water, flow, and adaptability. Represents movement, transformation, and the capacity to reshape while maintaining essential form.',
    meditationPrompt:
      'Let the twenty faces roll and tumble in your imagination like water. What emotions ask to flow more freely through you?',
    elements: ['triangle', 'point'],
    numerology: 20,
  },
  {
    id: 'golden-spiral',
    name: 'Golden Spiral',
    description:
      'A logarithmic spiral whose growth factor is the golden ratio (φ ≈ 1.618). Found throughout nature in shells, galaxies, and plant growth.',
    symbolism:
      'Growth, evolution, and natural harmony. Represents the pattern by which nature expands while maintaining proportion.',
    meditationPrompt:
      'Trace the spiral from center outward, or from infinite inward. Notice: both directions exist simultaneously. Where are you in your own spiral of growth?',
    elements: ['spiral', 'golden ratio', 'curve'],
    numerology: 1,
  },
  {
    id: 'torus',
    name: 'Torus',
    description:
      'A donut-shaped surface of revolution. Represents the fundamental shape of energy flow in the universe, from atoms to galaxies.',
    symbolism:
      'Energy flow, self-sustaining systems, and the cycle of creation. Represents how energy continuously flows, returns, and regenerates.',
    meditationPrompt:
      'Feel energy rising through your center, flowering outward, descending around you, and returning through your base. You are the torus.',
    elements: ['circle', 'sphere', 'vortex'],
    numerology: 0,
  },
]

/**
 * Get a sacred form by its ID
 */
export function getFormById(id: string): SacredForm | undefined {
  return SACRED_FORMS.find((form) => form.id === id)
}

/**
 * Get all form IDs
 */
export function getFormIds(): string[] {
  return SACRED_FORMS.map((form) => form.id)
}
