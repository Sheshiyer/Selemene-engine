import { describe, expect, test } from 'bun:test'
import type {
  ConsciousnessEngine,
  EngineHealthStatus,
  EngineInput,
  EngineMetadata,
  EngineOutput,
} from '../../types'
import { runCapabilityObservations } from '../app'
import { EngineRegistry } from '../registry'

const ENGINE_IDS = [
  'enneagram',
  'i-ching',
  'raaga',
  'sacred-geometry',
  'sigil-forge',
  'tarot',
] as const

type Mode = 'pass' | 'false' | 'throw' | 'timeout' | 'malformed' | 'optional' | 'absent'

class MatrixEngine implements ConsciousnessEngine {
  readonly selfCheck: (() => Promise<EngineHealthStatus>) | undefined
  calculateCalls = 0

  constructor(
    private readonly id: string,
    mode: Mode,
  ) {
    if (mode !== 'absent') {
      this.selfCheck = async () => {
        if (mode === 'pass') return { engine_id: id, healthy: true, detail: 'ok', latency_ms: 1 }
        if (mode === 'false')
          return { engine_id: id, healthy: false, detail: 'engine unavailable', latency_ms: 1 }
        if (mode === 'optional')
          return {
            engine_id: id,
            healthy: false,
            detail: 'optional dependency unavailable',
            latency_ms: 1,
          }
        if (mode === 'malformed')
          return {
            engine_id: id,
            healthy: 'yes' as unknown as boolean,
            detail: 'malformed',
            latency_ms: 1,
          }
        if (mode === 'timeout') return await new Promise<EngineHealthStatus>(() => undefined)
        throw new Error('/private/provider/secret token')
      }
    }
  }

  metadata(): EngineMetadata {
    return {
      id: this.id,
      name: this.id,
      description: 'matrix engine',
      version: 'test',
      required_phase: 0,
      input_schema: {},
    }
  }

  async calculate(_input: EngineInput): Promise<EngineOutput> {
    this.calculateCalls += 1
    return {
      engine_id: this.id,
      result: {},
      calculated_at: new Date().toISOString(),
      processing_time_ms: 1,
    }
  }
}

function matrixRegistry(
  id: string,
  mode: Mode,
): { registry: EngineRegistry; engine: MatrixEngine } {
  const registry = new EngineRegistry()
  const engine = new MatrixEngine(id, mode)
  registry.register(engine)
  return { registry, engine }
}

describe('TypeScript capability observations', () => {
  test('returns the six canonical rows in registry order with unsupported validation', async () => {
    const registry = new EngineRegistry()
    for (const id of ENGINE_IDS) registry.register(new MatrixEngine(id, 'absent'))
    const observations = await runCapabilityObservations(registry)
    const capabilities = registry.listCapabilityObservations(observations)

    expect(capabilities.map((capability) => capability.engine_id)).toEqual([...ENGINE_IDS])
    expect(capabilities).toHaveLength(6)
    expect(capabilities.every((capability) => capability.availability === 'declared')).toBe(true)
    expect(capabilities.every((capability) => capability.reason_code === 'NOT_OBSERVED')).toBe(true)
    expect(
      capabilities.every((capability) => capability.operations?.validate === 'unsupported'),
    ).toBe(true)
    expect(
      capabilities.every((capability) => capability.dependencies?.includes('typescript:bridge')),
    ).toBe(true)
  })

  test.each([
    ['pass', 'available', 'CAPABILITY_AVAILABLE'],
    ['false', 'unavailable', 'CAPABILITY_UNAVAILABLE'],
    ['throw', 'unavailable', 'MODULE_UNAVAILABLE'],
    ['timeout', 'unavailable', 'TIMEOUT'],
    ['malformed', 'unavailable', 'MALFORMED_OBSERVATION'],
    ['optional', 'degraded', 'OPTIONAL_DEPENDENCY_UNAVAILABLE'],
    ['absent', 'declared', 'NOT_OBSERVED'],
  ] as const)('%s self-check maps to bounded %s truth', async (mode, availability, reason_code) => {
    const { registry, engine } = matrixRegistry('tarot', mode)
    const observations = await runCapabilityObservations(registry)
    const capability = registry.listCapabilityObservations(observations)[0]
    expect(capability?.availability).toBe(availability)
    expect(capability?.reason_code).toBe(reason_code)
    expect(engine.calculateCalls).toBe(0)
    expect(JSON.stringify(capability)).not.toMatch(/private|provider|secret|token|https?:\/\//i)
  })
})
