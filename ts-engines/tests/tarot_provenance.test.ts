/**
 * Tarot provenance/confidence truth-surface test (GH #1461, Task 4 partial slice).
 *
 * Proves the missing-state: tarot's calculate() currently returns a full
 * reading with no signal of whether card interpretation text came from the
 * deterministic wisdom data vs a fallback path, and no confidence value at
 * all. This test asserts the minimal truth surface the plan's Step 1-4
 * calls for: an explicit provenance block with fallback_used and a
 * confidence value for the deterministic wisdom-data path.
 */
import { describe, expect, it } from 'bun:test'
import { TarotEngine } from '../src/engines/tarot'
import type { EngineInput } from '../src/types'

describe('TarotEngine provenance (GH #1461 partial slice)', () => {
  it('reports provenance with fallback_used=false and a confidence value for the deterministic wisdom-data path', async () => {
    const engine = new TarotEngine()
    const input: EngineInput = {
      consciousness_level: 0,
      parameters: { spread: 'single_card' },
      seed: 42,
    }

    const output = await engine.calculate(input)

    expect(output.provenance).toBeDefined()
    expect(output.provenance?.runtime_kind).toBe('typescript')
    expect(output.provenance?.fallback_used).toBe(false)
    expect(output.provenance?.cached).toBe(false)
    expect(typeof output.provenance?.implementation_version).toBe('string')
    expect(output.provenance?.confidence).toBe(1)
  })
})
