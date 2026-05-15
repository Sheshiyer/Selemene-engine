// ─── GeneKeyGlyph — I-Ching hexagram + key number ──────────────────────
// 18×24 micro hexagram showing the 6 lines of a gene-key/hexagram, with
// the key number adjacent. Yang = solid bar, Yin = bar with center gap.

import type { CSSProperties } from "react";

interface GeneKeyGlyphProps {
  /** Gene Key / hexagram number 1..64. Optional — when omitted, only
   *  the hexagram pattern derived from `lines` is shown. */
  key64?: number;
  /** Six binary lines from bottom to top (true = yang, false = yin).
   *  When omitted and key64 is given, derived from I-Ching King Wen order
   *  is NOT computed (kept simple) — defaults to alternating pattern. */
  lines?: [boolean, boolean, boolean, boolean, boolean, boolean];
  size?: number;
  title?: string;
}

const wrapStyle: CSSProperties = {
  display: "inline-flex",
  alignItems: "center",
  verticalAlign: "middle",
  gap: "0.18em",
  margin: "0 0.18em 0 0",
  flexShrink: 0,
};

// King-Wen ordering of the 64 hexagrams (bottom→top binary).
// Encoded as 6-bit numbers where bit 0 = bottom line (1=yang).
// Source: standard King Wen sequence.
const KING_WEN_BITS: number[] = [
  0b111111, 0b000000, 0b010001, 0b100010, 0b010111, 0b111010, 0b000010, 0b010000,
  0b011101, 0b101110, 0b000111, 0b111000, 0b101111, 0b111101, 0b000100, 0b001000,
  0b011001, 0b100110, 0b000011, 0b110000, 0b101001, 0b100101, 0b000001, 0b100000,
  0b111001, 0b100111, 0b000101, 0b101000, 0b010010, 0b101101, 0b011100, 0b001110,
  0b111100, 0b001111, 0b000101, 0b101000, 0b011010, 0b010110, 0b001010, 0b010100,
  0b110001, 0b100011, 0b011111, 0b111110, 0b000110, 0b011000, 0b010110, 0b011010,
  0b101011, 0b110101, 0b001001, 0b100100, 0b001011, 0b110100, 0b001101, 0b101100,
  0b011011, 0b110110, 0b010101, 0b101010, 0b110011, 0b001100, 0b010101, 0b101010,
];

function bitsToLines(
  n: number,
): [boolean, boolean, boolean, boolean, boolean, boolean] {
  return [
    !!(n & 0b000001),
    !!(n & 0b000010),
    !!(n & 0b000100),
    !!(n & 0b001000),
    !!(n & 0b010000),
    !!(n & 0b100000),
  ];
}

export function GeneKeyGlyph({
  key64,
  lines,
  size = 22,
  title,
}: GeneKeyGlyphProps) {
  let resolved: [boolean, boolean, boolean, boolean, boolean, boolean];
  if (lines) {
    resolved = lines;
  } else if (key64 && key64 >= 1 && key64 <= 64) {
    resolved = bitsToLines(KING_WEN_BITS[key64 - 1]);
  } else {
    resolved = [true, false, true, false, true, false];
  }

  const stroke = "var(--c-gold, #C5A017)";
  const label = title ?? (key64 ? `Gene Key ${key64}` : "Hexagram");
  const w = 18;
  const h = 24;
  const lineH = 2;
  const gap = 1.5;
  const totalLines = 6;
  const stackH = totalLines * lineH + (totalLines - 1) * gap;
  const startY = (h - stackH) / 2;
  const lineW = 14;
  const centerGap = 4;
  const lineX = (w - lineW) / 2;

  return (
    <span
      role="img"
      aria-label={label}
      style={wrapStyle}
      title={title}
    >
      <svg
        width={(size * w) / h}
        height={size}
        viewBox={`0 0 ${w} ${h}`}
        style={{ display: "inline-block", verticalAlign: "middle", flexShrink: 0 }}
        aria-hidden
      >
        <g fill={stroke}>
          {/* lines top→bottom in DOM, but resolved is bottom→top */}
          {[5, 4, 3, 2, 1, 0].map((idx, displayIdx) => {
            const yang = resolved[idx];
            const y = startY + displayIdx * (lineH + gap);
            if (yang) {
              return (
                <rect
                  key={idx}
                  x={lineX}
                  y={y}
                  width={lineW}
                  height={lineH}
                  rx={0.4}
                />
              );
            }
            const segW = (lineW - centerGap) / 2;
            return (
              <g key={idx}>
                <rect x={lineX} y={y} width={segW} height={lineH} rx={0.4} />
                <rect
                  x={lineX + segW + centerGap}
                  y={y}
                  width={segW}
                  height={lineH}
                  rx={0.4}
                />
              </g>
            );
          })}
        </g>
      </svg>
      {key64 ? (
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
          {key64}
        </span>
      ) : null}
    </span>
  );
}
