interface PageShellProps {
  title: string;
  summary: string;
  children: React.ReactNode;
}

export function PageShell({ title, summary, children }: PageShellProps) {
  return (
    <section className="panel">
      <h2>{title}</h2>
      <p className="helper">{summary}</p>
      <div className="grid">{children}</div>
    </section>
  );
}
