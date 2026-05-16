"use client";

// ─── ChapterScene — full-bleed scene wrapper per Part ──────────────────
// Per integrated-reading-design-v2.md § 4 (story arc, "Each Part is its
// own scene") and DESIGN.md § 5 (compass directions).
//
// Each Part is now a real "chapter scene":
//   • Full viewport min-height — chapter has breathing room
//   • Direction-tuned Kha-Ba-La gradient atmosphere (STABILIZE / HEAL /
//     CREATE / MUTATE) painted as a low-opacity overlay
//   • Subtle floating-particle accent specific to the chapter color
//   • Internal vertical stack: chapter-marker (top), content (center),
//     chapter-closing (bottom)
//   • Reading prose inside still constrains to clamp(18rem, 72vw, 80rem)
//     centered column; composition children opt in/out of that constraint
//
// Direction → gradient (per design v2 § 4 + DESIGN.md § 5):
//   STABILIZE : void → violet (Kha onset)
//   HEAL      : violet → indigo (Kha full)
//   CREATE    : emerald → gold (Ba arc)
//   MUTATE    : gold → void (La arc)

import { motion, useReducedMotion } from "motion/react";
import { useMemo } from "react";
import type { ReactNode } from "react";

export type ChapterDirection = "STABILIZE" | "HEAL" | "CREATE" | "MUTATE";

interface ChapterSceneProps {
  partNum: number;
  romanNumeral: string;
  title: string;
  direction: ChapterDirection;
  words?: number;
  xrefs?: number;
  children: ReactNode;
}

// Direction-tuned atmosphere gradients. Painted at low opacity over the
// global Void Black background so they tint the scene, not flood it.
const DIRECTION_GRADIENT: Record<ChapterDirection, string> = {
  STABILIZE:
    "linear-gradient(135deg, var(--c-void) 0%, var(--c-violet) 100%)",
  HEAL: "linear-gradient(135deg, var(--c-violet) 0%, var(--c-indigo) 100%)",
  CREATE: "linear-gradient(90deg, var(--c-emerald) 0%, var(--c-gold) 100%)",
  MUTATE: "linear-gradient(135deg, var(--c-gold) 0%, var(--c-void) 100%)",
};

// Direction-tuned accent color for particles, marker line, etc.
const DIRECTION_ACCENT: Record<ChapterDirection, string> = {
  STABILIZE: "var(--c-violet)",
  HEAL: "var(--c-indigo)",
  CREATE: "var(--c-emerald)",
  MUTATE: "var(--c-gold)",
};

const DIRECTION_LABEL: Record<ChapterDirection, string> = {
  STABILIZE: "STABILIZE · NORTH · GROUND",
  HEAL: "HEAL · EAST · RESTORE",
  CREATE: "CREATE · SOUTH · ACTIVATE",
  MUTATE: "MUTATE · WEST · TRANSFORM",
};

// ─── Atmosphere accent: minimal drifting motes tuned to chapter color ──
function AtmosphereAccent({ direction }: { direction: ChapterDirection }) {
  const reduce = useReducedMotion();
  const accent = DIRECTION_ACCENT[direction];

  // Deterministic mote positions so SSR + client match.
  const motes = useMemo(() => {
    // Seeded by direction so each chapter has a stable pattern.
    const seed =
      { STABILIZE: 1, HEAL: 2, CREATE: 3, MUTATE: 4 }[direction] ?? 1;
    const out: Array<{ x: number; y: number; r: number; d: number }> = [];
    for (let i = 0; i < 18; i++) {
      const v = Math.sin((i + 1) * seed * 1.61803);
      const u = Math.cos((i + 1) * seed * 2.39996);
      out.push({
        x: ((v + 1) / 2) * 100,
        y: ((u + 1) / 2) * 100,
        r: 1.2 + ((i * seed) % 5) * 0.4,
        d: 6 + ((i * 7) % 9),
      });
    }
    return out;
  }, [direction]);

  if (reduce) return null;

  return (
    <div
      aria-hidden="true"
      style={{
        position: "absolute",
        inset: 0,
        pointerEvents: "none",
        overflow: "hidden",
        zIndex: 0,
      }}
    >
      {motes.map((m, i) => (
        <motion.span
          key={i}
          initial={{ opacity: 0 }}
          animate={{
            opacity: [0, 0.55, 0],
            y: [0, -18, 0],
          }}
          transition={{
            duration: m.d,
            repeat: Infinity,
            delay: (i * 0.37) % m.d,
            ease: "easeInOut",
          }}
          style={{
            position: "absolute",
            left: `${m.x}%`,
            top: `${m.y}%`,
            width: `${m.r}px`,
            height: `${m.r}px`,
            borderRadius: "50%",
            background: accent,
            boxShadow: `0 0 ${m.r * 4}px ${accent}`,
            opacity: 0,
          }}
        />
      ))}
    </div>
  );
}

export function ChapterScene({
  partNum,
  romanNumeral,
  title,
  direction,
  words,
  xrefs,
  children,
}: ChapterSceneProps) {
  const gradient = DIRECTION_GRADIENT[direction];
  const accent = DIRECTION_ACCENT[direction];
  const directionLabel = DIRECTION_LABEL[direction];

  return (
    <section
      id={`part-${partNum}`}
      data-chapter-part={partNum}
      data-chapter-direction={direction}
      style={{
        position: "relative",
        // Let content size the chapter — no forced 100vh. Prose +
        // bento + atmosphere stack naturally; verses stay reachable.
        width: "100%",
        display: "flex",
        flexDirection: "column",
        alignItems: "stretch",
        justifyContent: "flex-start",
        paddingTop: "clamp(2rem, 5vh, 4rem)",
        paddingBottom: "clamp(2rem, 5vh, 4rem)",
        overflow: "hidden",
        scrollMarginTop: "0px",
        zIndex: 2,
      }}
    >
      {/* Direction-tuned atmosphere gradient, 8% opacity overlay. */}
      <div
        aria-hidden="true"
        style={{
          position: "absolute",
          inset: 0,
          background: gradient,
          opacity: 0.08,
          pointerEvents: "none",
          zIndex: 0,
        }}
      />

      {/* Subtle drifting motes accent. */}
      <AtmosphereAccent direction={direction} />

      {/* Top chapter marker. */}
      <motion.header
        initial={{ opacity: 0, y: 24 }}
        whileInView={{ opacity: 1, y: 0 }}
        viewport={{ margin: "0% 0% -20% 0%", once: true }}
        transition={{ duration: 0.9, ease: [0.2, 0.7, 0.2, 1] }}
        style={{
          position: "relative",
          zIndex: 1,
          width: "clamp(18rem, 72vw, 80rem)",
          margin: "0 auto",
          padding: "0 clamp(1rem, 2.4vw, 2.5rem)",
        }}
      >
        <div
          style={{
            display: "flex",
            alignItems: "center",
            gap: "1rem",
            marginBottom: "1.25rem",
          }}
        >
          <div
            aria-hidden="true"
            style={{
              width: "clamp(2.5rem, 6vw, 4rem)",
              height: "1px",
              background: accent,
              opacity: 0.7,
              boxShadow: `0 0 8px ${accent}`,
            }}
          />
          <div
            style={{
              fontFamily: "var(--font-mono)",
              fontSize: "clamp(0.72rem, 0.65rem + 0.15vw, 0.85rem)",
              letterSpacing: "0.45em",
              textTransform: "uppercase",
              color: "var(--c-gold)",
            }}
          >
            Part {romanNumeral}
          </div>
          <div
            style={{
              fontFamily: "var(--font-mono)",
              fontSize: "0.7rem",
              letterSpacing: "0.32em",
              color: accent,
              opacity: 0.85,
            }}
          >
            {directionLabel}
          </div>
        </div>
        <h1
          style={{
            fontFamily: "var(--font-display)",
            fontWeight: 800,
            fontSize: "clamp(2rem, 1.4rem + 2.4vw, 4.5rem)",
            lineHeight: 1.02,
            letterSpacing: "-0.022em",
            color: "var(--c-parchment)",
            margin: 0,
          }}
        >
          {title}
        </h1>
        {(words !== undefined || xrefs !== undefined) && (
          <div
            style={{
              marginTop: "0.85rem",
              fontFamily: "var(--font-mono)",
              fontSize: "0.85rem",
              color: "var(--c-emerald)",
              letterSpacing: "0.12em",
            }}
          >
            {words !== undefined ? `${words.toLocaleString()} words` : null}
            {words !== undefined && xrefs !== undefined ? " · " : null}
            {xrefs !== undefined ? `${xrefs} cross-references` : null}
          </div>
        )}
      </motion.header>

      {/* Center content — composition slot. Children control their own
          internal layout; reading prose constrains itself via VerseFlow. */}
      <div
        style={{
          position: "relative",
          zIndex: 1,
          flex: 1,
          display: "flex",
          flexDirection: "column",
          width: "100%",
          margin: "clamp(2rem, 4vh, 4rem) 0 0",
          fontSize: "clamp(1rem, 0.85rem + 0.45vw, 1.22rem)",
          lineHeight: 1.65,
          color: "var(--text)",
        }}
      >
        <div
          style={{
            width: "clamp(18rem, 72vw, 80rem)",
            margin: "0 auto",
            padding: "0 clamp(1rem, 2.4vw, 2.5rem)",
          }}
        >
          {children}
        </div>
      </div>

      {/* Bottom chapter-closing rule. */}
      <div
        aria-hidden="true"
        style={{
          position: "relative",
          zIndex: 1,
          width: "clamp(18rem, 72vw, 80rem)",
          margin: "clamp(3rem, 5vh, 5rem) auto 0",
          padding: "0 clamp(1rem, 2.4vw, 2.5rem)",
          display: "flex",
          alignItems: "center",
          gap: "1rem",
        }}
      >
        <div
          style={{
            flex: 1,
            height: "1px",
            background: `linear-gradient(90deg, transparent 0%, ${accent} 50%, transparent 100%)`,
            opacity: 0.45,
          }}
        />
        <div
          style={{
            fontFamily: "var(--font-mono)",
            fontSize: "0.7rem",
            letterSpacing: "0.32em",
            color: "var(--muted)",
          }}
        >
          ◆
        </div>
        <div
          style={{
            flex: 1,
            height: "1px",
            background: `linear-gradient(90deg, transparent 0%, ${accent} 50%, transparent 100%)`,
            opacity: 0.45,
          }}
        />
      </div>
    </section>
  );
}
