"use client";

// ─── YantraPlate3D — 3D registry/dispatcher for the 4 mandala variants ─
// Per W10 brief: same prop shape as W2 YantraPlate so the parent can
// swap with a single import change. Each kind mounts a Canvas with
// shared lighting + postprocessing Bloom (intensity 1.1, threshold 0.6,
// radius 0.7) and delegates to the corresponding 3D scene module.
//
// WebGL-unavailable / SSR pre-detect → falls back to the W2 2D
// YantraPlate so hydration matches and there's no blank flash.

import { Suspense } from "react";
import { Canvas } from "@react-three/fiber";
import { EffectComposer, Bloom } from "@react-three/postprocessing";
import { KernelSize } from "postprocessing";

import { YantraPlate as YantraPlateLegacy } from "../yantras/YantraPlate";
import type { YantraKind, YantraData } from "../yantras/YantraPlate";

import { useWebGL, usePrefersReducedMotion, COLOR } from "./_shared";
import { TriadMandala3DScene } from "./TriadMandala3D";
import { VesicaTrio3DScene } from "./VesicaTrio3D";
import { DashaSpiral3DScene } from "./DashaSpiral3D";
import { CompassTrine3DScene } from "./CompassTrine3D";

interface YantraPlate3DProps {
  kind: YantraKind;
  data?: YantraData;
  topologySvg?: string;
}

export function YantraPlate3D({ kind, data, topologySvg }: YantraPlate3DProps) {
  const webgl = useWebGL();
  const reduce = usePrefersReducedMotion();
  const d = data ?? {};

  if (webgl === null || webgl === false) {
    return <YantraPlateLegacy kind={kind} data={data} topologySvg={topologySvg} />;
  }

  let scene: React.ReactNode = null;
  switch (kind) {
    case "triad-mandala":
      scene = <TriadMandala3DScene reducedMotion={reduce} subjects={d.subjects} />;
      break;
    case "vesica-trio":
      scene = <VesicaTrio3DScene reducedMotion={reduce} subjects={d.subjects} />;
      break;
    case "dasha-spiral":
      scene = <DashaSpiral3DScene reducedMotion={reduce} dashaPeriods={d.dashaPeriods} />;
      break;
    case "compass-trine":
      scene = <CompassTrine3DScene reducedMotion={reduce} cardinals={d.cardinals} />;
      break;
  }

  return (
    <div
      style={{
        width: "clamp(20rem, 50vw, 50rem)",
        margin: "clamp(1.5rem, 4vw, 3rem) auto",
        aspectRatio: "1 / 1",
      }}
      aria-label={`Yantra plate — ${kind}`}
      role="img"
    >
      <Canvas
        dpr={[1, 2]}
        gl={{ alpha: true, antialias: true, powerPreference: "high-performance" }}
        camera={{ fov: 35, position: [0, 0, 4.2], near: 0.1, far: 50 }}
        style={{ width: "100%", height: "100%" }}
      >
        <ambientLight intensity={0.35} color={COLOR.parchment} />
        <directionalLight position={[3, 4, 5]} intensity={0.55} color={COLOR.gold} />
        <directionalLight position={[-3, -2, 3]} intensity={0.3} color={COLOR.indigo} />
        <pointLight position={[0, 0, 2]} intensity={0.4} color={COLOR.emerald} />

        <Suspense fallback={null}>{scene}</Suspense>

        <EffectComposer multisampling={0}>
          <Bloom
            intensity={1.1}
            luminanceThreshold={0.6}
            luminanceSmoothing={0.4}
            mipmapBlur
            radius={0.7}
            kernelSize={KernelSize.LARGE}
          />
        </EffectComposer>
      </Canvas>
    </div>
  );
}

// Aliased export so the barrel re-exports cleanly as { YantraPlate }.
export { YantraPlate3D as YantraPlate };
