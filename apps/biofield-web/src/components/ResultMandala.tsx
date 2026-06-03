"use client";

/**
 * ResultMandala — Wave 1, built 1:1 to docs/design/biofield-web/10-result-mandala-spec.png
 *
 * The post-capture biofield reading rendered as a sacred-geometry multi-ring
 * mandala (NOT a modal/card). Six metric arcs sit on nested concentric rings.
 * Each arc draws the CURRENT reading value; a faint GHOST arc behind it holds
 * the baseline. Where reading and baseline diverge, a delta segment extends
 * the arc — Coherence Emerald for improvement, Sacred Gold for decline. The
 * difference between ghost and reading IS the visual.
 *
 * Center: the coherence reading score. Top-right: an analysis-version label
 * (SF Mono) and the verdict drawn as a compass-seal — Coherence Emerald
 * octagon for ACCEPT, Terracotta for REJECTED.
 *
 * Motion: Anime.js v4 (named `animate`), guarded by prefers-reduced-motion
 * with cleanup:
 *   1. Mount — reading arcs draw/tween in over their ghost baseline arcs.
 *   2. Verdict — the compass-seal fades in after the arcs settle.
 *
 * Data: BiofieldMetrics (current + optional baseline). Each metric is
 * normalized to 0..1 for its arc. Deltas may be supplied via BiofieldMetricDelta[]
 * (keyed by metric); otherwise they are derived from baseline.
 */

import { useEffect, useMemo, useRef } from "react";
import { animate } from "animejs";
import type {
  BiofieldMetrics,
  BiofieldMetricDelta,
} from "@/lib/selemene/biofield-domain";

const GOLD = "#C5A017"; // Sacred Gold — decline
const EMERALD = "#10B5A7"; // Coherence Emerald — improvement / ACCEPT
const VIOLET = "#2D0050"; // Witness Violet
const PARCHMENT = "#F0EDE3"; // primary text
const SILVER = "#8A9BA8"; // muted labels
const TERRACOTTA = "#C65D3B"; // REJECTED only

const clamp01 = (v: number) => Math.max(0, Math.min(1, Number.isFinite(v) ? v : 0));
const rad = (deg: number) => ((deg - 90) * Math.PI) / 180; // 0deg = top

function polar(cx: number, cy: number, r: number, deg: number) {
  const a = rad(deg);
  return { x: cx + r * Math.cos(a), y: cy + r * Math.sin(a) };
}

/** Arc path sweeping `spanDeg` clockwise from startDeg at radius r. */
function arcPath(
  cx: number,
  cy: number,
  r: number,
  startDeg: number,
  spanDeg: number,
) {
  const s = polar(cx, cy, r, startDeg);
  const e = polar(cx, cy, r, startDeg + spanDeg);
  const large = Math.abs(spanDeg) > 180 ? 1 : 0;
  const sweep = spanDeg >= 0 ? 1 : 0;
  return `M ${s.x.toFixed(2)} ${s.y.toFixed(2)} A ${r} ${r} 0 ${large} ${sweep} ${e.x.toFixed(2)} ${e.y.toFixed(2)}`;
}

/**
 * Normalize a raw BiofieldMetrics into the six display values (0..1) shown on
 * the mandala. Most metrics are already ~0..1; the handful that aren't get a
 * light domain map so every arc reads on the same scale.
 */
function normalize(m: BiofieldMetrics) {
  return {
    // VITALITY — overall energy presence (avg intensity carries it)
    vitality: clamp01(m.average_intensity),
    // COHERENCE — inverse of inner noise, the "signal cleanliness"
    coherence: clamp01(1 - m.inner_noise),
    // SYMMETRY — bilateral body symmetry
    symmetry: clamp01(m.body_symmetry),
    // LUMINOSITY — light quanta density
    luminosity: clamp01(m.light_quanta_density),
    // ENTROPY — entropy/form coefficient (already 0..1 band)
    entropy: clamp01(m.entropy_form_coefficient),
    // RHYTHM — temporal/spatial pattern regularity
    rhythm: clamp01(m.pattern_regularity),
  };
}

type MetricKey = keyof ReturnType<typeof normalize>;

interface RingDef {
  key: MetricKey;
  /** maps back to a BiofieldMetricDelta.key when deltas are supplied */
  deltaKey: string;
  label: string;
  /** nested ring radius */
  r: number;
  /** compass position of the arc midpoint */
  centerDeg: number;
  /** how many degrees the full (100%) arc spans */
  span: number;
}

// Nested concentric rings, each metric on its own radius + compass sector,
// matching the staggered multi-ring layout in the spec sheet.
const RINGS: RingDef[] = [
  { key: "vitality", deltaKey: "average_intensity", label: "VITALITY", r: 150, centerDeg: 0, span: 96 },
  { key: "coherence", deltaKey: "inner_noise", label: "COHERENCE", r: 150, centerDeg: 60, span: 96 },
  { key: "rhythm", deltaKey: "pattern_regularity", label: "RHYTHM", r: 132, centerDeg: 132, span: 96 },
  { key: "luminosity", deltaKey: "light_quanta_density", label: "LUMINOSITY", r: 114, centerDeg: 210, span: 96 },
  { key: "symmetry", deltaKey: "body_symmetry", label: "SYMMETRY", r: 132, centerDeg: 288, span: 96 },
  { key: "entropy", deltaKey: "entropy_form_coefficient", label: "ENTROPY", r: 114, centerDeg: 180, span: 96 },
];

export interface ResultMandalaProps {
  metrics: BiofieldMetrics;
  baseline?: BiofieldMetrics | null;
  deltas?: BiofieldMetricDelta[];
  analysisVersion?: string;
  accepted?: boolean;
}

export function ResultMandala({
  metrics,
  baseline = null,
  deltas,
  analysisVersion = "ANL-V0.0.0",
  accepted = true,
}: ResultMandalaProps) {
  const VB = 520;
  const C = VB / 2;
  const GUIDES = [168, 132, 96, 60];

  const reading = useMemo(() => normalize(metrics), [metrics]);
  const baseValues = useMemo(
    () => (baseline ? normalize(baseline) : null),
    [baseline],
  );

  // delta lookup by metric key (when caller supplies pre-computed deltas)
  const deltaByKey = useMemo(() => {
    const map = new Map<string, BiofieldMetricDelta>();
    deltas?.forEach((d) => map.set(d.key, d));
    return map;
  }, [deltas]);

  const coherencePct = Math.round(reading.coherence * 100);

  const rootRef = useRef<SVGSVGElement | null>(null);
  const readingArcRefs = useRef<Array<SVGPathElement | null>>([]);
  const sealRef = useRef<SVGGElement | null>(null);

  // Per-ring geometry: ghost (baseline) arc, reading arc, delta segment.
  const rings = useMemo(() => {
    return RINGS.map((ring) => {
      const start = ring.centerDeg - ring.span / 2;
      const cur = reading[ring.key];

      // Baseline value for the ghost arc. Prefer normalized baseline metrics
      // (correct for inverted metrics like coherence = 1 - inner_noise). Fall
      // back to a supplied delta's baseline_value, else the reading itself
      // (no baseline -> no delta).
      const supplied = deltaByKey.get(ring.deltaKey);
      const base = baseValues
        ? baseValues[ring.key]
        : supplied
          ? clamp01(supplied.baseline_value)
          : cur;

      const curSpan = ring.span * cur;
      const baseSpan = ring.span * base;
      const improved = cur >= base;

      // Track (full faint sector), ghost (baseline fill), reading (current fill).
      const track = arcPath(C, C, ring.r, start, ring.span);
      const ghost = arcPath(C, C, ring.r, start, baseSpan);
      const readingArc = arcPath(C, C, ring.r, start, curSpan);

      // Delta segment: the slice between baseline and reading. Drawn from the
      // shorter end to the longer end so the *difference* is what stands out.
      const lo = Math.min(curSpan, baseSpan);
      const hi = Math.max(curSpan, baseSpan);
      const hasDelta = hi - lo > 0.5;
      const delta = hasDelta ? arcPath(C, C, ring.r, start + lo, hi - lo) : null;

      // label tick just outside the arc midpoint
      const tip = polar(C, C, ring.r + 16, ring.centerDeg);
      const tipAnchor =
        ring.centerDeg > 20 && ring.centerDeg < 160
          ? "start"
          : ring.centerDeg > 200 && ring.centerDeg < 340
            ? "end"
            : "middle";

      return {
        ...ring,
        cur,
        base,
        improved,
        track,
        ghost,
        readingArc,
        delta,
        tip,
        tipAnchor: tipAnchor as "start" | "end" | "middle",
        valueLabel: Math.round(cur * 100).toString().padStart(2, "0"),
      };
    });
  }, [reading, baseValues, deltaByKey, C]);

  const guideDefs = useMemo(
    () => GUIDES.map((r) => ({ r, d: arcPath(C, C, r, 0, 359.9) })),
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [C],
  );

  const sealColor = accepted ? EMERALD : TERRACOTTA;
  const sealWord = accepted ? "ACCEPT" : "REJECTED";
  // octagonal compass-seal points
  const sealPts = useMemo(() => {
    const r = 30;
    return Array.from({ length: 8 }, (_, i) => {
      const p = polar(0, 0, r, (360 / 8) * i + 22.5);
      return `${p.x.toFixed(2)},${p.y.toFixed(2)}`;
    }).join(" ");
  }, []);

  // Mount: draw guides, tween reading arcs over their ghosts, fade verdict seal.
  useEffect(() => {
    const reduce =
      typeof window !== "undefined" &&
      window.matchMedia?.("(prefers-reduced-motion: reduce)").matches;

    const cleanups: Array<() => void> = [];

    if (!reduce) {
      const guides =
        rootRef.current?.querySelectorAll<SVGPathElement>(".rm-guide");
      guides?.forEach((g) => {
        const len = g.getTotalLength();
        g.style.strokeDasharray = `${len}`;
        g.style.strokeDashoffset = `${len}`;
        const a = animate(g, {
          strokeDashoffset: [len, 0],
          duration: 1300,
          ease: "outQuart",
          delay: 100,
        });
        cleanups.push(() => a.pause());
      });
    }

    readingArcRefs.current.forEach((path) => {
      if (!path) return;
      const len = path.getTotalLength();
      path.style.strokeDasharray = `${len}`;
      if (reduce) {
        path.style.strokeDashoffset = "0";
        return;
      }
      path.style.strokeDashoffset = `${len}`;
      const a = animate(path, {
        strokeDashoffset: [len, 0],
        duration: 1200,
        ease: "outExpo",
        delay: 320,
      });
      cleanups.push(() => a.pause());
    });

    if (sealRef.current) {
      if (reduce) {
        sealRef.current.style.opacity = "1";
      } else {
        sealRef.current.style.opacity = "0";
        const a = animate(sealRef.current, {
          opacity: [0, 1],
          scale: [0.9, 1],
          duration: 900,
          ease: "outBack",
          delay: 1100,
        });
        cleanups.push(() => a.pause());
      }
    }

    return () => cleanups.forEach((fn) => fn());
  }, [rings, accepted]);

  return (
    <svg
      ref={rootRef}
      viewBox={`0 0 ${VB} ${VB}`}
      width={VB}
      height={VB}
      preserveAspectRatio="xMidYMid meet"
      role="img"
      aria-label={`Biofield reading mandala, coherence ${coherencePct}, verdict ${sealWord}`}
    >
      <defs>
        <radialGradient id="rm-core" cx="50%" cy="50%" r="50%">
          <stop offset="0%" stopColor={EMERALD} stopOpacity="0.55" />
          <stop offset="55%" stopColor={EMERALD} stopOpacity="0.1" />
          <stop offset="100%" stopColor={EMERALD} stopOpacity="0" />
        </radialGradient>
        <filter id="rm-glow" x="-60%" y="-60%" width="220%" height="220%">
          <feGaussianBlur stdDeviation="2.6" result="b" />
          <feMerge>
            <feMergeNode in="b" />
            <feMergeNode in="SourceGraphic" />
          </feMerge>
        </filter>
      </defs>

      {/* concentric guide rings */}
      {guideDefs.map((g, i) => (
        <path
          key={i}
          className="rm-guide"
          d={g.d}
          fill="none"
          stroke={GOLD}
          strokeWidth={0.6}
          opacity={0.2 - i * 0.035}
        />
      ))}

      {/* radial spokes to the six metric sectors */}
      {RINGS.map((ring) => {
        const p = polar(C, C, 168, ring.centerDeg);
        return (
          <line
            key={`spoke-${ring.key}`}
            x1={C}
            y1={C}
            x2={p.x}
            y2={p.y}
            stroke={VIOLET}
            strokeWidth={0.6}
            opacity={0.5}
          />
        );
      })}

      {/* metric arcs: track -> ghost(baseline) -> delta -> reading */}
      {rings.map((ring, i) => (
        <g key={ring.key}>
          {/* track */}
          <path
            d={ring.track}
            fill="none"
            stroke={SILVER}
            strokeWidth={2}
            opacity={0.1}
            strokeLinecap="round"
          />
          {/* ghost baseline arc */}
          <path
            d={ring.ghost}
            fill="none"
            stroke={PARCHMENT}
            strokeWidth={2}
            opacity={0.16}
            strokeLinecap="round"
          />
          {/* delta segment — emerald for improvement, gold for decline */}
          {ring.delta && (
            <path
              d={ring.delta}
              fill="none"
              stroke={ring.improved ? EMERALD : GOLD}
              strokeWidth={5}
              opacity={0.85}
              strokeLinecap="round"
              filter="url(#rm-glow)"
            />
          )}
          {/* reading arc (animated draw-in) */}
          <path
            ref={(el) => {
              readingArcRefs.current[i] = el;
            }}
            d={ring.readingArc}
            fill="none"
            stroke={ring.improved ? EMERALD : GOLD}
            strokeWidth={3}
            opacity={0.95}
            strokeLinecap="round"
          />
          {/* label + value */}
          <text
            x={ring.tip.x}
            y={ring.tip.y - 4}
            textAnchor={ring.tipAnchor}
            dominantBaseline="middle"
            fontFamily="var(--font-mono, monospace)"
            fontSize={10}
            letterSpacing="1.5"
            fill={PARCHMENT}
            opacity={0.65}
          >
            {ring.label}
          </text>
          <text
            x={ring.tip.x}
            y={ring.tip.y + 10}
            textAnchor={ring.tipAnchor}
            dominantBaseline="middle"
            fontFamily="var(--font-mono, monospace)"
            fontSize={13}
            fontWeight={600}
            fill={ring.improved ? EMERALD : GOLD}
          >
            {ring.valueLabel}
          </text>
        </g>
      ))}

      {/* bioluminescent core + center coherence readout */}
      <circle cx={C} cy={C} r={56} fill="url(#rm-core)" />
      <circle
        cx={C}
        cy={C}
        r={4}
        fill={EMERALD}
        filter="url(#rm-glow)"
      />
      <text
        x={C}
        y={C - 6}
        textAnchor="middle"
        fontFamily="var(--font-mono, monospace)"
        fontSize={46}
        fontWeight={600}
        letterSpacing="-0.02em"
        fill={PARCHMENT}
      >
        {coherencePct}
      </text>
      <text
        x={C}
        y={C + 18}
        textAnchor="middle"
        fontFamily="var(--font-mono, monospace)"
        fontSize={9}
        letterSpacing="3"
        fill={SILVER}
        opacity={0.7}
      >
        COHERENCE
      </text>

      {/* analysis-version label (SF Mono) — top-right block */}
      <text
        x={VB - 16}
        y={26}
        textAnchor="end"
        fontFamily="var(--font-mono, monospace)"
        fontSize={9}
        letterSpacing="2"
        fill={SILVER}
        opacity={0.6}
      >
        ANALYSIS VERSION
      </text>
      <text
        x={VB - 16}
        y={42}
        textAnchor="end"
        fontFamily="var(--font-mono, monospace)"
        fontSize={11}
        letterSpacing="1"
        fill={PARCHMENT}
        opacity={0.85}
      >
        {analysisVersion}
      </text>

      {/* verdict compass-seal — top-right */}
      <g
        ref={sealRef}
        transform={`translate(${VB - 52} 92)`}
        style={{ transformOrigin: `${VB - 52}px 92px` }}
      >
        <polygon
          points={sealPts}
          fill="none"
          stroke={sealColor}
          strokeWidth={1.4}
          opacity={0.55}
        />
        <polygon
          points={sealPts}
          fill="none"
          stroke={sealColor}
          strokeWidth={0.6}
          opacity={0.9}
          transform="scale(0.78)"
          filter="url(#rm-glow)"
        />
        <circle cx={0} cy={0} r={3} fill={sealColor} filter="url(#rm-glow)" />
        <text
          x={0}
          y={0}
          textAnchor="middle"
          dominantBaseline="middle"
          fontFamily="var(--font-mono, monospace)"
          fontSize={accepted ? 9 : 7.5}
          fontWeight={600}
          letterSpacing="1.5"
          fill={sealColor}
          dy={accepted ? 14 : 14}
        >
          {sealWord}
        </text>
      </g>
    </svg>
  );
}

export default ResultMandala;
