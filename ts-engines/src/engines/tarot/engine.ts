/**
 * TarotEngine - Consciousness engine implementation for tarot readings
 */

import type { ConsciousnessEngine, EngineMetadata, EngineInput, EngineOutput } from '../../types'
import { SpreadType, parseSpreadType, SPREAD_DEFINITIONS } from './spreads'
import { performReading } from './reading'
import { generateQuestionBasedPrompts } from './witness'

export class TarotEngine implements ConsciousnessEngine {
  metadata(): EngineMetadata {
    return {
      id: 'tarot',
      name: 'Tarot Consciousness Engine',
      description:
        'Tarot card readings with witness prompts for self-reflection. Supports multiple spread types including single card, three card, Celtic Cross, relationship, and career spreads.',
      version: '1.0.0',
      required_phase: 0,
      input_schema: {
        spread: {
          type: 'string',
          required: false,
          description:
            'The spread type to use for the reading (single_card, three_card, celtic_cross, relationship, career)',
          default: 'three_card',
          enum: Object.values(SpreadType),
        },
        question: {
          type: 'string',
          required: false,
          description: 'Optional question or intention for the reading',
        },
      },
    }
  }

  async calculate(input: EngineInput): Promise<EngineOutput> {
    const startTime = performance.now()

    // Extract parameters
    const spreadParam = (input.parameters.spread as string) ?? 'three_card'
    const question = input.question ?? (input.parameters.question as string)
    const seed = input.seed

    // Parse spread type
    const spreadType = parseSpreadType(spreadParam) ?? SpreadType.THREE_CARD

    // Perform the reading
    const reading = performReading(spreadType, seed, question)

    // Generate witness prompts
    const witnessPrompts = generateQuestionBasedPrompts(reading, seed)

    const endTime = performance.now()

    // Format the result
    const result = {
      spread: {
        type: reading.spread,
        name: reading.spreadName,
        description: SPREAD_DEFINITIONS[reading.spread].description,
      },
      question: reading.question,
      positions: reading.positions.map((p) => ({
        position: p.position.position,
        name: p.position.name,
        meaning: p.position.meaning,
        card: {
          id: p.drawnCard.card.id,
          name: p.drawnCard.card.name,
          arcana: p.drawnCard.card.arcana,
          suit: p.drawnCard.card.suit,
          number: p.drawnCard.card.number,
          element: p.drawnCard.card.element,
          isReversed: p.drawnCard.isReversed,
          interpretation: {
            meaning: p.interpretation.meaning,
            keywords: p.interpretation.keywords,
          },
        },
      })),
      seed: reading.seed,
    }

    return {
      engine_id: 'tarot',
      result,
      witness_prompts: witnessPrompts,
      calculated_at: reading.timestamp,
      processing_time_ms: Math.round(endTime - startTime),
    }
  }
}
