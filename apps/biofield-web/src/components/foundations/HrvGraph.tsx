"use client";

/**
 * HrvGraph — Wave 1, built 1:1 to docs/design/biofield-web/11-foundations-spec.png
 * (section 2 · GRAPH).
 *
 * HRV / metrics history rendered as a brand WAVEFORM OVERLAY crossing a faint
 * sacred-geometry frame — NOT a boxed line chart. The waveform stroke is
 * gradient-coded across the coherence band:
 *   chaos (Terracotta) -> optimal (Coherence Emerald) -> flow (Flow Indigo).
 * A slower 4:7:8 breath waveform sits beneath it. Axis labels are SF-Mono.
 *
 * Motion: Anime.js v4 (named `animate`), guarded by prefers-reduced-motion
 * and cleaned up on unmount. Mirrors CosmogramRing.tsx:
 *   1. Mount  — the HRV waveform draws itself (strokeDashoffset reveal).
 *   2. Breath — the 4:7:8 waveform glides left->right on the inhale/hold/exhale
 *               cadence (translateX loop), a calm carrier beneath the data.
 */

import { useEffect, useMemo, useRef } from "react";
import { animate } from "animejs";

const TERRACOTTA = "#C65D3B"; // chaos
const EMERALD = "#10B5A7"; // optimal
const INDIGO = "#0B50FB"; // flow
const GOLD = "#C5A017";
const PARCHMENT = "#F0EDE3";
const SILVER = "#8A9BA8";

export interface HrvGraphProps {
  /** History values 0..1. Optional — a plausible default is generated. */
  samples?: number[];
}

const VB_W = 720;
const VB_H = 300;
const PAD_X = 56;
const PAD_TOP = 28;
const HRV_BAND = 150; // vertical room for the HRV waveform
const BREATH_TOP = PAD_TOP + HRV_BAND + 36;
const BREATH_BAND = 56;

const clamp01 = (v: number) => Math.max(0, Math.min(1, v));

/** Deterministic plausible HRV history: a wandering low->high->settle arc. */
function defaultSamples(n: number): number[] {
  const out: number[] = [];
  for (let i = 0; i < n; i += 1) {
    const t = i / (n - 1);
    // rising coherence arc + layered ripples (no Math.random -> SSR-stable)
    const arc = 0.28 + 0.6 * Math.sin(t * Math.PI * 0.92);
    const ripple =
      0.12 * Math.sin(t * Math.PI * 9) + 0.06 * Math.sin(t * Math.PI * 21 + 1.3);
    out.push(clamp01(arc + ripple));
  }
  return out;
}

/** Smooth-ish polyline path through value points across the band. */
function wavePath(values: number[], top: number, band: number): string {
  const n = values.length;
  const span = VB_W - PAD_X * 2;
  const pts = values.map((v, i) => {
    const x = PAD_X + (span * i) / (n - 1);
    const y = top + band * (1 - clamp01(v));
    return [x, y] as const;
  });
  return pts
    .map(([x, y], i) => `${i === 0 ? "M" : "L"} ${x.toFixed(1)} ${y.toFixed(1)}`)
    .join(" ");
}

/** 4:7:8 breath carrier — inhale rises (4), hold plateaus (7), exhale falls (8). */
function breathPath(): string {
  const span = VB_W - PAD_X * 2;
  const total = 4 + 7 + 8;
  const top = BREATH_TOP;
  const band = BREATH_BAND;
  const lo = top + band;
  const hi = top;
  // Repeat the cadence ~2.2 cycles across the width for a continuous carrier.
  const cycles = 2.2;
  const pts: string[] = [];
  const steps = 160;
  for (let i = 0; i <= steps; i += 1) {
    const tt = (i / steps) * cycles;
    const phase = (tt % 1) * total; // 0..19 within one breath
    let y: number;
    if (phase < 4) {
      // inhale: lo -> hi
      const p = phase / 4;
      y = lo + (hi - lo) * (0.5 - 0.5 * Math.cos(p * Math.PI));
    } else if (phase < 4 + 7) {
      // hold: stay near hi with a gentle drift
      y = hi + 2 * Math.sin((phase - 4) * 0.6);
    } else {
      // exhale: hi -> lo
      const p = (phase - 11) / 8;
      y = hi + (lo - hi) * (0.5 - 0.5 * Math.cos(p * Math.PI));
    }
    const x = PAD_X + (span * i) / steps;
    pts.push(`${i === 0 ? "M" : "L"} ${x.toFixed(1)} ${y.toFixed(1)}`);
  }
  return pts.join(" ");
}

export function HrvGraph({ samples }: HrvGraphProps) {
  const data = useMemo(() => samples ?? defaultSamples(64), [samples]);

  const hrvRef = useRef<SVGPathElement | null>(null);
  const breathRef = useRef<SVGPathElement | null>(null);

  const hrvD = useMemo(() => wavePath(data, PAD_TOP, HRV_BAND), [data]);
  const breathD = useMemo(() => breathPath(), []);

  // Faint sacred-geometry frame: a Metatron-style ring of overlapping circles
  // centred in the plot, behind the waveforms.
  const geometry = useMemo(() => {
    const cx = VB_W / 2;
    const cy = PAD_TOP + HRV_BAND / 2;
    const r = HRV_BAND / 2.4;
    const ring = Array.from({ length: 6 }, (_, i) => {
      const a = (Math.PI / 3) * i;
      return { x: cx + r * Math.cos(a), y: cy + r * Math.sin(a) };
    });
    return { cx, cy, r, ring };
  }, []);

  // Y-axis grid lines (value reference) + SF-Mono labels.
  const yTicks = [0, 0.25, 0.5, 0.75, 1];

  useEffect(() => {
    const reduce =
      typeof window !== "undefined" &&
      window.matchMedia?.("(prefers-reduced-motion: reduce)").matches;

    const animations: Array<{ pause: () => void }> = [];

    // 1 · HRV waveform draws itself.
    if (hrvRef.current) {
      const len = hrvRef.current.getTotalLength();
      hrvRef.current.style.strokeDasharray = `${len}`;
      if (reduce) {
        hrvRef.current.style.strokeDashoffset = "0";
      } else {
        hrvRef.current.style.strokeDashoffset = `${len}`;
        animations.push(
          animate(hrvRef.current, {
            strokeDashoffset: [len, 0],
            duration: 1800,
            ease: "outQuart",
            delay: 200,
          }),
        );
      }
    }

    // 2 · Breath carrier glides on the 4:7:8 cadence (continuous translateX loop).
    if (breathRef.current && !reduce) {
      const span = VB_W - PAD_X * 2;
      const drift = span / 2.2; // one breath-cycle width
      animations.push(
        animate(breathRef.current, {
          translateX: [0, -drift],
          // 4:7:8 -> 19 beats; ~0.7s/beat keeps it meditative (~13s/cycle).
          duration: 13300,
          ease: "linear",
          loop: true,
        }),
      );
    }

    return () => {
      animations.forEach((a) => a.pause());
    };
  }, [hrvD, breathD]);

  return (
    <svg
      viewBox={`0 0 ${VB_W} ${VB_H}`}
      width="100%"
      preserveAspectRatio="xMidYMid meet"
      role="img"
      aria-label="HRV coherence history waveform with 4:7:8 breath carrier"
      style={{ maxWidth: VB_W, display: "block" }}
    >
      <defs>
        {/* chaos -> optimal -> flow band across X */}
        <linearGradient id="hrv-band" x1="0" y1="0" x2="1" y2="0">
          <stop offset="0%" stopColor={TERRACOTTA} />
          <stop offset="50%" stopColor={EMERALD} />
          <stop offset="100%" stopColor={INDIGO} />
        </linearGradient>
        <linearGradient id="hrv-breath" x1="0" y1="0" x2="1" y2="0">
          <stop offset="0%" stopColor={EMERALD} stopOpacity="0.5" />
          <stop offset="100%" stopColor={GOLD} stopOpacity="0.5" />
        </linearGradient>
        <filter id="hrv-glow" x="-20%" y="-20%" width="140%" height="140%">
          <feGaussianBlur stdDeviation="2.4" result="b" />
          <feMerge>
            <feMergeNode in="b" />
            <feMergeNode in="SourceGraphic" />
          </feMerge>
        </filter>
      </defs>

      {/* ── faint sacred-geometry frame (behind everything) ── */}
      <g opacity={0.16} stroke={GOLD} fill="none" strokeWidth={0.6}>
        <circle cx={geometry.cx} cy={geometry.cy} r={geometry.r} />
        {geometry.ring.map((p, i) => (
          <circle key={i} cx={p.x} cy={p.y} r={geometry.r} />
        ))}
        {/* connecting spokes for the Metatron feel */}
        {geometry.ring.map((p, i) => (
          <line key={`s${i}`} x1={geometry.cx} y1={geometry.cy} x2={p.x} y2={p.y} />
        ))}
      </g>

      {/* ── y grid + SF-Mono value labels (no surrounding box) ── */}
      {yTicks.map((t) => {
        const y = PAD_TOP + HRV_BAND * (1 - t);
        return (
          <g key={t}>
            <line
              x1={PAD_X}
              y1={y}
              x2={VB_W - PAD_X}
              y2={y}
              stroke={SILVER}
              strokeWidth={0.4}
              opacity={0.12}
            />
            <text
              x={PAD_X - 10}
              y={y + 3}
              textAnchor="end"
              fontFamily="var(--font-mono, monospace)"
              fontSize={9}
              letterSpacing="0.5"
              fill={SILVER}
              opacity={0.55}
            >
              {t.toFixed(2)}
            </text>
          </g>
        );
      })}

      {/* ── HRV waveform overlay (gradient-coded, draws on mount) ── */}
      <path
        ref={hrvRef}
        d={hrvD}
        fill="none"
        stroke="url(#hrv-band)"
        strokeWidth={2.4}
        strokeLinecap="round"
        strokeLinejoin="round"
        filter="url(#hrv-glow)"
      />

      {/* ── 4:7:8 breath carrier beneath ── */}
      <text
        x={PAD_X}
        y={BREATH_TOP - 10}
        fontFamily="var(--font-mono, monospace)"
        fontSize={9}
        letterSpacing="1.5"
        fill={GOLD}
        opacity={0.7}
      >
        BREATH · 4 : 7 : 8
      </text>
      {/* clip so the looping carrier never spills past the plot edges */}
      <clipPath id="hrv-breath-clip">
        <rect x={PAD_X} y={BREATH_TOP - 8} width={VB_W - PAD_X * 2} height={BREATH_BAND + 16} />
      </clipPath>
      <g clipPath="url(#hrv-breath-clip)">
        <path
          ref={breathRef}
          d={breathD}
          fill="none"
          stroke="url(#hrv-breath)"
          strokeWidth={1.6}
          strokeLinecap="round"
        />
      </g>

      {/* ── x-axis time label (SF-Mono), no boxed axis ── */}
      <text
        x={VB_W - PAD_X}
        y={BREATH_TOP + BREATH_BAND + 22}
        textAnchor="end"
        fontFamily="var(--font-mono, monospace)"
        fontSize={9}
        letterSpacing="1"
        fill={SILVER}
        opacity={0.5}
      >
        NOW
      </text>
      <text
        x={PAD_X}
        y={BREATH_TOP + BREATH_BAND + 22}
        fontFamily="var(--font-mono, monospace)"
        fontSize={9}
        letterSpacing="1"
        fill={SILVER}
        opacity={0.5}
      >
        −64 READINGS
      </text>

      {/* ── coherence-band legend (chaos/optimal/flow), stacked top-right ── */}
      <g fontFamily="var(--font-mono, monospace)" fontSize={9} letterSpacing="1">
        {(
          [
            ["CHAOS", TERRACOTTA],
            ["OPTIMAL", EMERALD],
            ["FLOW", INDIGO],
          ] as const
        ).map(([word, color], i) => {
          const ly = PAD_TOP + 4 + i * 16;
          return (
            <g key={word}>
              <circle cx={VB_W - PAD_X - 70} cy={ly} r={3} fill={color} />
              <text x={VB_W - PAD_X - 60} y={ly + 3} fill={PARCHMENT} opacity={0.6}>
                {word}
              </text>
            </g>
          );
        })}
      </g>
    </svg>
  );
}

export default HrvGraph;
