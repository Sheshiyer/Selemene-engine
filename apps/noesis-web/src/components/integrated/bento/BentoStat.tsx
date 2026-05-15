"use client";

// ─── BentoStat — big-number stat block for meta data ─────────────────────
// Two configurations:
//   - Vertical: label on top (mono small), value below (display large)
//   - Horizontal: value-pair-style label | value side by side
//
// Used inside BentoCard featured zones for compact data callouts.

import { type ReactNode } from "react";

interface BentoStatProps {
  /** Small uppercase label */
  label: string;
  /** The big value — string or React node */
  value: ReactNode;
  /** Optional unit / suffix shown small after the value */
  unit?: string;
  /** Layout direction */
  direction?: "vertical" | "horizontal";
  /** Color emphasis for the value */
  accent?: "parchment" | "gold" | "emerald" | "violet" | "indigo";
  /** Optional bottom annotation (extra context, smaller) */
  annotation?: string;
}

const ACCENT_COLOR: Record<NonNullable<BentoStatProps["accent"]>, string> = {
  parchment: "var(--c-parchment, #F0EDE3)",
  gold: "var(--c-gold, #C5A017)",
  emerald: "var(--c-emerald, #10B5A7)",
  violet: "var(--c-violet, #2D0050)",
  indigo: "var(--c-indigo, #0B50FB)",
};

export function BentoStat({
  label,
  value,
  unit,
  direction = "vertical",
  accent = "parchment",
  annotation,
}: BentoStatProps) {
  const labelEl = (
    <span
      style={{
        fontFamily: "var(--font-mono)",
        fontSize: "0.7rem",
        letterSpacing: "0.22em",
        textTransform: "uppercase",
        color: "var(--muted, rgba(240,237,227,0.55))",
      }}
    >
      {label}
    </span>
  );
  const valueEl = (
    <span
      style={{
        display: "inline-flex",
        alignItems: "baseline",
        gap: "0.35rem",
        fontFamily: "var(--font-display)",
        fontWeight: 700,
        fontSize: "clamp(1.5rem, 1rem + 1.6vw, 2.6rem)",
        lineHeight: 1.05,
        letterSpacing: "-0.02em",
        color: ACCENT_COLOR[accent],
      }}
    >
      {value}
      {unit && (
        <span
          style={{
            fontFamily: "var(--font-mono)",
            fontSize: "0.72rem",
            letterSpacing: "0.18em",
            textTransform: "uppercase",
            color: "var(--muted, rgba(240,237,227,0.55))",
            fontWeight: 500,
          }}
        >
          {unit}
        </span>
      )}
    </span>
  );
  const annotationEl = annotation ? (
    <span
      style={{
        fontFamily: "var(--font-body)",
        fontSize: "0.82rem",
        color: "var(--muted, rgba(240,237,227,0.5))",
      }}
    >
      {annotation}
    </span>
  ) : null;

  if (direction === "horizontal") {
    return (
      <div
        style={{
          display: "flex",
          alignItems: "baseline",
          gap: "0.8rem",
          flexWrap: "wrap",
        }}
      >
        {labelEl}
        {valueEl}
        {annotationEl}
      </div>
    );
  }
  return (
    <div style={{ display: "flex", flexDirection: "column", gap: "0.35rem" }}>
      {labelEl}
      {valueEl}
      {annotationEl}
    </div>
  );
}
