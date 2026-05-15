"use client";

// ─── LaArcFade — gradient breath-out closer between Parts ──────────────
// Per design MD § 3.9. Vertical gradient (Sacred Gold → Witness Violet →
// Void Black). animation-timeline: view() in browsers that support it;
// motion's whileInView for the rest.

import { motion } from "motion/react";

export function LaArcFade() {
  return (
    <motion.div
      aria-hidden="true"
      initial={{ opacity: 0.25, scaleY: 0.4 }}
      whileInView={{ opacity: [0.25, 0.7, 0.15], scaleY: [0.4, 1, 1] }}
      viewport={{ margin: "0% 0% -40% 0%", once: false }}
      transition={{ duration: 1.4, ease: [0.2, 0.7, 0.2, 1] }}
      style={{
        position: "relative",
        width: "100%",
        height: "clamp(72px, 12vh, 168px)",
        margin: "clamp(2rem, 4vw, 4rem) 0",
        background:
          "linear-gradient(180deg, rgba(197,160,23,0.55) 0%, rgba(45,0,80,0.65) 45%, rgba(7,11,29,0.95) 100%)",
        borderRadius: 1,
        transformOrigin: "top",
      }}
    />
  );
}
