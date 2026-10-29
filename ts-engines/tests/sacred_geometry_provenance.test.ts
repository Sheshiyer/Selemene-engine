/**
 * Sacred Geometry provenance / confidence / degradation probes (GH #1401).
 *
 * The engine's output mixes three different kinds of truth, and callers must
 * be able to tell them apart:
 *   - symbolic lookup  : form data read from the static SACRED_FORMS table
 *   - calculated geometry : computed per form (GH #1524); forms without a
 *                           faithful construction stay not_implemented
 *   - rendered artifact   : none today -- svg_preview only validates a
 *                           caller-supplied template; nothing is rendered
 *
 * Synthetic inputs only; no birth data.
 */
import { describe, expect, it } from 'bun:test'
import provenanceSchema from '../../contracts/v1/schemas/provenance.schema.json'
import { SacredGeometryEngine } from '../src/engines/sacred-geometry'
import type { EngineInput } from '../src/types'
import { EngineValidationError } from '../src/utils'

const engine = new SacredGeometryEngine()
const bridgeFixturePath = new URL(
  './fixtures/sacred-geometry/bridge-round-trip-seed-1401.json',
  import.meta.url,
)

function input(parameters: Record<string, unknown>, seed = 1401): EngineInput {
  return { consciousness_level: 0, parameters, seed }
}

// biome-ignore lint/suspicious/noExplicitAny: probing an engine-specific result shape
type AnyResult = any

describe('SacredGeometryEngine provenance (GH #1401)', () => {
  it('probe:happy-path - requested form is a symbolic lookup with a schema-valid provenance block', async () => {
    const output = await engine.calculate(input({ form: 'flower-of-life' }))
    const result = output.result as AnyResult

    expect(output.provenance).toEqual({
      runtime_kind: 'typescript',
      implementation_version: engine.metadata().version,
      cached: false,
      fallback_used: false,
      confidence: 1,
    })
    // Only keys the contracts/v1 provenance schema allows (additionalProperties: false).
    const allowed = Object.keys(provenanceSchema.properties)
    for (const key of Object.keys(output.provenance ?? {})) {
      expect(allowed).toContain(key)
    }
    for (const key of provenanceSchema.required) {
      expect(output.provenance).toHaveProperty(key)
    }

    const detail = result.provenance_detail
    expect(detail.form).toEqual({ kind: 'symbolic_lookup', source: 'wisdom.SACRED_FORMS' })
    expect(detail.form_selection).toEqual({ kind: 'requested' })
    expect(detail.meditation).toEqual({ kind: 'symbolic_lookup', source: 'wisdom.SACRED_FORMS' })
    expect(detail.calculated_geometry.status).toBe('calculated')
    expect(typeof detail.calculated_geometry.construction).toBe('string')
    expect(detail.rendered_artifact.status).toBe('not_implemented')
    expect(typeof detail.rendered_artifact.reason).toBe('string')
    expect(detail.svg_preview.kind).toBe('caller_template_validation')
    expect(detail.witness_prompts.kind).toBe('templated_narrative')
  })

  it('probe:boundary - omitted form is a seeded selection, deterministic for the same seed', async () => {
    const a = await engine.calculate(input({}, 0))
    const b = await engine.calculate(input({}, 0))
    const ra = a.result as AnyResult
    const rb = b.result as AnyResult

    expect(ra.form.id).toBe(rb.form.id)
    expect(ra.provenance_detail.form_selection).toEqual({ kind: 'seeded_random', seed: 0 })
    // Selecting the form at random does not make the form data itself random.
    expect(ra.provenance_detail.form.kind).toBe('symbolic_lookup')
    expect(a.provenance?.fallback_used).toBe(false)
    expect(a.provenance?.confidence).toBe(1)
  })

  it('probe:unavailable-dependency - rejected svg_template is not replaced by a fallback render', async () => {
    const output = await engine.calculate(
      input({ form: 'torus', svg_template: '<svg><script>x()</script></svg>' }),
    )
    const result = output.result as AnyResult

    expect(result.svg_preview.status).toBe('rejected')
    expect(result.provenance_detail.svg_preview).toEqual({
      kind: 'caller_template_validation',
      status: 'rejected',
    })
    // No substitute artifact, no silent degradation.
    expect(result.provenance_detail.rendered_artifact.status).toBe('not_implemented')
    expect(output.provenance?.fallback_used).toBe(false)
    expect(result.form.id).toBe('torus')
  })

  it('probe:invalid-input - unknown form fails with INVALID_SACRED_FORM and no partial output', async () => {
    let error: unknown
    try {
      await engine.calculate(input({ form: 'hypercube' }))
    } catch (e) {
      error = e
    }
    expect(error).toBeInstanceOf(EngineValidationError)
    expect((error as EngineValidationError).code).toBe('INVALID_SACRED_FORM')
  })

  it('probe:serialization - provenance and provenance_detail survive a JSON round trip', async () => {
    const output = await engine.calculate(input({ form: 'sri-yantra' }))
    const roundTripped = JSON.parse(JSON.stringify(output))

    expect(output.provenance).toBeDefined()
    expect((output.result as AnyResult).provenance_detail).toBeDefined()

    expect(roundTripped.provenance).toEqual(output.provenance)
    expect(roundTripped.result.provenance_detail).toEqual(
      (output.result as AnyResult).provenance_detail,
    )
  })

  it('probe:bridge-fixture - committed sidecar fixture matches live engine output', async () => {
    // crates/noesis-bridge replays this exact file through ts_response_into_output
    // (sacred_geometry_sidecar_fixture_round_trips_provenance_and_detail), so
    // the Rust probe can only pass against what this engine really emits.
    const fixture = JSON.parse(await Bun.file(bridgeFixturePath).text())
    const live = await engine.calculate(fixture.input)

    expect({
      ...live,
      calculated_at: fixture.sidecar_response.calculated_at,
      processing_time_ms: fixture.sidecar_response.processing_time_ms,
    }).toEqual(fixture.sidecar_response)
    expect(fixture.sidecar_response.result.provenance_detail.form_selection.kind).toBe(
      'seeded_random',
    )
    expect(fixture.sidecar_response.result.provenance_detail.svg_preview.status).toBe('rejected')
  })
})
