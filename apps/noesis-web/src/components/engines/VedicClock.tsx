import { useMemo } from "react";
import GenericEngineView from "./GenericEngineView";

/* ── Types ─────────────────────────────────────────────────────── */

interface VedicClockProps {
  result: Record<string, unknown>;
}

interface OrganEntry {
  organ: string;
  abbr: string;
  startHour: number;
  element: "Metal" | "Earth" | "Fire" | "Water" | "Wood";
}

/* ── Constants ─────────────────────────────────────────────────── */

const ORGANS: OrganEntry[] = [
  { organ: "Lung",             abbr: "LU", startHour: 3,  element: "Metal" },
  { organ: "Large Intestine",  abbr: "LI", startHour: 5,  element: "Metal" },
  { organ: "Stomach",          abbr: "ST", startHour: 7,  element: "Earth" },
  { organ: "Spleen",           abbr: "SP", startHour: 9,  element: "Earth" },
  { organ: "Heart",            abbr: "HT", startHour: 11, element: "Fire" },
  { organ: "Small Intestine",  abbr: "SI", startHour: 13, element: "Fire" },
  { organ: "Bladder",          abbr: "BL", startHour: 15, element: "Water" },
  { organ: "Kidney",           abbr: "KI", startHour: 17, element: "Water" },
  { organ: "Pericardium",      abbr: "PC", startHour: 19, element: "Wood" },
  { organ: "Triple Warmer",    abbr: "TW", startHour: 21, element: "Wood" },
  { organ: "Gallbladder",      abbr: "GB", startHour: 23, element: "Wood" },
  { organ: "Liver",            abbr: "LV", startHour: 1,  element: "Wood" },
];

const ELEMENT_FILLS: Record<OrganEntry["element"], string> = {
  Metal: "rgba(200,200,220,0.3)",
  Earth: "rgba(197,160,23,0.3)",
  Fire:  "rgba(239,107,115,0.3)",
  Water: "rgba(11,80,251,0.3)",
  Wood:  "rgba(16,181,167,0.3)",
};

const ACTIVE_FILL   = "rgba(16,181,167,0.5)";
const ACTIVE_STROKE = "rgba(16,181,167,1.0)";
const INACTIVE_STROKE = "rgba(197,160,23,0.25)";

const CX = 120;
const CY = 120;
const R_OUTER = 100;
const R_INNER = 60;
const R_TEXT  = 80;
const DEG = Math.PI / 180;
const SEG_DEG = 30; // 360 / 12

/* ── Helpers ───────────────────────────────────────────────────── */

function str(v: unknown): string { return v == null ? "—" : String(v); }

/** Map a clock hour (0-23) to SVG angle in degrees where 3am = top (-90°). */
function hourToAngle(hour: number): number {
  return ((hour - 3) * 15) - 90; // 15° per hour for the hand
}

/** Map organ index (0-11) to its start angle in degrees. */
function segStartAngle(index: number): number {
  return (index * SEG_DEG) - 90;
}

/** Build an SVG arc path for a donut segment. */
function arcPath(startDeg: number, endDeg: number, rOuter: number, rInner: number): string {
  const s1 = startDeg * DEG;
  const e1 = endDeg * DEG;
  const largeArc = (endDeg - startDeg) > 180 ? 1 : 0;

  const ox1 = CX + rOuter * Math.cos(s1);
  const oy1 = CY + rOuter * Math.sin(s1);
  const ox2 = CX + rOuter * Math.cos(e1);
  const oy2 = CY + rOuter * Math.sin(e1);

  const ix2 = CX + rInner * Math.cos(e1);
  const iy2 = CY + rInner * Math.sin(e1);
  const ix1 = CX + rInner * Math.cos(s1);
  const iy1 = CY + rInner * Math.sin(s1);

  return [
    `M ${ox1} ${oy1}`,
    `A ${rOuter} ${rOuter} 0 ${largeArc} 1 ${ox2} ${oy2}`,
    `L ${ix2} ${iy2}`,
    `A ${rInner} ${rInner} 0 ${largeArc} 0 ${ix1} ${iy1}`,
    "Z",
  ].join(" ");
}

/** Find active organ index from result data. */
function findActiveIndex(organName: string | undefined): number {
  if (!organName) return -1;
  const lower = organName.toLowerCase();
  return ORGANS.findIndex((o) => o.organ.toLowerCase() === lower);
}

/** Format hour as "HH:00". */
function fmtHour(h: number): string {
  const hh = ((h % 24) + 24) % 24;
  return `${String(hh).padStart(2, "0")}:00`;
}

/* ── SVG Sub-Components ────────────────────────────────────────── */

function OrganSegment({ entry, index, isActive }: { entry: OrganEntry; index: number; isActive: boolean }) {
  const startDeg = segStartAngle(index);
  const endDeg   = startDeg + SEG_DEG;
  const midDeg   = startDeg + SEG_DEG / 2;
  const midRad   = midDeg * DEG;

  const tx = CX + R_TEXT * Math.cos(midRad);
  const ty = CY + R_TEXT * Math.sin(midRad);

  // Rotate text so it reads outward — flip for bottom half
  const textRotation = midDeg > 90 && midDeg < 270
    ? midDeg + 180
    : midDeg;

  const fill   = isActive ? ACTIVE_FILL : ELEMENT_FILLS[entry.element];
  const stroke = isActive ? ACTIVE_STROKE : INACTIVE_STROKE;
  const sw     = isActive ? 2 : 0.5;

  return (
    <g>
      <path
        d={arcPath(startDeg, endDeg, R_OUTER, R_INNER)}
        fill={fill}
        stroke={stroke}
        strokeWidth={sw}
      />
      <text
        x={tx}
        y={ty}
        textAnchor="middle"
        dominantBaseline="central"
        transform={`rotate(${textRotation}, ${tx}, ${ty})`}
        fill={isActive ? "#fff" : "rgba(255,255,255,0.6)"}
        fontSize={isActive ? 9 : 7.5}
        fontWeight={isActive ? 700 : 500}
        fontFamily="var(--font-mono)"
      >
        {entry.abbr}
      </text>
    </g>
  );
}

function ClockHand() {
  const now = new Date();
  const hours = now.getHours() + now.getMinutes() / 60;
  const angle = hourToAngle(hours) * DEG;
  const handLen = R_INNER - 4;
  const x2 = CX + handLen * Math.cos(angle);
  const y2 = CY + handLen * Math.sin(angle);

  return (
    <line
      x1={CX}
      y1={CY}
      x2={x2}
      y2={y2}
      stroke="rgba(16,181,167,0.8)"
      strokeWidth={1.5}
      strokeLinecap="round"
    />
  );
}

function CenterLabel({ organName, timeRange }: { organName: string; timeRange: string }) {
  return (
    <g>
      <text
        x={CX}
        y={CY - 6}
        textAnchor="middle"
        dominantBaseline="central"
        fill="#fff"
        fontSize={12}
        fontWeight={700}
        fontFamily="var(--font-display)"
      >
        {organName}
      </text>
      <text
        x={CX}
        y={CY + 10}
        textAnchor="middle"
        dominantBaseline="central"
        fill="rgba(255,255,255,0.5)"
        fontSize={9}
        fontFamily="var(--font-mono)"
      >
        {timeRange}
      </text>
    </g>
  );
}

/* ── TCM Clock Ring ────────────────────────────────────────────── */

function TCMClockRing({ activeOrgan, timeRange }: { activeOrgan: string | undefined; timeRange: string | undefined }) {
  const activeIdx = useMemo(() => findActiveIndex(activeOrgan), [activeOrgan]);

  const displayOrgan = activeOrgan ?? ORGANS[0].organ;
  const displayRange = timeRange ?? `${fmtHour(ORGANS[0].startHour)}-${fmtHour(ORGANS[0].startHour + 2)}`;

  return (
    <div style={{ display: "flex", justifyContent: "center" }}>
      <svg
        viewBox="0 0 240 240"
        width="240"
        height="240"
        style={{ maxWidth: "240px", width: "100%" }}
        role="img"
        aria-label={`TCM Organ Clock — current organ: ${displayOrgan}`}
      >
        {/* Segments */}
        {ORGANS.map((entry, i) => (
          <OrganSegment
            key={entry.abbr}
            entry={entry}
            index={i}
            isActive={i === activeIdx}
          />
        ))}

        {/* Clock hand */}
        <ClockHand />

        {/* Small center dot */}
        <circle cx={CX} cy={CY} r={2} fill="rgba(16,181,167,0.6)" />

        {/* Center label */}
        <CenterLabel organName={displayOrgan} timeRange={displayRange} />
      </svg>
    </div>
  );
}

/* ── Styles ────────────────────────────────────────────────────── */

const s = {
  grid: { display: "grid", gridTemplateColumns: "repeat(auto-fit, minmax(200px, 1fr))", gap: "0.75rem" },
  cell: { background: "var(--field)", borderRadius: "var(--radius)", padding: "0.75rem", display: "flex", flexDirection: "column" as const, gap: "0.25rem" },
  label: { fontSize: "0.7rem", color: "var(--text-muted)", textTransform: "uppercase" as const, letterSpacing: "0.06em", fontWeight: 600 },
  value: { fontSize: "1rem", fontWeight: 600, color: "var(--gold)" },
  sub: { fontSize: "0.8rem", color: "var(--text-muted)", fontFamily: "var(--font-mono)" },
  section: { display: "flex", flexDirection: "column" as const, gap: "0.5rem", marginTop: "0.5rem" },
  sectionTitle: { fontSize: "0.75rem", color: "var(--text-muted)", textTransform: "uppercase" as const, fontWeight: 600, letterSpacing: "0.08em" },
  tag: { display: "inline-block", padding: "0.2rem 0.5rem", borderRadius: 4, background: "var(--gold-soft)", color: "var(--gold)", fontSize: "0.75rem", fontWeight: 600, marginRight: "0.25rem", marginBottom: "0.25rem" },
};

/* ── Main Component ────────────────────────────────────────────── */

export default function VedicClock({ result }: VedicClockProps) {
  const tcm = result.tcm_organ_clock as Record<string, unknown> | undefined;
  const ayurvedic = result.ayurvedic_timing as Record<string, unknown> | undefined;
  const current = tcm?.current_organ as Record<string, unknown> | undefined;
  const peaks = tcm?.peak_organs as Record<string, unknown>[] | undefined;

  if (!tcm && !ayurvedic) return <GenericEngineView result={result} />;

  const organName  = current?.organ as string | undefined;
  const timeRange  = current?.time_range as string | undefined;

  return (
    <div style={{ display: "flex", flexDirection: "column", gap: "1rem" }}>
      {/* ── SVG Organ Clock Ring (above text cells) ── */}
      <TCMClockRing activeOrgan={organName} timeRange={timeRange} />

      {/* ── Existing text cells ── */}
      {current != null && (
        <div style={s.grid}>
          <div style={s.cell}>
            <span style={s.label}>Current Organ</span>
            <span style={s.value}>{str(current.organ)}</span>
            <span style={s.sub}>{str(current.element)} · {str(current.time_range)}</span>
          </div>
          <div style={s.cell}>
            <span style={s.label}>Emotion / Virtue</span>
            <span style={s.value}>{str(current.emotion)}</span>
            <span style={s.sub}>{str(current.virtue)}</span>
          </div>
          {current.recommendation != null && (
            <div style={{ ...s.cell, gridColumn: "1 / -1" }}>
              <span style={s.label}>Recommendation</span>
              <span style={{ fontSize: "0.875rem", color: "var(--text)", lineHeight: 1.5 }}>{str(current.recommendation)}</span>
            </div>
          )}
        </div>
      )}

      {ayurvedic != null && (
        <div style={s.grid}>
          <div style={s.cell}>
            <span style={s.label}>Dosha</span>
            <span style={s.value}>{str(ayurvedic.dominant_dosha)}</span>
          </div>
          <div style={s.cell}>
            <span style={s.label}>Time Period</span>
            <span style={s.value}>{str(ayurvedic.period)}</span>
            <span style={s.sub}>{str(ayurvedic.rasa)}</span>
          </div>
        </div>
      )}

      {peaks != null && peaks.length > 0 && (
        <div style={s.section}>
          <span style={s.sectionTitle}>Peak Organs Today</span>
          <div style={{ display: "flex", flexWrap: "wrap" as const }}>
            {peaks.map((p, i) => (
              <span key={i} style={s.tag}>{str(p.organ)} {str(p.time_range)}</span>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}
