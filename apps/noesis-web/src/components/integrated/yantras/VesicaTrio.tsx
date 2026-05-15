"use client";

// ─── VesicaTrio — Pass β / resonance yantra ─────────────────────────────
// Per integrated-reading-design-v2.md § 5.4.
//
// Three interlocking circles arranged at 120° offsets from a central
// point. Each circle is a Goethe-spectrum tone (Gold / Emerald / Violet).
// Intersection regions illuminated by stacking translucent fills.
//
// Animates: stroke-builds (1.8s) then fills cross-fade up.

import { motion, useReducedMotion } from "motion/react";

interface VesicaTrioProps {
  subjects?: string[];
}

const CIRCLES = [
  { angleDeg: -90, stroke: "var(--c-gold)", fill: "rgba(197,160,23,0.18)" },
  { angleDeg: 30, stroke: "var(--c-emerald)", fill: "rgba(16,181,167,0.18)" },
  { angleDeg: 150, stroke: "var(--c-violet)", fill: "rgba(125,75,200,0.22)" },
] as const;

export function VesicaTrio({ subjects = [] }: VesicaTrioProps) {
  const reduce = useReducedMotion();
  const cx = 360;
  const cy = 360;
  const R = 200;
  // Offset distance: place circle centres so adjacent circles intersect
  // at the central point — classical vesica geometry uses centre-distance
  // = R. We bring them slightly closer (0.85R) so all three share a
  // common central region.
  const D = R * 0.85;

  const circles = CIRCLES.map((c, i) => {
    const a = (c.angleDeg * Math.PI) / 180;
    return {
      ...c,
      ccx: cx + D * Math.cos(a),
      ccy: cy + D * Math.sin(a),
      labelX: cx + (D + R + 24) * Math.cos(a),
      labelY: cy + (D + R + 24) * Math.sin(a),
      labelAngle: c.angleDeg,
      i,
    };
  });

  const drawDuration = reduce ? 0 : 1.8;

  return (
    <motion.svg
      viewBox="0 0 720 720"
      width="100%"
      height="auto"
      style={{
        width: "clamp(20rem, 50vw, 50rem)",
        margin: "clamp(1.5rem, 4vw, 3rem) auto",
        display: "block",
        aspectRatio: "1 / 1",
      }}
      initial="hidden"
      whileInView="visible"
      viewport={{ once: true, margin: "-10% 0px" }}
      aria-label="Vesica trio — three interlocking resonance circles"
      role="img"
    >
      <defs>
        <radialGradient id="vt-aura" cx="50%" cy="50%" r="50%">
          <stop offset="0%" stopColor="var(--c-gold)" stopOpacity="0.12" />
          <stop offset="100%" stopColor="var(--c-violet)" stopOpacity="0" />
        </radialGradient>
        <filter id="vt-glow" x="-40%" y="-40%" width="180%" height="180%">
          <feGaussianBlur stdDeviation="6" />
        </filter>
      </defs>

      <circle cx={cx} cy={cy} r={R * 2} fill="url(#vt-aura)" />

      {/* Translucent fills (mix-blend handled by stacking alpha) */}
      {circles.map((c) => (
        <motion.circle
          key={`f-${c.i}`}
          cx={c.ccx}
          cy={c.ccy}
          r={R}
          fill={c.fill}
          stroke="none"
          variants={{
            hidden: { opacity: 0 },
            visible: { opacity: 1 },
          }}
          transition={{
            duration: reduce ? 0 : 1.2,
            delay: reduce ? 0 : 1.4,
            ease: [0.16, 1, 0.3, 1],
          }}
        />
      ))}

      {/* Strokes — drawn line-by-line */}
      {circles.map((c) => (
        <motion.circle
          key={`s-${c.i}`}
          cx={c.ccx}
          cy={c.ccy}
          r={R}
          fill="none"
          stroke={c.stroke}
          strokeWidth={1.6}
          strokeOpacity={0.85}
          variants={{
            hidden: { pathLength: 0, opacity: 0 },
            visible: { pathLength: 1, opacity: 1 },
          }}
          transition={{
            duration: drawDuration,
            delay: c.i * 0.18,
            ease: [0.16, 1, 0.3, 1],
          }}
        />
      ))}

      {/* Centre — where all three intersect */}
      <motion.circle
        cx={cx}
        cy={cy}
        r={10}
        fill="var(--c-gold)"
        filter="url(#vt-glow)"
        variants={{
          hidden: { scale: 0, opacity: 0 },
          visible: { scale: 1, opacity: 1 },
        }}
        style={{ originX: `${cx}px`, originY: `${cy}px` }}
        transition={{ duration: 0.8, delay: 1.8 }}
      />
      <motion.circle
        cx={cx}
        cy={cy}
        r={5}
        fill="var(--c-parchment)"
        variants={{
          hidden: { scale: 0, opacity: 0 },
          visible: { scale: 1, opacity: 1 },
        }}
        style={{ originX: `${cx}px`, originY: `${cy}px` }}
        transition={{ duration: 0.8, delay: 2.0 }}
      />

      {/* Subject labels at each circle's far edge */}
      {subjects.slice(0, 3).map((s, i) => (
        <motion.text
          key={`l-${i}`}
          x={circles[i].labelX}
          y={circles[i].labelY}
          textAnchor="middle"
          style={{
            fontFamily: "var(--font-mono)",
            fontSize: 12,
            fill: "var(--c-parchment)",
            letterSpacing: "0.18em",
            textTransform: "uppercase",
          }}
          variants={{
            hidden: { opacity: 0 },
            visible: { opacity: 1 },
          }}
          transition={{
            duration: 0.8,
            delay: 1.6 + i * 0.12,
          }}
        >
          {s}
        </motion.text>
      ))}
    </motion.svg>
  );
}
