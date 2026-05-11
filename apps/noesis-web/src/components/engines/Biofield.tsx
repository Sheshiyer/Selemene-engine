const styles = {
  container: { display: "flex", flexDirection: "column" as const, gap: "0.75rem" },
  scoreRow: {
    display: "flex",
    alignItems: "center",
    gap: "0.75rem",
    padding: "0.75rem 1rem",
    background: "var(--field)",
    borderRadius: "var(--radius)",
  },
  scoreLabel: {
    fontSize: "0.75rem",
    color: "var(--text-muted)",
    textTransform: "uppercase" as const,
    fontWeight: 600,
  },
  scoreValue: {
    fontSize: "1.5rem",
    fontWeight: 800,
    fontFamily: "var(--font-display)",
    color: "var(--emerald)",
  },
  chakraRow: {
    display: "flex",
    alignItems: "center",
    gap: "0.75rem",
    padding: "0.5rem 0",
    borderBottom: "1px solid var(--line)",
  },
  chakraName: {
    fontSize: "0.85rem",
    color: "var(--text)",
    width: 120,
    flexShrink: 0,
  },
  barOuter: {
    flex: 1,
    height: 8,
    background: "var(--line)",
    borderRadius: 4,
    overflow: "hidden",
    position: "relative" as const,
  },
  pct: {
    fontSize: "0.75rem",
    color: "var(--text-muted)",
    fontFamily: "var(--font-mono)",
    width: 40,
    textAlign: "right" as const,
  },
  sectionTitle: {
    fontSize: "0.75rem",
    color: "var(--gold)",
    fontWeight: 600,
    textTransform: "uppercase" as const,
    letterSpacing: "0.06em",
  },
};

function num(v: unknown): number {
  if (typeof v === "number") return v;
  if (typeof v === "string") { const n = parseFloat(v); return isNaN(n) ? 0 : n; }
  return 0;
}

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

const CHAKRA_COLORS = [
  "#ef4444", "#f97316", "#eab308", "#22c55e",
  "#06b6d4", "#6366f1", "#a855f7",
];

const CHAKRA_NAMES = [
  "Root", "Sacral", "Solar Plexus", "Heart",
  "Throat", "Third Eye", "Crown",
];

interface BiofieldProps {
  result: Record<string, unknown>;
}

export default function Biofield({ result }: BiofieldProps) {
  const coherence = result.coherence ?? result.overall_coherence;
  const chakras = arr(result.chakras ?? result.chakra_activations ?? []);

  return (
    <div style={styles.container}>
      {coherence != null && (
        <div style={styles.scoreRow}>
          <span style={styles.scoreLabel}>Overall Coherence</span>
          <span style={styles.scoreValue}>{str(coherence)}</span>
        </div>
      )}

      <span style={styles.sectionTitle}>Chakra Activations</span>
      {chakras.length > 0
        ? chakras.map((c, i) => {
            const chakra = obj(c);
            const val = num(chakra.value ?? chakra.activation ?? chakra.energy ?? 0);
            const pct = Math.min(100, Math.max(0, val));
            const name = str(chakra.name ?? CHAKRA_NAMES[i] ?? `Chakra ${i + 1}`);
            return (
              <div key={i} style={styles.chakraRow}>
                <span style={styles.chakraName}>{name}</span>
                <div style={styles.barOuter}>
                  <div
                    style={{
                      position: "absolute",
                      left: 0,
                      top: 0,
                      height: "100%",
                      width: `${pct}%`,
                      background: CHAKRA_COLORS[i % CHAKRA_COLORS.length],
                      borderRadius: 4,
                      transition: "width 0.3s",
                    }}
                  />
                </div>
                <span style={styles.pct}>{Math.round(pct)}%</span>
              </div>
            );
          })
        : CHAKRA_NAMES.map((name, i) => {
            const key = name.toLowerCase().replace(/ /g, "_");
            const val = num(result[key] ?? 0);
            const pct = Math.min(100, Math.max(0, val));
            return (
              <div key={i} style={styles.chakraRow}>
                <span style={styles.chakraName}>{name}</span>
                <div style={styles.barOuter}>
                  <div
                    style={{
                      position: "absolute",
                      left: 0,
                      top: 0,
                      height: "100%",
                      width: `${pct}%`,
                      background: CHAKRA_COLORS[i],
                      borderRadius: 4,
                      transition: "width 0.3s",
                    }}
                  />
                </div>
                <span style={styles.pct}>{Math.round(pct)}%</span>
              </div>
            );
          })}
    </div>
  );
}
