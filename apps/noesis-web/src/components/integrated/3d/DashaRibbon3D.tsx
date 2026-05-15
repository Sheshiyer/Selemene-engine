"use client";

// ─── DashaRibbon3D — volumetric iridescent dasha timeline ──────────────
// Per W10 brief: replaces W2's flat 1200×280 SVG waveform with a real
// 3D tube along a Catmull-Rom spline. Past = Witness Violet, current =
// Sacred Gold (extra emissive), future = Coherence Emerald.
//
//  • TubeGeometry along ~480 spline samples.
//  • Vertex colors painted segment-by-segment.
//  • Pivot spheres at each segment boundary + thin cylindrical hairlines.
//  • Bloom postprocessing (intensity 1.4 — heavier than the yantras so
//    iridescent currents really glow).
//  • Slow horizontal flow drift via UV offset on the tube's material.
//  • prefers-reduced-motion → static ribbon, no drift.
//  • WebGL fallback → renders the legacy 2D DashaWaveform.

import { Suspense, useMemo, useRef } from "react";
import { Canvas, useFrame } from "@react-three/fiber";
import { EffectComposer, Bloom } from "@react-three/postprocessing";
import { KernelSize } from "postprocessing";
import * as THREE from "three";

import { DashaWaveform as DashaWaveformLegacy } from "../yantras/DashaWaveform";
import type { DashaSegment } from "../yantras/DashaWaveform";

import { useWebGL, usePrefersReducedMotion, COLOR } from "./_shared";

interface DashaRibbon3DProps {
  periods: DashaSegment[];
  pivots?: Array<{ iso: string; label: string }>;
  antardashas?: Array<{
    parent_lord: string;
    lord: string;
    start_iso: string;
    end_iso: string;
  }>;
}

// Intensity / amplitude per lord (matches W2 DashaWaveform's lookups).
const LORD_INTENSITY: Record<string, number> = {
  mars: 1.7, rahu: 1.4, sun: 1.2, mercury: 1.1, moon: 0.9,
  venus: 0.8, jupiter: 0.7, ketu: 1.5, saturn: 0.5,
};
const LORD_AMP: Record<string, number> = {
  mars: 1.0, rahu: 0.9, sun: 0.85, mercury: 0.7, moon: 0.75,
  venus: 0.6, jupiter: 0.55, ketu: 0.95, saturn: 0.4,
};

function colorForState(state: DashaSegment["state"]): THREE.Color {
  if (state === "current") return new THREE.Color(COLOR.gold);
  if (state === "past") return new THREE.Color(COLOR.violet);
  return new THREE.Color(COLOR.emerald);
}

interface RibbonProps {
  periods: DashaSegment[];
  reducedMotion: boolean;
}

function Ribbon({ periods, reducedMotion }: RibbonProps) {
  const materialRef = useRef<THREE.MeshStandardMaterial>(null);

  // Geometry — compute the spline, sample it, build a TubeGeometry with
  // per-vertex colors painted by the segment each sample falls within.
  const { tubeGeo, segLayout } = useMemo(() => {
    // World-space dimensions: width 6, height 1 (camera at z=6, FOV 35).
    const W = 6.0;
    const H = 1.0;
    const x0 = -W / 2;
    const total = periods.reduce((s, p) => s + p.duration_years, 0) || 1;

    // Pre-layout each segment's x-range.
    let cursor = x0;
    const segLayout = periods.map((p) => {
      const w = (p.duration_years / total) * W;
      const x0s = cursor;
      const x1s = cursor + w;
      cursor = x1s;
      return { ...p, x0s, x1s, w };
    });

    // Sample spline.
    const samples = 480;
    const pts: THREE.Vector3[] = [];
    for (let i = 0; i <= samples; i++) {
      const x = x0 + (i / samples) * W;
      const seg = segLayout.find((s) => x >= s.x0s && x <= s.x1s) ?? segLayout[segLayout.length - 1];
      const lord = seg.lord.toLowerCase();
      const amp = (LORD_AMP[lord] ?? 0.6) * H * 0.5;
      const freq = LORD_INTENSITY[lord] ?? 1.0;
      const localT = (x - seg.x0s) / Math.max(seg.w, 1e-6);
      const y =
        amp *
        Math.sin(
          (localT * Math.PI * 2 * freq * Math.max(seg.duration_years, 1)) / 4 +
            seg.x0s * 0.4,
        );
      // Slight z waver so the ribbon feels truly volumetric, not a flat
      // band in a plane.
      const z = Math.sin(localT * Math.PI * 2 + seg.x0s) * 0.04;
      pts.push(new THREE.Vector3(x, y, z));
    }

    const curve = new THREE.CatmullRomCurve3(pts, false, "centripetal");
    const tubularSegments = samples;
    const radialSegments = 10;
    const tubeGeo = new THREE.TubeGeometry(curve, tubularSegments, 0.04, radialSegments, false);

    // Paint vertex colors by segment state. Tube vertex count is
    // (tubularSegments + 1) * (radialSegments + 1). The tubularSegments
    // axis maps 1:1 to our sample index.
    const colorAttr = new Float32Array(tubeGeo.attributes.position.count * 3);
    const verticesPerRing = radialSegments + 1;
    for (let i = 0; i <= tubularSegments; i++) {
      const sampleX = x0 + (i / tubularSegments) * W;
      const seg = segLayout.find((s) => sampleX >= s.x0s && sampleX <= s.x1s) ?? segLayout[segLayout.length - 1];
      const c = colorForState(seg.state);
      for (let j = 0; j < verticesPerRing; j++) {
        const idx = (i * verticesPerRing + j) * 3;
        colorAttr[idx + 0] = c.r;
        colorAttr[idx + 1] = c.g;
        colorAttr[idx + 2] = c.b;
      }
    }
    tubeGeo.setAttribute("color", new THREE.BufferAttribute(colorAttr, 3));

    return { tubeGeo, segLayout };
  }, [periods]);

  useFrame((state) => {
    if (!materialRef.current) return;
    if (reducedMotion) {
      materialRef.current.emissiveIntensity = 0.9;
      return;
    }
    // Subtle emissive breathing — represents the "flow" without UV
    // shenanigans (TubeGeometry UVs aren't trivial to animate).
    const t = state.clock.elapsedTime;
    materialRef.current.emissiveIntensity = 0.7 + Math.sin(t * 0.4) * 0.25;
  });

  return (
    <group>
      {/* The ribbon itself. */}
      <mesh geometry={tubeGeo}>
        <meshStandardMaterial
          ref={materialRef}
          vertexColors
          emissive={COLOR.parchment}
          emissiveIntensity={0.8}
          metalness={0.1}
          roughness={0.4}
          toneMapped={false}
        />
      </mesh>

      {/* Pivot spheres + vertical hairlines at each segment boundary. */}
      {segLayout.slice(0, -1).map((seg, i) => {
        const x = seg.x1s;
        return (
          <group key={`pivot-${i}`}>
            <mesh position={[x, 0, 0]}>
              <sphereGeometry args={[0.05, 16, 16]} />
              <meshStandardMaterial
                color={COLOR.gold}
                emissive={COLOR.gold}
                emissiveIntensity={1.6}
                toneMapped={false}
              />
            </mesh>
            {/* Hairline above + below baseline. */}
            <mesh position={[x, 0, 0]}>
              <cylinderGeometry args={[0.004, 0.004, 0.95, 8]} />
              <meshStandardMaterial
                color={COLOR.gold}
                emissive={COLOR.gold}
                emissiveIntensity={0.7}
                transparent
                opacity={0.55}
                toneMapped={false}
              />
            </mesh>
          </group>
        );
      })}

      {/* Subtle baseline — faint emerald line so the eye registers the
          zero crossing without it being a competing visual element. */}
      <mesh rotation={[0, 0, 0]}>
        <boxGeometry args={[6, 0.003, 0.003]} />
        <meshBasicMaterial color={COLOR.emerald} transparent opacity={0.15} toneMapped={false} />
      </mesh>
    </group>
  );
}

export function DashaRibbon3D({ periods, pivots, antardashas }: DashaRibbon3DProps) {
  const webgl = useWebGL();
  const reduce = usePrefersReducedMotion();

  if (webgl === null || webgl === false) {
    return <DashaWaveformLegacy periods={periods} pivots={pivots} antardashas={antardashas} />;
  }

  return (
    <div
      style={{
        width: "100%",
        margin: "clamp(1.5rem, 4vw, 3rem) 0",
        aspectRatio: "16 / 5",
        maxWidth: "clamp(40rem, 90vw, 75rem)",
        marginLeft: "auto",
        marginRight: "auto",
      }}
      aria-label="Dasha ribbon — volumetric timeline"
      role="img"
    >
      <Canvas
        dpr={[1, 2]}
        gl={{ alpha: true, antialias: true, powerPreference: "high-performance" }}
        camera={{ fov: 35, position: [0, 0.5, 6], near: 0.1, far: 50 }}
        style={{ width: "100%", height: "100%" }}
      >
        <ambientLight intensity={0.4} color={COLOR.parchment} />
        <directionalLight position={[3, 4, 5]} intensity={0.6} color={COLOR.gold} />
        <directionalLight position={[-3, -2, 3]} intensity={0.3} color={COLOR.indigo} />
        <pointLight position={[0, 0, 2]} intensity={0.5} color={COLOR.emerald} />

        <Suspense fallback={null}>
          <Ribbon periods={periods} reducedMotion={reduce} />
        </Suspense>

        <EffectComposer multisampling={0}>
          <Bloom
            intensity={1.4}
            luminanceThreshold={0.55}
            luminanceSmoothing={0.4}
            mipmapBlur
            radius={0.8}
            kernelSize={KernelSize.LARGE}
          />
        </EffectComposer>
      </Canvas>
    </div>
  );
}

// Barrel-friendly alias so DashaWaveform can be drop-in replaced.
export { DashaRibbon3D as DashaWaveform };
