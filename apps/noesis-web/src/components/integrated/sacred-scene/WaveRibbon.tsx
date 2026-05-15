"use client";

// ─── WaveRibbon — iridescent tube along a Catmull-Rom path ───────────────
// Used by kind="cover" and kind="create". A flowing band that crosses the
// SigilCore at an angle, evoking the iridescent HRV waveform from
// Branding/witnessOS-sw/hrv.png.
//
// Vertex shader displaces tube path with sin + simplex noise; fragment
// shader produces iridescence via HSV→RGB shifted by view angle.

import { useMemo, useRef } from "react";
import { useFrame } from "@react-three/fiber";
import * as THREE from "three";

import { NOISE_3D_GLSL } from "./shaders/noise";
import { IRIDESCENCE_GLSL } from "./shaders/iridescence";

interface WaveRibbonProps {
  reducedMotion?: boolean;
  seed?: number;
}

const VERTEX_SHADER = /* glsl */ `
uniform float uTime;
uniform float uReducedMotion;

varying vec3 vWorldPos;
varying vec3 vWorldNormal;
varying vec3 vViewDir;

${NOISE_3D_GLSL}

void main() {
  vec3 pos = position;

  // Flowing displacement — sine wave along x + organic noise.
  float t = uReducedMotion > 0.5 ? 0.0 : uTime;
  float wave = sin(pos.x * 2.0 + t * 1.4) * 0.08;
  float n = snoise(vec3(pos.x * 0.8, pos.y * 0.8, t * 0.3)) * 0.04;
  pos.y += wave + n;
  pos.z += cos(pos.x * 1.4 + t * 0.9) * 0.05;

  vec4 worldPos = modelMatrix * vec4(pos, 1.0);
  vWorldPos = worldPos.xyz;
  vWorldNormal = normalize(mat3(modelMatrix) * normal);
  vViewDir = normalize(cameraPosition - worldPos.xyz);

  gl_Position = projectionMatrix * viewMatrix * worldPos;
}
`;

const FRAGMENT_SHADER = /* glsl */ `
uniform float uTime;

varying vec3 vWorldPos;
varying vec3 vWorldNormal;
varying vec3 vViewDir;

${IRIDESCENCE_GLSL}

void main() {
  // Base hue drifts slowly; iridescence shifts it further by view angle.
  float baseHue = fract(0.12 + uTime * 0.02 + vWorldPos.x * 0.05);
  vec3 col = iridescent(vViewDir, vWorldNormal, baseHue, 0.85, 1.0);

  // Brightness pulse along the ribbon length so it feels alive.
  float along = 0.5 + 0.5 * sin(vWorldPos.x * 1.8 + uTime * 0.7);
  col *= mix(0.55, 1.4, along);

  gl_FragColor = vec4(col, 0.85);
}
`;

export function WaveRibbon({ reducedMotion = false, seed = 0 }: WaveRibbonProps) {
  const matRef = useRef<THREE.ShaderMaterial>(null);

  // Catmull-Rom curve crossing the scene diagonally.
  const geometry = useMemo(() => {
    const offset = (seed % 7) * 0.05;
    const points = [
      new THREE.Vector3(-3.2,  0.6 + offset,  0.4),
      new THREE.Vector3(-2.0,  0.3,           0.0),
      new THREE.Vector3(-1.0, -0.2,          -0.3),
      new THREE.Vector3( 0.0,  0.0 + offset,  0.0),
      new THREE.Vector3( 1.0,  0.2,           0.3),
      new THREE.Vector3( 2.0, -0.3,           0.0),
      new THREE.Vector3( 3.2, -0.5 - offset, -0.4),
    ];
    const curve = new THREE.CatmullRomCurve3(points, false, "catmullrom", 0.5);
    return new THREE.TubeGeometry(curve, 128, 0.045, 12, false);
  }, [seed]);

  const uniforms = useMemo(
    () => ({
      uTime:          { value: 0 },
      uReducedMotion: { value: reducedMotion ? 1.0 : 0.0 },
    }),
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [],
  );

  if (matRef.current) {
    matRef.current.uniforms.uReducedMotion.value = reducedMotion ? 1.0 : 0.0;
  }

  useFrame((state) => {
    if (matRef.current) {
      matRef.current.uniforms.uTime.value = state.clock.elapsedTime;
    }
  });

  return (
    <mesh geometry={geometry} frustumCulled={false}>
      <shaderMaterial
        ref={matRef}
        vertexShader={VERTEX_SHADER}
        fragmentShader={FRAGMENT_SHADER}
        uniforms={uniforms}
        transparent
        depthWrite={false}
        side={THREE.DoubleSide}
        toneMapped={false}
      />
    </mesh>
  );
}
