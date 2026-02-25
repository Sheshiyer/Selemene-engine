import { PageShell } from "@/components/page-shell";

export default function HistorySyncPage() {
  return (
    <PageShell
      title="History Sync"
      summary="Track out-of-sync users/devices and inspect ingestion drift before repair actions."
    >
      <article className="panel">
        <h3>Route Contract</h3>
        <p className="helper">
          Planned API surface: <code>GET /api/v1/admin/history-sync/users</code>,{" "}
          <code>GET /api/v1/admin/history-sync/devices</code>,{" "}
          <code>GET /api/v1/admin/history-sync/events</code>.
        </p>
      </article>
    </PageShell>
  );
}
