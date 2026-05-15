"use client";

// ─── WitnessPulse3D — volumetric breathing-ring opener ─────────────────
// Per integrated-reading-design-v2.md § 5.3 + W10 brief.
//
// 3D upgrade of the W2 WitnessPulse: three concentric torus rings + a
// central emissive sphere, all pulsing on the 4:7:8 breath cycle.
//
//  • Canvas: 480×480, transparent.
//  • TorusGeometry for each ring (axis-aligned, viewed straight on).
//  • Central emissive sphere drives the bloom output.
//  • Bloom postprocessing (intensity 1.1, threshold 0.6, radius 0.7).
//  • Auto-orbit at 0.02 rad/s around Y for very subtle parallax.
//  • Cardinal direction label as DOM text below the Canvas.
//  • prefers-reduced-motion → no pulse, no orbit.
//  • WebGL fallback → renders the legacy 2D WitnessPulse.

import { Suspense, useRef } from "react";
import { Canvas, useFrame } from "@react-three/fiber";
import { EffectComposer, Bloom } from "@react-three/postprocessing";
import { KernelSize } from "postprocessing";
import * as THREE from "three";

import { WitnessPulse as WitnessPulseLegacy } from "../yantras/WitnessPulse";
import type { WitnessDirection } from "../yantras/WitnessPulse";
import { useWebGL, usePrefersReducedMotion, COLOR } from "./_shared";

interface WitnessPulse3DProps {
  direction: WitnessDirection;
  title?: string;
}

// Breath phase durations — sum 19s, exactly matches W2 WitnessPulse.
const PHASE = { inhale: 4, hold: 7, exhale: 8 } as const;
const CYCLE = PHASE.inhale + PHASE.hold + PHASE.exhale;

const PHASE_LABEL: Record<"inhale" | "hold" | "exhale", string> = {
  inhale: "INHALE · 4",
  hold: "HOLD · 7",
  exhale: "EXHALE · 8",
};

const DIRECTION_ACCENT: Record<WitnessDirection, string> = {
  STABILIZE: COLOR.violet,
  HEAL: COLOR.indigo,
  CREATE: COLOR.gold,
  MUTATE: COLOR.emerald,
};

/**
 * Return a (scale, phaseName) tuple for the given absolute time.
 * Scale follows 1.0 → 1.08 → 1.08 → 1.0 over [0 … inhale … inhale+hold … cycle].
 */
function breath(t: number): { scale: number; phase: "inhale" | "hold" | "exhale" } {
  const local = t % CYCLE;
  if (local < PHASE.inhale) {
    const u = local / PHASE.inhale;
    return { scale: 1.0 + 0.08 * u, phase: "inhale" };
  }
  if (local < PHASE.inhale + PHASE.hold) {
    return { scale: 1.08, phase: "hold" };
  }
  const exhaleT = local - PHASE.inhale - PHASE.hold;
  const u = exhaleT / PHASE.exhale;
  return { scale: 1.08 - 0.08 * u, phase: "exhale" };
}

interface SceneProps {
  accent: string;
  reducedMotion: boolean;
  onPhase: (p: "inhale" | "hold" | "exhale") => void;
}

function Scene({ accent, reducedMotion, onPhase }: SceneProps) {
  const groupRef = useRef<THREE.Group>(null);
  const coreMatRef = useRef<THREE.MeshStandardMaterial>(null);
  const lastPhase = useRef<"inhale" | "hold" | "exhale">("inhale");

  useFrame((state, delta) => {
    const t = state.clock.elapsedTime;
    if (!groupRef.current) return;

    // Auto-orbit on Y axis — extremely subtle.
    if (!reducedMotion) {
      groupRef.current.rotation.y += delta * 0.02;
    }

    // Breath pulse.
    const { scale, phase } = reducedMotion ? { scale: 1, phase: "hold" as const } : breath(t);
    groupRef.current.scale.setScalar(scale);

    if (coreMatRef.current) {
      // Central sphere glow modulates with the breath — brighter at the
      // top of the inhale, dims through exhale.
      const t01 = (scale - 1) / 0.08;
      coreMatRef.current.emissiveIntensity = reducedMotion ? 1.4 : 0.8 + t01 * 1.8;
    }

    if (!reducedMotion && lastPhase.current !== phase) {
      lastPhase.current = phase;
      onPhase(phase);
    }
  });

  // Three rings — radii chosen to read as the W2 SVG version
  // (180/140/100 SVG-px out of 240 ≈ 0.75/0.58/0.42 world units).
  const rings: Array<{ r: number; tube: number; color: string; opacity: number }> = [
    { r: 0.75, tube: 0.008, color: COLOR.indigo, opacity: 0.45 },
    { r: 0.58, tube: 0.012, color: accent, opacity: 0.85 },
    { r: 0.42, tube: 0.008, color: COLOR.indigo, opacity: 0.55 },
  ];

  return (
    <group ref={groupRef}>
      {/* Atmospheric outer disc — very faint accent halo. */}
      <mesh rotation={[Math.PI / 2, 0, 0]}>
        <ringGeometry args={[0.86, 0.92, 96]} />
        <meshBasicMaterial color={accent} transparent opacity={0.18} side={THREE.DoubleSide} toneMapped={false} />
      </mesh>

      {/* Three breathing rings — proper TorusGeometry so they have real
          volume (small tube radius gives a slim halo profile). */}
      {rings.map((r, i) => (
        <mesh key={i} rotation={[Math.PI / 2, 0, 0]}>
          <torusGeometry args={[r.r, r.tube, 16, 96]} />
          <meshStandardMaterial
            color={r.color}
            emissive={r.color}
            emissiveIntensity={1.0}
            transparent
            opacity={r.opacity}
            toneMapped={false}
          />
        </mesh>
      ))}

      {/* Central emissive sphere — drives bloom. */}
      <mesh>
        <sphereGeometry args={[0.18, 48, 48]} />
        <meshStandardMaterial
          ref={coreMatRef}
          color={accent}
          emissive={accent}
          emissiveIntensity={1.4}
          toneMapped={false}
        />
      </mesh>

      {/* Outer translucent halo around the core. */}
      <mesh>
        <sphereGeometry args={[0.30, 32, 32]} />
        <meshBasicMaterial color={accent} transparent opacity={0.12} toneMapped={false} />
      </mesh>
    </group>
  );
}

export function WitnessPulse3D({ direction, title }: WitnessPulse3DProps) {
  const webgl = useWebGL();
  const reduce = usePrefersReducedMotion();
  const accent = DIRECTION_ACCENT[direction];

  // Track phase via a ref-style state so we don't tick React every frame —
  // we only re-render when the phase changes.
  const phaseRef = useRef<"inhale" | "hold" | "exhale">("inhale");
  const [, force] = useForceRender();
  const onPhase = (p: "inhale" | "hold" | "exhale") => {
    if (phaseRef.current !== p) {
      phaseRef.current = p;
      force();
    }
  };

  // SSR / WebGL-unavailable → fall back to the W2 SVG version.
  if (webgl === null || webgl === false) {
    return <WitnessPulseLegacy direction={direction} title={title} />;
  }

  const stageLabel = reduce ? "STILL" : PHASE_LABEL[phaseRef.current];

  return (
    <div
      style={{
        width: "min(100%, 30rem)",
        margin: "clamp(1.5rem, 4vw, 3rem) auto",
        display: "flex",
        flexDirection: "column",
        alignItems: "center",
        gap: "clamp(0.75rem, 1.5vw, 1.25rem)",
      }}
      aria-label={`Witness pulse — ${direction.toLowerCase()} breath`}
      role="img"
    >
      <div
        style={{
          width: "min(100%, clamp(20rem, 50vw, 30rem))",
          aspectRatio: "1 / 1",
          maxWidth: 480,
        }}
      >
        <Canvas
          dpr={[1, 2]}
          gl={{ alpha: true, antialias: true, powerPreference: "high-performance" }}
          camera={{ fov: 35, position: [0, 0, 5], near: 0.1, far: 50 }}
          style={{ width: "100%", height: "100%" }}
        >
          <ambientLight intensity={0.35} color={COLOR.parchment} />
          <directionalLight position={[3, 4, 5]} intensity={0.5} color={COLOR.gold} />
          <directionalLight position={[-3, -2, 3]} intensity={0.25} color={COLOR.indigo} />
          <pointLight position={[0, 0, 2]} intensity={0.35} color={COLOR.emerald} />

          <Suspense fallback={null}>
            <Scene accent={accent} reducedMotion={reduce} onPhase={onPhase} />
          </Suspense>

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

      {/* Cardinal direction label — kept as DOM so it does not pick up
          bloom and remains crisply legible. */}
      <div
        style={{
          fontFamily: "var(--font-display)",
          fontWeight: 700,
          letterSpacing: "0.35em",
          fontSize: "0.85rem",
          color: "var(--c-parchment)",
          opacity: 0.86,
        }}
      >
        {direction}
      </div>

      {/* Stage label — drives the breath cue text. */}
      <div
        style={{
          fontFamily: "var(--font-mono)",
          fontSize: "0.72rem",
          letterSpacing: "0.32em",
          color: "var(--c-emerald)",
          textTransform: "uppercase",
          minHeight: "1em",
        }}
      >
        {stageLabel}
      </div>

      {title ? (
        <div
          style={{
            fontFamily: "var(--font-mono)",
            fontSize: "0.78rem",
            letterSpacing: "0.18em",
            color: "var(--muted)",
            textAlign: "center",
            maxWidth: "30ch",
          }}
        >
          {title}
        </div>
      ) : null}
    </div>
  );
}

// Tiny utility — cheap useReducer-style "kick" hook used by Scene above
// so phase changes propagate to the DOM label without re-rendering the
// 3D Canvas every animation frame.
import { useReducer } from "react";
function useForceRender() {
  return useReducer((x: number) => x + 1, 0);
}

// Aliased export so the barrel re-exports cleanly as { WitnessPulse }.
export { WitnessPulse3D as WitnessPulse };
