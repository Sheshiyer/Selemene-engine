"use client";

// ─── VesicaTrio3D — Pass β / resonance yantra in 3D ────────────────────
// Per W10 brief: three TorusGeometries at 120° offsets in the same
// Z-plane, with subtle Z-offsets for parallax depth. Their overlapping
// emissive bloom illuminates the intersection regions naturally.

import { useMemo, useRef } from "react";
import { useFrame } from "@react-three/fiber";
import { Float } from "@react-three/drei";
import * as THREE from "three";

import { COLOR } from "./_shared";

interface VesicaTrio3DProps {
  reducedMotion: boolean;
  subjects?: string[];
}

const TORI = [
  { angleDeg: -90, color: COLOR.gold, z: 0.04 },
  { angleDeg: 30, color: COLOR.emerald, z: 0.0 },
  { angleDeg: 150, color: COLOR.violet, z: -0.04 },
] as const;

export function VesicaTrio3DScene({ reducedMotion }: VesicaTrio3DProps) {
  const groupRef = useRef<THREE.Group>(null);
  const centerMatRef = useRef<THREE.MeshStandardMaterial>(null);

  // Torus radius and the centre-offset distance — vesica geometry uses
  // D ≈ 0.85R so all three intersect at the world origin.
  const R = 0.55;
  const D = R * 0.85;

  const positions = useMemo(() => {
    return TORI.map((t) => {
      const a = (t.angleDeg * Math.PI) / 180;
      return new THREE.Vector3(D * Math.cos(a), D * Math.sin(a), t.z);
    });
  }, [D]);

  useFrame((state, delta) => {
    if (!groupRef.current) return;
    if (!reducedMotion) {
      groupRef.current.rotation.z += delta * 0.025;
    }
    if (centerMatRef.current) {
      centerMatRef.current.emissiveIntensity = reducedMotion
        ? 1.8
        : 1.4 + Math.sin(state.clock.elapsedTime * 0.6) * 0.6;
    }
  });

  return (
    <Float
      speed={reducedMotion ? 0 : 0.5}
      rotationIntensity={reducedMotion ? 0 : 0.2}
      floatIntensity={reducedMotion ? 0 : 0.3}
    >
      <group ref={groupRef}>
        {/* Three tori, view-plane facing camera (rotation X = π/2). */}
        {TORI.map((t, i) => (
          <mesh key={i} position={positions[i]} rotation={[Math.PI / 2, 0, 0]}>
            <torusGeometry args={[R, 0.014, 16, 128]} />
            <meshStandardMaterial
              color={t.color}
              emissive={t.color}
              emissiveIntensity={1.2}
              transparent
              opacity={0.92}
              toneMapped={false}
            />
          </mesh>
        ))}

        {/* Faint translucent discs to brighten the intersection regions
            via additive bloom contribution. */}
        {TORI.map((t, i) => (
          <mesh key={`disc-${i}`} position={positions[i]} rotation={[Math.PI / 2, 0, 0]}>
            <circleGeometry args={[R, 96]} />
            <meshBasicMaterial color={t.color} transparent opacity={0.06} side={THREE.DoubleSide} toneMapped={false} />
          </mesh>
        ))}

        {/* Centre — where all three resonate. */}
        <mesh>
          <sphereGeometry args={[0.06, 32, 32]} />
          <meshStandardMaterial
            ref={centerMatRef}
            color={COLOR.gold}
            emissive={COLOR.gold}
            emissiveIntensity={1.8}
            toneMapped={false}
          />
        </mesh>
        <mesh>
          <sphereGeometry args={[0.12, 32, 32]} />
          <meshBasicMaterial color={COLOR.parchment} transparent opacity={0.18} toneMapped={false} />
        </mesh>
      </group>
    </Float>
  );
}
