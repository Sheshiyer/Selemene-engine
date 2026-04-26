/**
 * Sigil Forge Witness Prompts
 * Non-prescriptive inquiries for sigil creation and intention work
 */

import type { WitnessPrompt } from '../../types'
import { SeededRandom, getDefaultSeed } from '../../utils/random'
import type { SigilMethod } from './wisdom'

/**
 * Templates for contemplating intention during sigil work
 */
const INTENTION_TEMPLATES = [
  'What do you notice as you condense your intention into {result}?',
  'Where does this intention already live in your body?',
  'As the words dissolve into symbol, what shifts in your relationship to this desire?',
  "What would it feel like to have already received what you're asking for?",
  'Notice any resistance that arises. What is it protecting?',
]

const PROCESS_TEMPLATES = [
  'As you engage with the {method_name} method, what emerges?',
  'What intuitions arise as you work through these steps?',
  'Where do you feel called to deviate from the process? Follow that.',
  'What wants to be expressed that the steps cannot capture?',
]

const CHARGING_TEMPLATES = [
  'When you release this sigil, what are you truly letting go of?',
  'What attachment to outcome can you offer to the fire?',
  'Trust that the work is done. What does completion feel like?',
]

const EMBODIMENT_TEMPLATES = [
  'How does this intention want to live in your daily actions?',
  'Where in your life is this quality already present, however faintly?',
  'What is the smallest step that aligns with this intention?',
]

function fillTemplate(template: string, replacements: Record<string, string>): string {
  let result = template
  for (const [key, value] of Object.entries(replacements)) {
    result = result.replace(new RegExp(`\\{${key}\\}`, 'g'), value)
  }
  return result
}

/**
 * Generate witness prompts for sigil creation
 */
export function generateWitnessPrompts(
  method: SigilMethod,
  intention: string,
  processedResult?: string,
  seed?: number,
): WitnessPrompt[] {
  const rng = new SeededRandom(seed ?? getDefaultSeed())
  const prompts: WitnessPrompt[] = []

  // Intention contemplation
  const intentionTemplate = rng.pick(INTENTION_TEMPLATES)
  prompts.push({
    prompt: fillTemplate(intentionTemplate, {
      result: processedResult || 'symbol',
      intention: intention,
    }),
    context: 'Intention exploration',
    themes: ['intention', 'desire', 'transformation'],
  })

  // Process awareness
  const processTemplate = rng.pick(PROCESS_TEMPLATES)
  prompts.push({
    prompt: fillTemplate(processTemplate, {
      method_name: method.name,
    }),
    context: `Working with ${method.name}`,
    themes: ['process', 'creation', 'intuition'],
  })

  // Charging/release or embodiment
  if (rng.nextBool(0.5)) {
    prompts.push({
      prompt: rng.pick(CHARGING_TEMPLATES),
      context: 'Charging and release',
      themes: ['release', 'trust', 'completion'],
    })
  } else {
    prompts.push({
      prompt: rng.pick(EMBODIMENT_TEMPLATES),
      context: 'Embodiment',
      themes: ['action', 'embodiment', 'integration'],
    })
  }

  return prompts.slice(0, 3)
}
