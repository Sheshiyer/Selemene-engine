import GenericEngineView from "./GenericEngineView";

interface VedicClockProps {
  result: Record<string, unknown>;
}

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

function str(v: unknown): string { return v == null ? "—" : String(v); }

export default function VedicClock({ result }: VedicClockProps) {
  const tcm = result.tcm_organ_clock as Record<string, unknown> | undefined;
  const ayurvedic = result.ayurvedic_timing as Record<string, unknown> | undefined;
  const current = tcm?.current_organ as Record<string, unknown> | undefined;
  const peaks = tcm?.peak_organs as Record<string, unknown>[] | undefined;

  if (!tcm && !ayurvedic) return <GenericEngineView result={result} />;

  return (
    <div style={{ display: "flex", flexDirection: "column", gap: "1rem" }}>
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
