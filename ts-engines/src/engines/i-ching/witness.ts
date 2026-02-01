/**
 * I-Ching Witness Prompts
 */

import type { WitnessPrompt } from '../../types'
import type { Hexagram } from './wisdom'
import { SeededRandom, getDefaultSeed } from '../../utils/random'

const HEXAGRAM_TEMPLATES = [
  'What does {name} illuminate about your current situation?',
  'Where in your life do you recognize the energy of {name}?',
  'How does the image of {image} speak to what you are experiencing?',
  'What arises as you sit with the judgment: "{judgment}"?',
]

const CHANGING_TEMPLATES = [
  'As the situation transforms from {primary} to {relating}, what transition do you sense in your own life?',
  'The changing lines suggest movement. Where is change already in motion?',
]

export function generateWitnessPrompts(
  primary: Hexagram,
  relating?: Hexagram,
  changingLines?: number[],
  seed?: number,
): WitnessPrompt[] {
  const rng = new SeededRandom(seed ?? getDefaultSeed())
  const prompts: WitnessPrompt[] = []

  // Primary hexagram prompt
  const template = rng.pick(HEXAGRAM_TEMPLATES)
  prompts.push({
    prompt: template
      .replace('{name}', primary.name)
      .replace('{image}', primary.image.split(':')[0])
      .replace('{judgment}', primary.judgment.split('.')[0]),
    context: `Primary hexagram: ${primary.number}. ${primary.name}`,
    themes: ['change', 'wisdom', 'situation'],
  })

  // If there's a relating hexagram
  if (relating && changingLines && changingLines.length > 0) {
    const changingTemplate = rng.pick(CHANGING_TEMPLATES)
    prompts.push({
      prompt: changingTemplate
        .replace('{primary}', primary.name)
        .replace('{relating}', relating.name),
      context: `Transformation: ${primary.name} → ${relating.name}`,
      themes: ['transformation', 'change', 'movement'],
    })
  }

  // Overall reflection
  prompts.push({
    prompt: 'What question does this oracle seem to be asking you in return?',
    context: 'Reflection',
    themes: ['inquiry', 'self-reflection', 'wisdom'],
  })

  return prompts.slice(0, 3)
}
