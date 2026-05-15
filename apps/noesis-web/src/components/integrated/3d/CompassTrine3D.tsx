"use client";

// ─── CompassTrine3D — Pass δ / anti-dependency yantra in 3D ────────────
// Per W10 brief: octagonal compass-cage built from thin cylinders.
// Central seed = interlocking-triangle mesh + bloom-glow sphere.
// Cardinal labels rendered as drei <Billboard><Text> at the 4 compass
// points so they always face the camera even as the cage rotates.

import { useMemo, useRef } from "react";
import { useFrame } from "@react-three/fiber";
import { Billboard, Float, Text } from "@react-three/drei";
import * as THREE from "three";

import { COLOR } from "./_shared";

interface CompassTrine3DProps {
  reducedMotion: boolean;
  cardinals?: {
    stabilize?: string;
    heal?: string;
    create?: string;
    mutate?: string;
  };
}

const CARDINAL_POSITIONS = [
  { key: "STABILIZE", angle: -90, accent: COLOR.violet },
  { key: "HEAL", angle: 0, accent: COLOR.indigo },
  { key: "CREATE", angle: 90, accent: COLOR.gold },
  { key: "MUTATE", angle: 180, accent: COLOR.emerald },
] as const;

/** A single octagon edge — a thin cylinder between two vertices. */
function Edge({ a, b, color }: { a: THREE.Vector3; b: THREE.Vector3; color: string }) {
  const { position, quaternion, length } = useMemo(() => {
    const dir = new THREE.Vector3().subVectors(b, a);
    const len = dir.length();
    const mid = new THREE.Vector3().addVectors(a, b).multiplyScalar(0.5);
    // CylinderGeometry default axis is Y — rotate it onto the edge direction.
    const up = new THREE.Vector3(0, 1, 0);
    const q = new THREE.Quaternion().setFromUnitVectors(up, dir.clone().normalize());
    return { position: mid, quaternion: q, length: len };
  }, [a, b]);

  return (
    <mesh position={position} quaternion={quaternion}>
      <cylinderGeometry args={[0.008, 0.008, length, 8]} />
      <meshStandardMaterial
        color={color}
        emissive={color}
        emissiveIntensity={1.1}
        toneMapped={false}
      />
    </mesh>
  );
}

export function CompassTrine3DScene({ reducedMotion, cardinals }: CompassTrine3DProps) {
  const groupRef = useRef<THREE.Group>(null);
  const seedMatRef = useRef<THREE.MeshStandardMaterial>(null);

  const outerR = 0.95;

  // Eight octagon vertices.
  const octVertices = useMemo(() => {
    return Array.from({ length: 8 }, (_, i) => {
      const a = (-90 + i * 45) * (Math.PI / 180);
      return new THREE.Vector3(outerR * Math.cos(a), outerR * Math.sin(a), 0);
    });
  }, []);

  const edges = useMemo(() => {
    const out: Array<[THREE.Vector3, THREE.Vector3]> = [];
    for (let i = 0; i < octVertices.length; i++) {
      out.push([octVertices[i], octVertices[(i + 1) % octVertices.length]]);
    }
    return out;
  }, [octVertices]);

  useFrame((state, delta) => {
    if (!groupRef.current) return;
    if (!reducedMotion) {
      groupRef.current.rotation.z += delta * 0.015;
    }
    if (seedMatRef.current) {
      seedMatRef.current.emissiveIntensity = reducedMotion
        ? 2.0
        : 1.6 + Math.sin(state.clock.elapsedTime * 0.7) * 0.5;
    }
  });

  const cardinalSubtitle = (key: string): string | undefined => {
    if (!cardinals) return undefined;
    if (key === "STABILIZE") return cardinals.stabilize;
    if (key === "HEAL") return cardinals.heal;
    if (key === "CREATE") return cardinals.create;
    if (key === "MUTATE") return cardinals.mutate;
    return undefined;
  };

  return (
    <Float
      speed={reducedMotion ? 0 : 0.5}
      rotationIntensity={reducedMotion ? 0 : 0.15}
      floatIntensity={reducedMotion ? 0 : 0.25}
    >
      <group ref={groupRef}>
        {/* Octagonal cage. */}
        {edges.map(([a, b], i) => (
          <Edge key={i} a={a} b={b} color={COLOR.gold} />
        ))}

        {/* Inner circle — a slim torus. */}
        <mesh rotation={[Math.PI / 2, 0, 0]}>
          <torusGeometry args={[0.7, 0.005, 12, 128]} />
          <meshStandardMaterial
            color={COLOR.emerald}
            emissive={COLOR.emerald}
            emissiveIntensity={0.7}
            transparent
            opacity={0.55}
            toneMapped={false}
          />
        </mesh>

        {/* Spokes from seed to inner ring. */}
        {CARDINAL_POSITIONS.map((p, i) => {
          const a = (p.angle * Math.PI) / 180;
          const x0 = 0.22 * Math.cos(a);
          const y0 = 0.22 * Math.sin(a);
          const x1 = 0.7 * Math.cos(a);
          const y1 = 0.7 * Math.sin(a);
          return (
            <Edge
              key={`spoke-${i}`}
              a={new THREE.Vector3(x0, y0, 0)}
              b={new THREE.Vector3(x1, y1, 0)}
              color={p.accent}
            />
          );
        })}

        {/* Cardinal outpost dots + Billboard labels. */}
        {CARDINAL_POSITIONS.map((p, i) => {
          const a = (p.angle * Math.PI) / 180;
          const dotR = 0.78;
          const labelR = 1.08;
          const dx = dotR * Math.cos(a);
          const dy = dotR * Math.sin(a);
          const lx = labelR * Math.cos(a);
          const ly = labelR * Math.sin(a);
          const subtitle = cardinalSubtitle(p.key);
          return (
            <group key={`card-${i}`}>
              <mesh position={[dx, dy, 0]}>
                <sphereGeometry args={[0.03, 16, 16]} />
                <meshStandardMaterial
                  color={p.accent}
                  emissive={p.accent}
                  emissiveIntensity={1.6}
                  toneMapped={false}
                />
              </mesh>
              <Billboard position={[lx, ly, 0]}>
                <Text
                  fontSize={0.07}
                  color={COLOR.parchment}
                  anchorX="center"
                  anchorY="middle"
                  letterSpacing={0.18}
                  outlineWidth={0.002}
                  outlineColor={COLOR.void}
                >
                  {p.key}
                </Text>
                {subtitle ? (
                  <Text
                    position={[0, -0.08, 0]}
                    fontSize={0.04}
                    color={COLOR.parchment}
                    fillOpacity={0.7}
                    anchorX="center"
                    anchorY="middle"
                    letterSpacing={0.1}
                  >
                    {subtitle}
                  </Text>
                ) : null}
              </Billboard>
            </group>
          );
        })}

        {/* Central seed — interlocking triangles + glow sphere. */}
        <group>
          {/* Upward triangle — gold. */}
          <mesh>
            <ringGeometry args={[0.0, 0.001, 3]} />
            {/* Visible upward triangle implemented via a thin tube path. */}
            <meshBasicMaterial color={COLOR.gold} transparent opacity={0} toneMapped={false} />
          </mesh>
          <TriangleOutline radius={0.14} color={COLOR.gold} rotationZ={0} />
          <TriangleOutline radius={0.14} color={COLOR.emerald} rotationZ={Math.PI} />

          <mesh>
            <sphereGeometry args={[0.05, 24, 24]} />
            <meshStandardMaterial
              ref={seedMatRef}
              color={COLOR.gold}
              emissive={COLOR.gold}
              emissiveIntensity={1.8}
              toneMapped={false}
            />
          </mesh>
        </group>
      </group>
    </Float>
  );
}

/** Three thin cylinder edges arranged as an equilateral triangle. */
function TriangleOutline({
  radius,
  color,
  rotationZ,
}: {
  radius: number;
  color: string;
  rotationZ: number;
}) {
  const vertices = useMemo(() => {
    return Array.from({ length: 3 }, (_, i) => {
      const a = -Math.PI / 2 + (i * 2 * Math.PI) / 3 + rotationZ;
      return new THREE.Vector3(radius * Math.cos(a), radius * Math.sin(a), 0);
    });
  }, [radius, rotationZ]);
  return (
    <group>
      {vertices.map((v, i) => (
        <Edge key={i} a={v} b={vertices[(i + 1) % 3]} color={color} />
      ))}
    </group>
  );
}
