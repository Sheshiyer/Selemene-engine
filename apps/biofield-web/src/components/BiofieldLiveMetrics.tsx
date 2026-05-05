"use client";

import React from "react";
import type { CompositeScores } from "./pip/types";

// ─── Consciousness Color Spectrum labels ──────────────────────────────────────
const METRIC_MAP = [
  { key: "overallCoherence",    label: "COHERENCE",  unit: "field",    color: "var(--c-indigo)"  },
  { key: "bodySymmetry",        label: "SYMMETRY",   unit: "lateral",  color: "var(--c-emerald)" },
  { key: "lightQuantaDensity",  label: "LUMINANCE",  unit: "quanta",   color: "var(--c-gold)"    },
  { key: "patternRegularity",   label: "REGULARITY", unit: "pattern",  color: "var(--c-violet)"  },
  { key: "normalizedArea",      label: "PRESENCE",   unit: "frame",    color: "var(--accent)"    },
] as const;

// ─── Individual metric row ─────────────────────────────────────────────────────
function MetricRow({ label, unit, value, color }: { label: string; unit: string; value: number; color: string }) {
  const pct = Math.round(Math.max(0, Math.min(1, value)) * 100);
  return (
    <div style={{ display: "flex", flexDirection: "column", gap: 4 }}>
      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "baseline" }}>
        <div style={{ display: "flex", alignItems: "center", gap: 6 }}>
          <span style={{ width: 3, height: 3, borderRadius: "50%", background: color, flexShrink: 0 }} />
          <span style={{ fontFamily: "var(--font-mono)", fontSize: "0.6rem", fontWeight: 600, letterSpacing: "0.12em", color: "var(--muted)", textTransform: "uppercase" }}>
            {label}
          </span>
          <span style={{ fontFamily: "var(--font-mono)", fontSize: "0.56rem", color: "rgba(240,240,243,0.22)", textTransform: "lowercase" }}>
            {unit}
          </span>
        </div>
        <span style={{ fontFamily: "var(--font-mono)", fontSize: "0.78rem", fontWeight: 700, letterSpacing: "-0.04em", color: pct >= 75 ? color : "var(--text-2)", tabularNums: true } as React.CSSProperties}>
          {String(pct).padStart(3, "\u2007")}<span style={{ fontSize: "0.56rem", opacity: 0.45 }}>%</span>
        </span>
      </div>
      <div style={{ height: 2, borderRadius: 9999, background: "rgba(255,255,255,0.06)", overflow: "hidden" }}>
        <div style={{
          height: "100%", borderRadius: 9999, width: `${pct}%`,
          background: color, opacity: pct >= 75 ? 1 : 0.55,
          transition: "width 0.6s cubic-bezier(0.34,1.56,0.64,1), opacity 0.4s ease",
        }} />
      </div>
    </div>
  );
}

// ─── Arc gauge (kept for other usages) ───────────────────────────────────────
const RADIUS = 34;
const CIRC = 2 * Math.PI * RADIUS;
const ARC_FRACTION = 0.75;

export function ArcGauge({ value, label, accent }: {
  value: number;
  label: string;
  sublabel?: string;
  accent?: "signal" | "accent";
}) {
  const pct = Math.max(0, Math.min(1, value));
  const filled = CIRC * ARC_FRACTION * pct;
  const track  = CIRC * ARC_FRACTION;
  const gap    = CIRC * (1 - ARC_FRACTION);
  const color  = accent === "signal" ? "var(--signal)" : "var(--accent)";
  const glowColor = accent === "signal" ? "rgba(255,179,71,0.35)" : "rgba(124,124,255,0.35)";

  return (
    <div style={{ display: "flex", flexDirection: "column", alignItems: "center", gap: "0.3rem" }}>
      <div style={{ position: "relative", width: 88, height: 88 }}>
        <svg viewBox="0 0 88 88" width={88} height={88} style={{ overflow: "visible" }}>
          <circle cx={44} cy={44} r={RADIUS} fill="none" stroke="rgba(255,255,255,0.06)" strokeWidth={6}
            strokeDasharray={`${track} ${gap}`} strokeLinecap="round" transform="rotate(135 44 44)" />
          <circle cx={44} cy={44} r={RADIUS} fill="none" stroke={color} strokeWidth={6}
            strokeDasharray={`${filled} ${CIRC - filled}`} strokeLinecap="round" transform="rotate(135 44 44)"
            style={{ filter: `drop-shadow(0 0 6px ${glowColor})`, transition: "stroke-dasharray 0.7s cubic-bezier(0.34,1.56,0.64,1)" }} />
        </svg>
        <div style={{ position: "absolute", inset: 0, display: "flex", alignItems: "center", justifyContent: "center" }}>
          <span style={{ fontFamily: "var(--font-mono)", fontSize: "1.05rem", fontWeight: 700, letterSpacing: "-0.04em", color: pct > 0.7 ? color : "var(--text)" }}>
            {Math.round(pct * 100)}
          </span>
        </div>
      </div>
      <span style={{ fontFamily: "var(--font-mono)", fontSize: "0.68rem", fontWeight: 600, letterSpacing: "0.08em", textTransform: "uppercase", color: "var(--muted)", textAlign: "center" }}>
        {label}
      </span>
    </div>
  );
}

// ─── Coherence bar ────────────────────────────────────────────────────────────
export function CoherenceBar({ value }: { value: number }) {
  const pct = Math.round(value * 100);
  const tier =
    pct >= 80 ? { label: "Coherent",    color: "var(--c-indigo)" } :
    pct >= 55 ? { label: "Stabilising", color: "var(--signal)"   } :
                { label: "Calibrating", color: "var(--muted)"    };

  return (
    <div style={{ display: "flex", flexDirection: "column", gap: "0.45rem" }}>
      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
        <div style={{ display: "flex", alignItems: "center", gap: 6 }}>
          <span style={{ width: 5, height: 5, borderRadius: "50%", background: tier.color, animation: "pulse-dot 1.8s ease-in-out infinite" }} />
          <span style={{ fontFamily: "var(--font-mono)", fontSize: "0.6rem", fontWeight: 600, letterSpacing: "0.12em", textTransform: "uppercase", color: "var(--muted)" }}>
            Field coherence
          </span>
        </div>
        <div style={{ display: "flex", alignItems: "baseline", gap: 5 }}>
          <span style={{ fontFamily: "var(--font-mono)", fontSize: "0.68rem", color: tier.color, fontWeight: 600 }}>{tier.label}</span>
          <span style={{ fontFamily: "var(--font-mono)", fontSize: "0.92rem", fontWeight: 700, letterSpacing: "-0.04em", color: "var(--text)" }}>
            {pct}<span style={{ fontSize: "0.58rem", opacity: 0.45 }}>%</span>
          </span>
        </div>
      </div>
      <div style={{ height: 3, borderRadius: 9999, background: "rgba(255,255,255,0.06)", overflow: "hidden" }}>
        <div style={{
          height: "100%", borderRadius: 9999, width: `${pct}%`,
          background: `linear-gradient(90deg, ${tier.color} 0%, rgba(255,255,255,0.65) 100%)`,
          transition: "width 0.7s cubic-bezier(0.34,1.56,0.64,1)",
        }} />
      </div>
    </div>
  );
}

// ─── Main panel ───────────────────────────────────────────────────────────────
export function BiofieldLiveMetrics({ scores }: { scores: CompositeScores }) {
  return (
    <section style={{
      padding: "1.1rem 1.2rem",
      borderRadius: "var(--r-xl)",
      background: "var(--panel)",
      border: "1px solid var(--line-faint)",
      boxShadow: "inset 0 1px 0 rgba(255,255,255,0.04)",
      display: "flex",
      flexDirection: "column",
      gap: "0.85rem",
    }}>
      {/* Header */}
      <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between" }}>
        <div style={{ display: "flex", alignItems: "center", gap: 6 }}>
          <span style={{
            width: 6, height: 6, borderRadius: "50%",
            background: "var(--c-indigo)",
            animation: "pulse-dot 2s ease-in-out infinite",
          }} />
          <p style={{ margin: 0, fontFamily: "var(--font-mono)", fontSize: "0.62rem", fontWeight: 600, letterSpacing: "0.14em", textTransform: "uppercase", color: "var(--muted)" }}>
            Live field metrics
          </p>
        </div>
        <span style={{
          fontFamily: "var(--font-mono)", fontSize: "0.58rem", fontWeight: 600, letterSpacing: "0.1em",
          textTransform: "uppercase", padding: "0.18rem 0.5rem",
          borderRadius: "var(--r-pill)",
          background: "rgba(99,102,241,0.1)",
          border: "1px solid rgba(99,102,241,0.22)",
          color: "var(--c-indigo)",
        }}>
          Real-time
        </span>
      </div>

      {/* Coherence bar */}
      <CoherenceBar value={scores.overallCoherence} />

      <div style={{ width: "100%", height: 1, background: "var(--line-faint)" }} />

      {/* All 5 metric rows */}
      <div style={{ display: "flex", flexDirection: "column", gap: "0.65rem" }}>
        {METRIC_MAP.map(({ key, label, unit, color }) => (
          <MetricRow
            key={key}
            label={label}
            unit={unit}
            value={scores[key]}
            color={color}
          />
        ))}
      </div>
    </section>
  );
}

