import { afterAll, beforeAll, describe, expect, it } from 'bun:test'
import type { ImageProvider } from '../../providers/image-provider'
import {
  type GeneratedImage,
  type ImageGenOptions,
  MockImageProvider,
  createImageProvider,
} from '../../providers/image-provider' // T-035 provider tests
import { KimiImageProvider, buildYantraPrompt } from '../../providers/kimi' // T-061 yantra templates
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
    delete process.env[key]
  }
})

afterAll(() => {
  for (const key of PROVIDER_ENV_KEYS) {
    const value = savedProviderEnv.get(key)
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
    metadata?: {
      provider?: string
      finish_reason?: string
      provider_requested?: string
      provider_note?: string
    }
  } | null
  image_gen_available?: boolean
  provider_note?: string
  guidance?: { note: string; next_steps: string[] }
}

const TINY_PNG =
  'iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg=='

/** A stand-in for the real nvidia provider: available, returns a "real" image. */
class FakeNvidiaProvider implements ImageProvider {
  readonly name = 'nvidia'
  isAvailable(): boolean {
    return true
  }
  async generate(opts: ImageGenOptions): Promise<GeneratedImage> {
    return {
      b64_json: TINY_PNG,
      metadata: {
        model: opts.model ?? 'black-forest-labs/flux.1-dev',
        prompt: opts.prompt,
        provider: this.name,
        style: opts.style,
        seed: opts.seed,
        finish_reason: 'SUCCESS',
      },
    }
  }
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
    // A mock image is never surfaced as a real top-level generated_image.
    expect(output.generated_image).toBeUndefined()
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
    // Mock output is a test double: kept in result, never promoted to the FROZEN top-level slot.
    expect(output.generated_image).toBeUndefined()
    const res = output.result as SigilForgeResult
    expect(res.generated_image).toBeDefined()
    expect(res.provider).toBe('mock')
    if (!res.generated_image) {
      throw new Error('mock provider did not return generated_image')
    }
    expect(res.generated_image.b64_json).toBeDefined()
    expect(res.image_gen_available).toBe(true)
    expect(output.provenance).toEqual({
      runtime_kind: 'typescript',
      implementation_version: '2.0.0',
      cached: false,
      fallback_used: true,
      provider_id: 'mock',
      confidence: 0,
    })
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
    // ...and provenance says so: nothing generated => fallback_used, no provider_id
    expect(output.provenance?.fallback_used).toBe(true)
    expect(output.provenance?.provider_id).toBeUndefined()
    expect(output.provenance?.confidence).toBe(0)
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

  it('config-only switch to nano-banana is honest without RUNCOMFY_TOKEN (T-060)', async () => {
    const prov = createImageProvider({ provider: 'nano-banana' })
    const engine = new SigilForgeEngine(prov)
    const output = await engine.calculate({
      consciousness_level: 1,
      parameters: { intention: 'nano test', generate_image: true },
    })
    const res = output.result as SigilForgeResult
    expect(res.provider).toBe('nano-banana')
    expect(res.image_gen_available).toBe(false)
    expect(output.generated_image).toBeUndefined()
    expect(res.generated_image?.error).toContain('not configured')
    expect(output.provenance?.fallback_used).toBe(true)
    expect(output.provenance?.provider_id).toBeUndefined()
  })

  // T-061 kimi provider + yantra prompt templates
  it('config-only switch to kimi provider reports unavailable without KIMI_API_KEY', async () => {
    const prov = createImageProvider({ provider: 'kimi' })
    expect(prov.isAvailable()).toBe(false)
    const engine = new SigilForgeEngine(prov)
    const output = await engine.calculate({
      consciousness_level: 1,
      parameters: {
        intention: 'I manifest clarity through sacred form',
        generate_image: true,
        image_style: 'yantra',
      },
    })
    const res = output.result as SigilForgeResult
    expect(res.provider).toBe('kimi')
    expect(output.generated_image).toBeUndefined()
    expect(res.generated_image?.error).toBeDefined()
    expect(res.generated_image?.b64_json).toBeUndefined()
    expect(output.provenance?.fallback_used).toBe(true)
  })

  it('kimi never fabricates a placeholder PNG even with a key', async () => {
    const prov = new KimiImageProvider({ provider: 'kimi', apiKey: 'test-key' })
    expect(prov.isAvailable()).toBe(true)
    const res = await prov.generate({ prompt: 'yantra' })
    expect(res.b64_json).toBeUndefined()
    expect(res.url).toBeUndefined()
    expect(res.mock).toBe(true)
    expect(res.error).toBeDefined()
    expect(res.metadata.finish_reason).toBe('KIMI_IMAGE_ENDPOINT_UNAVAILABLE')
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

  it('isAvailable is false without RUNCOMFY_TOKEN', () => {
    expect(isNanoBananaAvailable()).toBe(false)
    expect(createNanoBananaProvider().isAvailable()).toBe(false)
  })

  it('generate returns an honest error (no placeholder PNG) without a token', async () => {
    const p = new NanoBananaImageProvider()
    const res = await p.generate({ prompt: 'test sigil runic style', style: 'runic', seed: 42 })
    expect(res.metadata.provider).toBe('nano-banana')
    expect(res.metadata.model).toContain('nano')
    expect(res.b64_json).toBeUndefined()
    expect(res.url).toBeUndefined()
    expect(res.mock).toBe(true)
    expect(res.error).toContain('RUNCOMFY_TOKEN')
    expect(res.metadata.finish_reason).toBe('MOCK_NO_TOKEN')
  })

  it('edit path returns the same honest error without a token', async () => {
    const p = new NanoBananaImageProvider()
    const res = await p.edit({
      image: TINY_PNG,
      prompt: 'refine sigil',
      instruction: 'more geometric',
    })
    expect(res.metadata.provider).toBe('nano-banana')
    expect(res.b64_json).toBeUndefined()
    expect(res.mock).toBe(true)
  })

  it('sigil engine + unconfigured nano provider keeps the error inside result', async () => {
    const prov = createNanoBananaProvider()
    const engine = new SigilForgeEngine(prov)
    const output = await engine.calculate({
      consciousness_level: 1,
      parameters: { intention: 'I seal the working', generate_image: true, image_style: 'runic' },
    })
    expect(output.generated_image).toBeUndefined()
    const res = output.result as SigilForgeResult
    expect(res.provider).toBe('nano-banana')
    expect(res.generated_image?.error).toBeDefined()
    expect(res.generated_image?.metadata?.finish_reason).toBe('PROVIDER_UNAVAILABLE')
  })
})

// ============================================================================
// Lane S4 (2026-10-26): provenance + honesty contract
// ============================================================================

describe('SigilForgeEngine provenance (lane S4)', () => {
  const sevenFields = [
    'runtime_kind',
    'implementation_version',
    'cached',
    'fallback_used',
    'backend_id',
    'provider_id',
    'confidence',
  ]

  it('mocked nvidia success: provider_id nvidia, fallback_used false, top-level image present', async () => {
    const engine = new SigilForgeEngine(new FakeNvidiaProvider())
    const output = await engine.calculate({
      consciousness_level: 1,
      parameters: { intention: 'I see clearly', generate_image: true },
      seed: 7,
    })
    expect(output.provenance).toEqual({
      runtime_kind: 'typescript',
      implementation_version: '2.0.0',
      cached: false,
      fallback_used: false,
      provider_id: 'nvidia',
      confidence: 0.9,
    })
    for (const key of Object.keys(output.provenance ?? {})) {
      expect(sevenFields).toContain(key)
    }
    expect(output.generated_image?.b64_json).toBe(TINY_PNG)
    expect(output.generated_image?.metadata?.provider).toBe('nvidia')
    expect(output.generated_image?.metadata?.finish_reason).toBe('SUCCESS')
    const res = output.result as SigilForgeResult
    expect(res.generated_image?.metadata?.finish_reason).toBe('SUCCESS')
    expect(res.generated_image?.error).toBeUndefined()
    expect(res.guidance?.note).toContain('AI-generated sigil image included')
    expect(res.guidance?.next_steps[0]).toContain('generated image')
  })

  it('no key: fallback_used true, no top-level image, error present, guidance is honest', async () => {
    const engine = new SigilForgeEngine(createImageProvider({ provider: 'nvidia' }))
    const output = await engine.calculate({
      consciousness_level: 1,
      parameters: { intention: 'I see clearly', generate_image: true },
      seed: 7,
    })
    expect(output.provenance?.fallback_used).toBe(true)
    expect(output.provenance?.confidence).toBe(0)
    expect(output.provenance?.provider_id).toBeUndefined()
    expect(output.generated_image).toBeUndefined()
    const res = output.result as SigilForgeResult
    expect(res.generated_image?.error).toContain('not configured')
    expect(res.generated_image?.metadata?.finish_reason).toBe('PROVIDER_UNAVAILABLE')
    expect(res.guidance?.note).not.toContain('AI-generated sigil image included')
    expect(res.guidance?.note).toContain('no real image was produced')
    expect(res.guidance?.next_steps[0]).toContain('Gather your materials')
  })

  it('MockImageProvider: provider_id mock, fallback_used true', async () => {
    const engine = new SigilForgeEngine(new MockImageProvider())
    const output = await engine.calculate({
      consciousness_level: 1,
      parameters: { intention: 'I see clearly', generate_image: true },
    })
    expect(output.provenance?.provider_id).toBe('mock')
    expect(output.provenance?.fallback_used).toBe(true)
    expect(output.provenance?.confidence).toBe(0)
    expect(output.generated_image).toBeUndefined()
    const res = output.result as SigilForgeResult
    expect(res.generated_image?.metadata?.finish_reason).toBe('SUCCESS_MOCK')
  })

  it('provider error: fallback_used true and the error is kept under result.generated_image', async () => {
    const failing: ImageProvider = {
      name: 'nvidia',
      isAvailable: () => true,
      generate: async () => {
        throw new Error('NVIDIA NIM image generation failed (503)')
      },
    }
    const engine = new SigilForgeEngine(failing)
    const output = await engine.calculate({
      consciousness_level: 1,
      parameters: { intention: 'I see clearly', generate_image: true },
    })
    expect(output.generated_image).toBeUndefined()
    expect(output.provenance?.fallback_used).toBe(true)
    expect(output.provenance?.provider_id).toBeUndefined()
    const res = output.result as SigilForgeResult
    expect(res.generated_image?.error).toContain('503')
    expect(res.generated_image?.metadata?.finish_reason).toBe('ERROR')
  })

  it('parameters.provider is honoured only when configured; otherwise default + note', async () => {
    const engine = new SigilForgeEngine(new FakeNvidiaProvider())
    const output = await engine.calculate({
      consciousness_level: 1,
      parameters: { intention: 'I see clearly', generate_image: true, provider: 'kimi' },
    })
    const res = output.result as SigilForgeResult
    expect(res.provider).toBe('nvidia')
    expect(res.provider_note).toContain("'kimi' is not configured")
    expect(res.generated_image?.metadata?.provider_requested).toBe('kimi')
    expect(res.generated_image?.metadata?.provider_note).toContain('kimi')
    expect(output.provenance?.provider_id).toBe('nvidia')
    expect(output.provenance?.fallback_used).toBe(false)
  })

  it('unknown parameters.provider falls back to the default with a note', async () => {
    const engine = new SigilForgeEngine(new FakeNvidiaProvider())
    const output = await engine.calculate({
      consciousness_level: 1,
      parameters: { intention: 'I see clearly', generate_image: true, provider: 'dall-e' },
    })
    const res = output.result as SigilForgeResult
    expect(res.provider).toBe('nvidia')
    expect(res.provider_note).toContain("Unknown provider 'dall-e'")
  })

  it('metadata input_schema advertises the provider parameter', () => {
    const schema = new SigilForgeEngine(new MockImageProvider()).metadata().input_schema
    expect(schema.provider?.enum).toEqual(['nvidia', 'nano-banana', 'kimi'])
  })
})
