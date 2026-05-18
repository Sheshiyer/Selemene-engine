// ─── SigilCascade — vertical stack for annotated-list tables ───────────
// Replaces a markdown table whose last column carries long prose (yoga
// names + descriptive sentences, combination + effect, etc.). Each row
// becomes a card with:
//   - a leading micro-sigil (cycled by row index)
//   - a name column (Panchang 600, prominent)
//   - optional middle column (status chip, mono small caps)
//   - a verse — the descriptive prose flowing as a sentence
//   - a hairline connector to the next row
//
// Also handles N=2 col tables (label + value) and ul/ol lists via a
// `mode` prop that strips chips when not applicable.

import { MicroSigil } from "../sigils";
import { renderInline } from "../parseBlocks";

interface SigilCascadeProps {
  headers?: string[];
  rows: string[][];
  accentColor: string;
  /** "list" mode hides chips/labels and just shows sigil + content. */
  mode?: "table" | "list";
  /** Optional caption rendered as the section eyebrow. */
  eyebrow?: string;
}

function withAlpha(hex: string, alpha: number): string {
  const a = Math.round(Math.max(0, Math.min(1, alpha)) * 255)
    .toString(16)
    .padStart(2, "0");
  return `${hex}${a}`;
}

export function SigilCascade({
  headers = [],
  rows,
  accentColor,
  mode = "table",
  eyebrow,
}: SigilCascadeProps) {
  if (rows.length === 0) return null;

  const nCols = mode === "list" ? 1 : headers.length || rows[0].length;
  const hasMiddleChip = mode === "table" && nCols >= 3;

  return (
    <section
      style={{
        margin: "clamp(2.5rem, 5vh, 4rem) 0",
        position: "relative",
      }}
      aria-label="Sigil cascade — annotated list"
    >
      {eyebrow && (
        <div
          style={{
            fontFamily: "var(--font-mono, 'SF Mono', monospace)",
            fontSize: "clamp(0.6rem, 0.75vw, 0.75rem)",
            letterSpacing: "0.4em",
            textTransform: "uppercase",
            color: accentColor,
            opacity: 0.78,
            marginBottom: "clamp(1rem, 2vh, 1.5rem)",
          }}
        >
          {eyebrow}
        </div>
      )}

      <ol
        style={{
          listStyle: "none",
          margin: 0,
          padding: 0,
          display: "grid",
          gap: "clamp(0.75rem, 1.5vh, 1.25rem)",
          position: "relative",
        }}
      >
        {/* Vertical connector — runs through all rows behind the sigils */}
        <div
          aria-hidden="true"
          style={{
            position: "absolute",
            left: "calc(clamp(0.75rem, 1vw, 1.1rem) + 12px)",
            top: "1rem",
            bottom: "1rem",
            width: "1px",
            background: `linear-gradient(180deg, ${withAlpha(accentColor, 0)}, ${withAlpha(accentColor, 0.35)} 12%, ${withAlpha(accentColor, 0.35)} 88%, ${withAlpha(accentColor, 0)})`,
            pointerEvents: "none",
          }}
        />

        {rows.map((row, ri) => {
          const name = row[0] ?? "";
          const status = hasMiddleChip ? row[1] : null;
          const description = hasMiddleChip
            ? row.slice(2).join(" — ")
            : mode === "list"
            ? row[0]
            : row.slice(1).join(" — ");
          const showName = mode === "table";
          return (
            <li
              key={ri}
              style={{
                display: "grid",
                gridTemplateColumns: "auto 1fr",
                gap: "clamp(0.75rem, 1.5vw, 1.25rem)",
                alignItems: "start",
                padding: "clamp(0.75rem, 1.5vh, 1.25rem) clamp(0.75rem, 1.5vw, 1.25rem)",
                background: `linear-gradient(135deg, ${withAlpha(accentColor, 0.05)} 0%, transparent 80%)`,
                borderLeft: `1px solid ${withAlpha(accentColor, 0.25)}`,
                borderRadius: "4px 14px 14px 4px",
              }}
            >
              {/* Leading sigil */}
              <div
                style={{
                  paddingTop: "2px",
                  filter: `drop-shadow(0 0 6px ${withAlpha(accentColor, 0.45)})`,
                }}
              >
                <MicroSigil index={ri} size={26} color={accentColor} />
              </div>

              <div style={{ display: "grid", gap: "0.4rem" }}>
                {/* Name + (optional) status chip side by side */}
                {showName && (
                  <div
                    style={{
                      display: "flex",
                      flexWrap: "wrap",
                      alignItems: "baseline",
                      gap: "0.75rem",
                    }}
                  >
                    <span
                      style={{
                        fontFamily: "var(--font-display, 'Panchang', serif)",
                        fontVariationSettings: "'wght' 620",
                        fontSize: "clamp(0.95rem, 1.15vw, 1.15rem)",
                        lineHeight: 1.2,
                        color: "var(--c-parchment, #F0EDE3)",
                      }}
                      dangerouslySetInnerHTML={{ __html: renderInline(name) }}
                    />
                    {status && (
                      <span
                        style={{
                          fontFamily: "var(--font-mono, 'SF Mono', monospace)",
                          fontSize: "clamp(0.6rem, 0.72vw, 0.72rem)",
                          letterSpacing: "0.25em",
                          textTransform: "uppercase",
                          color: accentColor,
                          padding: "0.2rem 0.6rem",
                          border: `1px solid ${withAlpha(accentColor, 0.35)}`,
                          borderRadius: "999px",
                          background: withAlpha(accentColor, 0.08),
                          whiteSpace: "nowrap",
                        }}
                        dangerouslySetInnerHTML={{ __html: renderInline(status) }}
                      />
                    )}
                  </div>
                )}

                {/* Description — verse-style sentence flow */}
                {description && (
                  <p
                    style={{
                      margin: 0,
                      fontFamily: "var(--font-body, 'Satoshi', sans-serif)",
                      fontSize: "clamp(0.92rem, 1.05vw, 1.08rem)",
                      lineHeight: 1.55,
                      color: "rgba(240, 237, 227, 0.86)",
                      maxWidth: "62ch",
                    }}
                    dangerouslySetInnerHTML={{ __html: renderInline(description) }}
                  />
                )}
              </div>
            </li>
          );
        })}
      </ol>
    </section>
  );
}
