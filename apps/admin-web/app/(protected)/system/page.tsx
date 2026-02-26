import { PageShell } from "@/components/page-shell";

export default function SystemPage() {
  return (
    <PageShell
      title="System Operations"
      summary="Read-only operational status for API, bridges, workflows, and cache layers."
    >
      <article className="panel">
        <h3>Route Contract</h3>
        <p className="helper">
          Planned API surface: <code>GET /api/v1/admin/system/health</code>,{" "}
          <code>GET /api/v1/admin/system/services</code>,{" "}
          <code>GET /api/v1/admin/system/workflows</code>,{" "}
          <code>GET /api/v1/admin/system/cache</code>.
        </p>
      </article>
    </PageShell>
  );
}
