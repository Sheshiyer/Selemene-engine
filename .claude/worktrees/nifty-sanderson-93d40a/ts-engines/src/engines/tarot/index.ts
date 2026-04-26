/**
 * Tarot Engine - Public API exports
 */

// Core types
export type { TarotCard, MajorArcana, MinorArcana, Suit, Element, TarotDeck } from './wisdom'

// Wisdom data
export { loadTarotDeck } from './wisdom'

// Shuffle utilities
export type { DrawnCard } from './shuffle'
export { shuffleDeck, drawCards, drawCardsForSpread } from './shuffle'

// Spread definitions
export {
  SpreadType,
  SPREAD_DEFINITIONS,
  getSpreadDefinition,
  getSpreadPositionCount,
  parseSpreadType,
} from './spreads'
export type { SpreadPosition, SpreadDefinition } from './spreads'

// Reading functionality
export type { TarotReading, PositionReading } from './reading'
export { performReading, performReadingFromString, formatCardSummary, getKeyCards } from './reading'

// Witness prompts
export { generateWitnessPrompts, generateQuestionBasedPrompts } from './witness'

// Engine
export { TarotEngine } from './engine'
