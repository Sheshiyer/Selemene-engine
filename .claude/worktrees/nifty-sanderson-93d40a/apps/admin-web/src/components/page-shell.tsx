import { SurfaceCard } from "@/components/admin-primitives";

interface PageShellProps {
  title: string;
  summary: string;
  eyebrow?: string;
  actions?: React.ReactNode;
  children: React.ReactNode;
}

export function PageShell({
  title,
  summary,
  eyebrow = "Operational Surface",
  actions,
  children
}: PageShellProps) {
  return (
    <SurfaceCard
      eyebrow={eyebrow}
      title={title}
      summary={summary}
      actions={actions}
      className="page-shell-panel"
    >
      <div className="grid page-shell-grid">{children}</div>
    </SurfaceCard>
  );
}
