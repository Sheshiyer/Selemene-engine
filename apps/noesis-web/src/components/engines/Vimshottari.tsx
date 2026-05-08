const styles = {
  container: { display: "flex", flexDirection: "column" as const, gap: "1rem" },
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
  sectionTitle: {
    fontSize: "0.75rem",
    color: "var(--gold)",
    fontWeight: 600,
    textTransform: "uppercase" as const,
    letterSpacing: "0.06em",
  },
  table: {
    width: "100%",
    borderCollapse: "collapse" as const,
    fontSize: "0.85rem",
  },
  th: {
    textAlign: "left" as const,
    padding: "0.375rem 0.5rem",
    color: "var(--text-muted)",
    fontWeight: 600,
    fontSize: "0.75rem",
    borderBottom: "1px solid var(--line)",
  },
  td: {
    padding: "0.375rem 0.5rem",
    borderBottom: "1px solid var(--line)",
    color: "var(--text)",
  },
};

function str(v: unknown): string {
  if (v === null || v === undefined) return "—";
  return String(v);
}

function obj(v: unknown): Record<string, unknown> {
  if (v && typeof v === "object" && !Array.isArray(v)) return v as Record<string, unknown>;
  return {};
}

function arr(v: unknown): unknown[] {
  return Array.isArray(v) ? v : [];
}

interface VimshottariProps {
  result: Record<string, unknown>;
}

export default function Vimshottari({ result }: VimshottariProps) {
  const mahadasha = obj(result.mahadasha ?? result.current_mahadasha);
  const antardasha = obj(result.antardasha ?? result.current_antardasha);
  const periods = arr(result.upcoming_periods ?? result.periods);

  return (
    <div style={styles.container}>
      <div style={styles.grid}>
        <div style={styles.cell}>
          <span style={styles.label}>Mahadasha (Major Period)</span>
          <span style={styles.value}>{str(mahadasha.planet ?? result.mahadasha)}</span>
          {mahadasha.years_remaining != null && (
            <span style={styles.sub}>{str(mahadasha.years_remaining)} years remaining</span>
          )}
          {mahadasha.start != null && <span style={styles.sub}>From: {str(mahadasha.start)}</span>}
          {mahadasha.end != null && <span style={styles.sub}>Until: {str(mahadasha.end)}</span>}
        </div>
        <div style={styles.cell}>
          <span style={styles.label}>Antardasha (Sub-Period)</span>
          <span style={styles.value}>{str(antardasha.planet ?? result.antardasha)}</span>
          {antardasha.start != null && <span style={styles.sub}>From: {str(antardasha.start)}</span>}
          {antardasha.end != null && <span style={styles.sub}>Until: {str(antardasha.end)}</span>}
        </div>
      </div>

      {periods.length > 0 && (
        <div>
          <span style={styles.sectionTitle}>Upcoming Periods</span>
          <table style={styles.table}>
            <thead>
              <tr>
                <th style={styles.th}>Planet</th>
                <th style={styles.th}>Start</th>
                <th style={styles.th}>End</th>
                <th style={styles.th}>Level</th>
              </tr>
            </thead>
            <tbody>
              {periods.slice(0, 8).map((p, i) => {
                const period = obj(p);
                return (
                  <tr key={i}>
                    <td style={styles.td}>{str(period.planet ?? period.name)}</td>
                    <td style={styles.td}>{str(period.start)}</td>
                    <td style={styles.td}>{str(period.end)}</td>
                    <td style={styles.td}>{str(period.level ?? period.type)}</td>
                  </tr>
                );
              })}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
}
