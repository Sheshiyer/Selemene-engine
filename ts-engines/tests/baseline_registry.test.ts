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
    const body = await response.json()

    expect(response.status).toBe(200)
    expect(body.count).toBe(6)
    expect(body.engines.map((engine: any) => engine.id).sort()).toEqual([
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
      body.engines.map((engine: any) => [engine.id, engine.version]),
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

  it('reports healthy sidecar readiness for the six registered engines (T-031)', async () => {
    const response = await fetch(`${baseUrl}/health/ready`)
    const body = await response.json()

    expect(response.status).toBe(200)
    expect(body.status).toBe('ready')
    expect(body.engines).toHaveLength(6)
    expect(body.failed_engines).toEqual([])
  })
})
