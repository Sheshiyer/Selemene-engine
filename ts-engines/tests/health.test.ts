import { afterAll, beforeAll, describe, expect, it } from 'bun:test'
import { createServer, EngineRegistry } from '../src/server'
import type {
  ConsciousnessEngine,
  EngineHealthStatus,
  EngineInput,
  EngineMetadata,
  EngineOutput,
} from '../src/types'

const TEST_PORT = Number(process.env.TS_ENGINES_HEALTH_TEST_PORT ?? '3098')

class FakeHealthEngine implements ConsciousnessEngine {
  constructor(
    private readonly meta: EngineMetadata,
    private healthy = true,
    private readonly detail = 'ok',
  ) {}

  metadata(): EngineMetadata {
    return this.meta
  }

  async calculate(_input: EngineInput): Promise<EngineOutput> {
    return {
      engine_id: this.meta.id,
      result: {},
      witness_prompts: [],
      calculated_at: new Date().toISOString(),
      processing_time_ms: 1,
    }
  }

  async selfCheck(): Promise<EngineHealthStatus> {
    return {
      engine_id: this.meta.id,
      healthy: this.healthy,
      detail: this.healthy ? this.detail : 'engine unhealthy',
      latency_ms: 1,
    }
  }

  setHealthy(healthy: boolean): void {
    this.healthy = healthy
  }
}

let server: ReturnType<typeof createServer> | null = null
let baseUrl: string
let registry: EngineRegistry
let healthyEngine: FakeHealthEngine
let flakyEngine: FakeHealthEngine

async function apiCall(path: string): Promise<{ status: number; data: unknown }> {
  const response = await fetch(`${baseUrl}${path}`)
  const data = await response.json()
  return { status: response.status, data }
}

beforeAll(() => {
  registry = new EngineRegistry()
  healthyEngine = new FakeHealthEngine({
    id: 'healthy-engine',
    name: 'Healthy Engine',
    description: 'Healthy test engine',
    version: '1.0.0',
    required_phase: 0,
    input_schema: {},
  })
  flakyEngine = new FakeHealthEngine(
    {
      id: 'flaky-engine',
      name: 'Flaky Engine',
      description: 'Degradable test engine',
      version: '1.0.0',
      required_phase: 0,
      input_schema: {},
    },
    true,
  )

  registry.register(healthyEngine)
  registry.register(flakyEngine)

  server = createServer(registry)
  server.listen(TEST_PORT)
  baseUrl = `http://localhost:${TEST_PORT}`
})

afterAll(() => {
  if (server) {
    server.stop()
  }
})

describe('TS sidecar health routes', () => {
  it('GET /health/live returns unconditional liveness', async () => {
    flakyEngine.setHealthy(false)

    const { status, data } = await apiCall('/health/live')

    expect(status).toBe(200)
    expect((data as any).status).toBe('alive')
    expect((data as any).uptime_ms).toBeGreaterThan(0)
  })

  it('GET /health/ready returns 200 with per-engine status when all engines are healthy', async () => {
    flakyEngine.setHealthy(true)

    const { status, data } = await apiCall('/health/ready')

    expect(status).toBe(200)
    expect((data as any).status).toBe('ready')
    expect((data as any).engines.length).toBe(2)
    expect((data as any).engines.every((engine: any) => engine.healthy)).toBe(true)
  })

  it('GET /health/engines returns per-engine health entries with latency', async () => {
    flakyEngine.setHealthy(true)

    const { status, data } = await apiCall('/health/engines')

    expect(status).toBe(200)
    expect((data as any).engines.length).toBe(2)
    expect((data as any).engines[0]).toHaveProperty('engine_id')
    expect((data as any).engines[0]).toHaveProperty('healthy')
    expect((data as any).engines[0]).toHaveProperty('detail')
    expect((data as any).engines[0]).toHaveProperty('latency_ms')
  })

  it('GET /health/ready returns 503 with failed engine details when one engine is unhealthy', async () => {
    flakyEngine.setHealthy(false)

    const { status, data } = await apiCall('/health/ready')

    expect(status).toBe(503)
    expect((data as any).status).toBe('degraded')
    expect((data as any).failed_engines).toEqual(['flaky-engine'])
    expect(
      (data as any).engines.find((engine: any) => engine.engine_id === 'flaky-engine').healthy,
    ).toBe(false)
  })

  it('GET /engines/capabilities marks unhealthy engines unavailable', async () => {
    flakyEngine.setHealthy(false)

    const { status, data } = await apiCall('/engines/capabilities')

    expect(status).toBe(200)
    const capabilities = (data as any).capabilities
    expect(capabilities).toHaveLength(2)
    expect(
      capabilities.find((capability: any) => capability.engine_id === 'healthy-engine'),
    ).toMatchObject({
      contract_version: 'v1',
      availability: 'available',
      runtime_kind: 'typescript',
    })
    expect(
      capabilities.find((capability: any) => capability.engine_id === 'flaky-engine'),
    ).toMatchObject({
      contract_version: 'v1',
      availability: 'unavailable',
      runtime_kind: 'typescript',
    })
  })

  it('GET /health/ready recovers to 200 after the unhealthy engine recovers', async () => {
    flakyEngine.setHealthy(false)
    let response = await apiCall('/health/ready')
    expect(response.status).toBe(503)

    flakyEngine.setHealthy(true)
    response = await apiCall('/health/ready')

    expect(response.status).toBe(200)
    expect((response.data as any).status).toBe('ready')
    expect((response.data as any).failed_engines).toEqual([])
  })
})
