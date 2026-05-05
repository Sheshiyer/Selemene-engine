"use client";

import React from "react";
import type { CompositeScores } from "./pip/types";

// ─── Consciousness Color Spectrum (Goethe / Kha-Ba-La) ────────────────────────
const METRIC_MAP = [
  { key: "overallCoherence",   label: "COHERENCE",  unit: "field",   color: "var(--c-indigo)",  glow: "rgba(11,80,251,0.4)"    },
  { key: "bodySymmetry",       label: "SYMMETRY",   unit: "lateral", color: "var(--c-emerald)", glow: "rgba(16,181,167,0.4)"   },
  { key: "lightQuantaDensity", label: "LUMINANCE",  unit: "quanta",  color: "var(--c-gold)",    glow: "rgba(197,160,23,0.4)"   },
  { key: "patternRegularity",  label: "REGULARITY", unit: "pattern", color: "var(--c-violet)",  glow: "rgba(45,0,80,0.8)"      },
  { key: "normalizedArea",     label: "PRESENCE",   unit: "frame",   color: "var(--c-indigo)",  glow: "rgba(11,80,251,0.25)"   },
] as const;

// ─── Single metric row — bioluminescent bar ────────────────────────────────────
function MetricRow({
  label, unit, value, color, glow,
}: { label: string; unit: string; value: number; color: string; glow: string }) {
  const pct = Math.round(Math.max(0, Math.min(1, value)) * 100);
  const active = pct >= 65;
  return (
    <div style={{ display: "flex", flexDirection: "column", gap: 5 }}>
      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "baseline" }}>
        {/* Label */}
        <div style={{ display: "flex", alignItems: "center", gap: 6 }}>
          <span style={{
            width: 4, height: 4, borderRadius: "50%",
            background: color,
            boxShadow: active ? `0 0 6px ${glow}` : "none",
            flexShrink: 0,
            transition: "box-shadow 0.4s ease",
          }} />
          <span style={{
            fontFamily: "var(--font-display)",
            fontSize: "0.58rem", fontWeight: 700,
            letterSpacing: "0.16em",
            color: active ? "var(--text-2)" : "var(--muted)",
            textTransform: "uppercase",
            transition: "color 0.4s ease",
          }}>
            {label}
          </span>
          <span style={{
            fontFamily: "var(--font-mono)",
            fontSize: "0.52rem",
            color: "rgba(240,237,227,0.2)",
            textTransform: "lowercase",
          }}>
            {unit}
          </span>
        </div>
        {/* Value */}
        <span style={{
          fontFamily: "var(--font-mono)",
          fontSize: "0.85rem", fontWeight: 700,
          letterSpacing: "-0.04em",
          color: active ? color : "var(--muted)",
          textShadow: active ? `0 0 12px ${glow}` : "none",
          transition: "color 0.4s ease, text-shadow 0.4s ease",
        } as React.CSSProperties}>
          {String(pct).padStart(3, "\u2007")}
          <span style={{ fontSize: "0.5rem", opacity: 0.45, fontWeight: 400 }}>%</span>
        </span>
      </div>
      {/* Bioluminescent bar */}
      <div style={{
        height: 2,
        borderRadius: 9999,
        background: "rgba(11,80,251,0.08)",
        overflow: "hidden",
        position: "relative",
      }}>
        <div style={{
          height: "100%", borderRadius: 9999,
          width: `${pct}%`,
          background: color,
          boxShadow: active ? `0 0 8px ${glow}` : "none",
          opacity: active ? 1 : 0.4,
          transition: "width 0.7s cubic-bezier(0.34,1.56,0.64,1), box-shadow 0.4s ease, opacity 0.4s ease",
        }} />
      </div>
    </div>
  );
}

// ─── Arc gauge (kept for external usage) ──────────────────────────────────────
const RADIUS = 34;
const CIRC = 2 * Math.PI * RADIUS;
const ARC_FRACTION = 0.75;

export function ArcGauge({ value, label, accent }: {
  value: number; label: string; sublabel?: string; accent?: "signal" | "accent";
}) {
  const pct  = Math.max(0, Math.min(1, value));
  const filled = CIRC * ARC_FRACTION * pct;
  const track  = CIRC * ARC_FRACTION;
  const gap    = CIRC * (1 - ARC_FRACTION);
  const color     = accent === "signal" ? "var(--c-gold)"   : "var(--c-indigo)";
  const glowColor = accent === "signal" ? "rgba(197,160,23,0.4)" : "rgba(11,80,251,0.4)";

  return (
    <div style={{ display: "flex", flexDirection: "column", alignItems: "center", gap: "0.3rem" }}>
      <div style={{ position: "relative", width: 88, height: 88 }}>
        <svg viewBox="0 0 88 88" width={88} height={88} style={{ overflow: "visible" }}>
          <circle cx={44} cy={44} r={RADIUS} fill="none" stroke="rgba(11,80,251,0.08)" strokeWidth={6}
            strokeDasharray={`${track} ${gap}`} strokeLinecap="round" transform="rotate(135 44 44)" />
          <circle cx={44} cy={44} r={RADIUS} fill="none" stroke={color} strokeWidth={6}
            strokeDasharray={`${filled} ${CIRC - filled}`} strokeLinecap="round" transform="rotate(135 44 44)"
            style={{ filter: `drop-shadow(0 0 6px ${glowColor})`, transition: "stroke-dasharray 0.7s cubic-bezier(0.34,1.56,0.64,1)" }} />
        </svg>
        <div style={{ position: "absolute", inset: 0, display: "flex", alignItems: "center", justifyContent: "center" }}>
          <span style={{
            fontFamily: "var(--font-mono)", fontSize: "1.05rem", fontWeight: 700,
            letterSpacing: "-0.04em", color: pct > 0.7 ? color : "var(--text)",
          }}>
            {Math.round(pct * 100)}
          </span>
        </div>
      </div>
      <span style={{
        fontFamily: "var(--font-display)", fontSize: "0.58rem", fontWeight: 700,
        letterSpacing: "0.12em", textTransform: "uppercase",
        color: "var(--muted)", textAlign: "center",
      }}>
        {label}
      </span>
    </div>
  );
}

// ─── Coherence bar (field-level indicator) ────────────────────────────────────
export function CoherenceBar({ value }: { value: number }) {
  const pct = Math.round(value * 100);
  const tier =
    pct >= 80 ? { label: "Coherent",    color: "var(--c-emerald)", glow: "rgba(16,181,167,0.5)" } :
    pct >= 55 ? { label: "Stabilising", color: "var(--c-gold)",    glow: "rgba(197,160,23,0.5)"  } :
                { label: "Calibrating", color: "var(--muted)",     glow: "none"                  };

  return (
    <div style={{ display: "flex", flexDirection: "column", gap: "0.5rem" }}>
      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
        <div style={{ display: "flex", alignItems: "center", gap: 6 }}>
          <span style={{
            width: 5, height: 5, borderRadius: "50%",
            background: tier.color,
            boxShadow: `0 0 8px ${tier.glow}`,
            animation: "pulse-dot 1.8s ease-in-out infinite",
          }} />
          <span style={{
            fontFamily: "var(--font-display)",
            fontSize: "0.6rem", fontWeight: 700,
            letterSpacing: "0.14em", textTransform: "uppercase",
            color: "var(--muted)",
          }}>
            Field coherence
          </span>
        </div>
        <div style={{ display: "flex", alignItems: "baseline", gap: 6 }}>
          <span style={{
            fontFamily: "var(--font-display)",
            fontSize: "0.62rem", fontWeight: 600,
            letterSpacing: "0.1em",
            color: tier.color,
            textShadow: `0 0 10px ${tier.glow}`,
          }}>
            {tier.label}
          </span>
          <span style={{
            fontFamily: "var(--font-mono)", fontSize: "0.95rem", fontWeight: 700,
            letterSpacing: "-0.04em", color: "var(--text)",
          }}>
            {pct}<span style={{ fontSize: "0.56rem", opacity: 0.45 }}>%</span>
          </span>
        </div>
      </div>
      <div style={{ height: 3, borderRadius: 9999, background: "rgba(11,80,251,0.08)", overflow: "hidden" }}>
        <div style={{
          height: "100%", borderRadius: 9999, width: `${pct}%`,
          background: `linear-gradient(90deg, ${tier.color} 0%, rgba(240,237,227,0.6) 100%)`,
          boxShadow: `0 0 10px ${tier.glow}`,
          transition: "width 0.7s cubic-bezier(0.34,1.56,0.64,1)",
        }} />
      </div>
    </div>
  );
}

// ─── Main metrics panel ───────────────────────────────────────────────────────
export function BiofieldLiveMetrics({ scores }: { scores: CompositeScores }) {
  return (
    <section style={{
      padding: "1.1rem 1.2rem 1rem",
      borderRadius: "var(--r-lg)",
      background: "linear-gradient(160deg, rgba(45,0,80,0.18) 0%, rgba(11,80,251,0.05) 100%)",
      border: "1px solid rgba(11,80,251,0.14)",
      boxShadow: "inset 0 1px 0 rgba(197,160,23,0.06)",
      display: "flex",
      flexDirection: "column",
      gap: "0.85rem",
    }}>
      {/* ── Panel header ── */}
      <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between" }}>
        <div style={{ display: "flex", alignItems: "center", gap: 7 }}>
          <span style={{
            width: 6, height: 6, borderRadius: "50%",
            background: "var(--c-indigo)",
            boxShadow: "0 0 8px rgba(11,80,251,0.7)",
            animation: "pulse-dot 2s ease-in-out infinite",
          }} />
          <p style={{
            margin: 0,
            fontFamily: "var(--font-display)",
            fontSize: "0.65rem", fontWeight: 700,
            letterSpacing: "0.18em", textTransform: "uppercase",
            color: "var(--muted)",
          }}>
            Live field metrics
          </p>
        </div>
        <span style={{
          fontFamily: "var(--font-display)", fontSize: "0.56rem", fontWeight: 700,
          letterSpacing: "0.12em", textTransform: "uppercase",
          padding: "0.16rem 0.55rem",
          borderRadius: "var(--r-pill)",
          background: "rgba(197,160,23,0.08)",
          border: "1px solid rgba(197,160,23,0.22)",
          color: "var(--c-gold)",
          textShadow: "0 0 10px rgba(197,160,23,0.4)",
        }}>
          Real-time
        </span>
      </div>

      {/* ── Field coherence master bar ── */}
      <CoherenceBar value={scores.overallCoherence} />

      {/* ── Divider (Ba Arc gradient) ── */}
      <div style={{ height: 1, background: "linear-gradient(90deg, transparent, rgba(16,181,167,0.2), rgba(197,160,23,0.2), transparent)" }} />

      {/* ── 5 bioluminescent metric rows ── */}
      <div style={{ display: "flex", flexDirection: "column", gap: "0.7rem" }}>
        {METRIC_MAP.map(({ key, label, unit, color, glow }) => (
          <MetricRow
            key={key}
            label={label}
            unit={unit}
            value={scores[key]}
            color={color}
            glow={glow}
          />
        ))}
      </div>
    </section>
  );
}
