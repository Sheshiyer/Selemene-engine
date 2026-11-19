/**
 * Seed validation across the seeded TypeScript engines.
 *
 * SeededRandom converts the seed with BigInt(), which throws a RangeError for
 * non-integer and non-finite numbers. The sidecar's route schema only checks
 * that `seed` is a number, so `seed: 1.5` used to escape as a 500
 * CALCULATION_ERROR. Every seeded engine must instead reject it up front with
 * the same INVALID_SEED validation error I Ching already uses.
 */
import { describe, expect, it } from 'bun:test'
import { EnneagramEngine } from '../src/engines/enneagram'
import { IChingEngine } from '../src/engines/i-ching'
import { SacredGeometryEngine } from '../src/engines/sacred-geometry'
import { SigilForgeEngine } from '../src/engines/sigil-forge'
import { TarotEngine } from '../src/engines/tarot'
import { EngineRegistry, createServer } from '../src/server'
import type { ConsciousnessEngine } from '../src/types'
import { EngineValidationError } from '../src/utils'

const ENGINES: Array<[string, ConsciousnessEngine, Record<string, unknown>]> = [
  ['tarot', new TarotEngine(), {}],
  ['i-ching', new IChingEngine(), {}],
  ['enneagram', new EnneagramEngine(), { type: 1 }],
  ['sacred-geometry', new SacredGeometryEngine(), {}],
  ['sigil-forge', new SigilForgeEngine(), { intention: 'a synthetic intention' }],
]

async function errorFor(
  engine: ConsciousnessEngine,
  parameters: Record<string, unknown>,
  seed: number,
) {
  try {
    await engine.calculate({ consciousness_level: 0, parameters, seed })
  } catch (error) {
    return error
  }
  return undefined
}

describe('seed validation', () => {
  for (const [id, engine, parameters] of ENGINES) {
    it(`${id} rejects non-integer and non-finite seeds with INVALID_SEED`, async () => {
      for (const seed of [
        1.5,
        -0.25,
        Number.NaN,
        Number.POSITIVE_INFINITY,
        Number.NEGATIVE_INFINITY,
      ]) {
        const error = await errorFor(engine, parameters, seed)
        expect(error).toBeInstanceOf(EngineValidationError)
        expect((error as EngineValidationError).code).toBe('INVALID_SEED')
        expect((error as EngineValidationError).details).toEqual({ field: 'seed' })
      }
    })

    it(`${id} still accepts integer seeds and an omitted seed`, async () => {
      for (const seed of [0, 42, 0xffffffff]) {
        expect(await errorFor(engine, parameters, seed)).toBeUndefined()
      }
      await engine.calculate({ consciousness_level: 0, parameters })
    })
  }

  it('the sidecar answers a non-integer seed with 422 INVALID_SEED, not 500', async () => {
    const registry = new EngineRegistry()
    registry.register(new TarotEngine())
    const app = createServer(registry)

    const response = await app.handle(
      new Request('http://localhost/engines/tarot/calculate', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ consciousness_level: 0, parameters: {}, seed: 1.5 }),
      }),
    )

    expect(response.status).toBe(422)
    expect(await response.json()).toMatchObject({
      error_code: 'INVALID_SEED',
      details: { field: 'seed' },
    })
  })
})
