"use client";

// ─── OverlayText — DOM SVG curved text layer ───────────────────────────
// Absolutely positioned over the WebGL Canvas. Preserves the textPath
// arc-following typography from the legacy OrbitalCover so the title,
// birth meta, wordmark, tagline all curve around the 3D sigil.
//
// Per design § 5.2 bloom-in sequence: starts at 1.6s, staggers 0.3s.

import { motion } from "motion/react";

interface OverlayTextProps {
  title: string;
  birthMeta: string;
  tagline: string;
  reducedMotion?: boolean;
}

export function OverlayText({ title, birthMeta, tagline, reducedMotion = false }: OverlayTextProps) {
  const initial = reducedMotion ? { opacity: 1 } : { opacity: 0 };
  const animate = { opacity: 1 };

  return (
    <svg
      viewBox="0 0 1000 1000"
      preserveAspectRatio="xMidYMid meet"
      xmlns="http://www.w3.org/2000/svg"
      aria-label="Composite cover curved text overlay"
      style={{
        position: "absolute",
        inset: 0,
        width: "min(100vw, 100vh)",
        height: "min(100vw, 100vh)",
        maxWidth: 1280,
        maxHeight: 1280,
        margin: "auto",
        pointerEvents: "none",
        zIndex: 3,
      }}
    >
      <defs>
        <path id="cover3d-arc-title" d="M 120 540 A 420 420 0 0 1 880 540" fill="none" />
        <path id="cover3d-arc-subtitle" d="M 130 480 A 410 410 0 0 0 870 480" fill="none" />
        <path id="cover3d-arc-wordmark" d="M 60 520 A 480 480 0 0 1 940 520" fill="none" />
        <path id="cover3d-arc-lineage" d="M 140 460 A 400 400 0 0 0 860 460" fill="none" />
        <filter id="cover3d-glow-soft" x="-100%" y="-100%" width="300%" height="300%">
          <feGaussianBlur in="SourceGraphic" stdDeviation="2.4" />
        </filter>
      </defs>

      {/* Wordmark — outer top arc */}
      <motion.text
        fill="var(--c-gold)"
        fontFamily="var(--font-mono)"
        fontSize={11}
        letterSpacing={9}
        opacity={0.85}
        initial={initial}
        animate={animate}
        transition={{ delay: reducedMotion ? 0 : 1.1, duration: 1.2 }}
      >
        <textPath href="#cover3d-arc-wordmark" startOffset="50%" textAnchor="middle">
          TRYAMBAKAM · NOESIS · INTEGRATED · READING
        </textPath>
      </motion.text>

      {/* Title — top arc, Panchang 800 */}
      <motion.text
        fill="var(--c-parchment)"
        fontFamily="var(--font-display)"
        fontWeight={800}
        fontSize={72}
        letterSpacing={3}
        filter="url(#cover3d-glow-soft)"
        initial={initial}
        animate={animate}
        transition={{ delay: reducedMotion ? 0 : 1.6, duration: 1.4 }}
      >
        <textPath href="#cover3d-arc-title" startOffset="50%" textAnchor="middle">
          {title}
        </textPath>
      </motion.text>

      {/* Birth meta — lower arc, SF Mono */}
      <motion.text
        fill="var(--c-emerald)"
        fontFamily="var(--font-mono)"
        fontSize={14}
        letterSpacing={4}
        initial={initial}
        animate={animate}
        transition={{ delay: reducedMotion ? 0 : 1.9, duration: 1.4 }}
      >
        <textPath href="#cover3d-arc-subtitle" startOffset="50%" textAnchor="middle">
          {birthMeta}
        </textPath>
      </motion.text>

      {/* Tagline — lineage arc */}
      <motion.text
        fill="var(--muted)"
        fontFamily="var(--font-display)"
        fontWeight={500}
        fontStyle="italic"
        fontSize={20}
        letterSpacing={1.6}
        initial={initial}
        animate={animate}
        transition={{ delay: reducedMotion ? 0 : 2.2, duration: 1.4 }}
      >
        <textPath href="#cover3d-arc-lineage" startOffset="50%" textAnchor="middle">
          {tagline}
        </textPath>
      </motion.text>

      {/* Stamp bottom */}
      <motion.text
        x={500}
        y={970}
        textAnchor="middle"
        fill="var(--c-gold)"
        fontFamily="var(--font-display)"
        fontWeight={700}
        fontSize={14}
        letterSpacing={6}
        opacity={0.9}
        initial={initial}
        animate={animate}
        transition={{ delay: reducedMotion ? 0 : 2.5, duration: 1.0 }}
      >
        ∴  NOESIS  ∴
      </motion.text>
    </svg>
  );
}
