import { PageShell } from "@/components/page-shell";

export default function AuditPage() {
  return (
    <PageShell
      title="Audit Trail"
      summary="Immutable event stream with actor/action/request tracing for admin actions."
    >
      <article className="panel">
        <h3>Route Contract</h3>
        <p className="helper">
          Planned API surface: <code>GET /api/v1/admin/audit-events</code>,{" "}
          <code>GET /api/v1/admin/audit-events/{"{event_id}"}</code>,{" "}
          <code>GET /api/v1/admin/audit-events/actions</code>.
        </p>
      </article>
    </PageShell>
  );
}
