/**
 * Sacred Geometry witness-boundary probes (GH #1411).
 *
 * Witness prompts are templated narrative layered on top of the deterministic
 * result. These probes check that the narrative stays auditable beneath it:
 * every prompt names its template family, whether it is an inquiry or
 * guidance, and the exact `result.*` fields it was built from; and the
 * narrative never feeds back into the deterministic output.
 *
 * Synthetic inputs only; no birth data.
 */
import { describe, expect, it } from 'bun:test'
import { SacredGeometryEngine } from '../src/engines/sacred-geometry'
import type { EngineInput, EngineOutput } from '../src/types'
import { EngineValidationError } from '../src/utils'

const engine = new SacredGeometryEngine()

// biome-ignore lint/suspicious/noExplicitAny: probing an engine-specific result shape
type AnyResult = any

function input(parameters: Record<string, unknown>, seed?: number): EngineInput {
  return { consciousness_level: 0, parameters, ...(seed === undefined ? {} : { seed }) }
}

function resolve(output: EngineOutput, path: string): unknown {
  return path.split('.').reduce<unknown>((value, key) => {
    return value == null ? undefined : (value as Record<string, unknown>)[key]
  }, output)
}

/** Fields each template family must name, because its text is built from them. */
const REQUIRED_INPUTS: Record<string, string[]> = {
  form_contemplation: ['result.form.name'],
  meditation: ['result.meditation.prompt'],
  intention: ['result.intention', 'result.form.name'],
  numerology: ['result.form.numerology', 'result.form.name'],
  integration: ['result.form.name'],
}

/** Every derivation entry must point at real fields whose values appear in its prompt. */
function expectAuditable(output: EngineOutput) {
  const result = output.result as AnyResult
  const derivation = result.provenance_detail.witness_prompts
  expect(derivation.kind).toBe('templated_narrative')
  expect(derivation.source).toBe('witness.generateWitnessPrompts')
  expect(derivation.seed).toBe(result.seed)
  const prompts = output.witness_prompts ?? []
  expect(prompts.length).toBeGreaterThan(0)
  expect(derivation.prompts).toHaveLength(prompts.length)

  derivation.prompts.forEach((entry: AnyResult, i: number) => {
    const text = prompts[i].prompt
    expect([
      'form_contemplation',
      'meditation',
      'intention',
      'numerology',
      'integration',
    ]).toContain(entry.template)
    expect(entry.mode).toBe(entry.template === 'meditation' ? 'guidance' : 'inquiry')
    if (entry.mode === 'inquiry') expect(text.trim().endsWith('?')).toBe(true)
    expect(entry.inputs.length).toBeGreaterThan(0)
    for (const required of REQUIRED_INPUTS[entry.template]) {
      expect(entry.inputs).toContain(required)
    }
    for (const path of entry.inputs as string[]) {
      expect(path.startsWith('result.')).toBe(true)
      const value = resolve(output, path)
      expect(value).toBeDefined()
      const shown = Array.isArray(value) ? value.slice(0, 3) : [value]
      for (const part of shown) expect(text).toContain(String(part))
    }
  })
}

/** The deterministic part of a result: everything except narrative and its echo. */
function deterministic(output: EngineOutput) {
  const { intention: _intention, provenance_detail, ...rest } = output.result as AnyResult
  const { witness_prompts: _narrative, ...detail } = provenance_detail
  return { ...rest, provenance_detail: detail }
}

describe('SacredGeometryEngine witness boundary (GH #1411)', () => {
  it('probe:happy-path - every witness prompt is traceable to deterministic result fields', async () => {
    const output = await engine.calculate(
      input({ form: 'sri-yantra', intention: 'steady attention' }, 1411),
    )
    const result = output.result as AnyResult

    expect(output.witness_prompts).toHaveLength(3)
    expectAuditable(output)
    const templates = result.provenance_detail.witness_prompts.prompts.map(
      (entry: AnyResult) => entry.template,
    )
    expect(templates).toEqual(['form_contemplation', 'meditation', 'intention'])

    // Guidance inside the deterministic result is labelled, not passed off as inquiry.
    expect(result.provenance_detail.meditation).toEqual({
      kind: 'symbolic_lookup',
      source: 'wisdom.SACRED_FORMS',
      mode: 'guidance',
      duration_suggestion: 'static_constant',
    })
  })

  it('probe:template-coverage - every template family is audited across seeded forms', async () => {
    const seen = new Set<string>()
    // Seeds are spread over the uint32 range like getDefaultSeed(): small
    // seeds bias SeededRandom's early draws, so integration is rare there.
    for (let i = 0; i < 200 && seen.size < 5; i++) {
      const seed = Math.imul(i + 1, 2654435761) >>> 0
      for (const intention of [undefined, 'a synthetic intention']) {
        const output = await engine.calculate(input(intention ? { intention } : {}, seed))
        expectAuditable(output)
        for (const entry of (output.result as AnyResult).provenance_detail.witness_prompts
          .prompts) {
          seen.add(entry.template)
        }
      }
    }
    expect([...seen].sort()).toEqual(Object.keys(REQUIRED_INPUTS).sort())
  })

  it('probe:separation - narrative inputs never change the deterministic output', async () => {
    const seed = 1411
    const withA = await engine.calculate(input({ form: 'torus', intention: 'rest' }, seed))
    const withB = await engine.calculate(input({ form: 'torus', intention: 'motion' }, seed))
    const without = await engine.calculate(input({ form: 'torus' }, seed))

    expect(deterministic(withA)).toEqual(deterministic(without))
    expect(deterministic(withB)).toEqual(deterministic(without))
    expect(withA.witness_prompts).not.toEqual(withB.witness_prompts)
    for (const output of [withA, withB, without]) expectAuditable(output)
  })

  it('probe:boundary - template tokens inside an intention are echoed verbatim, not filled', async () => {
    const intention = '{form} and {number}'
    const output = await engine.calculate(input({ form: 'cube', intention }, 7))
    const result = output.result as AnyResult

    const i = result.provenance_detail.witness_prompts.prompts.findIndex(
      (entry: AnyResult) => entry.template === 'intention',
    )
    expect(i).toBe(2)
    expect(output.witness_prompts?.[i]?.prompt).toContain(`"${intention}"`)
    expect(result.provenance_detail.witness_prompts.prompts[i].inputs).toEqual([
      'result.intention',
      'result.form.name',
    ])
    expectAuditable(output)
  })

  it('probe:unavailable-dependency - with no seed or a rejected svg the narrative is still replayable', async () => {
    const unseeded = await engine.calculate(input({}))
    const seed = (unseeded.result as AnyResult).seed
    expect(typeof seed).toBe('number')

    const replay = await engine.calculate(
      input({ svg_template: '<svg><script>x()</script></svg>' }, seed),
    )
    expect((replay.result as AnyResult).svg_preview.status).toBe('rejected')
    expect(replay.witness_prompts).toEqual(unseeded.witness_prompts)
    expect((replay.result as AnyResult).provenance_detail.witness_prompts).toEqual(
      (unseeded.result as AnyResult).provenance_detail.witness_prompts,
    )
    expect(replay.provenance?.fallback_used).toBe(false)
    expectAuditable(replay)
  })

  it('probe:invalid-input - an unknown form fails before any narrative is produced', async () => {
    let error: unknown
    try {
      await engine.calculate(input({ form: 'hypercube', intention: 'anything' }, 1))
    } catch (e) {
      error = e
    }
    expect(error).toBeInstanceOf(EngineValidationError)
    expect((error as EngineValidationError).code).toBe('INVALID_SACRED_FORM')
  })
})
