const styles = {
  container: { display: "flex", flexDirection: "column" as const, gap: "0.75rem" },
  hexHeader: {
    display: "flex",
    alignItems: "baseline",
    gap: "0.75rem",
    padding: "0.75rem 1rem",
    background: "var(--field)",
    borderRadius: "var(--radius)",
  },
  hexNumber: {
    fontSize: "2rem",
    fontWeight: 800,
    fontFamily: "var(--font-display)",
    color: "var(--gold)",
  },
  hexName: {
    fontSize: "1.1rem",
    fontWeight: 700,
    color: "var(--text)",
  },
  section: {
    display: "flex",
    flexDirection: "column" as const,
    gap: "0.25rem",
    paddingLeft: "0.5rem",
    borderLeft: "2px solid var(--line-gold)",
  },
  sectionTitle: {
    fontSize: "0.75rem",
    color: "var(--text-muted)",
    textTransform: "uppercase" as const,
    letterSpacing: "0.06em",
    fontWeight: 600,
  },
  text: {
    fontSize: "0.9rem",
    color: "var(--text)",
    lineHeight: 1.5,
  },
  changingLine: {
    fontSize: "0.85rem",
    color: "var(--gold)",
    padding: "0.25rem 0",
  },
};

function str(v: unknown): string {
  if (v === null || v === undefined) return "—";
  return String(v);
}

function arr(v: unknown): unknown[] {
  return Array.isArray(v) ? v : [];
}

interface IChingProps {
  result: Record<string, unknown>;
}

export default function IChing({ result }: IChingProps) {
  const number = result.hexagram_number ?? result.number ?? result.hexagram;
  const name = result.hexagram_name ?? result.name;
  const judgment = result.judgment ?? result.judgement;
  const image = result.image;
  const changing = arr(result.changing_lines);

  return (
    <div style={styles.container}>
      <div style={styles.hexHeader}>
        {number != null && <span style={styles.hexNumber}>#{str(number)}</span>}
        {name != null && <span style={styles.hexName}>{str(name)}</span>}
      </div>

      {judgment != null && (
        <div style={styles.section}>
          <span style={styles.sectionTitle}>Judgment</span>
          <p style={styles.text}>{str(judgment)}</p>
        </div>
      )}

      {image != null && (
        <div style={styles.section}>
          <span style={styles.sectionTitle}>Image</span>
          <p style={styles.text}>{str(image)}</p>
        </div>
      )}

      {changing.length > 0 && (
        <div style={styles.section}>
          <span style={styles.sectionTitle}>Changing Lines</span>
          {changing.map((line, i) => (
            <p key={i} style={styles.changingLine}>
              Line {i + 1}: {str(line)}
            </p>
          ))}
        </div>
      )}
    </div>
  );
}
