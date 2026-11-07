/**
 * Sri Yantra construction (GH #1532).
 *
 * The engine builds a concurrent Sri Yantra after Chiodo, "On the construction
 * of the Śrī Yantra", C. R. Math. 359 (2021) 377-397, doi:10.5802/crmath.163.
 * These probes re-check the paper's defining conditions (i)-(iii) on the
 * returned numbers alone, with the condition lists copied from the paper
 * rather than imported from the engine, and cross-check two values Huet
 * published for the same parameters (TCS 281 (2002); Chiodo fn. 17 and 20).
 *
 * Synthetic inputs only; no birth data.
 */
import { describe, expect, it } from 'bun:test'
import { SacredGeometryEngine } from '../src/engines/sacred-geometry'
import type { EngineInput } from '../src/types'

const engine = new SacredGeometryEngine()

// biome-ignore lint/suspicious/noExplicitAny: probing an engine-specific result shape
type AnyResult = any

/** Output is rounded to 6 decimals, so conditions hold to a few 1e-6. */
const TOL = 1e-5

// Chiodo sec. 1, condition (ii): apex of t' is the base point of t''.
const APEX_ON_BASE: [number, number][] = [
  [8, 1],
  [6, 2],
  [9, 3],
  [1, 6],
  [5, 7],
  [4, 8],
  [2, 9],
]

// Chiodo sec. 1, condition (iii): leg of downward t', base of t'', leg of upward t'''.
const CONCURRENT: [number, number, number][] = [
  [1, 2, 7],
  [2, 3, 7],
  [1, 3, 8],
  [1, 4, 6],
  [1, 5, 9],
  [4, 6, 9],
  [2, 7, 9],
  [3, 7, 8],
  [3, 8, 9],
  [4, 4, 8],
  [5, 5, 6],
  [2, 6, 6],
]

interface Tri {
  baseY: number
  half: number
  apexX: number
  apexY: number
}

async function sriYantra(seed = 1532) {
  const input: EngineInput = { consciousness_level: 0, parameters: { form: 'sri-yantra' }, seed }
  const result = (await engine.calculate(input)).result as AnyResult
  const tris: Tri[] = result.geometry.polygons.map((p: AnyResult) => {
    const [left, right, apex] = p.points
    expect(left[1]).toBe(right[1])
    expect(left[0]).toBe(-right[0])
    return { baseY: right[1], half: right[0], apexX: apex[0], apexY: apex[1] }
  })
  const t = (i: number) => tris[i - 1]
  /** x of t_i's right leg at height y. */
  const leg = (i: number, y: number) => (t(i).half * (y - t(i).apexY)) / (t(i).baseY - t(i).apexY)
  return { result, geometry: result.geometry, tris, t, leg }
}

describe('Sri Yantra construction (GH #1532)', () => {
  it('nine triangles: t1-t5 point down, t6-t9 point up, bases ordered top to bottom', async () => {
    const { tris, geometry } = await sriYantra()
    expect(tris).toHaveLength(9)
    tris.forEach((tri, i) => {
      expect(tri.apexX).toBe(0)
      expect(tri.half).toBeGreaterThan(0)
      if (i < 5) expect(tri.apexY).toBeLessThan(tri.baseY)
      else expect(tri.apexY).toBeGreaterThan(tri.baseY)
      if (i > 0) expect(tri.baseY).toBeLessThan(tris[i - 1].baseY)
    })
    expect(geometry.measures.downward_count).toBe(5)
    expect(geometry.measures.upward_count).toBe(4)
  })

  it('condition (i): t3 and t7 are inscribed in the outer circle', async () => {
    const { t, geometry } = await sriYantra()
    expect(geometry.circles).toEqual([{ cx: 0, cy: 0, r: 1 }])
    for (const i of [3, 7]) {
      expect(Math.abs(Math.hypot(t(i).half, t(i).baseY) - 1)).toBeLessThan(TOL)
      expect(Math.abs(Math.abs(t(i).apexY) - 1)).toBeLessThan(TOL)
    }
  })

  it('condition (ii): each of the seven apexes is a base point', async () => {
    const { t } = await sriYantra()
    for (const [apexOf, baseOf] of APEX_ON_BASE) {
      expect(Math.abs(t(apexOf).apexY - t(baseOf).baseY)).toBeLessThan(TOL)
    }
  })

  it('condition (iii): the twelve triples of lines meet in one point on the base', async () => {
    const { t, leg } = await sriYantra()
    for (const [down, base, up] of CONCURRENT) {
      const y = t(base).baseY
      const xDown = leg(down, y)
      const xUp = leg(up, y)
      expect(Math.abs(xDown - xUp)).toBeLessThan(TOL)
      expect(xDown).toBeLessThanOrEqual(t(base).half + TOL)
    }
  })

  it('every triangle lies inside the circle', async () => {
    const { tris } = await sriYantra()
    for (const tri of tris) {
      expect(Math.hypot(tri.half, tri.baseY)).toBeLessThanOrEqual(1 + TOL)
      expect(Math.abs(tri.apexY)).toBeLessThanOrEqual(1 + TOL)
    }
  })

  it("matches Huet's published concurrency half-widths for the same parameters", async () => {
    // Huet gives X_F = 0.126 for ((iii); t1,t3,t8) and X_A = 0.187 for
    // ((iii); t3,t8,t9), in units of the diameter (Chiodo fn. 16-17).
    const { t, leg, geometry } = await sriYantra()
    const xF = leg(1, t(3).baseY) / 2
    const xA = leg(3, t(8).baseY) / 2
    expect(Math.abs(xF - 0.126)).toBeLessThan(5e-4)
    expect(Math.abs(xA - 0.187)).toBeLessThan(5e-4)
    expect(Math.abs(geometry.measures.huet_x_f - xF)).toBeLessThan(TOL)
    expect(Math.abs(geometry.measures.huet_x_a - xA)).toBeLessThan(TOL)
  })

  it('uses the stated parameters and solves the remaining equation to machine precision', async () => {
    const { t, geometry } = await sriYantra()
    const m = geometry.measures
    expect([m.param_base_t3, m.param_base_t6, m.param_base_t7, m.param_base_t9]).toEqual([
      0.332, 0.537, 0.602, 0.835,
    ])
    // Diameter fraction from the top -> y in the unit circle.
    const y = (s: number) => 1 - 2 * s
    expect(t(3).baseY).toBeCloseTo(y(0.332), 6)
    expect(t(6).baseY).toBeCloseTo(y(0.537), 6)
    expect(t(7).baseY).toBeCloseTo(y(0.602), 6)
    expect(t(9).baseY).toBeCloseTo(y(0.835), 6)
    expect(m.max_condition_residual).toBeLessThan(1e-12)
    expect(geometry.construction).toContain('10.5802/crmath.163')
  })

  it('the bindu is the incentre of the central triangle t1 ∩ t5', async () => {
    const { t, leg, geometry } = await sriYantra()
    expect(geometry.points).toHaveLength(1)
    const [bx, by] = geometry.points[0]
    expect(bx).toBe(0)
    // Central triangle: base of t5 on top, legs of t1 down to t1's apex.
    const topY = t(5).baseY
    const halfTop = leg(1, topY)
    expect(halfTop).toBeLessThanOrEqual(t(5).half + TOL)
    expect(by).toBeLessThan(topY)
    expect(by).toBeGreaterThan(t(1).apexY)
    // Equidistant from the top edge and from t1's leg.
    const toTop = topY - by
    const apexY = t(1).apexY
    const toLeg = (halfTop * (by - apexY)) / Math.hypot(halfTop, topY - apexY)
    expect(Math.abs(toTop - toLeg)).toBeLessThan(TOL)
  })

  it('provenance is calculated and names the source', async () => {
    const { result, geometry } = await sriYantra()
    const detail = result.provenance_detail.calculated_geometry
    expect(detail.status).toBe('calculated')
    expect(detail.construction).toBe(geometry.construction)
    expect(detail.computed.polygons).toBe(9)
    expect(result.provenance_detail.rendered_artifact.status).toBe('not_implemented')
  })

  it('is identical across calls and seeds', async () => {
    const a = (await sriYantra(1)).geometry
    const b = (await sriYantra(99)).geometry
    expect(JSON.stringify(a)).toBe(JSON.stringify(b))
  })
})
