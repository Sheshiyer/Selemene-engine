"use client";

// ─── BentoGrid — responsive non-linear grid for bento cards ───────────────
// CSS Grid with auto-flow:dense + named columns. Cards declare `span`
// (1/2/3) and `rowSpan` (1/2) themselves; the grid arranges them
// non-linearly to fill rows efficiently.
//
// At desktop: 3-column grid (1fr 1fr 1fr) — cards can occupy 1, 2, or 3 cols.
// At tablet:  2-column grid (auto-collapse to 1 or 2)
// At mobile:  1-column stack
//
// Gap scales fluidly with viewport.

import { type ReactNode } from "react";

interface BentoGridProps {
  children: ReactNode;
  /** Outer max-width — defaults to a wide reading band, clamped responsively */
  maxWidth?: string;
  /** Optional className passthrough */
  className?: string;
  /** Vertical gap override */
  gap?: string;
}

export function BentoGrid({
  children,
  maxWidth = "min(96rem, 96vw)",
  className,
  gap,
}: BentoGridProps) {
  return (
    <div
      className={`bento-grid ${className ?? ""}`}
      style={{
        width: "100%",
        maxWidth,
        margin: "0 auto",
        padding: "clamp(1rem, 2vw, 2rem) clamp(0.75rem, 1.5vw, 1.5rem)",
        display: "grid",
        gridTemplateColumns: "repeat(3, minmax(0, 1fr))",
        gridAutoFlow: "dense",
        gridAutoRows: "minmax(clamp(14rem, 22vw, 22rem), auto)",
        gap: gap ?? "clamp(0.75rem, 1.2vw, 1.4rem)",
      }}
    >
      {children}
      <style>{`
        @media (max-width: 1080px) {
          .bento-grid { grid-template-columns: repeat(2, minmax(0, 1fr)) !important; }
          .bento-grid > * { grid-column: span 1 !important; }
        }
        @media (max-width: 680px) {
          .bento-grid { grid-template-columns: minmax(0, 1fr) !important; }
        }
      `}</style>
    </div>
  );
}
