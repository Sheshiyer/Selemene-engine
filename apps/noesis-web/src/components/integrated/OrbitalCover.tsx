"use client";

// ─── OrbitalCover — full-WOW cover hero ─────────────────────────────────
// Per design MD § 3.10. SVG with curved text on arcs around a centered
// topology sigil. Sequenced choreography via motion/react: rings draw,
// sigil blooms, constellation points (subject names) fade in at compass
// positions, curved text fades in last.
//
// Bioluminescent feel via feGaussianBlur filters + Kha Arc atmosphere.

import { motion } from "motion/react";

interface OrbitalCoverProps {
  title: string;
  birthMeta: string;
  subjects: string[];
  topologySvg: string;
  tagline?: string;
}

export function OrbitalCover({
  title,
  birthMeta,
  subjects,
  topologySvg,
  tagline = "Self-Consciousness as Technology · Body as Medium · Breath as Interface",
}: OrbitalCoverProps) {
  const N = subjects.length;
  const subjectLabels = subjects.map((name, i) => {
    const angle = (-Math.PI / 2) + (i * (2 * Math.PI / Math.max(N, 1)));
    const labelRadius = 560;
    const x = 500 + labelRadius * Math.cos(angle);
    const y = 500 + labelRadius * Math.sin(angle);
    const anchor =
      Math.abs(Math.cos(angle)) < 0.3
        ? "middle"
        : Math.cos(angle) > 0
        ? "start"
        : "end";
    const dotRadius = 525;
    const dx = 500 + dotRadius * Math.cos(angle);
    const dy = 500 + dotRadius * Math.sin(angle);
    return (
      <motion.g
        key={name}
        initial={{ opacity: 0, y: 8 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ delay: 1.2 + i * 0.18, duration: 0.9, ease: [0.16, 1, 0.3, 1] }}
      >
        <motion.circle
          cx={dx}
          cy={dy}
          r={3.5}
          fill="var(--c-gold)"
          opacity={0.85}
          filter="url(#cover-glow-soft)"
          animate={{ r: [3.5, 4.4, 3.5], opacity: [0.7, 1, 0.7] }}
          transition={{ duration: 4, repeat: Infinity, ease: "easeInOut" }}
        />
        <text
          x={x}
          y={y}
          textAnchor={anchor}
          dominantBaseline="middle"
          fill="var(--c-parchment)"
          fontFamily="var(--font-display)"
          fontWeight={500}
          fontSize={20}
          letterSpacing={2.4}
        >
          {name.toUpperCase()}
        </text>
      </motion.g>
    );
  });

  return (
    <section
      style={{
        position: "relative",
        width: "100vw",
        height: "100vh",
        minHeight: "100vh",
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        background:
          "radial-gradient(ellipse at 50% 30%, rgba(11,80,251,0.10) 0%, transparent 55%), radial-gradient(ellipse at 50% 70%, rgba(45,0,80,0.18) 0%, transparent 60%), var(--c-void)",
        overflow: "hidden",
        zIndex: 2,
      }}
    >
      <motion.svg
        viewBox="0 0 1000 1000"
        preserveAspectRatio="xMidYMid meet"
        xmlns="http://www.w3.org/2000/svg"
        aria-label="Composite cover sigil"
        style={{
          width: "min(100vw, 100vh)",
          height: "min(100vw, 100vh)",
          maxWidth: 1280,
          maxHeight: 1280,
        }}
        initial={{ opacity: 0, scale: 0.94, filter: "blur(8px)" }}
        animate={{ opacity: 1, scale: 1, filter: "blur(0px)" }}
        transition={{ duration: 2.0, ease: [0.2, 0.7, 0.2, 1] }}
      >
        <defs>
          <path id="cover-arc-title" d="M 120 540 A 420 420 0 0 1 880 540" fill="none" />
          <path id="cover-arc-subtitle" d="M 130 480 A 410 410 0 0 0 870 480" fill="none" />
          <path id="cover-arc-wordmark" d="M 60 520 A 480 480 0 0 1 940 520" fill="none" />
          <path id="cover-arc-lineage" d="M 140 460 A 400 400 0 0 0 860 460" fill="none" />
          <filter id="cover-glow-strong" x="-100%" y="-100%" width="300%" height="300%">
            <feGaussianBlur in="SourceGraphic" stdDeviation="6" result="blur" />
            <feMerge>
              <feMergeNode in="blur" />
              <feMergeNode in="SourceGraphic" />
            </feMerge>
          </filter>
          <filter id="cover-glow-soft" x="-100%" y="-100%" width="300%" height="300%">
            <feGaussianBlur in="SourceGraphic" stdDeviation="2.4" />
          </filter>
        </defs>

        {/* Atmospheric rings drawing in sequence */}
        {[
          { r: 320, color: "var(--c-gold)", opacity: 0.14, delay: 0.5 },
          { r: 380, color: "var(--c-gold)", opacity: 0.18, delay: 0.65 },
          { r: 430, color: "var(--c-emerald)", opacity: 0.22, delay: 0.8 },
          { r: 480, color: "var(--c-gold)", opacity: 0.10, delay: 0.95 },
        ].map((ring, i) => (
          <motion.circle
            key={i}
            cx={500}
            cy={500}
            r={ring.r}
            fill="none"
            stroke={ring.color}
            strokeWidth={0.5}
            opacity={ring.opacity}
            initial={{ pathLength: 0 }}
            animate={{ pathLength: 1 }}
            transition={{ duration: 2.0, delay: ring.delay, ease: "easeOut" }}
          />
        ))}

        {/* Centered topology sigil with bloom filter */}
        <motion.g
          transform="translate(290, 290) scale(0.42)"
          filter="url(#cover-glow-strong)"
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          transition={{ delay: 0.8, duration: 1.6 }}
          dangerouslySetInnerHTML={{ __html: topologySvg }}
        />

        {/* Subject constellation points */}
        {subjectLabels}

        {/* Wordmark — outer top arc */}
        <motion.text
          fill="var(--c-gold)"
          fontFamily="var(--font-mono)"
          fontSize={11}
          letterSpacing={9}
          opacity={0.85}
          initial={{ opacity: 0 }}
          animate={{ opacity: 0.85 }}
          transition={{ delay: 1.1, duration: 1.2 }}
        >
          <textPath href="#cover-arc-wordmark" startOffset="50%" textAnchor="middle">
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
          filter="url(#cover-glow-soft)"
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          transition={{ delay: 1.6, duration: 1.4 }}
        >
          <textPath href="#cover-arc-title" startOffset="50%" textAnchor="middle">
            {title}
          </textPath>
        </motion.text>

        {/* Birth meta — lower arc, SF Mono */}
        <motion.text
          fill="var(--c-emerald)"
          fontFamily="var(--font-mono)"
          fontSize={14}
          letterSpacing={4}
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          transition={{ delay: 1.9, duration: 1.4 }}
        >
          <textPath href="#cover-arc-subtitle" startOffset="50%" textAnchor="middle">
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
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          transition={{ delay: 2.2, duration: 1.4 }}
        >
          <textPath href="#cover-arc-lineage" startOffset="50%" textAnchor="middle">
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
          initial={{ opacity: 0 }}
          animate={{ opacity: 0.9 }}
          transition={{ delay: 2.5, duration: 1.0 }}
        >
          ∴  NOESIS  ∴
        </motion.text>
      </motion.svg>
    </section>
  );
}
