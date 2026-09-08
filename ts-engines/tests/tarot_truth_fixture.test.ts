import { describe, expect, it } from 'bun:test'
import { TarotEngine } from '../src/engines/tarot'
import type { EngineInput } from '../src/types'

type TarotTruthFixture = {
  input: EngineInput
  expected: {
    engine_id: string
    seed: number
    spread: string
    card_id: string
    card_name: string
    runtime_kind: string
    implementation_version: string
    fallback_used: boolean
    confidence: number
  }
}

type TarotResult = {
  seed: number
  spread: { type: string }
  positions: Array<{ card: { id: string; name: string } }>
}

describe('TarotEngine semantic truth fixture', () => {
  it('replays the same deterministic semantic result for a fixed seed', async () => {
    const fixture = (await Bun.file(new URL('./fixtures/tarot-truth.json', import.meta.url)).json()) as TarotTruthFixture
    const engine = new TarotEngine()

    const first = await engine.calculate(fixture.input)
    const second = await engine.calculate(fixture.input)
    const firstResult = first.result as TarotResult
    const secondResult = second.result as TarotResult

    expect(first.engine_id).toBe(fixture.expected.engine_id)
    expect(firstResult.seed).toBe(fixture.expected.seed)
    expect(firstResult.spread.type).toBe(fixture.expected.spread)
    expect(firstResult.positions[0]?.card.id).toBe(fixture.expected.card_id)
    expect(firstResult.positions[0]?.card.name).toBe(fixture.expected.card_name)
    expect(secondResult.positions).toEqual(firstResult.positions)
    expect(first.provenance).toMatchObject({
      runtime_kind: fixture.expected.runtime_kind,
      implementation_version: fixture.expected.implementation_version,
      fallback_used: fixture.expected.fallback_used,
      confidence: fixture.expected.confidence,
    })
  })
})
