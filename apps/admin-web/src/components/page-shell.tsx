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
    <section className="panel page-shell-panel filigree-frame">
      <header className="page-shell-header">
        <div>
          <div className="eyebrow">{eyebrow}</div>
          <h2>{title}</h2>
          <p className="helper">{summary}</p>
        </div>
        {actions ? <div className="page-shell-actions">{actions}</div> : null}
      </header>
      <div className="grid page-shell-grid">{children}</div>
    </section>
  );
}
