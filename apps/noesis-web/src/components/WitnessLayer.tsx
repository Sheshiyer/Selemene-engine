import type { WitnessLayer as WitnessLayerType } from "@/lib/api";

const styles = {
  container: {
    // Kha Arc gradient panel — observer field aesthetic
    padding: "1.75rem",
    background: "linear-gradient(135deg, #070B1D 0%, rgba(45,0,80,0.35) 55%, rgba(11,80,251,0.08) 100%)",
    border: "1px solid var(--line-strong)",
    borderRadius: "var(--r-md)",
    display: "flex",
    flexDirection: "column" as const,
    gap: "1.25rem",
    maxWidth: 680,
    // Inset glass edge
    boxShadow: "var(--inset-glow), var(--shadow-md)",
    animation: "slideUp 0.4s var(--ease-out-expo) both",
  },
  headerRow: {
    display: "flex",
    alignItems: "baseline",
    justifyContent: "space-between",
    gap: "1rem",
  },
  heading: {
    fontFamily: "var(--font-display)",
    fontSize: "1.35rem",
    fontWeight: 700,
    color: "var(--signal)",
    letterSpacing: "0.04em",
  },
  calibrationBadge: {
    fontSize: "0.68rem",
    fontFamily: "var(--font-mono)",
    color: "var(--c-emerald)",
    letterSpacing: "0.08em",
    padding: "0.15rem 0.5rem",
    border: "1px solid rgba(16,181,167,0.3)",
    borderRadius: "var(--r-xs)",
    whiteSpace: "nowrap" as const,
  },
  summary: {
    fontSize: "0.97rem",
    lineHeight: 1.75,
    color: "var(--text)",
    fontFamily: "var(--font-body)",
    maxWidth: 620,
  },
  section: {
    display: "flex",
    flexDirection: "column" as const,
    gap: "0.5rem",
  },
  sectionTitle: {
    fontSize: "0.7rem",
    textTransform: "uppercase" as const,
    letterSpacing: "0.1em",
    color: "var(--muted)",
    fontWeight: 600,
    fontFamily: "var(--font-mono)",
  },
  list: {
    listStyle: "none",
    padding: 0,
    display: "flex",
    flexDirection: "column" as const,
    gap: "0.375rem",
  },
  listItem: {
    fontSize: "0.9rem",
    color: "var(--text-2)",
    paddingLeft: "1rem",
    position: "relative" as const,
    lineHeight: 1.6,
    fontFamily: "var(--font-body)",
  },
  listBullet: {
    position: "absolute" as const,
    left: 0,
    color: "var(--c-emerald)",
  },
  // Witness Violet blockquote for questions — the philosophical heart
  question: {
    fontSize: "1.05rem",
    fontStyle: "italic",
    color: "var(--text)",
    lineHeight: 1.65,
    padding: "0.875rem 1.125rem",
    borderLeft: "2px solid var(--c-violet)",
    background: "rgba(45,0,80,0.15)",
    borderRadius: "0 var(--r-sm) var(--r-sm) 0",
    fontFamily: "var(--font-body)",
    maxWidth: 620,
  },
  // Emerald treatment for practices
  practice: {
    fontSize: "0.9rem",
    color: "var(--c-emerald)",
    padding: "0.75rem 1rem",
    background: "rgba(16,181,167,0.08)",
    borderRadius: "var(--r-sm)",
    border: "1px solid rgba(16,181,167,0.2)",
    lineHeight: 1.6,
    fontFamily: "var(--font-body)",
  },
  practiceLabel: {
    fontFamily: "var(--font-mono)",
    fontSize: "0.65rem",
    textTransform: "uppercase" as const,
    letterSpacing: "0.1em",
    color: "var(--c-emerald)",
    opacity: 0.7,
    marginBottom: "0.25rem",
    display: "block",
  },
};

interface WitnessLayerProps {
  data: WitnessLayerType;
}

export default function WitnessLayer({ data }: WitnessLayerProps) {
  return (
    <div style={styles.container}>
      <div style={styles.headerRow}>
        <h2 style={styles.heading}>{data.title || "Witness Synthesis"}</h2>
        {(data as unknown as Record<string, unknown>).calibration_level != null && (
          <span style={styles.calibrationBadge}>
            LEVEL {String((data as unknown as Record<string, unknown>).calibration_level)}
          </span>
        )}
      </div>

      {data.summary && <p style={styles.summary}>{data.summary}</p>}

      {data.convergences && data.convergences.length > 0 && (
        <div style={styles.section}>
          <span style={styles.sectionTitle}>Convergences</span>
          <ul style={styles.list}>
            {data.convergences.map((c, i) => (
              <li key={i} style={styles.listItem}>
                <span style={styles.listBullet}>◈</span>
                {c}
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
                <span style={styles.listBullet}>◇</span>
                {f}
              </li>
            ))}
          </ul>
        </div>
      )}

      {/* Violet blockquote — the witness question, the philosophical core */}
      {data.question && (
        <blockquote style={styles.question}>{data.question}</blockquote>
      )}

      {data.practice && (
        <div style={styles.practice}>
          <span style={styles.practiceLabel}>Practice</span>
          {data.practice}
        </div>
      )}
    </div>
  );
}
