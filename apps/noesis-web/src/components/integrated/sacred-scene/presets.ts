// ─── SacredScene per-kind presets ────────────────────────────────────────
// One config drives 8 visually-distinct scenes from a shared shader codebase.
// Color tokens map directly to the Goethe Consciousness Spectrum from
// apps/noesis-web/DESIGN.md (Sacred Gold / Coherence Emerald / Flow Indigo
// / Witness Violet / Void Black).

import * as THREE from "three";

export type SceneKind =
  | "cover"
  | "stabilize"
  | "heal"
  | "create"
  | "mutate"
  | "transition"
  | "closing"
  | "ambient";

export interface ScenePreset {
  /** Inner radial gradient color of SigilCore. */
  core: string;
  /** Outer rim color of SigilCore. */
  edge: string;
  /** SceneFog dominant tint. */
  fogTint: string;
  /** Bloom intensity multiplier (post-process). */
  bloom: number;
  /** Auto-rotation speed (rad/s). */
  rotate: number;
  /** Aura particle count. */
  particles: number;
  /** Show WaveRibbon. */
  ribbon: boolean;
  /** fbm domain scale on SigilCore surface. */
  noiseScale: number;
  /** Camera z position override. */
  cameraZ: number;
}

export const PRESETS: Record<SceneKind, ScenePreset> = {
  cover:      { core: "#C5A017", edge: "#0B50FB", fogTint: "#2D0050", bloom: 1.8, rotate: 0.06, particles: 2400, ribbon: true,  noiseScale: 1.3, cameraZ: 5.0 },
  stabilize:  { core: "#2D0050", edge: "#070B1D", fogTint: "#2D0050", bloom: 1.1, rotate: 0.03, particles: 1200, ribbon: false, noiseScale: 0.8, cameraZ: 4.6 },
  heal:       { core: "#0B50FB", edge: "#10B5A7", fogTint: "#0B50FB", bloom: 1.3, rotate: 0.04, particles: 1400, ribbon: false, noiseScale: 1.0, cameraZ: 4.6 },
  create:     { core: "#10B5A7", edge: "#C5A017", fogTint: "#10B5A7", bloom: 1.5, rotate: 0.05, particles: 1800, ribbon: true,  noiseScale: 1.2, cameraZ: 4.8 },
  mutate:     { core: "#C5A017", edge: "#070B1D", fogTint: "#C5A017", bloom: 1.3, rotate: 0.04, particles: 1600, ribbon: false, noiseScale: 1.1, cameraZ: 4.6 },
  transition: { core: "#C5A017", edge: "#C5A017", fogTint: "#070B1D", bloom: 2.2, rotate: 0.12, particles: 800,  ribbon: false, noiseScale: 0.5, cameraZ: 4.2 },
  closing:    { core: "#070B1D", edge: "#2D0050", fogTint: "#070B1D", bloom: 0.8, rotate: 0.02, particles: 600,  ribbon: false, noiseScale: 1.4, cameraZ: 5.4 },
  ambient:    { core: "#2D0050", edge: "#070B1D", fogTint: "#070B1D", bloom: 0.4, rotate: 0.01, particles: 400,  ribbon: false, noiseScale: 0.6, cameraZ: 5.0 },
};

/** Convert a hex color (#RRGGBB) to a THREE.Color in linear space for shader use. */
export function hexToVec3(hex: string): THREE.Color {
  return new THREE.Color(hex);
}
