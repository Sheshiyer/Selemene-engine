"use client";

// ─── ChapterTransition — brief full-bleed scene between Parts ──────────
// Per integrated-reading-design-v2.md § 5.12.
//
// Triggered as the reader scrolls into the gap between consecutive Parts.
// Composition:
//   • 100vh tall, full-bleed
//   • Centered: massive Roman numeral of the NEXT Part (Panchang 800,
//     12rem) — this is the "you're about to enter Part N" moment
//   • Horizontal Sacred Gold sweep bar animates left → right over 1.6s
//   • Sub-text below: next chapter's cardinal direction in SF Mono
//   • After the sweep finishes the whole thing fades out so the next
//     ChapterScene takes over
//
// Re-triggers each time it scrolls into view (`once: false`) so the
// transition replays if the reader scrolls back up.

import { motion, useInView, useReducedMotion } from "motion/react";
import { useRef } from "react";
import type { ChapterDirection } from "./ChapterScene";
import { SacredScene } from "../sacred-scene/SacredScene";

interface ChapterTransitionProps {
  fromPart: number;
  toPart: number;
  toRomanNumeral: string;
  toTitle: string;
  toDirection: ChapterDirection;
}

const DIRECTION_LABEL: Record<ChapterDirection, string> = {
  STABILIZE: "STABILIZE",
  HEAL: "HEAL",
  CREATE: "CREATE",
  MUTATE: "MUTATE",
};

const DIRECTION_ACCENT: Record<ChapterDirection, string> = {
  STABILIZE: "var(--c-violet)",
  HEAL: "var(--c-indigo)",
  CREATE: "var(--c-emerald)",
  MUTATE: "var(--c-gold)",
};

export function ChapterTransition({
  fromPart,
  toPart,
  toRomanNumeral,
  toTitle,
  toDirection,
}: ChapterTransitionProps) {
  const ref = useRef<HTMLDivElement>(null);
  // Mid-scene detection — we want the sweep to fire when the transition
  // is fully on screen, not when the edge first touches.
  const inView = useInView(ref, {
    margin: "-30% 0% -30% 0%",
    once: false,
  });
  const reduce = useReducedMotion();
  const accent = DIRECTION_ACCENT[toDirection];

  return (
    <section
      ref={ref}
      aria-hidden="true"
      data-chapter-transition-from={fromPart}
      data-chapter-transition-to={toPart}
      style={{
        position: "relative",
        height: "100vh",
        minHeight: "100vh",
        width: "100%",
        display: "flex",
        flexDirection: "column",
        alignItems: "center",
        justifyContent: "center",
        overflow: "hidden",
        zIndex: 2,
      }}
    >
      {/* SacredScene full-viewport backdrop — kind="transition" pulses in
          when the transition enters view. Sits at zIndex 0 with pointer
          events disabled so the existing overlay text stays interactive
          (transitions themselves are aria-hidden, but DOM order matters).
          Intensity ramps from 0 to 1.2 over the inView window. */}
      <motion.div
        initial={{ opacity: 0 }}
        animate={inView && !reduce ? { opacity: 1 } : { opacity: 0 }}
        transition={{ duration: 1.2, ease: [0.2, 0.7, 0.2, 1] }}
        style={{
          position: "absolute",
          inset: 0,
          zIndex: 0,
          pointerEvents: "none",
        }}
        aria-hidden="true"
      >
        <SacredScene
          kind="transition"
          intensity={inView ? 1.2 : 0}
          height="100%"
        />
      </motion.div>

      {/* Backdrop softener: very subtle Void Black wash to separate from
          adjacent chapters' atmospheres. Sits between the SacredScene and
          the overlay text. */}
      <div
        style={{
          position: "absolute",
          inset: 0,
          zIndex: 1,
          background:
            "radial-gradient(ellipse at center, rgba(7,11,29,0.45) 0%, rgba(7,11,29,0.78) 70%, rgba(7,11,29,0.92) 100%)",
          pointerEvents: "none",
        }}
      />

      {/* Gold sweep bar — left → right, 1.6s, breath-paced. */}
      <motion.div
        initial={{ scaleX: 0, opacity: 0, transformOrigin: "left center" }}
        animate={
          inView && !reduce
            ? { scaleX: 1, opacity: 1 }
            : { scaleX: 0, opacity: 0 }
        }
        transition={{ duration: 1.6, ease: [0.2, 0.7, 0.2, 1] }}
        style={{
          position: "absolute",
          top: "50%",
          left: 0,
          right: 0,
          height: "2px",
          background:
            "linear-gradient(90deg, transparent 0%, var(--c-gold) 50%, transparent 100%)",
          boxShadow: "var(--glow-gold)",
          transformOrigin: "left center",
          zIndex: 2,
        }}
      />

      {/* Massive Roman numeral of the next Part. */}
      <motion.div
        initial={{ opacity: 0, scale: 0.86 }}
        animate={
          inView
            ? { opacity: 1, scale: 1 }
            : { opacity: 0, scale: 0.86 }
        }
        transition={{
          duration: 1.2,
          delay: reduce ? 0 : 0.4,
          ease: [0.2, 0.7, 0.2, 1],
        }}
        style={{
          position: "relative",
          zIndex: 3,
          fontFamily: "var(--font-display)",
          fontWeight: 800,
          fontSize: "clamp(6rem, 14vw, 12rem)",
          lineHeight: 1,
          letterSpacing: "-0.03em",
          color: "var(--c-parchment)",
          textShadow: "0 0 40px rgba(197, 160, 23, 0.45)",
        }}
      >
        {toRomanNumeral}
      </motion.div>

      {/* Sub-text: cardinal direction + chapter title. */}
      <motion.div
        initial={{ opacity: 0, y: 12 }}
        animate={
          inView ? { opacity: 1, y: 0 } : { opacity: 0, y: 12 }
        }
        transition={{
          duration: 0.9,
          delay: reduce ? 0 : 1.0,
          ease: [0.2, 0.7, 0.2, 1],
        }}
        style={{
          position: "relative",
          zIndex: 3,
          marginTop: "1.5rem",
          display: "flex",
          flexDirection: "column",
          alignItems: "center",
          gap: "0.85rem",
          textAlign: "center",
        }}
      >
        <div
          style={{
            fontFamily: "var(--font-mono)",
            fontSize: "0.95rem",
            letterSpacing: "0.55em",
            textTransform: "uppercase",
            color: accent,
          }}
        >
          {DIRECTION_LABEL[toDirection]}
        </div>
        <div
          style={{
            fontFamily: "var(--font-display)",
            fontWeight: 500,
            fontStyle: "italic",
            fontSize: "clamp(1rem, 0.9rem + 0.5vw, 1.4rem)",
            letterSpacing: "0.02em",
            color: "var(--text-2)",
            maxWidth: "32ch",
          }}
        >
          {toTitle}
        </div>
        <div
          style={{
            marginTop: "0.4rem",
            fontFamily: "var(--font-mono)",
            fontSize: "0.7rem",
            letterSpacing: "0.32em",
            color: "var(--muted)",
          }}
        >
          {fromPart.toString().padStart(2, "0")} → {toPart.toString().padStart(2, "0")}
        </div>
      </motion.div>
    </section>
  );
}
