/**
 * Sacred Geometry construction data (GH #1524).
 *
 * Every form id the engine accepts either returns numeric construction data
 * under result.geometry (status `calculated`) or null geometry with a reason
 * (status `not_implemented`). The engine never returns SVG or HTML to render.
 *
 * Synthetic inputs only; no birth data.
 */
import { describe, expect, it } from 'bun:test'
import {
  SacredGeometryEngine,
  getConstructedFormIds,
  getFormIds,
  getUnsupportedFormIds,
} from '../src/engines/sacred-geometry'
import geometrySchema from '../src/engines/sacred-geometry/geometry.schema.json'
import type { EngineInput } from '../src/types'

const engine = new SacredGeometryEngine()

function input(form: string, seed = 1524): EngineInput {
  return { consciousness_level: 0, parameters: { form }, seed }
}

// biome-ignore lint/suspicious/noExplicitAny: probing an engine-specific result shape
type AnyResult = any
// biome-ignore lint/suspicious/noExplicitAny: walking an untyped JSON schema
type Schema = any

const EPS = 1e-5

/**
 * Validates `value` against the subset of JSON Schema 2020-12 that
 * geometry.schema.json uses. Returns a list of error paths (empty = valid).
 */
function validate(schema: Schema, value: unknown, path = '$', root: Schema = schema): string[] {
  if (schema.$ref) {
    const name = String(schema.$ref).replace('#/$defs/', '')
    return validate(root.$defs[name], value, path, root)
  }
  const errors: string[] = []
  if ('const' in schema && value !== schema.const) errors.push(`${path}: const`)
  if (schema.type) {
    const types: string[] = Array.isArray(schema.type) ? schema.type : [schema.type]
    const actual =
      value === null
        ? 'null'
        : Array.isArray(value)
          ? 'array'
          : Number.isInteger(value) && types.includes('integer')
            ? 'integer'
            : typeof value
    if (!types.includes(actual)) return [`${path}: type ${actual} not in ${types.join('|')}`]
  }
  if (value === null) return errors
  if (typeof value === 'string' && schema.minLength && value.length < schema.minLength) {
    errors.push(`${path}: minLength`)
  }
  if (typeof value === 'number') {
    if ('minimum' in schema && value < schema.minimum) errors.push(`${path}: minimum`)
    if ('exclusiveMinimum' in schema && value <= schema.exclusiveMinimum) {
      errors.push(`${path}: exclusiveMinimum`)
    }
  }
  if (Array.isArray(value)) {
    if ('minItems' in schema && value.length < schema.minItems) errors.push(`${path}: minItems`)
    if ('maxItems' in schema && value.length > schema.maxItems) errors.push(`${path}: maxItems`)
    if (schema.items) {
      value.forEach((item, i) => {
        errors.push(...validate(schema.items, item, `${path}[${i}]`, root))
      })
    }
  } else if (typeof value === 'object') {
    const obj = value as Record<string, unknown>
    for (const key of schema.required ?? []) {
      if (!(key in obj)) errors.push(`${path}: missing ${key}`)
    }
    for (const [key, child] of Object.entries(obj)) {
      const sub = schema.properties?.[key]
      if (sub) errors.push(...validate(sub, child, `${path}.${key}`, root))
      else if (schema.additionalProperties === false) errors.push(`${path}: extra ${key}`)
      else if (typeof schema.additionalProperties === 'object') {
        errors.push(...validate(schema.additionalProperties, child, `${path}.${key}`, root))
      }
    }
  }
  return errors
}

function allPoints(g: AnyResult): number[][] {
  return [
    ...g.points,
    ...g.polygons.flatMap((p: AnyResult) => p.points),
    ...g.polylines.flatMap((p: AnyResult) => p.points),
    ...g.lines.flatMap((l: AnyResult) => [
      [l.x1, l.y1],
      [l.x2, l.y2],
    ]),
  ]
}

describe('SacredGeometryEngine construction data (GH #1524)', () => {
  it('every accepted form id is either constructed or explicitly unsupported', () => {
    const constructed = getConstructedFormIds()
    const unsupported = getUnsupportedFormIds()
    expect([...constructed, ...unsupported].sort()).toEqual([...getFormIds()].sort())
    for (const id of unsupported) expect(constructed).not.toContain(id)
  })

  for (const id of getConstructedFormIds()) {
    it(`${id}: schema-valid, inside the unit circle, calculated provenance, deterministic`, async () => {
      const first = await engine.calculate(input(id))
      const result = first.result as AnyResult
      const g = result.geometry

      expect(validate(geometrySchema, g)).toEqual([])
      expect(g.unit).toBe('relative')
      expect(g.construction.length).toBeGreaterThan(20)
      expect(
        g.circles.length + g.polygons.length + g.polylines.length + g.lines.length,
      ).toBeGreaterThan(0)

      for (const [x, y] of allPoints(g)) {
        expect(Number.isFinite(x) && Number.isFinite(y)).toBe(true)
        expect(Math.hypot(x, y)).toBeLessThanOrEqual(1 + EPS)
      }
      for (const c of g.circles) {
        expect(Math.hypot(c.cx, c.cy) + c.r).toBeLessThanOrEqual(1 + EPS)
      }

      const detail = result.provenance_detail.calculated_geometry
      expect(detail.status).toBe('calculated')
      expect(detail.construction).toBe(g.construction)
      expect(detail.computed.circles).toBe(g.circles.length)
      expect(detail.computed.lines).toBe(g.lines.length)
      expect(result.provenance_detail.rendered_artifact.status).toBe('not_implemented')

      // Same form, different seed and a second call: identical geometry.
      const second = await engine.calculate(input(id, 99))
      expect(JSON.stringify((second.result as AnyResult).geometry)).toBe(JSON.stringify(g))
    })
  }

  for (const id of getUnsupportedFormIds()) {
    it(`${id}: no geometry, not_implemented with a reason`, async () => {
      const result = (await engine.calculate(input(id))).result as AnyResult
      expect(result.geometry).toBeNull()
      expect(validate(geometrySchema, result.geometry)).toEqual([])
      expect(result.provenance_detail.calculated_geometry.status).toBe('not_implemented')
      expect(result.provenance_detail.calculated_geometry.reason.length).toBeGreaterThan(20)
    })
  }

  it('flower-of-life: 19 equal circles on the hexagonal lattice', async () => {
    const g = ((await engine.calculate(input('flower-of-life'))).result as AnyResult).geometry
    expect(g.circles).toHaveLength(19)
    const r = g.circles[0].r
    for (const c of g.circles) expect(c.r).toBe(r)

    // Each centre is a lattice point a*(r,0) + b*(r/2, r*sqrt3/2) with integer
    // a, b within two steps of the origin.
    const keys = new Set<string>()
    for (const c of g.circles) {
      const b = (2 * c.cy) / (r * Math.sqrt(3))
      const a = c.cx / r - b / 2
      expect(Math.abs(a - Math.round(a))).toBeLessThan(EPS)
      expect(Math.abs(b - Math.round(b))).toBeLessThan(EPS)
      const ra = Math.round(a)
      const rb = Math.round(b)
      expect((Math.abs(ra) + Math.abs(rb) + Math.abs(ra + rb)) / 2).toBeLessThanOrEqual(2)
      keys.add(`${ra},${rb}`)
    }
    expect(keys.size).toBe(19)
  })

  it('seed-of-life: 7 equal circles, six passing through the centre', async () => {
    const g = ((await engine.calculate(input('seed-of-life'))).result as AnyResult).geometry
    expect(g.circles).toHaveLength(7)
    const r = g.circles[0].r
    for (const c of g.circles.slice(1)) {
      expect(c.r).toBe(r)
      expect(Math.abs(Math.hypot(c.cx, c.cy) - r)).toBeLessThan(EPS)
    }
  })

  it('metatrons-cube: 13 centres, every pair joined (78 lines)', async () => {
    const g = ((await engine.calculate(input('metatrons-cube'))).result as AnyResult).geometry
    expect(g.points).toHaveLength(13)
    expect(g.circles).toHaveLength(13)
    expect(new Set(g.points.map(String)).size).toBe(13)
    expect(g.lines).toHaveLength(78)
    expect(g.measures.centre_count).toBe(13)
  })

  it('vesica-piscis: each circle passes through the other centre; lens ratio sqrt(3)', async () => {
    const g = ((await engine.calculate(input('vesica-piscis'))).result as AnyResult).geometry
    const [a, b] = g.circles
    expect(Math.abs(Math.hypot(a.cx - b.cx, a.cy - b.cy) - a.r)).toBeLessThan(EPS)
    expect(Math.abs(g.measures.height_to_width - Math.sqrt(3))).toBeLessThan(EPS)
  })

  const SOLIDS: [string, number, number][] = [
    ['tetrahedron', 4, 6],
    ['cube', 8, 12],
    ['octahedron', 6, 12],
    ['dodecahedron', 20, 30],
    ['icosahedron', 12, 30],
  ]
  for (const [id, v, e] of SOLIDS) {
    it(`${id}: ${v} vertices on the unit sphere, ${e} equal edges, projected wireframe`, async () => {
      const g = ((await engine.calculate(input(id))).result as AnyResult).geometry
      expect(g.solid.vertices).toHaveLength(v)
      expect(g.solid.edges).toHaveLength(e)
      expect(g.lines).toHaveLength(e)
      expect(g.points).toHaveLength(v)
      for (const p of g.solid.vertices) expect(Math.abs(Math.hypot(...p) - 1)).toBeLessThan(EPS)
      const lengths = g.solid.edges.map(([i, j]: [number, number]) => {
        const p = g.solid.vertices[i]
        const q = g.solid.vertices[j]
        return Math.hypot(p[0] - q[0], p[1] - q[1], p[2] - q[2])
      })
      for (const l of lengths) expect(Math.abs(l - lengths[0])).toBeLessThan(EPS)
      // Euler: V - E + F = 2, with F faces for each solid.
      const faces = { tetrahedron: 4, cube: 6, octahedron: 8, dodecahedron: 12, icosahedron: 20 }
      expect(v - e + faces[id as keyof typeof faces]).toBe(2)
    })
  }

  it('golden-spiral: the sampled spiral passes through every whirling-square corner', async () => {
    const g = ((await engine.calculate(input('golden-spiral'))).result as AnyResult).geometry
    const spiral = g.polylines[0].points
    const perQuarter = g.measures.samples_per_quarter_turn
    const corners = g.points.slice(1)
    expect(corners).toHaveLength(g.measures.square_count + 1)
    corners.forEach((c: number[], k: number) => {
      const s = spiral[k * perQuarter]
      expect(Math.hypot(c[0] - s[0], c[1] - s[1])).toBeLessThan(EPS)
    })
    // Successive square sides shrink by 1/phi.
    const side = (k: number) => {
      const [p, q] = g.polygons[k + 1].points
      return Math.hypot(p[0] - q[0], p[1] - q[1])
    }
    expect(Math.abs(side(0) / side(1) - g.measures.phi)).toBeLessThan(1e-4)
  })

  it('torus: plan-view equators at R+r and R-r', async () => {
    const g = ((await engine.calculate(input('torus'))).result as AnyResult).geometry
    const radii = g.circles.map((c: AnyResult) => c.r).sort((a: number, b: number) => a - b)
    const { major_radius: R, tube_radius: r } = g.measures
    expect(radii).toEqual([R - r, R, R + r].map((x) => Math.round(x * 1e6) / 1e6))
  })

  it('no output contains SVG or HTML markup', async () => {
    for (const id of getFormIds()) {
      const output = await engine.calculate(input(id))
      const json = JSON.stringify(output)
      expect(json).not.toMatch(/<\s*\/?\s*[a-z!]/i)
      expect(json.toLowerCase()).not.toContain('svg>')
      expect(json).not.toMatch(/\b(?:viewBox|xmlns|<path|d=")/)
    }
  })
})
