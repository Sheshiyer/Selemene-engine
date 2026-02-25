import { PageShell } from "@/components/page-shell";

export default function DashboardPage() {
  return (
    <PageShell
      title="Dashboard"
      summary="Live platform signal snapshots for active users, key lifecycle, and service posture."
    >
      <div className="grid metrics">
        <article className="metric">
          <div className="label">Active Users (24h)</div>
          <div className="value">--</div>
        </article>
        <article className="metric">
          <div className="label">API Requests (24h)</div>
          <div className="value">--</div>
        </article>
        <article className="metric">
          <div className="label">Error Rate</div>
          <div className="value">--</div>
        </article>
      </div>
      <article className="panel">
        <h3>Recent Admin Activity</h3>
        <p className="helper">
          Endpoint integration pending: <code>GET /api/v1/admin/audit-events?limit=10</code>
        </p>
      </article>
    </PageShell>
  );
}
