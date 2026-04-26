import type { ReactNode } from "react";

function joinClasses(...values: Array<string | false | null | undefined>) {
  return values.filter(Boolean).join(" ");
}

interface SurfaceCardProps {
  eyebrow?: string;
  title?: string;
  summary?: string;
  actions?: ReactNode;
  className?: string;
  children: ReactNode;
}

export function SurfaceCard({
  eyebrow,
  title,
  summary,
  actions,
  className,
  children
}: SurfaceCardProps) {
  return (
    <section className={joinClasses("surface-card filigree-frame", className)}>
      {eyebrow || title || summary || actions ? (
        <header className="surface-card-header">
          <div>
            {eyebrow ? <div className="eyebrow">{eyebrow}</div> : null}
            {title ? <h3>{title}</h3> : null}
            {summary ? <p className="helper">{summary}</p> : null}
          </div>
          {actions ? <div className="surface-card-actions">{actions}</div> : null}
        </header>
      ) : null}
      <div className="surface-card-body">{children}</div>
    </section>
  );
}

interface MetricSurfaceProps {
  label: string;
  value: ReactNode;
  detail?: ReactNode;
  className?: string;
}

export function MetricSurface({ label, value, detail, className }: MetricSurfaceProps) {
  return (
    <article className={joinClasses("metric metric-surface", className)}>
      <div className="label">{label}</div>
      <div className="value">{value}</div>
      {detail ? <div className="helper metric-detail">{detail}</div> : null}
    </article>
  );
}

interface ActionRailProps {
  label?: string;
  className?: string;
  children: ReactNode;
}

export function ActionRail({
  label = "Action rail",
  className,
  children
}: ActionRailProps) {
  return (
    <section className={joinClasses("action-rail", className)} aria-label={label}>
      {children}
    </section>
  );
}
