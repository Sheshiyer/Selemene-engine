/** Non-prescriptive reflection, without quoting unverified or absent source text. */

import type { WitnessPrompt } from '../../types'
import { SeededRandom, getDefaultSeed } from '../../utils/random'
import type { Hexagram } from './wisdom'

const REFLECTIONS = [
  'What aspects of your situation would you like to examine through this symbolic reading?',
  'What assumptions about your situation could you hold more lightly while reflecting?',
]

export function generateWitnessPrompts(
  primary: Hexagram,
  relating?: Hexagram,
  changingLines?: number[],
  seed?: number,
): WitnessPrompt[] {
  const rng = new SeededRandom(seed ?? getDefaultSeed())
  const prompts: WitnessPrompt[] = [
    {
      prompt: rng.pick(REFLECTIONS),
      context: `Primary King Wen hexagram ${primary.number}. Text status: ${primary.wisdomStatus}.`,
      themes: ['inquiry', 'situation', 'reflection'],
    },
  ]
  if (relating && changingLines && changingLines.length > 0) {
    prompts.push({
      prompt: 'What changes are you already noticing, and what remains uncertain about them?',
      context: `Changing lines ${changingLines.join(', ')} (bottom to top) produce relating hexagram ${relating.number}.`,
      themes: ['change', 'uncertainty', 'reflection'],
    })
  }
  prompts.push({
    prompt: 'What question would help you explore this situation with more agency?',
    context: 'Symbolic reflection, not a prediction or instruction.',
    themes: ['inquiry', 'agency', 'reflection'],
  })
  return prompts.slice(0, 3)
}
