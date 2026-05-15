"use client";

// ─── DecisionPlate — ⌬ ACT card matching decisionscreen.png ─────────────
// Per design-v2 § 5.9. Triggered by `> ⌬ ACT: ...` blockquotes parsed by
// parseBlocks.ts. Vertical stack: coherence mandala → CTA pill →
// breath prompt → optimal-window waveform → date pill → witness quote.

import { motion, useInView, useReducedMotion } from "motion/react";
import { useRef } from "react";
import type { DecisionMarker } from "@/lib/integrated/parseBlocks";

interface DecisionPlateProps {
  marker: DecisionMarker;
}

// ─── WitnessPulse import (W2) with graceful fallback ───────────────────
// W2's WitnessPulse is already merged on this branch, but we still guard
// against the symbol being missing so this component stays useful in
// isolation (e.g. tests, Storybook).
import { WitnessPulse } from "@/components/integrated/yantras/WitnessPulse";

/** Inline concentric-rings fallback used if WitnessPulse isn't available
 *  for any reason — keeps the plate visually coherent in isolation. */
function ConcentricFallback({ size = 220 }: { size?: number }) {
  return (
    <svg width={size} height={size} viewBox="-100 -100 200 200" aria-hidden="true">
      {[42, 60, 78, 92].map((r, i) => (
        <circle
          key={i}
          cx="0"
          cy="0"
          r={r}
          fill="none"
          stroke="var(--c-gold)"
          strokeOpacity={0.18 + i * 0.12}
          strokeWidth={0.9 - i * 0.12}
        />
      ))}
      <circle cx="0" cy="0" r="6" fill="var(--c-gold)" />
    </svg>
  );
}

/** Coherence mandala — slowly rotating WitnessPulse with overlay text. */
function CoherenceMandala() {
  const reduced = useReducedMotion();
  let inner: React.ReactNode;
  try {
    inner = <WitnessPulse direction="STABILIZE" title="COHERENCE" />;
  } catch {
    inner = <ConcentricFallback size={240} />;
  }
  return (
    <motion.div
      style={{
        position: "relative",
        width: 260,
        height: 260,
        margin: "0 auto",
      }}
      animate={reduced ? undefined : { rotate: 360 }}
      transition={{ duration: 62.8, repeat: Infinity, ease: "linear" }}
    >
      {inner}
      <div
        aria-hidden="true"
        style={{
          position: "absolute",
          inset: 0,
          display: "flex",
          flexDirection: "column",
          alignItems: "center",
          justifyContent: "center",
          gap: "0.25rem",
          pointerEvents: "none",
        }}
      >
        <span
          style={{
            fontFamily: "var(--font-mono)",
            fontSize: "0.62rem",
            letterSpacing: "0.38em",
            color: "var(--c-gold)",
            opacity: 0.55,
          }}
        >
          COHERENCE
        </span>
        <span
          style={{
            fontFamily: "var(--font-display)",
            fontWeight: 700,
            fontSize: "1.1rem",
            letterSpacing: "0.32em",
            color: "var(--c-gold)",
          }}
        >
          OPTIMAL
        </span>
      </div>
    </motion.div>
  );
}

/** OPTIMAL WINDOW waveform — sine path with Sacred Gold "now" dot.  */
function WindowWaveform({ window }: { window?: string }) {
  // Sine path across the 360-unit canvas, centred vertically.
  const points: string[] = [];
  for (let x = 0; x <= 360; x += 4) {
    const y = 18 + Math.sin((x / 360) * Math.PI * 2.5) * 11;
    points.push(`${x === 0 ? "M" : "L"} ${x} ${y.toFixed(2)}`);
  }
  return (
    <div
      role="presentation"
      style={{
        position: "relative",
        width: "100%",
        maxWidth: 420,
        margin: "0.5rem auto 0",
      }}
    >
      <div
        style={{
          display: "flex",
          justifyContent: "space-between",
          fontFamily: "var(--font-mono)",
          fontSize: "0.6rem",
          letterSpacing: "0.28em",
          color: "rgba(240,237,227,0.55)",
          marginBottom: "0.35rem",
          textTransform: "uppercase",
        }}
      >
        <span>OPTIMAL WINDOW</span>
        <span style={{ color: "var(--c-gold)" }}>{window ?? "—"}</span>
      </div>
      <svg
        viewBox="0 0 360 36"
        width="100%"
        height="36"
        aria-hidden="true"
        style={{ display: "block", overflow: "visible" }}
      >
        <path d={points.join(" ")} fill="none" stroke="var(--c-emerald)" strokeOpacity="0.55" strokeWidth="1" />
        {/* Endpoint markers */}
        <circle cx="0" cy="18" r="2.5" fill="var(--c-emerald)" opacity="0.7" />
        <circle cx="360" cy="18" r="2.5" fill="var(--c-emerald)" opacity="0.7" />
        {/* "Now" dot — centre */}
        <circle cx="180" cy="18" r="4.5" fill="var(--c-gold)">
          <animate
            attributeName="r"
            values="4.5;6;4.5"
            dur="2.2s"
            repeatCount="indefinite"
          />
          <animate
            attributeName="opacity"
            values="0.9;1;0.9"
            dur="2.2s"
            repeatCount="indefinite"
          />
        </circle>
      </svg>
    </div>
  );
}

/** Format an ISO date as "SEP · 14"-style pancha-paced label. */
function formatDatePill(dateStr?: string): string | null {
  if (!dateStr) return null;
  const d = new Date(dateStr);
  if (Number.isNaN(d.valueOf())) return dateStr.toUpperCase();
  const month = d
    .toLocaleString("en-US", { month: "short" })
    .toUpperCase();
  return `${month} · ${String(d.getUTCDate()).padStart(2, "0")}`;
}

export function DecisionPlate({ marker }: DecisionPlateProps) {
  const ref = useRef<HTMLDivElement>(null);
  const inView = useInView(ref, { margin: "-25% 0% -25% 0%", once: true });
  const reduced = useReducedMotion();
  const datePill = formatDatePill(marker.date);
  return (
    <motion.aside
      ref={ref}
      role="complementary"
      aria-label="Decision plate — coherence action card"
      style={{
        position: "relative",
        width: "100%",
        maxWidth: 540,
        margin: "clamp(2.5rem, 5vw, 4.5rem) auto",
        padding: "clamp(1.75rem, 3vw, 2.5rem) clamp(1.5rem, 3vw, 2.5rem)",
        background:
          "radial-gradient(ellipse at top, rgba(197,160,23,0.06) 0%, rgba(7,11,29,0.85) 70%)",
        border: "1px solid rgba(197,160,23,0.35)",
        borderRadius: 22,
        boxShadow:
          "0 0 0 1px rgba(197,160,23,0.08), 0 20px 60px -20px rgba(45,0,80,0.4)",
        textAlign: "center" as const,
        display: "flex",
        flexDirection: "column",
        alignItems: "center",
        gap: "1.25rem",
      }}
      initial={reduced ? false : { opacity: 0, y: 24, scale: 0.96 }}
      animate={
        reduced
          ? { opacity: 1, y: 0, scale: 1 }
          : inView
          ? { opacity: 1, y: 0, scale: 1 }
          : { opacity: 0, y: 24, scale: 0.96 }
      }
      transition={{ duration: 0.85, ease: [0.16, 1, 0.3, 1] }}
    >
      <CoherenceMandala />

      <motion.button
        type="button"
        style={{
          display: "inline-block",
          padding: "1rem 2.25rem",
          background: "var(--c-gold)",
          color: "var(--c-void)",
          fontFamily: "var(--font-display)",
          fontWeight: 700,
          fontSize: "1.08rem",
          letterSpacing: "0.05em",
          border: "none",
          borderRadius: 999,
          cursor: "pointer",
          maxWidth: "30rem",
          lineHeight: 1.3,
          boxShadow: "0 0 0 0 rgba(197,160,23,0.5)",
        }}
        animate={
          reduced
            ? undefined
            : {
                boxShadow: [
                  "0 0 0 0 rgba(197,160,23,0.5)",
                  "0 0 0 14px rgba(197,160,23,0)",
                  "0 0 0 0 rgba(197,160,23,0.5)",
                ],
              }
        }
        transition={{ duration: 2.6, repeat: Infinity, ease: "easeInOut" }}
      >
        {marker.action}
      </motion.button>

      <div
        style={{
          fontFamily: "var(--font-body)",
          fontStyle: "italic",
          fontSize: "0.92rem",
          color: "rgba(240,237,227,0.78)",
          letterSpacing: "0.04em",
        }}
      >
        Breathe in… 4 : 7 : 8
      </div>

      <WindowWaveform window={marker.window} />

      {datePill && (
        <div
          style={{
            display: "inline-block",
            padding: "0.35rem 1.1rem",
            borderRadius: 999,
            border: "1px solid rgba(197,160,23,0.45)",
            fontFamily: "var(--font-mono)",
            fontSize: "0.72rem",
            letterSpacing: "0.32em",
            color: "var(--c-gold)",
          }}
        >
          {datePill}
        </div>
      )}

      {marker.quote && (
        <div
          style={{
            maxWidth: "36ch",
            fontFamily: "var(--font-display)",
            fontStyle: "italic",
            fontWeight: 500,
            fontSize: "0.95rem",
            color: "rgba(240,237,227,0.92)",
            lineHeight: 1.55,
            marginTop: "0.25rem",
          }}
        >
          “{marker.quote}”
        </div>
      )}
    </motion.aside>
  );
}
