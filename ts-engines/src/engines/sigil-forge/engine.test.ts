import { describe, expect, it } from 'bun:test'
import { SigilForgeEngine } from './engine'
import { buildSigilPrompt } from './prompt-builder'
import { SIGIL_METHODS } from './wisdom'

type SigilForgeResult = {
  intention?: string
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

describe('SigilForgeEngine image generation (guidance-only, no API key in test env)', () => {
  const engine = new SigilForgeEngine()

  it('returns generated_image=null when generate_image=false (default)', async () => {
    const output = await engine.calculate({
      consciousness_level: 1,
      parameters: { intention: 'I manifest creative abundance' },
      seed: 1,
    })
    const result = output.result as SigilForgeResult
    expect(result.generated_image).toBeNull()
  })

  it('returns error in generated_image or a valid result when generate_image=true', async () => {
    const output = await engine.calculate({
      consciousness_level: 1,
      parameters: { intention: 'I attract peace and clarity', generate_image: true },
      seed: 2,
    })
    const result = output.result as SigilForgeResult
    // Either the image was generated (key available) or an error was returned (no key)
    // Either way, generated_image should be a non-null object
    expect(result.generated_image).not.toBeNull()
    // Must have either a result or a graceful error — never throw
    const img = result.generated_image!
    const hasResult = img.b64_json !== undefined || img.url !== undefined
    const hasError = img.error !== undefined
    expect(hasResult || hasError).toBe(true)
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
    const method = SIGIL_METHODS[0]! // word-elimination
    const built = buildSigilPrompt('I release what no longer serves me', method, 'RLSWN')
    expect(built.prompt.length).toBeGreaterThan(20)
    expect(built.negative_prompt.length).toBeGreaterThan(10)
    // The negative_prompt should block text/alphabet, not the main prompt
    expect(built.negative_prompt).toContain('alphabet')
    expect(built.style).toBe('chaos') // word-elimination → chaos style
  })

  it('assigns geometric style for numerological method', () => {
    const method = SIGIL_METHODS.find(m => m.id === 'numerological') ?? SIGIL_METHODS[0]!
    const built = buildSigilPrompt('I embody mastery', method, undefined, 'geometric')
    expect(built.style).toBe('geometric')
  })

  it('does not include processedLetters hint when undefined', () => {
    const method = SIGIL_METHODS[2]! // pictographic
    const built = buildSigilPrompt('I am calm and strong', method, undefined)
    // When no processedLetters, the letter hint portion should not appear
    expect(built.prompt).not.toContain('letter-strokes:')
  })
})
