import { PageShell } from "@/components/page-shell";

export default function UsersPage() {
  return (
    <PageShell
      title="Users"
      summary="Search and inspect user accounts, then apply account state or tier actions."
    >
      <article className="panel">
        <h3>Route Contract</h3>
        <p className="helper">
          Planned API surface: <code>GET /api/v1/admin/users</code>,{" "}
          <code>PATCH /api/v1/admin/users/{"{user_id}"}/state</code>,{" "}
          <code>PATCH /api/v1/admin/users/{"{user_id}"}/tier</code>,{" "}
          <code>PUT /api/v1/admin/users/{"{user_id}"}/roles</code>.
        </p>
      </article>
    </PageShell>
  );
}
