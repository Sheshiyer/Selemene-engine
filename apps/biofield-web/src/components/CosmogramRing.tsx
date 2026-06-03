"use client";

/**
 * CosmogramRing — Wave 1, built 1:1 to docs/design/biofield-web/02-cosmogram-spec.png
 *
 * A single radial cosmogram: four axes COH / SYM / LUM / REG at the compass
 * points, each an arc gauge filled to its metric value along the Ba Arc
 * (Coherence Emerald -> Sacred Gold). Center shows the coherence score; a
 * bioluminescent emerald core breathes on a 4:7:8 cadence.
 *
 * Motion: Anime.js v4 (named `animate`). Three behaviours, all guarded by
 * prefers-reduced-motion:
 *   1. Mount — the geometry draws itself (strokeDashoffset reveal, staggered).
 *   2. Breath — the core pulses on the 4:7:8 inhale/hold/exhale ratio.
 *   3. Value — each arc tweens to its new fill when `scores` change.
 *
 * Data: CompositeScores (live PIP metrics). COH=overallCoherence,
 * SYM=bodySymmetry, LUM=lightQuantaDensity, REG=patternRegularity.
 */

import { useEffect, useMemo, useRef } from "react";
import { animate } from "animejs";
import type { CompositeScores } from "./pip/types";

const GOLD = "#C5A017";
const EMERALD = "#10B5A7";
const VIOLET = "#2D0050";
const PARCHMENT = "#F0EDE3";
const SILVER = "#8A9BA8";

type Size = "large" | "compact" | "mini";
const DIMS: Record<Size, number> = { large: 360, compact: 220, mini: 132 };

const clamp01 = (v: number) => Math.max(0, Math.min(1, v));
const rad = (deg: number) => ((deg - 90) * Math.PI) / 180; // 0deg = top

function polar(cx: number, cy: number, r: number, deg: number) {
  const a = rad(deg);
  return { x: cx + r * Math.cos(a), y: cy + r * Math.sin(a) };
}

/** Arc path from startDeg sweeping `spanDeg` clockwise at radius r. */
function arc(cx: number, cy: number, r: number, startDeg: number, spanDeg: number) {
  const s = polar(cx, cy, r, startDeg);
  const e = polar(cx, cy, r, startDeg + spanDeg);
  const large = spanDeg > 180 ? 1 : 0;
  return `M ${s.x.toFixed(2)} ${s.y.toFixed(2)} A ${r} ${r} 0 ${large} 1 ${e.x.toFixed(2)} ${e.y.toFixed(2)}`;
}

interface Axis {
  key: keyof CompositeScores;
  label: string;
  centerDeg: number; // compass position of this axis
}

// COH top, LUM right, REG bottom, SYM left (matches the spec sheet)
const AXES: Axis[] = [
  { key: "overallCoherence", label: "COH", centerDeg: 0 },
  { key: "lightQuantaDensity", label: "LUM", centerDeg: 90 },
  { key: "patternRegularity", label: "REG", centerDeg: 180 },
  { key: "bodySymmetry", label: "SYM", centerDeg: 270 },
];

const ARC_SPAN = 64; // degrees each axis arc occupies, centered on its compass point

export interface CosmogramRingProps {
  scores: CompositeScores;
  size?: Size;
}

export function CosmogramRing({ scores, size = "large" }: CosmogramRingProps) {
  const VB = 320;
  const C = VB / 2;
  const RING_R = 128;
  const GUIDES = [128, 100, 74];

  const coherence = clamp01(scores.overallCoherence);
  const cohPct = coherence.toFixed(2);
  const stateColor = coherence >= 0.75 ? EMERALD : coherence >= 0.5 ? GOLD : VIOLET;
  const stateWord = coherence >= 0.75 ? "COHERENT" : coherence >= 0.5 ? "BUILDING" : "ATTUNING";

  const rootRef = useRef<SVGSVGElement | null>(null);
  const coreRef = useRef<SVGCircleElement | null>(null);
  const arcRefs = useRef<Array<SVGPathElement | null>>([]);

  // Geometry that doesn't depend on live values
  const guideDefs = useMemo(
    () => GUIDES.map((r) => ({ r, d: arc(C, C, r, 0, 359.9) })),
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [],
  );
  const axisArcs = useMemo(
    () =>
      AXES.map((ax) => {
        const full = arc(C, C, RING_R, ax.centerDeg - ARC_SPAN / 2, ARC_SPAN);
        const tip = polar(C, C, RING_R + 18, ax.centerDeg);
        return { ...ax, full, tip };
      }),
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [],
  );
  const constellation = useMemo(
    () =>
      Array.from({ length: 48 }, (_, i) => {
        const p = polar(C, C, 150, (360 / 48) * i);
        return { ...p, big: i % 6 === 0 };
      }),
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [],
  );

  // Mount: draw-in + breath. Runs once.
  useEffect(() => {
    const reduce =
      typeof window !== "undefined" &&
      window.matchMedia?.("(prefers-reduced-motion: reduce)").matches;
    if (reduce) return;

    const guides = rootRef.current?.querySelectorAll<SVGPathElement>(".cg-guide");
    guides?.forEach((g) => {
      const len = g.getTotalLength();
      g.style.strokeDasharray = `${len}`;
      g.style.strokeDashoffset = `${len}`;
      animate(g, { strokeDashoffset: [len, 0], duration: 1400, ease: "outQuart", delay: 120 });
    });

    if (coreRef.current) {
      // 4:7:8 breath — inhale 4, hold 7, exhale 8 (proportional keyframes).
      animate(coreRef.current, {
        scale: [
          { to: 1.16, duration: 4000, ease: "inOutSine" },
          { to: 1.16, duration: 7000 },
          { to: 1.0, duration: 8000, ease: "inOutSine" },
        ],
        opacity: [
          { to: 1, duration: 4000 },
          { to: 1, duration: 7000 },
          { to: 0.65, duration: 8000 },
        ],
        loop: true,
      });
    }
  }, []);

  // Value tween: fill each axis arc to its metric value.
  useEffect(() => {
    const reduce =
      typeof window !== "undefined" &&
      window.matchMedia?.("(prefers-reduced-motion: reduce)").matches;

    arcRefs.current.forEach((path, i) => {
      if (!path) return;
      const len = path.getTotalLength();
      const value = clamp01(Number(scores[AXES[i].key]) || 0);
      const hidden = len * (1 - value);
      path.style.strokeDasharray = `${len}`;
      if (reduce) {
        path.style.strokeDashoffset = `${hidden}`;
        return;
      }
      animate(path, { strokeDashoffset: [len, hidden], duration: 1100, ease: "outExpo", delay: 300 });
    });
  }, [scores]);

  const fontScale = DIMS[size] / DIMS.large;

  return (
    <svg
      ref={rootRef}
      viewBox={`0 0 ${VB} ${VB}`}
      width={DIMS[size]}
      height={DIMS[size]}
      preserveAspectRatio="xMidYMid meet"
      role="img"
      aria-label={`Biofield cosmogram, coherence ${cohPct}`}
    >
      <defs>
        <radialGradient id="cg-core" cx="50%" cy="50%" r="50%">
          <stop offset="0%" stopColor={stateColor} stopOpacity="0.9" />
          <stop offset="60%" stopColor={stateColor} stopOpacity="0.18" />
          <stop offset="100%" stopColor={stateColor} stopOpacity="0" />
        </radialGradient>
        <filter id="cg-glow" x="-60%" y="-60%" width="220%" height="220%">
          <feGaussianBlur stdDeviation="3.5" result="b" />
          <feMerge>
            <feMergeNode in="b" />
            <feMergeNode in="SourceGraphic" />
          </feMerge>
        </filter>
      </defs>

      {/* constellation grid */}
      <g opacity="0.5">
        {constellation.map((p, i) => (
          <circle key={i} cx={p.x} cy={p.y} r={p.big ? 1.3 : 0.7} fill={GOLD} opacity={p.big ? 0.4 : 0.18} />
        ))}
      </g>

      {/* concentric guide rings */}
      {guideDefs.map((g, i) => (
        <path
          key={i}
          className="cg-guide"
          d={g.d}
          fill="none"
          stroke={GOLD}
          strokeWidth={0.6}
          opacity={0.22 - i * 0.05}
        />
      ))}

      {/* axis arc gauges */}
      {axisArcs.map((ax, i) => (
        <g key={ax.label}>
          {/* track */}
          <path d={ax.full} fill="none" stroke={SILVER} strokeWidth={2} opacity={0.12} strokeLinecap="round" />
          {/* value fill (animated) */}
          <path
            ref={(el) => {
              arcRefs.current[i] = el;
            }}
            d={ax.full}
            fill="none"
            stroke={i === 0 ? EMERALD : GOLD}
            strokeWidth={3.5}
            strokeLinecap="round"
            opacity={0.9}
          />
          {/* axis tick + label */}
          <text
            x={ax.tip.x}
            y={ax.tip.y}
            textAnchor="middle"
            dominantBaseline="middle"
            fontFamily="var(--font-mono, monospace)"
            fontSize={11}
            letterSpacing="1.5"
            fill={PARCHMENT}
            opacity={0.6}
          >
            {ax.label}
          </text>
        </g>
      ))}

      {/* bioluminescent core */}
      <circle cx={C} cy={C} r={40} fill="url(#cg-core)" />
      <circle
        ref={coreRef}
        cx={C}
        cy={C}
        r={6}
        fill={stateColor}
        filter="url(#cg-glow)"
        style={{ transformOrigin: `${C}px ${C}px` }}
      />

      {/* center readout */}
      <text
        x={C}
        y={C - 4}
        textAnchor="middle"
        fontFamily="var(--font-mono, monospace)"
        fontSize={38 * fontScale}
        fontWeight={600}
        letterSpacing="-0.02em"
        fill={PARCHMENT}
      >
        {cohPct}
      </text>
      <text
        x={C}
        y={C + 18}
        textAnchor="middle"
        fontFamily="var(--font-display, 'Panchang', sans-serif)"
        fontSize={8}
        letterSpacing="3"
        fill={SILVER}
        opacity={0.7}
      >
        COHERENCE SCORE
      </text>
      <text
        x={C}
        y={C + 34}
        textAnchor="middle"
        fontFamily="var(--font-display, 'Panchang', sans-serif)"
        fontSize={8}
        letterSpacing="3"
        fill={stateColor}
        opacity={0.85}
      >
        {stateWord}
      </text>
    </svg>
  );
}

export default CosmogramRing;
