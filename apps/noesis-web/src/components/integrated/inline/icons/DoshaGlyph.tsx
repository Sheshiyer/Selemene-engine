// ─── DoshaGlyph — warning-style hex for classical doshas ───────────────
// Small hexagon with a glyph inside, drawn in Sacred Gold.

import type { CSSProperties } from "react";

export type DoshaName =
  | "sade-sati"
  | "kala-sarpa"
  | "mangal-dosha"
  | "pitru-dosha"
  | "kemadruma";

interface DoshaGlyphProps {
  dosha: DoshaName;
  size?: number;
  title?: string;
}

const baseStyle: CSSProperties = {
  display: "inline-block",
  verticalAlign: "middle",
  margin: "0 0.18em 0 0",
  flexShrink: 0,
};

function hexPath(cx: number, cy: number, r: number): string {
  const pts: string[] = [];
  for (let i = 0; i < 6; i++) {
    const a = (Math.PI / 3) * i - Math.PI / 2;
    pts.push(`${cx + r * Math.cos(a)},${cy + r * Math.sin(a)}`);
  }
  return `M ${pts.join(" L ")} Z`;
}

function DoshaInner({ dosha }: { dosha: DoshaName }) {
  const stroke = "var(--c-gold, #C5A017)";
  const sw = 1.1;
  const common = {
    stroke,
    strokeWidth: sw,
    strokeLinecap: "round" as const,
    strokeLinejoin: "round" as const,
    fill: "none",
  };
  switch (dosha) {
    case "sade-sati":
      // Saturn-shadow — sickle
      return (
        <g {...common}>
          <path d="M10 8 L10 16" />
          <path d="M9 9 L13 9" />
          <path d="M10 16 C 10 17.5, 13 17.5, 13 16 C 13 14, 16 14, 16 11.5" />
        </g>
      );
    case "kala-sarpa":
      // Serpent wave between Rahu and Ketu
      return (
        <g {...common}>
          <path d="M7 13 C 9 9, 11 17, 13 13 C 15 9, 17 17, 17 13" />
          <circle cx="7" cy="13" r="0.9" fill={stroke} stroke="none" />
          <circle cx="17" cy="13" r="0.9" fill={stroke} stroke="none" />
        </g>
      );
    case "mangal-dosha":
      // Mars wound — arrow
      return (
        <g {...common}>
          <circle cx="11" cy="13" r="2.6" />
          <path d="M12.8 11.2 L16 8" />
          <path d="M16 8 L13.2 8" />
          <path d="M16 8 L16 10.8" />
        </g>
      );
    case "pitru-dosha":
      // Ancestral — bowl with rising lines
      return (
        <g {...common}>
          <path d="M7 14 C 7 17, 17 17, 17 14" />
          <path d="M9 11 L9 9" />
          <path d="M12 11 L12 8" />
          <path d="M15 11 L15 9" />
        </g>
      );
    case "kemadruma":
      // Lonely Moon — single crescent
      return (
        <g {...common}>
          <path d="M15 9 A 4.5 4.5 0 1 0 15 17 A 3.4 3.4 0 1 1 15 9 Z" />
        </g>
      );
  }
}

export function DoshaGlyph({ dosha, size = 20, title }: DoshaGlyphProps) {
  const stroke = "var(--c-gold, #C5A017)";
  const label = title ?? dosha;
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
      <path
        d={hexPath(12, 12, 9)}
        stroke={stroke}
        strokeWidth={1.0}
        fill="none"
        opacity={0.7}
      />
      <DoshaInner dosha={dosha} />
    </svg>
  );
}
