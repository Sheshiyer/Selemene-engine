"use client";

// ─── TriadMandala — Pass α / opening yantra ─────────────────────────────
// Per integrated-reading-design-v2.md § 5.4.
//
// v1: accept a `topologySvg` string (from witness-agents) and inject it.
// Falls back to a generative triadic triangle drawn from `subjects`.
//
// Animates via motion/react: an aura ring breathes in, then the injected
// topology fades up with a Sacred Gold halo.

import { motion, useReducedMotion } from "motion/react";

interface TriadMandalaProps {
  topologySvg?: string;
  subjects?: string[];
}

export function TriadMandala({ topologySvg, subjects = [] }: TriadMandalaProps) {
  const reduce = useReducedMotion();

  if (topologySvg && topologySvg.trim().length > 0) {
    return (
      <motion.div
        style={{
          width: "clamp(20rem, 50vw, 50rem)",
          margin: "clamp(1.5rem, 4vw, 3rem) auto",
          aspectRatio: "1 / 1",
          position: "relative",
          filter: "drop-shadow(0 0 24px rgba(197,160,23,0.18))",
        }}
        initial={reduce ? { opacity: 1 } : { opacity: 0, scale: 0.96 }}
        whileInView={{ opacity: 1, scale: 1 }}
        viewport={{ once: true, margin: "-10% 0px" }}
        transition={{ duration: 1.8, ease: [0.16, 1, 0.3, 1] }}
        aria-label="Triadic mandala — composite-field topology"
        role="img"
        dangerouslySetInnerHTML={{ __html: topologySvg }}
      />
    );
  }

  // Generative fallback — three vertices around centre. Works for N=3 cleanly,
  // and degrades gracefully for any N by spacing vertices on a circle.
  const N = Math.max(subjects.length, 3);
  const R = 280;
  const cx = 360;
  const cy = 360;
  const pts = Array.from({ length: N }, (_, i) => {
    const angle = -Math.PI / 2 + (i * 2 * Math.PI) / N;
    return { x: cx + R * Math.cos(angle), y: cy + R * Math.sin(angle), angle };
  });

  // Build pairwise lines (the triadic relations).
  const pairs: Array<[typeof pts[number], typeof pts[number]]> = [];
  for (let i = 0; i < pts.length; i++) {
    for (let j = i + 1; j < pts.length; j++) {
      pairs.push([pts[i], pts[j]]);
    }
  }

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
      aria-label="Triadic mandala"
      role="img"
    >
      <defs>
        <radialGradient id="tm-aura" cx="50%" cy="50%" r="50%">
          <stop offset="0%" stopColor="var(--c-gold)" stopOpacity="0.18" />
          <stop offset="100%" stopColor="var(--c-violet)" stopOpacity="0" />
        </radialGradient>
        <filter id="tm-glow" x="-40%" y="-40%" width="180%" height="180%">
          <feGaussianBlur stdDeviation="3" />
        </filter>
      </defs>

      <circle cx={cx} cy={cy} r={R + 40} fill="url(#tm-aura)" />

      {/* Outer ring */}
      <motion.circle
        cx={cx}
        cy={cy}
        r={R}
        fill="none"
        stroke="var(--c-gold)"
        strokeOpacity={0.45}
        strokeWidth={1.2}
        variants={{
          hidden: { pathLength: 0, opacity: 0 },
          visible: { pathLength: 1, opacity: 1 },
        }}
        transition={{ duration: drawDuration, ease: [0.16, 1, 0.3, 1] }}
      />

      {/* Triadic relation lines */}
      {pairs.map(([a, b], i) => (
        <motion.line
          key={i}
          x1={a.x}
          y1={a.y}
          x2={b.x}
          y2={b.y}
          stroke="var(--c-emerald)"
          strokeOpacity={0.6}
          strokeWidth={1.4}
          variants={{
            hidden: { pathLength: 0, opacity: 0 },
            visible: { pathLength: 1, opacity: 1 },
          }}
          transition={{
            duration: drawDuration,
            delay: 0.4 + i * 0.12,
            ease: [0.16, 1, 0.3, 1],
          }}
        />
      ))}

      {/* Vertex nodes + subject labels */}
      {pts.map((p, i) => (
        <motion.g
          key={i}
          variants={{
            hidden: { opacity: 0 },
            visible: { opacity: 1 },
          }}
          transition={{ duration: 0.7, delay: 0.9 + i * 0.18 }}
        >
          <circle
            cx={p.x}
            cy={p.y}
            r={10}
            fill="var(--c-gold)"
            filter="url(#tm-glow)"
          />
          <circle
            cx={p.x}
            cy={p.y}
            r={5}
            fill="var(--c-parchment)"
          />
          {subjects[i] ? (
            <text
              x={p.x}
              y={p.y + (p.y < cy ? -22 : 30)}
              textAnchor="middle"
              style={{
                fontFamily: "var(--font-mono)",
                fontSize: 12,
                fill: "var(--c-parchment)",
                letterSpacing: "0.18em",
                textTransform: "uppercase",
              }}
            >
              {subjects[i]}
            </text>
          ) : null}
        </motion.g>
      ))}

      {/* Centre seed */}
      <motion.circle
        cx={cx}
        cy={cy}
        r={18}
        fill="none"
        stroke="var(--c-gold)"
        strokeWidth={1.5}
        variants={{
          hidden: { scale: 0, opacity: 0 },
          visible: { scale: 1, opacity: 1 },
        }}
        style={{ originX: `${cx}px`, originY: `${cy}px` }}
        transition={{ duration: 0.8, delay: 1.4 }}
      />
    </motion.svg>
  );
}
