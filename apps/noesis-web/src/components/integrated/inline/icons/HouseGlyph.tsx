// ─── HouseGlyph — 12 bhavas as a tiny ringed numeral ───────────────────
// Small circle with a Roman numeral inside, Sacred Gold strokes.
// Used inline next to "1st Bhāva", "9th House", etc.

import type { CSSProperties } from "react";

interface HouseGlyphProps {
  /** 1..12 */
  house: number;
  size?: number;
  title?: string;
}

const baseStyle: CSSProperties = {
  display: "inline-block",
  verticalAlign: "middle",
  margin: "0 0.18em 0 0",
  flexShrink: 0,
};

const ROMAN = [
  "",
  "I",
  "II",
  "III",
  "IV",
  "V",
  "VI",
  "VII",
  "VIII",
  "IX",
  "X",
  "XI",
  "XII",
];

export function HouseGlyph({ house, size = 18, title }: HouseGlyphProps) {
  const n = Math.min(12, Math.max(1, Math.round(house)));
  const roman = ROMAN[n] ?? String(n);
  const stroke = "var(--c-gold, #C5A017)";
  const label = title ?? `House ${n}`;
  // Roman font-size scales down for 3+ chars to fit.
  const fs = roman.length >= 3 ? 7 : roman.length === 2 ? 9 : 11;
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
      <circle
        cx="12"
        cy="12"
        r="9.5"
        fill="none"
        stroke={stroke}
        strokeWidth={1.25}
      />
      <text
        x="12"
        y="12.2"
        textAnchor="middle"
        dominantBaseline="central"
        fill={stroke}
        fontFamily="var(--font-mono, 'JetBrains Mono', monospace)"
        fontSize={fs}
        fontWeight={500}
        letterSpacing="0.02em"
      >
        {roman}
      </text>
    </svg>
  );
}
