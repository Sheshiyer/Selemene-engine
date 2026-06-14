"use client";

/**
 * BiofieldMandala — Wave 1, built 1:1 to docs/design/biofield-web/05-metrics-panel-spec.png
 *
 * The 11 biofield metrics rendered as a sacred-geometry concentric mandala
 * (NOT a list / cards). Three nested rings, one per metric group:
 *
 *   OUTER  ENERGY   (Sacred Gold #C5A017) — light_quanta_density, normalized_area,
 *                    average_intensity, inner_noise
 *   MIDDLE GEOMETRY (Coherence Emerald #10B5A7) — body_symmetry, contour_complexity,
 *                    pattern_regularity
 *   INNER  CHAOS    (Flow Indigo #0B50FB) — fractal_dimension, correlation_dimension,
 *                    entropy_form_coefficient
 *
 * Each metric is a labeled arc segment whose fill-length encodes its normalized
 * (0..1) value, with a small SF-Mono value printed at the rim. Compass ticks sit
 * at the four cardinal points; a faint Coherence Emerald bioluminescent core
 * breathes at center; a hairline (Muted Silver) constellation grid sits behind.
 *
 * Motion: Anime.js v4 (named `animate`), mirroring CosmogramRing. All guarded by
 * prefers-reduced-motion + cleaned up on unmount:
 *   1. Mount — each ring's guide circle draws itself in (strokeDashoffset),
 *      staggered outer -> inner.
 *   2. Value — each arc tweens its fill to the metric value when `metrics` change.
 *   3. Breath — the emerald core pulses on a slow inhale/exhale cadence.
 */

import { useEffect, useMemo, useRef } from "react";
import { animate } from "animejs";
import type { BiofieldMetrics } from "@selemene/biofield-domain";

const GOLD = "#C5A017"; // Sacred Gold   — ENERGY ring
const EMERALD = "#10B5A7"; // Coherence Emerald — GEOMETRY ring + core
const INDIGO = "#0B50FB"; // Flow Indigo   — CHAOS ring
const PARCHMENT = "#F0EDE3"; // primary text
const SILVER = "#8A9BA8"; // Muted Silver — hairlines / tracks

const clamp01 = (v: number) => Math.max(0, Math.min(1, v));
const rad = (deg: number) => ((deg - 90) * Math.PI) / 180; // 0deg = top (N)

function polar(cx: number, cy: number, r: number, deg: number) {
  const a = rad(deg);
  return { x: cx + r * Math.cos(a), y: cy + r * Math.sin(a) };
}

/** Arc path from startDeg sweeping `spanDeg` clockwise at radius r. */
function arc(cx: number, cy: number, r: number, startDeg: number, spanDeg: number) {
  const s = polar(cx, cy, r, startDeg);
  const e = polar(cx, cy, r, startDeg + spanDeg);
  const large = Math.abs(spanDeg) > 180 ? 1 : 0;
  const sweep = spanDeg >= 0 ? 1 : 0;
  return `M ${s.x.toFixed(2)} ${s.y.toFixed(2)} A ${r} ${r} 0 ${large} ${sweep} ${e.x.toFixed(2)} ${e.y.toFixed(2)}`;
}

/**
 * Per-metric normalization. Several biofield metrics are already ~0..1; others
 * (densities, areas, intensities, fractal dimensions) live on wider domains and
 * are scaled against sensible maxima so the arc fill reads truthfully. Sample
 * values in the spec sheet (0.73 / 0.68 / 0.81 / 0.22 / 0.76 / 0.54 / 0.47) sit
 * comfortably mid-range under these.
 */
function normalize(key: keyof BiofieldMetrics, raw: number): number {
  const v = Number(raw) || 0;
  switch (key) {
    // Already bounded 0..1 in the engine output.
    case "body_symmetry":
    case "pattern_regularity":
    case "correlation_dimension":
    case "inner_noise":
    case "entropy_form_coefficient":
    case "normalized_area":
      return clamp01(v);
    // Fractal dimension of a 2D form lives in ~[1, 2]; map onto 0..1.
    case "fractal_dimension":
      return clamp01(v - 1);
    // Contour complexity: ratio-like, occasionally > 1.
    case "contour_complexity":
      return clamp01(v / 2);
    // Photon-count density — large integer-ish counts.
    case "light_quanta_density":
      return clamp01(v / 1000);
    // Mean pixel intensity on a 0..255 scale.
    case "average_intensity":
      return clamp01(v / 255);
    default:
      return clamp01(v);
  }
}

interface Metric {
  key: keyof BiofieldMetrics;
  label: string;
  centerDeg: number; // compass position of the segment's middle
}

interface RingDef {
  group: string;
  color: string;
  r: number; // arc radius
  metrics: Metric[];
}

// Segment span (deg) per metric and the gap between segments on a ring.
const SEG_GAP = 14;

/**
 * Ring layout, outer -> inner. Compass positions chosen to match the spec:
 * ENERGY metrics anchor the four diagonals (NW/NE/SE/SW); GEOMETRY anchors
 * top / lower-left / lower-right; CHAOS anchors top / lower-left / bottom.
 */
const RINGS: RingDef[] = [
  {
    group: "ENERGY",
    color: GOLD,
    r: 132,
    metrics: [
      { key: "light_quanta_density", label: "LIGHT QUANTA DENSITY", centerDeg: 315 },
      { key: "normalized_area", label: "NORMALIZED AREA", centerDeg: 45 },
      { key: "average_intensity", label: "AVERAGE INTENSITY", centerDeg: 135 },
      { key: "inner_noise", label: "INNER NOISE", centerDeg: 225 },
    ],
  },
  {
    group: "GEOMETRY",
    color: EMERALD,
    r: 96,
    metrics: [
      { key: "body_symmetry", label: "BODY SYMMETRY", centerDeg: 0 },
      { key: "pattern_regularity", label: "PATTERN REGULARITY", centerDeg: 120 },
      { key: "contour_complexity", label: "CONTOUR COMPLEXITY", centerDeg: 240 },
    ],
  },
  {
    group: "CHAOS",
    color: INDIGO,
    r: 62,
    metrics: [
      { key: "fractal_dimension", label: "FRACTAL DIM", centerDeg: 300 },
      { key: "correlation_dimension", label: "CORRELATION DIM", centerDeg: 60 },
      { key: "entropy_form_coefficient", label: "ENTROPY", centerDeg: 180 },
    ],
  },
];

// Golden-ratio concentric guide rings behind the arcs (φ ≈ 1.618 spacing).
const GUIDE_RADII = [150, 132, 96, 62, 38];

const COMPASS = [
  { label: "N", deg: 0 },
  { label: "E", deg: 90 },
  { label: "S", deg: 180 },
  { label: "W", deg: 270 },
];

export interface BiofieldMandalaProps {
  metrics: BiofieldMetrics;
  size?: number;
}

export function BiofieldMandala({ metrics, size = 360 }: BiofieldMandalaProps) {
  const VB = 320;
  const C = VB / 2;

  const rootRef = useRef<SVGSVGElement | null>(null);
  const coreRef = useRef<SVGCircleElement | null>(null);
  const arcRefs = useRef<Array<SVGPathElement | null>>([]);

  // Flatten metrics in render order so arcRefs[] indices are stable.
  const segments = useMemo(() => {
    let i = 0;
    return RINGS.flatMap((ring) =>
      ring.metrics.map((m) => {
        const span = ring.r === RINGS[0].r ? 56 : 64; // outer slightly tighter
        const startDeg = m.centerDeg - span / 2;
        const track = arc(C, C, ring.r, startDeg, span);
        const labelPt = polar(C, C, ring.r + (ring.r === RINGS[0].r ? 22 : 0), m.centerDeg);
        const valuePt = polar(C, C, ring.r, m.centerDeg + span / 2 + SEG_GAP / 2);
        return {
          index: i++,
          ring,
          metric: m,
          span,
          startDeg,
          track,
          labelPt,
          valuePt,
          value: normalize(m.key, metrics[m.key] as number),
        };
      }),
    );
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [metrics]);

  const guides = useMemo(
    () => GUIDE_RADII.map((r) => ({ r, d: arc(C, C, r, 0, 359.9) })),
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [],
  );

  // Faint constellation grid behind everything (hairline silver points).
  const constellation = useMemo(
    () =>
      Array.from({ length: 60 }, (_, i) => {
        const p = polar(C, C, 152, (360 / 60) * i);
        return { ...p, big: i % 5 === 0 };
      }),
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [],
  );

  // Group ring labels sit at the top of each band.
  const groupLabels = useMemo(
    () => RINGS.map((ring) => ({ group: ring.group, color: ring.color, pt: polar(C, C, ring.r, 0) })),
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [],
  );

  // Mount: draw-in guides (staggered outer -> inner) + breath the core. Runs once.
  useEffect(() => {
    const reduce =
      typeof window !== "undefined" &&
      window.matchMedia?.("(prefers-reduced-motion: reduce)").matches;
    if (reduce) return;

    const guideEls = rootRef.current?.querySelectorAll<SVGPathElement>(".bm-guide");
    const guideAnims = Array.from(guideEls ?? []).map((g, i) => {
      const len = g.getTotalLength();
      g.style.strokeDasharray = `${len}`;
      g.style.strokeDashoffset = `${len}`;
      return animate(g, {
        strokeDashoffset: [len, 0],
        duration: 1400,
        ease: "outQuart",
        delay: 120 + i * 110, // outer first, inner last
      });
    });

    let breath: ReturnType<typeof animate> | undefined;
    if (coreRef.current) {
      breath = animate(coreRef.current, {
        scale: [
          { to: 1.18, duration: 4200, ease: "inOutSine" },
          { to: 1.0, duration: 5200, ease: "inOutSine" },
        ],
        opacity: [
          { to: 1, duration: 4200 },
          { to: 0.6, duration: 5200 },
        ],
        loop: true,
      });
    }

    return () => {
      guideAnims.forEach((a) => a.pause());
      breath?.pause();
    };
  }, []);

  // Value tween: fill each arc to its normalized metric value.
  useEffect(() => {
    const reduce =
      typeof window !== "undefined" &&
      window.matchMedia?.("(prefers-reduced-motion: reduce)").matches;

    const anims: Array<ReturnType<typeof animate>> = [];
    arcRefs.current.forEach((path, i) => {
      if (!path) return;
      const len = path.getTotalLength();
      const value = segments[i]?.value ?? 0;
      const hidden = len * (1 - value);
      path.style.strokeDasharray = `${len}`;
      if (reduce) {
        path.style.strokeDashoffset = `${hidden}`;
        return;
      }
      anims.push(
        animate(path, {
          strokeDashoffset: [len, hidden],
          duration: 1100,
          ease: "outExpo",
          delay: 300 + i * 40,
        }),
      );
    });
    return () => anims.forEach((a) => a.pause());
  }, [segments]);

  return (
    <svg
      ref={rootRef}
      viewBox={`0 0 ${VB} ${VB}`}
      width={size}
      height={size}
      preserveAspectRatio="xMidYMid meet"
      role="img"
      aria-label="Biofield mandala — eleven metrics across energy, geometry and chaos rings"
    >
      <defs>
        <radialGradient id="bm-core" cx="50%" cy="50%" r="50%">
          <stop offset="0%" stopColor={EMERALD} stopOpacity="0.95" />
          <stop offset="55%" stopColor={EMERALD} stopOpacity="0.2" />
          <stop offset="100%" stopColor={EMERALD} stopOpacity="0" />
        </radialGradient>
        <filter id="bm-glow" x="-60%" y="-60%" width="220%" height="220%">
          <feGaussianBlur stdDeviation="3" result="b" />
          <feMerge>
            <feMergeNode in="b" />
            <feMergeNode in="SourceGraphic" />
          </feMerge>
        </filter>
      </defs>

      {/* constellation grid (hairline silver) */}
      <g opacity="0.5">
        {constellation.map((p, i) => (
          <circle
            key={i}
            cx={p.x}
            cy={p.y}
            r={p.big ? 1.1 : 0.6}
            fill={SILVER}
            opacity={p.big ? 0.35 : 0.16}
          />
        ))}
      </g>

      {/* concentric golden-ratio guide rings (draw in on mount) */}
      {guides.map((g, i) => (
        <path
          key={i}
          className="bm-guide"
          d={g.d}
          fill="none"
          stroke={SILVER}
          strokeWidth={0.6}
          opacity={0.2 - i * 0.025}
        />
      ))}

      {/* radial compass spokes to the cardinal ticks */}
      {COMPASS.map((c) => {
        const a = polar(C, C, 40, c.deg);
        const b = polar(C, C, 150, c.deg);
        return (
          <line
            key={c.label}
            x1={a.x}
            y1={a.y}
            x2={b.x}
            y2={b.y}
            stroke={SILVER}
            strokeWidth={0.5}
            opacity={0.14}
          />
        );
      })}

      {/* arc segments — one per metric, grouped by ring */}
      {segments.map((seg) => (
        <g key={seg.metric.key}>
          {/* track */}
          <path
            d={seg.track}
            fill="none"
            stroke={SILVER}
            strokeWidth={4}
            opacity={0.12}
            strokeLinecap="round"
          />
          {/* value fill (animated) */}
          <path
            ref={(el) => {
              arcRefs.current[seg.index] = el;
            }}
            d={seg.track}
            fill="none"
            stroke={seg.ring.color}
            strokeWidth={4}
            strokeLinecap="round"
            opacity={0.92}
          />
          {/* metric label — outer ring labels sit beyond the rim; inner labels ride the band */}
          <text
            x={seg.labelPt.x}
            y={seg.labelPt.y}
            textAnchor="middle"
            dominantBaseline="middle"
            fontFamily="var(--font-mono, monospace)"
            fontSize={seg.ring.r === RINGS[0].r ? 6.5 : 6}
            letterSpacing="0.6"
            fill={PARCHMENT}
            opacity={seg.ring.r === RINGS[0].r ? 0.62 : 0.78}
          >
            {seg.metric.label}
          </text>
          {/* SF-Mono value at the rim */}
          <text
            x={seg.valuePt.x}
            y={seg.valuePt.y}
            textAnchor="middle"
            dominantBaseline="middle"
            fontFamily="var(--font-mono, monospace)"
            fontSize={8}
            fontWeight={600}
            letterSpacing="-0.02em"
            fill={seg.ring.color}
            opacity={0.95}
          >
            {seg.value.toFixed(2)}
          </text>
        </g>
      ))}

      {/* group ring labels (ENERGY / GEOMETRY / CHAOS) at the top of each band */}
      {groupLabels.map((g) => (
        <text
          key={g.group}
          x={g.pt.x}
          y={g.pt.y - 5}
          textAnchor="middle"
          fontFamily="var(--font-mono, monospace)"
          fontSize={8}
          letterSpacing="2.5"
          fill={g.color}
          opacity={0.85}
        >
          {g.group}
        </text>
      ))}

      {/* bioluminescent emerald core */}
      <circle cx={C} cy={C} r={34} fill="url(#bm-core)" />
      <circle
        ref={coreRef}
        cx={C}
        cy={C}
        r={5}
        fill={EMERALD}
        filter="url(#bm-glow)"
        style={{ transformOrigin: `${C}px ${C}px` }}
      />

      {/* compass cardinal ticks */}
      {COMPASS.map((c) => {
        const p = polar(C, C, 158, c.deg);
        return (
          <text
            key={c.label}
            x={p.x}
            y={p.y}
            textAnchor="middle"
            dominantBaseline="middle"
            fontFamily="var(--font-mono, monospace)"
            fontSize={9}
            letterSpacing="1"
            fill={SILVER}
            opacity={0.7}
          >
            {c.label}
          </text>
        );
      })}
    </svg>
  );
}

export default BiofieldMandala;
