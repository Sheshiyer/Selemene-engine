// ─── TarotMinorGlyph — tiny suit symbol + optional rank ────────────────
// Small Cups / Wands / Swords / Pentacles glyph for inline use.

import type { CSSProperties } from "react";

export type TarotSuit = "cups" | "wands" | "swords" | "pentacles";

interface TarotMinorGlyphProps {
  suit: TarotSuit;
  /** 1..10 or "Page" | "Knight" | "Queen" | "King" — rendered as small label */
  rank?: number | string;
  size?: number;
  title?: string;
}

const wrapStyle: CSSProperties = {
  display: "inline-flex",
  alignItems: "center",
  verticalAlign: "middle",
  gap: "0.1em",
  margin: "0 0.18em 0 0",
  flexShrink: 0,
};

function SuitPath({ suit }: { suit: TarotSuit }) {
  const stroke = "var(--c-gold, #C5A017)";
  const sw = 1.1;
  const common = {
    stroke,
    strokeWidth: sw,
    strokeLinecap: "round" as const,
    strokeLinejoin: "round" as const,
    fill: "none",
  };
  switch (suit) {
    case "cups":
      return (
        <g {...common}>
          <path d="M6 8 L18 8 C 18 14, 14 16, 12 16 C 10 16, 6 14, 6 8 Z" />
          <path d="M12 16 L12 19" />
          <path d="M9 19 L15 19" />
        </g>
      );
    case "wands":
      return (
        <g {...common}>
          <path d="M5 19 L19 5" />
          <path d="M5 7 L7 5 L8 8 Z" />
          <path d="M16 16 L17 19 L19 17 Z" />
        </g>
      );
    case "swords":
      return (
        <g {...common}>
          <path d="M12 4 L12 18" />
          <path d="M8 8 L16 8" />
          <path d="M10 20 L14 20" />
        </g>
      );
    case "pentacles":
      return (
        <g {...common}>
          <circle cx="12" cy="12" r="7" />
          <path d="M12 6 L13.8 10.4 L18.5 10.4 L14.8 13.2 L16.3 17.6 L12 14.8 L7.7 17.6 L9.2 13.2 L5.5 10.4 L10.2 10.4 Z" />
        </g>
      );
  }
}

export function TarotMinorGlyph({
  suit,
  rank,
  size = 20,
  title,
}: TarotMinorGlyphProps) {
  const stroke = "var(--c-gold, #C5A017)";
  const label = title ?? `${rank ?? ""} of ${suit}`.trim();
  return (
    <span
      role="img"
      aria-label={label}
      style={wrapStyle}
      title={title}
    >
      <svg
        width={size}
        height={size}
        viewBox="0 0 24 24"
        style={{ display: "inline-block", verticalAlign: "middle", flexShrink: 0 }}
        aria-hidden
      >
        <SuitPath suit={suit} />
      </svg>
      {rank !== undefined && rank !== null && rank !== "" ? (
        <span
          style={{
            fontFamily: "var(--font-mono, 'JetBrains Mono', monospace)",
            fontSize: "0.62em",
            letterSpacing: "0.05em",
            color: stroke,
            opacity: 0.85,
            lineHeight: 1,
          }}
          aria-hidden
        >
          {String(rank)}
        </span>
      ) : null}
    </span>
  );
}
