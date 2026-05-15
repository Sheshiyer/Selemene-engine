"use client";

// ─── DashaSpiral — Pass γ / phase-lock yantra ───────────────────────────
// Per integrated-reading-design-v2.md § 5.4.
//
// 9 concentric rings, one per mahadasha lord. Radii proportional to the
// standard Vimshottari durations (Sun=6yr smallest → Venus=20yr largest).
// Past periods: Witness Violet. Current: Sacred Gold (with bloom).
// Future: Coherence Emerald.

import { motion, useReducedMotion } from "motion/react";

interface DashaPeriod {
  lord: string;
  start_iso: string;
  end_iso: string;
  current?: boolean;
}

interface DashaSpiralProps {
  dashaPeriods?: DashaPeriod[];
}

// Canonical Vimshottari period lengths in years.
const VIMSHOTTARI_YEARS: Record<string, number> = {
  ketu: 7,
  venus: 20,
  sun: 6,
  moon: 10,
  mars: 7,
  rahu: 18,
  jupiter: 16,
  saturn: 19,
  mercury: 17,
};

// Canonical traversal order.
const VIMSHOTTARI_ORDER = [
  "ketu",
  "venus",
  "sun",
  "moon",
  "mars",
  "rahu",
  "jupiter",
  "saturn",
  "mercury",
] as const;

function lordSymbol(lord: string): string {
  const map: Record<string, string> = {
    sun: "☉",
    moon: "☽",
    mars: "♂",
    rahu: "☊",
    jupiter: "♃",
    saturn: "♄",
    mercury: "☿",
    ketu: "☋",
    venus: "♀",
  };
  return map[lord.toLowerCase()] ?? lord.slice(0, 2).toUpperCase();
}

export function DashaSpiral({ dashaPeriods = [] }: DashaSpiralProps) {
  const reduce = useReducedMotion();

  // Determine state per lord. Prefer caller-supplied state via the
  // `current` flag and start/end ISO ordering; otherwise mark all as
  // "future" (visual neutral).
  const today = new Date();
  const stateByLord: Record<string, "past" | "current" | "future"> = {};
  for (const lord of VIMSHOTTARI_ORDER) stateByLord[lord] = "future";
  for (const p of dashaPeriods) {
    const start = new Date(p.start_iso);
    const end = new Date(p.end_iso);
    if (p.current || (today >= start && today <= end)) {
      stateByLord[p.lord.toLowerCase()] = "current";
    } else if (end < today) {
      stateByLord[p.lord.toLowerCase()] = "past";
    }
  }

  const cx = 360;
  const cy = 360;
  // Scale ring radii by sqrt(years) for visual balance — pure linear
  // makes Venus dominate too aggressively.
  const minR = 60;
  const maxR = 320;
  const years = VIMSHOTTARI_ORDER.map((l) => VIMSHOTTARI_YEARS[l]);
  const yrMax = Math.max(...years);
  const yrMin = Math.min(...years);
  const radiusFor = (lord: string) => {
    const y = VIMSHOTTARI_YEARS[lord];
    const t = (Math.sqrt(y) - Math.sqrt(yrMin)) / (Math.sqrt(yrMax) - Math.sqrt(yrMin));
    return minR + t * (maxR - minR);
  };

  const strokeFor = (state: "past" | "current" | "future") => {
    if (state === "current") return "var(--c-gold)";
    if (state === "past") return "var(--c-violet)";
    return "var(--c-emerald)";
  };

  const opacityFor = (state: "past" | "current" | "future") => {
    if (state === "current") return 0.9;
    if (state === "past") return 0.4;
    return 0.55;
  };

  const drawDuration = reduce ? 0 : 1.8;

  // Sort rings by radius asc so labels stack predictably.
  const rings = VIMSHOTTARI_ORDER.map((lord, idx) => ({
    lord,
    state: stateByLord[lord],
    r: radiusFor(lord),
    orderIndex: idx,
  })).sort((a, b) => a.r - b.r);

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
      aria-label="Dasha spiral — Vimshottari mahadasha rings"
      role="img"
    >
      <defs>
        <radialGradient id="ds-aura" cx="50%" cy="50%" r="50%">
          <stop offset="0%" stopColor="var(--c-gold)" stopOpacity="0.14" />
          <stop offset="100%" stopColor="var(--c-violet)" stopOpacity="0" />
        </radialGradient>
        <filter id="ds-bloom" x="-50%" y="-50%" width="200%" height="200%">
          <feGaussianBlur stdDeviation="5" result="b" />
          <feMerge>
            <feMergeNode in="b" />
            <feMergeNode in="SourceGraphic" />
          </feMerge>
        </filter>
      </defs>

      <circle cx={cx} cy={cy} r={maxR + 20} fill="url(#ds-aura)" />

      {rings.map((ring, i) => (
        <motion.circle
          key={ring.lord}
          cx={cx}
          cy={cy}
          r={ring.r}
          fill="none"
          stroke={strokeFor(ring.state)}
          strokeWidth={ring.state === "current" ? 2.2 : 1.2}
          strokeOpacity={opacityFor(ring.state)}
          filter={ring.state === "current" ? "url(#ds-bloom)" : undefined}
          variants={{
            hidden: { pathLength: 0, opacity: 0 },
            visible: { pathLength: 1, opacity: 1 },
          }}
          transition={{
            duration: drawDuration,
            delay: i * 0.1,
            ease: [0.16, 1, 0.3, 1],
          }}
        />
      ))}

      {/* Lord glyphs along the top of each ring */}
      {rings.map((ring, i) => (
        <motion.text
          key={`g-${ring.lord}`}
          x={cx}
          y={cy - ring.r - 6}
          textAnchor="middle"
          style={{
            fontFamily: "var(--font-mono)",
            fontSize: 11,
            fill:
              ring.state === "current"
                ? "var(--c-gold)"
                : "var(--c-parchment)",
            opacity: ring.state === "past" ? 0.55 : 0.92,
            letterSpacing: "0.1em",
          }}
          variants={{
            hidden: { opacity: 0 },
            visible: { opacity: 1 },
          }}
          transition={{ duration: 0.6, delay: 1.2 + i * 0.08 }}
        >
          {lordSymbol(ring.lord)} {ring.lord.toUpperCase()}
        </motion.text>
      ))}

      {/* Centre seed */}
      <motion.circle
        cx={cx}
        cy={cy}
        r={12}
        fill="var(--c-gold)"
        filter="url(#ds-bloom)"
        variants={{
          hidden: { scale: 0, opacity: 0 },
          visible: { scale: 1, opacity: 1 },
        }}
        style={{ originX: `${cx}px`, originY: `${cy}px` }}
        transition={{ duration: 0.8, delay: 1.6 }}
      />
    </motion.svg>
  );
}
