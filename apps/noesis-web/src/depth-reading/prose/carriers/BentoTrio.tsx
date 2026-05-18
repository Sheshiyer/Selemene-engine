// ─── BentoTrio — compact hex-framed chips for severity rollups ─────────
// Replaces small ≤4-row × ≤2-col tables (Severity × Count, Score × Value,
// etc.). Each row becomes a hex-framed chip in a grid. The grid shape
// adapts to row count:
//   1 → single hero chip, centered
//   2 → side-by-side pair
//   3 → triangle (top-center + bottom-pair)
//   4 → 2×2 grid
//
// Each chip carries ONE data point: label (mono small caps) + value
// (Panchang 700 large). The hex frame outlines in the section's accent.
// Background gradient: Witness Violet → Flow Indigo (the Kha arc).

import { HexFrame } from "../sigils";
import { renderInline } from "../parseBlocks";

interface BentoTrioProps {
  headers?: string[];
  rows: string[][];
  accentColor: string;
}

function withAlpha(hex: string, alpha: number): string {
  const a = Math.round(Math.max(0, Math.min(1, alpha)) * 255)
    .toString(16)
    .padStart(2, "0");
  return `${hex}${a}`;
}

/** Determine grid template based on row count. */
function gridFor(n: number): { columns: string; areas?: string[] } {
  if (n === 1) return { columns: "1fr" };
  if (n === 2) return { columns: "1fr 1fr" };
  if (n === 3)
    return {
      columns: "1fr 1fr",
      areas: ["top top", "left right"],
    };
  return { columns: "1fr 1fr" }; // 4 → 2×2
}

export function BentoTrio({ headers = [], rows, accentColor }: BentoTrioProps) {
  if (rows.length === 0) return null;
  const grid = gridFor(rows.length);
  const headerLabel = headers[0] ?? null;
  const headerValue = headers[1] ?? null;

  return (
    <section
      style={{
        margin: "clamp(2.5rem, 5vh, 4rem) 0",
      }}
      aria-label="Bento rollup"
    >
      {/* Eyebrow */}
      {(headerLabel || headerValue) && (
        <div
          style={{
            fontFamily: "var(--font-mono, 'SF Mono', monospace)",
            fontSize: "clamp(0.6rem, 0.75vw, 0.72rem)",
            letterSpacing: "0.4em",
            textTransform: "uppercase",
            color: accentColor,
            opacity: 0.78,
            marginBottom: "clamp(1rem, 2vh, 1.4rem)",
            textAlign: "center",
          }}
        >
          {headerLabel}
          {headerValue ? ` · ${headerValue}` : ""}
        </div>
      )}

      <div
        style={{
          display: "grid",
          gridTemplateColumns: grid.columns,
          gap: "clamp(0.85rem, 1.8vw, 1.5rem)",
          maxWidth: "min(560px, 100%)",
          margin: "0 auto",
        }}
      >
        {rows.map((row, i) => {
          // Special placement for the 3-row triangle layout: row 0 spans both cols
          const spanTop = rows.length === 3 && i === 0;
          return (
            <div
              key={i}
              style={{
                gridColumn: spanTop ? "1 / -1" : undefined,
                position: "relative",
                aspectRatio: "1 / 1.16", // matches HexFrame proportions
                maxWidth: spanTop ? "280px" : "100%",
                margin: spanTop ? "0 auto" : undefined,
              }}
            >
              {/* Hex frame outline */}
              <HexFrame
                size={spanTop ? 240 : 200}
                color={accentColor}
                strokeWidth={1.25}
                style={{
                  position: "absolute",
                  inset: 0,
                  width: "100%",
                  height: "100%",
                }}
              />

              {/* Soft inner gradient */}
              <div
                aria-hidden="true"
                style={{
                  position: "absolute",
                  top: "12%",
                  left: "14%",
                  right: "14%",
                  bottom: "12%",
                  background: `radial-gradient(80% 70% at 50% 40%, ${withAlpha("#2D0050", 0.55)} 0%, ${withAlpha("#0B50FB", 0.18)} 60%, transparent 100%)`,
                  borderRadius: "50%",
                  filter: "blur(8px)",
                  zIndex: 0,
                }}
              />

              {/* Content stack */}
              <div
                style={{
                  position: "absolute",
                  inset: 0,
                  display: "flex",
                  flexDirection: "column",
                  alignItems: "center",
                  justifyContent: "center",
                  padding: "1rem",
                  textAlign: "center",
                  zIndex: 1,
                }}
              >
                {/* Label */}
                <div
                  style={{
                    fontFamily: "var(--font-mono, 'SF Mono', monospace)",
                    fontSize: "clamp(0.55rem, 0.7vw, 0.7rem)",
                    letterSpacing: "0.32em",
                    textTransform: "uppercase",
                    color: accentColor,
                    opacity: 0.88,
                    marginBottom: "0.5rem",
                    lineHeight: 1.3,
                    maxWidth: "16ch",
                  }}
                  dangerouslySetInnerHTML={{ __html: renderInline(row[0] ?? "") }}
                />
                {/* Value */}
                <div
                  style={{
                    fontFamily: "var(--font-display, 'Panchang', serif)",
                    fontVariationSettings: "'wght' 700",
                    fontSize: spanTop
                      ? "clamp(2rem, 3.5vw, 3rem)"
                      : "clamp(1.4rem, 2.6vw, 2.2rem)",
                    lineHeight: 1,
                    color: "var(--c-parchment, #F0EDE3)",
                    textShadow: `0 2px 22px ${withAlpha(accentColor, 0.45)}`,
                  }}
                  dangerouslySetInnerHTML={{ __html: renderInline(row[1] ?? "—") }}
                />
              </div>
            </div>
          );
        })}
      </div>
    </section>
  );
}
