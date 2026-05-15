"use client";

// ─── Aura — particle shell wrapping the SigilCore ────────────────────────
// N points distributed on a spherical shell (radius 1.5–2.5). Each
// particle has a per-instance phase offset baked into a vertex attribute,
// driving an independent size + alpha pulse. Color drifts from Witness
// Violet → Flow Indigo → Coherence Emerald based on per-particle phase +
// global time.

import { useMemo, useRef } from "react";
import { useFrame } from "@react-three/fiber";
import * as THREE from "three";

interface AuraProps {
  count: number;
  reducedMotion?: boolean;
  seed?: number;
}

const VERTEX_SHADER = /* glsl */ `
attribute float aPhase;
attribute float aRadius;

uniform float uTime;
uniform float uReducedMotion;
uniform float uPixelRatio;

varying float vPhase;

void main() {
  vPhase = aPhase;

  // Slow orbital drift so the shell isn't rigid (skipped in reduced motion).
  float drift = uReducedMotion > 0.5 ? 0.0 : uTime * 0.05;
  float c = cos(drift + aPhase * 6.2831853);
  float s = sin(drift + aPhase * 6.2831853);
  vec3 pos = position;
  vec3 rotated = vec3(
    pos.x * c - pos.z * s,
    pos.y,
    pos.x * s + pos.z * c
  );

  vec4 mvPosition = modelViewMatrix * vec4(rotated, 1.0);
  gl_Position = projectionMatrix * mvPosition;

  // Pulse size — sine wave on per-particle phase + global time.
  float pulse = 0.5 + 0.5 * sin(uTime * 1.3 + aPhase * 6.2831853);
  float size = mix(1.4, 3.6, pulse) * aRadius;

  // Perspective-correct size (Three's standard formula).
  gl_PointSize = size * uPixelRatio * (300.0 / -mvPosition.z);
}
`;

const FRAGMENT_SHADER = /* glsl */ `
uniform float uTime;
varying float vPhase;

// Three brand-aligned colors — Witness Violet, Flow Indigo, Coherence Emerald.
const vec3 C0 = vec3(0.176, 0.0,   0.314);  // #2D0050
const vec3 C1 = vec3(0.043, 0.314, 0.984);  // #0B50FB
const vec3 C2 = vec3(0.063, 0.710, 0.655);  // #10B5A7

void main() {
  // Circular point sprite — soft, falloff alpha.
  vec2 uv = gl_PointCoord - 0.5;
  float d = length(uv);
  if (d > 0.5) discard;
  float alpha = smoothstep(0.5, 0.0, d);

  // Color drift across the C0→C1→C2 spectrum.
  float t = fract(vPhase + uTime * 0.04);
  vec3 col = t < 0.5
    ? mix(C0, C1, t * 2.0)
    : mix(C1, C2, (t - 0.5) * 2.0);

  // Per-particle pulse modulates brightness so bloom picks bright peaks.
  float pulse = 0.65 + 0.35 * sin(uTime * 1.3 + vPhase * 6.2831853);
  gl_FragColor = vec4(col * pulse * 1.4, alpha * 0.85);
}
`;

export function Aura({ count, reducedMotion = false, seed = 0 }: AuraProps) {
  const matRef = useRef<THREE.ShaderMaterial>(null);
  const geomRef = useRef<THREE.BufferGeometry>(null);

  // Build positions + per-particle phase + radius attributes once.
  const { positions, phases, radii } = useMemo(() => {
    const positions = new Float32Array(count * 3);
    const phases = new Float32Array(count);
    const radii = new Float32Array(count);

    // Deterministic seeded RNG (mulberry32) so multiple mounts diverge
    // visually but a single mount is stable across reloads.
    let s = (seed * 2654435761 + 1) >>> 0;
    const rng = () => {
      s += 0x6D2B79F5;
      let t = s;
      t = Math.imul(t ^ (t >>> 15), t | 1);
      t ^= t + Math.imul(t ^ (t >>> 7), t | 61);
      return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
    };

    for (let i = 0; i < count; i++) {
      // Uniformly distributed direction on the sphere (Marsaglia method).
      const u = rng() * 2 - 1;
      const theta = rng() * Math.PI * 2;
      const r = Math.sqrt(1 - u * u);
      const x = r * Math.cos(theta);
      const y = u;
      const z = r * Math.sin(theta);

      // Shell radius 1.5–2.5
      const shell = 1.5 + rng() * 1.0;
      positions[i * 3 + 0] = x * shell;
      positions[i * 3 + 1] = y * shell;
      positions[i * 3 + 2] = z * shell;

      phases[i] = rng();
      radii[i]  = 0.5 + rng() * 0.8;
    }

    return { positions, phases, radii };
  }, [count, seed]);

  const uniforms = useMemo(
    () => ({
      uTime:          { value: 0 },
      uReducedMotion: { value: reducedMotion ? 1.0 : 0.0 },
      uPixelRatio:    { value: typeof window !== "undefined" ? Math.min(window.devicePixelRatio, 2) : 1 },
    }),
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [],
  );

  // Keep uReducedMotion uniform in sync without recompile.
  if (matRef.current) {
    matRef.current.uniforms.uReducedMotion.value = reducedMotion ? 1.0 : 0.0;
  }

  useFrame((state) => {
    if (matRef.current) {
      matRef.current.uniforms.uTime.value = state.clock.elapsedTime;
    }
  });

  return (
    <points frustumCulled={false}>
      <bufferGeometry ref={geomRef}>
        <bufferAttribute
          attach="attributes-position"
          args={[positions, 3]}
        />
        <bufferAttribute
          attach="attributes-aPhase"
          args={[phases, 1]}
        />
        <bufferAttribute
          attach="attributes-aRadius"
          args={[radii, 1]}
        />
      </bufferGeometry>
      <shaderMaterial
        ref={matRef}
        vertexShader={VERTEX_SHADER}
        fragmentShader={FRAGMENT_SHADER}
        uniforms={uniforms}
        transparent
        depthWrite={false}
        blending={THREE.AdditiveBlending}
        toneMapped={false}
      />
    </points>
  );
}
