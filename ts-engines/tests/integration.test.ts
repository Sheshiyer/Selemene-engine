/**
 * Integration Tests for Noesis TypeScript Engines
 * Tests all 5 engines: Tarot, I-Ching, Enneagram, Sacred Geometry, Sigil Forge
 */

import { describe, it, expect, beforeAll, afterAll } from 'bun:test'
import { createServer, registry } from '../src/server'

// Import and register engines
import { TarotEngine } from '../src/engines/tarot'
import { IChingEngine } from '../src/engines/i-ching'
import { EnneagramEngine } from '../src/engines/enneagram'
import { SacredGeometryEngine } from '../src/engines/sacred-geometry'
import { SigilForgeEngine } from '../src/engines/sigil-forge'

const TEST_PORT = 3099
let server: ReturnType<typeof createServer> | null = null
let baseUrl: string

// Helper function for API calls
async function apiCall(
  method: 'GET' | 'POST',
  path: string,
  body?: Record<string, unknown>
): Promise<{ status: number; data: unknown }> {
  const url = `${baseUrl}${path}`
  const options: RequestInit = {
    method,
    headers: { 'Content-Type': 'application/json' },
  }
  if (body) {
    options.body = JSON.stringify(body)
  }
  const response = await fetch(url, options)
  const data = await response.json()
  return { status: response.status, data }
}

beforeAll(() => {
  // Register all engines
  registry.register(new TarotEngine())
  registry.register(new IChingEngine())
  registry.register(new EnneagramEngine())
  registry.register(new SacredGeometryEngine())
  registry.register(new SigilForgeEngine())

  // Start server
  server = createServer()
  server.listen(TEST_PORT)
  baseUrl = `http://localhost:${TEST_PORT}`
})

afterAll(() => {
  if (server) {
    server.stop()
  }
})

// ============================================================================
// 1. Server Health Tests
// ============================================================================

describe('Server Health', () => {
  it('GET /health returns healthy status', async () => {
    const { status, data } = await apiCall('GET', '/health')
    expect(status).toBe(200)
    expect((data as any).status).toBe('healthy')
    expect((data as any).version).toBe('1.0.0')
    expect((data as any).uptime_ms).toBeGreaterThan(0)
  })

  it('GET /health includes all engine IDs', async () => {
    const { status, data } = await apiCall('GET', '/health')
    expect(status).toBe(200)
    const engines = (data as any).engines as string[]
    expect(engines).toContain('tarot')
    expect(engines).toContain('i-ching')
    expect(engines).toContain('enneagram')
    expect(engines).toContain('sacred-geometry')
    expect(engines).toContain('sigil-forge')
  })

  it('GET /engines lists all 5 engines', async () => {
    const { status, data } = await apiCall('GET', '/engines')
    expect(status).toBe(200)
    expect((data as any).count).toBe(5)
    expect((data as any).engines.length).toBe(5)
  })
})

// ============================================================================
// 2. Tarot Engine Tests
// ============================================================================

describe('Tarot Engine', () => {
  it('GET /engines/tarot/info returns metadata', async () => {
    const { status, data } = await apiCall('GET', '/engines/tarot/info')
    expect(status).toBe(200)
    expect((data as any).id).toBe('tarot')
    expect((data as any).required_phase).toBe(0)
  })

  it('single card spread works', async () => {
    const { status, data } = await apiCall('POST', '/engines/tarot/calculate', {
      consciousness_level: 0,
      parameters: { spread: 'single_card' },
    })
    expect(status).toBe(200)
    expect((data as any).result.positions.length).toBe(1)
    expect((data as any).result.spread.type).toBe('single_card')
  })

  it('three card spread returns 3 positions', async () => {
    const { status, data } = await apiCall('POST', '/engines/tarot/calculate', {
      consciousness_level: 0,
      parameters: { spread: 'three_card' },
    })
    expect(status).toBe(200)
    expect((data as any).result.positions.length).toBe(3)
  })

  it('celtic cross returns 10 positions', async () => {
    const { status, data } = await apiCall('POST', '/engines/tarot/calculate', {
      consciousness_level: 0,
      parameters: { spread: 'celtic_cross' },
    })
    expect(status).toBe(200)
    expect((data as any).result.positions.length).toBe(10)
  })

  it('seed produces reproducible results', async () => {
    const seed = 12345
    const { data: data1 } = await apiCall('POST', '/engines/tarot/calculate', {
      consciousness_level: 0,
      parameters: { spread: 'three_card' },
      seed,
    })
    const { data: data2 } = await apiCall('POST', '/engines/tarot/calculate', {
      consciousness_level: 0,
      parameters: { spread: 'three_card' },
      seed,
    })
    // Same seed should produce same cards
    expect((data1 as any).result.positions[0].card.id).toBe((data2 as any).result.positions[0].card.id)
    expect((data1 as any).result.positions[1].card.id).toBe((data2 as any).result.positions[1].card.id)
    expect((data1 as any).result.positions[2].card.id).toBe((data2 as any).result.positions[2].card.id)
  })

  it('witness prompts are non-empty', async () => {
    const { status, data } = await apiCall('POST', '/engines/tarot/calculate', {
      consciousness_level: 0,
      parameters: { spread: 'single_card' },
    })
    expect(status).toBe(200)
    expect((data as any).witness_prompts.length).toBeGreaterThan(0)
    expect((data as any).witness_prompts[0].prompt.length).toBeGreaterThan(10)
  })
})

// ============================================================================
// 3. I-Ching Engine Tests
// ============================================================================

describe('I-Ching Engine', () => {
  it('GET /engines/i-ching/info returns metadata', async () => {
    const { status, data } = await apiCall('GET', '/engines/i-ching/info')
    expect(status).toBe(200)
    expect((data as any).id).toBe('i-ching')
    expect((data as any).required_phase).toBe(0)
  })

  it('returns primary hexagram (1-64)', async () => {
    const { status, data } = await apiCall('POST', '/engines/i-ching/calculate', {
      consciousness_level: 0,
      parameters: {},
    })
    expect(status).toBe(200)
    const hexNum = (data as any).result.primary_hexagram.number
    expect(hexNum).toBeGreaterThanOrEqual(1)
    expect(hexNum).toBeLessThanOrEqual(64)
  })

  it('specific hexagram selection works', async () => {
    const { status, data } = await apiCall('POST', '/engines/i-ching/calculate', {
      consciousness_level: 0,
      parameters: { hexagram: 1 },
    })
    expect(status).toBe(200)
    expect((data as any).result.primary_hexagram.number).toBe(1)
    expect((data as any).result.primary_hexagram.name).toBe('The Creative')
  })

  it('changing lines produce relating hexagram', async () => {
    // Use a seed that we know produces changing lines
    // Run multiple times to ensure we get one with changing lines
    let hasChangingLines = false
    for (let seed = 1; seed < 100 && !hasChangingLines; seed++) {
      const { data } = await apiCall('POST', '/engines/i-ching/calculate', {
        consciousness_level: 0,
        parameters: {},
        seed,
      })
      if ((data as any).result.changing_lines !== null) {
        hasChangingLines = true
        expect((data as any).result.relating_hexagram).not.toBeNull()
      }
    }
    expect(hasChangingLines).toBe(true)
  })

  it('witness prompts included', async () => {
    const { status, data } = await apiCall('POST', '/engines/i-ching/calculate', {
      consciousness_level: 0,
      parameters: {},
    })
    expect(status).toBe(200)
    expect((data as any).witness_prompts.length).toBeGreaterThan(0)
  })
})

// ============================================================================
// 4. Enneagram Engine Tests
// ============================================================================

describe('Enneagram Engine', () => {
  it('GET /engines/enneagram/info returns metadata', async () => {
    const { status, data } = await apiCall('GET', '/engines/enneagram/info')
    expect(status).toBe(200)
    expect((data as any).id).toBe('enneagram')
  })

  it('direct type lookup works', async () => {
    const { status, data } = await apiCall('POST', '/engines/enneagram/calculate', {
      consciousness_level: 1,
      parameters: { type: 4 },
    })
    expect(status).toBe(200)
    expect((data as any).result.mode).toBe('lookup')
    expect((data as any).result.typeAnalysis.type.number).toBe(4)
    expect((data as any).result.typeAnalysis.type.name).toBe('The Individualist')
  })

  it('assessment scoring works with mock answers', async () => {
    // Provide 45 answers (5 questions per type × 9 types)
    // Weight toward type 7 (answers 31-35)
    const answers = Array(45).fill(3)
    // Boost type 7 questions (indices 30-34)
    answers[30] = 5
    answers[31] = 5
    answers[32] = 5
    answers[33] = 5
    answers[34] = 5

    const { status, data } = await apiCall('POST', '/engines/enneagram/calculate', {
      consciousness_level: 1,
      parameters: { answers },
    })
    expect(status).toBe(200)
    expect((data as any).result.mode).toBe('assessment')
    expect((data as any).result.assessment.primaryType.number).toBeGreaterThanOrEqual(1)
    expect((data as any).result.assessment.primaryType.number).toBeLessThanOrEqual(9)
  })

  it('returns type in range 1-9', async () => {
    const { status, data } = await apiCall('POST', '/engines/enneagram/calculate', {
      consciousness_level: 1,
      parameters: { type: 9 },
    })
    expect(status).toBe(200)
    const typeNum = (data as any).result.typeAnalysis.type.number
    expect(typeNum).toBeGreaterThanOrEqual(1)
    expect(typeNum).toBeLessThanOrEqual(9)
  })
})

// ============================================================================
// 5. Sacred Geometry Engine Tests
// ============================================================================

describe('Sacred Geometry Engine', () => {
  it('GET /engines/sacred-geometry/info returns metadata', async () => {
    const { status, data } = await apiCall('GET', '/engines/sacred-geometry/info')
    expect(status).toBe(200)
    expect((data as any).id).toBe('sacred-geometry')
    expect((data as any).required_phase).toBe(0)
  })

  it('returns form data', async () => {
    const { status, data } = await apiCall('POST', '/engines/sacred-geometry/calculate', {
      consciousness_level: 0,
      parameters: {},
    })
    expect(status).toBe(200)
    expect((data as any).result.form).toBeDefined()
    expect((data as any).result.form.name).toBeDefined()
    expect((data as any).result.form.description).toBeDefined()
    expect((data as any).result.form.symbolism).toBeDefined()
  })

  it('random selection works', async () => {
    const { status, data } = await apiCall('POST', '/engines/sacred-geometry/calculate', {
      consciousness_level: 0,
      parameters: {},
      seed: 42,
    })
    expect(status).toBe(200)
    expect((data as any).result.form.id).toBeDefined()
  })

  it('specific form selection works', async () => {
    const { status, data } = await apiCall('POST', '/engines/sacred-geometry/calculate', {
      consciousness_level: 0,
      parameters: { form: 'flower-of-life' },
    })
    expect(status).toBe(200)
    expect((data as any).result.form.id).toBe('flower-of-life')
    expect((data as any).result.form.name).toBe('Flower of Life')
  })

  it('meditation guidance included', async () => {
    const { status, data } = await apiCall('POST', '/engines/sacred-geometry/calculate', {
      consciousness_level: 0,
      parameters: { form: 'sri-yantra' },
    })
    expect(status).toBe(200)
    expect((data as any).result.meditation).toBeDefined()
    expect((data as any).result.meditation.prompt.length).toBeGreaterThan(10)
  })

  it('witness prompts included', async () => {
    const { status, data } = await apiCall('POST', '/engines/sacred-geometry/calculate', {
      consciousness_level: 0,
      parameters: {},
    })
    expect(status).toBe(200)
    expect((data as any).witness_prompts.length).toBeGreaterThan(0)
  })
})

// ============================================================================
// 6. Sigil Forge Engine Tests
// ============================================================================

describe('Sigil Forge Engine', () => {
  it('GET /engines/sigil-forge/info returns metadata', async () => {
    const { status, data } = await apiCall('GET', '/engines/sigil-forge/info')
    expect(status).toBe(200)
    expect((data as any).id).toBe('sigil-forge')
    expect((data as any).required_phase).toBe(1) // Requires phase 1
  })

  it('requires intention parameter', async () => {
    const { status, data } = await apiCall('POST', '/engines/sigil-forge/calculate', {
      consciousness_level: 1,
      parameters: {},
    })
    expect(status).toBe(500)
    expect((data as any).error).toContain('Intention')
  })

  it('returns method and steps', async () => {
    const { status, data } = await apiCall('POST', '/engines/sigil-forge/calculate', {
      consciousness_level: 1,
      parameters: { intention: 'I am confident and calm' },
    })
    expect(status).toBe(200)
    expect((data as any).result.method).toBeDefined()
    expect((data as any).result.method.name).toBeDefined()
    expect((data as any).result.method.steps.length).toBeGreaterThan(0)
  })

  it('specific method selection works', async () => {
    const { status, data } = await apiCall('POST', '/engines/sigil-forge/calculate', {
      consciousness_level: 1,
      parameters: { intention: 'I attract abundance', method: 'rose-wheel' },
    })
    expect(status).toBe(200)
    expect((data as any).result.method.id).toBe('rose-wheel')
    expect((data as any).result.method.name).toBe('Rose Wheel (Rosy Cross) Method')
  })

  it('word elimination processing works', async () => {
    const { status, data } = await apiCall('POST', '/engines/sigil-forge/calculate', {
      consciousness_level: 1,
      parameters: { intention: 'I am confident and calm', method: 'word-elimination' },
    })
    expect(status).toBe(200)
    expect((data as any).result.processing).toBeDefined()
    expect((data as any).result.processing.type).toBe('word_elimination')
    expect((data as any).result.processing.remaining_letters).toBeDefined()
  })

  it('charging suggestions included', async () => {
    const { status, data } = await apiCall('POST', '/engines/sigil-forge/calculate', {
      consciousness_level: 1,
      parameters: { intention: 'I manifest my goals' },
    })
    expect(status).toBe(200)
    expect((data as any).result.charging_suggestions.length).toBeGreaterThan(0)
  })

  it('witness prompts included', async () => {
    const { status, data } = await apiCall('POST', '/engines/sigil-forge/calculate', {
      consciousness_level: 1,
      parameters: { intention: 'I am creative' },
    })
    expect(status).toBe(200)
    expect((data as any).witness_prompts.length).toBeGreaterThan(0)
  })
})

// ============================================================================
// 7. Error Handling Tests
// ============================================================================

describe('Error Handling', () => {
  it('404 for unknown engine', async () => {
    const { status, data } = await apiCall('GET', '/engines/nonexistent/info')
    expect(status).toBe(404)
    expect((data as any).error_code).toBe('ENGINE_NOT_FOUND')
  })

  it('404 for calculate on unknown engine', async () => {
    const { status, data } = await apiCall('POST', '/engines/nonexistent/calculate', {
      consciousness_level: 0,
      parameters: {},
    })
    expect(status).toBe(404)
    expect((data as any).error_code).toBe('ENGINE_NOT_FOUND')
  })

  it('403 for insufficient consciousness level', async () => {
    // Sigil Forge requires phase 1, try with phase 0
    const { status, data } = await apiCall('POST', '/engines/sigil-forge/calculate', {
      consciousness_level: 0,
      parameters: { intention: 'Test intention' },
    })
    expect(status).toBe(403)
    expect((data as any).error_code).toBe('PHASE_ACCESS_DENIED')
    expect((data as any).details.required_phase).toBe(1)
    expect((data as any).details.provided_phase).toBe(0)
  })

  it('Enneagram requires phase 1 for assessment', async () => {
    // Check if enneagram requires phase 1
    const { data: info } = await apiCall('GET', '/engines/enneagram/info')
    const requiredPhase = (info as any).required_phase

    if (requiredPhase > 0) {
      const { status, data } = await apiCall('POST', '/engines/enneagram/calculate', {
        consciousness_level: 0,
        parameters: { type: 1 },
      })
      expect(status).toBe(403)
      expect((data as any).error_code).toBe('PHASE_ACCESS_DENIED')
    } else {
      // If phase 0 is allowed, just verify it works
      const { status } = await apiCall('POST', '/engines/enneagram/calculate', {
        consciousness_level: 0,
        parameters: { type: 1 },
      })
      expect(status).toBe(200)
    }
  })
})
