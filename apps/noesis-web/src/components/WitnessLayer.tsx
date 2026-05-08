import type { WitnessLayer as WitnessLayerType } from "@/lib/api";

const styles = {
  container: {
    padding: "1.5rem",
    background:
      "linear-gradient(135deg, var(--gold-soft), var(--bg-elevated))",
    border: "1px solid var(--line-gold)",
    borderRadius: "var(--radius)",
    display: "flex",
    flexDirection: "column" as const,
    gap: "1rem",
  },
  heading: {
    fontFamily: "'Exo 2', sans-serif",
    fontSize: "1.25rem",
    fontWeight: 700,
    color: "var(--gold)",
  },
  summary: {
    fontSize: "0.95rem",
    lineHeight: 1.65,
    color: "var(--text)",
  },
  section: {
    display: "flex",
    flexDirection: "column" as const,
    gap: "0.375rem",
  },
  sectionTitle: {
    fontSize: "0.75rem",
    textTransform: "uppercase" as const,
    letterSpacing: "0.08em",
    color: "var(--text-muted)",
    fontWeight: 600,
  },
  list: {
    listStyle: "none",
    padding: 0,
    display: "flex",
    flexDirection: "column" as const,
    gap: "0.25rem",
  },
  listItem: {
    fontSize: "0.875rem",
    color: "var(--text)",
    paddingLeft: "0.75rem",
    position: "relative" as const,
  },
  question: {
    fontSize: "1.05rem",
    fontStyle: "italic",
    color: "var(--gold)",
    lineHeight: 1.5,
    padding: "0.75rem 1rem",
    borderLeft: "3px solid var(--gold)",
    background: "var(--gold-soft)",
    borderRadius: "0 var(--radius) var(--radius) 0",
  },
  practice: {
    fontSize: "0.9rem",
    color: "var(--emerald)",
    padding: "0.75rem 1rem",
    background: "var(--emerald-soft)",
    borderRadius: "var(--radius)",
    lineHeight: 1.5,
  },
};

interface WitnessLayerProps {
  data: WitnessLayerType;
}

export default function WitnessLayer({ data }: WitnessLayerProps) {
  return (
    <div style={styles.container}>
      <h2 style={styles.heading}>{data.title || "Witness Layer"}</h2>
      {data.summary && <p style={styles.summary}>{data.summary}</p>}

      {data.convergences && data.convergences.length > 0 && (
        <div style={styles.section}>
          <span style={styles.sectionTitle}>Convergences</span>
          <ul style={styles.list}>
            {data.convergences.map((c, i) => (
              <li key={i} style={styles.listItem}>
                ◈ {c}
              </li>
            ))}
          </ul>
        </div>
      )}

      {data.frictions && data.frictions.length > 0 && (
        <div style={styles.section}>
          <span style={styles.sectionTitle}>Frictions</span>
          <ul style={styles.list}>
            {data.frictions.map((f, i) => (
              <li key={i} style={styles.listItem}>
                ◇ {f}
              </li>
            ))}
          </ul>
        </div>
      )}

      {data.question && <p style={styles.question}>{data.question}</p>}
      {data.practice && <p style={styles.practice}>Practice: {data.practice}</p>}
    </div>
  );
}
