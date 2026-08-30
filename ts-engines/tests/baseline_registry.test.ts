import { afterAll, beforeAll, describe, expect, it } from 'bun:test'
import { EnneagramEngine } from '../src/engines/enneagram'
import { IChingEngine } from '../src/engines/i-ching'
import { SacredGeometryEngine } from '../src/engines/sacred-geometry'
import { SigilForgeEngine } from '../src/engines/sigil-forge'
import { RaagaEngine } from '../src/engines/raaga'
import { TarotEngine } from '../src/engines/tarot'
import { createServer, EngineRegistry } from '../src/server'

const TEST_PORT = Number(process.env.TS_ENGINES_BASELINE_TEST_PORT ?? '3099')

let server: ReturnType<typeof createServer> | null = null
let baseUrl: string

interface EngineSummary {
  id: string
  version: string
}

interface EngineListBody {
  count: number
  engines: EngineSummary[]
}

interface EngineCapabilitySummary {
  contract_version: 'v1'
  engine_id: string
  display_name: string
  availability: string
  runtime_kind: string
  dependencies: string[]
  required_phase?: number
  implementation_version?: string
}

interface EngineCapabilityListBody {
  count: number
  capabilities: EngineCapabilitySummary[]
}

interface ReadinessBody {
  status: string
  engines: unknown[]
  failed_engines: unknown[]
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null
}

function parseEngineListBody(value: unknown): EngineListBody {
  if (!isRecord(value) || typeof value.count !== 'number' || !Array.isArray(value.engines)) {
    throw new Error('invalid /engines response shape')
  }
  const engines = value.engines.map((engine): EngineSummary => {
    if (!isRecord(engine) || typeof engine.id !== 'string' || typeof engine.version !== 'string') {
      throw new Error('invalid engine metadata in /engines response')
    }
    return { id: engine.id, version: engine.version }
  })
  return { count: value.count, engines }
}

function parseEngineCapabilityListBody(value: unknown): EngineCapabilityListBody {
  if (!isRecord(value) || typeof value.count !== 'number' || !Array.isArray(value.capabilities)) {
    throw new Error('invalid /engines/capabilities response shape')
  }
  const capabilities = value.capabilities.map((capability): EngineCapabilitySummary => {
    if (
      !isRecord(capability) ||
      capability.contract_version !== 'v1' ||
      typeof capability.engine_id !== 'string' ||
      typeof capability.display_name !== 'string' ||
      typeof capability.availability !== 'string' ||
      typeof capability.runtime_kind !== 'string' ||
      !Array.isArray(capability.dependencies)
    ) {
      throw new Error('invalid engine capability in /engines/capabilities response')
    }
    return capability as unknown as EngineCapabilitySummary
  })
  return { count: value.count, capabilities }
}

function parseReadinessBody(value: unknown): ReadinessBody {
  if (
    !isRecord(value) ||
    typeof value.status !== 'string' ||
    !Array.isArray(value.engines) ||
    !Array.isArray(value.failed_engines)
  ) {
    throw new Error('invalid /health/ready response shape')
  }
  return {
    status: value.status,
    engines: value.engines,
    failed_engines: value.failed_engines,
  }
}

beforeAll(() => {
  const registry = new EngineRegistry()
  registry.register(new TarotEngine())
  registry.register(new IChingEngine())
  registry.register(new EnneagramEngine())
  registry.register(new SacredGeometryEngine())
  registry.register(new SigilForgeEngine())
  registry.register(new RaagaEngine())

  server = createServer(registry)
  server.listen(TEST_PORT)
  baseUrl = `http://localhost:${TEST_PORT}`
})

afterAll(() => {
  server?.stop()
})

describe('TS baseline registry', () => {
  it('registers the six bridge engines (incl raaga T-031) with stable metadata', async () => {
    const response = await fetch(`${baseUrl}/engines`)
    const body = parseEngineListBody(await response.json())

    expect(response.status).toBe(200)
    expect(body.count).toBe(6)
    expect(body.engines.map((engine) => engine.id).sort()).toEqual([
      'enneagram',
      'i-ching',
      'raaga',
      'sacred-geometry',
      'sigil-forge',
      'tarot',
    ])
    // sigil-forge bumped to 2.0.0 when image-gen support shipped; the
    // other four remain at 1.0.0. Test the actual version mapping rather
    // than asserting global 1.0.0 (which silently drifted out of date).
    const versionsById = Object.fromEntries(
      body.engines.map((engine) => [engine.id, engine.version]),
    )
    expect(versionsById).toEqual({
      enneagram: '1.0.0',
      'i-ching': '1.0.0',
      raaga: '1.0.0',
      'sacred-geometry': '1.0.0',
      'sigil-forge': '2.0.0',
      tarot: '1.0.0',
    })
  })

  it('exposes registered engines as live v1 TypeScript capability records', async () => {
    const response = await fetch(`${baseUrl}/engines/capabilities`)
    expect(response.status).toBe(200)

    const body = parseEngineCapabilityListBody(await response.json())

    expect(body.count).toBe(6)
    const capabilitiesById = Object.fromEntries(
      body.capabilities.map((capability) => [capability.engine_id, capability]),
    )
    expect(Object.keys(capabilitiesById).sort()).toEqual([
      'enneagram',
      'i-ching',
      'raaga',
      'sacred-geometry',
      'sigil-forge',
      'tarot',
    ])
    expect(capabilitiesById.tarot).toEqual({
      contract_version: 'v1',
      engine_id: 'tarot',
      display_name: 'Tarot Consciousness Engine',
      availability: 'available',
      runtime_kind: 'typescript',
      dependencies: [],
      required_phase: 0,
      implementation_version: '1.0.0',
    })
    expect(body.capabilities.every((capability) => capability.runtime_kind === 'typescript')).toBe(
      true,
    )
    expect(body.capabilities.every((capability) => capability.contract_version === 'v1')).toBe(true)
  })

  it('reports healthy sidecar readiness for the six registered engines (T-031)', async () => {
    const response = await fetch(`${baseUrl}/health/ready`)
    const body = parseReadinessBody(await response.json())

    expect(response.status).toBe(200)
    expect(body.status).toBe('ready')
    expect(body.engines).toHaveLength(6)
    expect(body.failed_engines).toEqual([])
  })
})
