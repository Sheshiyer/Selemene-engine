"use client";

/**
 * BiofieldCosmogram v4 — Borderless floating geometry panels
 *
 * Grid symmetry preserved, all box/divider lines removed.
 * Each panel is an isolated floating geometry node — click label to collapse.
 * Responsive: SVG viewBox scales to fill container, all coords percentage-derived.
 *
 * Actual metric names from PIP_Analysis_System_Specification:
 *   ENERGY        — Light Quanta Density + average intensity
 *   COHERENCE     — Pattern regularity + Hurst exponent + temporal stability
 *   SYMMETRY      — Body bilateral symmetry (SSIM)
 *   COMPLEXITY    — Fractal dimension + color entropy
 *   REGULATION    — Lyapunov + DFA alpha + temporal variance
 *   COLOR FIELD   — Saturation mean + color coherence
 */

import React, { useMemo, useState } from "react";
import type { CompositeScores } from "./pip/types";

// ── Brand tokens ──────────────────────────────────────────────────────────────
const GOLD      = "#C5A017";
const EMERALD   = "#10B5A7";
const VIOLET    = "#6B3FA0";
const INDIGO    = "#0B50FB";
const PARCHMENT = "#F0EDE3";

// ── SVG canvas ────────────────────────────────────────────────────────────────
const VW = 420;
const VH = 360;
const MCX = VW / 2;    // 210 — mandala center x
const MCY = VH * 0.44; // 158 — mandala center y (slightly above half)

// ── Grid anchor points (6 panels around the mandala) ─────────────────────────
const PANELS = {
  topLeft:    { x: 60,  y: 72  },
  topRight:   { x: 360, y: 72  },
  midLeft:    { x: 52,  y: 198 },
  midRight:   { x: 368, y: 198 },
  botLeft:    { x: 60,  y: 318 },
  botCenter:  { x: 210, y: 328 },
} as const;

// ── Math helpers ──────────────────────────────────────────────────────────────
const TAU = Math.PI * 2;
function rad(d: number) { return (d * Math.PI) / 180; }
function clamp(v: number, lo = 0, hi = 1) { return Math.max(lo, Math.min(hi, v)); }
function px(n: number) { return n.toFixed(2); }

function lotus(cx: number, cy: number, angleDeg: number, len: number, wid: number): string {
  const a = rad(angleDeg);
  const p = a + Math.PI / 2;
  const tx = cx + len * Math.cos(a), ty = cy + len * Math.sin(a);
  const ax = cx + len * 0.60 * Math.cos(a) + wid * Math.cos(p);
  const ay = cy + len * 0.60 * Math.sin(a) + wid * Math.sin(p);
  const bx = cx + len * 0.60 * Math.cos(a) - wid * Math.cos(p);
  const by = cy + len * 0.60 * Math.sin(a) - wid * Math.sin(p);
  return `M${px(cx)},${px(cy)} Q${px(ax)},${px(ay)} ${px(tx)},${px(ty)} Q${px(bx)},${px(by)} ${px(cx)},${px(cy)} Z`;
}

function diamond(cx: number, cy: number, angleDeg: number, len: number, wid: number): string {
  const a = rad(angleDeg);
  const p = a + Math.PI / 2;
  const tx = cx + len * Math.cos(a), ty = cy + len * Math.sin(a);
  const mx = cx + len * 0.42 * Math.cos(a), my = cy + len * 0.42 * Math.sin(a);
  const ax = mx + wid * Math.cos(p), ay = my + wid * Math.sin(p);
  const bx = mx - wid * Math.cos(p), by = my - wid * Math.sin(p);
  return `M${px(cx)},${px(cy)} L${px(ax)},${px(ay)} L${px(tx)},${px(ty)} L${px(bx)},${px(by)} Z`;
}

function circlePts(cx: number, cy: number, r: number, n: number) {
  return Array.from({ length: n }, (_, i) => {
    const a = rad(360 / n * i - 90);
    return { x: cx + r * Math.cos(a), y: cy + r * Math.sin(a) };
  });
}

function sineWave(x0: number, y0: number, w: number, amp: number, cycles: number): string {
  return Array.from({ length: 65 }, (_, i) => {
    const t = i / 64;
    return `${i === 0 ? "M" : "L"}${(x0 + t * w).toFixed(1)},${(y0 + amp * Math.sin(t * cycles * TAU)).toFixed(1)}`;
  }).join(" ");
}

function squiggle(x0: number, y0: number, w: number, amp: number): string {
  return Array.from({ length: 65 }, (_, i) => {
    const t = i / 64;
    const y = y0 + amp * (
      Math.sin(t * 4.0 * TAU) * 0.65 +
      Math.sin(t * 7.3 * TAU + 0.5) * 0.25 +
      Math.sin(t * 2.1 * TAU - 0.3) * 0.10
    );
    return `${i === 0 ? "M" : "L"}${(x0 + t * w).toFixed(1)},${y.toFixed(1)}`;
  }).join(" ");
}

/** Radial arc gauge (no box — just the arc + metric glow dot) */
function arcPath(cx: number, cy: number, r: number, value: number, startDeg: number, sweepDeg: number) {
  const startR = rad(startDeg);
  const endR   = rad(startDeg + sweepDeg * value);
  const large  = sweepDeg * value > 180 ? 1 : 0;
  const x1 = cx + r * Math.cos(startR), y1 = cy + r * Math.sin(startR);
  const x2 = cx + r * Math.cos(endR),   y2 = cy + r * Math.sin(endR);
  return `M ${px(x1)} ${px(y1)} A ${r} ${r} 0 ${large} 1 ${px(x2)} ${px(y2)}`;
}

// ── Tiny geometric "icon" per metric ─────────────────────────────────────────
function EnergyIcon({ cx, cy, val, color }: { cx: number; cy: number; val: number; color: string }) {
  const r = 16;
  const pts = Array.from({ length: 6 }, (_, i) => {
    const a = rad(i * 60 - 90);
    const rv = r * (0.4 + val * 0.6);
    return `${(cx + rv * Math.cos(a)).toFixed(1)},${(cy + rv * Math.sin(a)).toFixed(1)}`;
  }).join(" ");
  return (
    <g>
      <polygon points={pts} fill="none" stroke={color} strokeWidth="1.1" opacity="0.75" />
      <circle cx={cx} cy={cy} r="3" fill={color} opacity="0.8" />
      {Array.from({ length: 6 }, (_, i) => {
        const a = rad(i * 60 - 90);
        const rv = r * (0.4 + val * 0.6);
        return <circle key={i} cx={cx + rv * Math.cos(a)} cy={cy + rv * Math.sin(a)} r="1.4" fill={color} opacity="0.55" />;
      })}
    </g>
  );
}

function CoherenceIcon({ cx, cy, val, color }: { cx: number; cy: number; val: number; color: string }) {
  const rings = [14, 10, 7, 4];
  return (
    <g>
      {rings.map((r, i) => (
        <circle key={i} cx={cx} cy={cy} r={r}
          fill="none" stroke={color}
          strokeWidth={i === 0 ? 0.6 : 0.4}
          opacity={0.15 + val * 0.55 * (1 - i * 0.22)}
          strokeDasharray={i % 2 === 0 ? "none" : "2 3"}
        />
      ))}
      <circle cx={cx} cy={cy} r="2.5" fill={color} opacity={0.6 + val * 0.4} />
    </g>
  );
}

function SymmetryIcon({ cx, cy, val, color }: { cx: number; cy: number; val: number; color: string }) {
  const r = 13;
  return (
    <g>
      {[0, 45, 90, 135].map((a, i) => (
        <path key={i}
          d={diamond(cx, cy, a - 90, r * (0.5 + val * 0.5), 4 + val * 3)}
          fill="none" stroke={color} strokeWidth="0.75" opacity="0.6"
        />
      ))}
      <line x1={cx - r} y1={cy} x2={cx + r} y2={cy}
        stroke={color} strokeWidth="0.4" opacity="0.3"
      />
      <line x1={cx} y1={cy - r} x2={cx} y2={cy + r}
        stroke={color} strokeWidth="0.4" opacity="0.3"
      />
    </g>
  );
}

function ComplexityIcon({ cx, cy, val, color }: { cx: number; cy: number; val: number; color: string }) {
  const pts = Array.from({ length: 5 }, (_, i) => {
    const a = rad(i * 72 - 90);
    const outer = 15;
    const inner = 6 + val * 5;
    const isOuter = i % 2 === 0;
    const rv = isOuter ? outer : inner;
    return { x: cx + rv * Math.cos(a), y: cy + rv * Math.sin(a) };
  });
  const d = pts.map((p, i) => `${i === 0 ? "M" : "L"}${p.x.toFixed(1)},${p.y.toFixed(1)}`).join(" ") + " Z";
  return (
    <g>
      <path d={d} fill="none" stroke={color} strokeWidth="0.9" opacity="0.65" />
      {pts.map((p, i) => (
        <circle key={i} cx={p.x} cy={p.y} r="1.2" fill={color} opacity="0.5" />
      ))}
    </g>
  );
}

function RegulationIcon({ cx, cy, val, color }: { cx: number; cy: number; val: number; color: string }) {
  const path = sineWave(cx - 18, cy, 36, 6 * val + 2, 2.5);
  return (
    <g>
      <path d={path} fill="none" stroke={color} strokeWidth="1.4" opacity="0.75" />
      <circle cx={cx} cy={cy} r="2.5" fill={color} opacity="0.7" />
    </g>
  );
}

function ColorFieldIcon({ cx, cy, val, color }: { cx: number; cy: number; val: number; color: string }) {
  const n = 8;
  const r = 14;
  return (
    <g>
      {Array.from({ length: n }, (_, i) => {
        const a = rad(i * (360 / n) - 90);
        const rv = r * (0.55 + val * 0.45);
        const cols = [GOLD, EMERALD, VIOLET, INDIGO, GOLD, EMERALD, VIOLET, INDIGO];
        return (
          <path key={i}
            d={lotus(cx, cy, (i * (360 / n)) - 90, rv, 5 + val * 4)}
            fill="none" stroke={cols[i]} strokeWidth="0.7" opacity={0.4 + val * 0.45}
          />
        );
      })}
    </g>
  );
}

// ── Panel component — floating geometry node, collapsible ─────────────────────
interface PanelProps {
  id: string;
  cx: number;
  cy: number;
  label: string;
  sublabel?: string;
  value: number;
  displayVal: string;
  color: string;
  collapsed: boolean;
  onToggle: (id: string) => void;
  icon: React.ReactNode;
  anchor?: "start" | "middle" | "end";
}

function FloatingPanel({ id, cx, cy, label, sublabel, value, displayVal, color, collapsed, onToggle, icon, anchor = "middle" }: PanelProps) {
  const tAnchor = anchor;
  const dx = anchor === "start" ? -8 : anchor === "end" ? 8 : 0;
  const iconDy = collapsed ? 0 : -10;

  return (
    <g onClick={() => onToggle(id)} style={{ cursor: "pointer" }}>
      {/* Faint glow circle behind icon */}
      {!collapsed && (
        <circle cx={cx} cy={cy + iconDy - 4} r="20"
          fill={color} opacity="0.05"
        />
      )}

      {/* Icon */}
      {!collapsed && (
        <g transform={`translate(${cx},${cy + iconDy - 4})`}>
          {icon}
        </g>
      )}

      {/* Value */}
      {!collapsed && (
        <text x={cx + dx} y={cy + 16}
          textAnchor={tAnchor}
          fontFamily="var(--font-mono,monospace)"
          fontSize="11" fontWeight="700" letterSpacing="-0.02em"
          fill={PARCHMENT} opacity="0.88"
        >{displayVal}</text>
      )}

      {/* Label (always visible, acts as toggle) */}
      <text x={cx + dx} y={cy + (collapsed ? 6 : 28)}
        textAnchor={tAnchor}
        fontFamily="var(--font-display,'Panchang',monospace)"
        fontSize="6.5" fontWeight="700" letterSpacing="2"
        fill={PARCHMENT} opacity={collapsed ? 0.35 : 0.55}
      >{label}</text>

      {sublabel && !collapsed && (
        <text x={cx + dx} y={cy + 38}
          textAnchor={tAnchor}
          fontFamily="var(--font-display,'Panchang',monospace)"
          fontSize="6" letterSpacing="1.5"
          fill={PARCHMENT} opacity="0.38"
        >{sublabel}</text>
      )}

      {/* Thin arc progress ring (collapsed state shows just the arc) */}
      <path
        d={arcPath(cx, cy + (collapsed ? 0 : 14), collapsed ? 10 : 5, value, -140, 280)}
        fill="none" stroke={color}
        strokeWidth={collapsed ? 1.8 : 1.2}
        opacity={collapsed ? 0.7 : 0.45}
        strokeLinecap="round"
      />
    </g>
  );
}

// ═══════════════════════════════════════════════════════════════════════════════
// Main component
// ═══════════════════════════════════════════════════════════════════════════════
export interface BiofieldCosmogramProps {
  scores: CompositeScores;
  engineResult?: Record<string, unknown> | null;
}

export function BiofieldCosmogram({ scores }: BiofieldCosmogramProps) {
  const coherence   = clamp(scores.overallCoherence);
  const energy      = clamp(scores.lightQuantaDensity);
  const symmetry    = clamp(scores.bodySymmetry);
  const complexity  = clamp(scores.patternRegularity);      // best proxy until fractal dim lands
  const regulation  = clamp((scores as any).regulationScore ?? coherence * 0.8);
  const colorField  = clamp((scores as any).colorBalance ?? scores.normalizedArea);

  const stateColor = coherence >= 0.75 ? EMERALD : coherence >= 0.5 ? INDIGO : GOLD;
  const stateWord  = coherence >= 0.75 ? "OPTIMAL" : coherence >= 0.5 ? "BUILDING" : "ATTUNING";
  const cohPct     = Math.round(coherence * 100);

  // Collapsed panel state
  const [collapsed, setCollapsed] = useState<Record<string, boolean>>({});
  const toggle = (id: string) => setCollapsed(prev => ({ ...prev, [id]: !prev[id] }));
  const isOff  = (id: string) => !!collapsed[id];

  // Mandala geometry
  const outerDots = useMemo(() => circlePts(MCX, MCY, 82, 40), []);
  const innerDots = useMemo(() => circlePts(MCX, MCY, 70, 28), []);

  // HRV wave (bottom center)
  const hrvPath = useMemo(
    () => sineWave(PANELS.botCenter.x - 48, PANELS.botCenter.y - 8, 96, 8 * coherence + 3, 2.5),
    [coherence]
  );

  // Regulation squiggle (top-left alert area)
  const alertPath = useMemo(() => squiggle(PANELS.topLeft.x - 28, PANELS.topLeft.y + 14, 56, 5), []);

  return (
    <section style={{ width: "100%", height: "100%", display: "flex", background: "transparent" }}>
      <svg
        viewBox={`0 0 ${VW} ${VH}`}
        preserveAspectRatio="xMidYMid meet"
        width="100%"
        height="100%"
        aria-label="Biofield consciousness instrument"
      >
        <defs>
          <filter id="g4" x="-50%" y="-50%" width="200%" height="200%">
            <feGaussianBlur stdDeviation="4" result="b"/>
            <feMerge><feMergeNode in="b"/><feMergeNode in="SourceGraphic"/></feMerge>
          </filter>
          <filter id="g2" x="-30%" y="-30%" width="160%" height="160%">
            <feGaussianBlur stdDeviation="2" result="b"/>
            <feMerge><feMergeNode in="b"/><feMergeNode in="SourceGraphic"/></feMerge>
          </filter>
          <radialGradient id="mGlow" cx="50%" cy="50%" r="50%">
            <stop offset="0%"   stopColor={stateColor} stopOpacity="0.22"/>
            <stop offset="55%"  stopColor={stateColor} stopOpacity="0.06"/>
            <stop offset="100%" stopColor={stateColor} stopOpacity="0"/>
          </radialGradient>
          <linearGradient id="hrvG" x1="0%" y1="0%" x2="100%" y2="0%">
            <stop offset="0%"   stopColor={GOLD} stopOpacity="0.3"/>
            <stop offset="50%"  stopColor={GOLD} stopOpacity="1.0"/>
            <stop offset="100%" stopColor={GOLD} stopOpacity="0.3"/>
          </linearGradient>
          <style>{`
            .cd-pulse { animation: cdp 3.5s ease-in-out infinite; }
            @keyframes cdp { 0%,100%{opacity:1} 50%{opacity:0.58} }
            .cd-spin   { animation: cds 90s linear infinite; }
            @keyframes cds { from{transform:rotate(0deg)} to{transform:rotate(360deg)} }
          `}</style>
        </defs>

        {/* ══ CENTRAL MANDALA ════════════════════════════════════════════════ */}
        <ellipse cx={MCX} cy={MCY} rx={92} ry={92} fill="url(#mGlow)" />

        {/* Outer thin gold circle */}
        <circle cx={MCX} cy={MCY} r={88}
          fill="none" stroke={GOLD} strokeWidth="0.55" opacity="0.28"
        />

        {/* Outer spinning dot ring */}
        <g className="cd-spin" style={{ transformOrigin: `${MCX}px ${MCY}px` }}>
          {outerDots.map((d, i) => (
            <circle key={`od${i}`} cx={d.x} cy={d.y}
              r={i % 5 === 0 ? 1.4 : 1.0}
              fill={GOLD} opacity={i % 5 === 0 ? 0.48 : 0.26}
            />
          ))}
        </g>

        {/* Inner static dot ring */}
        {innerDots.map((d, i) => (
          <circle key={`id${i}`} cx={d.x} cy={d.y} r="0.8"
            fill={GOLD} opacity={i % 4 === 0 ? 0.34 : 0.16}
          />
        ))}

        {/* 8 compass star tips */}
        {Array.from({ length: 8 }, (_, i) => (
          <path key={`st${i}`}
            d={diamond(MCX, MCY, i * 45 - 90, 75, 8)}
            fill="none" stroke={GOLD} strokeWidth="0.75" opacity="0.54"
          />
        ))}

        {/* Outer lotus petals (N/S/E/W) */}
        {[0, 90, 180, 270].map((deg, i) => (
          <path key={`lp1${i}`}
            d={lotus(MCX, MCY, deg - 90, 63, 37)}
            fill="none" stroke={GOLD} strokeWidth="1.0" opacity="0.72"
          />
        ))}

        {/* Mid lotus petals (diagonals) */}
        {[45, 135, 225, 315].map((deg, i) => (
          <path key={`lp2${i}`}
            d={lotus(MCX, MCY, deg - 90, 50, 29)}
            fill="none" stroke={GOLD} strokeWidth="0.85" opacity="0.62"
          />
        ))}

        {/* Core petals */}
        {[0, 90, 180, 270].map((deg, i) => (
          <path key={`lp3${i}`}
            d={lotus(MCX, MCY, deg - 90, 23, 12)}
            fill="none" stroke={GOLD} strokeWidth="0.7" opacity="0.84"
          />
        ))}

        {/* Center dot */}
        <circle cx={MCX} cy={MCY} r="9" fill={stateColor} opacity="0.14" filter="url(#g4)" />
        <circle cx={MCX} cy={MCY} r="3.5" fill={stateColor} opacity="0.92" className="cd-pulse" />

        {/* State word */}
        <text x={MCX} y={MCY + 18}
          textAnchor="middle"
          fontFamily="var(--font-display,'Panchang',monospace)"
          fontSize="7" fontWeight="700" letterSpacing="2.5"
          fill={stateColor} opacity="0.75" className="cd-pulse"
        >{stateWord}</text>
        <text x={MCX} y={MCY + 30}
          textAnchor="middle"
          fontFamily="var(--font-mono,monospace)"
          fontSize="9" fontWeight="600"
          fill={PARCHMENT} opacity="0.28"
        >{cohPct}%</text>

        {/* ══ SIX FLOATING PANELS ════════════════════════════════════════════ */}

        {/* top-left — ENERGY */}
        <FloatingPanel
          id="energy" cx={PANELS.topLeft.x} cy={PANELS.topLeft.y}
          label="ENERGY" sublabel="Light Quanta"
          value={energy} displayVal={`${Math.round(energy * 100)}`}
          color={GOLD} collapsed={isOff("energy")} onToggle={toggle}
          anchor="middle"
          icon={<EnergyIcon cx={0} cy={0} val={energy} color={GOLD} />}
        />

        {/* top-right — SYMMETRY */}
        <FloatingPanel
          id="symmetry" cx={PANELS.topRight.x} cy={PANELS.topRight.y}
          label="SYMMETRY" sublabel="Bilateral"
          value={symmetry} displayVal={`${Math.round(symmetry * 100)}%`}
          color={EMERALD} collapsed={isOff("symmetry")} onToggle={toggle}
          anchor="middle"
          icon={<SymmetryIcon cx={0} cy={0} val={symmetry} color={EMERALD} />}
        />

        {/* mid-left — COHERENCE */}
        <FloatingPanel
          id="coherence" cx={PANELS.midLeft.x} cy={PANELS.midLeft.y}
          label="COHERENCE" sublabel="Hurst · Pattern"
          value={coherence} displayVal={`${Math.round(coherence * 100)}%`}
          color={INDIGO} collapsed={isOff("coherence")} onToggle={toggle}
          anchor="middle"
          icon={<CoherenceIcon cx={0} cy={0} val={coherence} color={INDIGO} />}
        />

        {/* mid-right — COMPLEXITY */}
        <FloatingPanel
          id="complexity" cx={PANELS.midRight.x} cy={PANELS.midRight.y}
          label="COMPLEXITY" sublabel="Fractal Dim"
          value={complexity} displayVal={`${Math.round(complexity * 100)}`}
          color={VIOLET} collapsed={isOff("complexity")} onToggle={toggle}
          anchor="middle"
          icon={<ComplexityIcon cx={0} cy={0} val={complexity} color={VIOLET} />}
        />

        {/* bot-left — REGULATION */}
        <FloatingPanel
          id="regulation" cx={PANELS.botLeft.x} cy={PANELS.botLeft.y}
          label="REGULATION" sublabel="DFA · Lyapunov"
          value={regulation} displayVal={`${Math.round(regulation * 100)}`}
          color={EMERALD} collapsed={isOff("regulation")} onToggle={toggle}
          anchor="middle"
          icon={<RegulationIcon cx={0} cy={0} val={regulation} color={EMERALD} />}
        />

        {/* bot-center — COHERENCE WAVE (HRV proxy) */}
        {!isOff("hrv") && (
          <g onClick={() => toggle("hrv")} style={{ cursor: "pointer" }}>
            <path d={hrvPath}
              fill="none" stroke="url(#hrvG)" strokeWidth="1.5"
              filter="url(#g2)" opacity={0.6 + coherence * 0.4}
            />
            <text x={PANELS.botCenter.x} y={PANELS.botCenter.y + 18}
              textAnchor="middle"
              fontFamily="var(--font-display,'Panchang',monospace)"
              fontSize="6.5" fontWeight="700" letterSpacing="2"
              fill={PARCHMENT} opacity="0.45"
            >COLOR FIELD</text>
          </g>
        )}
        {isOff("hrv") && (
          <g onClick={() => toggle("hrv")} style={{ cursor: "pointer" }}>
            <text x={PANELS.botCenter.x} y={PANELS.botCenter.y + 6}
              textAnchor="middle"
              fontFamily="var(--font-display,'Panchang',monospace)"
              fontSize="6.5" fontWeight="700" letterSpacing="2"
              fill={PARCHMENT} opacity="0.25"
            >COLOR FIELD</text>
          </g>
        )}

      </svg>
    </section>
  );
}
