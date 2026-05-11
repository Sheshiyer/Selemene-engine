/**
 * King Wen sequence: hexagram number (1-indexed) → 6-bit integer.
 * Bits 0..5 correspond to lines 1..6 (bottom to top). 1 = yang (solid), 0 = yin (broken).
 */
const KING_WEN: readonly number[] = [
  /* 0 (unused) */ 0,
  /* 1  ䷀ */ 63, /* 2  ䷁ */  0, /* 3  ䷂ */ 34, /* 4  ䷃ */ 17,
  /* 5  ䷄ */ 55, /* 6  ䷅ */ 46, /* 7  ䷆ */  2, /* 8  ䷇ */ 16,
  /* 9  ䷈ */ 55, /* 10 ䷉ */ 59, /* 11 ䷊ */  7, /* 12 ䷋ */ 56,
  /* 13 ䷌ */ 53, /* 14 ䷍ */ 47, /* 15 ䷎ */  4, /* 16 ䷏ */  8,
  /* 17 ䷐ */ 25, /* 18 ䷑ */ 38, /* 19 ䷒ */  3, /* 20 ䷓ */ 48,
  /* 21 ䷔ */ 41, /* 22 ䷕ */ 37, /* 23 ䷖ */ 32, /* 24 ䷗ */  1,
  /* 25 ䷘ */ 57, /* 26 ䷙ */ 39, /* 27 ䷚ */ 33, /* 28 ䷛ */ 30,
  /* 29 ䷜ */ 18, /* 30 ䷝ */ 45, /* 31 ䷞ */ 28, /* 32 ䷟ */ 14,
  /* 33 ䷠ */ 60, /* 34 ䷡ */ 15, /* 35 ䷢ */ 40, /* 36 ䷣ */  5,
  /* 37 ䷤ */ 53, /* 38 ䷥ */ 43, /* 39 ䷦ */ 20, /* 40 ䷧ */ 10,
  /* 41 ䷨ */ 35, /* 42 ䷩ */ 28, /* 43 ䷪ */ 31, /* 44 ䷫ */ 62,
  /* 45 ䷬ */ 24, /* 46 ䷭ */  6, /* 47 ䷮ */ 22, /* 48 ䷯ */ 26,
  /* 49 ䷰ */ 45, /* 50 ䷱ */ 46, /* 51 ䷲ */  9, /* 52 ䷳ */ 36,
  /* 53 ䷴ */ 52, /* 54 ䷵ */ 11, /* 55 ䷶ */ 13, /* 56 ䷷ */ 44,
  /* 57 ䷸ */ 54, /* 58 ䷹ */ 27, /* 59 ䷺ */ 50, /* 60 ䷻ */ 19,
  /* 61 ䷼ */ 51, /* 62 ䷽ */ 12, /* 63 ䷾ */ 21, /* 64 ䷿ */ 42,
] as const;

/** Resolve the 6 lines (bottom→top) as booleans. true = yang (solid). */
function resolveLines(result: Record<string, unknown>): boolean[] {
  // If the engine already provided lines as an array of booleans / 0|1
  const raw = result.lines;
  if (Array.isArray(raw) && raw.length === 6) {
    return raw.map((v) => !!v);
  }

  // Look up King Wen table by hexagram number
  const num = Number(result.hexagram_number ?? result.number ?? result.hexagram ?? 0);
  if (num >= 1 && num <= 64) {
    const bits = KING_WEN[num];
    return Array.from({ length: 6 }, (_, i) => !!((bits >> i) & 1));
  }

  // Fallback: approximate visual from number as raw bits (not King Wen accurate)
  if (num > 0) {
    return Array.from({ length: 6 }, (_, i) => !!((num >> i) & 1));
  }

  return [true, true, true, true, true, true]; // default: all yang
}

/** Which line positions (1-based, bottom=1) are changing */
function resolveChangingPositions(result: Record<string, unknown>): Set<number> {
  const raw = result.changing_lines;
  if (!Array.isArray(raw)) return new Set();

  const positions = new Set<number>();
  for (const entry of raw) {
    if (typeof entry === "number") {
      positions.add(entry);
    } else if (typeof entry === "object" && entry !== null && "line" in entry) {
      positions.add(Number((entry as Record<string, unknown>).line));
    } else {
      // Fallback: index+1 position for non-structured entries
    }
  }
  return positions;
}

// ─── SVG Hexagram Drawing ────────────────────────────────────────────────────

const SVG_W = 80;
const LINE_W = 60;
const LINE_H = 6;
const GAP_Y = 10;        // vertical gap between lines
const CELL_H = LINE_H + GAP_Y; // 16px per line cell
const SVG_H = 6 * LINE_H + 5 * GAP_Y; // 86px
const X_OFF = (SVG_W - LINE_W) / 2; // 10px left offset

const YANG_COLOR = "#C5A017";
const YIN_COLOR = "rgba(197,160,23,0.4)";
const DOT_RADIUS = 3;
const DOT_COLOR = "#C5A017";

interface HexagramSvgProps {
  lines: boolean[];
  changingPositions: Set<number>;
}

function HexagramSvg({ lines, changingPositions }: HexagramSvgProps) {
  return (
    <svg
      width={SVG_W}
      height={SVG_H}
      viewBox={`0 0 ${SVG_W} ${SVG_H}`}
      role="img"
      aria-label="Hexagram line drawing"
      style={{ flexShrink: 0 }}
    >
      {lines.map((isYang, idx) => {
        // idx 0 = line 1 (bottom), rendered at the bottom of the SVG
        const lineNum = idx + 1;
        const y = SVG_H - LINE_H - idx * CELL_H; // bottom-up positioning
        const isChanging = changingPositions.has(lineNum);

        return (
          <g key={lineNum}>
            {isYang ? (
              /* Yang: single solid line */
              <rect
                x={X_OFF}
                y={y}
                width={LINE_W}
                height={LINE_H}
                rx={3}
                ry={3}
                fill={YANG_COLOR}
              />
            ) : (
              /* Yin: two segments with gap */
              <>
                <rect
                  x={X_OFF}
                  y={y}
                  width={24}
                  height={LINE_H}
                  rx={3}
                  ry={3}
                  fill={YIN_COLOR}
                />
                <rect
                  x={X_OFF + 36}
                  y={y}
                  width={24}
                  height={LINE_H}
                  rx={3}
                  ry={3}
                  fill={YIN_COLOR}
                />
              </>
            )}

            {/* Changing line indicator: small dot to the right */}
            {isChanging && (
              <circle
                cx={X_OFF + LINE_W + 6}
                cy={y + LINE_H / 2}
                r={DOT_RADIUS}
                fill={DOT_COLOR}
              />
            )}
          </g>
        );
      })}
    </svg>
  );
}

// ─── Styles ──────────────────────────────────────────────────────────────────

const styles = {
  container: { display: "flex", flexDirection: "column" as const, gap: "0.75rem" },
  hexHeader: {
    display: "flex",
    alignItems: "center" as const,
    gap: "0.75rem",
    padding: "0.75rem 1rem",
    background: "var(--field)",
    borderRadius: "var(--radius)",
  },
  hexInfo: {
    display: "flex",
    flexDirection: "column" as const,
    gap: "0.15rem",
  },
  hexNumber: {
    fontSize: "2rem",
    fontWeight: 800,
    fontFamily: "var(--font-display)",
    color: "var(--gold)",
    lineHeight: 1,
  },
  hexName: {
    fontSize: "1.1rem",
    fontWeight: 700,
    color: "var(--text)",
  },
  section: {
    display: "flex",
    flexDirection: "column" as const,
    gap: "0.25rem",
    paddingLeft: "0.5rem",
    borderLeft: "2px solid var(--line-gold)",
  },
  sectionTitle: {
    fontSize: "0.75rem",
    color: "var(--text-muted)",
    textTransform: "uppercase" as const,
    letterSpacing: "0.06em",
    fontWeight: 600,
  },
  text: {
    fontSize: "0.9rem",
    color: "var(--text)",
    lineHeight: 1.5,
  },
  changingLine: {
    fontSize: "0.85rem",
    color: "var(--gold)",
    padding: "0.25rem 0",
  },
};

// ─── Helpers ─────────────────────────────────────────────────────────────────

function str(v: unknown): string {
  if (v === null || v === undefined) return "—";
  return String(v);
}

function arr(v: unknown): unknown[] {
  return Array.isArray(v) ? v : [];
}

// ─── Component ───────────────────────────────────────────────────────────────

interface IChingProps {
  result: Record<string, unknown>;
}

export default function IChing({ result }: IChingProps) {
  const number = result.hexagram_number ?? result.number ?? result.hexagram;
  const name = result.hexagram_name ?? result.name;
  const judgment = result.judgment ?? result.judgement;
  const image = result.image;
  const changing = arr(result.changing_lines);

  const lines = resolveLines(result);
  const changingPositions = resolveChangingPositions(result);

  return (
    <div style={styles.container}>
      <div style={styles.hexHeader}>
        <HexagramSvg lines={lines} changingPositions={changingPositions} />
        <div style={styles.hexInfo}>
          {number != null && <span style={styles.hexNumber}>#{str(number)}</span>}
          {name != null && <span style={styles.hexName}>{str(name)}</span>}
        </div>
      </div>

      {judgment != null && (
        <div style={styles.section}>
          <span style={styles.sectionTitle}>Judgment</span>
          <p style={styles.text}>{str(judgment)}</p>
        </div>
      )}

      {image != null && (
        <div style={styles.section}>
          <span style={styles.sectionTitle}>Image</span>
          <p style={styles.text}>{str(image)}</p>
        </div>
      )}

      {changing.length > 0 && (
        <div style={styles.section}>
          <span style={styles.sectionTitle}>Changing Lines</span>
          {changing.map((line, i) => (
            <p key={i} style={styles.changingLine}>
              Line {i + 1}: {str(line)}
            </p>
          ))}
        </div>
      )}
    </div>
  );
}
