// ─── YantraLattice — hexagonal lattice for comparison matrices ─────────
// Renders an N-row × M-col markdown table as a grid of hex-framed cells.
// Each column gets its own gradient tint along the Goethe spectrum.
// The first column (typically the row-label) renders as a vesica-banded
// title; the remaining cells render as hex-framed mini-cards.
//
// Visual logic:
//   ╭─ Row 0:  [label band]  ⬡ ⬡ ⬡ ⬡   ← M-1 hex cells
//   ├─ Row 1:  [label band]  ⬡ ⬡ ⬡ ⬡
//   ╰─ …
//
// On viewport-center entry, the whole lattice fades from 0.5 → 1.0
// opacity (no per-cell stagger — the geometry IS the anchor).

import { HexFrame } from "../sigils";
import { renderInline } from "../parseBlocks";

interface YantraLatticeProps {
  headers: string[];
  rows: string[][];
  accentColor: string;
}

// Goethe column gradient seeds — each column picks the next tint.
const COL_TINTS = [
  "#2D0050", // Witness Violet
  "#0B50FB", // Flow Indigo
  "#10B5A7", // Coherence Emerald
  "#C5A017", // Sacred Gold
  "#F0EDE3", // Parchment
  "#070B1D", // Void (rarely used)
];

function tintFor(colIndex: number): string {
  return COL_TINTS[colIndex % COL_TINTS.length];
}

/** Add a low-alpha hex suffix to a #RRGGBB color. */
function withAlpha(hex: string, alpha: number): string {
  const a = Math.round(Math.max(0, Math.min(1, alpha)) * 255)
    .toString(16)
    .padStart(2, "0");
  return `${hex}${a}`;
}

export function YantraLattice({ headers, rows, accentColor }: YantraLatticeProps) {
  if (rows.length === 0) return null;
  const dataCols = headers.length - 1; // first col is the row label
  return (
    <section
      style={{
        margin: "clamp(2.5rem, 5vh, 4rem) 0",
        position: "relative",
      }}
      aria-label="Yantra lattice — comparison matrix"
    >
      {/* Column headers — small mono labels above each hex column */}
      <div
        style={{
          display: "grid",
          gridTemplateColumns: `minmax(8rem, 14rem) repeat(${dataCols}, 1fr)`,
          gap: "clamp(0.5rem, 1vw, 1rem)",
          marginBottom: "clamp(0.75rem, 1.5vh, 1.25rem)",
          paddingLeft: "clamp(0.5rem, 1vw, 1rem)",
        }}
      >
        <div /> {/* placeholder under label column */}
        {headers.slice(1).map((h, ci) => (
          <div
            key={ci}
            style={{
              fontFamily: "var(--font-mono, 'SF Mono', monospace)",
              fontSize: "clamp(0.55rem, 0.7vw, 0.7rem)",
              letterSpacing: "0.32em",
              textTransform: "uppercase",
              color: tintFor(ci),
              opacity: 0.82,
              textAlign: "center",
              lineHeight: 1.4,
            }}
            dangerouslySetInnerHTML={{ __html: renderInline(h) }}
          />
        ))}
      </div>

      {/* Rows — label band + hex cells */}
      <div style={{ display: "grid", gap: "clamp(0.85rem, 1.8vh, 1.5rem)" }}>
        {rows.map((row, ri) => (
          <div
            key={ri}
            style={{
              display: "grid",
              gridTemplateColumns: `minmax(8rem, 14rem) repeat(${dataCols}, 1fr)`,
              gap: "clamp(0.5rem, 1vw, 1rem)",
              alignItems: "stretch",
            }}
          >
            {/* Row label — vesica-banded title on the left */}
            <div
              style={{
                position: "relative",
                display: "flex",
                alignItems: "center",
                padding: "0.6rem 1rem",
                fontFamily: "var(--font-display, 'Panchang', serif)",
                fontVariationSettings: "'wght' 600",
                fontSize: "clamp(0.85rem, 1.05vw, 1rem)",
                lineHeight: 1.25,
                color: "var(--c-parchment, #F0EDE3)",
                background: `linear-gradient(90deg, ${withAlpha(accentColor, 0.16)} 0%, ${withAlpha(accentColor, 0.04)} 100%)`,
                borderLeft: `2px solid ${accentColor}`,
                borderRadius: "0 12px 12px 0",
              }}
              dangerouslySetInnerHTML={{ __html: renderInline(row[0] ?? "") }}
            />

            {/* Data cells — hex-framed */}
            {row.slice(1).map((cell, ci) => {
              const tint = tintFor(ci);
              return (
                <div
                  key={ci}
                  style={{
                    position: "relative",
                    minHeight: "clamp(4rem, 7vh, 5.5rem)",
                    display: "flex",
                    flexDirection: "column",
                    alignItems: "center",
                    justifyContent: "center",
                    padding: "0.6rem 0.8rem",
                    background: `radial-gradient(120% 100% at 50% 0%, ${withAlpha(tint, 0.18)} 0%, ${withAlpha(tint, 0.03)} 60%, transparent 100%)`,
                    border: `1px solid ${withAlpha(tint, 0.32)}`,
                    borderRadius: "10px",
                    overflow: "hidden",
                  }}
                >
                  {/* Decorative hex frame in the corner */}
                  <div
                    aria-hidden="true"
                    style={{
                      position: "absolute",
                      top: "-3px",
                      right: "-3px",
                      pointerEvents: "none",
                      opacity: 0.55,
                    }}
                  >
                    <HexFrame size={28} color={tint} strokeWidth={1.0} />
                  </div>

                  <div
                    style={{
                      fontFamily: "var(--font-body, 'Satoshi', sans-serif)",
                      fontSize: "clamp(0.78rem, 0.95vw, 0.95rem)",
                      lineHeight: 1.4,
                      color: "var(--c-parchment, #F0EDE3)",
                      textAlign: "center",
                    }}
                    dangerouslySetInnerHTML={{ __html: renderInline(cell ?? "") }}
                  />
                </div>
              );
            })}
          </div>
        ))}
      </div>
    </section>
  );
}
