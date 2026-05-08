import GenericEngineView from "./GenericEngineView";

interface FaceReadingProps {
  result: Record<string, unknown>;
}

const s = {
  grid: { display: "grid", gridTemplateColumns: "repeat(auto-fit, minmax(200px, 1fr))", gap: "0.75rem" },
  cell: { background: "var(--field)", borderRadius: "var(--radius)", padding: "0.75rem", display: "flex", flexDirection: "column" as const, gap: "0.25rem" },
  label: { fontSize: "0.7rem", color: "var(--text-muted)", textTransform: "uppercase" as const, letterSpacing: "0.06em", fontWeight: 600 },
  value: { fontSize: "1rem", fontWeight: 600, color: "var(--gold)" },
  sub: { fontSize: "0.8rem", color: "var(--text-muted)" },
  section: { display: "flex", flexDirection: "column" as const, gap: "0.5rem" },
  sectionTitle: { fontSize: "0.75rem", color: "var(--text-muted)", textTransform: "uppercase" as const, fontWeight: 600, letterSpacing: "0.08em" },
  featureRow: { display: "flex", justifyContent: "space-between" as const, padding: "0.5rem 0.75rem", background: "var(--field)", borderRadius: "var(--radius)", fontSize: "0.85rem" },
};

function str(v: unknown): string { return v == null ? "—" : String(v); }

export default function FaceReading({ result }: FaceReadingProps) {
  const analysis = result.analysis as Record<string, unknown> | undefined;
  const constitution = (analysis?.constitution ?? result.constitution) as Record<string, unknown> | undefined;
  const features = (analysis?.facial_features ?? result.facial_features) as Record<string, unknown> | undefined;
  const element = (analysis?.element ?? result.element) as Record<string, unknown> | string | undefined;

  if (!analysis && !constitution && !features) return <GenericEngineView result={result} />;

  const primaryDosha = str(constitution?.primary_dosha ?? constitution?.dosha);
  const secondaryDosha = str(constitution?.secondary_dosha);
  const faceShape = str(features?.face_shape ?? result.face_shape);
  const eyeType = str(features?.eyes ?? result.eye_type);
  const vitality = result.vitality_score ?? analysis?.vitality_score;
  const elementName = typeof element === "string" ? element : str((element as Record<string, unknown> | undefined)?.name);

  const featureMap: [string, unknown][] = features
    ? Object.entries(features).filter(([, v]) => v != null)
    : [];

  return (
    <div style={{ display: "flex", flexDirection: "column", gap: "1rem" }}>
      <div style={s.grid}>
        {constitution != null && (
          <div style={s.cell}>
            <span style={s.label}>Primary Dosha</span>
            <span style={s.value}>{primaryDosha}</span>
            {secondaryDosha !== "—" && <span style={s.sub}>Secondary: {secondaryDosha}</span>}
          </div>
        )}
        {element != null && (
          <div style={s.cell}>
            <span style={s.label}>Element</span>
            <span style={s.value}>{elementName}</span>
          </div>
        )}
        {vitality != null && (
          <div style={s.cell}>
            <span style={s.label}>Vitality</span>
            <span style={s.value}>{str(vitality)}</span>
          </div>
        )}
        {faceShape !== "—" && (
          <div style={s.cell}>
            <span style={s.label}>Face Shape</span>
            <span style={s.value}>{faceShape}</span>
          </div>
        )}
        {eyeType !== "—" && (
          <div style={s.cell}>
            <span style={s.label}>Eyes</span>
            <span style={s.value}>{eyeType}</span>
          </div>
        )}
      </div>

      {featureMap.length > 0 && (
        <div style={s.section}>
          <span style={s.sectionTitle}>Facial Features</span>
          {featureMap.slice(0, 8).map(([k, v]) => (
            <div key={k} style={s.featureRow}>
              <span style={{ color: "var(--text-muted)", textTransform: "capitalize" as const }}>{k.replace(/_/g, " ")}</span>
              <span style={{ color: "var(--text)", fontWeight: 500 }}>{str(v)}</span>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
