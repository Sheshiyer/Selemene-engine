// ─── YogaGlyph — classical Vedic yoga sigils ────────────────────────────
// Small interlocking shapes for the named yogas. Sacred Gold strokes.

import type { CSSProperties } from "react";

export type YogaName =
  | "raj"
  | "gajakesari"
  | "saraswati"
  | "dhana"
  | "vipreet-raj";

interface YogaGlyphProps {
  yoga: YogaName;
  size?: number;
  title?: string;
}

const baseStyle: CSSProperties = {
  display: "inline-block",
  verticalAlign: "middle",
  margin: "0 0.18em 0 0",
  flexShrink: 0,
};

function YogaPath({ yoga }: { yoga: YogaName }) {
  const stroke = "var(--c-gold, #C5A017)";
  const sw = 1.1;
  const common = {
    stroke,
    strokeWidth: sw,
    strokeLinecap: "round" as const,
    strokeLinejoin: "round" as const,
    fill: "none",
  };
  switch (yoga) {
    case "raj":
      // Royal — interlocked diamonds
      return (
        <g {...common}>
          <path d="M12 4 L19 12 L12 20 L5 12 Z" />
          <path d="M8 12 L16 12" />
          <path d="M12 8 L12 16" />
        </g>
      );
    case "gajakesari":
      // Jupiter-Moon — circle inside square (elephant + lion)
      return (
        <g {...common}>
          <path d="M5 5 L19 5 L19 19 L5 19 Z" />
          <circle cx="12" cy="12" r="4.5" />
        </g>
      );
    case "saraswati":
      // Trinity of benefics — three interlocked arcs (vesica trio)
      return (
        <g {...common}>
          <circle cx="9" cy="14" r="5" />
          <circle cx="15" cy="14" r="5" />
          <circle cx="12" cy="9" r="5" />
        </g>
      );
    case "dhana":
      // Wealth — overlapping coin discs
      return (
        <g {...common}>
          <circle cx="9" cy="12" r="5" />
          <circle cx="15" cy="12" r="5" />
          <circle cx="12" cy="12" r="1.4" fill={stroke} stroke="none" />
        </g>
      );
    case "vipreet-raj":
      // Reversal — inverted triangle inside upright triangle
      return (
        <g {...common}>
          <path d="M12 4 L20 18 L4 18 Z" />
          <path d="M12 20 L4 6 L20 6 Z" opacity={0.6} />
        </g>
      );
  }
}

export function YogaGlyph({ yoga, size = 20, title }: YogaGlyphProps) {
  const label = title ?? `${yoga} yoga`;
  return (
    <svg
      width={size}
      height={size}
      viewBox="0 0 24 24"
      role="img"
      aria-label={label}
      style={baseStyle}
    >
      {title ? <title>{title}</title> : null}
      <YogaPath yoga={yoga} />
    </svg>
  );
}
