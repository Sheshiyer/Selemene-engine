"use client";

/**
 * BiofieldCosmogram v3 — Consciousness Dashboard
 *
 * Matches dashboard.png reference exactly:
 * - Outer rounded card with gold border
 * - "CONSCIOUSNESS DASHBOARD" title in gold
 * - 3×3 grid with thin gold dividers
 * - Central multi-layer gold lotus mandala (spans rows 1+2 of center column)
 * - 6 surrounding panel visualizations (pure sacred geometry, zero chart libs)
 * - HRV COHERENCE bottom-center with animated sine wave
 *
 * All metrics data-driven from CompositeScores (live biofield pipeline).
 */

import React, { useMemo } from "react";
import type { CompositeScores } from "./pip/types";

// ── Brand tokens ──────────────────────────────────────────────────────────────
const GOLD      = "#C5A017";
const EMERALD   = "#10B5A7";
const VIOLET    = "#6B3FA0";
const PARCHMENT = "#F0EDE3";

// ── SVG layout ────────────────────────────────────────────────────────────────
const VW    = 420;
const VH    = 360;
const GRID_T = 28;   // top of grid area (below title)
const GRID_B = 356;  // bottom of grid area
const L_W   = 112;   // left column width
const R_W   = 112;   // right column width

// Divider positions
const DIV_X1 = L_W;            // x=112
const DIV_X2 = VW - R_W;       // x=308
const R1_H   = 124;
const R2_H   = 124;
const DIV_Y1 = GRID_T + R1_H;  // y=152 (row1/row2)
const DIV_Y2 = DIV_Y1 + R2_H;  // y=276 (row2/row3)
const R3_H   = GRID_B - DIV_Y2; // 80

// Cell reference coordinates
const MCX = VW / 2;                   // 210 — mandala x
const MCY = (GRID_T + DIV_Y2) / 2;   // 152 — mandala center y (mid of rows 1+2)
const LCX = L_W / 2;                  //  56 — left col center x
const RCX = DIV_X2 + R_W / 2;        // 364 — right col center x
const CY1 = GRID_T + R1_H / 2;       //  90 — row1 center y
const CY2 = DIV_Y1 + R2_H / 2;       // 214 — row2 center y
const CY3 = DIV_Y2 + R3_H / 2;       // 316 — row3 center y

// ── Mandala geometry ──────────────────────────────────────────────────────────
const MR_OUTER  = 88;   // outer thin circle
const MR_DOTOUT = 82;   // outer dashed-dot ring
const MR_DOTIN  = 71;   // inner dashed-dot ring
const MSTAR_LEN = 75;   // 8-star tip length (sharp diamond)
const MSTAR_WID = 8;    // star tip width
const MLP1_LEN  = 63;   // outer lotus petal length (4, cardinal)
const MLP1_WID  = 37;   // outer lotus petal width  (plump)
const MLP2_LEN  = 50;   // mid lotus petal length   (4, diagonal)
const MLP2_WID  = 29;   // mid lotus petal width
const MLP3_LEN  = 23;   // tiny core petal length   (4, cardinal)
const MLP3_WID  = 12;   // tiny core petal width

// ── Math helpers ──────────────────────────────────────────────────────────────
const TAU = Math.PI * 2;
function rad(d: number) { return (d * Math.PI) / 180; }
function clamp(v: number, lo = 0, hi = 1) { return Math.max(lo, Math.min(hi, v)); }
function px(n: number) { return n.toFixed(2); }

/** Wide teardrop lotus petal (two quadratic bezier curves meeting at tip) */
function lotus(cx: number, cy: number, angleDeg: number, len: number, wid: number): string {
  const a = rad(angleDeg);
  const p = a + Math.PI / 2;
  const tx = cx + len * Math.cos(a),          ty = cy + len * Math.sin(a);
  const ax = cx + len * 0.60 * Math.cos(a) + wid * Math.cos(p);
  const ay = cy + len * 0.60 * Math.sin(a) + wid * Math.sin(p);
  const bx = cx + len * 0.60 * Math.cos(a) - wid * Math.cos(p);
  const by = cy + len * 0.60 * Math.sin(a) - wid * Math.sin(p);
  return `M${px(cx)},${px(cy)} Q${px(ax)},${px(ay)} ${px(tx)},${px(ty)} Q${px(bx)},${px(by)} ${px(cx)},${px(cy)} Z`;
}

/** Sharp diamond petal (for 8-pointed star compass tips) */
function diamond(cx: number, cy: number, angleDeg: number, len: number, wid: number): string {
  const a = rad(angleDeg);
  const p = a + Math.PI / 2;
  const tx = cx + len * Math.cos(a),           ty = cy + len * Math.sin(a);
  const mx = cx + len * 0.42 * Math.cos(a),   my = cy + len * 0.42 * Math.sin(a);
  const ax = mx + wid * Math.cos(p),           ay = my + wid * Math.sin(p);
  const bx = mx - wid * Math.cos(p),           by = my - wid * Math.sin(p);
  return `M${px(cx)},${px(cy)} L${px(ax)},${px(ay)} L${px(tx)},${px(ty)} L${px(bx)},${px(by)} Z`;
}

/** Evenly-spaced dot positions on a circle */
function circlePts(cx: number, cy: number, r: number, n: number): { x: number; y: number }[] {
  return Array.from({ length: n }, (_, i) => {
    const a = rad(360 / n * i - 90);
    return { x: cx + r * Math.cos(a), y: cy + r * Math.sin(a) };
  });
}

/** Sine wave SVG path */
function sineWave(x0: number, y0: number, w: number, amp: number, cycles: number): string {
  return Array.from({ length: 65 }, (_, i) => {
    const t = i / 64;
    return `${i === 0 ? "M" : "L"}${(x0 + t * w).toFixed(1)},${(y0 + amp * Math.sin(t * cycles * TAU)).toFixed(1)}`;
  }).join(" ");
}

/** Irregular EEG-like squiggle (multi-frequency, for ALERTS panel) */
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

// ═══════════════════════════════════════════════════════════════════════════════
// Component
// ═══════════════════════════════════════════════════════════════════════════════
export interface BiofieldCosmogramProps {
  scores: CompositeScores;
  engineResult?: Record<string, unknown> | null;
}

export function BiofieldCosmogram({ scores }: BiofieldCosmogramProps) {
  const coherence = clamp(scores.overallCoherence);
  const energy    = clamp(scores.lightQuantaDensity);
  const symmetry  = clamp(scores.bodySymmetry);
  const pattern   = clamp(scores.patternRegularity);
  const presence  = clamp(scores.normalizedArea);

  const stateColor = coherence >= 0.75 ? EMERALD : coherence >= 0.5 ? "#0B50FB" : GOLD;
  const stateWord  = coherence >= 0.75 ? "OPTIMAL" : coherence >= 0.5 ? "BUILDING" : "ATTUNING";
  const alertColor = coherence < 0.45  ? VIOLET : coherence < 0.75 ? GOLD : EMERALD;
  const alertText  = coherence < 0.45  ? "Coherence Dip" : coherence < 0.75 ? "Building" : "Aligned";
  const sigCount   = Math.max(1, Math.round(pattern * 4 + 1));
  const recovVal   = (energy * 9.9).toFixed(1);
  const cohPct     = Math.round(coherence * 100);

  // Mandala dot rings
  const outerDots = useMemo(() => circlePts(MCX, MCY, MR_DOTOUT, 40), []);
  const innerDots = useMemo(() => circlePts(MCX, MCY, MR_DOTIN,  28), []);

  // BIOFIELD CORRELATIONS constellation (mid-left)
  const bioNodes = useMemo(() => [
    { x: LCX - 30, y: CY2 - 26, s: 0.9 },
    { x: LCX + 16, y: CY2 - 38, s: 0.7 + symmetry * 0.4 },
    { x: LCX + 34, y: CY2 - 5,  s: 1.0 },
    { x: LCX + 22, y: CY2 + 28, s: 0.8 },
    { x: LCX - 26, y: CY2 + 20, s: 0.65 + symmetry * 0.3 },
    { x: LCX - 6,  y: CY2 + 6,  s: 0.5 + symmetry * 0.5 },
  ], [symmetry]);
  const bioEdges = [[0,1],[1,2],[2,3],[3,4],[4,5],[5,0],[0,3]];

  // INSIGHTS constellation (mid-right)
  const patNodes = useMemo(() => [
    { x: RCX - 32, y: CY2 - 32, s: 0.8 + pattern * 0.4 },
    { x: RCX + 8,  y: CY2 - 40, s: 0.9 },
    { x: RCX + 34, y: CY2 - 16, s: 0.7 + pattern * 0.3 },
    { x: RCX + 28, y: CY2 + 22, s: 1.0 },
    { x: RCX - 6,  y: CY2 + 36, s: 0.8 },
    { x: RCX - 36, y: CY2 + 10, s: 0.65 + pattern * 0.3 },
    { x: RCX + 4,  y: CY2 + 4,  s: 0.5 + pattern * 0.5 },
    { x: RCX - 18, y: CY2 - 14, s: 0.7 },
  ], [pattern]);
  const patEdges = [[0,1],[1,2],[2,3],[3,4],[4,5],[5,0],[1,7],[7,6],[6,3]];

  // CORRELATIONS scatter (bot-right)
  const corrNodes = useMemo(() => [
    { x: RCX - 24, y: CY3 - 18, s: 0.9 },
    { x: RCX + 8,  y: CY3 - 26, s: 0.7 + presence * 0.3 },
    { x: RCX + 28, y: CY3 - 4,  s: 0.85 },
    { x: RCX + 14, y: CY3 + 20, s: 1.0 },
    { x: RCX - 28, y: CY3 + 14, s: 0.75 },
    { x: RCX - 4,  y: CY3 + 6,  s: 0.6 + presence * 0.4 },
  ], [presence]);
  const corrEdges = [[0,1],[1,2],[2,3],[3,4],[4,5],[0,5],[1,5],[2,5]];

  // Wave paths (memoised — shapes are static, only data changes)
  const alertPath = useMemo(() => squiggle(10, CY1 + 8, 90, 7), []);
  const oppPath   = useMemo(() => sineWave(DIV_X2 + 10, CY1 + 20, 90, 7, 3.2), []);
  const hrvPath   = useMemo(
    () => sineWave(DIV_X1 + 14, CY3 + 14, DIV_X2 - DIV_X1 - 28, 9, 2.5),
    []
  );
  const recvPath  = useMemo(() => sineWave(10, CY3 + 18, 90, 6, 2), []);

  return (
    <section style={{ width: "100%", height: "100%", display: "flex", background: "transparent" }}>
      <svg
        viewBox={`0 0 ${VW} ${VH}`}
        preserveAspectRatio="xMidYMid meet"
        width="100%"
        height="100%"
        aria-label="Consciousness dashboard"
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
          <filter id="gt" x="-20%" y="-50%" width="140%" height="200%">
            <feGaussianBlur stdDeviation="2.5" result="b"/>
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
          <linearGradient id="recvG" x1="0%" y1="0%" x2="100%" y2="0%">
            <stop offset="0%"   stopColor={EMERALD} stopOpacity="0.5"/>
            <stop offset="100%" stopColor={EMERALD} stopOpacity="0.85"/>
          </linearGradient>
          <linearGradient id="alertG" x1="0%" y1="0%" x2="100%" y2="0%">
            <stop offset="0%"   stopColor={VIOLET} stopOpacity="0.5"/>
            <stop offset="100%" stopColor={VIOLET} stopOpacity="0.9"/>
          </linearGradient>

          <style>{`
            .cd-pulse { animation: cdp 3.5s ease-in-out infinite; }
            @keyframes cdp { 0%,100%{opacity:1} 50%{opacity:0.58} }
            .cd-spin  { animation: cds 90s linear infinite; }
            @keyframes cds  { from{transform:rotate(0deg)} to{transform:rotate(360deg)} }
          `}</style>
        </defs>

        {/* ── Card border ─────────────────────────────────────────────────── */}
        <rect x="0.5" y="0.5" width={VW - 1} height={VH - 1}
          rx="10" ry="10" fill="none"
          stroke={GOLD} strokeWidth="0.8" opacity="0.38"
        />

        {/* ── Title ───────────────────────────────────────────────────────── */}
        <text x={MCX} y={19}
          textAnchor="middle"
          fontFamily="var(--font-display,'Panchang',monospace)"
          fontSize="9" fontWeight="800" letterSpacing="4.5"
          fill={GOLD} filter="url(#gt)" opacity="0.9"
        >CONSCIOUSNESS DASHBOARD</text>

        {/* ── Grid dividers ───────────────────────────────────────────────── */}
        <line x1="1" y1={GRID_T} x2={VW-1} y2={GRID_T} stroke={GOLD} strokeWidth="0.5" opacity="0.28"/>
        <line x1={DIV_X1} y1={GRID_T} x2={DIV_X1} y2={GRID_B} stroke={GOLD} strokeWidth="0.5" opacity="0.28"/>
        <line x1={DIV_X2} y1={GRID_T} x2={DIV_X2} y2={GRID_B} stroke={GOLD} strokeWidth="0.5" opacity="0.28"/>
        {/* Mid divider — left & right columns only (center spans rows 1+2) */}
        <line x1="1" y1={DIV_Y1} x2={DIV_X1} y2={DIV_Y1} stroke={GOLD} strokeWidth="0.5" opacity="0.28"/>
        <line x1={DIV_X2} y1={DIV_Y1} x2={VW-1} y2={DIV_Y1} stroke={GOLD} strokeWidth="0.5" opacity="0.28"/>
        {/* Row-3 top — full width */}
        <line x1="1" y1={DIV_Y2} x2={VW-1} y2={DIV_Y2} stroke={GOLD} strokeWidth="0.5" opacity="0.28"/>
        <line x1="1" y1={GRID_B} x2={VW-1} y2={GRID_B} stroke={GOLD} strokeWidth="0.3" opacity="0.18"/>

        {/* ══ CENTRAL MANDALA ════════════════════════════════════════════════ */}
        {/* Ambient radial glow */}
        <ellipse cx={MCX} cy={MCY} rx={MR_OUTER * 1.08} ry={MR_OUTER * 1.04}
          fill="url(#mGlow)"
        />

        {/* Outer thin gold circle */}
        <circle cx={MCX} cy={MCY} r={MR_OUTER}
          fill="none" stroke={GOLD} strokeWidth="0.55" opacity="0.30"
        />

        {/* Outer dashed-dot ring (slow spin) */}
        <g className="cd-spin" style={{ transformOrigin: `${MCX}px ${MCY}px` }}>
          {outerDots.map((d, i) => (
            <circle key={`od${i}`} cx={d.x} cy={d.y}
              r={i % 5 === 0 ? 1.4 : 1.0}
              fill={GOLD} opacity={i % 5 === 0 ? 0.48 : 0.26}
            />
          ))}
        </g>

        {/* Inner dashed-dot ring (static) */}
        {innerDots.map((d, i) => (
          <circle key={`id${i}`} cx={d.x} cy={d.y} r="0.8"
            fill={GOLD} opacity={i % 4 === 0 ? 0.34 : 0.16}
          />
        ))}

        {/* 8 sharp star/compass tips */}
        {Array.from({ length: 8 }, (_, i) => (
          <path key={`st${i}`}
            d={diamond(MCX, MCY, i * 45 - 90, MSTAR_LEN, MSTAR_WID)}
            fill="none" stroke={GOLD} strokeWidth="0.75" opacity="0.54"
          />
        ))}

        {/* 4 outer plump lotus petals (N/S/E/W) */}
        {[0, 90, 180, 270].map((deg, i) => (
          <path key={`lp1${i}`}
            d={lotus(MCX, MCY, deg - 90, MLP1_LEN, MLP1_WID)}
            fill="none" stroke={GOLD} strokeWidth="1.0" opacity="0.72"
          />
        ))}

        {/* 4 mid lotus petals (NE/NW/SE/SW) */}
        {[45, 135, 225, 315].map((deg, i) => (
          <path key={`lp2${i}`}
            d={lotus(MCX, MCY, deg - 90, MLP2_LEN, MLP2_WID)}
            fill="none" stroke={GOLD} strokeWidth="0.85" opacity="0.62"
          />
        ))}

        {/* 4 tiny core petals */}
        {[0, 90, 180, 270].map((deg, i) => (
          <path key={`lp3${i}`}
            d={lotus(MCX, MCY, deg - 90, MLP3_LEN, MLP3_WID)}
            fill="none" stroke={GOLD} strokeWidth="0.7" opacity="0.84"
          />
        ))}

        {/* Center glow + dot */}
        <circle cx={MCX} cy={MCY} r="9"
          fill={stateColor} opacity="0.14" filter="url(#g4)"
        />
        <circle cx={MCX} cy={MCY} r="3.5"
          fill={stateColor} opacity="0.92" className="cd-pulse"
        />

        {/* State word + coherence % */}
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
          fill={PARCHMENT} opacity="0.32"
        >{cohPct}%</text>

        {/* ══ PANEL: top-left — ALERTS ═══════════════════════════════════════ */}
        <text x="10" y={GRID_T + 13}
          fontFamily="var(--font-display,'Panchang',monospace)"
          fontSize="7" fontWeight="700" letterSpacing="2"
          fill={PARCHMENT} opacity="0.55"
        >ALERTS</text>

        <path d={alertPath}
          fill="none" stroke="url(#alertG)" strokeWidth="1.3"
          filter="url(#g2)"
        />

        <text x="10" y={DIV_Y1 - 14}
          fontFamily="var(--font-display,'Panchang',monospace)"
          fontSize="7" fontWeight="500" letterSpacing="0.5"
          fill={alertColor} opacity="0.80"
        >{alertText}</text>

        {/* ══ PANEL: top-right — OPPORTUNITIES ══════════════════════════════ */}
        <text x={VW - 10} y={GRID_T + 13}
          textAnchor="end"
          fontFamily="var(--font-display,'Panchang',monospace)"
          fontSize="7" fontWeight="700" letterSpacing="2"
          fill={PARCHMENT} opacity="0.55"
        >OPPORTUNITIES</text>

        <text x={RCX} y={CY1 - 4}
          textAnchor="middle"
          fontFamily="var(--font-display,'Panchang',monospace)"
          fontSize="22" fontWeight="700"
          fill={PARCHMENT} opacity="0.82"
        >{sigCount}</text>
        <text x={RCX} y={CY1 + 11}
          textAnchor="middle"
          fontFamily="var(--font-display,'Panchang',monospace)"
          fontSize="6.5" letterSpacing="1.5"
          fill={PARCHMENT} opacity="0.42"
        >signals</text>

        <path d={oppPath}
          fill="none" stroke="url(#alertG)" strokeWidth="1.1" opacity="0.75"
        />

        {/* ══ PANEL: mid-left — BIOFIELD CORRELATIONS ═══════════════════════ */}
        <text x="10" y={DIV_Y1 + 13}
          fontFamily="var(--font-display,'Panchang',monospace)"
          fontSize="6.5" fontWeight="700" letterSpacing="1.5"
          fill={PARCHMENT} opacity="0.55"
        >BIOFIELD</text>
        <text x="10" y={DIV_Y1 + 23}
          fontFamily="var(--font-display,'Panchang',monospace)"
          fontSize="6.5" fontWeight="700" letterSpacing="1.5"
          fill={PARCHMENT} opacity="0.55"
        >CORRELATIONS</text>

        {bioEdges.map(([a, b], i) => (
          <line key={`be${i}`}
            x1={bioNodes[a].x} y1={bioNodes[a].y}
            x2={bioNodes[b].x} y2={bioNodes[b].y}
            stroke={GOLD} strokeWidth="0.45" opacity="0.30"
          />
        ))}
        {bioNodes.map((n, i) => (
          <circle key={`bn${i}`} cx={n.x} cy={n.y} r={n.s * 1.8}
            fill={GOLD} opacity={0.35 + symmetry * 0.45}
          />
        ))}

        {/* ══ PANEL: mid-right — INSIGHTS ════════════════════════════════════ */}
        {patEdges.map(([a, b], i) => (
          <line key={`pe${i}`}
            x1={patNodes[a].x} y1={patNodes[a].y}
            x2={patNodes[b].x} y2={patNodes[b].y}
            stroke={GOLD} strokeWidth="0.4" opacity="0.28"
          />
        ))}
        {patNodes.map((n, i) => (
          <circle key={`pn${i}`} cx={n.x} cy={n.y} r={n.s * 1.8}
            fill={GOLD} opacity={0.30 + pattern * 0.50}
          />
        ))}
        <text x={VW - 10} y={DIV_Y2 - 12}
          textAnchor="end"
          fontFamily="var(--font-display,'Panchang',monospace)"
          fontSize="7" fontWeight="700" letterSpacing="2"
          fill={PARCHMENT} opacity="0.55"
        >INSIGHTS</text>

        {/* ══ PANEL: bot-left — STRESS RECOVERY ═════════════════════════════ */}
        <text x="10" y={DIV_Y2 + 13}
          fontFamily="var(--font-display,'Panchang',monospace)"
          fontSize="6.5" fontWeight="700" letterSpacing="1.5"
          fill={PARCHMENT} opacity="0.55"
        >STRESS</text>
        <text x="10" y={DIV_Y2 + 23}
          fontFamily="var(--font-display,'Panchang',monospace)"
          fontSize="6.5" fontWeight="700" letterSpacing="1.5"
          fill={PARCHMENT} opacity="0.55"
        >RECOVERY</text>

        <text x="12" y={CY3 + 4}
          fontFamily="var(--font-display,'Panchang',monospace)"
          fontSize="22" fontWeight="700"
          fill={EMERALD} filter="url(#g2)"
        >{recovVal}</text>

        <path d={recvPath}
          fill="none" stroke="url(#recvG)" strokeWidth="1.4"
        />

        {/* ══ PANEL: bot-center — HRV COHERENCE ═════════════════════════════ */}
        <text x={MCX} y={DIV_Y2 + 14}
          textAnchor="middle"
          fontFamily="var(--font-display,'Panchang',monospace)"
          fontSize="8" fontWeight="700" letterSpacing="3.5"
          fill={GOLD} opacity="0.72"
        >HRV COHERENCE</text>

        <path d={hrvPath}
          fill="none" stroke="url(#hrvG)" strokeWidth="1.5"
        />
        <circle r="2.5" fill={stateColor} opacity="0.9" filter="url(#g2)">
          <animateMotion dur="4.5s" repeatCount="indefinite" path={hrvPath}/>
        </circle>

        {/* ══ PANEL: bot-right — CORRELATIONS ════════════════════════════════ */}
        <text x={VW - 10} y={DIV_Y2 + 14}
          textAnchor="end"
          fontFamily="var(--font-display,'Panchang',monospace)"
          fontSize="7" fontWeight="400" letterSpacing="1"
          fill={PARCHMENT} opacity="0.42"
        >Correlations</text>

        {corrEdges.map(([a, b], i) => (
          <line key={`ce${i}`}
            x1={corrNodes[a].x} y1={corrNodes[a].y}
            x2={corrNodes[b].x} y2={corrNodes[b].y}
            stroke={GOLD} strokeWidth="0.45" opacity="0.32"
          />
        ))}
        {corrNodes.map((n, i) => (
          <circle key={`cn${i}`} cx={n.x} cy={n.y} r={n.s * 2}
            fill={GOLD} opacity={0.45 + presence * 0.40}
          />
        ))}
      </svg>
    </section>
  );
}
