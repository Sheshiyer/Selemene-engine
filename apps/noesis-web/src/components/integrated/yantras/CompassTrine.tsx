"use client";

// ─── CompassTrine — Pass δ / anti-dependency yantra ─────────────────────
// Per integrated-reading-design-v2.md § 5.4.
//
// Octagonal compass frame with cardinal labels (N=STABILIZE, E=HEAL,
// S=CREATE, W=MUTATE) and a central seed sigil. Reference: breathnav.png.

import { motion, useReducedMotion } from "motion/react";

interface CompassTrineProps {
  cardinals?: {
    stabilize?: string;
    heal?: string;
    create?: string;
    mutate?: string;
  };
}

const CARDINAL_POSITIONS = [
  { key: "STABILIZE", angle: -90, accent: "var(--c-violet)" },
  { key: "HEAL", angle: 0, accent: "var(--c-indigo)" },
  { key: "CREATE", angle: 90, accent: "var(--c-gold)" },
  { key: "MUTATE", angle: 180, accent: "var(--c-emerald)" },
] as const;

export function CompassTrine({ cardinals }: CompassTrineProps) {
  const reduce = useReducedMotion();

  const cx = 360;
  const cy = 360;
  const outerR = 300;
  const innerR = 220;
  const seedR = 70;

  const drawDuration = reduce ? 0 : 1.8;

  // Octagon points — 8 vertices at 45° intervals starting from top
  const octPoints = Array.from({ length: 8 }, (_, i) => {
    const a = (-90 + i * 45) * (Math.PI / 180);
    return `${cx + outerR * Math.cos(a)},${cy + outerR * Math.sin(a)}`;
  }).join(" ");

  const cardinalText = (key: string): string | undefined => {
    if (!cardinals) return undefined;
    switch (key) {
      case "STABILIZE":
        return cardinals.stabilize;
      case "HEAL":
        return cardinals.heal;
      case "CREATE":
        return cardinals.create;
      case "MUTATE":
        return cardinals.mutate;
    }
    return undefined;
  };

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
      aria-label="Compass trine — cardinal-direction sigil"
      role="img"
    >
      <defs>
        <radialGradient id="ct-aura" cx="50%" cy="50%" r="50%">
          <stop offset="0%" stopColor="var(--c-gold)" stopOpacity="0.12" />
          <stop offset="100%" stopColor="var(--c-violet)" stopOpacity="0" />
        </radialGradient>
        <filter id="ct-glow" x="-50%" y="-50%" width="200%" height="200%">
          <feGaussianBlur stdDeviation="3" />
        </filter>
      </defs>

      <circle cx={cx} cy={cy} r={outerR + 20} fill="url(#ct-aura)" />

      {/* Outer octagonal frame */}
      <motion.polygon
        points={octPoints}
        fill="none"
        stroke="var(--c-gold)"
        strokeWidth={1.4}
        strokeOpacity={0.55}
        variants={{
          hidden: { pathLength: 0, opacity: 0 },
          visible: { pathLength: 1, opacity: 1 },
        }}
        transition={{ duration: drawDuration, ease: [0.16, 1, 0.3, 1] }}
      />

      {/* Inner circle */}
      <motion.circle
        cx={cx}
        cy={cy}
        r={innerR}
        fill="none"
        stroke="var(--c-emerald)"
        strokeWidth={1.1}
        strokeOpacity={0.45}
        variants={{
          hidden: { pathLength: 0, opacity: 0 },
          visible: { pathLength: 1, opacity: 1 },
        }}
        transition={{
          duration: drawDuration,
          delay: 0.2,
          ease: [0.16, 1, 0.3, 1],
        }}
      />

      {/* Spokes from cardinals to centre */}
      {CARDINAL_POSITIONS.map((p, i) => {
        const a = (p.angle * Math.PI) / 180;
        const x1 = cx + seedR * Math.cos(a);
        const y1 = cy + seedR * Math.sin(a);
        const x2 = cx + innerR * Math.cos(a);
        const y2 = cy + innerR * Math.sin(a);
        return (
          <motion.line
            key={`spoke-${p.key}`}
            x1={x1}
            y1={y1}
            x2={x2}
            y2={y2}
            stroke={p.accent}
            strokeWidth={1.2}
            strokeOpacity={0.7}
            variants={{
              hidden: { pathLength: 0, opacity: 0 },
              visible: { pathLength: 1, opacity: 1 },
            }}
            transition={{
              duration: drawDuration,
              delay: 0.4 + i * 0.1,
              ease: [0.16, 1, 0.3, 1],
            }}
          />
        );
      })}

      {/* Cardinal labels + outpost dots */}
      {CARDINAL_POSITIONS.map((p, i) => {
        const a = (p.angle * Math.PI) / 180;
        const dotR = innerR + 20;
        const labelR = innerR + 50;
        const dx = cx + dotR * Math.cos(a);
        const dy = cy + dotR * Math.sin(a);
        const lx = cx + labelR * Math.cos(a);
        const ly = cy + labelR * Math.sin(a);
        // Anchor adjust based on quadrant.
        const anchor =
          Math.abs(Math.cos(a)) < 0.3
            ? "middle"
            : Math.cos(a) > 0
            ? "start"
            : "end";
        const subline = cardinalText(p.key);
        return (
          <motion.g
            key={`card-${p.key}`}
            variants={{
              hidden: { opacity: 0 },
              visible: { opacity: 1 },
            }}
            transition={{ duration: 0.7, delay: 1.0 + i * 0.12 }}
          >
            <circle
              cx={dx}
              cy={dy}
              r={6}
              fill={p.accent}
              filter="url(#ct-glow)"
            />
            <text
              x={lx}
              y={ly}
              textAnchor={anchor}
              style={{
                fontFamily: "var(--font-display)",
                fontWeight: 700,
                fontSize: 14,
                fill: "var(--c-parchment)",
                letterSpacing: "0.32em",
              }}
            >
              {p.key}
            </text>
            {subline ? (
              <text
                x={lx}
                y={ly + 16}
                textAnchor={anchor}
                style={{
                  fontFamily: "var(--font-mono)",
                  fontSize: 10,
                  fill: "var(--muted)",
                  letterSpacing: "0.14em",
                }}
              >
                {subline}
              </text>
            ) : null}
          </motion.g>
        );
      })}

      {/* Central seed — interlocking triangle sigil */}
      <motion.g
        variants={{
          hidden: { opacity: 0, scale: 0.6 },
          visible: { opacity: 1, scale: 1 },
        }}
        style={{ originX: `${cx}px`, originY: `${cy}px` }}
        transition={{ duration: 0.9, delay: 1.4, ease: [0.16, 1, 0.3, 1] }}
      >
        <polygon
          points={`${cx},${cy - 40} ${cx - 34},${cy + 22} ${cx + 34},${cy + 22}`}
          fill="none"
          stroke="var(--c-gold)"
          strokeWidth={1.4}
          strokeOpacity={0.85}
        />
        <polygon
          points={`${cx},${cy + 40} ${cx - 34},${cy - 22} ${cx + 34},${cy - 22}`}
          fill="none"
          stroke="var(--c-emerald)"
          strokeWidth={1.4}
          strokeOpacity={0.85}
        />
        <circle
          cx={cx}
          cy={cy}
          r={5}
          fill="var(--c-gold)"
          filter="url(#ct-glow)"
        />
      </motion.g>
    </motion.svg>
  );
}
