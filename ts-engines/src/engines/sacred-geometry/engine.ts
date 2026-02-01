/**
 * SacredGeometryEngine - Consciousness engine for sacred geometric forms
 */

import type { ConsciousnessEngine, EngineInput, EngineMetadata, EngineOutput } from '../../types'
import { SeededRandom, getDefaultSeed } from '../../utils/random'
import { SACRED_FORMS, getFormById, getFormIds } from './wisdom'
import { generateWitnessPrompts } from './witness'

export class SacredGeometryEngine implements ConsciousnessEngine {
  metadata(): EngineMetadata {
    return {
      id: 'sacred-geometry',
      name: 'Sacred Geometry Consciousness Engine',
      description:
        'Exploration of sacred geometric forms with symbolism, meditation guidance, and witness prompts. Includes Flower of Life, Platonic Solids, Sri Yantra, and more.',
      version: '1.0.0',
      required_phase: 0,
      input_schema: {
        form: {
          type: 'string',
          required: false,
          description: 'Specific form to contemplate. If not provided, a random form is selected.',
          enum: getFormIds(),
        },
        intention: {
          type: 'string',
          required: false,
          description: 'Optional intention or question to hold while contemplating the form.',
        },
      },
    }
  }

  async calculate(input: EngineInput): Promise<EngineOutput> {
    const startTime = performance.now()

    // Extract parameters
    const formParam = input.parameters.form as string | undefined
    const intention = input.question ?? (input.parameters.intention as string | undefined)
    const seed = input.seed ?? getDefaultSeed()

    // Select form
    let form = formParam ? getFormById(formParam) : undefined

    if (!form) {
      // Random selection if no valid form specified
      const rng = new SeededRandom(seed)
      form = rng.pick(SACRED_FORMS)
    }

    // Generate witness prompts
    const witnessPrompts = generateWitnessPrompts(form, intention, seed)

    const endTime = performance.now()

    // Build result
    const result = {
      form: {
        id: form.id,
        name: form.name,
        description: form.description,
        symbolism: form.symbolism,
        elements: form.elements,
        numerology: form.numerology,
      },
      meditation: {
        prompt: form.meditationPrompt,
        duration_suggestion: '5-15 minutes',
      },
      intention: intention ?? null,
      seed,
    }

    return {
      engine_id: 'sacred-geometry',
      result,
      witness_prompts: witnessPrompts,
      calculated_at: new Date().toISOString(),
      processing_time_ms: Math.round(endTime - startTime),
    }
  }
}
