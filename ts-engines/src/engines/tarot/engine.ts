/**
 * TarotEngine - Consciousness engine implementation for tarot readings
 */

import type { ConsciousnessEngine, EngineInput, EngineMetadata, EngineOutput } from '../../types'
import { EngineValidationError } from '../../utils'
import { performReading } from './reading'
import { SPREAD_DEFINITIONS, SpreadType, parseSpreadType } from './spreads'
import { generateQuestionBasedPrompts } from './witness'

type ResultCard = {
  position: number
  name: string
  meaning: string
  id: string
  cardName: string
  arcana: string
  suit?: string
  number: number
  element?: string
  isReversed: boolean
  interpretation: {
    meaning: string
    keywords: string[]
  }
}

function extractSpreadParam(input: EngineInput): string {
  const spread = input.parameters.spread
  const spreadType = input.parameters.spread_type
  const spreadTypeCamel = input.parameters.spreadType

  const candidate = [spread, spreadType, spreadTypeCamel].find((value) => typeof value === 'string')
  return (candidate as string | undefined) ?? 'three_card'
}

function getYesNoDecision(card: ResultCard): {
  answer: 'yes' | 'no'
  confidence: number
  rationale: string
} {
  if (card.isReversed) {
    return {
      answer: 'no',
      confidence: 0.74,
      rationale: `${card.cardName} appeared reversed, signaling friction or delay in this path.`,
    }
  }

  return {
    answer: 'yes',
    confidence: 0.74,
    rationale: `${card.cardName} appeared upright, suggesting a supportive current for this direction.`,
  }
}

export class TarotEngine implements ConsciousnessEngine {
  metadata(): EngineMetadata {
    return {
      id: 'tarot',
      name: 'Tarot Consciousness Engine',
      description:
        'Tarot card readings with witness prompts for self-reflection. Supports single card, three card, Celtic Cross, horseshoe, relationship, career, and yes/no spreads.',
      version: '1.0.0',
      required_phase: 0,
      input_schema: {
        spread: {
          type: 'string',
          required: false,
          description:
            'The spread type to use for the reading (single_card, three_card, celtic_cross, horseshoe, relationship, career, yes_no)',
          default: 'three_card',
          enum: Object.values(SpreadType),
        },
        spread_type: {
          type: 'string',
          required: false,
          description:
            'Compatibility alias for spread. Use one of: single_card, three_card, celtic_cross, horseshoe, relationship, career, yes_no',
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
    const spreadParam = extractSpreadParam(input)
    const question = input.question ?? (input.parameters.question as string)
    const seed = input.seed

    // Parse spread type
    const parsedSpread = parseSpreadType(spreadParam)
    if (!parsedSpread) {
      throw new EngineValidationError(
        'Invalid spread_type. Use one of single_card, three_card, celtic_cross, horseshoe, relationship, career, yes_no.',
        'INVALID_SPREAD_TYPE',
        {
          spread_type: spreadParam,
          supported: Object.values(SpreadType),
        },
      )
    }
    const spreadType = parsedSpread

    if (question !== undefined && typeof question === 'string' && question.trim() === '') {
      throw new EngineValidationError(
        'Question cannot be empty when provided.',
        'INVALID_QUESTION',
        {
          field: 'question',
        },
      )
    }

    // Perform the reading
    const reading = performReading(spreadType, seed, question)

    // Generate witness prompts
    const witnessPrompts = generateQuestionBasedPrompts(reading, seed)

    const endTime = performance.now()

    const positions = reading.positions.map((p) => ({
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
    }))

    const cards: ResultCard[] = positions.map((p) => ({
      position: p.position,
      name: p.name,
      meaning: p.meaning,
      id: p.card.id,
      cardName: p.card.name,
      arcana: p.card.arcana,
      suit: p.card.suit,
      number: p.card.number,
      element: p.card.element,
      isReversed: p.card.isReversed,
      interpretation: p.card.interpretation,
    }))

    // Format the result
    const result = {
      spread: {
        type: reading.spread,
        name: reading.spreadName,
        description: SPREAD_DEFINITIONS[reading.spread].description,
        card_count: cards.length,
        available_types: Object.values(SpreadType),
      },
      question: reading.question,
      positions,
      cards,
      decision: spreadType === SpreadType.YES_NO && cards[0] ? getYesNoDecision(cards[0]) : undefined,
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
