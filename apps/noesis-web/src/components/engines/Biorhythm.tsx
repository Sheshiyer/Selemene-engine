/* ── helpers ─────────────────────────────────────────────────────── */

function num(v: unknown): number {
  if (typeof v === "number") return v;
  if (typeof v === "string") {
    const n = parseFloat(v);
    return isNaN(n) ? 0 : n;
  }
  return 0;
}

function str(v: unknown): string {
  if (v === null || v === undefined) return "—";
  return String(v);
}

function obj(v: unknown): Record<string, unknown> {
  if (v && typeof v === "object" && !Array.isArray(v))
    return v as Record<string, unknown>;
  return {};
}

/* ── cycle definitions ──────────────────────────────────────────── */

interface CycleInfo {
  key: string;
  label: string;
  color: string;
  period: number; // days
}

const CYCLES: CycleInfo[] = [
  { key: "physical", label: "Physical", color: "#ef6b73", period: 23 },
  { key: "emotional", label: "Emotional", color: "var(--c-indigo, #0B50FB)", period: 28 },
  { key: "intellectual", label: "Intellectual", color: "var(--c-gold, #C5A017)", period: 33 },
];

/* ── SVG constants ──────────────────────────────────────────────── */

const VB_W = 600;
const VB_H = 180;
const WAVE_AMPLITUDE = 70; // px half-range for the wave
const WAVE_CENTER_Y = VB_H / 2; // 90 — vertical center
const TOTAL_DAYS = 60;
const PAST_DAYS = 30;
const POINTS_PER_DAY = 4; // smooth curve via polyline oversampling
const TODAY_X = (PAST_DAYS / TOTAL_DAYS) * VB_W; // 300

/**
 * Given a cycle's current value (-100…+100) and its period,
 * compute the current phase in radians, then return the Y value
 * at each sub-day sample across the 60-day window.
 *
 * Phase derivation: val = sin(currentPhase) * 100
 *   → currentPhase = asin(clamp(val/100, -1, 1))
 *
 * We also check the derivative sign via phase_days if available,
 * otherwise default to the principal asin value.
 */
function buildWavePoints(
  val: number,
  period: number,
  phaseDays: number | null,
): string {
  const TWO_PI = 2 * Math.PI;
  const clamped = Math.max(-1, Math.min(1, val / 100));

  let currentPhase: number;
  if (phaseDays !== null) {
    // phase_days tells how many days into the cycle we are
    currentPhase = (TWO_PI * phaseDays) / period;
  } else {
    // infer from value — principal branch (ascending half)
    currentPhase = Math.asin(clamped);
  }

  const totalSamples = TOTAL_DAYS * POINTS_PER_DAY;
  const parts: string[] = [];

  for (let i = 0; i <= totalSamples; i++) {
    const dayOffset = (i / POINTS_PER_DAY) - PAST_DAYS; // -30 … +30
    const phase = currentPhase + (TWO_PI * dayOffset) / period;
    const y = WAVE_CENTER_Y - Math.sin(phase) * WAVE_AMPLITUDE;
    const x = (i / totalSamples) * VB_W;
    parts.push(`${x.toFixed(1)},${y.toFixed(1)}`);
  }

  return parts.join(" ");
}

/* ── styles ──────────────────────────────────────────────────────── */

const styles = {
  container: { display: "flex", flexDirection: "column" as const, gap: "1rem" },
  grid: {
    display: "grid",
    gridTemplateColumns: "repeat(auto-fit, minmax(160px, 1fr))",
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
    fontSize: "1.1rem",
    fontWeight: 700,
    fontFamily: "var(--font-mono)",
  },
  sub: {
    fontSize: "0.7rem",
    color: "var(--text-muted)",
    fontFamily: "var(--font-mono)",
  },
  svgWrap: {
    borderRadius: "var(--radius)",
    background: "rgba(7,11,29,0.6)",
    overflow: "hidden" as const,
  },
};

/* ── component ──────────────────────────────────────────────────── */

interface BiorhythmProps {
  result: Record<string, unknown>;
}

export default function Biorhythm({ result }: BiorhythmProps) {
  /* ── pre-compute per-cycle data ──────────────────────────────── */
  const cycleData = CYCLES.map((c) => {
    const raw = result[c.key];
    const data = obj(raw);
    const val = num(data.value ?? raw);
    const nextPeak = str(data.next_peak ?? "");
    const nextTrough = str(data.next_trough ?? "");

    const phaseDaysRaw = data.phase_days ?? data.phaseDays ?? data.day_in_cycle ?? null;
    const phaseDays = phaseDaysRaw !== null ? num(phaseDaysRaw) : null;

    const points = buildWavePoints(val, c.period, phaseDays);

    return { ...c, val, nextPeak, nextTrough, points };
  });

  /* ── day labels for x-axis ──────────────────────────────────── */
  const dayMarkers = [-30, -20, -10, 0, 10, 20, 30];

  return (
    <div style={styles.container}>
      {/* ── stat cells ─────────────────────────────────────────── */}
      <div style={styles.grid}>
        {cycleData.map((c) => (
          <div key={c.key} style={styles.cell}>
            <span style={styles.label}>{c.label}</span>
            <span style={{ ...styles.value, color: c.color }}>
              {c.val > 0 ? "+" : ""}
              {c.val}%
            </span>
            {c.nextPeak && c.nextPeak !== "—" && (
              <span style={styles.sub}>▲ Peak {c.nextPeak}</span>
            )}
            {c.nextTrough && c.nextTrough !== "—" && (
              <span style={styles.sub}>▼ Trough {c.nextTrough}</span>
            )}
          </div>
        ))}
      </div>

      {/* ── SVG wave chart ────────────────────────────────────── */}
      <div style={styles.svgWrap}>
        <svg
          viewBox={`0 0 ${VB_W} ${VB_H}`}
          width="100%"
          preserveAspectRatio="xMidYMid meet"
          role="img"
          aria-label="Biorhythm sine wave chart showing physical, emotional, and intellectual cycles over 60 days"
        >
          {/* ── zero axis ──────────────────────────────────────── */}
          <line
            x1={0}
            y1={WAVE_CENTER_Y}
            x2={VB_W}
            y2={WAVE_CENTER_Y}
            stroke="rgba(255,255,255,0.15)"
            strokeWidth={1}
          />

          {/* ── day grid lines + labels ────────────────────────── */}
          {dayMarkers.map((d) => {
            const x = ((d + PAST_DAYS) / TOTAL_DAYS) * VB_W;
            return (
              <g key={d}>
                {d !== 0 && (
                  <line
                    x1={x}
                    y1={8}
                    x2={x}
                    y2={VB_H - 8}
                    stroke="rgba(255,255,255,0.06)"
                    strokeWidth={0.5}
                  />
                )}
                <text
                  x={x}
                  y={VB_H - 2}
                  textAnchor="middle"
                  fill="rgba(255,255,255,0.35)"
                  fontSize={9}
                  fontFamily="var(--font-mono)"
                >
                  {d === 0 ? "today" : d > 0 ? `+${d}d` : `${d}d`}
                </text>
              </g>
            );
          })}

          {/* ── +100 / -100 labels ─────────────────────────────── */}
          <text
            x={4}
            y={WAVE_CENTER_Y - WAVE_AMPLITUDE + 3}
            fill="rgba(255,255,255,0.2)"
            fontSize={8}
            fontFamily="var(--font-mono)"
          >
            +100
          </text>
          <text
            x={4}
            y={WAVE_CENTER_Y + WAVE_AMPLITUDE + 3}
            fill="rgba(255,255,255,0.2)"
            fontSize={8}
            fontFamily="var(--font-mono)"
          >
            -100
          </text>

          {/* ── sine waves ─────────────────────────────────────── */}
          {cycleData.map((c) => (
            <polyline
              key={c.key}
              points={c.points}
              fill="none"
              stroke={c.color}
              strokeWidth={1.8}
              strokeLinejoin="round"
              strokeLinecap="round"
              opacity={0.85}
            />
          ))}

          {/* ── today marker (vertical dashed) ─────────────────── */}
          <line
            x1={TODAY_X}
            y1={6}
            x2={TODAY_X}
            y2={VB_H - 12}
            stroke="rgba(255,255,255,0.5)"
            strokeWidth={1}
            strokeDasharray="4 3"
          />

          {/* ── dots at today's value ──────────────────────────── */}
          {cycleData.map((c) => {
            const y =
              WAVE_CENTER_Y -
              (Math.max(-100, Math.min(100, c.val)) / 100) * WAVE_AMPLITUDE;
            return (
              <circle
                key={`dot-${c.key}`}
                cx={TODAY_X}
                cy={y}
                r={4}
                fill={c.color}
                stroke="rgba(7,11,29,0.8)"
                strokeWidth={1.5}
              />
            );
          })}
        </svg>
      </div>
    </div>
  );
}
