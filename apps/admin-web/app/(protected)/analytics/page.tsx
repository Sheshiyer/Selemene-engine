import { PageShell } from "@/components/page-shell";

export default function AnalyticsPage() {
  return (
    <PageShell
      title="Usage Analytics"
      summary="Time-windowed usage, engine segmentation, and top-consumer attribution."
    >
      <div className="grid metrics">
        <article className="metric">
          <div className="label">Requests</div>
          <div className="value">--</div>
        </article>
        <article className="metric">
          <div className="label">P95 (ms)</div>
          <div className="value">--</div>
        </article>
        <article className="metric">
          <div className="label">Unique Keys</div>
          <div className="value">--</div>
        </article>
      </div>
      <article className="panel">
        <h3>Route Contract</h3>
        <p className="helper">
          Planned API surface: <code>GET /api/v1/admin/analytics/summary</code>,{" "}
          <code>GET /api/v1/admin/analytics/usage-timeseries</code>,{" "}
          <code>GET /api/v1/admin/analytics/usage-breakdown</code>,{" "}
          <code>GET /api/v1/admin/analytics/top-consumers</code>.
        </p>
      </article>
    </PageShell>
  );
}
