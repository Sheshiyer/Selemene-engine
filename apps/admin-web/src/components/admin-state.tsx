import { SurfaceCard } from "@/components/admin-primitives";

function joinClasses(...values: Array<string | false | null | undefined>) {
  return values.filter(Boolean).join(" ");
}

interface StateBannerProps {
  variant: "error" | "success";
  title: string;
  description?: string;
  className?: string;
}

export function StateBanner({ variant, title, description, className }: StateBannerProps) {
  const eyebrow = variant === "error" ? "Error" : "Success";

  return (
    <section
      className={joinClasses("state-banner", `state-banner-${variant}`, className)}
      role={variant === "error" ? "alert" : "status"}
    >
      <div className="telemetry-caption">{eyebrow}</div>
      <div className="state-banner-title">{title}</div>
      {description ? <div className="helper">{description}</div> : null}
    </section>
  );
}

interface StatePanelProps {
  variant: "loading" | "empty" | "info";
  title: string;
  description: string;
  className?: string;
}

export function StatePanel({ variant, title, description, className }: StatePanelProps) {
  const eyebrow =
    variant === "loading" ? "Loading" : variant === "empty" ? "Empty" : "State";

  return (
    <SurfaceCard
      eyebrow={eyebrow}
      title={title}
      summary={description}
      className={joinClasses("state-panel", `state-panel-${variant}`, className)}
    >
      <div className="ornament-rule" />
    </SurfaceCard>
  );
}

interface TableEmptyStateRowProps {
  colSpan: number;
  description: string;
  title?: string;
}

export function TableEmptyStateRow({
  colSpan,
  description,
  title = "Nothing to show"
}: TableEmptyStateRowProps) {
  return (
    <tr>
      <td colSpan={colSpan} className="state-table-empty-cell">
        <div className="state-table-empty">
          <div className="telemetry-caption">Empty</div>
          <div className="state-table-empty-title">{title}</div>
          <div className="helper">{description}</div>
        </div>
      </td>
    </tr>
  );
}
