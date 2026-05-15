"use client";

// ─── BentoChip — small rounded badge for meta information ────────────────
// Matches the Brand Design System's small-tag / status-pill aesthetic.
// Used in BentoCard featured zones, headings, and inline meta rows.
//
// Variants:
//   default  — neutral parchment-on-void
//   gold     — Sacred Gold accent (active/highlighted)
//   emerald  — Coherence Emerald (success/coherent)
//   violet   — Witness Violet (past/memory)
//   indigo   — Flow Indigo (flowing/active)
//   ghost    — outline only, transparent fill

import { type ReactNode } from "react";

export type BentoChipVariant =
  | "default"
  | "gold"
  | "emerald"
  | "violet"
  | "indigo"
  | "ghost";

interface BentoChipProps {
  children: ReactNode;
  variant?: BentoChipVariant;
  size?: "sm" | "md";
  /** Optional label that sits ABOVE the chip (small mono caption) */
  label?: string;
  /** Optional tiny SVG icon at left */
  icon?: ReactNode;
}

const VARIANT_STYLES: Record<BentoChipVariant, React.CSSProperties> = {
  default: {
    background: "rgba(240,237,227,0.06)",
    border: "1px solid rgba(240,237,227,0.16)",
    color: "var(--c-parchment, #F0EDE3)",
  },
  gold: {
    background: "rgba(197,160,23,0.14)",
    border: "1px solid rgba(197,160,23,0.45)",
    color: "var(--c-gold, #C5A017)",
  },
  emerald: {
    background: "rgba(16,181,167,0.14)",
    border: "1px solid rgba(16,181,167,0.45)",
    color: "var(--c-emerald, #10B5A7)",
  },
  violet: {
    background: "rgba(45,0,80,0.32)",
    border: "1px solid rgba(45,0,80,0.55)",
    color: "rgba(240,237,227,0.85)",
  },
  indigo: {
    background: "rgba(11,80,251,0.14)",
    border: "1px solid rgba(11,80,251,0.45)",
    color: "rgba(240,237,227,0.92)",
  },
  ghost: {
    background: "transparent",
    border: "1px solid rgba(240,237,227,0.18)",
    color: "var(--c-parchment, #F0EDE3)",
  },
};

const SIZE_STYLES = {
  sm: { padding: "0.32rem 0.7rem", fontSize: "0.7rem", letterSpacing: "0.18em" },
  md: { padding: "0.45rem 0.95rem", fontSize: "0.78rem", letterSpacing: "0.14em" },
} as const;

export function BentoChip({
  children,
  variant = "default",
  size = "md",
  label,
  icon,
}: BentoChipProps) {
  const chip = (
    <span
      style={{
        display: "inline-flex",
        alignItems: "center",
        gap: "0.4rem",
        borderRadius: "999px",
        fontFamily: "var(--font-mono)",
        fontWeight: 500,
        textTransform: "uppercase" as const,
        whiteSpace: "nowrap" as const,
        ...SIZE_STYLES[size],
        ...VARIANT_STYLES[variant],
      }}
    >
      {icon && <span style={{ display: "inline-flex" }}>{icon}</span>}
      {children}
    </span>
  );
  if (!label) return chip;
  return (
    <span style={{ display: "inline-flex", flexDirection: "column", gap: "0.3rem" }}>
      <span
        style={{
          fontFamily: "var(--font-mono)",
          fontSize: "0.62rem",
          letterSpacing: "0.22em",
          textTransform: "uppercase",
          color: "var(--muted, rgba(240,237,227,0.55))",
        }}
      >
        {label}
      </span>
      {chip}
    </span>
  );
}
