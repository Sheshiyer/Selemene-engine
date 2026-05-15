"use client";

// ─── AtmosphericRings — concentric 3D rings ────────────────────────────
// Four ring meshes radiating outward from the sigil. Drawn in via opacity
// fade at staggered delays per design § 5.2 (0.5s+).

import { useRef } from "react";
import { useFrame } from "@react-three/fiber";
import * as THREE from "three";

interface RingConfig {
  radius: number;
  color: string;
  opacity: number;
  delay: number;
  thickness: number;
}

const RINGS: RingConfig[] = [
  { radius: 1.35, color: "#C5A017", opacity: 0.14, delay: 0.5, thickness: 0.006 },
  { radius: 1.60, color: "#C5A017", opacity: 0.18, delay: 0.65, thickness: 0.006 },
  { radius: 1.80, color: "#10B5A7", opacity: 0.22, delay: 0.8, thickness: 0.008 },
  { radius: 2.05, color: "#C5A017", opacity: 0.10, delay: 0.95, thickness: 0.005 },
];

function Ring({ config, reducedMotion }: { config: RingConfig; reducedMotion: boolean }) {
  const matRef = useRef<THREE.MeshBasicMaterial>(null);
  const t0 = useRef<number | null>(null);

  useFrame((state) => {
    if (!matRef.current) return;
    if (t0.current === null) t0.current = state.clock.elapsedTime;
    const elapsed = state.clock.elapsedTime - t0.current;
    if (reducedMotion) {
      matRef.current.opacity = config.opacity;
      return;
    }
    const localT = Math.max(0, elapsed - config.delay);
    const fade = Math.min(1, localT / 2.0);
    matRef.current.opacity = config.opacity * fade;
  });

  return (
    <mesh rotation={[Math.PI / 2, 0, 0]}>
      <ringGeometry args={[config.radius - config.thickness, config.radius + config.thickness, 128]} />
      <meshBasicMaterial
        ref={matRef}
        color={config.color}
        transparent
        opacity={0}
        side={THREE.DoubleSide}
        toneMapped={false}
      />
    </mesh>
  );
}

export function AtmosphericRings({ reducedMotion = false }: { reducedMotion?: boolean }) {
  return (
    <group>
      {RINGS.map((r, i) => (
        <Ring key={i} config={r} reducedMotion={reducedMotion} />
      ))}
    </group>
  );
}
