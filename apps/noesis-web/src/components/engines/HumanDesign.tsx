const styles = {
  grid: {
    display: "grid",
    gridTemplateColumns: "repeat(auto-fit, minmax(180px, 1fr))",
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
    fontFamily: "var(--font-mono)",
  },
  section: {
    marginTop: "0.75rem",
    display: "flex",
    flexDirection: "column" as const,
    gap: "0.375rem",
  },
  sectionTitle: {
    fontSize: "0.75rem",
    color: "var(--gold)",
    fontWeight: 600,
    textTransform: "uppercase" as const,
    letterSpacing: "0.06em",
  },
  centerRow: {
    display: "flex",
    justifyContent: "space-between",
    padding: "0.25rem 0.5rem",
    borderBottom: "1px solid var(--line)",
    fontSize: "0.85rem",
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

const CENTERS = [
  "Head", "Ajna", "Throat", "G", "Heart",
  "Sacral", "Spleen", "Solar Plexus", "Root",
];

interface HumanDesignProps {
  result: Record<string, unknown>;
}

export default function HumanDesign({ result }: HumanDesignProps) {
  const profile = obj(result.profile);
  const centers = obj(result.centers);
  const cross = obj(result.incarnation_cross);
  const sunGate = result.sun_gate ?? cross.sun;
  const earthGate = result.earth_gate ?? cross.earth;

  return (
    <div>
      <div style={styles.grid}>
        <div style={styles.cell}>
          <span style={styles.label}>Type</span>
          <span style={styles.value}>{str(result.type)}</span>
        </div>
        <div style={styles.cell}>
          <span style={styles.label}>Strategy</span>
          <span style={styles.value}>{str(result.strategy)}</span>
        </div>
        <div style={styles.cell}>
          <span style={styles.label}>Authority</span>
          <span style={styles.value}>{str(result.authority)}</span>
        </div>
        <div style={styles.cell}>
          <span style={styles.label}>Profile</span>
          <span style={styles.value}>
            {profile.line1 && profile.line2
              ? `${str(profile.line1)}/${str(profile.line2)}`
              : str(result.profile)}
          </span>
        </div>
        <div style={styles.cell}>
          <span style={styles.label}>Definition</span>
          <span style={styles.value}>{str(result.definition)}</span>
        </div>
        <div style={styles.cell}>
          <span style={styles.label}>Not-Self Theme</span>
          <span style={styles.value}>{str(result.not_self_theme)}</span>
        </div>
      </div>

      {(sunGate != null || earthGate != null) && (
        <div style={styles.section}>
          <span style={styles.sectionTitle}>Incarnation Cross Seed</span>
          <div style={styles.grid}>
            <div style={styles.cell}>
              <span style={styles.label}>Sun Gate</span>
              <span style={styles.value}>{str(sunGate)}</span>
            </div>
            <div style={styles.cell}>
              <span style={styles.label}>Earth Gate</span>
              <span style={styles.value}>{str(earthGate)}</span>
            </div>
          </div>
        </div>
      )}

      <div style={styles.section}>
        <span style={styles.sectionTitle}>Centers</span>
        {CENTERS.map((name) => {
          const state = centers[name.toLowerCase().replace(/ /g, "_")];
          const defined =
            state === true ||
            state === "defined" ||
            (typeof state === "object" && state !== null && (state as Record<string, unknown>).defined === true);
          return (
            <div key={name} style={styles.centerRow}>
              <span>{name}</span>
              <span
                style={{
                  color: defined ? "var(--emerald)" : "var(--text-dim)",
                  fontWeight: defined ? 600 : 400,
                }}
              >
                {defined ? "Defined" : "Undefined"}
              </span>
            </div>
          );
        })}
      </div>
    </div>
  );
}
