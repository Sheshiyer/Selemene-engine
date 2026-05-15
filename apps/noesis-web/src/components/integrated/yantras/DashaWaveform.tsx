"use client";

// ─── DashaWaveform — iridescent 9-segment timeline ──────────────────────
// Per integrated-reading-design-v2.md § 5.8.
//
// Reference: Branding/witnessOS-sw/hrv.png — iridescent waveform.
//
// 9 horizontal mahadasha segments, widths proportional to duration.
// Past=Witness Violet, current=Sacred Gold (with bloom), future=Coherence
// Emerald. A smooth sine wave crosses all segments with amplitude/freq
// per-lord. Pivot dates marked with vertical Sacred Gold hairlines.
// Antardasha sub-segments drawn as fainter ticks inside the current MD.
//
// Animation: waveform draws left-to-right on viewport entry over 2s.

import { motion, useReducedMotion } from "motion/react";

export interface DashaSegment {
  lord: string;
  start_iso: string;
  end_iso: string;
  duration_years: number;
  state: "past" | "current" | "future";
}

interface DashaWaveformProps {
  periods: DashaSegment[];
  pivots?: Array<{ iso: string; label: string }>;
  antardashas?: Array<{
    parent_lord: string;
    lord: string;
    start_iso: string;
    end_iso: string;
  }>;
}

// Tonal intensity per lord — drives sine frequency. Higher = busier.
const LORD_INTENSITY: Record<string, number> = {
  mars: 1.7,
  rahu: 1.4,
  sun: 1.2,
  mercury: 1.1,
  moon: 0.9,
  venus: 0.8,
  jupiter: 0.7,
  ketu: 1.5,
  saturn: 0.5,
};

// Amplitude per lord, in viewport units (out of waveAmp).
const LORD_AMP: Record<string, number> = {
  mars: 1.0,
  rahu: 0.9,
  sun: 0.85,
  mercury: 0.7,
  moon: 0.75,
  venus: 0.6,
  jupiter: 0.55,
  ketu: 0.95,
  saturn: 0.4,
};

function strokeFor(state: DashaSegment["state"]): string {
  if (state === "current") return "var(--c-gold)";
  if (state === "past") return "var(--c-violet)";
  return "var(--c-emerald)";
}

function opacityFor(state: DashaSegment["state"]): number {
  if (state === "current") return 0.95;
  if (state === "past") return 0.55;
  return 0.7;
}

function formatYear(iso: string): string {
  const m = /^(\d{4})/.exec(iso);
  return m ? m[1] : iso;
}

export function DashaWaveform({
  periods,
  pivots = [],
  antardashas = [],
}: DashaWaveformProps) {
  const reduce = useReducedMotion();

  const W = 1200;
  const H = 280;
  const padL = 32;
  const padR = 32;
  const padT = 36;
  const padB = 56;
  const drawW = W - padL - padR;
  const drawH = H - padT - padB;
  const midY = padT + drawH / 2;
  const waveAmp = drawH * 0.42;

  const total = periods.reduce((s, p) => s + p.duration_years, 0) || 1;

  // Pre-compute segment x ranges.
  let cursor = padL;
  const segLayout = periods.map((p) => {
    const w = (p.duration_years / total) * drawW;
    const x0 = cursor;
    const x1 = cursor + w;
    cursor = x1;
    return { ...p, x0, x1, w };
  });

  // Sample the composite waveform — sample points across drawW, picking
  // amplitude/frequency from whichever segment that x falls inside.
  const samples = 480;
  const pathPoints: Array<{ x: number; y: number }> = [];
  for (let i = 0; i <= samples; i++) {
    const x = padL + (i / samples) * drawW;
    const seg = segLayout.find((s) => x >= s.x0 && x <= s.x1) ?? segLayout[segLayout.length - 1];
    const lord = seg.lord.toLowerCase();
    const amp = waveAmp * (LORD_AMP[lord] ?? 0.6);
    const freq = LORD_INTENSITY[lord] ?? 1.0;
    // Local phase within the segment so the wave reads coherent per period.
    const localT = (x - seg.x0) / Math.max(seg.w, 1);
    const y =
      midY +
      amp *
        Math.sin(
          (localT * Math.PI * 2 * freq * Math.max(seg.duration_years, 1)) / 4 +
            seg.x0 * 0.01,
        );
    pathPoints.push({ x, y });
  }
  const wavePath = pathPoints
    .map((p, i) => `${i === 0 ? "M" : "L"} ${p.x.toFixed(2)} ${p.y.toFixed(2)}`)
    .join(" ");

  // Pivot date → x. Resolves by linear date mapping inside the matching
  // segment, falling back to segment boundary if pivot equals end.
  const dateToX = (iso: string): number | null => {
    const target = new Date(iso).getTime();
    if (Number.isNaN(target)) return null;
    for (const seg of segLayout) {
      const s = new Date(seg.start_iso).getTime();
      const e = new Date(seg.end_iso).getTime();
      if (target >= s && target <= e) {
        const t = (target - s) / Math.max(e - s, 1);
        return seg.x0 + t * seg.w;
      }
    }
    return null;
  };

  return (
    <motion.div
      style={{
        width: "100%",
        margin: "clamp(1.5rem, 4vw, 3rem) 0",
        overflow: "visible",
      }}
      initial={{ opacity: 0 }}
      whileInView={{ opacity: 1 }}
      viewport={{ once: true, margin: "-10% 0px" }}
      transition={{ duration: 1.0, ease: [0.16, 1, 0.3, 1] }}
    >
      <svg
        viewBox={`0 0 ${W} ${H}`}
        width="100%"
        height="auto"
        preserveAspectRatio="xMidYMid meet"
        style={{
          width: "100%",
          maxWidth: "clamp(40rem, 90vw, 75rem)",
          display: "block",
          margin: "0 auto",
        }}
        role="img"
        aria-label="Dasha waveform timeline — past, current, and future mahadasha periods"
      >
        <defs>
          <linearGradient id="dw-grad" x1="0" y1="0" x2="1" y2="0">
            <stop offset="0%" stopColor="var(--c-violet)" />
            <stop offset="50%" stopColor="var(--c-gold)" />
            <stop offset="100%" stopColor="var(--c-emerald)" />
          </linearGradient>
          <filter id="dw-bloom" x="-20%" y="-50%" width="140%" height="200%">
            <feGaussianBlur stdDeviation="3" result="b" />
            <feMerge>
              <feMergeNode in="b" />
              <feMergeNode in="SourceGraphic" />
            </feMerge>
          </filter>
        </defs>

        {/* Baseline */}
        <line
          x1={padL}
          y1={midY}
          x2={W - padR}
          y2={midY}
          stroke="var(--line-faint)"
          strokeWidth={1}
        />

        {/* Segment rectangles (faint backdrop) */}
        {segLayout.map((s, i) => (
          <motion.rect
            key={`seg-${i}`}
            x={s.x0}
            y={padT}
            width={s.w}
            height={drawH}
            fill={strokeFor(s.state)}
            fillOpacity={s.state === "current" ? 0.1 : 0.04}
            stroke={strokeFor(s.state)}
            strokeOpacity={opacityFor(s.state) * 0.45}
            strokeWidth={s.state === "current" ? 1.4 : 0.8}
            filter={s.state === "current" ? "url(#dw-bloom)" : undefined}
            whileHover={{ opacity: 1.15, scale: 1 }}
            style={{ transformOrigin: `${s.x0 + s.w / 2}px ${midY}px` }}
            initial={reduce ? undefined : { opacity: 0 }}
            whileInView={reduce ? undefined : { opacity: 1 }}
            viewport={{ once: true, margin: "-10% 0px" }}
            transition={{ duration: 0.6, delay: 0.1 + i * 0.05 }}
          />
        ))}

        {/* Segment labels */}
        {segLayout.map((s, i) => {
          const labelY = padT - 12;
          const labelX = s.x0 + s.w / 2;
          return (
            <g key={`lbl-${i}`}>
              <text
                x={labelX}
                y={labelY}
                textAnchor="middle"
                style={{
                  fontFamily: "var(--font-mono)",
                  fontSize: 10,
                  fill:
                    s.state === "current"
                      ? "var(--c-gold)"
                      : "var(--c-parchment)",
                  letterSpacing: "0.18em",
                  opacity: s.state === "past" ? 0.6 : 0.95,
                }}
              >
                {s.lord.toUpperCase()}
              </text>
              <text
                x={labelX}
                y={H - padB + 16}
                textAnchor="middle"
                style={{
                  fontFamily: "var(--font-mono)",
                  fontSize: 9,
                  fill: "var(--muted)",
                  letterSpacing: "0.12em",
                }}
              >
                {formatYear(s.start_iso)}–{formatYear(s.end_iso)}
              </text>
            </g>
          );
        })}

        {/* The waveform itself */}
        <motion.path
          d={wavePath}
          fill="none"
          stroke="url(#dw-grad)"
          strokeWidth={2}
          strokeLinecap="round"
          strokeLinejoin="round"
          filter="url(#dw-bloom)"
          initial={reduce ? { pathLength: 1 } : { pathLength: 0 }}
          whileInView={{ pathLength: 1 }}
          viewport={{ once: true, margin: "-10% 0px" }}
          transition={{ duration: reduce ? 0 : 2.0, ease: [0.16, 1, 0.3, 1] }}
        />

        {/* Antardasha ticks inside the current MD */}
        {(() => {
          const current = segLayout.find((s) => s.state === "current");
          if (!current) return null;
          const subs = antardashas.filter(
            (a) => a.parent_lord.toLowerCase() === current.lord.toLowerCase(),
          );
          if (subs.length === 0) return null;
          const segStart = new Date(current.start_iso).getTime();
          const segEnd = new Date(current.end_iso).getTime();
          const segDur = Math.max(segEnd - segStart, 1);
          return subs.map((sub, i) => {
            const subStart = new Date(sub.start_iso).getTime();
            const t = (subStart - segStart) / segDur;
            if (t < 0 || t > 1) return null;
            const tx = current.x0 + t * current.w;
            return (
              <line
                key={`ad-${i}`}
                x1={tx}
                y1={midY - 16}
                x2={tx}
                y2={midY + 16}
                stroke="var(--c-gold)"
                strokeOpacity={0.45}
                strokeWidth={0.8}
                strokeDasharray="2 3"
              />
            );
          });
        })()}

        {/* Pivot hairlines */}
        {pivots.map((p, i) => {
          const x = dateToX(p.iso);
          if (x === null) return null;
          return (
            <motion.g
              key={`pv-${i}`}
              initial={reduce ? undefined : { opacity: 0 }}
              whileInView={reduce ? undefined : { opacity: 1 }}
              viewport={{ once: true, margin: "-10% 0px" }}
              transition={{ duration: 0.8, delay: 1.6 + i * 0.1 }}
            >
              <line
                x1={x}
                y1={padT - 4}
                x2={x}
                y2={H - padB + 4}
                stroke="var(--c-gold)"
                strokeOpacity={0.85}
                strokeWidth={1}
                strokeDasharray="3 4"
              />
              <text
                x={x}
                y={H - padB + 38}
                textAnchor="middle"
                style={{
                  fontFamily: "var(--font-mono)",
                  fontSize: 10,
                  fill: "var(--c-gold)",
                  letterSpacing: "0.14em",
                }}
              >
                {p.label}
              </text>
            </motion.g>
          );
        })}
      </svg>
    </motion.div>
  );
}
