"use client";

/**
 * BiofieldCosmogram — Geometric consciousness field display
 *
 * Renders:
 *   1. Hexagonal radar with 6 biofield axes (sacred geometry ring guides + yantra glyph)
 *   2. Chakra spine — 7 horizontally-arranged chakra nodes from engine output
 *   3. Numeric readout grid — precise score values in brand typography
 *
 * References:
 *   - BV-PIP Analysis System Specification (composite scores)
 *   - brand-config.yaml (exact hex values, Kha-Ba-La gradient system)
 */

import React, { useMemo } from "react";
import type { CompositeScores } from "./pip/types";

// ── Type for chakra_readings from Rust biofield engine ────────────────────────
interface ChakraReading {
  chakra: string;
  chakra_name: string;
  activity_level: number;
  balance: number;
  color_intensity: number;
  location?: string;
  element?: string;
}

// ── SVG geometry constants ────────────────────────────────────────────────────
const CX = 108;
const CY = 96;
const OUTER_R = 68;  // max data radius
const LABEL_R = 84;  // axis label distance from center

function toRad(deg: number) {
  return (deg * Math.PI) / 180;
}
function polar(angle: number, r: number) {
  return { x: CX + r * Math.cos(toRad(angle)), y: CY + r * Math.sin(toRad(angle)) };
}

// ── 6-axis hexagram definition ────────────────────────────────────────────────
// Angles at -90°, -30°, 30°, 90°, 150°, 210° (classic hexagon)
// Mapped to biofield metrics in Kha-Ba-La brand colors
const AXES = [
  { key: "lightQuantaDensity", label: "ENERGY",     angle: -90,  color: "#C5A017", glow: "rgba(197,160,23,0.65)"  },
  { key: "bodySymmetry",       label: "SYMMETRY",   angle: -30,  color: "#10B5A7", glow: "rgba(16,181,167,0.65)"  },
  { key: "_complexity",        label: "COMPLEXITY", angle: 30,   color: "#6B21D4", glow: "rgba(107,33,212,0.65)"  },
  { key: "normalizedArea",     label: "PRESENCE",   angle: 90,   color: "#0B50FB", glow: "rgba(11,80,251,0.65)"   },
  { key: "overallCoherence",   label: "COHERENCE",  angle: 150,  color: "#0B50FB", glow: "rgba(11,80,251,0.8)"    },
  { key: "patternRegularity",  label: "REGULARITY", angle: 210,  color: "#10B5A7", glow: "rgba(16,181,167,0.55)"  },
] as const;

type AxisKey = (typeof AXES)[number]["key"];

// ── Chakra metadata (Crown → Root order for display) ─────────────────────────
const CHAKRA_META: Record<string, { color: string; glow: string; abbrev: string }> = {
  Crown:      { color: "#9B59B6", glow: "rgba(155,89,182,0.7)", abbrev: "SA" },
  ThirdEye:   { color: "#6B21D4", glow: "rgba(107,33,212,0.7)", abbrev: "AJ" },
  Throat:     { color: "#0B50FB", glow: "rgba(11,80,251,0.7)",  abbrev: "VI" },
  Heart:      { color: "#10B5A7", glow: "rgba(16,181,167,0.7)", abbrev: "AN" },
  SolarPlexus:{ color: "#C5A017", glow: "rgba(197,160,23,0.7)", abbrev: "MA" },
  Sacral:     { color: "#E8884A", glow: "rgba(232,136,74,0.7)", abbrev: "SV" },
  Root:       { color: "#C84B31", glow: "rgba(200,75,49,0.7)",  abbrev: "MU" },
};

// Display order: Crown first, Root last
const CHAKRA_ORDER = ["Crown", "ThirdEye", "Throat", "Heart", "SolarPlexus", "Sacral", "Root"];

// ── Ring guide paths ──────────────────────────────────────────────────────────
function ringPath(frac: number): string {
  return AXES.map(({ angle }, i) => {
    const p = polar(angle, frac * OUTER_R);
    return `${i === 0 ? "M" : "L"}${p.x.toFixed(2)},${p.y.toFixed(2)}`;
  }).join(" ") + " Z";
}

// ── Axis tick lines ───────────────────────────────────────────────────────────
function axisLine(angle: number) {
  const outer = polar(angle, OUTER_R);
  return `M${CX},${CY} L${outer.x.toFixed(2)},${outer.y.toFixed(2)}`;
}

// ── Label anchor helper ───────────────────────────────────────────────────────
function labelAnchor(angle: number): React.SVGAttributes<SVGTextElement>["textAnchor"] {
  const norm = ((angle % 360) + 360) % 360;
  if (norm < 30 || norm > 330) return "middle";
  if (norm <= 150) return "start";
  if (norm >= 210) return "end";
  return "middle";
}

function labelDy(angle: number): string {
  // nudge top/bottom labels vertically
  const norm = ((angle % 360) + 360) % 360;
  if (norm === 270 || angle === -90) return "-0.3em";
  if (norm === 90) return "0.9em";
  return "0.35em";
}

// ═══════════════════════════════════════════════════════════════════════════════
// Main component
// ═══════════════════════════════════════════════════════════════════════════════
export interface BiofieldCosmogramProps {
  scores: CompositeScores;
  /** result.* from noesis biofield engine EngineOutput — provides chakra_readings & vitality_index */
  engineResult?: Record<string, unknown> | null;
}

export function BiofieldCosmogram({ scores, engineResult }: BiofieldCosmogramProps) {
  // ── Derive 6 axis values ────────────────────────────────────────────────────
  const axisValues = useMemo<Record<AxisKey, number>>(() => {
    const complexity = 1 - scores.patternRegularity; // entropy = inverted regularity
    // Try to use vitality_index from engine (used for COMPLEXITY axis if available)
    const engMetrics = engineResult?.metrics as Record<string, number> | undefined;
    const engineComplexity = engMetrics?.entropy ?? null;
    return {
      lightQuantaDensity: scores.lightQuantaDensity,
      bodySymmetry: scores.bodySymmetry,
      _complexity: engineComplexity !== null ? engineComplexity : complexity,
      normalizedArea: scores.normalizedArea,
      overallCoherence: scores.overallCoherence,
      patternRegularity: scores.patternRegularity,
    };
  }, [scores, engineResult]);

  // ── Build data polygon ──────────────────────────────────────────────────────
  const dataPolygon = useMemo(() => {
    const pts = AXES.map(({ key, angle }) => {
      const v = Math.max(0.05, Math.min(1, axisValues[key] ?? 0.5));
      const p = polar(angle, v * OUTER_R);
      return `${p.x.toFixed(2)},${p.y.toFixed(2)}`;
    });
    return pts.join(" ");
  }, [axisValues]);

  // ── Chakra readings from engine ─────────────────────────────────────────────
  const chakras = useMemo<ChakraReading[]>(() => {
    if (!engineResult) return [];
    const cr = engineResult.chakra_readings;
    if (!Array.isArray(cr)) return [];
    // Sort to match CHAKRA_ORDER (Crown → Root)
    return CHAKRA_ORDER.map((name) =>
      (cr as ChakraReading[]).find((r) => r.chakra === name || r.chakra_name === name) ?? {
        chakra: name, chakra_name: name,
        activity_level: 0.4, balance: 0, color_intensity: 0.3,
      }
    );
  }, [engineResult]);

  // Use mock chakras from scores when engine data unavailable
  const chakraMock = useMemo<ChakraReading[]>(() => {
    if (chakras.length > 0) return chakras;
    // Derive plausible chakra activity from composite scores
    return CHAKRA_ORDER.map((name, i) => ({
      chakra: name, chakra_name: name,
      activity_level: clamp(scores.overallCoherence * 0.6 + scores.lightQuantaDensity * 0.4 + (i % 3) * 0.08),
      balance: 0, color_intensity: scores.lightQuantaDensity,
    }));
  }, [chakras, scores]);

  const coherence = scores.overallCoherence;

  // ── Sacred geometry center glyph (Star of David / yantra) ──────────────────
  const yantra = useMemo(() => {
    const r = 10 + coherence * 14; // 10–24 px radius scales with coherence
    // Upward equilateral triangle
    const up = [
      polar(-90, r), polar(30, r), polar(150, r),
    ].map((p, i) => `${i === 0 ? "M" : "L"}${p.x.toFixed(2)},${p.y.toFixed(2)}`).join(" ") + " Z";
    // Downward equilateral triangle (rotated 180°)
    const dn = [
      polar(90, r), polar(210, r), polar(330, r),
    ].map((p, i) => `${i === 0 ? "M" : "L"}${p.x.toFixed(2)},${p.y.toFixed(2)}`).join(" ") + " Z";
    return { up, dn, r };
  }, [coherence]);

  const coherenceColor = coherence >= 0.75 ? "#10B5A7" : coherence >= 0.5 ? "#0B50FB" : "#C5A017";
  const coherenceGlow = coherence >= 0.75
    ? "rgba(16,181,167,0.5)" : coherence >= 0.5
    ? "rgba(11,80,251,0.5)" : "rgba(197,160,23,0.4)";

  return (
    <section style={{
      padding: "0.9rem 1rem 0.75rem",
      borderRadius: "var(--r-lg)",
      background: "linear-gradient(160deg, rgba(45,0,80,0.18) 0%, rgba(11,80,251,0.06) 60%, rgba(7,11,29,0) 100%)",
      border: "1px solid rgba(11,80,251,0.14)",
      boxShadow: "inset 0 1px 0 rgba(197,160,23,0.06)",
      display: "flex",
      flexDirection: "column",
      gap: "0.65rem",
    }}>

      {/* ── Panel header ──────────────────────────────────────────────────── */}
      <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between" }}>
        <div style={{ display: "flex", alignItems: "center", gap: 7 }}>
          <span style={{
            width: 6, height: 6, borderRadius: "50%",
            background: coherenceColor,
            boxShadow: `0 0 8px ${coherenceGlow}`,
            animation: "pulse-dot 2s ease-in-out infinite",
            flexShrink: 0,
          }} />
          <p style={{
            margin: 0,
            fontFamily: "var(--font-display)",
            fontSize: "0.62rem", fontWeight: 700,
            letterSpacing: "0.18em", textTransform: "uppercase",
            color: "var(--muted)",
          }}>
            Biofield Cosmogram
          </p>
        </div>
        <span style={{
          fontFamily: "var(--font-display)", fontSize: "0.55rem", fontWeight: 700,
          letterSpacing: "0.12em", textTransform: "uppercase",
          padding: "0.14rem 0.5rem",
          borderRadius: "var(--r-pill)",
          background: `rgba(${coherence >= 0.75 ? "16,181,167" : "11,80,251"},0.08)`,
          border: `1px solid rgba(${coherence >= 0.75 ? "16,181,167" : "11,80,251"},0.22)`,
          color: coherenceColor,
          textShadow: `0 0 10px ${coherenceGlow}`,
        }}>
          Live
        </span>
      </div>

      {/* ── Hexagonal radar ───────────────────────────────────────────────── */}
      <div style={{ display: "flex", gap: "0.75rem", alignItems: "flex-start" }}>
        {/* SVG hexagram */}
        <svg
          viewBox="0 0 216 196"
          width="100%"
          style={{ flex: "0 0 auto", width: "min(100%, 200px)", overflow: "visible" }}
          aria-label="Biofield hexagram"
        >
          <defs>
            <filter id="yantra-glow" x="-50%" y="-50%" width="200%" height="200%">
              <feGaussianBlur stdDeviation="3" result="blur" />
              <feMerge><feMergeNode in="blur" /><feMergeNode in="SourceGraphic" /></feMerge>
            </filter>
            <filter id="poly-glow" x="-30%" y="-30%" width="160%" height="160%">
              <feGaussianBlur stdDeviation="2.5" result="blur" />
              <feMerge><feMergeNode in="blur" /><feMergeNode in="SourceGraphic" /></feMerge>
            </filter>
          </defs>

          {/* Ring guides at 25 / 50 / 75 / 100% */}
          {[0.25, 0.5, 0.75, 1.0].map((frac) => (
            <path
              key={frac}
              d={ringPath(frac)}
              fill="none"
              stroke={frac === 1.0 ? "rgba(11,80,251,0.18)" : "rgba(11,80,251,0.08)"}
              strokeWidth={frac === 1.0 ? 0.8 : 0.5}
            />
          ))}

          {/* Axis tick lines */}
          {AXES.map(({ angle, color }) => (
            <path
              key={angle}
              d={axisLine(angle)}
              stroke={color}
              strokeWidth={0.6}
              opacity={0.25}
            />
          ))}

          {/* Axis dot endpoints */}
          {AXES.map(({ angle, color }) => {
            const p = polar(angle, OUTER_R);
            return (
              <circle key={`ep-${angle}`} cx={p.x} cy={p.y} r={2} fill={color} opacity={0.5} />
            );
          })}

          {/* Data polygon — filled */}
          <polygon
            points={dataPolygon}
            fill={`rgba(11,80,251,0.10)`}
            stroke="none"
          />
          {/* Data polygon — stroked with glow */}
          <polygon
            points={dataPolygon}
            fill="none"
            stroke="rgba(11,80,251,0.7)"
            strokeWidth={1.2}
            filter="url(#poly-glow)"
            style={{ transition: "all 0.6s cubic-bezier(0.34,1.56,0.64,1)" }}
          />

          {/* Axis data dots */}
          {AXES.map(({ key, angle, color, glow }) => {
            const v = Math.max(0.05, Math.min(1, axisValues[key] ?? 0.5));
            const p = polar(angle, v * OUTER_R);
            return (
              <circle
                key={`dot-${angle}`}
                cx={p.x} cy={p.y} r={3}
                fill={color}
                style={{ filter: `drop-shadow(0 0 4px ${glow})` }}
              />
            );
          })}

          {/* Sacred geometry yantra at center */}
          <g filter="url(#yantra-glow)">
            <path d={yantra.up} fill="none" stroke={coherenceColor} strokeWidth={0.9} opacity={0.55} />
            <path d={yantra.dn} fill="none" stroke={coherenceColor} strokeWidth={0.9} opacity={0.55} />
          </g>

          {/* Center coherence dot */}
          <circle
            cx={CX} cy={CY} r={3.5}
            fill={coherenceColor}
            style={{ filter: `drop-shadow(0 0 6px ${coherenceGlow})` }}
          />

          {/* Axis labels */}
          {AXES.map(({ angle, label, color }) => {
            const lp = polar(angle, LABEL_R);
            return (
              <text
                key={`lbl-${angle}`}
                x={lp.x} y={lp.y}
                textAnchor={labelAnchor(angle)}
                dy={labelDy(angle)}
                fill={color}
                fontSize={6.2}
                fontFamily="var(--font-display, monospace)"
                fontWeight={700}
                letterSpacing={1.2}
                opacity={0.8}
              >
                {label}
              </text>
            );
          })}
        </svg>

        {/* Right of hexagram: numeric score readout */}
        <div style={{
          flex: 1, minWidth: 0,
          display: "flex", flexDirection: "column", gap: "0.4rem",
          paddingTop: "0.2rem",
        }}>
          {AXES.map(({ key, label, color, glow }) => {
            const v = axisValues[key] ?? 0;
            const pct = Math.round(Math.max(0, Math.min(1, v)) * 100);
            const active = pct >= 60;
            return (
              <div key={key} style={{ display: "flex", flexDirection: "column", gap: 2 }}>
                <div style={{ display: "flex", justifyContent: "space-between", alignItems: "baseline" }}>
                  <span style={{
                    fontFamily: "var(--font-display)",
                    fontSize: "0.52rem", fontWeight: 700,
                    letterSpacing: "0.14em",
                    color: active ? "var(--text-2)" : "var(--muted)",
                    transition: "color 0.4s ease",
                    textTransform: "uppercase",
                  }}>{label.slice(0, 3)}</span>
                  <span style={{
                    fontFamily: "var(--font-mono)",
                    fontSize: "0.78rem", fontWeight: 700,
                    letterSpacing: "-0.04em",
                    color: active ? color : "var(--muted)",
                    textShadow: active ? `0 0 10px ${glow}` : "none",
                    transition: "color 0.4s ease, text-shadow 0.4s ease",
                  }}>
                    {pct}
                    <span style={{ fontSize: "0.42rem", opacity: 0.4, fontWeight: 400 }}>%</span>
                  </span>
                </div>
                <div style={{ height: 1.5, borderRadius: 9999, background: "rgba(11,80,251,0.07)", overflow: "hidden" }}>
                  <div style={{
                    height: "100%", borderRadius: 9999,
                    width: `${pct}%`,
                    background: color,
                    boxShadow: active ? `0 0 6px ${glow}` : "none",
                    opacity: active ? 1 : 0.35,
                    transition: "width 0.65s cubic-bezier(0.34,1.56,0.64,1)",
                  }} />
                </div>
              </div>
            );
          })}
        </div>
      </div>

      {/* ── Chakra Spine ──────────────────────────────────────────────────── */}
      <div style={{
        display: "flex", flexDirection: "column", gap: "0.3rem",
        paddingTop: "0.15rem",
      }}>
        <p style={{
          margin: 0,
          fontFamily: "var(--font-display)",
          fontSize: "0.52rem", fontWeight: 700,
          letterSpacing: "0.18em", textTransform: "uppercase",
          color: "var(--muted)", opacity: 0.6,
        }}>
          Chakra Field
        </p>
        <div style={{
          display: "flex",
          gap: "0.3rem",
          alignItems: "center",
          padding: "0.5rem 0.6rem",
          borderRadius: "var(--r-md)",
          background: "rgba(7,11,29,0.4)",
          border: "1px solid rgba(11,80,251,0.08)",
        }}>
          {chakraMock.map((reading) => {
            const meta = CHAKRA_META[reading.chakra] ?? CHAKRA_META["Heart"];
            const activity = Math.max(0.1, Math.min(1, reading.activity_level));
            const dotSize = 6 + activity * 10; // 6–16 px
            const active = activity >= 0.55;
            return (
              <div key={reading.chakra} style={{
                flex: 1, display: "flex", flexDirection: "column",
                alignItems: "center", gap: "0.2rem",
              }}>
                <div style={{
                  width: dotSize, height: dotSize, borderRadius: "50%",
                  background: meta.color,
                  boxShadow: active ? `0 0 ${4 + activity * 8}px ${meta.glow}` : "none",
                  transition: "all 0.5s ease",
                  flexShrink: 0,
                }} />
                <span style={{
                  fontFamily: "var(--font-mono)",
                  fontSize: "0.44rem", fontWeight: 700,
                  color: active ? meta.color : "rgba(240,237,227,0.2)",
                  letterSpacing: "0.04em",
                  textTransform: "uppercase",
                  lineHeight: 1,
                }}>
                  {meta.abbrev}
                </span>
                {/* Activity bar */}
                <div style={{ width: "100%", height: 2, borderRadius: 9999, background: "rgba(11,80,251,0.06)" }}>
                  <div style={{
                    height: "100%", borderRadius: 9999,
                    width: `${Math.round(activity * 100)}%`,
                    background: meta.color,
                    opacity: 0.7,
                    transition: "width 0.65s ease",
                  }} />
                </div>
              </div>
            );
          })}
        </div>
        {/* Chakra label strip */}
        <div style={{ display: "flex", justifyContent: "space-between", paddingLeft: "0.3rem", paddingRight: "0.3rem" }}>
          {CHAKRA_ORDER.map((name) => (
            <span key={name} style={{
              flex: 1, textAlign: "center",
              fontFamily: "var(--font-display)",
              fontSize: "0.4rem",
              color: "rgba(240,237,227,0.18)",
              letterSpacing: "0.04em",
              textTransform: "uppercase",
            }}>
              {name === "SolarPlexus" ? "SOLAR" : name === "ThirdEye" ? "3EYE" : name.slice(0, 4).toUpperCase()}
            </span>
          ))}
        </div>
      </div>

    </section>
  );
}

function clamp(v: number, lo = 0, hi = 1): number {
  return Math.max(lo, Math.min(hi, v));
}
