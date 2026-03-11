import { afterAll, beforeAll, describe, expect, it } from 'bun:test'
import { EnneagramEngine } from '../src/engines/enneagram'
import { IChingEngine } from '../src/engines/i-ching'
import { SacredGeometryEngine } from '../src/engines/sacred-geometry'
import { SigilForgeEngine } from '../src/engines/sigil-forge'
import { TarotEngine } from '../src/engines/tarot'
import { createServer, EngineRegistry } from '../src/server'

const TEST_PORT = Number(process.env.TS_ENGINES_BASELINE_TEST_PORT ?? '3099')

let server: ReturnType<typeof createServer> | null = null
let baseUrl: string

beforeAll(() => {
  const registry = new EngineRegistry()
  registry.register(new TarotEngine())
  registry.register(new IChingEngine())
  registry.register(new EnneagramEngine())
  registry.register(new SacredGeometryEngine())
  registry.register(new SigilForgeEngine())

  server = createServer(registry)
  server.listen(TEST_PORT)
  baseUrl = `http://localhost:${TEST_PORT}`
})

afterAll(() => {
  server?.stop()
})

describe('TS baseline registry', () => {
  it('registers the five bridge engines with stable metadata', async () => {
    const response = await fetch(`${baseUrl}/engines`)
    const body = await response.json()

    expect(response.status).toBe(200)
    expect(body.count).toBe(5)
    expect(body.engines.map((engine: any) => engine.id).sort()).toEqual([
      'enneagram',
      'i-ching',
      'sacred-geometry',
      'sigil-forge',
      'tarot',
    ])
    expect(body.engines.every((engine: any) => engine.version === '1.0.0')).toBe(true)
  })

  it('reports healthy sidecar readiness for the five registered engines', async () => {
    const response = await fetch(`${baseUrl}/health/ready`)
    const body = await response.json()

    expect(response.status).toBe(200)
    expect(body.status).toBe('ready')
    expect(body.engines).toHaveLength(5)
    expect(body.failed_engines).toEqual([])
  })
})
