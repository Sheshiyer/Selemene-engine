import GenericEngineView from "./GenericEngineView";

interface EnneagramProps {
  result: Record<string, unknown>;
}

const s = {
  grid: { display: "grid", gridTemplateColumns: "repeat(auto-fit, minmax(180px, 1fr))", gap: "0.75rem" },
  cell: { background: "var(--field)", borderRadius: "var(--radius)", padding: "0.75rem", display: "flex", flexDirection: "column" as const, gap: "0.25rem" },
  label: { fontSize: "0.7rem", color: "var(--text-muted)", textTransform: "uppercase" as const, letterSpacing: "0.06em", fontWeight: 600 },
  value: { fontSize: "1rem", fontWeight: 600, color: "var(--gold)" },
  typeNum: { fontSize: "3rem", fontWeight: 800, color: "var(--gold)", fontFamily: "var(--font-display)", lineHeight: 1 },
  sub: { fontSize: "0.8rem", color: "var(--text-muted)" },
  fearDesire: { padding: "0.75rem 1rem", background: "var(--field)", borderRadius: "var(--radius)", fontSize: "0.875rem", color: "var(--text)", lineHeight: 1.5, borderLeft: "3px solid var(--gold)" },
};

function str(v: unknown): string { return v == null ? "—" : String(v); }

/* ── Enneagram SVG geometry ─────────────────────────────────────── */

const CX = 110;
const CY = 110;
const R = 90;
const DEG = Math.PI / 180;

/** Type 9 sits at top (−90°), then clockwise in order 9,1,2,…,8 */
function typeAngle(n: number): number {
  const idx = n === 9 ? 0 : n;
  return (-90 + idx * 40) * DEG;
}

interface Point { x: number; y: number }

function typePoint(n: number): Point {
  const a = typeAngle(n);
  return { x: CX + R * Math.cos(a), y: CY + R * Math.sin(a) };
}

const ALL_POINTS: readonly Point[] = [1, 2, 3, 4, 5, 6, 7, 8, 9].map(typePoint);

function polyline(types: readonly number[]): string {
  return types.map((n) => { const p = typePoint(n); return `${p.x},${p.y}`; }).join(" ");
}

/* Hexad: 1→4→2→8→5→7→1 */
const HEXAD: readonly number[] = [1, 4, 2, 8, 5, 7, 1];
/* Triangle: 3→6→9→3 */
const TRIANGLE: readonly number[] = [3, 6, 9, 3];

const COL = {
  gold: "rgba(197,160,23,",
  teal: "rgba(16,181,167,",
  indigo: "rgba(11,80,251,",
};

interface DiagramProps {
  activeType: number | null;
  wingType: number | null;
}

function EnneagramDiagram({ activeType, wingType }: DiagramProps) {
  return (
    <svg
      viewBox="0 0 220 220"
      xmlns="http://www.w3.org/2000/svg"
      style={{ width: "100%", maxWidth: 220, height: "auto", display: "block", margin: "0 auto" }}
      role="img"
      aria-label={`Enneagram diagram${activeType ? `, type ${activeType} highlighted` : ""}`}
    >
      <defs>
        <filter id="enn-glow">
          <feGaussianBlur stdDeviation="3" result="blur" />
          <feMerge>
            <feMergeNode in="blur" />
            <feMergeNode in="SourceGraphic" />
          </feMerge>
        </filter>
      </defs>

      {/* outer circle */}
      <circle cx={CX} cy={CY} r={R} fill="none" stroke={`${COL.gold}0.2)`} strokeWidth={1.2} />

      {/* inner hexad lines */}
      <polyline
        points={polyline(HEXAD)}
        fill="none"
        stroke={`${COL.gold}0.25)`}
        strokeWidth={0.8}
        strokeLinejoin="round"
      />

      {/* inner triangle lines */}
      <polyline
        points={polyline(TRIANGLE)}
        fill="none"
        stroke={`${COL.gold}0.25)`}
        strokeWidth={0.8}
        strokeLinejoin="round"
      />

      {/* points + labels */}
      {[1, 2, 3, 4, 5, 6, 7, 8, 9].map((n) => {
        const p = ALL_POINTS[n - 1];
        const isActive = n === activeType;
        const isWing = n === wingType;

        let circleFill: string;
        let circleStroke: string;
        let radius: number;
        let textFill: string;
        let filter: string | undefined;

        if (isActive) {
          circleFill = `${COL.teal}0.6)`;
          circleStroke = `${COL.teal}1.0)`;
          radius = 10;
          textFill = `${COL.teal}1.0)`;
          filter = "url(#enn-glow)";
        } else if (isWing) {
          circleFill = `${COL.indigo}0.35)`;
          circleStroke = `${COL.indigo}0.7)`;
          radius = 9;
          textFill = `${COL.indigo}0.9)`;
          filter = undefined;
        } else {
          circleFill = `${COL.gold}0.15)`;
          circleStroke = `${COL.gold}0.4)`;
          radius = 8;
          textFill = `${COL.gold}0.6)`;
          filter = undefined;
        }

        return (
          <g key={n}>
            <circle
              cx={p.x}
              cy={p.y}
              r={radius}
              fill={circleFill}
              stroke={circleStroke}
              strokeWidth={isActive ? 1.6 : 1}
              filter={filter}
            />
            <text
              x={p.x}
              y={p.y}
              dy="0.36em"
              textAnchor="middle"
              fontSize={isActive ? 11 : 10}
              fontFamily="monospace"
              fontWeight={isActive ? 700 : 500}
              fill={textFill}
              style={{ pointerEvents: "none", userSelect: "none" }}
            >
              {n}
            </text>
          </g>
        );
      })}
    </svg>
  );
}

/* ── Main component ─────────────────────────────────────────────── */

export default function Enneagram({ result }: EnneagramProps) {
  const assessment = result.assessment as Record<string, unknown> | undefined;
  const typeAnalysis = result.typeAnalysis as Record<string, unknown> | undefined;
  const primary = (assessment?.primaryType ?? typeAnalysis?.type) as Record<string, unknown> | undefined;
  const wing = assessment?.wing as Record<string, unknown> | undefined;
  const confidence = assessment?.confidence as number | undefined;

  if (!primary && !assessment && !typeAnalysis) return <GenericEngineView result={result} />;

  const typeNum = primary?.number ?? primary?.type;
  const typeName = primary?.name ?? primary?.archetype;
  const center = primary?.center;
  const triad = primary?.triad;
  const coreDesire = primary?.coreDesire ?? primary?.core_desire;
  const coreFear = primary?.coreFear ?? primary?.core_fear;

  const activeN = typeof typeNum === "number" && typeNum >= 1 && typeNum <= 9 ? typeNum : null;
  const wingN = typeof wing?.number === "number" && (wing.number as number) >= 1 && (wing.number as number) <= 9 ? (wing.number as number) : null;

  return (
    <div style={{ display: "flex", flexDirection: "column", gap: "1rem" }}>
      {/* SVG diagram above the data cells */}
      <div style={{ display: "flex", justifyContent: "center", padding: "0.5rem 0" }}>
        <EnneagramDiagram activeType={activeN} wingType={wingN} />
      </div>

      <div style={s.grid}>
        <div style={{ ...s.cell, alignItems: "center" as const, justifyContent: "center" as const }}>
          <span style={s.typeNum}>{str(typeNum)}</span>
          <span style={{ fontSize: "0.75rem", color: "var(--text-muted)", textTransform: "uppercase" as const, letterSpacing: "0.08em" }}>Type</span>
        </div>
        <div style={s.cell}>
          <span style={s.label}>Archetype</span>
          <span style={s.value}>{str(typeName)}</span>
          {center != null && <span style={s.sub}>{str(center)} Center</span>}
        </div>
        {wing != null && (
          <div style={s.cell}>
            <span style={s.label}>Wing</span>
            <span style={s.value}>{str(wing.number)} · {str(wing.name)}</span>
          </div>
        )}
        {triad != null && (
          <div style={s.cell}>
            <span style={s.label}>Triad</span>
            <span style={s.value}>{str(triad)}</span>
          </div>
        )}
        {confidence != null && (
          <div style={s.cell}>
            <span style={s.label}>Confidence</span>
            <span style={s.value}>{Math.round(confidence * 100)}%</span>
          </div>
        )}
      </div>

      {coreDesire != null && (
        <div style={s.fearDesire}>
          <div style={{ fontSize: "0.7rem", color: "var(--emerald)", fontWeight: 700, marginBottom: "0.25rem", textTransform: "uppercase" as const, letterSpacing: "0.06em" }}>Core Desire</div>
          {str(coreDesire)}
        </div>
      )}
      {coreFear != null && (
        <div style={{ ...s.fearDesire, borderLeftColor: "var(--danger)" }}>
          <div style={{ fontSize: "0.7rem", color: "var(--danger)", fontWeight: 700, marginBottom: "0.25rem", textTransform: "uppercase" as const, letterSpacing: "0.06em" }}>Core Fear</div>
          {str(coreFear)}
        </div>
      )}
    </div>
  );
}
