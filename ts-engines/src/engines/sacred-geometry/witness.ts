/**
 * Sacred Geometry Witness Prompts
 * Non-prescriptive inquiries for contemplating geometric forms
 */

import type { WitnessPrompt } from '../../types'
import { SeededRandom, getDefaultSeed } from '../../utils/random'
import type { SacredForm } from './wisdom'

/**
 * Templates for contemplating geometric forms
 */
const FORM_CONTEMPLATION_TEMPLATES = [
  'As you visualize {form}, what patterns do you notice in your own life?',
  'Where do you see the principle of {form} reflected in nature around you?',
  'What does the geometry of {form} evoke in your body?',
  'How might the symbolism of {form} speak to your current situation?',
  'If {form} were a teacher, what might it be showing you?',
  'What feelings arise as you hold the image of {form} in your awareness?',
  'Where in your life do the elements of {form} — {elements} — appear?',
]

const NUMEROLOGY_TEMPLATES = [
  'The number {number} resonates through {form}. Where does this number appear in your life?',
  '{form} carries the energy of {number}. What significance does this number hold for you?',
]

const MEDITATION_TEMPLATES = [
  '{prompt} Take three breaths and notice what arises.',
  'Before you begin: {prompt}',
  'As you close your eyes: {prompt}',
]

const INTEGRATION_TEMPLATES = [
  'How might you carry the essence of {form} into your day?',
  'What one quality of {form} wants to be more present in your life?',
  'If {form} left you with a single insight, what would it be?',
]

function fillTemplate(template: string, replacements: Record<string, string | number>): string {
  let result = template
  for (const [key, value] of Object.entries(replacements)) {
    result = result.replace(new RegExp(`\\{${key}\\}`, 'g'), String(value))
  }
  return result
}

/** Template family a witness prompt came from (GH #1411). */
export type WitnessTemplate =
  | 'form_contemplation'
  | 'meditation'
  | 'intention'
  | 'numerology'
  | 'integration'

/**
 * How one witness prompt was derived: its template family, whether it is an
 * inquiry (a question) or guidance (an instruction), and the deterministic
 * `result.*` fields whose values it was filled from.
 */
export interface WitnessDerivation {
  template: WitnessTemplate
  mode: 'inquiry' | 'guidance'
  inputs: string[]
}

/** Result field each template placeholder is filled from. */
const PLACEHOLDER_INPUTS: Record<string, string> = {
  form: 'result.form.name',
  elements: 'result.form.elements',
  number: 'result.form.numerology',
  prompt: 'result.meditation.prompt',
}

/** The result fields a template actually uses, in the order they appear. */
function placeholderInputs(template: string): string[] {
  const inputs: string[] = []
  for (const match of template.matchAll(/\{(\w+)\}/g)) {
    const path = PLACEHOLDER_INPUTS[match[1]]
    if (path && !inputs.includes(path)) inputs.push(path)
  }
  return inputs
}

/**
 * Generate witness prompts for a sacred form contemplation, together with
 * how each one was derived from the deterministic result.
 */
export function generateWitnessNarrative(
  form: SacredForm,
  intention?: string,
  seed?: number,
): { prompts: WitnessPrompt[]; derivation: WitnessDerivation[] } {
  const rng = new SeededRandom(seed ?? getDefaultSeed())
  const prompts: WitnessPrompt[] = []
  const derivation: WitnessDerivation[] = []

  // Primary contemplation prompt based on the form
  const contemplationTemplate = rng.pick(FORM_CONTEMPLATION_TEMPLATES)
  prompts.push({
    prompt: fillTemplate(contemplationTemplate, {
      form: form.name,
      elements: form.elements.slice(0, 3).join(', '),
    }),
    context: `Contemplating ${form.name}`,
    themes: ['geometry', 'pattern', 'awareness', ...form.elements],
  })
  derivation.push({
    template: 'form_contemplation',
    mode: 'inquiry',
    inputs: placeholderInputs(contemplationTemplate),
  })

  // Meditation guidance
  const meditationTemplate = rng.pick(MEDITATION_TEMPLATES)
  prompts.push({
    prompt: fillTemplate(meditationTemplate, {
      prompt: form.meditationPrompt,
    }),
    context: 'Meditation guidance',
    themes: ['meditation', 'presence', 'visualization'],
  })
  derivation.push({
    template: 'meditation',
    mode: 'guidance',
    inputs: placeholderInputs(meditationTemplate),
  })

  // If intention provided, add intention-based prompt
  if (intention) {
    prompts.push({
      prompt: `Holding your intention — "${intention}" — how does ${form.name} reflect or inform it?`,
      context: 'Intention reflection',
      themes: ['intention', 'guidance', 'clarity'],
    })
    derivation.push({
      template: 'intention',
      mode: 'inquiry',
      inputs: ['result.intention', 'result.form.name'],
    })
  } else {
    // Otherwise add numerology or integration prompt
    if (rng.nextBool(0.5)) {
      const numTemplate = rng.pick(NUMEROLOGY_TEMPLATES)
      prompts.push({
        prompt: fillTemplate(numTemplate, {
          form: form.name,
          number: form.numerology,
        }),
        context: 'Numerological resonance',
        themes: ['numerology', 'symbolism', 'synchronicity'],
      })
      derivation.push({
        template: 'numerology',
        mode: 'inquiry',
        inputs: placeholderInputs(numTemplate),
      })
    } else {
      const intTemplate = rng.pick(INTEGRATION_TEMPLATES)
      prompts.push({
        prompt: fillTemplate(intTemplate, { form: form.name }),
        context: 'Integration',
        themes: ['integration', 'embodiment', 'practice'],
      })
      derivation.push({
        template: 'integration',
        mode: 'inquiry',
        inputs: placeholderInputs(intTemplate),
      })
    }
  }

  return { prompts: prompts.slice(0, 3), derivation: derivation.slice(0, 3) }
}

/**
 * Generate witness prompts for a sacred form contemplation
 */
export function generateWitnessPrompts(
  form: SacredForm,
  intention?: string,
  seed?: number,
): WitnessPrompt[] {
  return generateWitnessNarrative(form, intention, seed).prompts
}
