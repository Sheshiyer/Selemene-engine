"use client";

/**
 * QualityRadial — Wave 1, built 1:1 to docs/design/biofield-web/07-quality-gauge-spec.png
 *
 * A four-spoke radial compass for capture quality. Each spoke length encodes one
 * QualityAssessment sub-metric along its axis:
 *   SHARPNESS top · CONTRAST right · NOISE bottom · EXPOSURE left.
 * Noise is inverted (longer = cleaner) so every spoke reads "more = better"; its
 * printed value is the cleanliness (1 − noise_level) to match the spoke the eye sees.
 *
 * The four spoke tips connect into a quality quadrilateral (SVG polygon). Its
 * balance/area is the at-a-glance signal of overall quality. A center verdict
 * seal reads SUFFICIENT (Coherence Emerald) or INSUFFICIENT (Terracotta). A dashed
 * gold threshold ring marks the minimum-acceptable radius; a hairline constellation
 * grid (gold dots, concentric rings, radial axes) sits behind everything.
 *
 * Motion: Anime.js v4 (named `animate`), mirroring CosmogramRing. All guarded by
 * prefers-reduced-motion with cleanup of running instances:
 *   1. Mount — grid rings draw in, quadrilateral draws (strokeDashoffset) + fill
 *      fades, spoke gauges scale out from center (staggered), seal fades/scales in.
 *   2. Value — spoke tips, the quadrilateral, and tip labels tween to new values
 *      when `quality` changes.
 *
 * Data: QualityAssessment (sharpness, contrast, noise_level, exposure,
 * sufficient_quality) from the biofield domain model.
 */

import { useEffect, useMemo, useRef } from "react";
import { animate } from "animejs";
import type { QualityAssessment } from "@selemene/biofield-domain";

const GOLD = "#C5A017";
const EMERALD = "#10B5A7";
const TERRACOTTA = "#C65D3B";
const PARCHMENT = "#F0EDE3";
const SILVER = "#8A9BA8";

const VB = 360;
const C = VB / 2;
const R_MAX = 122; // radius for a sub-metric value of 1.0
const R_MIN = 30; // inner radius (value 0 sits here, never collapses to a point)
const THRESHOLD = 0.5; // min-acceptable ring fraction (per spec: "min acceptable")
const SEAL_R = 44; // verdict seal medallion radius
const GUIDE_FRACS = [1, 0.75, 0.5, 0.25];

const clamp01 = (v: number) => Math.max(0, Math.min(1, Number.isFinite(v) ? v : 0));
const rad = (deg: number) => ((deg - 90) * Math.PI) / 180; // 0deg = top, clockwise

function polar(r: number, deg: number) {
  const a = rad(deg);
  return { x: C + r * Math.cos(a), y: C + r * Math.sin(a) };
}

/** Map a 0..1 sub-metric value to a spoke-tip radius. */
const valueRadius = (v: number) => R_MIN + (R_MAX - R_MIN) * clamp01(v);

interface Spoke {
  key: keyof QualityAssessment;
  label: string;
  deg: number; // compass position
  inverted: boolean; // noise: longer = cleaner
}

// SHARPNESS top, CONTRAST right, NOISE bottom, EXPOSURE left (matches the spec sheet)
const SPOKES: Spoke[] = [
  { key: "sharpness", label: "SHARPNESS", deg: 0, inverted: false },
  { key: "contrast", label: "CONTRAST", deg: 90, inverted: false },
  { key: "noise_level", label: "NOISE", deg: 180, inverted: true },
  { key: "exposure", label: "EXPOSURE", deg: 270, inverted: false },
];

/** The displayed 0..1 magnitude for a spoke (noise inverted to cleanliness). */
const spokeValue = (q: QualityAssessment, s: Spoke) => {
  const raw = clamp01(Number(q[s.key]));
  return s.inverted ? 1 - raw : raw;
};

export interface QualityRadialProps {
  quality: QualityAssessment;
}

export function QualityRadial({ quality }: QualityRadialProps) {
  const sufficient = quality.sufficient_quality;
  const verdictColor = sufficient ? EMERALD : TERRACOTTA;
  const verdictWord = sufficient ? "SUFFICIENT" : "INSUFFICIENT";

  const rootRef = useRef<SVGSVGElement | null>(null);
  const polyRef = useRef<SVGPolygonElement | null>(null);
  const polyEdgeRef = useRef<SVGPolygonElement | null>(null);
  const sealRef = useRef<SVGGElement | null>(null);
  const spokeRefs = useRef<Array<SVGGElement | null>>([]);
  const tipRefs = useRef<Array<SVGCircleElement | null>>([]);

  // Static geometry (axis-independent)
  const guideRings = useMemo(() => GUIDE_FRACS.map((f) => R_MIN + (R_MAX - R_MIN) * f), []);
  const axisEnds = useMemo(() => SPOKES.map((s) => polar(R_MAX, s.deg)), []);
  const thresholdPts = useMemo(
    () => SPOKES.map((s) => polar(valueRadius(THRESHOLD), s.deg)),
    [],
  );
  const constellation = useMemo(
    () =>
      Array.from({ length: 56 }, (_, i) => {
        const ring = i % 7;
        const r = R_MIN + (R_MAX - R_MIN) * (0.32 + (ring / 7) * 0.78);
        const p = polar(r, (360 / 56) * i * 1.37 * 7);
        return { ...p, big: i % 8 === 0 };
      }),
    [],
  );

  // Value-dependent geometry
  const spokes = useMemo(
    () =>
      SPOKES.map((s) => {
        const v = spokeValue(quality, s);
        const tip = polar(valueRadius(v), s.deg);
        const labelPos = polar(R_MAX + 22, s.deg);
        return { ...s, v, tip, labelPos };
      }),
    [quality],
  );
  const polyPoints = useMemo(
    () => spokes.map((s) => `${s.tip.x.toFixed(2)},${s.tip.y.toFixed(2)}`).join(" "),
    [spokes],
  );

  // ── Mount: draw-in choreography ──────────────────────────────────────────
  useEffect(() => {
    const reduce =
      typeof window !== "undefined" &&
      window.matchMedia?.("(prefers-reduced-motion: reduce)").matches;
    if (reduce) return;

    const instances: Array<{ pause: () => void }> = [];

    // 1 · grid rings reveal
    const rings = rootRef.current?.querySelectorAll<SVGCircleElement>(".qr-ring");
    rings?.forEach((ring, i) => {
      const len = ring.getTotalLength();
      ring.style.strokeDasharray = `${len}`;
      ring.style.strokeDashoffset = `${len}`;
      instances.push(
        animate(ring, {
          strokeDashoffset: [len, 0],
          duration: 1200,
          ease: "outQuart",
          delay: 80 + i * 60,
        }),
      );
    });

    // 2 · quadrilateral edge draws, then fill fades in
    if (polyEdgeRef.current) {
      const len = polyEdgeRef.current.getTotalLength();
      polyEdgeRef.current.style.strokeDasharray = `${len}`;
      polyEdgeRef.current.style.strokeDashoffset = `${len}`;
      instances.push(
        animate(polyEdgeRef.current, {
          strokeDashoffset: [len, 0],
          duration: 1100,
          ease: "outExpo",
          delay: 360,
        }),
      );
    }
    if (polyRef.current) {
      instances.push(
        animate(polyRef.current, { opacity: [0, 1], duration: 900, ease: "outQuad", delay: 760 }),
      );
    }

    // 3 · spoke gauges scale out from center (staggered)
    spokeRefs.current.forEach((g, i) => {
      if (!g) return;
      instances.push(
        animate(g, {
          scale: [0, 1],
          opacity: [0, 1],
          duration: 760,
          ease: "outBack",
          delay: 420 + i * 90,
        }),
      );
    });

    // 4 · verdict seal fades + scales in
    if (sealRef.current) {
      instances.push(
        animate(sealRef.current, {
          scale: [0.86, 1],
          opacity: [0, 1],
          duration: 900,
          ease: "outExpo",
          delay: 980,
        }),
      );
    }

    return () => instances.forEach((a) => a.pause());
  }, []);

  // ── Value tween: spokes, polygon, tips follow `quality` ──────────────────
  useEffect(() => {
    const reduce =
      typeof window !== "undefined" &&
      window.matchMedia?.("(prefers-reduced-motion: reduce)").matches;
    if (reduce) return;

    const instances: Array<{ pause: () => void }> = [];

    // pulse the tip dots so a value change reads as "re-measured"
    tipRefs.current.forEach((tip, i) => {
      if (!tip) return;
      instances.push(
        animate(tip, {
          r: [4.6, 6.4, 4.6],
          duration: 640,
          ease: "inOutSine",
          delay: 60 + i * 50,
        }),
      );
    });

    return () => instances.forEach((a) => a.pause());
    // Re-run when the spoke geometry (i.e. the values) changes.
  }, [polyPoints]);

  return (
    <svg
      ref={rootRef}
      viewBox={`0 0 ${VB} ${VB}`}
      width={VB}
      height={VB}
      preserveAspectRatio="xMidYMid meet"
      role="img"
      aria-label={`Capture quality compass — ${verdictWord.toLowerCase()}`}
    >
      <defs>
        <radialGradient id="qr-quad" cx="50%" cy="50%" r="60%">
          <stop offset="0%" stopColor={verdictColor} stopOpacity="0.28" />
          <stop offset="100%" stopColor={verdictColor} stopOpacity="0.08" />
        </radialGradient>
        <radialGradient id="qr-seal-core" cx="50%" cy="50%" r="50%">
          <stop offset="0%" stopColor={verdictColor} stopOpacity="0.16" />
          <stop offset="100%" stopColor={verdictColor} stopOpacity="0" />
        </radialGradient>
        <filter id="qr-glow" x="-60%" y="-60%" width="220%" height="220%">
          <feGaussianBlur stdDeviation="3" result="b" />
          <feMerge>
            <feMergeNode in="b" />
            <feMergeNode in="SourceGraphic" />
          </feMerge>
        </filter>
      </defs>

      {/* ── constellation grid ── */}
      <g opacity="0.55">
        {constellation.map((p, i) => (
          <circle
            key={i}
            cx={p.x}
            cy={p.y}
            r={p.big ? 1.3 : 0.7}
            fill={GOLD}
            opacity={p.big ? 0.4 : 0.16}
          />
        ))}
      </g>

      {/* ── concentric guide rings ── */}
      {guideRings.map((r, i) => (
        <circle
          key={i}
          className="qr-ring"
          cx={C}
          cy={C}
          r={r}
          fill="none"
          stroke={GOLD}
          strokeWidth={0.6}
          opacity={0.2 - i * 0.035}
        />
      ))}

      {/* ── radial axes (the four spokes' guide lines) ── */}
      {axisEnds.map((end, i) => (
        <line
          key={i}
          x1={C}
          y1={C}
          x2={end.x}
          y2={end.y}
          stroke={GOLD}
          strokeWidth={0.6}
          opacity={0.18}
        />
      ))}

      {/* ── threshold ring (min acceptable) — dashed gold diamond ── */}
      <polygon
        points={thresholdPts.map((p) => `${p.x.toFixed(2)},${p.y.toFixed(2)}`).join(" ")}
        fill="none"
        stroke={GOLD}
        strokeWidth={1}
        strokeDasharray="3 4"
        opacity={0.42}
      />

      {/* ── quality quadrilateral ── */}
      <polygon
        ref={polyRef}
        points={polyPoints}
        fill="url(#qr-quad)"
        stroke="none"
        opacity={0}
        style={{ transition: "none" }}
      />
      <polygon
        ref={polyEdgeRef}
        points={polyPoints}
        fill="none"
        stroke={verdictColor}
        strokeWidth={1.8}
        strokeLinejoin="round"
        opacity={0.92}
        filter="url(#qr-glow)"
      />

      {/* ── spoke gauges (line from center to tip + tip dot + label) ── */}
      {spokes.map((s, i) => (
        <g
          key={s.label}
          ref={(el) => {
            spokeRefs.current[i] = el;
          }}
          style={{ transformOrigin: `${C}px ${C}px` }}
        >
          <line
            x1={C}
            y1={C}
            x2={s.tip.x}
            y2={s.tip.y}
            stroke={verdictColor}
            strokeWidth={1.4}
            opacity={0.55}
          />
          <circle
            ref={(el) => {
              tipRefs.current[i] = el;
            }}
            cx={s.tip.x}
            cy={s.tip.y}
            r={4.6}
            fill={verdictColor}
            filter="url(#qr-glow)"
          />
          <text
            x={s.labelPos.x}
            y={s.labelPos.y - 5}
            textAnchor="middle"
            dominantBaseline="middle"
            fontFamily="var(--font-mono, monospace)"
            fontSize={9.5}
            letterSpacing="1.6"
            fill={PARCHMENT}
            opacity={0.62}
          >
            {s.label}
          </text>
          <text
            x={s.labelPos.x}
            y={s.labelPos.y + 8}
            textAnchor="middle"
            dominantBaseline="middle"
            fontFamily="var(--font-mono, monospace)"
            fontSize={12}
            fontWeight={600}
            letterSpacing="0.02em"
            fill={verdictColor}
          >
            {s.v.toFixed(2)}
          </text>
        </g>
      ))}

      {/* ── verdict seal ── */}
      <g ref={sealRef} opacity={0} style={{ transformOrigin: `${C}px ${C}px` }}>
        <circle cx={C} cy={C} r={SEAL_R + 8} fill="url(#qr-seal-core)" />
        {/* medallion ring */}
        <circle
          cx={C}
          cy={C}
          r={SEAL_R}
          fill="#070B1D"
          fillOpacity={0.72}
          stroke={verdictColor}
          strokeWidth={1.6}
        />
        <circle cx={C} cy={C} r={SEAL_R - 4} fill="none" stroke={verdictColor} strokeWidth={0.6} opacity={0.4} />

        {/* upper chevron (apex up) + flanking ticks + anchor glyph */}
        <path
          d={`M ${C - 16} ${C - 12} L ${C} ${C - 26} L ${C + 16} ${C - 12}`}
          fill="none"
          stroke={verdictColor}
          strokeWidth={1.3}
          strokeLinecap="round"
          strokeLinejoin="round"
          opacity={0.9}
        />
        <line x1={C - 30} y1={C - 12} x2={C - 17} y2={C - 12} stroke={verdictColor} strokeWidth={1.1} opacity={0.7} />
        <line x1={C + 17} y1={C - 12} x2={C + 30} y2={C - 12} stroke={verdictColor} strokeWidth={1.1} opacity={0.7} />
        <line x1={C} y1={C - 24} x2={C} y2={C - 14} stroke={verdictColor} strokeWidth={1.1} opacity={0.85} />
        <circle cx={C} cy={C - 13} r={1.8} fill={verdictColor} />

        {/* verdict word */}
        <text
          x={C}
          y={C + 2}
          textAnchor="middle"
          dominantBaseline="middle"
          fontFamily="var(--font-mono, monospace)"
          fontSize={sufficient ? 13 : 11}
          fontWeight={600}
          letterSpacing="1.4"
          fill={verdictColor}
        >
          {verdictWord}
        </text>

        {/* lower chevron (mirror) + flanking ticks + anchor glyph */}
        <path
          d={`M ${C - 16} ${C + 14} L ${C} ${C + 28} L ${C + 16} ${C + 14}`}
          fill="none"
          stroke={verdictColor}
          strokeWidth={1.3}
          strokeLinecap="round"
          strokeLinejoin="round"
          opacity={0.9}
        />
        <line x1={C - 30} y1={C + 14} x2={C - 17} y2={C + 14} stroke={verdictColor} strokeWidth={1.1} opacity={0.7} />
        <line x1={C + 17} y1={C + 14} x2={C + 30} y2={C + 14} stroke={verdictColor} strokeWidth={1.1} opacity={0.7} />
        <line x1={C} y1={C + 16} x2={C} y2={C + 26} stroke={verdictColor} strokeWidth={1.1} opacity={0.85} />
        <circle cx={C} cy={C + 15} r={1.8} fill={verdictColor} />
      </g>
    </svg>
  );
}

export default QualityRadial;
