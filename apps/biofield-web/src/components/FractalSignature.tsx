"use client";

/**
 * FractalSignature — Wave 1, built 1:1 to docs/design/biofield-web/06-fractal-chaos-spec.png
 *
 * A luminous self-similar strange attractor (Lorenz) drawn in Sacred Gold over a faint
 * Flow Indigo phase-space grid, with a hairline constellation field behind everything.
 * Below the plot: three large SF-Mono readouts (Fractal Dimension, Correlation Dimension,
 * Entropy). At the foot: an interpretive scale labelled ordered -> complex -> chaotic with
 * a Coherence Emerald marker whose position is derived from the metric values.
 *
 * Motion: Anime.js v4 (named `animate`). Two behaviours, both guarded by
 * prefers-reduced-motion and cleaned up on unmount:
 *   1. Mount — the attractor path draws itself (strokeDashoffset reveal).
 *   2. Value — the scale marker tweens (translateX) to its derived position.
 *
 * Data: BiofieldMetrics — fractal_dimension, correlation_dimension, entropy_form_coefficient.
 * Deterministic by construction (fixed Lorenz seed, fixed iteration, seeded dot field) so
 * the SSR and client markup match exactly.
 */

import { useEffect, useMemo, useRef } from "react";
import { animate } from "animejs";

const GOLD = "#C5A017"; // Sacred Gold — the attractor
const EMERALD = "#10B5A7"; // Coherence Emerald — the scale marker
const INDIGO = "#0B50FB"; // Flow Indigo — phase-space grid
const PARCHMENT = "#F0EDE3"; // primary text / readout values
const SILVER = "#8A9BA8"; // muted labels

const clamp01 = (v: number) => Math.max(0, Math.min(1, v));

// Hero canvas — wider than tall, matching the spec panel.
const VB_W = 600;
const VB_H = 560;

// Plot rectangle (phase-space window) inside the canvas.
const PLOT = { x: 70, y: 40, w: 460, h: 300 };

/** Deterministic PRNG (mulberry32) — keeps the constellation identical SSR/CSR. */
function mulberry32(seed: number) {
  let a = seed >>> 0;
  return () => {
    a |= 0;
    a = (a + 0x6d2b79f5) | 0;
    let t = Math.imul(a ^ (a >>> 15), 1 | a);
    t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t;
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}

/**
 * Integrate the Lorenz system to a fixed point set, project (x, z) to 2D, and
 * normalise into the plot rect. Pure + deterministic: same input → same path.
 */
function lorenzPath(): string {
  const sigma = 10;
  const rho = 28;
  const beta = 8 / 3;
  const dt = 0.006;
  const steps = 2600;
  const warmup = 60;

  let x = 0.1;
  let y = 0;
  let z = 0;

  const pts: Array<[number, number]> = [];
  for (let i = 0; i < steps; i += 1) {
    const dx = sigma * (y - x);
    const dy = x * (rho - z) - y;
    const dz = x * y - beta * z;
    x += dx * dt;
    y += dy * dt;
    z += dz * dt;
    if (i >= warmup) pts.push([x, z]);
  }

  let minX = Infinity;
  let maxX = -Infinity;
  let minZ = Infinity;
  let maxZ = -Infinity;
  for (const [px, pz] of pts) {
    if (px < minX) minX = px;
    if (px > maxX) maxX = px;
    if (pz < minZ) minZ = pz;
    if (pz > maxZ) maxZ = pz;
  }

  const pad = 16;
  const sx = (PLOT.w - pad * 2) / (maxX - minX);
  const sz = (PLOT.h - pad * 2) / (maxZ - minZ);

  return pts
    .map(([px, pz], i) => {
      const cx = PLOT.x + pad + (px - minX) * sx;
      // invert z so the attractor sits upright in screen space
      const cy = PLOT.y + pad + (PLOT.h - pad * 2 - (pz - minZ) * sz);
      return `${i === 0 ? "M" : "L"} ${cx.toFixed(2)} ${cy.toFixed(2)}`;
    })
    .join(" ");
}

export interface FractalSignatureProps {
  fractalDimension: number;
  correlationDimension: number;
  entropy: number;
}

export function FractalSignature({
  fractalDimension,
  correlationDimension,
  entropy,
}: FractalSignatureProps) {
  const pathRef = useRef<SVGPathElement | null>(null);
  const markerRef = useRef<SVGGElement | null>(null);

  // Geometry that never depends on live values — computed once.
  const attractor = useMemo(() => lorenzPath(), []);
  const gridLines = useMemo(() => {
    const cols = 8;
    const rows = 5;
    const v = Array.from({ length: cols + 1 }, (_, i) => PLOT.x + (PLOT.w / cols) * i);
    const h = Array.from({ length: rows + 1 }, (_, i) => PLOT.y + (PLOT.h / rows) * i);
    return { v, h };
  }, []);
  const constellation = useMemo(() => {
    const rnd = mulberry32(0x5e1e3e);
    return Array.from({ length: 90 }, () => {
      const cx = rnd() * VB_W;
      const cy = rnd() * VB_H;
      const big = rnd() > 0.86;
      return { cx, cy, r: big ? 1.2 : 0.6, o: big ? 0.32 : 0.14 };
    });
  }, []);

  // Scale geometry + derived marker position.
  const scaleY = 500;
  const scaleX0 = 90;
  const scaleX1 = VB_W - 90;
  const scaleW = scaleX1 - scaleX0;
  // Fractal dimension over [1,2] is the primary driver; entropy nudges it.
  const markerT = clamp01(0.7 * (fractalDimension - 1) + 0.3 * entropy);
  const markerX = scaleX0 + markerT * scaleW;

  const readouts = useMemo(
    () => [
      { label: "FRACTAL DIMENSION", value: fractalDimension },
      { label: "CORRELATION DIMENSION", value: correlationDimension },
      { label: "ENTROPY", value: entropy },
    ],
    [fractalDimension, correlationDimension, entropy],
  );

  const prefersReduced = () =>
    typeof window !== "undefined" &&
    window.matchMedia?.("(prefers-reduced-motion: reduce)").matches;

  // Mount: the attractor draws itself. Runs once.
  useEffect(() => {
    const path = pathRef.current;
    if (!path) return;
    const len = path.getTotalLength();
    path.style.strokeDasharray = `${len}`;
    if (prefersReduced()) {
      path.style.strokeDashoffset = "0";
      return;
    }
    path.style.strokeDashoffset = `${len}`;
    const a = animate(path, {
      strokeDashoffset: [len, 0],
      duration: 2400,
      ease: "outQuart",
      delay: 120,
    });
    return () => {
      a.pause();
    };
  }, []);

  // Value: the marker tweens to its derived scale position.
  useEffect(() => {
    const marker = markerRef.current;
    if (!marker) return;
    if (prefersReduced()) {
      marker.style.transform = `translateX(${markerX}px)`;
      return;
    }
    const a = animate(marker, {
      translateX: [scaleX0, markerX],
      duration: 1200,
      ease: "outExpo",
      delay: 600,
    });
    return () => {
      a.pause();
    };
  }, [markerX]);

  const f2 = (v: number) => (Number.isFinite(v) ? v.toFixed(2) : "—");

  return (
    <svg
      viewBox={`0 0 ${VB_W} ${VB_H}`}
      width={VB_W}
      height={VB_H}
      preserveAspectRatio="xMidYMid meet"
      role="img"
      aria-label={`Fractal chaos signature, fractal dimension ${f2(fractalDimension)}`}
    >
      <defs>
        <filter id="fs-glow" x="-20%" y="-20%" width="140%" height="140%">
          <feGaussianBlur stdDeviation="2.2" result="b" />
          <feMerge>
            <feMergeNode in="b" />
            <feMergeNode in="SourceGraphic" />
          </feMerge>
        </filter>
        <filter id="fs-marker-glow" x="-80%" y="-80%" width="260%" height="260%">
          <feGaussianBlur stdDeviation="2.6" result="b" />
          <feMerge>
            <feMergeNode in="b" />
            <feMergeNode in="SourceGraphic" />
          </feMerge>
        </filter>
      </defs>

      {/* hairline constellation field, behind everything */}
      <g>
        {constellation.map((p, i) => (
          <circle key={i} cx={p.cx} cy={p.cy} r={p.r} fill={GOLD} opacity={p.o} />
        ))}
      </g>

      {/* faint Flow Indigo phase-space grid */}
      <g opacity={0.5}>
        {gridLines.v.map((vx, i) => (
          <line
            key={`v${i}`}
            x1={vx}
            y1={PLOT.y}
            x2={vx}
            y2={PLOT.y + PLOT.h}
            stroke={INDIGO}
            strokeWidth={0.6}
            opacity={0.12}
          />
        ))}
        {gridLines.h.map((hy, i) => (
          <line
            key={`h${i}`}
            x1={PLOT.x}
            y1={hy}
            x2={PLOT.x + PLOT.w}
            y2={hy}
            stroke={INDIGO}
            strokeWidth={0.6}
            opacity={0.12}
          />
        ))}
        {/* plot frame */}
        <rect
          x={PLOT.x}
          y={PLOT.y}
          width={PLOT.w}
          height={PLOT.h}
          fill="none"
          stroke={INDIGO}
          strokeWidth={0.8}
          opacity={0.22}
        />
      </g>

      {/* the strange attractor — Sacred Gold, bioluminescent */}
      <path
        ref={pathRef}
        d={attractor}
        fill="none"
        stroke={GOLD}
        strokeWidth={1.1}
        strokeLinecap="round"
        strokeLinejoin="round"
        opacity={0.88}
        filter="url(#fs-glow)"
      />

      {/* three SF-Mono readouts */}
      {readouts.map((r, i) => {
        const colW = VB_W / 3;
        const cx = colW * i + colW / 2;
        return (
          <g key={r.label}>
            <text
              x={cx}
              y={398}
              textAnchor="middle"
              fontFamily="var(--font-mono, monospace)"
              fontSize={10}
              letterSpacing="1.4"
              fill={SILVER}
              opacity={0.8}
            >
              {r.label}
            </text>
            <text
              x={cx}
              y={438}
              textAnchor="middle"
              fontFamily="var(--font-mono, monospace)"
              fontSize={38}
              fontWeight={600}
              letterSpacing="-0.02em"
              fill={PARCHMENT}
            >
              {f2(r.value)}
            </text>
          </g>
        );
      })}

      {/* interpretive scale: ordered -> complex -> chaotic */}
      <g>
        {/* track */}
        <line
          x1={scaleX0}
          y1={scaleY}
          x2={scaleX1}
          y2={scaleY}
          stroke={SILVER}
          strokeWidth={1}
          opacity={0.3}
        />
        {/* tick marks at thirds */}
        {[0, 0.5, 1].map((t) => (
          <line
            key={t}
            x1={scaleX0 + t * scaleW}
            y1={scaleY - 4}
            x2={scaleX0 + t * scaleW}
            y2={scaleY + 4}
            stroke={SILVER}
            strokeWidth={1}
            opacity={0.4}
          />
        ))}
        {/* labels */}
        <text
          x={scaleX0}
          y={scaleY + 24}
          textAnchor="start"
          fontFamily="var(--font-mono, monospace)"
          fontSize={11}
          letterSpacing="1"
          fill={SILVER}
          opacity={0.75}
        >
          ordered
        </text>
        <text
          x={scaleX0 + scaleW / 2}
          y={scaleY + 24}
          textAnchor="middle"
          fontFamily="var(--font-mono, monospace)"
          fontSize={11}
          letterSpacing="1"
          fill={SILVER}
          opacity={0.75}
        >
          complex
        </text>
        <text
          x={scaleX1}
          y={scaleY + 24}
          textAnchor="end"
          fontFamily="var(--font-mono, monospace)"
          fontSize={11}
          letterSpacing="1"
          fill={SILVER}
          opacity={0.75}
        >
          chaotic
        </text>
        {/* Coherence Emerald marker — translated into place by anime.js.
            Drawn at local origin; the group's transform sets x. */}
        <g
          ref={markerRef}
          style={{ transform: `translateX(${scaleX0}px)` }}
        >
          <polygon
            points={`0,${scaleY - 12} -6,${scaleY - 2} 6,${scaleY - 2}`}
            fill={EMERALD}
            filter="url(#fs-marker-glow)"
          />
          <line
            x1={0}
            y1={scaleY - 2}
            x2={0}
            y2={scaleY + 2}
            stroke={EMERALD}
            strokeWidth={1.5}
          />
        </g>
      </g>
    </svg>
  );
}

export default FractalSignature;
