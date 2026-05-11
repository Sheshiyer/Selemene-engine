const styles = {
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
  number: {
    fontSize: "1.75rem",
    fontWeight: 800,
    fontFamily: "var(--font-display)",
    color: "var(--gold)",
  },
  name: {
    fontSize: "0.875rem",
    fontWeight: 600,
    color: "var(--text)",
  },
  desc: {
    fontSize: "0.8rem",
    color: "var(--text-muted)",
    lineHeight: 1.4,
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

interface NEntry {
  key: string;
  label: string;
}

const NUMBERS: NEntry[] = [
  { key: "life_path", label: "Life Path" },
  { key: "expression", label: "Expression" },
  { key: "soul_urge", label: "Soul Urge" },
  { key: "personality", label: "Personality" },
  { key: "personal_year", label: "Personal Year" },
];

interface NumerologyProps {
  result: Record<string, unknown>;
}

export default function Numerology({ result }: NumerologyProps) {
  return (
    <div style={styles.grid}>
      {NUMBERS.map((n) => {
        const raw = result[n.key];
        const data = obj(raw);
        const num = data.number ?? (typeof raw === "number" ? raw : null);
        const meaning = str(data.meaning ?? data.interpretation ?? data.description ?? "");

        return (
          <div key={n.key} style={styles.cell}>
            <span style={styles.label}>{n.label}</span>
            <span style={styles.number}>{num != null ? str(num) : "—"}</span>
            {meaning && meaning !== "—" && <span style={styles.desc}>{meaning}</span>}
          </div>
        );
      })}
    </div>
  );
}
