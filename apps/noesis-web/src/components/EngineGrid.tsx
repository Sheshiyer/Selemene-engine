"use client";

import type { CompassMode } from "@/components/CompassSelector";

export interface EngineGridItem {
  id: string;
  label: string;
  sigil: string;
  direction: Exclude<CompassMode, "full-spectrum">;
}

const directionColor: Record<EngineGridItem["direction"], string> = {
  stabilize: "var(--c-violet)",
  heal: "var(--c-indigo)",
  create: "var(--c-emerald)",
  mutate: "var(--signal)",
};

const styles = {
  wrapper: {
    display: "flex",
    flexDirection: "column" as const,
    gap: "0.875rem",
  },
  header: {
    display: "flex",
    alignItems: "center",
    justifyContent: "space-between",
    gap: "1rem",
  },
  title: {
    fontFamily: "var(--font-display)",
    fontSize: "1rem",
    letterSpacing: "0.06em",
    color: "var(--signal)",
  },
  meta: {
    fontFamily: "var(--font-mono)",
    fontSize: "0.68rem",
    letterSpacing: "0.08em",
    color: "var(--muted)",
    textTransform: "uppercase" as const,
  },
  grid: {
    display: "grid",
    gap: "0.75rem",
  },
  cell: {
    width: 80,
    minHeight: 108,
    display: "flex",
    flexDirection: "column" as const,
    alignItems: "center",
    gap: "0.45rem",
    color: "var(--muted)",
    textAlign: "center" as const,
    animation: "growIn 0.35s var(--ease-out-expo) both",
  },
  square: {
    position: "relative" as const,
    width: 80,
    height: 80,
    padding: 1,
    borderRadius: "var(--r-sm)",
    background: "var(--line-mid)",
    transition: "box-shadow 0.18s, background 0.18s, transform 0.18s",
  },
  squareInner: {
    width: "100%",
    height: "100%",
    borderRadius: "calc(var(--r-sm) - 1px)",
    background: "var(--surface)",
    display: "flex",
    alignItems: "center",
    justifyContent: "center",
  },
  activeSquare: {
    background: "var(--grad-ba)",
    boxShadow: "var(--glow-indigo)",
    transform: "translateY(-1px)",
  },
  sigil: {
    fontFamily: "var(--font-display)",
    fontSize: "1.65rem",
    lineHeight: 1,
  },
  dot: {
    position: "absolute" as const,
    top: 7,
    right: 7,
    width: 7,
    height: 7,
    borderRadius: "50%",
    background: "var(--c-emerald)",
    boxShadow: "var(--glow-emerald)",
  },
  label: {
    fontFamily: "var(--font-mono)",
    fontSize: "0.62rem",
    lineHeight: 1.25,
    letterSpacing: "0.04em",
  },
};

interface EngineGridProps {
  engines: EngineGridItem[];
  activeEngineId: string;
  availableEngineIds: Set<string>;
  onSelect: (engineId: string) => void;
}

export default function EngineGrid({
  engines,
  activeEngineId,
  availableEngineIds,
  onSelect,
}: EngineGridProps) {
  return (
    <section style={styles.wrapper} aria-label="Engine geometry grid">
      <div style={styles.header}>
        <h2 style={styles.title}>Engine Mandala</h2>
        <span style={styles.meta}>{engines.length} visible engines</span>
      </div>
      <div className="engine-icon-grid" style={styles.grid}>
        {engines.map((engine, index) => {
          const active = activeEngineId === engine.id;
          const hasData = availableEngineIds.has(engine.id);
          return (
            <button
              key={engine.id}
              type="button"
              onClick={() => onSelect(engine.id)}
              style={{
                ...styles.cell,
                animationDelay: `${index * 30}ms`,
                color: active ? "var(--text)" : "var(--muted)",
              }}
              aria-pressed={active}
            >
              <span
                style={{
                  ...styles.square,
                  ...(active ? styles.activeSquare : {}),
                }}
              >
                <span style={styles.squareInner}>
                  <span
                    style={{
                      ...styles.sigil,
                      color: active
                        ? "var(--signal)"
                        : directionColor[engine.direction],
                    }}
                  >
                    {engine.sigil}
                  </span>
                </span>
                {hasData && <span style={styles.dot} aria-hidden />}
              </span>
              <span style={styles.label}>{engine.label}</span>
            </button>
          );
        })}
      </div>
    </section>
  );
}
