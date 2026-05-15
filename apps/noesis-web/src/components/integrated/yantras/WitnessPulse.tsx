"use client";

// ─── WitnessPulse — concentric breathing-ring opener for each Part ──────
// Per integrated-reading-design-v2.md § 5.3.
//
// Visual reference: Branding/witnessOS-sw/breathnav-screen.png — concentric
// circles with INHALE / HOLD / EXHALE labels.
//
// 4:7:8 breath cycle (inhale 4s · hold 7s · exhale 8s) — circles scale
// 1.0 → 1.08 → 1.0. Pure motion/react, no Lottie dep.
//
// prefers-reduced-motion: rings stay at rest (scale 1.0), label still
// shown but the cardinal stage label freezes on "STILL".

import { motion, useReducedMotion } from "motion/react";
import { useEffect, useState } from "react";

export type WitnessDirection = "STABILIZE" | "HEAL" | "CREATE" | "MUTATE";

interface WitnessPulseProps {
  direction: WitnessDirection;
  title?: string;
}

// Breath phases — sum 19s.
const PHASE_DURATIONS = { inhale: 4, hold: 7, exhale: 8 } as const;
const CYCLE_SECONDS =
  PHASE_DURATIONS.inhale + PHASE_DURATIONS.hold + PHASE_DURATIONS.exhale;

const PHASE_LABEL: Record<"inhale" | "hold" | "exhale", string> = {
  inhale: "INHALE · 4",
  hold: "HOLD · 7",
  exhale: "EXHALE · 8",
};

// Map cardinal → tonal accent for the central glow.
const DIRECTION_ACCENT: Record<WitnessDirection, string> = {
  STABILIZE: "var(--c-violet)",
  HEAL: "var(--c-indigo)",
  CREATE: "var(--c-gold)",
  MUTATE: "var(--c-emerald)",
};

export function WitnessPulse({ direction, title }: WitnessPulseProps) {
  const reduce = useReducedMotion();
  const accent = DIRECTION_ACCENT[direction];

  // Local breath-phase clock for the textual stage label.
  const [phase, setPhase] = useState<"inhale" | "hold" | "exhale">("inhale");

  useEffect(() => {
    if (reduce) return;
    let raf = 0;
    const start = performance.now();
    const tick = (now: number) => {
      const t = ((now - start) / 1000) % CYCLE_SECONDS;
      const next: typeof phase =
        t < PHASE_DURATIONS.inhale
          ? "inhale"
          : t < PHASE_DURATIONS.inhale + PHASE_DURATIONS.hold
          ? "hold"
          : "exhale";
      setPhase((prev) => (prev === next ? prev : next));
      raf = requestAnimationFrame(tick);
    };
    raf = requestAnimationFrame(tick);
    return () => cancelAnimationFrame(raf);
  }, [reduce]);

  // Ring keyframes — three rings breathe with slight offset to imply depth.
  // 4:7:8 means: scale 1.0 (start of inhale) → 1.08 (end of inhale, held
  // through hold) → 1.0 (end of exhale).
  const breathKeyframes = reduce
    ? { scale: 1 }
    : {
        scale: [1, 1.08, 1.08, 1],
      };
  const breathTimes = [
    0,
    PHASE_DURATIONS.inhale / CYCLE_SECONDS,
    (PHASE_DURATIONS.inhale + PHASE_DURATIONS.hold) / CYCLE_SECONDS,
    1,
  ];
  const breathTransition = reduce
    ? undefined
    : {
        duration: CYCLE_SECONDS,
        times: breathTimes,
        ease: "easeInOut" as const,
        repeat: Infinity,
      };

  const stageLabel = reduce ? "STILL" : PHASE_LABEL[phase];

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
      <motion.svg
        viewBox="0 0 480 480"
        width="100%"
        height="auto"
        style={{
          display: "block",
          maxWidth: 480,
          aspectRatio: "1 / 1",
          width: "min(100%, clamp(20rem, 50vw, 30rem))",
        }}
        initial={{ opacity: 0 }}
        whileInView={{ opacity: 1 }}
        viewport={{ once: true, margin: "-10% 0px" }}
        transition={{ duration: 1.2, ease: [0.16, 1, 0.3, 1] }}
        aria-hidden="true"
      >
        <defs>
          <radialGradient id={`wp-grad-${direction}`} cx="50%" cy="50%" r="50%">
            <stop offset="0%" stopColor={accent} stopOpacity="0.55" />
            <stop offset="55%" stopColor="var(--c-indigo)" stopOpacity="0.2" />
            <stop offset="100%" stopColor="var(--c-violet)" stopOpacity="0" />
          </radialGradient>
          <filter id={`wp-glow-${direction}`} x="-50%" y="-50%" width="200%" height="200%">
            <feGaussianBlur stdDeviation="6" result="blur" />
            <feMerge>
              <feMergeNode in="blur" />
              <feMergeNode in="SourceGraphic" />
            </feMerge>
          </filter>
        </defs>

        {/* Outer atmosphere — breathes most softly */}
        <motion.circle
          cx="240"
          cy="240"
          r="210"
          fill={`url(#wp-grad-${direction})`}
          style={{ originX: "240px", originY: "240px" }}
          animate={breathKeyframes}
          transition={breathTransition}
        />

        {/* Three concentric rings — each breathes with a tiny phase offset
            via different delays so the depth reads as organic. */}
        {[180, 140, 100].map((r, i) => (
          <motion.circle
            key={r}
            cx="240"
            cy="240"
            r={r}
            fill="none"
            stroke={i === 1 ? accent : "var(--c-indigo)"}
            strokeWidth={i === 1 ? 1.6 : 1.1}
            strokeOpacity={0.35 + i * 0.12}
            style={{ originX: "240px", originY: "240px" }}
            animate={breathKeyframes}
            transition={
              breathTransition
                ? { ...breathTransition, delay: i * 0.15 }
                : undefined
            }
          />
        ))}

        {/* Core seed — solid, breathes with the rings */}
        <motion.circle
          cx="240"
          cy="240"
          r="48"
          fill={accent}
          fillOpacity={0.18}
          stroke={accent}
          strokeWidth={1.2}
          filter={`url(#wp-glow-${direction})`}
          style={{ originX: "240px", originY: "240px" }}
          animate={breathKeyframes}
          transition={breathTransition}
        />

        {/* Cardinal direction text — inside the SVG, centered under the
            core seed. Kept inside the SVG so it scales fluidly. */}
        <text
          x="240"
          y="310"
          textAnchor="middle"
          style={{
            fontFamily: "var(--font-display)",
            fontWeight: 700,
            letterSpacing: "0.35em",
            fontSize: 14,
            fill: "var(--c-parchment)",
            opacity: 0.86,
          }}
        >
          {direction}
        </text>
      </motion.svg>

      {/* Stage label below the SVG — drives the breath cue text. */}
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
