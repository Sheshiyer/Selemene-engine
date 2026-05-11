import { useMemo } from "react";

/* ── constants ─────────────────────────────────────────────────────── */

const NAKSHATRAS: readonly string[] = [
  "Ashwini", "Bharani", "Krittika", "Rohini", "Mrigashira", "Ardra",
  "Punarvasu", "Pushya", "Ashlesha", "Magha", "Purva Phalguni",
  "Uttara Phalguni", "Hasta", "Chitra", "Swati", "Vishakha",
  "Anuradha", "Jyeshtha", "Mula", "Purva Ashadha", "Uttara Ashadha",
  "Shravana", "Dhanishtha", "Shatabhisha", "Purva Bhadrapada",
  "Uttara Bhadrapada", "Revati",
] as const;

const CX = 110;
const CY = 110;
const OUTER_R = 90;
const INNER_R = 60;
const TITHI_R = 54;
const SEG_DEG = 360 / 27;
const SEG_GAP = 0.6; // degrees gap between segments

const COL_INACTIVE = "rgba(197,160,23,0.12)";
const COL_ACTIVE_FILL = "rgba(16,181,167,0.45)";
const COL_ACTIVE_STROKE = "rgba(16,181,167,0.9)";
const COL_GOLD = "#C5A017";

/* ── styles ────────────────────────────────────────────────────────── */

const styles = {
  grid: {
    display: "grid",
    gridTemplateColumns: "repeat(auto-fit, minmax(200px, 1fr))",
    gap: "0.75rem",
  },
  cell: {
    background: "var(--field)",
    borderRadius: "var(--radius)",
    padding: "0.75rem",
    display: "flex",
    flexDirection: "column" as const,
    gap: "0.25rem",
  },
  label: {
    fontSize: "0.7rem",
    color: "var(--text-muted)",
    textTransform: "uppercase" as const,
    letterSpacing: "0.06em",
    fontWeight: 600,
  },
  value: {
    fontSize: "1rem",
    fontWeight: 600,
    color: "var(--text)",
  },
  sub: {
    fontSize: "0.8rem",
    color: "var(--text-muted)",
    fontFamily: "var(--font-mono)",
  },
  quality: {
    display: "inline-block",
    padding: "0.15rem 0.5rem",
    borderRadius: 4,
    fontSize: "0.7rem",
    fontWeight: 600,
    textTransform: "uppercase" as const,
  },
  ringWrap: {
    display: "flex",
    justifyContent: "center",
    marginTop: "1rem",
  },
  ringContainer: {
    maxWidth: 220,
    width: "100%",
  },
};

/* ── helpers ────────────────────────────────────────────────────────── */

function str(v: unknown): string {
  if (v === null || v === undefined) return "—";
  if (typeof v === "object") return JSON.stringify(v);
  return String(v);
}

function obj(v: unknown): Record<string, unknown> {
  if (v && typeof v === "object" && !Array.isArray(v))
    return v as Record<string, unknown>;
  return {};
}

/** Polar → cartesian, angle in degrees (0° = 3 o'clock, CW) */
function polar(cx: number, cy: number, r: number, deg: number): [number, number] {
  const rad = ((deg - 90) * Math.PI) / 180; // rotate so 0° = 12 o'clock
  return [cx + r * Math.cos(rad), cy + r * Math.sin(rad)];
}

/** SVG arc-only path (no fill, used for tithi indicator) */
function arcPath(
  cx: number, cy: number, r: number,
  startDeg: number, endDeg: number,
): string {
  const [x1, y1] = polar(cx, cy, r, startDeg);
  const [x2, y2] = polar(cx, cy, r, endDeg);
  const large = endDeg - startDeg > 180 ? 1 : 0;
  return `M ${x1} ${y1} A ${r} ${r} 0 ${large} 1 ${x2} ${y2}`;
}

/** Donut-segment closed path (outer arc CW, inner arc CCW) */
function segmentPath(
  cx: number, cy: number,
  outerR: number, innerR: number,
  startDeg: number, endDeg: number,
): string {
  const [ox1, oy1] = polar(cx, cy, outerR, startDeg);
  const [ox2, oy2] = polar(cx, cy, outerR, endDeg);
  const [ix1, iy1] = polar(cx, cy, innerR, endDeg);
  const [ix2, iy2] = polar(cx, cy, innerR, startDeg);
  const large = endDeg - startDeg > 180 ? 1 : 0;
  return [
    `M ${ox1} ${oy1}`,
    `A ${outerR} ${outerR} 0 ${large} 1 ${ox2} ${oy2}`,
    `L ${ix1} ${iy1}`,
    `A ${innerR} ${innerR} 0 ${large} 0 ${ix2} ${iy2}`,
    "Z",
  ].join(" ");
}

/** Case-insensitive partial match of a name against the 27 nakshatras. Returns 0-26 or -1 */
function findNakshatraIndex(name: string): number {
  const lower = name.toLowerCase().trim();
  // exact match first
  const exact = NAKSHATRAS.findIndex((n) => n.toLowerCase() === lower);
  if (exact !== -1) return exact;
  // partial match (name starts with or contains)
  return NAKSHATRAS.findIndex(
    (n) => n.toLowerCase().startsWith(lower) || lower.startsWith(n.toLowerCase()),
  );
}

/** Extract tithi number (1-30) from result */
function parseTithiNumber(raw: unknown): number {
  const o = obj(raw);
  if (typeof o.number === "number") return o.number;
  if (typeof o.number === "string") {
    const n = parseInt(o.number, 10);
    if (!isNaN(n)) return n;
  }
  // try to parse from name like "Shukla Pratipada" → not reliable, fallback 0
  if (typeof raw === "number") return raw;
  if (typeof raw === "string") {
    const n = parseInt(raw, 10);
    if (!isNaN(n)) return n;
  }
  return 0;
}

/* ── ring sub-component ────────────────────────────────────────────── */

interface RingProps {
  activeIndex: number; // 0-26, -1 for none
  activeName: string;
  activePada: string;
  tithiNumber: number; // 1-30, 0 = hide
}

function NakshatraRing({ activeIndex, activeName, activePada, tithiNumber }: RingProps) {
  const segments = useMemo(() => {
    const result: Array<{ d: string; active: boolean; idx: number }> = [];
    for (let i = 0; i < 27; i++) {
      const start = i * SEG_DEG + SEG_GAP / 2;
      const end = (i + 1) * SEG_DEG - SEG_GAP / 2;
      result.push({
        d: segmentPath(CX, CY, OUTER_R, INNER_R, start, end),
        active: i === activeIndex,
        idx: i,
      });
    }
    return result;
  }, [activeIndex]);

  // tithi arc: map 1-30 → 0-360°
  const tithiArc = useMemo(() => {
    if (tithiNumber < 1 || tithiNumber > 30) return null;
    const endDeg = (tithiNumber / 30) * 360;
    return arcPath(CX, CY, TITHI_R, 0, endDeg);
  }, [tithiNumber]);

  const displayNumber = activeIndex >= 0 ? activeIndex + 1 : null;

  return (
    <svg
      viewBox="0 0 220 220"
      width="100%"
      height="100%"
      style={{ display: "block" }}
      role="img"
      aria-label={`Nakshatra ring — active: ${activeName || "none"}`}
    >
      {/* glow filter for active segment */}
      <defs>
        <filter id="panch-glow" x="-30%" y="-30%" width="160%" height="160%">
          <feGaussianBlur in="SourceGraphic" stdDeviation="3" result="blur" />
          <feMerge>
            <feMergeNode in="blur" />
            <feMergeNode in="SourceGraphic" />
          </feMerge>
        </filter>
      </defs>

      {/* 27 segments */}
      {segments.map((seg) => (
        <path
          key={seg.idx}
          d={seg.d}
          fill={seg.active ? COL_ACTIVE_FILL : COL_INACTIVE}
          stroke={seg.active ? COL_ACTIVE_STROKE : "transparent"}
          strokeWidth={seg.active ? 1.5 : 0}
          filter={seg.active ? "url(#panch-glow)" : undefined}
        />
      ))}

      {/* tithi arc indicator (inside the donut) */}
      {tithiArc && (
        <path
          d={tithiArc}
          fill="none"
          stroke={COL_GOLD}
          strokeWidth={2.5}
          strokeLinecap="round"
          opacity={0.85}
        />
      )}

      {/* center labels */}
      {activeIndex >= 0 && (
        <>
          <text
            x={CX}
            y={CY - 10}
            textAnchor="middle"
            dominantBaseline="auto"
            fill={COL_GOLD}
            fontFamily="var(--font-display), serif"
            fontSize="12"
            fontWeight={700}
          >
            {activeName}
          </text>
          <text
            x={CX}
            y={CY + 8}
            textAnchor="middle"
            dominantBaseline="auto"
            fill={COL_GOLD}
            fontFamily="var(--font-display), serif"
            fontSize="10"
            opacity={0.7}
          >
            #{displayNumber}{activePada ? ` · Pada ${activePada}` : ""}
          </text>
        </>
      )}
    </svg>
  );
}

/* ── main component ────────────────────────────────────────────────── */

interface PanchangaProps {
  result: Record<string, unknown>;
}

export default function Panchanga({ result }: PanchangaProps) {
  const tithi = obj(result.tithi);
  const nakshatra = obj(result.nakshatra);
  const yoga = obj(result.yoga);
  const karana = obj(result.karana);
  const vara = result.vara ?? result.weekday;
  const muhurta = obj(result.muhurta);
  const quality = str(muhurta.quality ?? result.quality ?? "");

  const qualityColor =
    quality.toLowerCase().includes("good") || quality.toLowerCase().includes("auspi")
      ? "var(--emerald)"
      : quality.toLowerCase().includes("bad") || quality.toLowerCase().includes("inauspi")
        ? "var(--danger)"
        : "var(--gold)";

  /* ── derive ring data ──────────────────────────────────────────── */
  const nakshatraName = str(nakshatra.name ?? result.nakshatra);
  const nakshatraPada = nakshatra.pada != null ? str(nakshatra.pada) : "";
  const activeIndex = nakshatraName !== "—" ? findNakshatraIndex(nakshatraName) : -1;
  const tithiNumber = parseTithiNumber(result.tithi);

  return (
    <>
      {/* existing text grid */}
      <div style={styles.grid}>
        <div style={styles.cell}>
          <span style={styles.label}>Tithi</span>
          <span style={styles.value}>{str(tithi.name ?? result.tithi)}</span>
          {tithi.percentage != null && (
            <span style={styles.sub}>{str(tithi.percentage)}% elapsed</span>
          )}
        </div>

        <div style={styles.cell}>
          <span style={styles.label}>Nakshatra</span>
          <span style={styles.value}>{str(nakshatra.name ?? result.nakshatra)}</span>
          {nakshatra.pada != null && <span style={styles.sub}>Pada {str(nakshatra.pada)}</span>}
          {nakshatra.lord != null && <span style={styles.sub}>Lord: {str(nakshatra.lord)}</span>}
        </div>

        <div style={styles.cell}>
          <span style={styles.label}>Yoga</span>
          <span style={styles.value}>{str(yoga.name ?? result.yoga)}</span>
        </div>

        <div style={styles.cell}>
          <span style={styles.label}>Karana</span>
          <span style={styles.value}>{str(karana.name ?? result.karana)}</span>
        </div>

        <div style={styles.cell}>
          <span style={styles.label}>Vara (Weekday)</span>
          <span style={styles.value}>{str(vara)}</span>
        </div>

        <div style={styles.cell}>
          <span style={styles.label}>Muhurta Quality</span>
          <span
            style={{
              ...styles.quality,
              background: `${qualityColor}20`,
              color: qualityColor,
            }}
          >
            {quality || "—"}
          </span>
        </div>
      </div>

      {/* nakshatra ring visualization */}
      <div style={styles.ringWrap}>
        <div style={styles.ringContainer}>
          <NakshatraRing
            activeIndex={activeIndex}
            activeName={activeIndex >= 0 ? NAKSHATRAS[activeIndex] : nakshatraName}
            activePada={nakshatraPada}
            tithiNumber={tithiNumber}
          />
        </div>
      </div>
    </>
  );
}
