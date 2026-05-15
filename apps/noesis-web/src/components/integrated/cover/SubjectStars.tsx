"use client";

// ─── SubjectStars — orbiting constellation labels ──────────────────────
// Each subject name floats at a compass position around the sigil. We
// use drei's Text (Troika SDF) rather than Text3D because:
//   • No public typeface.json font asset is provisioned in this repo
//   • SDF text renders crisp under bloom postprocessing
//   • Bloom on Text3D's triangulated geometry tends to over-glow edges
//
// Per design § 5.2: stagger 0.18s, orbit slow, billboarded so labels
// always face camera regardless of group rotation.

import { useRef } from "react";
import { useFrame } from "@react-three/fiber";
import { Billboard, Text } from "@react-three/drei";
import * as THREE from "three";

interface SubjectStarsProps {
  subjects: string[];
  reducedMotion?: boolean;
}

const RADIUS = 3.0;
const STAGGER = 0.18;
const BASE_DELAY = 1.2;

function StarLabel({ name, angle, delay, reducedMotion }: {
  name: string;
  angle: number;
  delay: number;
  reducedMotion: boolean;
}) {
  const groupRef = useRef<THREE.Group>(null);
  const textRef = useRef<{ material?: THREE.Material; fillOpacity?: number; outlineOpacity?: number } | null>(null);
  const dotRef = useRef<THREE.MeshBasicMaterial>(null);
  const t0 = useRef<number | null>(null);

  useFrame((state) => {
    if (t0.current === null) t0.current = state.clock.elapsedTime;
    const elapsed = state.clock.elapsedTime - t0.current;
    const local = Math.max(0, elapsed - delay);
    const fade = reducedMotion ? 1 : Math.min(1, local / 0.9);

    // Position with offset-y for the entry slide
    const yOffset = reducedMotion ? 0 : (1 - fade) * 0.1;

    if (groupRef.current) {
      const x = RADIUS * Math.cos(angle);
      const z = RADIUS * Math.sin(angle);
      groupRef.current.position.set(x, yOffset, z);
    }

    // Opacity ramp
    if (textRef.current) {
      // troika-text exposes fillOpacity directly
      (textRef.current as { fillOpacity: number }).fillOpacity = fade;
    }
    if (dotRef.current) {
      dotRef.current.opacity = fade * 0.9;
    }
  });

  return (
    <group ref={groupRef}>
      <Billboard follow lockX={false} lockY={false} lockZ={false}>
        <Text
          ref={(r) => {
            textRef.current = r as unknown as { fillOpacity: number };
          }}
          font={undefined}
          fontSize={0.18}
          letterSpacing={0.18}
          color="#F0EDE3"
          anchorX="center"
          anchorY="middle"
          outlineWidth={0.005}
          outlineColor="#0B50FB"
          outlineOpacity={0.4}
          fillOpacity={0}
        >
          {name.toUpperCase()}
        </Text>
        {/* Pulse dot just below the label, toward sigil */}
        <mesh position={[0, -0.22, 0]}>
          <circleGeometry args={[0.025, 24]} />
          <meshBasicMaterial
            ref={dotRef}
            color="#C5A017"
            transparent
            opacity={0}
            toneMapped={false}
          />
        </mesh>
      </Billboard>
    </group>
  );
}

export function SubjectStars({ subjects, reducedMotion = false }: SubjectStarsProps) {
  const N = subjects.length;
  return (
    <group>
      {subjects.map((name, i) => {
        // First subject at top (-π/2 in XZ plane = -Z direction)
        // Distribute clockwise around Y-axis. In XZ plane, clockwise viewed
        // from +Y means angle increases negatively.
        const angle = -Math.PI / 2 + (i * 2 * Math.PI) / Math.max(N, 1);
        const delay = BASE_DELAY + i * STAGGER;
        return (
          <StarLabel
            key={name}
            name={name}
            angle={angle}
            delay={delay}
            reducedMotion={reducedMotion}
          />
        );
      })}
    </group>
  );
}
