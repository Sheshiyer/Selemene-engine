const styles = {
  container: { display: "flex", flexDirection: "column" as const, gap: "0.75rem" },
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
    gap: "0.375rem",
  },
  label: {
    fontSize: "0.7rem",
    color: "var(--text-muted)",
    textTransform: "uppercase" as const,
    letterSpacing: "0.06em",
    fontWeight: 600,
  },
  value: {
    fontSize: "0.9rem",
    fontWeight: 600,
    color: "var(--text)",
  },
  sectionTitle: {
    fontSize: "0.75rem",
    color: "var(--gold)",
    fontWeight: 600,
    textTransform: "uppercase" as const,
    letterSpacing: "0.06em",
    marginTop: "0.5rem",
  },
  row: {
    display: "flex",
    justifyContent: "space-between",
    padding: "0.375rem 0.5rem",
    borderBottom: "1px solid var(--line)",
    fontSize: "0.85rem",
  },
  badge: {
    fontSize: "0.7rem",
    padding: "0.15rem 0.4rem",
    borderRadius: 4,
    fontWeight: 600,
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

interface TransitsProps {
  result: Record<string, unknown>;
}

export default function Transits({ result }: TransitsProps) {
  const positions = arr(result.planetary_positions ?? result.positions ?? []);
  const aspects = arr(result.significant_aspects ?? result.aspects ?? []);
  const sadeSati = obj(result.sade_sati);

  return (
    <div style={styles.container}>
      {positions.length > 0 && (
        <div>
          <span style={styles.sectionTitle}>Planetary Positions</span>
          <div style={styles.grid}>
            {positions.map((p, i) => {
              const planet = obj(p);
              return (
                <div key={i} style={styles.cell}>
                  <span style={styles.label}>{str(planet.planet ?? planet.name)}</span>
                  <span style={styles.value}>
                    {str(planet.sign ?? planet.rashi)} {planet.degree != null ? `${str(planet.degree)}°` : ""}
                  </span>
                  {planet.nakshatra != null && (
                    <span style={{ fontSize: "0.8rem", color: "var(--text-muted)" }}>
                      {str(planet.nakshatra)}
                    </span>
                  )}
                </div>
              );
            })}
          </div>
        </div>
      )}

      {aspects.length > 0 && (
        <div>
          <span style={styles.sectionTitle}>Significant Aspects</span>
          {aspects.map((a, i) => {
            const aspect = obj(a);
            return (
              <div key={i} style={styles.row}>
                <span style={{ color: "var(--text)" }}>
                  {str(aspect.planet1 ?? aspect.from)} — {str(aspect.planet2 ?? aspect.to)}
                </span>
                <span
                  style={{
                    ...styles.badge,
                    background: "var(--gold-soft)",
                    color: "var(--gold)",
                  }}
                >
                  {str(aspect.type ?? aspect.aspect)}
                </span>
              </div>
            );
          })}
        </div>
      )}

      {sadeSati.active != null && (
        <div>
          <span style={styles.sectionTitle}>Sade Sati</span>
          <div style={styles.row}>
            <span>Status</span>
            <span
              style={{
                ...styles.badge,
                background: sadeSati.active ? "rgba(239,107,115,0.12)" : "var(--emerald-soft)",
                color: sadeSati.active ? "var(--danger)" : "var(--emerald)",
              }}
            >
              {sadeSati.active ? "Active" : "Not Active"}
            </span>
          </div>
          {sadeSati.phase != null && (
            <div style={styles.row}>
              <span>Phase</span>
              <span style={{ color: "var(--text)" }}>{str(sadeSati.phase)}</span>
            </div>
          )}
        </div>
      )}
    </div>
  );
}
