import type { ReactNode } from "react";

function joinClasses(...values: Array<string | false | null | undefined>) {
  return values.filter(Boolean).join(" ");
}

export interface EventStreamMetaItem {
  label: string;
  value: ReactNode;
}

interface EventStreamProps {
  label?: string;
  className?: string;
  children: ReactNode;
}

export function EventStream({
  label = "Event stream",
  className,
  children
}: EventStreamProps) {
  return (
    <ol className={joinClasses("event-stream", className)} aria-label={label}>
      {children}
    </ol>
  );
}

interface EventStreamItemProps {
  eyebrow?: string;
  title: ReactNode;
  subtitle?: ReactNode;
  badge?: ReactNode;
  metadata?: EventStreamMetaItem[];
  summary?: ReactNode;
  action?: ReactNode;
  className?: string;
  children?: ReactNode;
}

export function EventStreamItem({
  eyebrow,
  title,
  subtitle,
  badge,
  metadata = [],
  summary,
  action,
  className,
  children
}: EventStreamItemProps) {
  return (
    <li className={joinClasses("event-stream-item", className)}>
      <div className="event-stream-node" aria-hidden="true" />
      <article className="event-stream-surface filigree-frame">
        <header className="event-stream-header">
          <div className="event-stream-heading">
            {eyebrow ? <div className="telemetry-caption">{eyebrow}</div> : null}
            <div className="event-stream-title-row">
              <div className="event-stream-title">{title}</div>
              {badge ? <div className="event-stream-badge">{badge}</div> : null}
            </div>
            {subtitle ? <div className="helper">{subtitle}</div> : null}
          </div>
          {action ? <div className="event-stream-action">{action}</div> : null}
        </header>

        {metadata.length > 0 ? (
          <dl className="event-stream-meta">
            {metadata.map((item, index) => (
              <div className="event-stream-meta-item" key={`${item.label}-${index}`}>
                <dt>{item.label}</dt>
                <dd>{item.value}</dd>
              </div>
            ))}
          </dl>
        ) : null}

        {summary ? <div className="helper event-stream-summary">{summary}</div> : null}
        {children ? <div className="event-stream-detail">{children}</div> : null}
      </article>
    </li>
  );
}
