import GenericEngineView from "./GenericEngineView";

interface NadabrahmanProps {
  result: Record<string, unknown>;
}

const s = {
  grid: { display: "grid", gridTemplateColumns: "repeat(auto-fit, minmax(180px, 1fr))", gap: "0.75rem" },
  cell: { background: "var(--field)", borderRadius: "var(--radius)", padding: "0.75rem", display: "flex", flexDirection: "column" as const, gap: "0.25rem" },
  label: { fontSize: "0.7rem", color: "var(--text-muted)", textTransform: "uppercase" as const, letterSpacing: "0.06em", fontWeight: 600 },
  value: { fontSize: "1rem", fontWeight: 600, color: "var(--gold)" },
  sub: { fontSize: "0.8rem", color: "var(--text-muted)", fontFamily: "'IBM Plex Mono', monospace" },
  mantraBox: { padding: "1rem", background: "var(--gold-soft)", border: "1px solid var(--line-gold)", borderRadius: "var(--radius)", textAlign: "center" as const },
  mantra: { fontSize: "1.5rem", fontWeight: 700, color: "var(--gold)", fontFamily: "'Exo 2', sans-serif", letterSpacing: "0.1em" },
  tag: { display: "inline-block", padding: "0.2rem 0.5rem", borderRadius: 4, background: "var(--field)", color: "var(--text)", fontSize: "0.75rem", marginRight: "0.25rem", marginBottom: "0.25rem", border: "1px solid var(--line)" },
};

function str(v: unknown): string { return v == null ? "—" : String(v); }

export default function Nadabrahman({ result }: NadabrahmanProps) {
  const chakraFreq = result.chakra_frequency as Record<string, unknown> | undefined;
  const dosha = result.dosha_recommendation as string | undefined;
  const rasa = result.rasa_mapping as Record<string, unknown> | undefined;
  const recommendations = result.recommendations as string[] | undefined;
  const mantra = (chakraFreq?.mantra as string | undefined) ?? (result.mantra as string | undefined);
  const hz = chakraFreq?.frequency_hz ?? result.frequency_hz;
  const note = chakraFreq?.note ?? result.note;

  if (!chakraFreq && !mantra && !dosha) return <GenericEngineView result={result} />;

  return (
    <div style={{ display: "flex", flexDirection: "column", gap: "1rem" }}>
      {mantra != null && (
        <div style={s.mantraBox}>
          <div style={s.mantra}>{mantra}</div>
          {note != null && <div style={{ fontSize: "0.8rem", color: "var(--text-muted)", marginTop: "0.5rem" }}>Note: {str(note)}{hz != null ? ` · ${str(hz)} Hz` : ""}</div>}
        </div>
      )}

      <div style={s.grid}>
        {chakraFreq?.chakra != null && (
          <div style={s.cell}>
            <span style={s.label}>Chakra</span>
            <span style={s.value}>{str(chakraFreq.chakra)}</span>
            {chakraFreq.element != null && <span style={s.sub}>{str(chakraFreq.element)}</span>}
          </div>
        )}
        {dosha != null && (
          <div style={s.cell}>
            <span style={s.label}>Dosha</span>
            <span style={s.value}>{dosha}</span>
          </div>
        )}
        {rasa != null && (
          <div style={s.cell}>
            <span style={s.label}>Rasa</span>
            <span style={s.value}>{str(rasa.rasa ?? rasa.name)}</span>
            {rasa.emotion != null && <span style={s.sub}>{str(rasa.emotion)}</span>}
          </div>
        )}
      </div>

      {recommendations != null && recommendations.length > 0 && (
        <div style={{ display: "flex", flexDirection: "column" as const, gap: "0.5rem" }}>
          <span style={s.label}>Sound Practices</span>
          <div style={{ display: "flex", flexWrap: "wrap" as const }}>
            {recommendations.slice(0, 6).map((r, i) => (
              <span key={i} style={s.tag}>{r}</span>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}
