/**
 * Tarot Shuffle - Fisher-Yates shuffle implementation using SeededRandom
 */

import { SeededRandom, getDefaultSeed } from '../../utils/random'
import type { TarotCard } from './wisdom'

export interface DrawnCard {
  card: TarotCard
  isReversed: boolean
  position?: number
}

/**
 * Shuffle a deck of cards using Fisher-Yates algorithm with seeded randomness
 */
export function shuffleDeck(cards: TarotCard[], seed?: number): TarotCard[] {
  const rng = new SeededRandom(seed ?? getDefaultSeed())
  const deck = [...cards] // Create a copy to avoid mutating original
  return rng.shuffle(deck)
}

/**
 * Draw a specified number of cards from a shuffled deck
 * Each card has a 50% chance of being reversed
 */
export function drawCards(deck: TarotCard[], count: number, seed?: number): DrawnCard[] {
  const rng = new SeededRandom(seed ?? getDefaultSeed())
  const shuffled = rng.shuffle([...deck])
  const drawn: DrawnCard[] = []

  for (let i = 0; i < count && i < shuffled.length; i++) {
    drawn.push({
      card: shuffled[i],
      isReversed: rng.nextBool(0.5),
      position: i,
    })
  }

  return drawn
}

/**
 * Draw cards specifically for a spread with position assignments
 */
export function drawCardsForSpread(
  deck: TarotCard[],
  positionCount: number,
  seed?: number,
): DrawnCard[] {
  return drawCards(deck, positionCount, seed)
}
