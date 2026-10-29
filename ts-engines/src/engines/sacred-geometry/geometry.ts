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
}

const UNSUPPORTED: Record<string, UnsupportedConstruction> = {
  'sri-yantra': {
    reason:
      'The nine interlocking triangles must meet concurrency constraints that are only met by a numeric solve against a chosen canonical construction; none is implemented, so no coordinates are guessed.',
  },
}

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
