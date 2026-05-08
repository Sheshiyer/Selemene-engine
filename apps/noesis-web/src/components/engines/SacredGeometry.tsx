import GenericEngineView from "./GenericEngineView";

interface SacredGeometryProps {
  result: Record<string, unknown>;
}

const s = {
  grid: { display: "grid", gridTemplateColumns: "repeat(auto-fit, minmax(200px, 1fr))", gap: "0.75rem" },
  cell: { background: "var(--field)", borderRadius: "var(--radius)", padding: "0.75rem", display: "flex", flexDirection: "column" as const, gap: "0.25rem" },
  label: { fontSize: "0.7rem", color: "var(--text-muted)", textTransform: "uppercase" as const, letterSpacing: "0.06em", fontWeight: 600 },
  value: { fontSize: "1rem", fontWeight: 600, color: "var(--gold)" },
  sub: { fontSize: "0.8rem", color: "var(--text-muted)" },
  desc: { fontSize: "0.875rem", color: "var(--text)", lineHeight: 1.65, padding: "0.75rem", background: "var(--field)", borderRadius: "var(--radius)" },
  meditation: { padding: "0.75rem 1rem", background: "var(--gold-soft)", border: "1px solid var(--line-gold)", borderRadius: "var(--radius)", fontSize: "0.875rem", color: "var(--text)", lineHeight: 1.65, fontStyle: "italic" },
};

function str(v: unknown): string { return v == null ? "—" : String(v); }

export default function SacredGeometry({ result }: SacredGeometryProps) {
  const form = result.form as Record<string, unknown> | undefined;
  const meditation = result.meditation_guidance as string | undefined;

  if (!form && !result.form_id && !result.primary_form) return <GenericEngineView result={result} />;

  const name = form?.name ?? result.form_name ?? result.primary_form;
  const category = form?.category ?? result.category;
  const elements = (form?.elements ?? result.elements) as string[] | undefined;
  const symbolism = form?.symbolism ?? result.symbolism;
  const ratio = (form?.golden_ratio_present ?? result.golden_ratio) as boolean | undefined;
  const description = form?.description ?? result.description;
  const intention = result.intention as string | undefined;

  return (
    <div style={{ display: "flex", flexDirection: "column", gap: "1rem" }}>
      <div style={s.grid}>
        <div style={s.cell}>
          <span style={s.label}>Form</span>
          <span style={s.value}>{str(name)}</span>
          {category != null && <span style={s.sub}>{str(category)}</span>}
        </div>
        {symbolism != null && (
          <div style={s.cell}>
            <span style={s.label}>Symbolism</span>
            <span style={s.value}>{str(symbolism)}</span>
          </div>
        )}
        {ratio != null && (
          <div style={s.cell}>
            <span style={s.label}>Golden Ratio</span>
            <span style={s.value}>{ratio ? "Present ◈" : "Absent"}</span>
          </div>
        )}
        {elements != null && elements.length > 0 && (
          <div style={s.cell}>
            <span style={s.label}>Elements</span>
            <span style={{ fontSize: "0.85rem", color: "var(--text)" }}>{elements.join(", ")}</span>
          </div>
        )}
      </div>

      {intention != null && (
        <div style={{ fontSize: "0.8rem", color: "var(--text-muted)", fontStyle: "italic" }}>
          Held intention: &ldquo;{intention}&rdquo;
        </div>
      )}

      {description != null && <div style={s.desc}>{str(description)}</div>}

      {meditation != null && (
        <div style={s.meditation}>
          <div style={{ fontSize: "0.7rem", color: "var(--gold)", fontWeight: 700, marginBottom: "0.5rem", textTransform: "uppercase" as const, letterSpacing: "0.06em", fontStyle: "normal" }}>Meditation Guidance</div>
          {meditation}
        </div>
      )}
    </div>
  );
}
