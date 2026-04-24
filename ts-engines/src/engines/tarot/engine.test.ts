import { describe, expect, it } from 'bun:test'
import { TarotEngine } from './engine'

type TarotResult = {
  spread: {
    type: string
    card_count: number
    available_types: string[]
  }
  positions: Array<{
    position: number
    name: string
    meaning: string
    card: {
      id: string
      name: string
      isReversed: boolean
      interpretation: {
        meaning: string
        keywords: string[]
      }
    }
  }>
  cards: Array<{
    position: number
    name: string
    meaning: string
    id: string
    cardName: string
  }>
  decision?: {
    answer: 'yes' | 'no'
    confidence: number
    rationale: string
  }
}

describe('TarotEngine spread compatibility and payload scaffolding', () => {
  const engine = new TarotEngine()

  it('accepts legacy spread_type and returns celtic cross positions', async () => {
    const output = await engine.calculate({
      consciousness_level: 0,
      parameters: {
        spread_type: 'celtic_cross',
        question: 'What should I focus on?'
      },
      seed: 42,
    })

    const result = output.result as TarotResult
    expect(result.spread.type).toBe('celtic_cross')
    expect(result.positions.length).toBe(10)
    expect(result.cards.length).toBe(10)
  })

  it('accepts spreadType alias and supports horseshoe spread', async () => {
    const output = await engine.calculate({
      consciousness_level: 0,
      parameters: {
        spreadType: 'horseshoe',
      },
      seed: 42,
    })

    const result = output.result as TarotResult
    expect(result.spread.type).toBe('horseshoe')
    expect(result.positions.length).toBe(7)
    expect(result.spread.card_count).toBe(7)
  })

  it('supports yes_no spread with explicit decision payload', async () => {
    const output = await engine.calculate({
      consciousness_level: 0,
      parameters: {
        spread: 'yes_no',
      },
      seed: 42,
    })

    const result = output.result as TarotResult
    expect(result.spread.type).toBe('yes_no')
    expect(result.positions.length).toBe(1)
    expect(result.cards.length).toBe(1)
    expect(result.decision).toBeDefined()
    expect(result.decision?.answer === 'yes' || result.decision?.answer === 'no').toBeTrue()
    expect(result.decision?.confidence).toBeGreaterThan(0)
    expect(result.decision?.rationale.length).toBeGreaterThan(0)
  })

  it('exposes available spread variants in scaffolded payload', async () => {
    const output = await engine.calculate({
      consciousness_level: 0,
      parameters: {
        spread: 'three_card',
      },
      seed: 42,
    })

    const result = output.result as TarotResult
    expect(result.spread.available_types).toContain('three_card')
    expect(result.spread.available_types).toContain('celtic_cross')
    expect(result.spread.available_types).toContain('horseshoe')
    expect(result.spread.available_types).toContain('relationship')
    expect(result.spread.available_types).toContain('career')
    expect(result.spread.available_types).toContain('yes_no')
  })
})
