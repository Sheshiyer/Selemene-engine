/**
 * Sacred Geometry Engine - Public API exports
 */

// Wisdom data
export type { SacredForm } from './wisdom'
export { SACRED_FORMS, getFormById, getFormIds } from './wisdom'

// Construction data
export type {
  GeometryCircle,
  GeometryLine,
  GeometryPolygon,
  GeometryPolyline,
  GeometrySolid,
  Point2,
  Point3,
  SacredGeometryConstruction,
  UnsupportedConstruction,
} from './geometry'
export { buildGeometry, getConstructedFormIds, getUnsupportedFormIds } from './geometry'

// Witness prompts
export { generateWitnessPrompts } from './witness'

// Engine
export { SacredGeometryEngine } from './engine'
