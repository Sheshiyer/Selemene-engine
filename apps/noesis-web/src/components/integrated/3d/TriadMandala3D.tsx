"use client";

// ─── TriadMandala3D — Pass α / opening yantra in 3D ────────────────────
// Per W10 brief: same prop shape as W2 TriadMandala. Three luminous
// spheres at triangle vertices, tube-geometry edges connecting them,
// central liquid-glass core sphere. Drei <Float> for subtle organic
// motion; postprocessing Bloom for the bioluminescent finish.

import { useMemo, useRef } from "react";
import { useFrame } from "@react-three/fiber";
import { Float, MeshTransmissionMaterial } from "@react-three/drei";
import * as THREE from "three";

import { COLOR } from "./_shared";

interface TriadMandala3DProps {
  reducedMotion: boolean;
  subjects?: string[];
}

const VERTEX_COLORS = [COLOR.gold, COLOR.emerald, COLOR.violet];

export function TriadMandala3DScene({ reducedMotion, subjects = [] }: TriadMandala3DProps) {
  const groupRef = useRef<THREE.Group>(null);
  const emissiveRef = useRef<THREE.MeshStandardMaterial>(null);

  // Triangle vertex positions — apex up, base centred. Radius 0.95 fits
  // comfortably inside the camera frustum at z=5.
  const N = Math.max(subjects.length, 3);
  const R = 0.95;
  const pts = useMemo(() => {
    return Array.from({ length: N }, (_, i) => {
      const a = -Math.PI / 2 + (i * 2 * Math.PI) / N;
      return new THREE.Vector3(R * Math.cos(a), R * Math.sin(a), 0);
    });
  }, [N]);

  // Edge tubes — pairwise relations between vertices.
  const edgeTubes = useMemo(() => {
    const tubes: THREE.TubeGeometry[] = [];
    for (let i = 0; i < pts.length; i++) {
      for (let j = i + 1; j < pts.length; j++) {
        const curve = new THREE.LineCurve3(pts[i], pts[j]);
        tubes.push(new THREE.TubeGeometry(curve, 1, 0.012, 8, false));
      }
    }
    return tubes;
  }, [pts]);

  useFrame((state, delta) => {
    if (!groupRef.current) return;
    if (!reducedMotion) {
      groupRef.current.rotation.z += delta * 0.04;
    }
    if (emissiveRef.current) {
      const pulse = reducedMotion ? 1.6 : 1.4 + Math.sin(state.clock.elapsedTime * 0.8) * 0.5;
      emissiveRef.current.emissiveIntensity = pulse;
    }
  });

  return (
    <Float
      speed={reducedMotion ? 0 : 0.5}
      rotationIntensity={reducedMotion ? 0 : 0.2}
      floatIntensity={reducedMotion ? 0 : 0.3}
    >
      <group ref={groupRef}>
        {/* Vertices — luminous orbs in the three sacred tones. */}
        {pts.map((p, i) => (
          <group key={i} position={p}>
            <mesh>
              <sphereGeometry args={[0.085, 32, 32]} />
              <meshStandardMaterial
                color={VERTEX_COLORS[i % VERTEX_COLORS.length]}
                emissive={VERTEX_COLORS[i % VERTEX_COLORS.length]}
                emissiveIntensity={1.6}
                toneMapped={false}
              />
            </mesh>
            <mesh>
              <sphereGeometry args={[0.26, 32, 32]} />
              <meshBasicMaterial
                color={VERTEX_COLORS[i % VERTEX_COLORS.length]}
                transparent
                opacity={0.08}
                toneMapped={false}
              />
            </mesh>
          </group>
        ))}

        {/* Edges — Sacred Gold tubes between every pair. */}
        {edgeTubes.map((geo, i) => (
          <mesh key={i} geometry={geo}>
            <meshStandardMaterial
              color={COLOR.gold}
              emissive={COLOR.gold}
              emissiveIntensity={1.1}
              toneMapped={false}
            />
          </mesh>
        ))}

        {/* Central liquid-glass core — same MeshTransmissionMaterial
            recipe as the cover, but smaller. */}
        <group>
          <mesh rotation={[Math.PI / 2, 0, 0]}>
            <ringGeometry args={[0.22, 0.225, 96]} />
            <meshBasicMaterial color={COLOR.gold} transparent opacity={0.55} side={THREE.DoubleSide} toneMapped={false} />
          </mesh>
          <mesh>
            <sphereGeometry args={[0.16, 64, 64]} />
            <MeshTransmissionMaterial
              backside
              samples={4}
              thickness={0.5}
              roughness={0.05}
              chromaticAberration={0.08}
              transmission={1}
              ior={1.3}
              distortion={0.18}
              distortionScale={0.4}
              temporalDistortion={0.08}
              color={COLOR.gold}
              attenuationColor={COLOR.emerald}
              attenuationDistance={0.6}
            />
          </mesh>
          <mesh>
            <sphereGeometry args={[0.09, 32, 32]} />
            <meshStandardMaterial
              ref={emissiveRef}
              color={COLOR.emerald}
              emissive={COLOR.emerald}
              emissiveIntensity={1.4}
              toneMapped={false}
            />
          </mesh>
        </group>
      </group>
    </Float>
  );
}
