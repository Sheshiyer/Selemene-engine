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

  return (
    <div style={{ display: "flex", flexDirection: "column", gap: "1rem" }}>
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
