"use client";

// ─── SceneFog — atmospheric back plane with animated 2D simplex noise ────
// A 10×10 plane sitting at z = -3 behind the SigilCore. Fragment shader
// composites a radial gradient (tint from preset) with slow-drifting
// 2D simplex noise to evoke consciousness fog — not flat color, never
// just feGaussianBlur SVG.

import { useMemo, useRef } from "react";
import { useFrame } from "@react-three/fiber";
import * as THREE from "three";

import { NOISE_2D_GLSL } from "./shaders/noise";
import { FBM_2D_GLSL } from "./shaders/fbm";
import { hexToVec3 } from "./presets";

interface SceneFogProps {
  tint: string;
  reducedMotion?: boolean;
}

const VERTEX_SHADER = /* glsl */ `
varying vec2 vUv;
void main() {
  vUv = uv;
  gl_Position = projectionMatrix * modelViewMatrix * vec4(position, 1.0);
}
`;

const FRAGMENT_SHADER = /* glsl */ `
uniform float uTime;
uniform vec3  uTint;
uniform float uReducedMotion;

varying vec2 vUv;

${NOISE_2D_GLSL}
${FBM_2D_GLSL}

void main() {
  vec2 p = (vUv - 0.5) * 2.0;
  float t = uReducedMotion > 0.5 ? 0.0 : uTime * 0.05;

  // FBM cloud drift — two displaced samples mixed for parallax depth.
  float n1 = fbm2(p * 1.3 + vec2(t, t * 0.7), 5, 2.0, 0.5);
  float n2 = fbm2(p * 2.4 + vec2(-t * 0.6, t * 0.3), 4, 2.0, 0.5);
  float n  = (n1 * 0.7 + n2 * 0.3) * 0.5 + 0.5;

  // Radial vignette so fog concentrates around the sigil and falls off
  // toward the corners (DESIGN.md never wants a flat backdrop).
  float r = length(p);
  float vignette = smoothstep(1.4, 0.0, r);

  // Tint + noise modulation — bloom catches the brighter wisps.
  vec3 col = uTint * (0.15 + 0.55 * n) * vignette;

  // Soft alpha falloff at the edges.
  float alpha = vignette * 0.85;
  gl_FragColor = vec4(col, alpha);
}
`;

export function SceneFog({ tint, reducedMotion = false }: SceneFogProps) {
  const matRef = useRef<THREE.ShaderMaterial>(null);

  const uniforms = useMemo(
    () => ({
      uTime:          { value: 0 },
      uTint:          { value: hexToVec3(tint) },
      uReducedMotion: { value: reducedMotion ? 1.0 : 0.0 },
    }),
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [],
  );

  if (matRef.current) {
    (matRef.current.uniforms.uTint.value as THREE.Color).set(tint);
    matRef.current.uniforms.uReducedMotion.value = reducedMotion ? 1.0 : 0.0;
  }

  useFrame((state) => {
    if (matRef.current) {
      matRef.current.uniforms.uTime.value = state.clock.elapsedTime;
    }
  });

  return (
    <mesh position={[0, 0, -3]} frustumCulled={false}>
      <planeGeometry args={[10, 10, 1, 1]} />
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
