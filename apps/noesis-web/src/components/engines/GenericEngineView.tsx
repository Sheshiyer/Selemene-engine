const styles = {
  container: { display: "flex", flexDirection: "column" as const, gap: "0.5rem" },
  row: {
    display: "flex",
    justifyContent: "space-between",
    alignItems: "baseline",
    padding: "0.375rem 0",
    borderBottom: "1px solid var(--line)",
  },
  key: {
    fontSize: "0.8rem",
    color: "var(--text-muted)",
    textTransform: "capitalize" as const,
  },
  value: {
    fontSize: "0.875rem",
    color: "var(--text)",
    fontFamily: "var(--font-mono)",
    textAlign: "right" as const,
    maxWidth: "60%",
    wordBreak: "break-word" as const,
  },
  nested: {
    paddingLeft: "1rem",
    borderLeft: "2px solid var(--line)",
    marginTop: "0.25rem",
    display: "flex",
    flexDirection: "column" as const,
    gap: "0.25rem",
  },
  label: {
    fontSize: "0.75rem",
    color: "var(--gold)",
    fontWeight: 600,
    textTransform: "uppercase" as const,
    letterSpacing: "0.05em",
    marginTop: "0.5rem",
  },
};

function formatKey(key: string): string {
  return key.replace(/[_-]/g, " ").replace(/\b\w/g, (c) => c.toUpperCase());
}

function renderValue(val: unknown, depth: number): React.ReactNode {
  if (val === null || val === undefined) return <span style={{ color: "var(--text-dim)" }}>—</span>;
  if (typeof val === "boolean") return val ? "Yes" : "No";
  if (typeof val === "number") return String(val);
  if (typeof val === "string") return val;

  if (Array.isArray(val)) {
    if (val.length === 0) return <span style={{ color: "var(--text-dim)" }}>—</span>;
    if (val.every((v) => typeof v === "string" || typeof v === "number")) {
      return val.join(", ");
    }
    return (
      <div style={styles.nested}>
        {val.map((item, i) => (
          <div key={i}>{renderValue(item, depth + 1)}</div>
        ))}
      </div>
    );
  }

  if (typeof val === "object" && depth < 4) {
    const obj = val as Record<string, unknown>;
    return (
      <div style={styles.nested}>
        {Object.entries(obj).map(([k, v]) => (
          <div key={k} style={styles.row}>
            <span style={styles.key}>{formatKey(k)}</span>
            <span style={styles.value}>{renderValue(v, depth + 1)}</span>
          </div>
        ))}
      </div>
    );
  }

  return JSON.stringify(val);
}

interface GenericEngineViewProps {
  result: Record<string, unknown>;
}

export default function GenericEngineView({ result }: GenericEngineViewProps) {
  const entries = Object.entries(result);

  if (entries.length === 0) {
    return <p style={{ color: "var(--text-dim)", fontSize: "0.875rem" }}>No data available.</p>;
  }

  return (
    <div style={styles.container}>
      {entries.map(([key, value]) => {
        if (typeof value === "object" && value !== null && !Array.isArray(value)) {
          return (
            <div key={key}>
              <div style={styles.label}>{formatKey(key)}</div>
              {renderValue(value, 1)}
            </div>
          );
        }
        return (
          <div key={key} style={styles.row}>
            <span style={styles.key}>{formatKey(key)}</span>
            <span style={styles.value}>{renderValue(value, 0)}</span>
          </div>
        );
      })}
    </div>
  );
}
