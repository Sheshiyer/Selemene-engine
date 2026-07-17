/**
 * Sigil Forge Prompt Builder
 * Converts sigil creation context into NVIDIA NIM image generation prompts.
 *
 * Sigils are sacred symbols encoded with intention. The prompt must produce:
 * - A single, unified glyph/symbol (not a scene or illustration)
 * - Abstract and non-representational enough to feel archetypal
 * - High contrast, suitable for charging rituals
 */

import type { SigilMethod } from './wisdom'
import type { GeneratedImage, ImageGenOptions } from '../../providers/image-provider' // T-035 + T-060: correct relative (engines/sigil → providers)

/** Visual style presets for different sigil aesthetics */
export type SigilStyle =
  | 'ceremonial' // classical magic circle / alchemical style
  | 'chaos' // chaos magick — raw, angular, deconstructed letterforms
  | 'organic' // flowing, plantlike, biomorphic
  | 'geometric' // sacred geometry, platonic solids, mandalas
  | 'runic' // Norse/Elder Futhark influenced angular forms
  | 'ethereal' // soft glowing light on dark field

const STYLE_DESCRIPTORS: Record<SigilStyle, { visual: string; medium: string; palette: string }> = {
  ceremonial: {
    visual: 'ceremonial magic circle, alchemical glyph, enochian sigil, classical occult symbol',
    medium: 'black ink on aged parchment, fine line etching, woodcut print',
    palette: 'sepia and black, high contrast, antique',
  },
  chaos: {
    visual:
      'chaos magick sigil, deconstructed letterforms merged into abstract glyph, angular angular',
    medium: 'bold black marker on white, stark graphic, cut paper collage aesthetic',
    palette: 'pure black and white, maximum contrast',
  },
  organic: {
    visual:
      'biomorphic sacred sigil, flowing organic glyph, single spiral contained within a circle, nature-inspired occult symbol, NOT a plant illustration',
    medium: 'fine pen and ink, delicate linework, contained within a circle, single unified mark',
    palette: 'dark ink on cream, subtle warm tones',
  },
  geometric: {
    visual:
      'sacred geometry sigil, overlapping circles and triangles forming a unified glyph, Metatrons cube influence',
    medium: 'precise geometric line drawing, compass and straightedge aesthetic',
    palette: 'gold on black, or black on white, exact',
  },
  runic: {
    visual:
      'runic sigil, angular interlocking staves, bind rune, Elder Futhark inspired unified glyph',
    medium: 'carved stone inscription, woodburned symbol, bold angular marks',
    palette: 'dark charcoal on bone, aged texture',
  },
  ethereal: {
    visual:
      'neon glowing sigil, luminous unified glyph, intricate rune radiating energy, magical symbol',
    medium: 'fantasy digital art, masterwork illustration, neon light art aesthetic',
    palette:
      'electric blue and purple neon light on dark indigo background, luminous glowing lines, high luminosity',
  },
}

/** Map sigil method to best default visual style */
const METHOD_STYLE_MAP: Record<string, SigilStyle> = {
  'word-elimination': 'chaos',
  'rose-wheel': 'ceremonial',
  pictographic: 'organic',
  'chaos-star': 'chaos',
  numerological: 'geometric',
  runic: 'runic',
}

function elementFromIntention(intention: string): string {
  const lower = intention.toLowerCase()
  if (/fire|passion|will|power|strength|courage|bold/.test(lower)) return 'fire element influence'
  if (/water|flow|emotion|heal|love|heart|feel/.test(lower)) return 'water element influence'
  if (/earth|ground|stable|safe|home|body|manifest|physical/.test(lower))
    return 'earth element influence'
  if (/air|mind|clarity|wisdom|truth|speak|voice|word/.test(lower)) return 'air element influence'
  if (/spirit|soul|divine|cosmic|infinite|transcend|higher/.test(lower))
    return 'spirit/aether influence'
  return 'balanced elemental influence'
}

function qualityFromIntention(intention: string): string {
  const lower = intention.toLowerCase()
  if (/protect|shield|ward|guard|banish|repel/.test(lower))
    return 'protective ward, banishing sigil'
  if (/attract|draw|bring|receive|abundance|love|magnet/.test(lower))
    return 'attraction sigil, drawing energy'
  if (/transform|change|shift|evolve|release|break/.test(lower))
    return 'transformation sigil, releasing old patterns'
  if (/create|manifest|build|grow|become|achieve/.test(lower))
    return 'manifestation sigil, creative force'
  if (/know|learn|understand|see|reveal|clarity|insight/.test(lower))
    return 'divination sigil, revealing hidden knowledge'
  return 'intention sigil'
}

export interface BuiltPrompt {
  prompt: string
  negative_prompt: string
  /** Suggested NIM inference steps */
  num_inference_steps: number
  /** Suggested guidance scale */
  guidance_scale: number
  style: SigilStyle
}

/**
 * Build an optimised text-to-image prompt for a sigil based on intention and method.
 */
export function buildSigilPrompt(
  intention: string,
  method: SigilMethod,
  processedLetters?: string,
  styleOverride?: SigilStyle,
): BuiltPrompt {
  const style = styleOverride ?? METHOD_STYLE_MAP[method.id] ?? 'ceremonial'
  const sd = STYLE_DESCRIPTORS[style]
  const element = elementFromIntention(intention)
  const quality = qualityFromIntention(intention)
  const letterHint = processedLetters
    ? `incorporates the abstract forms of these letter-strokes: ${processedLetters.split('').join(' ')},`
    : ''

  const prompt = [
    'A single unified sigil symbol,',
    `${quality},`,
    letterHint,
    `${element},`,
    `${sd.visual},`,
    `${sd.medium},`,
    `${sd.palette},`,
    'centered on plain background, isolated symbol, no text, no words, no labels,',
    'single glyph, hermetic tradition, occult art, esoteric symbol,',
    'masterwork, intricate fine detail, spiritually charged',
  ]
    .filter(Boolean)
    .join(' ')
    .replace(/,\s*,/g, ',') // remove double commas
    .replace(/\s+/g, ' ')
    .trim()

  const negative_prompt = [
    'text, letters, words, alphabet, typography, label, caption, watermark,',
    'photograph, realistic, 3d render, CGI,',
    'multiple symbols, pattern repeat, decorative border,',
    'human figure, face, hands, body,',
    'blurry, noisy, low quality, pixelated, disfigured,',
    'cross, star of david, swastika, satanic, demonic, skull',
  ].join(' ')

  // Flux dev uses more steps; schnell/sana are faster
  const num_inference_steps = style === 'ethereal' ? 30 : 25
  const guidance_scale = style === 'geometric' ? 8.0 : 7.5

  return { prompt, negative_prompt, num_inference_steps, guidance_scale, style }
}

/**
 * Build a prompt for editing an existing sigil image.
 * Used when the user provides a previous sigil and wants a variation or refinement.
 */
export function buildSigilEditPrompt(
  intention: string,
  editInstruction: string,
  originalStyle: SigilStyle = 'ceremonial',
): { prompt: string; negative_prompt: string } {
  const sd = STYLE_DESCRIPTORS[originalStyle]
  return {
    prompt: [
      `Refine this sigil: ${editInstruction}.`,
      'Maintain the unified symbolic form,',
      `${sd.visual},`,
      `${sd.medium},`,
      'no text, no words, single glyph, occult art',
    ].join(' '),
    negative_prompt:
      'text, letters, words, photograph, realistic, multiple symbols, blurry, low quality',
  }
}

// T-035 + T-060: re-export provider-aligned types for consumers (prompts feed ImageProvider.generate/edit)
export type { GeneratedImage, ImageGenOptions } from '../../providers/image-provider'
