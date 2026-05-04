"use client";

import type { CompositeScores } from "./pip/types";

// ─── Arc gauge ────────────────────────────────────────────────────────────────
// Draws a 270° arc (from 135° to 45° going clockwise) as a progress indicator.
const RADIUS = 34;
const CIRC = 2 * Math.PI * RADIUS;
const ARC_FRACTION = 0.75; // 270° / 360°

function ArcGauge({ value, label, sublabel, accent }: {
  value: number;   // 0–1
  label: string;
  sublabel?: string;
  accent?: "signal" | "accent";
}) {
  const pct = Math.max(0, Math.min(1, value));
  const filled = CIRC * ARC_FRACTION * pct;
  const track  = CIRC * ARC_FRACTION;
  const gap    = CIRC * (1 - ARC_FRACTION);

  const color = accent === "signal"
    ? "var(--signal)"
    : "var(--accent)";

  const glowColor = accent === "signal"
    ? "rgba(255,179,71,0.35)"
    : "rgba(124,124,255,0.35)";

  return (
    <div style={{ display: "flex", flexDirection: "column", alignItems: "center", gap: "0.3rem" }}>
      <div style={{ position: "relative", width: 88, height: 88 }}>
        <svg viewBox="0 0 88 88" width={88} height={88} style={{ overflow: "visible" }}>
          <defs>
            <filter id={`glow-${label}`} x="-50%" y="-50%" width="200%" height="200%">
              <feGaussianBlur stdDeviation="2.5" result="blur" />
              <feMerge><feMergeNode in="blur" /><feMergeNode in="SourceGraphic" /></feMerge>
            </filter>
          </defs>
          {/* Track */}
          <circle
            cx={44} cy={44} r={RADIUS}
            fill="none"
            stroke="rgba(255,255,255,0.06)"
            strokeWidth={6}
            strokeDasharray={`${track} ${gap}`}
            strokeLinecap="round"
            transform="rotate(135 44 44)"
          />
          {/* Fill */}
          <circle
            cx={44} cy={44} r={RADIUS}
            fill="none"
            stroke={color}
            strokeWidth={6}
            strokeDasharray={`${filled} ${CIRC - filled}`}
            strokeLinecap="round"
            transform="rotate(135 44 44)"
            style={{
              filter: `drop-shadow(0 0 6px ${glowColor})`,
              transition: "stroke-dasharray 0.7s cubic-bezier(0.34,1.56,0.64,1)",
            }}
          />
        </svg>
        {/* Centre value */}
        <div style={{
          position: "absolute", inset: 0,
          display: "flex", flexDirection: "column",
          alignItems: "center", justifyContent: "center",
          gap: "0px",
        }}>
          <span style={{
            fontSize: "1.05rem", fontWeight: 700,
            letterSpacing: "-0.04em",
            color: pct > 0.7 ? color : "var(--text)",
            transition: "color 0.4s ease",
          }}>
            {Math.round(pct * 100)}
          </span>
        </div>
      </div>
      <span style={{ fontSize: "0.68rem", fontWeight: 600, letterSpacing: "0.08em", textTransform: "uppercase", color: "var(--muted)", textAlign: "center" }}>
        {label}
      </span>
      {sublabel && (
        <span style={{ fontSize: "0.6rem", color: "rgba(240,240,243,0.28)", textAlign: "center" }}>
          {sublabel}
        </span>
      )}
    </div>
  );
}

// ─── Coherence bar (full-width horizontal) ───────────────────────────────────
function CoherenceBar({ value }: { value: number }) {
  const pct = Math.round(value * 100);
  const tier =
    pct >= 80 ? { label: "Coherent", color: "var(--accent)" } :
    pct >= 55 ? { label: "Stabilising", color: "var(--signal)" } :
                { label: "Calibrating", color: "var(--muted)" };

  return (
    <div style={{ display: "flex", flexDirection: "column", gap: "0.5rem" }}>
      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "baseline" }}>
        <span style={{ fontSize: "0.7rem", fontWeight: 600, letterSpacing: "0.1em", textTransform: "uppercase", color: "var(--muted)" }}>
          Field coherence
        </span>
        <div style={{ display: "flex", alignItems: "center", gap: "0.45rem" }}>
          <span style={{
            width: 5, height: 5, borderRadius: "50%",
            background: tier.color,
            boxShadow: `0 0 6px ${tier.color}`,
            animation: "pulse-dot 1.8s ease-in-out infinite",
          }} />
          <span style={{ fontSize: "0.7rem", color: tier.color, fontWeight: 600 }}>{tier.label}</span>
          <span style={{ fontSize: "0.88rem", fontWeight: 700, letterSpacing: "-0.03em", color: "var(--text)" }}>
            {pct}<span style={{ fontSize: "0.6rem", opacity: 0.5 }}>%</span>
          </span>
        </div>
      </div>
      <div style={{
        height: 3, borderRadius: 9999,
        background: "rgba(255,255,255,0.06)",
        overflow: "hidden",
      }}>
        <div style={{
          height: "100%", borderRadius: 9999,
          width: `${pct}%`,
          background: `linear-gradient(90deg, ${tier.color} 0%, rgba(255,255,255,0.7) 100%)`,
          boxShadow: `0 0 8px ${tier.color}`,
          transition: "width 0.7s cubic-bezier(0.34,1.56,0.64,1)",
        }} />
      </div>
    </div>
  );
}

// ─── Public export ────────────────────────────────────────────────────────────
export function BiofieldLiveMetrics({ scores }: { scores: CompositeScores }) {
  return (
    <section style={{
      padding: "1.4rem 1.6rem",
      borderRadius: "var(--r-xl)",
      background: "var(--panel)",
      border: "1px solid var(--line-faint)",
      boxShadow: "inset 0 1px 0 rgba(255,255,255,0.04)",
      display: "flex",
      flexDirection: "column",
      gap: "1.2rem",
    }}>
      {/* Header */}
      <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between" }}>
        <div style={{ display: "flex", alignItems: "center", gap: "0.6rem" }}>
          <span style={{
            width: 6, height: 6, borderRadius: "50%",
            background: "var(--accent)",
            boxShadow: "0 0 8px rgba(124,124,255,0.6)",
            animation: "pulse-dot 2s ease-in-out infinite",
          }} />
          <p style={{ margin: 0, fontSize: "0.72rem", fontWeight: 600, letterSpacing: "0.12em", textTransform: "uppercase", color: "var(--muted)" }}>
            Live field metrics
          </p>
        </div>
        <span style={{
          fontSize: "0.62rem", fontWeight: 600, letterSpacing: "0.1em",
          textTransform: "uppercase",
          padding: "0.18rem 0.5rem",
          borderRadius: "var(--r-pill)",
          background: "rgba(124,124,255,0.1)",
          border: "1px solid rgba(124,124,255,0.25)",
          color: "var(--accent)",
        }}>
          Real-time
        </span>
      </div>

      {/* Coherence bar */}
      <CoherenceBar value={scores.overallCoherence} />

      {/* Arc gauges grid */}
      <div style={{
        display: "grid",
        gridTemplateColumns: "repeat(4, 1fr)",
        gap: "0.75rem 0.5rem",
        justifyItems: "center",
      }}>
        <ArcGauge value={scores.bodySymmetry}      label="Symmetry"   accent="accent" />
        <ArcGauge value={scores.lightQuantaDensity} label="Luminance"  accent="accent" />
        <ArcGauge value={scores.patternRegularity} label="Regularity" accent="signal" />
        <ArcGauge value={scores.normalizedArea}    label="Presence"   accent="signal" />
      </div>
    </section>
  );
}
