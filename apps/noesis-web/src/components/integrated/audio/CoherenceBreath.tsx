"use client";

// ─── CoherenceBreath ───────────────────────────────────────────────────
// Viewport-wide 4:7:8 breath pulse. A fixed, pointer-events:none gold
// hairline overlay whose opacity breathes 0 → 0.04 → 0 over 19 s
// (4s inhale + 7s hold + 8s exhale).
//
// Implementation: motion.div with keyframe `animate` driven by a single
// transition. mix-blend-mode: screen so the gold reads as a near-
// imperceptible warmth lift on dark fields and a faint glow on light.
//
// Respects prefers-reduced-motion: rendered at flat opacity 0 (invisible).
// Per design § 5.11 — "makes the whole page feel like it's breathing".

import { motion, useReducedMotion } from "motion/react";

const TIMES = [0, 4 / 19, 11 / 19, 1];
const OPACITIES = [0, 0.04, 0.04, 0];
// 4s inhale → peak; 7s hold at peak; 8s exhale → 0.

export function CoherenceBreath() {
  const reduced = useReducedMotion();

  if (reduced) {
    // Disable animation entirely; render nothing visible.
    return (
      <div
        aria-hidden="true"
        style={{
          position: "fixed",
          inset: 0,
          pointerEvents: "none",
          zIndex: 25,
          opacity: 0,
        }}
      />
    );
  }

  return (
    <motion.div
      aria-hidden="true"
      style={{
        position: "fixed",
        inset: 0,
        pointerEvents: "none",
        zIndex: 25,
        mixBlendMode: "screen",
        // Soft radial — gold concentrated near center, fading outward, so
        // the breath reads as a centered presence rather than a flat tint.
        background:
          "radial-gradient(ellipse at center, rgba(197, 160, 23, 1) 0%, rgba(197, 160, 23, 0.55) 35%, rgba(197, 160, 23, 0) 75%)",
        willChange: "opacity",
      }}
      initial={{ opacity: 0 }}
      animate={{ opacity: OPACITIES }}
      transition={{
        duration: 19,
        times: TIMES,
        ease: ["easeOut", "linear", "easeIn", "linear"],
        repeat: Infinity,
        repeatType: "loop",
      }}
    />
  );
}
