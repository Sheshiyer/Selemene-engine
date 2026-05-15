"use client";

// ─── SigilCore — yantra heart with breathing displacement + fbm plasma ───
// Icosahedron (radius 1, detail 4) with a fully-custom ShaderMaterial:
//   • Vertex shader  — Perlin/simplex noise displaces vertices along the
//                      normal, modulated by a 4:7:8 breath cycle phase.
//   • Fragment shader — radial-gradient core→edge color with procedural
//                      fbm noise for plasma surface texture. Soft edge
//                      falloff (no alpha clip — keeps bloom happy).
//
// Per DESIGN § 1: bioluminescent, not fluorescent — emission comes from
// within the geometry, not an external spotlight.

import { useMemo, useRef } from "react";
import { useFrame } from "@react-three/fiber";
import * as THREE from "three";

import { NOISE_3D_GLSL } from "./shaders/noise";
import { FBM_3D_GLSL } from "./shaders/fbm";
import { hexToVec3 } from "./presets";

interface SigilCoreProps {
  coreColor: string;
  edgeColor: string;
  noiseScale: number;
  reducedMotion?: boolean;
  /** Per-mount seed so duplicate mounts diverge visually. */
  seed?: number;
}

const VERTEX_SHADER = /* glsl */ `
uniform float uTime;
uniform float uBreathPhase;
uniform float uNoiseScale;
uniform float uSeed;

varying vec3 vNormal;
varying vec3 vPosition;
varying vec3 vWorldNormal;
varying float vNoise;

${NOISE_3D_GLSL}
${FBM_3D_GLSL}

void main() {
  // Breath: 4 in / 7 hold / 8 out = 19s cycle. Smoothed sinusoid.
  float breath = 0.5 + 0.5 * sin(uBreathPhase * 6.2831853);
  float breathAmp = mix(0.04, 0.10, breath);

  // Procedural displacement along normal — fbm of position + time-drift
  vec3 noisePos = position * uNoiseScale + vec3(uSeed, uTime * 0.15, -uTime * 0.10);
  float n = fbm3(noisePos, 4, 2.0, 0.5);
  vNoise = n;

  vec3 displaced = position + normal * (n * breathAmp);

  vNormal = normal;
  vPosition = position;
  vWorldNormal = normalize(normalMatrix * normal);

  gl_Position = projectionMatrix * modelViewMatrix * vec4(displaced, 1.0);
}
`;

const FRAGMENT_SHADER = /* glsl */ `
uniform vec3  uColorCore;
uniform vec3  uColorEdge;
uniform float uTime;
uniform float uBloomBoost;

varying vec3 vNormal;
varying vec3 vPosition;
varying vec3 vWorldNormal;
varying float vNoise;

void main() {
  // Radial gradient: inner sphere = core color, rim = edge color.
  float r = length(vPosition);
  float t = smoothstep(0.4, 1.05, r);

  // Plasma modulation — vNoise carried from vertex stage adds organic
  // banding to the gradient.
  float plasma = 0.5 + 0.5 * vNoise;
  vec3 col = mix(uColorCore, uColorEdge, t);
  col += (plasma - 0.5) * 0.35 * uColorCore;

  // Fresnel-style rim brightening — the edge glows brighter at grazing
  // angles, simulating bioluminescent skin without an actual rim light.
  float fres = pow(1.0 - max(dot(vWorldNormal, vec3(0.0, 0.0, 1.0)), 0.0), 2.0);
  col += uColorEdge * fres * 0.6;

  // Subtle pulse — never fully off; sets the bloom luminance floor.
  float pulse = 0.85 + 0.15 * sin(uTime * 0.6);
  col *= pulse * uBloomBoost;

  // Soft alpha at rim so the core feels luminous rather than capped.
  float alpha = 1.0 - smoothstep(0.92, 1.10, r);
  gl_FragColor = vec4(col, alpha);
}
`;

export function SigilCore({
  coreColor,
  edgeColor,
  noiseScale,
  reducedMotion = false,
  seed = 0,
}: SigilCoreProps) {
  const matRef = useRef<THREE.ShaderMaterial>(null);

  // Uniforms: stable identity across renders so r3f doesn't recreate the
  // shader program every frame.
  const uniforms = useMemo(
    () => ({
      uTime:        { value: 0 },
      uBreathPhase: { value: 0 },
      uNoiseScale:  { value: noiseScale },
      uSeed:        { value: seed * 13.37 },
      uColorCore:   { value: hexToVec3(coreColor) },
      uColorEdge:   { value: hexToVec3(edgeColor) },
      uBloomBoost:  { value: 1.0 },
    }),
    // Initial values only — live updates happen in useFrame and via the
    // effects below so a color change doesn't force a shader recompile.
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [],
  );

  // Live-update uniforms when props change (no shader recompile).
  if (matRef.current) {
    (matRef.current.uniforms.uColorCore.value as THREE.Color).set(coreColor);
    (matRef.current.uniforms.uColorEdge.value as THREE.Color).set(edgeColor);
    matRef.current.uniforms.uNoiseScale.value = noiseScale;
  }

  useFrame((state) => {
    if (!matRef.current) return;
    const t = state.clock.elapsedTime;
    matRef.current.uniforms.uTime.value = t;

    // 4-7-8 breath = 19s cycle. In reduced-motion, freeze at mid-phase.
    matRef.current.uniforms.uBreathPhase.value = reducedMotion
      ? 0.5
      : (t / 19.0) % 1.0;
  });

  // Dispose geometry / material on unmount via key-stable refs.
  return (
    <mesh frustumCulled={false}>
      <icosahedronGeometry args={[1, 4]} />
      <shaderMaterial
        ref={matRef}
        vertexShader={VERTEX_SHADER}
        fragmentShader={FRAGMENT_SHADER}
        uniforms={uniforms}
        transparent
        depthWrite={false}
        toneMapped={false}
      />
    </mesh>
  );
}
