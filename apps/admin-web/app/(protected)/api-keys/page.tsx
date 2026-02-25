import { PageShell } from "@/components/page-shell";

export default function ApiKeysPage() {
  return (
    <PageShell
      title="API Keys"
      summary="Manage key creation, revocation, rotation, and one-time secret reveal controls."
    >
      <article className="panel">
        <h3>Route Contract</h3>
        <p className="helper">
          Planned API surface: <code>GET /api/v1/admin/api-keys</code>,{" "}
          <code>POST /api/v1/admin/api-keys</code>,{" "}
          <code>POST /api/v1/admin/api-keys/{"{key_id}"}/revoke</code>,{" "}
          <code>POST /api/v1/admin/api-keys/{"{key_id}"}/rotate</code>.
        </p>
      </article>
    </PageShell>
  );
}
