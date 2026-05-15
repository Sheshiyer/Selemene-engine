"use client";

// ─── CoverScene — Three.js bloom-shader hero ───────────────────────────
// Replaces the v0 OrbitalCover SVG with a volumetric 3D scene:
//   • react-three-fiber Canvas (transparent, dpr-capped)
//   • Auto-rotating sigil group (0.05 rad/s) with Float
//   • MeshTransmissionMaterial liquid-glass core + emissive bloom driver
//   • Subject names as Billboard-anchored Text constellation
//   • DOM SVG curved text overlay above Canvas (preserves textPath arcs)
//   • Bloom postprocessing: intensity 1.3, threshold 0.7, radius 0.8
//   • WebGL fallback: legacy OrbitalCover SVG-only render
//   • prefers-reduced-motion: disables rotation + Float, static bloom
//
// Per design § 5.2 + Chapter 0.

import { Suspense, useEffect, useMemo, useState } from "react";
import { Canvas } from "@react-three/fiber";
import { EffectComposer, Bloom } from "@react-three/postprocessing";
import { KernelSize } from "postprocessing";

import { AtmosphericRings } from "./AtmosphericRings";
import { SigilMesh } from "./SigilMesh";
import { SubjectStars } from "./SubjectStars";
import { OverlayText } from "./OverlayText";
import { OrbitalCover } from "../OrbitalCover";

interface CoverSceneProps {
  title: string;
  birthMeta: string;
  subjects: string[];
  topologySvg: string;
  tagline?: string;
}

/** Detect WebGL2 availability — falls back to a 2D SVG cover when absent. */
function detectWebGL(): boolean {
  if (typeof window === "undefined") return false;
  try {
    const canvas = document.createElement("canvas");
    const gl = canvas.getContext("webgl2") || canvas.getContext("webgl");
    return !!gl;
  } catch {
    return false;
  }
}

/** Watch prefers-reduced-motion media query. */
function usePrefersReducedMotion(): boolean {
  const [reduced, setReduced] = useState(false);
  useEffect(() => {
    if (typeof window === "undefined") return;
    const mq = window.matchMedia("(prefers-reduced-motion: reduce)");
    setReduced(mq.matches);
    const handler = (e: MediaQueryListEvent) => setReduced(e.matches);
    mq.addEventListener("change", handler);
    return () => mq.removeEventListener("change", handler);
  }, []);
  return reduced;
}

export function CoverScene({
  title,
  birthMeta,
  subjects,
  topologySvg,
  tagline = "Self-Consciousness as Technology · Body as Medium · Breath as Interface",
}: CoverSceneProps) {
  const [webgl, setWebgl] = useState<boolean | null>(null);
  const reducedMotion = usePrefersReducedMotion();

  useEffect(() => {
    setWebgl(detectWebGL());
  }, []);

  // SSR / pre-detect: render the legacy SVG cover so hydration matches and
  // there is never a blank flash. WebGL upgrade replaces it on the client.
  if (webgl === null || webgl === false) {
    return (
      <OrbitalCover
        title={title}
        birthMeta={birthMeta}
        subjects={subjects}
        topologySvg={topologySvg}
        tagline={tagline}
      />
    );
  }

  return (
    <section
      style={{
        position: "relative",
        width: "100vw",
        height: "100vh",
        minHeight: "100vh",
        overflow: "hidden",
        background:
          "radial-gradient(ellipse at 50% 30%, rgba(11,80,251,0.10) 0%, transparent 55%), radial-gradient(ellipse at 50% 70%, rgba(45,0,80,0.18) 0%, transparent 60%), var(--c-void)",
        zIndex: 2,
      }}
    >
      <Canvas
        dpr={[1, 2]}
        gl={{ alpha: true, antialias: true, powerPreference: "high-performance" }}
        camera={{ fov: 35, position: [0, 0, 5], near: 0.1, far: 50 }}
        style={{ position: "absolute", inset: 0, width: "100%", height: "100%" }}
      >
        {/* Ambient + directional lights — Three Laws compliant
            (no clinical fluorescents, soft Goethe-derived warm/cool pair) */}
        <ambientLight intensity={0.35} color="#F0EDE3" />
        <directionalLight position={[3, 4, 5]} intensity={0.6} color="#C5A017" />
        <directionalLight position={[-3, -2, 3]} intensity={0.3} color="#0B50FB" />
        <pointLight position={[0, 0, 2]} intensity={0.4} color="#10B5A7" />

        <Suspense fallback={null}>
          <AtmosphericRings reducedMotion={reducedMotion} />
          <SigilMesh topologySvg={topologySvg} reducedMotion={reducedMotion} />
          <SubjectStars subjects={subjects} reducedMotion={reducedMotion} />
        </Suspense>

        <EffectComposer multisampling={0}>
          <Bloom
            intensity={1.3}
            luminanceThreshold={0.7}
            luminanceSmoothing={0.4}
            mipmapBlur
            radius={0.8}
            kernelSize={KernelSize.LARGE}
          />
        </EffectComposer>
      </Canvas>

      <OverlayText
        title={title}
        birthMeta={birthMeta}
        tagline={tagline}
        reducedMotion={reducedMotion}
      />
    </section>
  );
}
