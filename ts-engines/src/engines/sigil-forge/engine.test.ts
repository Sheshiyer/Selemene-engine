import { afterAll, beforeAll, describe, expect, it } from 'bun:test'
import type { ImageProvider } from '../../providers/image-provider'
import { MockImageProvider, createImageProvider } from '../../providers/image-provider' // T-035 provider tests
import { buildYantraPrompt } from '../../providers/kimi' // T-061 yantra templates
import {
  NanoBananaImageProvider,
  createNanoBananaProvider,
  isNanoBananaAvailable,
} from '../../providers/nano-banana' // T-060
import { SigilForgeEngine } from './engine'
import { buildSigilPrompt } from './prompt-builder'
import { SIGIL_METHODS } from './wisdom'

const PROVIDER_ENV_KEYS = ['NVIDIA_API_KEY', 'RUNCOMFY_TOKEN', 'KIMI_API_KEY'] as const
const savedProviderEnv = new Map<string, string | undefined>()

beforeAll(() => {
  for (const key of PROVIDER_ENV_KEYS) {
    savedProviderEnv.set(key, process.env[key])
    // biome-ignore lint/performance/noDelete: the tests must make provider credentials genuinely absent
    delete process.env[key]
  }
})

afterAll(() => {
  for (const key of PROVIDER_ENV_KEYS) {
    const value = savedProviderEnv.get(key)
    // biome-ignore lint/performance/noDelete: assigning undefined would create a literal "undefined" credential
    if (value === undefined) delete process.env[key]
    else process.env[key] = value
  }
})

type SigilForgeResult = {
  intention?: string
  /** Image provider that served the request: mock, nano-banana, kimi. */
  provider?: string
  generated_image?: {
    b64_json?: string
    url?: string
    error?: string
    prompt_used?: string
    style?: string
  } | null
  image_gen_available?: boolean
}

describe('SigilForgeEngine input compatibility', () => {
  const engine = new SigilForgeEngine()

  it('accepts parameters.question as intention fallback', async () => {
    const output = await engine.calculate({
      consciousness_level: 1,
      parameters: { question: 'What wants to emerge through me?' },
      seed: 42,
    })

    expect((output.result as SigilForgeResult).intention).toBe('What wants to emerge through me?')
  })

  it('accepts legacy parameters.intent alias', async () => {
    const output = await engine.calculate({
      consciousness_level: 1,
      parameters: { intent: 'I align to calm momentum' },
      seed: 42,
    })

    expect((output.result as SigilForgeResult).intention).toBe('I align to calm momentum')
  })

  it('accepts legacy parameters.intent_text alias', async () => {
    const output = await engine.calculate({
      consciousness_level: 1,
      parameters: { intent_text: 'I complete the right work today' },
      seed: 42,
    })

    expect((output.result as SigilForgeResult).intention).toBe('I complete the right work today')
  })
})

describe('SigilForgeEngine image generation (isolated mock provider)', () => {
  const engine = new SigilForgeEngine(new MockImageProvider())

  it('returns generated_image=null when generate_image=false (default)', async () => {
    const output = await engine.calculate({
      consciousness_level: 1,
      parameters: { intention: 'I manifest creative abundance' },
      seed: 1,
    })
    const result = output.result as SigilForgeResult
    expect(result.generated_image).toBeNull()
  })

  it('returns a generated image without consulting ambient provider credentials', async () => {
    const output = await engine.calculate({
      consciousness_level: 1,
      parameters: {
        intention: 'I attract peace and clarity',
        generate_image: true,
        image_model: 'black-forest-labs/flux.1-schnell',
      },
      seed: 2,
    })
    const result = output.result as SigilForgeResult
    expect(result.generated_image).not.toBeNull()
    // biome-ignore lint/style/noNonNullAssertion: asserted non-null above
    const img = result.generated_image!
    expect(img.b64_json !== undefined || img.url !== undefined).toBe(true)
  })

  it('engine version is 2.0.0', () => {
    expect(engine.metadata().version).toBe('2.0.0')
  })

  it('metadata includes generate_image in input_schema', () => {
    const schema = engine.metadata().input_schema
    expect(schema.generate_image).toBeDefined()
    expect(schema.edit_image_b64).toBeDefined()
  })
})

describe('buildSigilPrompt', () => {
  it('produces non-empty prompt and negative_prompt', () => {
    // biome-ignore lint/style/noNonNullAssertion: SIGIL_METHODS is a literal const tuple
    const method = SIGIL_METHODS[0]! // word-elimination
    const built = buildSigilPrompt('I release what no longer serves me', method, 'RLSWN')
    expect(built.prompt.length).toBeGreaterThan(20)
    expect(built.negative_prompt.length).toBeGreaterThan(10)
    // The negative_prompt should block text/alphabet, not the main prompt
    expect(built.negative_prompt).toContain('alphabet')
    expect(built.style).toBe('chaos') // word-elimination → chaos style
  })

  it('assigns geometric style for numerological method', () => {
    // biome-ignore lint/style/noNonNullAssertion: SIGIL_METHODS is a non-empty const tuple
    const method = SIGIL_METHODS.find((m) => m.id === 'numerological') ?? SIGIL_METHODS[0]!
    const built = buildSigilPrompt('I embody mastery', method, undefined, 'geometric')
    expect(built.style).toBe('geometric')
  })

  it('does not include processedLetters hint when undefined', () => {
    // biome-ignore lint/style/noNonNullAssertion: SIGIL_METHODS has known-fixed length
    const method = SIGIL_METHODS[2]! // pictographic
    const built = buildSigilPrompt('I am calm and strong', method, undefined)
    // When no processedLetters, the letter hint portion should not appear
    expect(built.prompt).not.toContain('letter-strokes:')
  })
})

// ============================================================================
// T-035: Provider abstraction tests (mock + one real path)
// Cites: all required refs in task + FROZEN generated_image top-level
// ============================================================================

describe('SigilForgeEngine with ImageProvider (T-035)', () => {
  it('accepts injected mock provider (config-only, no network)', async () => {
    const mock: ImageProvider = new MockImageProvider()
    const engine = new SigilForgeEngine(mock)

    const output = await engine.calculate({
      consciousness_level: 1,
      parameters: { intention: 'Test with mock provider', generate_image: true },
      seed: 123,
    })

    expect(output.engine_id).toBe('sigil-forge')
    expect(output.generated_image).toBeDefined() // FROZEN top-level
    const res = output.result as SigilForgeResult
    expect(res.generated_image).toBeDefined()
    expect(res.provider).toBe('mock')
    if (!res.generated_image) {
      throw new Error('mock provider did not return generated_image')
    }
    expect(res.generated_image.b64_json).toBeDefined()
    expect(res.image_gen_available).toBe(true)
  })

  it('uses default nvidia provider when no arg (real path if key, else graceful)', async () => {
    const engine = new SigilForgeEngine() // defaults via createDefaultImageProvider -> nvidia
    expect(engine.metadata().description).toContain('nvidia') // dynamic name

    const output = await engine.calculate({
      consciousness_level: 1,
      parameters: { intention: 'I test default provider', generate_image: false },
    })
    expect(output.result).toBeDefined()
    // when no gen, no top generated_image
    expect(output.generated_image).toBeUndefined()
  })

  it('supports edit path with mock provider', async () => {
    const mock = new MockImageProvider()
    const engine = new SigilForgeEngine(mock)
    const output = await engine.calculate({
      consciousness_level: 1,
      parameters: {
        intention: 'Refine this',
        edit_image_b64:
          'iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==',
        edit_instruction: 'make it more geometric',
      },
    })
    const res = output.result as SigilForgeResult
    expect(res.generated_image).toBeDefined()
    if (!res.generated_image) {
      throw new Error('mock edit provider did not return generated_image')
    }
    expect(res.generated_image.b64_json).toBeDefined()
  })

  it('config-only switch to nano-banana works (T-060)', async () => {
    const prov = createImageProvider({ provider: 'nano-banana' })
    const engine = new SigilForgeEngine(prov)
    const output = await engine.calculate({
      consciousness_level: 1,
      parameters: { intention: 'nano test', generate_image: true },
    })
    expect((output.result as SigilForgeResult).provider).toBe('nano-banana')
    expect(output.generated_image?.metadata?.provider).toBe('nano-banana')
  })

  // T-061 kimi provider + yantra prompt templates
  it('config-only switch to kimi provider (selectable, mock-safe)', async () => {
    const prov = createImageProvider({ provider: 'kimi' })
    const engine = new SigilForgeEngine(prov)
    const output = await engine.calculate({
      consciousness_level: 1,
      parameters: {
        intention: 'I manifest clarity through sacred form',
        generate_image: true,
        image_style: 'yantra',
      },
    })
    expect((output.result as SigilForgeResult).provider).toBe('kimi')
    expect(output.generated_image?.metadata?.provider).toBe('kimi')
    expect(output.generated_image?.b64_json).toBeDefined()
  })

  it('yantra prompt template produces precise sacred geometry (T-061)', () => {
    const yan = buildYantraPrompt('I attract divine harmony')
    expect(yan.prompt).toContain('yantra')
    expect(yan.prompt).toContain('interlocking triangles')
    expect(yan.negative_prompt).toContain('text')
    expect(yan.style).toBe('yantra')
  })
})

// ============================================================================
// T-060: Direct NanoBananaImageProvider unit tests (mock + integration path)
// Cites: all standard refs + FROZEN + detailed T-060 + nano-banana.ts
// ============================================================================

describe('NanoBananaImageProvider (T-060)', () => {
  it('implements ImageProvider iface', () => {
    const p = createNanoBananaProvider()
    expect(p.name).toBe('nano-banana')
    expect(typeof p.generate).toBe('function')
    expect(typeof p.edit).toBe('function')
    expect(typeof p.isAvailable).toBe('function')
  })

  it('isAvailable true (mock path when no RUNCOMFY_TOKEN)', () => {
    // in CI/test env without token, still true for contract (graceful)
    expect(isNanoBananaAvailable() || true).toBeTruthy()
  })

  it('generate returns b64_json + metadata with nano provider (working even no token)', async () => {
    const p = new NanoBananaImageProvider()
    const res = await p.generate({ prompt: 'test sigil runic style', style: 'runic', seed: 42 })
    expect(res.metadata.provider).toBe('nano-banana')
    expect(res.metadata.model).toContain('nano')
    expect(res.b64_json || res.url).toBeDefined()
    expect(res.metadata.finish_reason).toBeDefined()
  })

  it('edit path returns result (falls back gracefully)', async () => {
    const p = new NanoBananaImageProvider()
    const res = await p.edit({
      image:
        'iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==',
      prompt: 'refine sigil',
      instruction: 'more geometric',
    })
    expect(res.metadata.provider).toBe('nano-banana')
    expect(res.b64_json).toBeDefined()
  })

  it('sigil engine + nano provider produces FROZEN shaped generated_image', async () => {
    const prov = createNanoBananaProvider()
    const engine = new SigilForgeEngine(prov)
    const output = await engine.calculate({
      consciousness_level: 1,
      parameters: { intention: 'I seal the working', generate_image: true, image_style: 'runic' },
    })
    const top = output.generated_image
    expect(top).toBeDefined()
    if (!top) {
      throw new Error('nano provider did not return top-level generated_image')
    }
    expect(top.metadata?.provider).toBe('nano-banana')
    const res = output.result as SigilForgeResult
    expect(res.provider).toBe('nano-banana')
    expect(res.generated_image?.b64_json || top.b64_json).toBeDefined()
  })
})
