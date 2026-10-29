/**
 * Enneagram provenance / confidence / degradation probes (GH #1311, W3E enneagram:07).
 *
 * The engine has two ways to name a type: scoring the 45-question assessment,
 * or echoing a type the caller supplied. These probes pin that callers can
 * tell the two apart from the output alone (result.typeSource and
 * provenance.backend_id), and that a direct lookup never carries an
 * assessment confidence.
 *
 * All answers below are synthetic.
 */
import { afterEach, describe, expect, it } from 'bun:test'
import { readFileSync } from 'node:fs'
import { join } from 'node:path'
import { EnneagramEngine } from '../src/engines/enneagram'
import type { EngineInput, EngineOutput } from '../src/types'
import { EngineValidationError } from '../src/utils'

const QUESTION_COUNT = 45

// Synthetic answer sheet: neutral 3s with the five type-7 questions (indices 30-34) at 5.
function type7Answers(): number[] {
  const answers = Array(QUESTION_COUNT).fill(3)
  for (let i = 30; i < 35; i++) answers[i] = 5
  return answers
}

function run(parameters: Record<string, unknown>): Promise<EngineOutput> {
  const input: EngineInput = { consciousness_level: 1, parameters, seed: 7 }
  return new EnneagramEngine().calculate(input)
}

const schema = JSON.parse(
  readFileSync(join(import.meta.dir, '../../contracts/v1/schemas/provenance.schema.json'), 'utf8'),
) as {
  required: string[]
  properties: Record<string, { enum?: string[]; minimum?: number; maximum?: number }>
}

function expectSchemaConformant(provenance: EngineOutput['provenance']): void {
  expect(provenance).toBeDefined()
  const block = provenance as unknown as Record<string, unknown>
  for (const key of schema.required) expect(block[key]).toBeDefined()
  for (const key of Object.keys(block)) expect(Object.keys(schema.properties)).toContain(key)
  expect(schema.properties.runtime_kind.enum).toContain(block.runtime_kind as string)
  if (block.confidence !== undefined) {
    expect(block.confidence as number).toBeGreaterThanOrEqual(0)
    expect(block.confidence as number).toBeLessThanOrEqual(1)
  }
}

describe('enneagram provenance: happy path', () => {
  it('assessment: assessment-derived type carries the assessment confidence', async () => {
    const output = await run({ answers: type7Answers() })
    const result = output.result as Record<string, any>

    expect(result.mode).toBe('assessment')
    expect(result.typeSource).toBe('assessment')
    expect(result.assessment.primaryType.number).toBe(7)
    expect(result.assessment.answeredCount).toBe(QUESTION_COUNT)
    expect(result.assessment.questionCount).toBe(QUESTION_COUNT)
    expect(result.assessment.clampedCount).toBe(0)

    expectSchemaConformant(output.provenance)
    expect(output.provenance).toEqual({
      runtime_kind: 'typescript',
      implementation_version: '1.0.0',
      cached: false,
      fallback_used: false,
      backend_id: 'enneagram-assessment',
      confidence: result.assessment.confidence,
    })
    expect(output.provenance?.confidence).toBeGreaterThan(0)
  })

  it('direct lookup: user-supplied type carries no confidence at all', async () => {
    const output = await run({ type: 4, wing: 5 })
    const result = output.result as Record<string, any>

    expect(result.mode).toBe('lookup')
    expect(result.typeSource).toBe('user-supplied')
    expect(result.assessment).toBeUndefined()
    expect(result.suppliedWing).toBe(5)

    expectSchemaConformant(output.provenance)
    expect(output.provenance).toEqual({
      runtime_kind: 'typescript',
      implementation_version: '1.0.0',
      cached: false,
      fallback_used: false,
      backend_id: 'enneagram-type-lookup',
    })
    expect('confidence' in (output.provenance ?? {})).toBe(false)
  })

  it('assessment and lookup naming the same type stay distinguishable', async () => {
    const assessed = await run({ answers: type7Answers() })
    const looked = await run({ type: 7 })

    expect((assessed.result as any).typeAnalysis.type.number).toBe(7)
    expect((looked.result as any).typeAnalysis.type.number).toBe(7)
    expect((assessed.result as any).typeSource).not.toBe((looked.result as any).typeSource)
    expect(assessed.provenance?.backend_id).not.toBe(looked.provenance?.backend_id)
    expect(looked.provenance?.confidence).toBeUndefined()
  })

  it('answers win over a supplied type, and the output says so', async () => {
    const output = await run({ answers: type7Answers(), type: 2 })
    const result = output.result as Record<string, any>

    expect(result.typeSource).toBe('assessment')
    expect(result.typeAnalysis.type.number).toBe(7)
    expect(output.provenance?.backend_id).toBe('enneagram-assessment')
  })

  it('question catalogue and empty request name no type and claim no confidence', async () => {
    for (const parameters of [{ includeAssessment: true }, {}]) {
      const output = await run(parameters)
      const result = output.result as Record<string, any>

      expect(result.mode).toBe('questions')
      expect(result.typeSource).toBe('none')
      expectSchemaConformant(output.provenance)
      expect(output.provenance?.backend_id).toBe('enneagram-question-catalogue')
      expect(output.provenance?.confidence).toBeUndefined()
      expect(output.provenance?.fallback_used).toBe(false)
    }
  })
})

describe('enneagram provenance: boundary', () => {
  it('a partial answer sheet scales provenance confidence by coverage', async () => {
    // Only the first 35 answers: type-8 and type-9 questions are unanswered.
    const full = await run({ answers: type7Answers() })
    const partial = await run({ answers: type7Answers().slice(0, 35) })
    const result = partial.result as Record<string, any>

    expect(result.typeSource).toBe('assessment')
    expect(result.assessment.answeredCount).toBe(35)
    expect(result.assessment.questionCount).toBe(QUESTION_COUNT)
    expect(partial.provenance?.confidence).toBe(
      Math.round(result.assessment.confidence * (35 / QUESTION_COUNT) * 100) / 100,
    )
    expect(partial.provenance?.confidence as number).toBeLessThan(
      full.provenance?.confidence as number,
    )
  })

  it('answers beyond question 45 are ignored and not counted', async () => {
    const output = await run({ answers: [...type7Answers(), 5, 5, 5] })
    expect((output.result as any).assessment.answeredCount).toBe(QUESTION_COUNT)
  })

  it('a flat answer sheet reports zero confidence, not a confident type', async () => {
    const output = await run({ answers: Array(QUESTION_COUNT).fill(3) })
    expect((output.result as any).assessment.confidence).toBe(0)
    expect(output.provenance?.confidence).toBe(0)
  })

  it('out-of-scale answers are clamped and the clamp is reported', async () => {
    const answers = type7Answers()
    answers[0] = 0
    answers[1] = 9
    const output = await run({ answers })
    expect((output.result as any).assessment.clampedCount).toBe(2)
  })
})

describe('enneagram provenance: unavailable dependency', () => {
  const realFetch = globalThis.fetch
  afterEach(() => {
    globalThis.fetch = realFetch
  })

  it('has no external dependency: identical output with the network down, never a fallback', async () => {
    const baseline = await run({ answers: type7Answers() })
    globalThis.fetch = (() => {
      throw new Error('network unavailable (probe)')
    }) as unknown as typeof fetch

    const offline = await run({ answers: type7Answers() })
    expect(offline.result).toEqual(baseline.result)
    expect(offline.provenance).toEqual(baseline.provenance)
    expect(offline.provenance?.fallback_used).toBe(false)
    expect(offline.provenance?.provider_id).toBeUndefined()
  })
})

describe('enneagram provenance: invalid input', () => {
  it.each([
    ['a non-array', 'not-an-array'],
    ['a non-numeric entry', [...Array(44).fill(3), 'x']],
    ['a NaN entry', [...Array(44).fill(3), Number.NaN]],
  ])('rejects answers given as %s with INVALID_ENNEAGRAM_ANSWERS', async (_label, answers) => {
    const error = await run({ answers }).catch((e) => e)
    expect(error).toBeInstanceOf(EngineValidationError)
    expect(error.code).toBe('INVALID_ENNEAGRAM_ANSWERS')
  })

  it('rejects an out-of-range direct type with INVALID_ENNEAGRAM_TYPE', async () => {
    const error = await run({ type: 10 }).catch((e) => e)
    expect(error).toBeInstanceOf(EngineValidationError)
    expect(error.code).toBe('INVALID_ENNEAGRAM_TYPE')
  })
})
