// ─── DepthBubbles — small sparkle particles riding the trail head ───────
// Port of codrops/atmospheric-depth-gallery TrailHeadParticles.js (MIT,
// Houmahani Kane). Adapted to TypeScript + Goethe palette (parchment +
// sacred-gold).
//
// Maintains a small pool (default 18) of sphere meshes. Each frame, a
// small number of particles spawn around the trail head position with
// random velocity + life. Particles drift, fade, and recycle into the
// pool when their life expires.

import * as THREE from "three";

export interface DepthBubblesOptions {
  color?: string;
  maxParticles?: number;
  spawnPerSecond?: number;
  spawnRadius?: number;
  speedMin?: number;
  speedMax?: number;
  lifeMin?: number;
  lifeMax?: number;
  sizeMin?: number;
  sizeMax?: number;
  dragPerFrame?: number;
}

interface BubbleParticle {
  mesh: THREE.Mesh;
  velocity: THREE.Vector3;
  lifeRemaining: number;
  totalLife: number;
}

export class DepthBubbles {
  readonly group = new THREE.Group();
  private isEnabled = true;
  private particles: BubbleParticle[] = [];
  private sharedGeometry: THREE.SphereGeometry;
  private spawnAccumulator = 0;
  private nextSpawnIndex = 0;

  private readonly maxParticles: number;
  private readonly spawnPerSecond: number;
  private readonly spawnRadius: number;
  private readonly speedMin: number;
  private readonly speedMax: number;
  private readonly lifeMin: number;
  private readonly lifeMax: number;
  private readonly sizeMin: number;
  private readonly sizeMax: number;
  private readonly dragPerFrame: number;

  constructor(opts: DepthBubblesOptions = {}) {
    this.maxParticles = opts.maxParticles ?? 18;
    this.spawnPerSecond = opts.spawnPerSecond ?? 20;
    this.spawnRadius = opts.spawnRadius ?? 0.52;
    this.speedMin = opts.speedMin ?? 0.05;
    this.speedMax = opts.speedMax ?? 0.22;
    this.lifeMin = opts.lifeMin ?? 0.25;
    this.lifeMax = opts.lifeMax ?? 0.6;
    this.sizeMin = opts.sizeMin ?? 0.007;
    this.sizeMax = opts.sizeMax ?? 0.02;
    this.dragPerFrame = opts.dragPerFrame ?? 0.94;

    this.group.renderOrder = 1300;
    this.sharedGeometry = new THREE.SphereGeometry(1, 5, 4);

    const color = new THREE.Color(opts.color ?? "#F0EDE3");
    for (let i = 0; i < this.maxParticles; i++) {
      const material = new THREE.MeshBasicMaterial({
        color,
        transparent: true,
        opacity: 0,
        depthWrite: false,
        depthTest: false,
      });
      const mesh = new THREE.Mesh(this.sharedGeometry, material);
      mesh.visible = false;
      mesh.renderOrder = 1300;
      this.group.add(mesh);
      this.particles.push({
        mesh,
        velocity: new THREE.Vector3(),
        lifeRemaining: 0,
        totalLife: 0,
      });
    }
  }

  setEnabled(enabled: boolean): void {
    if (this.isEnabled && !enabled) this.clear();
    this.isEnabled = enabled;
    this.group.visible = enabled;
  }

  update(
    deltaSeconds: number,
    headPosition: THREE.Vector3,
    opacity = 1,
    shouldSpawn = true,
  ): void {
    const safeDelta = Math.min(Math.max(deltaSeconds || 0, 0), 0.1);

    if (this.isEnabled && shouldSpawn && safeDelta > 0) {
      this.spawnAccumulator += safeDelta * this.spawnPerSecond;
      const spawnCount = Math.floor(this.spawnAccumulator);
      this.spawnAccumulator -= spawnCount;
      for (let i = 0; i < spawnCount; i++) this.spawnParticle(headPosition);
    } else {
      this.spawnAccumulator = 0;
    }

    const clampedOpacity = THREE.MathUtils.clamp(opacity, 0, 1);
    const drag = Math.pow(this.dragPerFrame, safeDelta * 60);

    for (const p of this.particles) {
      if (p.lifeRemaining <= 0) continue;
      p.lifeRemaining -= safeDelta;
      if (p.lifeRemaining <= 0) {
        p.lifeRemaining = 0;
        p.mesh.visible = false;
        (p.mesh.material as THREE.MeshBasicMaterial).opacity = 0;
        continue;
      }
      p.velocity.multiplyScalar(drag);
      p.mesh.position.addScaledVector(p.velocity, safeDelta);
      const lifeRatio = p.lifeRemaining / p.totalLife;
      (p.mesh.material as THREE.MeshBasicMaterial).opacity = lifeRatio * clampedOpacity * 0.75;
    }
  }

  clear(): void {
    for (const p of this.particles) {
      p.lifeRemaining = 0;
      p.mesh.visible = false;
      (p.mesh.material as THREE.MeshBasicMaterial).opacity = 0;
    }
    this.spawnAccumulator = 0;
  }

  dispose(): void {
    for (const p of this.particles) {
      (p.mesh.material as THREE.MeshBasicMaterial).dispose();
    }
    this.sharedGeometry.dispose();
  }

  private spawnParticle(headPosition: THREE.Vector3): void {
    const p = this.particles[this.nextSpawnIndex];
    this.nextSpawnIndex = (this.nextSpawnIndex + 1) % this.particles.length;

    // Random offset on a unit sphere * spawnRadius
    const theta = Math.random() * Math.PI * 2;
    const phi = Math.acos(2 * Math.random() - 1);
    const r = this.spawnRadius * (0.4 + 0.6 * Math.random());
    p.mesh.position.set(
      headPosition.x + r * Math.sin(phi) * Math.cos(theta),
      headPosition.y + r * Math.sin(phi) * Math.sin(theta),
      headPosition.z + r * Math.cos(phi),
    );

    const speed = THREE.MathUtils.lerp(this.speedMin, this.speedMax, Math.random());
    p.velocity.set(
      Math.cos(theta) * Math.sin(phi) * speed,
      Math.sin(theta) * Math.sin(phi) * speed,
      Math.cos(phi) * speed,
    );

    const size = THREE.MathUtils.lerp(this.sizeMin, this.sizeMax, Math.random());
    p.mesh.scale.setScalar(size);

    const life = THREE.MathUtils.lerp(this.lifeMin, this.lifeMax, Math.random());
    p.lifeRemaining = life;
    p.totalLife = life;
    p.mesh.visible = true;
  }
}
