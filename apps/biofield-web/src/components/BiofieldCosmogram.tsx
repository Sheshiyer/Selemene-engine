"use client";

/**
 * BiofieldCosmogram — WitnessOS sacred geometry consciousness display
 *
 * Pure SVG hand-crafted geometry — zero chart dependencies.
 * Renders:
 *   1. Lotus mandala ornament  — thin gold teardrop petals + dot rings (decorative bg)
 *   2. Coherence ring          — stroke-dasharray progress arc + state word (OPTIMAL/BUILDING/ATTUNING)
 *   3. Chakra dot orbit        — 7 colored orbs on inner ring
 *   4. Floating metric glyphs  — ENERGY / PRESENCE / SYMMETRY / PATTERN corner labels
 *   5. Biorhythm wave strip    — gradient sine wave (CHAOS→FLOW) with animated leading dot
 *
 * References: witnessOS-sw/breathnav.png, hrv.png, dashboard.png
 * Brand: Kha-Ba-La spectrum — Void Black / Sacred Gold / Coherence Emerald / Flow Indigo
 */

import React, { useMemo } from "react";
import type { CompositeScores } from "./pip/types";

// ── Types ─────────────────────────────────────────────────────────────────────
interface ChakraReading {
  chakra: string;
  chakra_name?: string;
  activity_level: number;
  balance: number;
  color_intensity: number;
}

// ── Brand tokens (SVG attributes cannot use CSS vars directly) ────────────────
const GOLD      = "#C5A017";
const EMERALD   = "#10B5A7";
const INDIGO    = "#0B50FB";
const PARCHMENT = "#F0EDE3";

// ── SVG layout constants ──────────────────────────────────────────────────────
const VW = 320;
const VH = 290;
const CX = VW / 2;   // 160
const CY = 140;      // center Y (elevated to leave wave room below)

const RING_R = 56;
const RING_C = 2 * Math.PI * RING_R;  // ≈ 351.9

// Lotus ornament
const PETAL_LEN  = 80;
const PETAL_W    = 26;
const IPETAL_LEN = 54;
const IPETAL_W   = 18;

const OUTER_R = 92;  // outermost decorative ring

// Wave strip
const WAVE_Y = 244;
const WAVE_H = 30;

// ── Helpers ───────────────────────────────────────────────────────────────────
function toRad(d: number) { return (d * Math.PI) / 180; }
function clamp(v: number, lo = 0, hi = 1) { return Math.max(lo, Math.min(hi, v)); }
function f(n: number) { return n.toFixed(2); }

/** Teardrop lotus petal via two quadratic bezier curves */
function petalPath(angleDeg: number, len: number, wid: number): string {
  const a = toRad(angleDeg);
  const p = a + Math.PI / 2;
  const tx = CX + len * Math.cos(a),   ty = CY + len * Math.sin(a);
  const ax = CX + len * 0.62 * Math.cos(a) + wid * Math.cos(p);
  const ay = CY + len * 0.62 * Math.sin(a) + wid * Math.sin(p);
  const bx = CX + len * 0.62 * Math.cos(a) - wid * Math.cos(p);
  const by = CY + len * 0.62 * Math.sin(a) - wid * Math.sin(p);
  return `M${CX},${CY} Q${f(ax)},${f(ay)} ${f(tx)},${f(ty)} Q${f(bx)},${f(by)} ${CX},${CY} Z`;
}

/** Evenly spaced points on a circle */
function circlePoints(r: number, n: number, offsetDeg = -90) {
  return Array.from({ length: n }, (_, i) => {
    const a = toRad((360 / n) * i + offsetDeg);
    return { x: CX + r * Math.cos(a), y: CY + r * Math.sin(a) };
  });
}

/** Sine wave SVG path */
function buildWave(xStart: number, width: number, yMid: number, amp: number, cycles: number): string {
  const pts: string[] = [];
  const steps = 120;
  for (let i = 0; i <= steps; i++) {
    const t = i / steps;
    const x = xStart + t * width;
    const y = yMid + amp * Math.sin(t * cycles * 2 * Math.PI);
    pts.push(`${i === 0 ? "M" : "L"}${x.toFixed(1)},${y.toFixed(1)}`);
  }
  return pts.join(" ");
}

// ── Chakra metadata ───────────────────────────────────────────────────────────
const CHAKRA_META: Record<string, { color: string }> = {
  Crown:       { color: "#9B59B6" },
  ThirdEye:    { color: "#6B21D4" },
  Throat:      { color: INDIGO    },
  Heart:       { color: EMERALD   },
  SolarPlexus: { color: GOLD      },
  Sacral:      { color: "#E8884A" },
  Root:        { color: "#C84B31" },
};
const CHAKRA_ORDER = ["Crown", "ThirdEye", "Throat", "Heart", "SolarPlexus", "Sacral", "Root"];

// ── Floating metric positions ─────────────────────────────────────────────────
type TA = "start" | "end" | "middle";
const METRICS: Array<{ key: keyof CompositeScores; label: string; ax: number; ay: number; ta: TA }> = [
  { key: "lightQuantaDensity", label: "ENERGY",   ax: 22,        ay: 88,  ta: "start" },
  { key: "normalizedArea",     label: "PRESENCE", ax: 22,        ay: 200, ta: "start" },
  { key: "bodySymmetry",       label: "SYMMETRY", ax: VW - 22,   ay: 88,  ta: "end"   },
  { key: "patternRegularity",  label: "PATTERN",  ax: VW - 22,   ay: 200, ta: "end"   },
];

// ═══════════════════════════════════════════════════════════════════════════════
// Component
// ═══════════════════════════════════════════════════════════════════════════════
export interface BiofieldCosmogramProps {
  scores: CompositeScores;
  /** result.* from noesis biofield engine EngineOutput — provides chakra_readings */
  engineResult?: Record<string, unknown> | null;
}

export function BiofieldCosmogram({ scores, engineResult }: BiofieldCosmogramProps) {
  const coherence = clamp(scores.overallCoherence);

  const state      = coherence >= 0.75 ? "OPTIMAL"  : coherence >= 0.5 ? "BUILDING"  : "ATTUNING";
  const stateColor = coherence >= 0.75 ? EMERALD    : coherence >= 0.5 ? INDIGO      : GOLD;

  const dash = coherence * RING_C;
  const gap  = RING_C - dash;

  const chakras = useMemo<ChakraReading[]>(() => {
    const cr = engineResult?.chakra_readings;
    if (Array.isArray(cr)) {
      return CHAKRA_ORDER.map((name) =>
        (cr as ChakraReading[]).find((r) => r.chakra === name || r.chakra_name === name) ?? {
          chakra: name, activity_level: 0.4, balance: 0, color_intensity: 0.3,
        }
      );
    }
    return CHAKRA_ORDER.map((name, i) => ({
      chakra: name,
      activity_level: clamp(coherence * 0.55 + scores.lightQuantaDensity * 0.35 + (i % 3) * 0.07),
      balance: 0, color_intensity: scores.lightQuantaDensity,
    }));
  }, [engineResult, coherence, scores.lightQuantaDensity]);

  const wavePath = useMemo(
    () => buildWave(22, VW - 44, WAVE_Y + WAVE_H / 2, 10.5, 3.5),
    []
  );

  return (
    <section style={{ width: "100%", height: "100%", display: "flex", background: "transparent" }}>
      <svg
        viewBox={`0 0 ${VW} ${VH}`}
        preserveAspectRatio="xMidYMid meet"
        width="100%"
        height="100%"
        overflow="visible"
        aria-label="Biofield cosmogram"
      >
        <defs>
          <filter id="coh-glow" x="-50%" y="-50%" width="200%" height="200%">
            <feGaussianBlur stdDeviation="4.5" result="blur" />
            <feMerge><feMergeNode in="blur" /><feMergeNode in="SourceGraphic" /></feMerge>
          </filter>
          <filter id="soft-glow" x="-40%" y="-40%" width="180%" height="180%">
            <feGaussianBlur stdDeviation="2.5" result="blur" />
            <feMerge><feMergeNode in="blur" /><feMergeNode in="SourceGraphic" /></feMerge>
          </filter>
          <filter id="text-glow" x="-25%" y="-50%" width="150%" height="200%">
            <feGaussianBlur stdDeviation="3" result="blur" />
            <feMerge><feMergeNode in="blur" /><feMergeNode in="SourceGraphic" /></feMerge>
          </filter>

          <radialGradient id="center-aura" cx="50%" cy="50%" r="50%">
            <stop offset="0%"   stopColor={stateColor} stopOpacity="0.16" />
            <stop offset="60%"  stopColor={stateColor} stopOpacity="0.04" />
            <stop offset="100%" stopColor={stateColor} stopOpacity="0"    />
          </radialGradient>

          <linearGradient id="wave-grad" x1="0%" y1="0%" x2="100%" y2="0%">
            <stop offset="0%"   stopColor="#C84B31" stopOpacity="0.9" />
            <stop offset="28%"  stopColor={GOLD}    stopOpacity="0.95" />
            <stop offset="62%"  stopColor={EMERALD} stopOpacity="1" />
            <stop offset="100%" stopColor={INDIGO}  stopOpacity="0.9" />
          </linearGradient>

          <clipPath id="wave-clip">
            <rect x="22" y={WAVE_Y} width={VW - 44} height={WAVE_H} />
          </clipPath>

          <style>{`
            .cosmo-pulse { animation: cosmo-pulse 3.2s ease-in-out infinite; }
            @keyframes cosmo-pulse { 0%,100%{opacity:1} 50%{opacity:0.72} }
          `}</style>
        </defs>

        {/* Ambient center radial glow */}
        <ellipse cx={CX} cy={CY} rx={RING_R * 1.45} ry={RING_R * 1.35} fill="url(#center-aura)" />

        {/* Lotus ornament — outer petals (8 × 45°) */}
        {Array.from({ length: 8 }, (_, i) => (
          <path
            key={`po-${i}`}
            d={petalPath(i * 45 - 90, PETAL_LEN, PETAL_W)}
            fill="none" stroke={GOLD} strokeWidth={0.65} opacity={0.155}
          />
        ))}

        {/* Lotus ornament — inner petals (8, offset 22.5°) */}
        {Array.from({ length: 8 }, (_, i) => (
          <path
            key={`pi-${i}`}
            d={petalPath(i * 45 - 67.5, IPETAL_LEN, IPETAL_W)}
            fill="none" stroke={GOLD} strokeWidth={0.5} opacity={0.09}
          />
        ))}

        {/* Spider-web concentric guide rings */}
        {[0.36, 0.57, 0.79, 1.0].map((frac, ri) => (
          <circle
            key={`wr-${ri}`}
            cx={CX} cy={CY} r={OUTER_R * frac}
            fill="none" stroke={GOLD} strokeWidth={0.4}
            strokeDasharray={frac >= 0.95 ? "2.5 8.5" : "1.5 6"}
            opacity={frac >= 0.95 ? 0.21 : 0.08}
          />
        ))}

        {/* Radial spokes (12 × 30°) */}
        {Array.from({ length: 12 }, (_, i) => {
          const a = toRad(i * 30 - 90);
          return (
            <line
              key={`sp-${i}`}
              x1={CX + 20 * Math.cos(a)} y1={CY + 20 * Math.sin(a)}
              x2={CX + OUTER_R * Math.cos(a)} y2={CY + OUTER_R * Math.sin(a)}
              stroke={GOLD}
              strokeWidth={i % 3 === 0 ? 0.5 : 0.25}
              opacity={i % 3 === 0 ? 0.18 : 0.07}
            />
          );
        })}

        {/* Outer ring intersection dots */}
        {circlePoints(OUTER_R, 24).map((p, i) => (
          <circle key={`d1-${i}`} cx={p.x} cy={p.y} r={0.85} fill={GOLD} opacity={0.30} />
        ))}
        {circlePoints(OUTER_R * 0.57, 16).map((p, i) => (
          <circle key={`d2-${i}`} cx={p.x} cy={p.y} r={0.6} fill={GOLD} opacity={0.13} />
        ))}

        {/* Coherence ring — background track */}
        <circle cx={CX} cy={CY} r={RING_R} fill="none" stroke="rgba(197,160,23,0.07)" strokeWidth={3.5} />

        {/* Coherence ring — filled arc */}
        <circle
          cx={CX} cy={CY} r={RING_R}
          fill="none"
          stroke={stateColor}
          strokeWidth={3.5}
          strokeDasharray={`${dash.toFixed(1)} ${gap.toFixed(1)}`}
          strokeLinecap="round"
          transform={`rotate(-90 ${CX} ${CY})`}
          filter="url(#coh-glow)"
          style={{ transition: "stroke-dasharray 0.95s cubic-bezier(0.34,1.56,0.64,1), stroke 0.6s ease" }}
        />

        {/* Cardinal tick marks */}
        {[0, 90, 180, 270].map((deg) => {
          const a = toRad(deg - 90);
          const r = RING_R + 6.5;
          return (
            <circle key={`cm-${deg}`}
              cx={CX + r * Math.cos(a)} cy={CY + r * Math.sin(a)}
              r={1.5} fill={GOLD} opacity={0.42}
            />
          );
        })}

        {/* "COHERENCE" small label */}
        <text
          x={CX} y={CY - 15}
          textAnchor="middle"
          fontFamily="var(--font-display, 'Panchang', monospace)"
          fontSize={5.8} fontWeight={700} letterSpacing={2.8}
          fill={GOLD} opacity={0.42}
        >
          COHERENCE
        </text>

        {/* Primary state word */}
        <text
          x={CX} y={CY + 7}
          textAnchor="middle"
          fontFamily="var(--font-display, 'Panchang', monospace)"
          fontSize={16} fontWeight={800} letterSpacing={3.5}
          fill={stateColor}
          filter="url(#text-glow)"
          className="cosmo-pulse"
          style={{ transition: "fill 0.65s ease" }}
        >
          {state}
        </text>

        {/* Numeric coherence */}
        <text
          x={CX} y={CY + 23}
          textAnchor="middle"
          fontFamily="var(--font-mono, monospace)"
          fontSize={8.5} fill={PARCHMENT} opacity={0.36}
        >
          {Math.round(coherence * 100)}%
        </text>

        {/* Chakra dot orbit (inner ring at r ≈ RING_R × 0.43) */}
        {chakras.map((reading, i) => {
          const meta = CHAKRA_META[reading.chakra] ?? CHAKRA_META["Heart"];
          const act  = clamp(reading.activity_level);
          const a    = toRad((360 / 7) * i - 90);
          const r    = RING_R * 0.43;
          return (
            <circle
              key={`ch-${i}`}
              cx={CX + r * Math.cos(a)} cy={CY + r * Math.sin(a)}
              r={2 + act * 2.5}
              fill={meta.color}
              opacity={0.42 + act * 0.48}
              style={{ filter: act > 0.55 ? `drop-shadow(0 0 3px ${meta.color})` : "none" }}
            />
          );
        })}

        {/* Floating metric glyphs */}
        {METRICS.map(({ key, label, ax, ay, ta }) => {
          const v      = clamp(scores[key]);
          const pct    = Math.round(v * 100);
          const active = pct >= 48;
          return (
            <g key={key}>
              <text
                x={ax} y={ay}
                textAnchor={ta}
                fontFamily="var(--font-display, 'Panchang', monospace)"
                fontSize={5.5} fontWeight={700} letterSpacing={1.9}
                fill={GOLD} opacity={active ? 0.60 : 0.26}
              >
                {label}
              </text>
              <text
                x={ax} y={ay + 14}
                textAnchor={ta}
                fontFamily="var(--font-mono, monospace)"
                fontSize={14} fontWeight={700}
                fill={PARCHMENT} opacity={active ? 0.84 : 0.25}
              >
                {pct}
                <tspan fontSize={7} opacity={0.45} fontWeight={400}>%</tspan>
              </text>
            </g>
          );
        })}

        {/* Wave separator */}
        <line
          x1={30} y1={WAVE_Y - 6} x2={VW - 30} y2={WAVE_Y - 6}
          stroke={GOLD} strokeWidth={0.3} opacity={0.18}
        />

        {/* Biorhythm wave strip */}
        <g clipPath="url(#wave-clip)">
          <path d={wavePath} fill="none" stroke={stateColor} strokeWidth={8} opacity={0.06} />
          <path d={wavePath} fill="none" stroke="url(#wave-grad)" strokeWidth={1.5} />
          <circle r={2.5} fill={stateColor} opacity={0.9} filter="url(#soft-glow)">
            <animateMotion dur="5.5s" repeatCount="indefinite" path={wavePath} />
          </circle>
        </g>

        <text
          x={30} y={WAVE_Y + WAVE_H + 10}
          textAnchor="start"
          fontFamily="var(--font-display, 'Panchang', monospace)"
          fontSize={4.8} fill={GOLD} opacity={0.36} letterSpacing={2}
        >
          BIORHYTHM
        </text>
        <text
          x={VW - 30} y={WAVE_Y + WAVE_H + 10}
          textAnchor="end"
          fontFamily="var(--font-display, 'Panchang', monospace)"
          fontSize={4.8} fill={stateColor} opacity={0.68} letterSpacing={1.5}
        >
          {state}
        </text>
      </svg>
    </section>
  );
}
