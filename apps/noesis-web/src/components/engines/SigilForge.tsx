import GenericEngineView from "./GenericEngineView";

interface SigilForgeProps {
  result: Record<string, unknown>;
}

const s = {
  grid: { display: "grid", gridTemplateColumns: "repeat(auto-fit, minmax(180px, 1fr))", gap: "0.75rem" },
  cell: { background: "var(--field)", borderRadius: "var(--radius)", padding: "0.75rem", display: "flex", flexDirection: "column" as const, gap: "0.25rem" },
  label: { fontSize: "0.7rem", color: "var(--text-muted)", textTransform: "uppercase" as const, letterSpacing: "0.06em", fontWeight: 600 },
  value: { fontSize: "1rem", fontWeight: 600, color: "var(--gold)" },
  tag: { display: "inline-block", padding: "0.2rem 0.5rem", borderRadius: 4, background: "var(--gold-soft)", color: "var(--gold)", fontSize: "0.75rem", marginRight: "0.25rem", marginBottom: "0.25rem", border: "1px solid var(--line-gold)" },
  intentionBox: { padding: "1rem", background: "var(--field)", borderRadius: "var(--radius)", borderLeft: "3px solid var(--gold)", fontSize: "0.9rem", color: "var(--text)", lineHeight: 1.5, fontStyle: "italic" },
  svgBox: { padding: "1rem", background: "var(--bg-elevated)", border: "1px solid var(--line)", borderRadius: "var(--radius)", display: "flex", justifyContent: "center" as const },
};

function str(v: unknown): string { return v == null ? "—" : String(v); }

export default function SigilForge({ result }: SigilForgeProps) {
  const sigil = result.sigil as Record<string, unknown> | undefined;
  const elements = (result.activated_elements ?? result.elements) as string[] | undefined;
  const intention = (result.intention_field ?? result.intention) as string | undefined;
  const svgPreview = (sigil?.svg_preview ?? result.svg_preview) as string | undefined;

  if (!sigil && !elements && !intention) return <GenericEngineView result={result} />;

  const name = sigil?.name ?? result.sigil_name;
  const symbolSet = sigil?.symbol_set ?? result.symbol_set;
  const numerologicalBase = sigil?.numerological_base ?? result.numerological_base;
  const activationPhrase = (sigil?.activation_phrase ?? result.activation_phrase) as string | undefined;

  return (
    <div style={{ display: "flex", flexDirection: "column", gap: "1rem" }}>
      {svgPreview != null && (
        <div style={s.svgBox} dangerouslySetInnerHTML={{ __html: svgPreview }} />
      )}

      <div style={s.grid}>
        {name != null && (
          <div style={s.cell}>
            <span style={s.label}>Sigil</span>
            <span style={s.value}>{str(name)}</span>
            {symbolSet != null && <span style={{ fontSize: "0.8rem", color: "var(--text-muted)" }}>{str(symbolSet)}</span>}
          </div>
        )}
        {numerologicalBase != null && (
          <div style={s.cell}>
            <span style={s.label}>Num. Base</span>
            <span style={s.value}>{str(numerologicalBase)}</span>
          </div>
        )}
      </div>

      {intention != null && (
        <div style={s.intentionBox}>
          <div style={{ fontSize: "0.7rem", color: "var(--gold)", fontWeight: 700, marginBottom: "0.25rem", textTransform: "uppercase" as const, letterSpacing: "0.06em", fontStyle: "normal" }}>Intention Field</div>
          {intention}
        </div>
      )}

      {elements != null && elements.length > 0 && (
        <div>
          <div style={{ ...s.label, marginBottom: "0.5rem" }}>Activated Elements</div>
          <div style={{ display: "flex", flexWrap: "wrap" as const }}>
            {elements.map((e, i) => <span key={i} style={s.tag}>{e}</span>)}
          </div>
        </div>
      )}

      {activationPhrase != null && (
        <div style={{ fontSize: "0.8rem", color: "var(--text-muted)", fontStyle: "italic", textAlign: "center" as const, padding: "0.5rem" }}>
          &ldquo;{activationPhrase}&rdquo;
        </div>
      )}
    </div>
  );
}
