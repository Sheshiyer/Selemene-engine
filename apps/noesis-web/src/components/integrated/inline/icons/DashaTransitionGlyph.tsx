// ─── DashaTransitionGlyph — "from → to" mini sigil ─────────────────────
// 32×16 pill showing source planet → target planet with an arrow.
// E.g. Rahu→Jupiter, Mars→Rahu.

import type { CSSProperties } from "react";
import { PlanetGlyph, type PlanetName } from "./PlanetGlyph";

interface DashaTransitionGlyphProps {
  from: PlanetName;
  to: PlanetName;
  /** Height in px. Width auto-scales. */
  size?: number;
  title?: string;
}

const wrapStyle: CSSProperties = {
  display: "inline-flex",
  alignItems: "center",
  verticalAlign: "middle",
  gap: "0.05em",
  margin: "0 0.18em 0 0",
  flexShrink: 0,
};

export function DashaTransitionGlyph({
  from,
  to,
  size = 18,
  title,
}: DashaTransitionGlyphProps) {
  const stroke = "var(--c-gold, #C5A017)";
  const arrowH = size;
  const arrowW = Math.round(size * 0.75);
  const label = title ?? `${from} → ${to}`;
  return (
    <span
      role="img"
      aria-label={label}
      style={wrapStyle}
      title={title}
    >
      <PlanetGlyph planet={from} size={size} />
      <svg
        width={arrowW}
        height={arrowH}
        viewBox="0 0 18 24"
        style={{ display: "inline-block", verticalAlign: "middle", flexShrink: 0 }}
        aria-hidden
      >
        <g
          stroke={stroke}
          strokeWidth={1.25}
          strokeLinecap="round"
          strokeLinejoin="round"
          fill="none"
        >
          <path d="M2 12 L15 12" />
          <path d="M11 8 L15 12 L11 16" />
        </g>
      </svg>
      <PlanetGlyph planet={to} size={size} />
    </span>
  );
}
