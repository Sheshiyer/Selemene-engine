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
    fontSize: "1.1rem",
    fontWeight: 700,
    fontFamily: "'IBM Plex Mono', monospace",
  },
  barOuter: {
    width: "100%",
    height: 8,
    background: "var(--line)",
    borderRadius: 4,
    overflow: "hidden",
    position: "relative" as const,
  },
  sub: {
    fontSize: "0.75rem",
    color: "var(--text-muted)",
    fontFamily: "'IBM Plex Mono', monospace",
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

interface CycleInfo {
  key: string;
  label: string;
  color: string;
}

const CYCLES: CycleInfo[] = [
  { key: "physical", label: "Physical", color: "#ef6b73" },
  { key: "emotional", label: "Emotional", color: "#5a8fbb" },
  { key: "intellectual", label: "Intellectual", color: "var(--gold)" },
];

interface BiorhythmProps {
  result: Record<string, unknown>;
}

export default function Biorhythm({ result }: BiorhythmProps) {
  return (
    <div style={styles.container}>
      <div style={styles.grid}>
        {CYCLES.map((c) => {
          const raw = result[c.key];
          const data = obj(raw);
          const val = num(data.value ?? raw);
          const pct = Math.round(((val + 100) / 200) * 100);
          const nextPeak = str(data.next_peak ?? "");
          const nextTrough = str(data.next_trough ?? "");

          return (
            <div key={c.key} style={styles.cell}>
              <span style={styles.label}>{c.label}</span>
              <span style={{ ...styles.value, color: c.color }}>
                {val > 0 ? "+" : ""}{val}%
              </span>
              <div style={styles.barOuter}>
                <div
                  style={{
                    position: "absolute",
                    left: 0,
                    top: 0,
                    height: "100%",
                    width: `${pct}%`,
                    background: c.color,
                    borderRadius: 4,
                    transition: "width 0.3s",
                  }}
                />
              </div>
              {nextPeak && nextPeak !== "—" && (
                <span style={styles.sub}>Peak: {nextPeak}</span>
              )}
              {nextTrough && nextTrough !== "—" && (
                <span style={styles.sub}>Trough: {nextTrough}</span>
              )}
            </div>
          );
        })}
      </div>
    </div>
  );
}
