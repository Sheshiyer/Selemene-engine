const styles = {
  container: { display: "flex", flexDirection: "column" as const, gap: "1rem" },
  sectionTitle: {
    fontSize: "0.75rem",
    color: "var(--gold)",
    fontWeight: 600,
    textTransform: "uppercase" as const,
    letterSpacing: "0.06em",
  },
  sphereGrid: {
    display: "grid",
    gridTemplateColumns: "repeat(auto-fit, minmax(220px, 1fr))",
    gap: "0.75rem",
  },
  sphere: {
    background: "var(--field)",
    borderRadius: "var(--radius)",
    padding: "0.75rem",
    display: "flex",
    flexDirection: "column" as const,
    gap: "0.375rem",
  },
  sphereName: {
    fontSize: "0.85rem",
    fontWeight: 700,
    color: "var(--text)",
  },
  triplet: {
    display: "flex",
    gap: "0.375rem",
    flexWrap: "wrap" as const,
  },
  badge: {
    fontSize: "0.7rem",
    padding: "0.15rem 0.4rem",
    borderRadius: 4,
    fontWeight: 600,
  },
  shadow: { background: "rgba(239,107,115,0.12)", color: "var(--danger)" },
  gift: { background: "var(--emerald-soft)", color: "var(--emerald)" },
  siddhi: { background: "var(--gold-soft)", color: "var(--gold)" },
  gateNum: {
    fontSize: "0.8rem",
    color: "var(--text-muted)",
    fontFamily: "'IBM Plex Mono', monospace",
  },
  sub: {
    fontSize: "0.8rem",
    color: "var(--text-muted)",
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

const SPHERE_NAMES = ["Life's Work", "Evolution", "Radiance", "Purpose"];

interface GeneKeysProps {
  result: Record<string, unknown>;
}

export default function GeneKeys({ result }: GeneKeysProps) {
  const activation = obj(result.activation_sequence ?? result.activation);
  const spheres = arr(activation.spheres ?? result.spheres);
  const partner = result.programming_partner ?? result.partner;

  return (
    <div style={styles.container}>
      <span style={styles.sectionTitle}>Activation Sequence</span>
      <div style={styles.sphereGrid}>
        {spheres.length > 0
          ? spheres.map((s, i) => {
              const sphere = obj(s);
              return (
                <div key={i} style={styles.sphere}>
                  <span style={styles.sphereName}>
                    {str(sphere.name ?? SPHERE_NAMES[i] ?? `Sphere ${i + 1}`)}
                  </span>
                  {sphere.gate != null && (
                    <span style={styles.gateNum}>Gate {str(sphere.gate)}</span>
                  )}
                  <div style={styles.triplet}>
                    <span style={{ ...styles.badge, ...styles.shadow }}>
                      Shadow: {str(sphere.shadow)}
                    </span>
                    <span style={{ ...styles.badge, ...styles.gift }}>
                      Gift: {str(sphere.gift)}
                    </span>
                    <span style={{ ...styles.badge, ...styles.siddhi }}>
                      Siddhi: {str(sphere.siddhi)}
                    </span>
                  </div>
                </div>
              );
            })
          : SPHERE_NAMES.map((name, i) => {
              const key = name.toLowerCase().replace(/['\s]/g, "_");
              const sphere = obj(activation[key] ?? result[key]);
              return (
                <div key={i} style={styles.sphere}>
                  <span style={styles.sphereName}>{name}</span>
                  {sphere.gate != null && (
                    <span style={styles.gateNum}>Gate {str(sphere.gate)}</span>
                  )}
                  <div style={styles.triplet}>
                    <span style={{ ...styles.badge, ...styles.shadow }}>
                      Shadow: {str(sphere.shadow)}
                    </span>
                    <span style={{ ...styles.badge, ...styles.gift }}>
                      Gift: {str(sphere.gift)}
                    </span>
                    <span style={{ ...styles.badge, ...styles.siddhi }}>
                      Siddhi: {str(sphere.siddhi)}
                    </span>
                  </div>
                </div>
              );
            })}
      </div>

      {partner != null && (
        <div>
          <span style={styles.sectionTitle}>Programming Partner</span>
          <p style={styles.sub}>{str(partner)}</p>
        </div>
      )}
    </div>
  );
}
