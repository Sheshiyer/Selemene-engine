import type { ReactNode } from "react";

interface EngineCardProps {
  name: string;
  status: "ready" | "loading" | "error" | "idle";
  children: ReactNode;
}

export default function EngineCard({ name, status, children }: EngineCardProps) {
  const borderState =
    status === "ready"
      ? { borderColor: "var(--line-strong)", borderTop: "1px solid var(--c-emerald)" }
      : status === "loading"
        ? { borderColor: "var(--c-indigo)", animation: "pulse-inset 1.6s ease-in-out infinite" }
        : status === "error"
          ? { borderColor: "var(--error)", boxShadow: "0 0 8px rgba(198,93,59,0.25)" }
          : { borderColor: "var(--line-faint)" };

  return (
    <div
      style={{
        background: "var(--surface-1)",
        border: "1px solid var(--line-faint)",
        borderRadius: "var(--r-md)",
        overflow: "hidden",
        transition: "border-color 0.3s, box-shadow 0.3s",
        ...borderState,
      }}
    >
      <div
        style={{
          display: "flex",
          alignItems: "center",
          padding: "0.75rem 1rem",
          borderBottom: "1px solid var(--line-mid)",
        }}
      >
        <span
          style={{
            fontFamily: "var(--font-display)",
            fontSize: "1.05rem",
            fontWeight: 700,
            color: "var(--text)",
            letterSpacing: "0.03em",
          }}
        >
          {name}
        </span>
      </div>
      <div style={{ padding: "1rem" }}>{children}</div>
    </div>
  );
}
