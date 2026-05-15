"use client";

// ─── CoverScene — clean vertical-stack hero ────────────────────────────
// Rebuilt from the failing orbital-textPath approach: typography no
// longer fights the 3D sigil. Composition (top → bottom):
//
//   1. Top mono band — TRYAMBAKAM · NOESIS · INTEGRATED READING
//   2. Massive centered display title (Panchang 800, clamp 3rem→9rem)
//   3. 3D sigil scene (R3F + Bloom intensity 1.8, radius 0.95)
//   4. Horizontal subjects row separated by dots
//   5. Bottom mono meta band (mode · register · word count)
//   6. Centered tagline italic
//   7. Animated scroll chevron
//
// All animations are entry-only (fade-up on mount, subject stagger,
// gentle chevron bounce). prefers-reduced-motion snaps to final state
// and disables every motion.
//
// SSR / no-WebGL: legacy OrbitalCover render preserved so hydration
// matches and there's never a blank flash.
//
// Per design § 5.2 + Chapter 0 — Three Laws compliant.

import { useEffect, useState } from "react";
import { motion } from "motion/react";

import { OrbitalCover } from "../OrbitalCover";
import { SacredScene } from "../sacred-scene/SacredScene";

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

  // Animation defaults — snap to final when reduced motion is requested.
  const fadeUp = (delay: number, duration = 1.2) =>
    reducedMotion
      ? { initial: { opacity: 1, y: 0 }, animate: { opacity: 1, y: 0 }, transition: { duration: 0 } }
      : {
          initial: { opacity: 0, y: 18 },
          animate: { opacity: 1, y: 0 },
          transition: { delay, duration, ease: [0.16, 1, 0.3, 1] as [number, number, number, number] },
        };

  return (
    <section
      style={{
        position: "relative",
        width: "100vw",
        minHeight: "100vh",
        overflow: "hidden",
        display: "flex",
        flexDirection: "column",
        alignItems: "center",
        justifyContent: "flex-start",
        padding: "clamp(2.5rem, 5vh, 4.5rem) clamp(1.25rem, 4vw, 3rem)",
        gap: "clamp(1.25rem, 3vh, 2.5rem)",
        background:
          "radial-gradient(ellipse at 50% 25%, rgba(11,80,251,0.10) 0%, transparent 55%), radial-gradient(ellipse at 50% 80%, rgba(45,0,80,0.20) 0%, transparent 60%), var(--c-void)",
        zIndex: 2,
      }}
    >
      {/* ── 1. Top mono band ──────────────────────────────────────── */}
      <motion.header
        {...fadeUp(0.0)}
        style={{
          fontFamily: "var(--font-mono)",
          fontSize: "clamp(0.625rem, 0.85vw, 0.8125rem)",
          letterSpacing: "0.4em",
          color: "var(--c-gold)",
          opacity: 0.85,
          textTransform: "uppercase",
          textAlign: "center",
        }}
      >
        TRYAMBAKAM · NOESIS · INTEGRATED READING
      </motion.header>

      {/* ── 2. Massive display title ──────────────────────────────── */}
      <motion.h1
        {...fadeUp(0.2, 1.4)}
        style={{
          margin: 0,
          fontFamily: "var(--font-display)",
          fontWeight: 800,
          fontSize: "clamp(3rem, 6vw, 9rem)",
          lineHeight: 1.02,
          letterSpacing: "0.04em",
          textAlign: "center",
          color: "var(--c-parchment)",
          textShadow:
            "0 0 24px rgba(197,160,23,0.35), 0 0 64px rgba(11,80,251,0.18)",
        }}
      >
        {title}
      </motion.h1>

      {/* ── 3. 3D sigil stage — driven by the SacredScene GLSL primitive
              (procedural fbm noise, breathing icosahedron, particle aura,
              wave ribbon, scene fog — all from one shader codebase) ── */}
      <motion.div
        initial={reducedMotion ? { opacity: 1 } : { opacity: 0 }}
        animate={{ opacity: 1 }}
        transition={{ delay: reducedMotion ? 0 : 0.6, duration: reducedMotion ? 0 : 2.0 }}
        style={{
          position: "relative",
          width: "min(720px, 78vw, 68vh)",
          aspectRatio: "1 / 1",
          flexShrink: 0,
        }}
      >
        <SacredScene kind="cover" intensity={1} height="100%" />
      </motion.div>

      {/* ── 4. Subjects row ───────────────────────────────────────── */}
      <div
        style={{
          display: "flex",
          flexWrap: "wrap",
          justifyContent: "center",
          alignItems: "center",
          gap: "clamp(0.75rem, 1.5vw, 1.25rem)",
          fontFamily: "var(--font-display)",
          fontWeight: 500,
          fontSize: "clamp(0.9375rem, 1.15vw, 1.125rem)",
          letterSpacing: "0.18em",
          color: "var(--c-parchment)",
          textTransform: "uppercase",
        }}
      >
        {subjects.map((name, i) => (
          <motion.span
            key={name}
            {...fadeUp(1.2 + i * 0.15, 1.0)}
            style={{ display: "inline-flex", alignItems: "center", gap: "clamp(0.75rem, 1.5vw, 1.25rem)" }}
          >
            <span>{name}</span>
            {i < subjects.length - 1 && (
              <span aria-hidden="true" style={{ color: "var(--c-gold)", opacity: 0.7 }}>
                ·
              </span>
            )}
          </motion.span>
        ))}
      </div>

      {/* ── 5. Meta band ──────────────────────────────────────────── */}
      <motion.div
        {...fadeUp(1.2 + subjects.length * 0.15 + 0.1, 1.0)}
        style={{
          fontFamily: "var(--font-mono)",
          fontSize: "clamp(0.6875rem, 0.9vw, 0.875rem)",
          letterSpacing: "0.32em",
          color: "var(--c-emerald)",
          opacity: 0.9,
          textAlign: "center",
          textTransform: "uppercase",
        }}
      >
        {birthMeta}
      </motion.div>

      {/* ── 6. Tagline ────────────────────────────────────────────── */}
      <motion.p
        {...fadeUp(1.2 + subjects.length * 0.15 + 0.25, 1.0)}
        style={{
          margin: 0,
          maxWidth: "min(640px, 80vw)",
          fontFamily: "var(--font-display)",
          fontWeight: 500,
          fontStyle: "italic",
          fontSize: "clamp(0.9375rem, 1.1vw, 1.125rem)",
          lineHeight: 1.5,
          letterSpacing: "0.04em",
          color: "var(--muted)",
          textAlign: "center",
        }}
      >
        {tagline}
      </motion.p>

      {/* ── 7. Scroll cue ─────────────────────────────────────────── */}
      <motion.div
        initial={reducedMotion ? { opacity: 1 } : { opacity: 0 }}
        animate={{ opacity: 1 }}
        transition={{ delay: reducedMotion ? 0 : 1.2 + subjects.length * 0.15 + 0.5, duration: 1.0 }}
        style={{
          display: "flex",
          flexDirection: "column",
          alignItems: "center",
          gap: "0.5rem",
          marginTop: "auto",
          paddingTop: "clamp(1rem, 2vh, 2rem)",
          fontFamily: "var(--font-mono)",
          fontSize: "0.6875rem",
          letterSpacing: "0.4em",
          color: "var(--c-gold)",
          opacity: 0.7,
          textTransform: "uppercase",
        }}
      >
        <motion.span
          aria-hidden="true"
          animate={reducedMotion ? undefined : { y: [0, 8, 0] }}
          transition={
            reducedMotion
              ? undefined
              : { duration: 1.4, repeat: Infinity, ease: "easeInOut" }
          }
          style={{ fontSize: "1rem", lineHeight: 1 }}
        >
          ↓
        </motion.span>
        <span>Scroll to begin</span>
      </motion.div>
    </section>
  );
}
