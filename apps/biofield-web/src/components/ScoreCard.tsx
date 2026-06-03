"use client";

/**
 * ScoreCard — Wave 1, built 1:1 to docs/design/biofield-web/04-scorecard-spec.png
 *
 * A single metric rendered as a radial compass-node RING — NOT a boxed card,
 * NO progress bar. One open arc (gap at the bottom) is the gauge track; the
 * value fills it clockwise along the Ba Arc (Coherence Emerald -> Sacred Gold).
 * A large SF-Mono numeral sits dead-center, the metric label in wide caps
 * beneath it, a faint constellation grid behind, and a small radial "notch"
 * marks the baseline on the ring.
 *
 * Motion: Anime.js v4 (named `animate`). Three behaviours, all guarded by
 * prefers-reduced-motion and cleaned up on unmount / re-run:
 *   1. Mount — the value arc draws itself in (strokeDashoffset reveal).
 *   2. Value — the arc tweens to its new fill when `value` changes.
 *   3. Glow  — a subtle opacity pulse breathes on the center numeral.
 *
 * Props: { label, value (0..1), baseline?, accent?, size? }.
 */

import { useEffect, useMemo, useRef } from "react";
import { animate } from "animejs";

const GOLD = "#C5A017";
const EMERALD = "#10B5A7";
const PARCHMENT = "#F0EDE3";
const SILVER = "#8A9BA8";

type Size = "large" | "compact" | "mini";
const DIMS: Record<Size, number> = { large: 240, compact: 168, mini: 104 };

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

// The gauge is an open compass ring: a 270deg sweep with a 90deg gap centered
// at the bottom. Sweep starts at the lower-left (225deg) and ends lower-right.
const GAUGE_START = 225;
const GAUGE_SPAN = 270;

export interface ScoreCardProps {
  /** Metric name, rendered in wide caps beneath the numeral. */
  label: string;
  /** Normalized metric value, 0..1. */
  value: number;
  /** Optional reference value, 0..1 — drawn as a radial notch on the ring. */
  baseline?: number;
  /** Optional accent override for the arc tip / numeral glow (defaults to the Ba Arc gold end). */
  accent?: string;
  size?: Size;
}

export function ScoreCard({
  label,
  value,
  baseline,
  accent = GOLD,
  size = "large",
}: ScoreCardProps) {
  const VB = 240;
  const C = VB / 2;
  const RING_R = 96;
  const GUIDES = [96, 72, 48];

  const v = clamp01(value);
  const valueText = v.toFixed(2);
  // Stable per-instance gradient id so multiple cards don't collide.
  const uid = useMemo(() => Math.random().toString(36).slice(2, 8), []);
  const gradId = `sc-ba-${uid}`;
  const coreId = `sc-core-${uid}`;
  const glowId = `sc-glow-${uid}`;

  const rootRef = useRef<SVGSVGElement | null>(null);
  const valueArcRef = useRef<SVGPathElement | null>(null);
  const numeralRef = useRef<SVGTextElement | null>(null);

  // Static geometry — independent of the live value.
  const trackPath = useMemo(
    () => arc(C, C, RING_R, GAUGE_START, GAUGE_SPAN),
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [],
  );
  const valuePath = useMemo(
    () => arc(C, C, RING_R, GAUGE_START, GAUGE_SPAN),
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [],
  );
  const guideDefs = useMemo(
    () => GUIDES.map((r) => ({ r, d: arc(C, C, r, GAUGE_START, GAUGE_SPAN) })),
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [],
  );
  // Faint constellation grid — dots scattered on a ring behind the gauge.
  const constellation = useMemo(
    () =>
      Array.from({ length: 40 }, (_, i) => {
        const p = polar(C, C, 112, (360 / 40) * i);
        return { ...p, big: i % 5 === 0 };
      }),
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [],
  );

  // Baseline notch — a short radial tick crossing the ring at the baseline pos.
  const notch = useMemo(() => {
    if (baseline == null) return null;
    const deg = GAUGE_START + clamp01(baseline) * GAUGE_SPAN;
    const inner = polar(C, C, RING_R - 7, deg);
    const outer = polar(C, C, RING_R + 7, deg);
    return { inner, outer };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [baseline]);

  // Mount: draw the faint guides in. Runs once.
  useEffect(() => {
    const reduce =
      typeof window !== "undefined" &&
      window.matchMedia?.("(prefers-reduced-motion: reduce)").matches;
    if (reduce) return;

    const anims: ReturnType<typeof animate>[] = [];
    const guides = rootRef.current?.querySelectorAll<SVGPathElement>(".sc-guide");
    guides?.forEach((g, i) => {
      const len = g.getTotalLength();
      g.style.strokeDasharray = `${len}`;
      g.style.strokeDashoffset = `${len}`;
      anims.push(
        animate(g, {
          strokeDashoffset: [len, 0],
          duration: 1300,
          ease: "outQuart",
          delay: 100 + i * 90,
        }),
      );
    });

    // Subtle glow pulse on the center numeral.
    if (numeralRef.current) {
      anims.push(
        animate(numeralRef.current, {
          opacity: [
            { to: 1, duration: 2200, ease: "inOutSine" },
            { to: 0.78, duration: 2200, ease: "inOutSine" },
          ],
          loop: true,
        }),
      );
    }

    return () => anims.forEach((a) => a.revert());
  }, []);

  // Value tween: fill the gauge arc to `value`. Re-runs when value changes.
  useEffect(() => {
    const path = valueArcRef.current;
    if (!path) return;

    const len = path.getTotalLength();
    const hidden = len * (1 - v);
    path.style.strokeDasharray = `${len}`;

    const reduce =
      typeof window !== "undefined" &&
      window.matchMedia?.("(prefers-reduced-motion: reduce)").matches;
    if (reduce) {
      path.style.strokeDashoffset = `${hidden}`;
      return;
    }

    const a = animate(path, {
      strokeDashoffset: [len, hidden],
      duration: 1200,
      ease: "outExpo",
      delay: 260,
    });
    return () => {
      a.revert();
    };
  }, [v]);

  const fontScale = DIMS[size] / DIMS.large;

  return (
    <svg
      ref={rootRef}
      viewBox={`0 0 ${VB} ${VB}`}
      width={DIMS[size]}
      height={DIMS[size]}
      preserveAspectRatio="xMidYMid meet"
      role="img"
      aria-label={`${label}, ${valueText}`}
    >
      <defs>
        {/* Ba Arc — emerald -> gold, swept along the gauge. */}
        <linearGradient id={gradId} x1="0%" y1="100%" x2="100%" y2="0%">
          <stop offset="0%" stopColor={EMERALD} />
          <stop offset="100%" stopColor={GOLD} />
        </linearGradient>
        <radialGradient id={coreId} cx="50%" cy="50%" r="50%">
          <stop offset="0%" stopColor={accent} stopOpacity="0.22" />
          <stop offset="70%" stopColor={accent} stopOpacity="0.05" />
          <stop offset="100%" stopColor={accent} stopOpacity="0" />
        </radialGradient>
        <filter id={glowId} x="-60%" y="-60%" width="220%" height="220%">
          <feGaussianBlur stdDeviation="2.6" result="b" />
          <feMerge>
            <feMergeNode in="b" />
            <feMergeNode in="SourceGraphic" />
          </feMerge>
        </filter>
      </defs>

      {/* faint constellation grid */}
      <g opacity="0.5">
        {constellation.map((p, i) => (
          <circle
            key={i}
            cx={p.x}
            cy={p.y}
            r={p.big ? 1.2 : 0.6}
            fill={GOLD}
            opacity={p.big ? 0.34 : 0.14}
          />
        ))}
      </g>

      {/* concentric guide arcs */}
      {guideDefs.map((g, i) => (
        <path
          key={i}
          className="sc-guide"
          d={g.d}
          fill="none"
          stroke={GOLD}
          strokeWidth={0.6}
          opacity={0.2 - i * 0.05}
        />
      ))}

      {/* soft core wash */}
      <circle cx={C} cy={C} r={70} fill={`url(#${coreId})`} />

      {/* gauge track */}
      <path
        d={trackPath}
        fill="none"
        stroke={SILVER}
        strokeWidth={2.5}
        opacity={0.12}
        strokeLinecap="round"
      />

      {/* gauge value fill (animated) */}
      <path
        ref={valueArcRef}
        d={valuePath}
        fill="none"
        stroke={`url(#${gradId})`}
        strokeWidth={4}
        strokeLinecap="round"
        filter={`url(#${glowId})`}
      />

      {/* baseline notch */}
      {notch && (
        <line
          x1={notch.inner.x}
          y1={notch.inner.y}
          x2={notch.outer.x}
          y2={notch.outer.y}
          stroke={PARCHMENT}
          strokeWidth={1.4}
          opacity={0.55}
          strokeLinecap="round"
        />
      )}

      {/* center numeral */}
      <text
        ref={numeralRef}
        x={C}
        y={C + 2}
        textAnchor="middle"
        dominantBaseline="middle"
        fontFamily="var(--font-mono, monospace)"
        fontSize={44 * fontScale}
        fontWeight={600}
        letterSpacing="-0.03em"
        fill={PARCHMENT}
      >
        {valueText}
      </text>

      {/* metric label */}
      <text
        x={C}
        y={C + 30 * fontScale}
        textAnchor="middle"
        dominantBaseline="middle"
        fontFamily="var(--font-mono, monospace)"
        fontSize={9 * fontScale}
        letterSpacing="3"
        fill={SILVER}
        opacity={0.75}
      >
        {label.toUpperCase()}
      </text>
    </svg>
  );
}

export default ScoreCard;
