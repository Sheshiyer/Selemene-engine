/**
 * SigilForgeEngine - Consciousness engine for sigil creation guidance
 */

import type { ConsciousnessEngine, EngineInput, EngineMetadata, EngineOutput } from '../../types'
import { SeededRandom, getDefaultSeed } from '../../utils/random'
import {
  CHARGING_METHODS,
  SIGIL_METHODS,
  getMethodById,
  getMethodIds,
  processWordElimination,
} from './wisdom'
import { generateWitnessPrompts } from './witness'

export class SigilForgeEngine implements ConsciousnessEngine {
  metadata(): EngineMetadata {
    return {
      id: 'sigil-forge',
      name: 'Sigil Forge Consciousness Engine',
      description:
        'Guided sigil creation process with multiple methods. Provides step-by-step instructions for creating personal symbols of intention. Note: This engine provides guidance, not visual sigil generation.',
      version: '1.0.0',
      required_phase: 1, // Requires phase 1 consciousness
      input_schema: {
        intention: {
          type: 'string',
          required: true,
          description:
            'The intention or desire to encode into a sigil. Write as a present-tense statement.',
        },
        method: {
          type: 'string',
          required: false,
          description:
            'Sigil creation method to use. If not specified, one is recommended based on the intention.',
          enum: getMethodIds(),
        },
      },
    }
  }

  async calculate(input: EngineInput): Promise<EngineOutput> {
    const startTime = performance.now()

    // Extract parameters
    const intention = input.question ?? (input.parameters.intention as string)
    const methodParam = input.parameters.method as string | undefined
    const seed = input.seed ?? getDefaultSeed()

    // Validate intention
    if (!intention || typeof intention !== 'string' || intention.trim() === '') {
      throw new Error(
        'Intention parameter is required. Please provide a present-tense statement of your desire.',
      )
    }

    const cleanIntention = intention.trim()

    // Select method
    let method = methodParam ? getMethodById(methodParam) : undefined

    if (!method) {
      // Auto-select based on intention characteristics
      const rng = new SeededRandom(seed)
      // Longer intentions work better with word elimination
      // Shorter/conceptual intentions work well with pictographic
      if (cleanIntention.length > 50) {
        method = SIGIL_METHODS[0] // word-elimination
      } else if (cleanIntention.split(' ').length <= 3) {
        method = SIGIL_METHODS[2] // pictographic
      } else {
        method = rng.pick(SIGIL_METHODS)
      }
    }

    // Process intention for word elimination method
    let processedLetters: string | null = null
    if (method.id === 'word-elimination') {
      processedLetters = processWordElimination(cleanIntention)
    }

    // Select charging suggestions
    const rng = new SeededRandom(seed + 1)
    const chargingSuggestions = rng.sample(CHARGING_METHODS, 2)

    // Generate witness prompts
    const witnessPrompts = generateWitnessPrompts(
      method,
      cleanIntention,
      processedLetters ?? undefined,
      seed,
    )

    const endTime = performance.now()

    // Build result
    const result = {
      intention: cleanIntention,
      method: {
        id: method.id,
        name: method.name,
        description: method.description,
        steps: method.steps,
      },
      processing: processedLetters
        ? {
            type: 'word_elimination',
            original: cleanIntention,
            remaining_letters: processedLetters,
            letter_count: processedLetters.length,
          }
        : null,
      charging_suggestions: chargingSuggestions.map((c) => ({
        name: c.name,
        description: c.description,
      })),
      guidance: {
        note: 'This engine provides the process for sigil creation. The actual visual sigil must be created by you — this personal investment is essential to the magic.',
        next_steps: [
          'Gather your materials (paper, pen, or digital canvas)',
          `Follow the ${method.name} steps above`,
          'Allow intuition to guide the final form',
          'Choose a charging method that resonates',
          'Release attachment to outcome',
        ],
      },
      seed,
    }

    return {
      engine_id: 'sigil-forge',
      result,
      witness_prompts: witnessPrompts,
      calculated_at: new Date().toISOString(),
      processing_time_ms: Math.round(endTime - startTime),
    }
  }
}
