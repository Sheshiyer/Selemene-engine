// ─── DepthTrail — wavy thread weaving through the planes ─────────────────
// Port of codrops/atmospheric-depth-gallery Trail.js (MIT, Houmahani Kane).
// Adapted to TypeScript + Goethe palette (parchment with sacred-gold glow).
//
// Accumulates a sequence of head positions, builds a tapered Catmull-Rom
// tube geometry through them. As the scroll progress moves the head
// forward, the tail trails behind with a slight ease-in.

import * as THREE from "three";

export interface DepthTrailOptions {
  /** Trail color (the visible body of the thread). */
  color?: string;
  /** Emissive glow color around the thread. */
  glowColor?: string;
  glowIntensity?: number;
  baseOpacity?: number;
  /** Max number of accumulated points before trimming. */
  maxPoints?: number;
  /** Tube radius at the leading (newest) point. */
  radiusHead?: number;
  /** Tube radius at the trailing (oldest) point. */
  radiusTail?: number;
  /** Radial subdivisions around the tube. */
  radialSegments?: number;
  /** Catmull-Rom curve tension (0 = loose, 1 = tight). */
  curveTension?: number;
  /** Smoothing factor when adding a new point (lerp with previous tip). */
  pointSmoothing?: number;
  /** Minimum distance before a new point is accepted (squared). */
  minDistance?: number;
}

export class DepthTrail {
  readonly group = new THREE.Group();
  readonly material: THREE.MeshStandardMaterial;
  private points: THREE.Vector3[] = [];
  private mesh: THREE.Mesh | null = null;

  // Config
  private maxPoints: number;
  private radiusHead: number;
  private radiusTail: number;
  private radialSegments: number;
  private curveTension: number;
  private pointSmoothing: number;
  private minDistance: number;

  constructor(opts: DepthTrailOptions = {}) {
    this.maxPoints = opts.maxPoints ?? 220;
    this.radiusHead = opts.radiusHead ?? 0.012;
    this.radiusTail = opts.radiusTail ?? 0.003;
    this.radialSegments = opts.radialSegments ?? 8;
    this.curveTension = opts.curveTension ?? 0.67;
    this.pointSmoothing = opts.pointSmoothing ?? 0.45;
    this.minDistance = opts.minDistance ?? 0.006;

    this.material = new THREE.MeshStandardMaterial({
      color: new THREE.Color(opts.color ?? "#F0EDE3"),
      emissive: new THREE.Color(opts.glowColor ?? "#C5A017"),
      emissiveIntensity: opts.glowIntensity ?? 1.4,
      roughness: 0.25,
      metalness: 0.05,
      transparent: true,
      opacity: opts.baseOpacity ?? 0.55,
      depthWrite: false,
      depthTest: false,
      blending: THREE.NormalBlending,
    });
    this.group.renderOrder = 1200;
  }

  /** Add a new head position. No-op if too close to the previous head. */
  addPoint(position: THREE.Vector3): void {
    const last = this.points[this.points.length - 1] || null;
    if (last) {
      const distSq = position.distanceToSquared(last);
      if (distSq < this.minDistance * this.minDistance) return;
    }
    const next = position.clone();
    const eased = last ? last.clone().lerp(next, this.pointSmoothing) : next;
    this.points.push(eased);

    // Trim oldest points back to maxPoints (small budget per frame to avoid jitter)
    let trimBudget = 4;
    while (this.points.length > this.maxPoints && trimBudget > 0) {
      this.points.shift();
      trimBudget -= 1;
    }
    if (this.points.length < 2) return;

    const curve = new THREE.CatmullRomCurve3(this.points, false, "centripetal", this.curveTension);
    const segments = Math.max(24, Math.min(this.points.length * 4, 220));
    const geometry = this.createTaperedTube(curve, segments, this.radiusHead, this.radiusTail);

    if (this.mesh) {
      this.mesh.geometry.dispose();
      this.group.remove(this.mesh);
    }
    this.mesh = new THREE.Mesh(geometry, this.material);
    this.mesh.renderOrder = 1200;
    this.group.add(this.mesh);
  }

  /** Discard all points and the current mesh — start fresh. */
  reset(): void {
    this.points = [];
    if (this.mesh) {
      this.mesh.geometry.dispose();
      this.group.remove(this.mesh);
      this.mesh = null;
    }
  }

  dispose(): void {
    this.reset();
    this.material.dispose();
  }

  /** Build a TubeGeometry with linearly-tapered radius (head → tail). */
  private createTaperedTube(
    curve: THREE.CatmullRomCurve3,
    segments: number,
    radiusHead: number,
    radiusTail: number,
  ): THREE.BufferGeometry {
    // Three.js TubeGeometry has constant radius. We emulate taper by
    // building our own geometry: walk the curve in N steps, lerping
    // radius from radiusHead (start) to radiusTail (end), build radial
    // ring vertices around each step.
    const radial = this.radialSegments;
    const positions: number[] = [];
    const normals: number[] = [];
    const indices: number[] = [];

    const frames = curve.computeFrenetFrames(segments, false);

    for (let i = 0; i <= segments; i++) {
      const t = i / segments;
      const point = curve.getPoint(t);
      // Tail-to-head: t=0 is OLDEST (smaller), t=1 is NEWEST (larger).
      // In codrops the head is the most-recently-added point at the end of
      // the array, so the curve's last point (t=1) gets the head radius.
      const radius = THREE.MathUtils.lerp(radiusTail, radiusHead, t);
      const normal = frames.normals[i];
      const binormal = frames.binormals[i];

      for (let r = 0; r < radial; r++) {
        const angle = (r / radial) * Math.PI * 2;
        const sin = Math.sin(angle);
        const cos = Math.cos(angle);
        const nx = cos * normal.x + sin * binormal.x;
        const ny = cos * normal.y + sin * binormal.y;
        const nz = cos * normal.z + sin * binormal.z;
        positions.push(point.x + radius * nx, point.y + radius * ny, point.z + radius * nz);
        normals.push(nx, ny, nz);
      }
    }

    // Faces
    for (let i = 0; i < segments; i++) {
      for (let r = 0; r < radial; r++) {
        const a = i * radial + r;
        const b = a + radial;
        const c = i * radial + ((r + 1) % radial);
        const d = c + radial;
        indices.push(a, b, c);
        indices.push(b, d, c);
      }
    }

    const geom = new THREE.BufferGeometry();
    geom.setAttribute("position", new THREE.Float32BufferAttribute(positions, 3));
    geom.setAttribute("normal", new THREE.Float32BufferAttribute(normals, 3));
    geom.setIndex(indices);
    return geom;
  }
}
