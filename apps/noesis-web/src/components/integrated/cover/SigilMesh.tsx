"use client";

// ─── SigilMesh — volumetric extruded topology ──────────────────────────
// Parses the topology SVG string with SVGLoader, extracts the line and
// circle primitives, then synthesizes a 3D sigil from them. The witness
// triangle (3 nodes + 3 edges + center pulse) is rebuilt geometrically
// rather than literally extruding strokes — extruding 1.1px-wide SVG
// strokes produces invisible threads at scale.
//
// Per design § 5.2 + Chapter 0: liquid-glass via MeshTransmissionMaterial,
// emissive Coherence Emerald, central body in Sacred Gold.

import { useMemo, useRef } from "react";
import { useFrame } from "@react-three/fiber";
import { Float, MeshTransmissionMaterial } from "@react-three/drei";
import * as THREE from "three";

interface SigilMeshProps {
  topologySvg: string;
  reducedMotion?: boolean;
}

// Extract subject-node positions from the SVG using a regex over the
// halo-pulse circles (radius 96.8 markers).
function extractNodes(svg: string): Array<{ x: number; y: number; color: string }> {
  const out: Array<{ x: number; y: number; color: string }> = [];
  const re = /<circle\s+cx="([\d.]+)"\s+cy="([\d.]+)"\s+r="96\.8"[^>]*stroke="(#[0-9A-Fa-f]{6})"/g;
  let m: RegExpExecArray | null;
  while ((m = re.exec(svg)) !== null) {
    out.push({ x: parseFloat(m[1]), y: parseFloat(m[2]), color: m[3] });
  }
  return out;
}

/** Map SVG-space (720×720, y-down) to Three-space (centered, y-up, ±1.5). */
function svgToWorld(x: number, y: number, scale = 1 / 240): [number, number, number] {
  return [(x - 360) * scale, -(y - 360) * scale, 0];
}

export function SigilMesh({ topologySvg, reducedMotion = false }: SigilMeshProps) {
  const groupRef = useRef<THREE.Group>(null);
  const emissiveRef = useRef<THREE.MeshStandardMaterial>(null);
  const t0 = useRef<number | null>(null);

  const nodes = useMemo(() => extractNodes(topologySvg), [topologySvg]);

  // Build connection lines between nodes (triad triangle)
  const edges = useMemo(() => {
    if (nodes.length < 2) return [] as Array<[THREE.Vector3, THREE.Vector3]>;
    const pts = nodes.map((n) => new THREE.Vector3(...svgToWorld(n.x, n.y)));
    const result: Array<[THREE.Vector3, THREE.Vector3]> = [];
    for (let i = 0; i < pts.length; i++) {
      for (let j = i + 1; j < pts.length; j++) {
        result.push([pts[i], pts[j]]);
      }
    }
    return result;
  }, [nodes]);

  // Auto-rotate the sigil group
  useFrame((state, delta) => {
    if (t0.current === null) t0.current = state.clock.elapsedTime;
    const elapsed = state.clock.elapsedTime - t0.current;

    if (groupRef.current && !reducedMotion) {
      groupRef.current.rotation.y += delta * 0.06; // 0.06 rad/s
    }

    if (emissiveRef.current) {
      // Fade emissive from 0 → 0.4 over 1.6s starting at 0.8s
      const local = Math.max(0, elapsed - 0.8);
      const t = Math.min(1, local / 1.6);
      const targetEmissive = 0.4 * t;
      emissiveRef.current.emissiveIntensity = reducedMotion ? 0.4 : targetEmissive;
      emissiveRef.current.opacity = Math.min(1, t * 1.2);
    }
  });

  // Tube geometries for connecting edges
  const edgeTubes = useMemo(() => {
    return edges.map(([a, b]) => {
      const curve = new THREE.LineCurve3(a, b);
      return new THREE.TubeGeometry(curve, 1, 0.012, 8, false);
    });
  }, [edges]);

  // Central core ring positions from SVG (the inner field circle at
  // cx=360, cy=374, r=44)
  const center = svgToWorld(360, 374);

  return (
    <Float
      speed={reducedMotion ? 0 : 1.2}
      rotationIntensity={reducedMotion ? 0 : 0.15}
      floatIntensity={reducedMotion ? 0 : 0.4}
    >
      <group ref={groupRef}>
        {/* Subject nodes — luminous orbs at each triangle vertex */}
        {nodes.map((n, i) => {
          const [x, y, z] = svgToWorld(n.x, n.y);
          return (
            <group key={i} position={[x, y, z]}>
              {/* Inner solid sphere — emits color */}
              <mesh>
                <sphereGeometry args={[0.09, 32, 32]} />
                <meshStandardMaterial
                  color={n.color}
                  emissive={n.color}
                  emissiveIntensity={1.4}
                  toneMapped={false}
                />
              </mesh>
              {/* Halo — translucent outer shell */}
              <mesh>
                <sphereGeometry args={[0.32, 32, 32]} />
                <meshBasicMaterial
                  color={n.color}
                  transparent
                  opacity={0.08}
                  toneMapped={false}
                />
              </mesh>
              {/* Ring — flat band around node */}
              <mesh rotation={[Math.PI / 2, 0, 0]}>
                <ringGeometry args={[0.34, 0.36, 64]} />
                <meshBasicMaterial
                  color={n.color}
                  transparent
                  opacity={0.6}
                  side={THREE.DoubleSide}
                  toneMapped={false}
                />
              </mesh>
            </group>
          );
        })}

        {/* Connecting threads — luminous tubes between every pair of nodes */}
        {edgeTubes.map((geo, i) => (
          <mesh key={i} geometry={geo}>
            <meshStandardMaterial
              color="#C5A017"
              emissive="#C5A017"
              emissiveIntensity={1.2}
              toneMapped={false}
            />
          </mesh>
        ))}

        {/* Central liquid-glass core — TRIAD FIELD */}
        <group position={[center[0], center[1], 0]}>
          {/* Outer ring */}
          <mesh rotation={[Math.PI / 2, 0, 0]}>
            <ringGeometry args={[0.42, 0.44, 96]} />
            <meshBasicMaterial color="#C5A017" transparent opacity={0.55} side={THREE.DoubleSide} toneMapped={false} />
          </mesh>
          {/* Mid ring */}
          <mesh rotation={[Math.PI / 2, 0, 0]}>
            <ringGeometry args={[0.30, 0.305, 96]} />
            <meshBasicMaterial color="#C5A017" transparent opacity={0.35} side={THREE.DoubleSide} toneMapped={false} />
          </mesh>
          {/* Inner ring */}
          <mesh rotation={[Math.PI / 2, 0, 0]}>
            <ringGeometry args={[0.18, 0.184, 96]} />
            <meshBasicMaterial color="#C5A017" transparent opacity={0.25} side={THREE.DoubleSide} toneMapped={false} />
          </mesh>
          {/* Liquid-glass core sphere — MeshTransmissionMaterial */}
          <mesh>
            <sphereGeometry args={[0.18, 64, 64]} />
            <MeshTransmissionMaterial
              backside
              samples={4}
              thickness={0.6}
              roughness={0.05}
              chromaticAberration={0.08}
              transmission={1}
              ior={1.3}
              distortion={0.2}
              distortionScale={0.4}
              temporalDistortion={0.1}
              color="#C5A017"
              attenuationColor="#10B5A7"
              attenuationDistance={0.6}
            />
          </mesh>
          {/* Emissive bloom-driver sphere — modulates the bloom output */}
          <mesh>
            <sphereGeometry args={[0.10, 32, 32]} />
            <meshStandardMaterial
              ref={emissiveRef}
              color="#10B5A7"
              emissive="#10B5A7"
              emissiveIntensity={0}
              transparent
              opacity={0}
              toneMapped={false}
            />
          </mesh>
        </group>
      </group>
    </Float>
  );
}
