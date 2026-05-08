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
    fontFamily: "'IBM Plex Mono', monospace",
  },
  quality: {
    display: "inline-block",
    padding: "0.15rem 0.5rem",
    borderRadius: 4,
    fontSize: "0.7rem",
    fontWeight: 600,
    textTransform: "uppercase" as const,
  },
};

function str(v: unknown): string {
  if (v === null || v === undefined) return "—";
  if (typeof v === "object") return JSON.stringify(v);
  return String(v);
}

function obj(v: unknown): Record<string, unknown> {
  if (v && typeof v === "object" && !Array.isArray(v)) return v as Record<string, unknown>;
  return {};
}

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

  return (
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
  );
}
