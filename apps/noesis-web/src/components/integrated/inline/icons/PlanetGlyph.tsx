// ─── PlanetGlyph — 9 grahas as small inline SVG sigils ─────────────────
// Stylized planet glyphs in Sacred Gold strokes. Default 18px. Used
// inline next to planet names in the verse prose (e.g. "Jupiter ♃").
//
// Per integrated-reading-design-v2.md § 5.5 — micro-yantras inline.

import type { CSSProperties } from "react";

export type PlanetName =
  | "sun"
  | "moon"
  | "mars"
  | "mercury"
  | "jupiter"
  | "venus"
  | "saturn"
  | "rahu"
  | "ketu";

interface PlanetGlyphProps {
  planet: PlanetName;
  size?: number;
  title?: string;
}

const baseStyle: CSSProperties = {
  display: "inline-block",
  verticalAlign: "middle",
  margin: "0 0.18em 0 0",
  flexShrink: 0,
};

// Stroke geometry per planet — drawn on a 0..24 viewBox.
function PlanetPath({ planet }: { planet: PlanetName }) {
  const stroke = "var(--c-gold, #C5A017)";
  const sw = 1.25;
  const common = {
    stroke,
    strokeWidth: sw,
    strokeLinecap: "round" as const,
    strokeLinejoin: "round" as const,
    fill: "none",
  };
  switch (planet) {
    case "sun":
      // ☉ — circle + central dot
      return (
        <g {...common}>
          <circle cx="12" cy="12" r="6.5" />
          <circle cx="12" cy="12" r="1.1" fill={stroke} />
        </g>
      );
    case "moon":
      // ☽ — crescent
      return (
        <g {...common}>
          <path d="M16 5.5 A 7 7 0 1 0 16 18.5 A 5.2 5.2 0 1 1 16 5.5 Z" />
        </g>
      );
    case "mars":
      // ♂ — circle with NE arrow
      return (
        <g {...common}>
          <circle cx="10" cy="14" r="4.6" />
          <path d="M13.2 10.8 L19 5" />
          <path d="M19 5 L14.4 5" />
          <path d="M19 5 L19 9.6" />
        </g>
      );
    case "mercury":
      // ☿ — horns + circle + cross
      return (
        <g {...common}>
          <path d="M7.6 3.5 A 4.4 4.4 0 0 0 16.4 3.5" />
          <circle cx="12" cy="11" r="3.4" />
          <path d="M12 14.4 L12 20.5" />
          <path d="M9 18 L15 18" />
        </g>
      );
    case "jupiter":
      // ♃ — stylized "4"
      return (
        <g {...common}>
          <path d="M5.5 8 A 3 3 0 0 1 11.5 8 L 11.5 19" />
          <path d="M5.5 14.5 L 18.5 14.5" />
        </g>
      );
    case "venus":
      // ♀ — circle with cross below
      return (
        <g {...common}>
          <circle cx="12" cy="9" r="4.4" />
          <path d="M12 13.4 L12 21" />
          <path d="M9 18 L15 18" />
        </g>
      );
    case "saturn":
      // ♄ — sickle / lowercase h
      return (
        <g {...common}>
          <path d="M7 5 L7 18.5" />
          <path d="M5 6.5 L9 6.5" />
          <path d="M7 18.5 C 7 21.5, 12 21.5, 12 18.5 C 12 15, 18 15, 18 11" />
          <path d="M16.5 9.5 L18.5 11 L19.2 8.5" />
        </g>
      );
    case "rahu":
      // ☊ — ascending node (cup with feet)
      return (
        <g {...common}>
          <path d="M6 14 C 6 7.5, 18 7.5, 18 14" />
          <circle cx="6" cy="16" r="1.4" fill={stroke} stroke="none" />
          <circle cx="18" cy="16" r="1.4" fill={stroke} stroke="none" />
          <path d="M6 17.5 L6 20" />
          <path d="M18 17.5 L18 20" />
        </g>
      );
    case "ketu":
      // ☋ — descending node (inverted cup)
      return (
        <g {...common}>
          <path d="M6 10 C 6 16.5, 18 16.5, 18 10" />
          <circle cx="6" cy="8" r="1.4" fill={stroke} stroke="none" />
          <circle cx="18" cy="8" r="1.4" fill={stroke} stroke="none" />
          <path d="M6 6.5 L6 4" />
          <path d="M18 6.5 L18 4" />
        </g>
      );
  }
}

export function PlanetGlyph({ planet, size = 18, title }: PlanetGlyphProps) {
  const label = title ?? `${planet[0].toUpperCase()}${planet.slice(1)}`;
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
      <PlanetPath planet={planet} />
    </svg>
  );
}
