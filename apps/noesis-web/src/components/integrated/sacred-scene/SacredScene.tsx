"use client";

// ─── <SacredScene> — GLSL-shader-driven Three.js scene primitive ─────────
// Single reusable React component for cover, Part headers, transitions,
// chapter dividers, and closing. Eight `kind` presets produce eleven
// visually-distinct scenes from one shader codebase.
//
//   • Canvas (r3f) with DPR cap [1, 2], FOV 35°, transparent background
//   • Per-kind preset drives: core/edge color, fog tint, bloom, rotate
//     speed, particle count, ribbon visibility, fbm noise scale, camera Z
//   • EffectComposer + Bloom (intensity × kind.bloom × intensity prop)
//   • WebGL detection — parchment-colored placeholder if unavailable
//   • prefers-reduced-motion — freezes animation, keeps static composition
//   • Children overlay (DOM) sits above the Canvas
//
// Per DESIGN.md Three Laws: bioluminescent (no external spotlights),
// architectural (every element structural), data as sacred form.

import {
  Suspense,
  useEffect,
  useMemo,
  useRef,
  useState,
  type ReactNode,
  type CSSProperties,
} from "react";
import { Canvas, useFrame } from "@react-three/fiber";
import { EffectComposer, Bloom } from "@react-three/postprocessing";
import { KernelSize } from "postprocessing";
import * as THREE from "three";

import { PRESETS, type SceneKind } from "./presets";
import { SigilCore } from "./SigilCore";
import { Aura } from "./Aura";
import { WaveRibbon } from "./WaveRibbon";
import { SceneFog } from "./SceneFog";

export interface SacredSceneProps {
  kind: SceneKind;
  /** 0-1 intensity multiplier — scales bloom + animation amplitude. */
  intensity?: number;
  /** Procedural seed; different mounts diverge visually. */
  seed?: number;
  /** Container height. */
  height?: string | number;
  /** DOM overlay above the canvas. */
  children?: ReactNode;
  /** Optional extra class for the outer section. */
  className?: string;
  /** Optional inline style override for the outer section. */
  style?: CSSProperties;
}

/** WebGL2/WebGL1 capability probe. SSR-safe. */
function detectWebGL(): boolean {
  if (typeof window === "undefined") return false;
  try {
    const c = document.createElement("canvas");
    const gl = c.getContext("webgl2") || c.getContext("webgl");
    return !!gl;
  } catch {
    return false;
  }
}

/** Watch prefers-reduced-motion. */
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

/** Auto-rotating group — wraps everything that should orbit slowly. */
function RotatingGroup({
  speed,
  reducedMotion,
  children,
}: {
  speed: number;
  reducedMotion: boolean;
  children: ReactNode;
}) {
  const ref = useRef<THREE.Group>(null);
  useFrame((_, delta) => {
    if (ref.current && !reducedMotion) {
      ref.current.rotation.y += delta * speed;
    }
  });
  return <group ref={ref}>{children}</group>;
}

export function SacredScene({
  kind,
  intensity = 1,
  seed = 0,
  height = "100vh",
  children,
  className,
  style,
}: SacredSceneProps) {
  const preset = PRESETS[kind];
  const reducedMotion = usePrefersReducedMotion();
  const [webgl, setWebgl] = useState<boolean | null>(null);

  useEffect(() => {
    setWebgl(detectWebGL());
  }, []);

  const clampedIntensity = Math.max(0, Math.min(1, intensity));
  const bloomIntensity = preset.bloom * clampedIntensity * 1.0;

  // Container shell — radial gradient backdrop tinted by the fog color so
  // even before Canvas mounts the user sees the right palette (no white
  // flash, no Void Black hole).
  const sectionStyle = useMemo<CSSProperties>(
    () => ({
      position: "relative",
      width: "100%",
      height,
      overflow: "hidden",
      background: `radial-gradient(ellipse at 50% 45%, ${preset.fogTint}40 0%, transparent 55%), var(--c-void, #070B1D)`,
      ...style,
    }),
    [preset.fogTint, height, style],
  );

  // No WebGL — render parchment-colored placeholder with the overlay still
  // visible so the page is never broken.
  if (webgl === false) {
    return (
      <section
        className={className}
        style={{
          ...sectionStyle,
          background: `radial-gradient(ellipse at 50% 45%, ${preset.fogTint}33 0%, transparent 60%), var(--c-void, #070B1D)`,
        }}
        data-sacred-scene={kind}
        data-fallback="webgl-unavailable"
      >
        {children}
      </section>
    );
  }

  // Pre-detect SSR / hydration: render container only. Canvas mounts client-side.
  return (
    <section
      className={className}
      style={sectionStyle}
      data-sacred-scene={kind}
    >
      {webgl === true && (
        <Canvas
          dpr={[1, 2]}
          gl={{
            alpha: true,
            antialias: true,
            powerPreference: "high-performance",
          }}
          camera={{
            fov: 35,
            position: [0, 0, preset.cameraZ],
            near: 0.1,
            far: 50,
          }}
          style={{
            position: "absolute",
            inset: 0,
            width: "100%",
            height: "100%",
          }}
        >
          {/* Bioluminescent ambient + soft warm/cool pair only.
              Per DESIGN.md: no external spotlights. */}
          <ambientLight intensity={0.25} color="#F0EDE3" />
          <directionalLight position={[3, 4, 5]} intensity={0.35} color="#C5A017" />
          <directionalLight position={[-3, -2, 3]} intensity={0.2} color="#0B50FB" />

          <Suspense fallback={null}>
            <SceneFog tint={preset.fogTint} reducedMotion={reducedMotion} />

            <RotatingGroup speed={preset.rotate} reducedMotion={reducedMotion}>
              <SigilCore
                coreColor={preset.core}
                edgeColor={preset.edge}
                noiseScale={preset.noiseScale}
                reducedMotion={reducedMotion}
                seed={seed}
              />
              <Aura
                count={Math.round(preset.particles * clampedIntensity)}
                reducedMotion={reducedMotion}
                seed={seed}
              />
              {preset.ribbon && (
                <WaveRibbon reducedMotion={reducedMotion} seed={seed} />
              )}
            </RotatingGroup>
          </Suspense>

          <EffectComposer multisampling={0}>
            <Bloom
              intensity={bloomIntensity}
              luminanceThreshold={0.3}
              luminanceSmoothing={0.4}
              mipmapBlur
              radius={0.9}
              kernelSize={KernelSize.LARGE}
            />
          </EffectComposer>
        </Canvas>
      )}

      {children}
    </section>
  );
}
