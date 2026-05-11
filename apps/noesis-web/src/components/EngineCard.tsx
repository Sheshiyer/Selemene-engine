import type { ReactNode } from "react";

const styles = {
  card: {
    background: "var(--bg-panel)",
    border: "1px solid var(--line)",
    borderRadius: "var(--radius)",
    overflow: "hidden",
  },
  header: {
    display: "flex",
    alignItems: "center",
    justifyContent: "space-between",
    padding: "0.75rem 1rem",
    borderBottom: "1px solid var(--line)",
  },
  title: {
    fontFamily: "var(--font-display)",
    fontSize: "0.95rem",
    fontWeight: 700,
    color: "var(--text)",
  },
  badge: {
    fontSize: "0.65rem",
    fontWeight: 600,
    textTransform: "uppercase" as const,
    letterSpacing: "0.06em",
    padding: "0.2rem 0.5rem",
    borderRadius: 4,
  },
  badgeReady: {
    background: "var(--emerald-soft)",
    color: "var(--emerald)",
  },
  badgeLoading: {
    background: "var(--gold-soft)",
    color: "var(--gold)",
  },
  badgeError: {
    background: "rgba(239,107,115,0.15)",
    color: "var(--danger)",
  },
  body: {
    padding: "1rem",
  },
};

interface EngineCardProps {
  name: string;
  status: "ready" | "loading" | "error" | "idle";
  children: ReactNode;
}

export default function EngineCard({ name, status, children }: EngineCardProps) {
  const badgeLabel =
    status === "ready"
      ? "Ready"
      : status === "loading"
        ? "Loading…"
        : status === "error"
          ? "Error"
          : "Idle";

  const badgeStyle =
    status === "ready"
      ? styles.badgeReady
      : status === "loading"
        ? styles.badgeLoading
        : status === "error"
          ? styles.badgeError
          : styles.badgeLoading;

  return (
    <div style={styles.card}>
      <div style={styles.header}>
        <span style={styles.title}>{name}</span>
        <span style={{ ...styles.badge, ...badgeStyle }}>{badgeLabel}</span>
      </div>
      <div style={styles.body}>{children}</div>
    </div>
  );
}
