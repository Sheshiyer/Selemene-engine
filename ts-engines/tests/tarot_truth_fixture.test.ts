/**
 * Tarot semantic truth fixture (Phase 4 / ENG-01 / ENG-02).
 *
 * The fixture captures the deterministic local path only. It deliberately does
 * not claim generated, fallback, or unavailable behavior because Tarot has no
 * provider or sidecar dependency in the current implementation.
 */
import { describe, expect, it } from 'bun:test'
import { TarotEngine } from '../src/engines/tarot'
import type { EngineInput, EngineOutput } from '../src/types'

type TarotTruthFixture = {
  input: EngineInput
  expected: Pick<EngineOutput, 'engine_id' | 'result' | 'witness_prompts' | 'provenance'>
}

const fixturePath = new URL('./fixtures/tarot/three-card-seed-12345.json', import.meta.url)

function stableOutput(output: EngineOutput): TarotTruthFixture['expected'] {
  return {
    engine_id: output.engine_id,
    result: output.result,
    witness_prompts: output.witness_prompts,
    provenance: output.provenance,
  }
}

describe('TarotEngine semantic truth fixture', () => {
  it('replays the fixture with stable cards, interpretations, prompts, and provenance', async () => {
    const fixture = JSON.parse(await Bun.file(fixturePath).text()) as TarotTruthFixture
    const engine = new TarotEngine()

    const first = stableOutput(await engine.calculate(fixture.input))
    const second = stableOutput(await engine.calculate(fixture.input))

    expect(first).toEqual(fixture.expected)
    expect(second).toEqual(fixture.expected)
    expect(second).toEqual(first)
  })

  it('rejects an unsupported spread instead of silently defaulting', async () => {
    const engine = new TarotEngine()

    await expect(
      engine.calculate({
        consciousness_level: 0,
        parameters: { spread: 'unsupported-spread' },
        seed: 12345,
      }),
    ).rejects.toMatchObject({ name: 'EngineValidationError', code: 'INVALID_SPREAD_TYPE' })
  })
})
