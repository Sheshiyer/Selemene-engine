import { describe, expect, it } from 'bun:test'
import { SigilForgeEngine } from './engine'

type SigilForgeResult = {
  intention?: string
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
