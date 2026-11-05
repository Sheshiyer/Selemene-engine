/**
 * Sacred Geometry construction data (GH #1524).
 *
 * Every form is built from its geometric definition and returned as plain
 * numbers, so clients can draw it themselves. The engine never emits SVG or
 * HTML. The schema for this shape is `geometry.schema.json` next to this file.
 *
 * Frame: coordinates are relative, centred on the origin with y pointing up,
 * and every primitive lies inside the unit circle (|p| <= 1). Values are
 * rounded to 6 decimals so repeated calls serialise identically.
 */

export type Point2 = [number, number]
export type Point3 = [number, number, number]

export interface GeometryCircle {
  cx: number
  cy: number
  r: number
}

export interface GeometryLine {
  x1: number
  y1: number
  x2: number
  y2: number
}

/** A closed polygon; the last point connects back to the first. */
export interface GeometryPolygon {
  points: Point2[]
}

/** An open polyline (e.g. sampled points of a curve). */
export interface GeometryPolyline {
  points: Point2[]
}

export interface GeometrySolid {
  /** 3D vertices, scaled so the circumradius is 1. */
  vertices: Point3[]
  /** Vertex index pairs. */
  edges: [number, number][]
  /** Unit vector the solid is projected along (before the in-plane turn). */
  view_axis: Point3
}

export interface SacredGeometryConstruction {
  unit: 'relative'
  frame: { center: Point2; radius: 1; y_axis: 'up' }
  /** The construction rule that produced the numbers below. */
  construction: string
  circles: GeometryCircle[]
  polygons: GeometryPolygon[]
  polylines: GeometryPolyline[]
  lines: GeometryLine[]
  points: Point2[]
  /** Named numeric facts of the construction (ratios, counts, radii). */
  measures: Record<string, number>
  /** Present for the Platonic solids only. */
  solid?: GeometrySolid
}

/** Why a form has no construction, for forms that are not computed. */
export interface UnsupportedConstruction {
  reason: string
}

const PHI = (1 + Math.sqrt(5)) / 2
const SQRT3 = Math.sqrt(3)

function round(value: number): number {
  const v = Math.round(value * 1e6) / 1e6
  // Avoid -0, which serialises as 0 but compares differently.
  return v === 0 ? 0 : v
}

function pt(x: number, y: number): Point2 {
  return [round(x), round(y)]
}

function circle(cx: number, cy: number, r: number): GeometryCircle {
  return { cx: round(cx), cy: round(cy), r: round(r) }
}

function line(a: Point2, b: Point2): GeometryLine {
  return { x1: a[0], y1: a[1], x2: b[0], y2: b[1] }
}

function empty(construction: string): SacredGeometryConstruction {
  return {
    unit: 'relative',
    frame: { center: [0, 0], radius: 1, y_axis: 'up' },
    construction,
    circles: [],
    polygons: [],
    polylines: [],
    lines: [],
    points: [],
    measures: {},
  }
}

/**
 * Centres of the hexagonal circle lattice within `rings` steps of the origin,
 * spaced `spacing` apart, ordered ring by ring and counter-clockwise from +x.
 */
function hexLattice(rings: number, spacing: number): Point2[] {
  const out: { ring: number; angle: number; p: Point2 }[] = []
  for (let a = -rings; a <= rings; a++) {
    for (let b = -rings; b <= rings; b++) {
      const ring = (Math.abs(a) + Math.abs(b) + Math.abs(a + b)) / 2
      if (ring > rings) continue
      const x = spacing * (a + b / 2)
      const y = spacing * ((b * SQRT3) / 2)
      const angle = ring === 0 ? 0 : (Math.atan2(y, x) + 2 * Math.PI) % (2 * Math.PI)
      out.push({ ring, angle, p: [x, y] })
    }
  }
  out.sort((m, n) => m.ring - n.ring || m.angle - n.angle)
  return out.map(({ p }) => pt(p[0], p[1]))
}

function circleLattice(rings: number, construction: string): SacredGeometryConstruction {
  // The outermost circle reaches rings*r + r from the centre; fit it to 1.
  const r = 1 / (rings + 1)
  const g = empty(construction)
  const centres = hexLattice(rings, r)
  g.circles = centres.map(([x, y]) => circle(x, y, r))
  g.points = centres
  g.measures = { circle_count: centres.length, circle_radius: round(r), lattice_spacing: round(r) }
  return g
}

function flowerOfLife(): SacredGeometryConstruction {
  return circleLattice(
    2,
    'Flower of Life: 19 equal circles centred on the hexagonal lattice points within two steps of the centre; lattice spacing equals the radius, so each circle passes through its neighbours’ centres.',
  )
}

function seedOfLife(): SacredGeometryConstruction {
  return circleLattice(
    1,
    'Seed of Life: 7 equal circles, one at the centre and six centred on it at 60° steps, each passing through the centre.',
  )
}

function vesicaPiscis(): SacredGeometryConstruction {
  // Centres at (±r/2, 0); the outer edge reaches r/2 + r = 1.
  const r = 2 / 3
  const g = empty(
    'Vesica Piscis: two equal circles, each centred on the other’s circumference; the lens is bounded by their two intersection points.',
  )
  const left = pt(-r / 2, 0)
  const right = pt(r / 2, 0)
  const top = pt(0, (r * SQRT3) / 2)
  const bottom = pt(0, (-r * SQRT3) / 2)
  g.circles = [circle(left[0], left[1], r), circle(right[0], right[1], r)]
  g.points = [left, right, top, bottom]
  g.lines = [line(left, right), line(top, bottom)]
  g.measures = {
    circle_radius: round(r),
    centre_distance: round(r),
    lens_width: round(r),
    lens_height: round(r * SQRT3),
    height_to_width: round(SQRT3),
  }
  return g
}

function metatronsCube(): SacredGeometryConstruction {
  // Fruit of Life: inner ring at distance d, outer ring at 2d on the same rays,
  // tangent circles of radius d/2. The outer edge reaches 2d + d/2 = 1.
  const d = 0.4
  const r = d / 2
  const g = empty(
    "Metatron's Cube: the 13 centres of the Fruit of Life (centre, six at distance d and six at 2d on the same rays at 60° steps, circles of radius d/2), with every pair of centres joined by a straight line (78 lines).",
  )
  const centres: Point2[] = [pt(0, 0)]
  for (const dist of [d, 2 * d]) {
    for (let k = 0; k < 6; k++) {
      const a = Math.PI / 2 + (k * Math.PI) / 3
      centres.push(pt(dist * Math.cos(a), dist * Math.sin(a)))
    }
  }
  g.circles = centres.map(([x, y]) => circle(x, y, r))
  g.points = centres
  for (let i = 0; i < centres.length; i++) {
    for (let j = i + 1; j < centres.length; j++) {
      g.lines.push(line(centres[i], centres[j]))
    }
  }
  g.measures = {
    centre_count: centres.length,
    line_count: g.lines.length,
    circle_radius: round(r),
    inner_ring_distance: round(d),
    outer_ring_distance: round(2 * d),
  }
  return g
}

// --- Platonic solids -------------------------------------------------------

type Vec3 = [number, number, number]

const sub = (a: Vec3, b: Vec3): Vec3 => [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
const dot = (a: Vec3, b: Vec3): number => a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
const norm = (a: Vec3): number => Math.sqrt(dot(a, a))
const scale = (a: Vec3, s: number): Vec3 => [a[0] * s, a[1] * s, a[2] * s]
const cross = (a: Vec3, b: Vec3): Vec3 => [
  a[1] * b[2] - a[2] * b[1],
  a[2] * b[0] - a[0] * b[2],
  a[0] * b[1] - a[1] * b[0],
]

/** Every sign combination of the non-zero coordinates of `v`. */
function signs(v: Vec3): Vec3[] {
  let out: Vec3[] = [[0, 0, 0]]
  for (let i = 0; i < 3; i++) {
    const options = v[i] === 0 ? [0] : [v[i], -v[i]]
    out = out.flatMap((p) => options.map((c) => p.map((x, j) => (j === i ? c : x)) as Vec3))
  }
  return out
}

/** Cyclic permutations (x,y,z), (y,z,x), (z,x,y). */
function cyclic(v: Vec3): Vec3[] {
  return [v, [v[1], v[2], v[0]], [v[2], v[0], v[1]]]
}

interface SolidSpec {
  name: string
  vertices: Vec3[]
  axis: Vec3
  axisLabel: string
}

const SOLIDS: Record<string, SolidSpec> = {
  tetrahedron: {
    name: 'Tetrahedron',
    vertices: [
      [1, 1, 1],
      [1, -1, -1],
      [-1, 1, -1],
      [-1, -1, 1],
    ],
    axis: [1, 1, 1],
    axisLabel: 'a vertex (3-fold axis)',
  },
  cube: {
    name: 'Cube',
    vertices: signs([1, 1, 1]),
    axis: [1, 1, 1],
    axisLabel: 'a body diagonal (3-fold axis)',
  },
  octahedron: {
    name: 'Octahedron',
    vertices: cyclic([1, 0, 0]).flatMap(signs),
    axis: [1, 1, 1],
    axisLabel: 'a face normal (3-fold axis)',
  },
  dodecahedron: {
    name: 'Dodecahedron',
    vertices: [...signs([1, 1, 1]), ...cyclic([0, 1 / PHI, PHI]).flatMap(signs)],
    // (0, φ, 1) has dot product φ^2 with exactly five vertices, the pentagonal
    // face (0,1/φ,φ), (±1,1,1), (±1/φ,φ,0): a 5-fold axis.
    axis: [0, PHI, 1],
    axisLabel: 'a face normal (5-fold axis)',
  },
  icosahedron: {
    name: 'Icosahedron',
    vertices: cyclic([0, 1, PHI]).flatMap(signs),
    axis: [0, 1, PHI],
    axisLabel: 'a vertex (5-fold axis)',
  },
}

/** Rotation matrix taking unit vector `a` onto +z (Rodrigues). */
function rotationToZ(a: Vec3): (v: Vec3) => Vec3 {
  const z: Vec3 = [0, 0, 1]
  const k = cross(a, z)
  const s = norm(k)
  const c = dot(a, z)
  if (s < 1e-12) {
    return c > 0 ? (v) => v : (v) => [v[0], -v[1], -v[2]]
  }
  const u = scale(k, 1 / s)
  return (v) => {
    // v cos + (u x v) sin + u (u.v)(1 - cos)
    const uxv = cross(u, v)
    const udv = dot(u, v)
    return [
      v[0] * c + uxv[0] * s + u[0] * udv * (1 - c),
      v[1] * c + uxv[1] * s + u[1] * udv * (1 - c),
      v[2] * c + uxv[2] * s + u[2] * udv * (1 - c),
    ]
  }
}

function platonicSolid(id: string): SacredGeometryConstruction {
  const spec = SOLIDS[id]
  const circumradius = norm(spec.vertices[0])
  const vertices = spec.vertices.map((v) => scale(v, 1 / circumradius))

  // Edges join the vertex pairs at the minimum distance.
  const pairs: { i: number; j: number; d: number }[] = []
  for (let i = 0; i < vertices.length; i++) {
    for (let j = i + 1; j < vertices.length; j++) {
      pairs.push({ i, j, d: norm(sub(vertices[i], vertices[j])) })
    }
  }
  const edgeLength = Math.min(...pairs.map((p) => p.d))
  const edges = pairs
    .filter((p) => Math.abs(p.d - edgeLength) < 1e-9)
    .map((p) => [p.i, p.j] as [number, number])

  // Orthographic projection along the view axis, then an in-plane turn that
  // puts the first outermost projected vertex straight up, scaled so that
  // vertex sits on the unit circle.
  const axis = scale(spec.axis, 1 / norm(spec.axis))
  const rotate = rotationToZ(axis)
  const flat = vertices.map((v) => {
    const r = rotate(v)
    return [r[0], r[1]] as [number, number]
  })
  const radii = flat.map(([x, y]) => Math.hypot(x, y))
  const maxR = Math.max(...radii)
  const top = flat[radii.findIndex((r) => Math.abs(r - maxR) < 1e-9)]
  const turn = Math.PI / 2 - Math.atan2(top[1], top[0])
  const cosT = Math.cos(turn)
  const sinT = Math.sin(turn)
  const k = 1 / maxR
  const projected = flat.map(([x, y]) => pt(k * (x * cosT - y * sinT), k * (x * sinT + y * cosT)))

  const g = empty(
    `${spec.name}: the ${vertices.length} vertices from their canonical coordinates, scaled to circumradius 1, with edges joining vertex pairs at the minimum distance (${edges.length} edges); drawn as an orthographic wireframe projection along ${spec.axisLabel}, turned so an outermost vertex points up and scaled to touch the unit circle.`,
  )
  g.points = projected
  g.lines = edges.map(([i, j]) => line(projected[i], projected[j]))
  g.measures = {
    vertex_count: vertices.length,
    edge_count: edges.length,
    edge_length: round(edgeLength),
    circumradius: 1,
    projection_scale: round(k),
  }
  g.solid = {
    vertices: vertices.map((v) => [round(v[0]), round(v[1]), round(v[2])]),
    edges,
    view_axis: [round(axis[0]), round(axis[1]), round(axis[2])],
  }
  return g
}

// --- Golden spiral ---------------------------------------------------------

function goldenSpiral(): SacredGeometryConstruction {
  // Golden rectangle [0,φ]x[0,1]. Cutting the unit square off its left side
  // leaves a rectangle similar to it: S(z) = c*z + t with c = -i/φ (a
  // clockwise quarter turn and a 1/φ scale) and t = 1 + i. The spiral's pole
  // is S's fixed point, and the logarithmic spiral O + (z0 - O) c^s passes
  // exactly through every square's corner S^k(0) at integer s.
  const squares = 8
  const samplesPerQuarter = 16
  type C = [number, number]
  const cmul = (a: C, b: C): C => [a[0] * b[0] - a[1] * b[1], a[0] * b[1] + a[1] * b[0]]
  const cadd = (a: C, b: C): C => [a[0] + b[0], a[1] + b[1]]
  const cdiv = (a: C, b: C): C => {
    const den = b[0] * b[0] + b[1] * b[1]
    return [(a[0] * b[0] + a[1] * b[1]) / den, (a[1] * b[0] - a[0] * b[1]) / den]
  }
  const c: C = [0, -1 / PHI]
  const t: C = [1, 1]
  const S = (z: C): C => cadd(cmul(c, z), t)
  const pole = cdiv(t, [1 - c[0], -c[1]])

  // Normalise: centre the outer rectangle on the origin, half-diagonal = 1.
  const half = Math.hypot(PHI, 1) / 2
  const fit = (z: C): Point2 => pt((z[0] - PHI / 2) / half, (z[1] - 0.5) / half)

  const rect: C[] = [
    [0, 0],
    [PHI, 0],
    [PHI, 1],
    [0, 1],
  ]
  let square: C[] = [
    [0, 0],
    [1, 0],
    [1, 1],
    [0, 1],
  ]
  const polygons: GeometryPolygon[] = [{ points: rect.map(fit) }]
  const corners: Point2[] = []
  let corner: C = [0, 0]
  for (let k = 0; k < squares; k++) {
    polygons.push({ points: square.map(fit) })
    corners.push(fit(corner))
    square = square.map(S)
    corner = S(corner)
  }
  corners.push(fit(corner))

  // c^s = φ^-s * e^{-iπ s/2}
  const z0: C = [0 - pole[0], 0 - pole[1]]
  const spiral: Point2[] = []
  for (let n = 0; n <= squares * samplesPerQuarter; n++) {
    const s = n / samplesPerQuarter
    const mag = PHI ** -s
    const ang = (-Math.PI * s) / 2
    const cs: C = [mag * Math.cos(ang), mag * Math.sin(ang)]
    spiral.push(fit(cadd(pole, cmul(cs, z0))))
  }

  const g = empty(
    `Golden spiral: a golden rectangle (φ:1) divided into ${squares} whirling squares, each cut leaving a rectangle similar to the last; the logarithmic spiral r ∝ φ^(-2θ/π) about the similarity's fixed point passes exactly through the squares' corners and is sampled ${samplesPerQuarter} times per quarter turn.`,
  )
  g.polygons = polygons
  g.polylines = [{ points: spiral }]
  g.points = [fit(pole), ...corners]
  g.measures = {
    phi: round(PHI),
    growth_per_quarter_turn: round(PHI),
    square_count: squares,
    samples_per_quarter_turn: samplesPerQuarter,
  }
  return g
}

// --- Torus -----------------------------------------------------------------

function torus(): SacredGeometryConstruction {
  // Ring torus with major radius R and tube radius r, R + r = 1.
  const R = 0.6
  const r = 0.4
  const meridians = 12
  const g = empty(
    `Torus: orthographic plan view, along the axis of revolution, of a ring torus with major radius ${R} and tube radius ${r}: outer equator (R+r), inner equator (R-r) and tube centre circle (R), with ${meridians} meridian circles at equal angles, which project to radial segments.`,
  )
  g.circles = [circle(0, 0, R + r), circle(0, 0, R - r), circle(0, 0, R)]
  for (let k = 0; k < meridians; k++) {
    const a = Math.PI / 2 + (2 * Math.PI * k) / meridians
    g.lines.push(
      line(
        pt((R - r) * Math.cos(a), (R - r) * Math.sin(a)),
        pt((R + r) * Math.cos(a), (R + r) * Math.sin(a)),
      ),
    )
  }
  g.points = [pt(0, 0)]
  g.measures = { major_radius: R, tube_radius: r, meridian_count: meridians }
  return g
}

// --- Sri Yantra ------------------------------------------------------------

/**
 * Concurrent Sri Yantra after A. Chiodo, "On the construction of the Śrī
 * Yantra", C. R. Math. 359 (2021) 377-397, doi:10.5802/crmath.163 (CC-BY 4.0).
 *
 * Triangles t1..t9 are numbered by the height of their base, top first; t1-t5
 * point down, t6-t9 point up. A diagram is concurrent when (Chiodo, sec. 1):
 *  (i)   t3 and t7 share the circumscribed circle;
 *  (ii)  the apex of t' is the base point of t'' for each pair in SRI_APEX_ON_BASE;
 *  (iii) on each side of the axis, the leg of the downward t', the leg of the
 *        upward t''' and the base of t'' meet in one point, for each triple in
 *        SRI_CONCURRENT.
 * Up to scale the family has four parameters, the base points of t3, t6, t7
 * and t9 on the vertical diameter (sec. 2.2.2). Every line then follows from
 * (i)-(iii) except the leg of t1, whose slope solves one equation (sec. 2.3.2:
 * the legs of t1 and t6 and the legs of t4 and t8 must meet on the same base,
 * that of t4). Of its roots exactly one gives an ordered diagram (sec. 2.3.3);
 * a fixed scan and bisection find it, and the build fails if it is not unique.
 */
const SRI_APEX_ON_BASE: [number, number][] = [
  [8, 1],
  [6, 2],
  [9, 3],
  [1, 6],
  [5, 7],
  [4, 8],
  [2, 9],
]

const SRI_CONCURRENT: [number, number, number][] = [
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

/**
 * Base points of t3, t6, t7, t9 as fractions of the diameter from its top.
 * These are Huet's parameters as Chiodo restates them (fn. 20) and uses for
 * his final figure (Fig. 11); Huet, "Śrī Yantra Geometry", TCS 281 (2002).
 */
const SRI_PARAMS = { t3: 0.332, t6: 0.537, t7: 0.602, t9: 0.835 }

/** One isosceles triangle: apex and base depth from the top, base half-width. */
interface SriTriangle {
  apex: number
  base: number
  half: number
}

/** In-house coordinates: x from the axis, s = depth below the top of a unit diameter. */
type XS = [number, number]

const onAxis = (s: number): XS => [0, s]
const halfChord = (s: number): number => Math.sqrt(s * (1 - s))

/** x on the line through a and b at depth s. */
function xAtDepth(a: XS, b: XS, s: number): number {
  return a[0] + ((b[0] - a[0]) * (s - a[1])) / (b[1] - a[1])
}

function meetLines(a: XS, b: XS, c: XS, d: XS): XS {
  const ux = b[0] - a[0]
  const us = b[1] - a[1]
  const vx = d[0] - c[0]
  const vs = d[1] - c[1]
  const k = ((c[0] - a[0]) * vs - (c[1] - a[1]) * vx) / (ux * vs - us * vx)
  return [a[0] + k * ux, a[1] + k * us]
}

/** x of triangle t's right leg at depth s. */
function legX(t: SriTriangle, s: number): number {
  return (t.half * (s - t.apex)) / (t.base - t.apex)
}

/**
 * The nine triangles for a trial leg of t1 through (u, base of t3), plus the
 * mismatch of the one equation left to solve.
 */
function sriTrial(u: number): { triangles: SriTriangle[]; mismatch: number } {
  const { t3: P, t6: Q, t7: R, t9: S } = SRI_PARAMS
  const U: XS = [halfChord(P), P] // right base end of t3, on the circle (i)
  const V: XS = [halfChord(R), R] // right base end of t7, on the circle (i)
  const top = onAxis(0) // apex of t7
  const bottom = onAxis(1) // apex of t3
  const W = meetLines(top, V, onAxis(P), U) // leg t7 x base t3: on leg t2 (2,3,7)
  const X: XS = [xAtDepth(onAxis(S), W, R), R] // leg t2 x base t7: on leg t9 (2,7,9)
  const Y = meetLines(onAxis(P), X, U, bottom) // legs t9 x t3: on base t8 (3,8,9)
  const b8 = Y[1]
  const F: XS = [xAtDepth(onAxis(P), X, Q), Q] // leg t9 x base t6: on leg t4 (4,6,9)
  const G: XS = [xAtDepth(onAxis(S), W, Q), Q] // leg t2 x base t6: base end of t6 (2,6,6)
  const A: XS = [u, P] // trial: leg t1 x base t3, on leg t8 (1,3,8)
  const b2 = meetLines(onAxis(Q), A, top, V)[1] // legs t1 x t7 on base t2 (1,2,7)
  const Z: XS = [xAtDepth(U, bottom, R), R] // leg t3 x base t7: on leg t8 (3,7,8)
  const b1 = meetLines(A, Z, top, bottom)[1] // apex of t8 = base point of t1
  const D = meetLines(onAxis(Q), A, onAxis(b2), G) // legs t1 x t6: on base t4 (1,4,6)
  const I = meetLines(onAxis(b8), F, onAxis(b1), A) // legs t4 x t8: base end of t4 (4,4,8)
  const b4 = D[1]
  const b5 = meetLines(onAxis(Q), A, onAxis(P), X)[1] // legs t1 x t9 on base t5 (1,5,9)
  const t6: SriTriangle = { apex: b2, base: Q, half: G[0] }
  const triangles: SriTriangle[] = [
    { apex: Q, base: b1, half: xAtDepth(onAxis(Q), A, b1) },
    { apex: S, base: b2, half: xAtDepth(onAxis(S), W, b2) },
    { apex: 1, base: P, half: U[0] },
    { apex: b8, base: b4, half: xAtDepth(onAxis(b8), F, b4) },
    { apex: R, base: b5, half: legX(t6, b5) }, // base end on leg t6 (5,5,6)
    t6,
    { apex: 0, base: R, half: V[0] },
    { apex: b1, base: b8, half: xAtDepth(onAxis(b1), A, b8) },
    { apex: P, base: S, half: xAtDepth(onAxis(P), X, S) },
  ]
  return { triangles, mismatch: D[1] - I[1] }
}

/** Largest violation of (i)-(iii), or Infinity if the diagram is not ordered. */
function sriResidual(t: SriTriangle[]): number {
  const tri = (i: number) => t[i - 1]
  for (let i = 0; i < 9; i++) {
    const { apex, base, half } = t[i]
    const down = i < 5
    if (!(half > 0) || (down ? apex <= base : apex >= base)) return Number.POSITIVE_INFINITY
    if (half > halfChord(base) + 1e-12) return Number.POSITIVE_INFINITY
    if (i > 0 && !(base > t[i - 1].base)) return Number.POSITIVE_INFINITY
  }
  let worst = 0
  for (const i of [3, 7]) {
    const { apex, base, half } = tri(i)
    worst = Math.max(
      worst,
      Math.abs(Math.hypot(half, base - 0.5) - 0.5),
      Math.abs(Math.abs(apex - 0.5) - 0.5),
    )
  }
  for (const [a, b] of SRI_APEX_ON_BASE)
    worst = Math.max(worst, Math.abs(tri(a).apex - tri(b).base))
  for (const [down, base, up] of SRI_CONCURRENT) {
    const s = tri(base).base
    const x = legX(tri(down), s)
    if (x > tri(base).half + 1e-12) return Number.POSITIVE_INFINITY
    worst = Math.max(worst, Math.abs(x - legX(tri(up), s)))
  }
  return worst
}

function solveSriYantra(): { triangles: SriTriangle[]; residual: number } {
  const steps = 4096
  const bisections = 200
  const span = halfChord(SRI_PARAMS.t3)
  const solutions: { triangles: SriTriangle[]; residual: number }[] = []
  let prevU = span / steps
  let prevF = sriTrial(prevU).mismatch
  for (let k = 2; k < steps; k++) {
    const u = (span * k) / steps
    const f = sriTrial(u).mismatch
    if (Math.sign(f) !== Math.sign(prevF)) {
      let lo = prevU
      let hi = u
      let fLo = prevF
      for (let n = 0; n < bisections && hi - lo > 0; n++) {
        const mid = (lo + hi) / 2
        if (mid === lo || mid === hi) break
        const fMid = sriTrial(mid).mismatch
        if (Math.sign(fMid) === Math.sign(fLo)) {
          lo = mid
          fLo = fMid
        } else {
          hi = mid
        }
      }
      const root = (lo + hi) / 2
      const { triangles, mismatch } = sriTrial(root)
      const residual = Math.max(sriResidual(triangles), Math.abs(mismatch))
      // A sign change across a pole of the mismatch leaves a large residual.
      if (residual < 1e-12) solutions.push({ triangles, residual })
    }
    prevU = u
    prevF = f
  }
  if (solutions.length !== 1) {
    throw new Error(`Sri Yantra solve found ${solutions.length} concurrent diagrams, expected 1.`)
  }
  return solutions[0]
}

// The solve depends only on SRI_PARAMS, so it runs once per process.
let sriSolution: ReturnType<typeof solveSriYantra> | undefined

function sriYantra(): SacredGeometryConstruction {
  sriSolution ??= solveSriYantra()
  const { triangles, residual } = sriSolution
  const { t3: P, t6: Q, t7: R, t9: S } = SRI_PARAMS
  // Unit diameter with depth s from the top -> unit circle, y up.
  const toFrame = (x: number, s: number): Point2 => pt(2 * x, 1 - 2 * s)

  // Bindu: the incentre of the central triangle t1 ∩ t5 (Chiodo, sec. 1.1),
  // bounded above by the base of t5 and by the legs of t1 down to its apex.
  // Chiodo does not place the bindu; the incentre is the point Kulaichev's
  // extra condition (d) refers to, and is used here without imposing (d).
  const t1 = triangles[0]
  const t5 = triangles[4]
  const halfTop = legX(t1, t5.base)
  const height = t1.apex - t5.base
  const inradius = (halfTop * height) / (halfTop + Math.hypot(halfTop, height))
  const huetXF = legX(t1, P) // concurrency (1,3,8): Huet's X_F
  const huetXA = legX(triangles[2], triangles[7].base) // concurrency (3,8,9): Huet's X_A

  const g = empty(
    `Sri Yantra: the nine triangles of a concurrent Sri Yantra after Chiodo (C. R. Math. 359 (2021) 377-397, doi:10.5802/crmath.163), meeting his conditions (i)-(iii): t3 and t7 inscribed in the outer circle, seven apexes on base points and twelve triple concurrencies. Free parameters: the base points of t3, t6, t7 and t9 at ${P}, ${Q}, ${R} and ${S} of the vertical diameter from the top (Huet's values, as in Chiodo's Fig. 11); the leg of t1 is solved to machine precision. polygons[0..8] are t1..t9 by base height from the top (t1-t5 point down, t6-t9 up), each [base left, base right, apex]. The bindu is the incentre of the central triangle t1∩t5, a choice the source leaves open. The lotus petals and the bhupura are omitted: the source does not construct them. measures.param_* and huet_x_* are fractions of the diameter; huet_x_f and huet_x_a are the half-widths of concurrencies (1,3,8) and (3,8,9), which Huet gives as 0.126 and 0.187.`,
  )
  g.circles = [circle(0, 0, 1)]
  g.polygons = triangles.map((t) => ({
    points: [toFrame(-t.half, t.base), toFrame(t.half, t.base), toFrame(0, t.apex)],
  }))
  g.points = [toFrame(0, t5.base + inradius)]
  g.measures = {
    triangle_count: 9,
    downward_count: 5,
    upward_count: 4,
    param_base_t3: P,
    param_base_t6: Q,
    param_base_t7: R,
    param_base_t9: S,
    huet_x_f: round(huetXF),
    huet_x_a: round(huetXA),
    max_condition_residual: residual,
  }
  return g
}

// --- Registry --------------------------------------------------------------

const BUILDERS: Record<string, () => SacredGeometryConstruction> = {
  'flower-of-life': flowerOfLife,
  'seed-of-life': seedOfLife,
  'metatrons-cube': metatronsCube,
  'vesica-piscis': vesicaPiscis,
  tetrahedron: () => platonicSolid('tetrahedron'),
  cube: () => platonicSolid('cube'),
  octahedron: () => platonicSolid('octahedron'),
  dodecahedron: () => platonicSolid('dodecahedron'),
  icosahedron: () => platonicSolid('icosahedron'),
  'golden-spiral': goldenSpiral,
  torus,
  'sri-yantra': sriYantra,
}

// Every form currently has a construction. A form added without one goes
// here with the reason, and stays not_implemented rather than guessed.
const UNSUPPORTED: Record<string, UnsupportedConstruction> = {}

/**
 * Deterministic construction data for a form, or the reason it is not
 * computed. Every form id in SACRED_FORMS appears in exactly one of the two.
 */
export function buildGeometry(
  formId: string,
): { geometry: SacredGeometryConstruction } | { unsupported: UnsupportedConstruction } {
  const build = BUILDERS[formId]
  if (build) return { geometry: build() }
  return {
    unsupported: UNSUPPORTED[formId] ?? { reason: `No construction is defined for '${formId}'.` },
  }
}

export function getConstructedFormIds(): string[] {
  return Object.keys(BUILDERS)
}

export function getUnsupportedFormIds(): string[] {
  return Object.keys(UNSUPPORTED)
}
