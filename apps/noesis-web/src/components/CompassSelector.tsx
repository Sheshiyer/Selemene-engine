"use client";

export type CompassMode =
  | "full-spectrum"
  | "stabilize"
  | "heal"
  | "create"
  | "mutate";

export interface CompassOption {
  id: CompassMode;
  label: string;
  sanskrit: string;
  intention: string;
  direction: string;
  workflowId: string;
  gradient: string;
  glyph: string;
}

export const COMPASS_OPTIONS: CompassOption[] = [
  {
    id: "stabilize",
    label: "STABILIZE",
    sanskrit: "sthira",
    intention: "Ground, root, anchor",
    direction: "NORTH",
    workflowId: "daily-practice",
    gradient: "var(--grad-stabilize)",
    glyph: "△",
  },
  {
    id: "heal",
    label: "HEAL",
    sanskrit: "cikitsa",
    intention: "Restore, integrate, clear",
    direction: "EAST",
    workflowId: "self-inquiry",
    gradient: "var(--grad-heal)",
    glyph: "◐",
  },
  {
    id: "create",
    label: "CREATE",
    sanskrit: "srishti",
    intention: "Activate, express, generate",
    direction: "SOUTH",
    workflowId: "creative-expression",
    gradient: "var(--grad-create)",
    glyph: "✦",
  },
  {
    id: "mutate",
    label: "MUTATE",
    sanskrit: "vikara",
    intention: "Transform, see, dissolve",
    direction: "WEST",
    workflowId: "decision-support",
    gradient: "var(--grad-mutate)",
    glyph: "◇",
  },
];

const fullSpectrum: CompassOption = {
  id: "full-spectrum",
  label: "FULL SPECTRUM",
  sanskrit: "sarva-mandala",
  intention: "Run the complete 17-engine reading",
  direction: "CENTER",
  workflowId: "full-spectrum",
  gradient: "var(--grad-kha)",
  glyph: "◈",
};

const styles = {
  shell: {
    display: "flex",
    flexDirection: "column" as const,
    gap: "0.75rem",
  },
  grid: {
    display: "grid",
    gridTemplateColumns: "repeat(2, minmax(0, 1fr))",
    gap: "0.75rem",
  },
  card: {
    position: "relative" as const,
    minHeight: 118,
    padding: "1px",
    borderRadius: "var(--r-md)",
    background: "var(--line-mid)",
    transition: "box-shadow 0.18s, transform 0.18s, background 0.18s",
  },
  inner: {
    height: "100%",
    padding: "1rem",
    borderRadius: "calc(var(--r-md) - 1px)",
    background: "rgba(7,11,29,0.88)",
    display: "grid",
    gridTemplateColumns: "44px 1fr",
    gap: "0.875rem",
    alignItems: "center",
    textAlign: "left" as const,
  },
  active: {
    background: "var(--grad-ba)",
    boxShadow: "var(--glow-indigo)",
  },
  glyph: {
    width: 42,
    height: 42,
    borderRadius: "50%",
    border: "1px solid var(--line-strong)",
    display: "flex",
    alignItems: "center",
    justifyContent: "center",
    color: "var(--signal)",
    fontFamily: "var(--font-display)",
    fontSize: "1.35rem",
  },
  direction: {
    display: "block",
    fontFamily: "var(--font-mono)",
    fontSize: "0.62rem",
    letterSpacing: "0.1em",
    color: "var(--muted)",
  },
  label: {
    display: "block",
    fontFamily: "var(--font-display)",
    fontSize: "0.9rem",
    letterSpacing: "0.06em",
    color: "var(--text)",
    marginTop: "0.2rem",
  },
  sanskrit: {
    display: "block",
    fontFamily: "var(--font-mono)",
    fontSize: "0.66rem",
    color: "var(--signal)",
    fontStyle: "italic",
    letterSpacing: "0.05em",
    marginTop: "0.15rem",
  },
  intention: {
    display: "block",
    fontSize: "0.76rem",
    color: "var(--muted)",
    lineHeight: 1.45,
    marginTop: "0.35rem",
  },
  fullButton: {
    padding: "0.8rem 1rem",
    borderRadius: "var(--r-sm)",
    border: "1px solid var(--line-mid)",
    background: "rgba(11,80,251,0.05)",
    color: "var(--text-2)",
    fontFamily: "var(--font-mono)",
    fontSize: "0.72rem",
    letterSpacing: "0.08em",
    textAlign: "center" as const,
    transition: "border-color 0.18s, color 0.18s, box-shadow 0.18s",
  },
  fullActive: {
    borderColor: "rgba(197,160,23,0.5)",
    color: "var(--signal)",
    boxShadow: "var(--glow-gold)",
  },
};

interface CompassSelectorProps {
  selected: CompassMode;
  onSelect: (mode: CompassMode) => void;
}

export default function CompassSelector({
  selected,
  onSelect,
}: CompassSelectorProps) {
  return (
    <section style={styles.shell} aria-label="Compass workflow selector">
      <div style={styles.grid}>
        {COMPASS_OPTIONS.map((option) => {
          const active = selected === option.id;
          return (
            <button
              key={option.id}
              type="button"
              onClick={() => onSelect(option.id)}
              style={{
                ...styles.card,
                ...(active ? styles.active : { background: option.gradient }),
              }}
              aria-pressed={active}
            >
              <div style={styles.inner}>
                <span style={styles.glyph} aria-hidden>
                  {option.glyph}
                </span>
                <span>
                  <span style={styles.direction}>{option.direction}</span>
                  <span style={styles.label}>{option.label}</span>
                  <span style={styles.sanskrit}>{option.sanskrit}</span>
                  <span style={styles.intention}>{option.intention}</span>
                </span>
              </div>
            </button>
          );
        })}
      </div>
      <button
        type="button"
        onClick={() => onSelect("full-spectrum")}
        style={{
          ...styles.fullButton,
          ...(selected === "full-spectrum" ? styles.fullActive : {}),
        }}
        aria-pressed={selected === "full-spectrum"}
      >
        {fullSpectrum.glyph} {fullSpectrum.label} · {fullSpectrum.sanskrit}
      </button>
    </section>
  );
}
