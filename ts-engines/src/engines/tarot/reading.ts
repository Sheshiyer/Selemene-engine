/**
 * Tarot Reading - Performs readings with spread layouts
 */

import { loadTarotDeck, type TarotCard } from './wisdom'
import { drawCardsForSpread, type DrawnCard } from './shuffle'
import {
  SpreadType,
  getSpreadDefinition,
  getSpreadPositionCount,
  parseSpreadType,
  type SpreadPosition,
} from './spreads'

export interface PositionReading {
  position: SpreadPosition
  drawnCard: DrawnCard
  interpretation: {
    meaning: string
    keywords: string[]
  }
}

export interface TarotReading {
  spread: SpreadType
  spreadName: string
  question?: string
  positions: PositionReading[]
  timestamp: string
  seed?: number
}

/**
 * Get the appropriate meaning based on card orientation
 */
function getCardMeaning(card: TarotCard, isReversed: boolean): string {
  return isReversed ? card.reversedMeaning : card.uprightMeaning
}

/**
 * Perform a tarot reading with the specified spread
 */
export function performReading(
  spreadType: SpreadType,
  seed?: number,
  question?: string,
): TarotReading {
  const deck = loadTarotDeck()
  const spreadDef = getSpreadDefinition(spreadType)
  const positionCount = getSpreadPositionCount(spreadType)

  const drawnCards = drawCardsForSpread(deck.allCards, positionCount, seed)

  const positions: PositionReading[] = spreadDef.positions.map((position, index) => {
    const drawnCard = drawnCards[index]
    return {
      position,
      drawnCard,
      interpretation: {
        meaning: getCardMeaning(drawnCard.card, drawnCard.isReversed),
        keywords: drawnCard.card.keywords,
      },
    }
  })

  return {
    spread: spreadType,
    spreadName: spreadDef.name,
    question,
    positions,
    timestamp: new Date().toISOString(),
    seed,
  }
}

/**
 * Perform a reading from a string spread type
 */
export function performReadingFromString(
  spreadTypeString: string,
  seed?: number,
  question?: string,
): TarotReading {
  const spreadType = parseSpreadType(spreadTypeString)
  if (!spreadType) {
    // Default to three card if invalid
    return performReading(SpreadType.THREE_CARD, seed, question)
  }
  return performReading(spreadType, seed, question)
}

/**
 * Get a formatted summary of a card in a reading
 */
export function formatCardSummary(positionReading: PositionReading): string {
  const { position, drawnCard, interpretation } = positionReading
  const orientation = drawnCard.isReversed ? 'Reversed' : 'Upright'

  return `${position.name}: ${drawnCard.card.name} (${orientation})
Position Meaning: ${position.meaning}
Card Meaning: ${interpretation.meaning}
Keywords: ${interpretation.keywords.join(', ')}`
}

/**
 * Get key cards from a reading (Major Arcana or significant positions)
 */
export function getKeyCards(reading: TarotReading): PositionReading[] {
  // Major Arcana are always key cards
  const majorArcanaCards = reading.positions.filter((p) => p.drawnCard.card.arcana === 'major')

  // Also include first and last positions as key
  const keyPositionIndices = new Set([0, reading.positions.length - 1])
  const keyPositionCards = reading.positions.filter((_, index) => keyPositionIndices.has(index))

  // Combine and deduplicate
  const keyCards = new Map<string, PositionReading>()
  for (const card of [...majorArcanaCards, ...keyPositionCards]) {
    keyCards.set(card.drawnCard.card.id, card)
  }

  return Array.from(keyCards.values())
}
