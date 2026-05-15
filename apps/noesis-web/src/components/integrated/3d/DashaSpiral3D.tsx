"use client";

// ─── DashaSpiral3D — Pass γ / phase-lock yantra in 3D ──────────────────
// Per W10 brief: 9 concentric tori sized by Vimshottari period years.
// Past = Witness Violet, current = Sacred Gold (extra emissive),
// future = Coherence Emerald. The whole spiral rotates around the
// Z-axis at 0.02 rad/s.

import { useMemo, useRef } from "react";
import { useFrame } from "@react-three/fiber";
import { Float } from "@react-three/drei";
import * as THREE from "three";

import { COLOR } from "./_shared";

interface DashaPeriod {
  lord: string;
  start_iso: string;
  end_iso: string;
  current?: boolean;
}

interface DashaSpiral3DProps {
  reducedMotion: boolean;
  dashaPeriods?: DashaPeriod[];
}

// Canonical Vimshottari period lengths (years).
const VIMSHOTTARI_YEARS: Record<string, number> = {
  ketu: 7,
  venus: 20,
  sun: 6,
  moon: 10,
  mars: 7,
  rahu: 18,
  jupiter: 16,
  saturn: 19,
  mercury: 17,
};

const VIMSHOTTARI_ORDER = [
  "ketu",
  "venus",
  "sun",
  "moon",
  "mars",
  "rahu",
  "jupiter",
  "saturn",
  "mercury",
] as const;

type LordState = "past" | "current" | "future";

function colorFor(state: LordState): string {
  if (state === "current") return COLOR.gold;
  if (state === "past") return COLOR.violet;
  return COLOR.emerald;
}

function opacityFor(state: LordState): number {
  if (state === "current") return 0.95;
  if (state === "past") return 0.45;
  return 0.6;
}

export function DashaSpiral3DScene({ reducedMotion, dashaPeriods = [] }: DashaSpiral3DProps) {
  const groupRef = useRef<THREE.Group>(null);
  const currentMatRef = useRef<THREE.MeshStandardMaterial>(null);

  // Compute state per lord using caller data; default future.
  const stateByLord = useMemo(() => {
    const today = new Date();
    const map: Record<string, LordState> = {};
    for (const lord of VIMSHOTTARI_ORDER) map[lord] = "future";
    for (const p of dashaPeriods) {
      const start = new Date(p.start_iso);
      const end = new Date(p.end_iso);
      const lord = p.lord.toLowerCase();
      if (p.current || (today >= start && today <= end)) {
        map[lord] = "current";
      } else if (end < today) {
        map[lord] = "past";
      }
    }
    return map;
  }, [dashaPeriods]);

  // Map sqrt(years) → radius so Venus (20yr) doesn't visually dominate.
  const rings = useMemo(() => {
    const years = VIMSHOTTARI_ORDER.map((l) => VIMSHOTTARI_YEARS[l]);
    const yrMax = Math.max(...years);
    const yrMin = Math.min(...years);
    const minR = 0.18;
    const maxR = 1.05;
    return VIMSHOTTARI_ORDER.map((lord) => {
      const y = VIMSHOTTARI_YEARS[lord];
      const t = (Math.sqrt(y) - Math.sqrt(yrMin)) / (Math.sqrt(yrMax) - Math.sqrt(yrMin));
      return {
        lord,
        state: stateByLord[lord],
        r: minR + t * (maxR - minR),
      };
    }).sort((a, b) => a.r - b.r);
  }, [stateByLord]);

  useFrame((state, delta) => {
    if (groupRef.current && !reducedMotion) {
      groupRef.current.rotation.z += delta * 0.02;
    }
    if (currentMatRef.current) {
      currentMatRef.current.emissiveIntensity = reducedMotion
        ? 2.0
        : 1.6 + Math.sin(state.clock.elapsedTime * 0.5) * 0.6;
    }
  });

  return (
    <Float
      speed={reducedMotion ? 0 : 0.5}
      rotationIntensity={reducedMotion ? 0 : 0.15}
      floatIntensity={reducedMotion ? 0 : 0.25}
    >
      <group ref={groupRef}>
        {rings.map((ring, i) => {
          const isCurrent = ring.state === "current";
          return (
            <mesh key={ring.lord} rotation={[Math.PI / 2, 0, 0]}>
              <torusGeometry args={[ring.r, isCurrent ? 0.012 : 0.006, 12, 128]} />
              <meshStandardMaterial
                ref={isCurrent ? currentMatRef : undefined}
                color={colorFor(ring.state)}
                emissive={colorFor(ring.state)}
                emissiveIntensity={isCurrent ? 2.0 : 0.7}
                transparent
                opacity={opacityFor(ring.state)}
                toneMapped={false}
              />
            </mesh>
          );
        })}

        {/* Centre seed. */}
        <mesh>
          <sphereGeometry args={[0.07, 32, 32]} />
          <meshStandardMaterial
            color={COLOR.gold}
            emissive={COLOR.gold}
            emissiveIntensity={2.2}
            toneMapped={false}
          />
        </mesh>
      </group>
    </Float>
  );
}
