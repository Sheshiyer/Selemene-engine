"use client";

import { useEffect, useState } from "react";
import { SurfaceCard } from "@/components/admin-primitives";
import { StateBanner, TableEmptyStateRow } from "@/components/admin-state";
import { PageShell } from "@/components/page-shell";
import { getAuthToken } from "@/lib/auth";
import { ApiClientError, getAdminBillingPlans } from "@/lib/api";
import type { AdminBillingPlanItem } from "@/types/admin";

function formatDateTime(value: string): string {
  const date = new Date(value);
  return Number.isNaN(date.getTime()) ? value : date.toLocaleString();
}

export default function AdminBillingPlansPage() {
  const [items, setItems] = useState<AdminBillingPlanItem[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const token = getAuthToken() ?? undefined;
let cancelled = false;
    getAdminBillingPlans(token)
      .then((resp) => {
        if (!cancelled) {
          setItems(resp.items);
          setError(null);
        }
      })
      .catch((err) => {
        if (cancelled) return;
        setError(
          err instanceof ApiClientError
            ? err.message
            : err instanceof Error
              ? err.message
              : "Failed to load plans"
        );
      })
      .finally(() => {
        if (!cancelled) setLoading(false);
      });
    return () => {
      cancelled = true;
    };
  }, []);

  return (
    <PageShell
      title="Plan catalog"
      summary="The plan_catalog table is the source of truth. Editing is intentionally not exposed in the UI — change via migration + restart."
    >
      {error ? (
        <StateBanner
          variant="error"
          title="Unable to load plans"
          description={error}
        />
      ) : null}

      <SurfaceCard
        eyebrow="Billing"
        title="Plan catalog"
        summary="Source of truth for the plan_catalog table. Editing is intentionally not exposed here — change via migration + restart."
      >
        <div style={{ overflowX: "auto" }}>
          <table className="data-table">
            <thead>
              <tr>
                <th>Code</th>
                <th>Display name</th>
                <th>Active</th>
                <th>Dodo product ID</th>
                <th>Updated</th>
              </tr>
            </thead>
            <tbody>
              {loading && items.length === 0 ? (
                <TableEmptyStateRow colSpan={5} description="Loading plans…" />
              ) : items.length === 0 ? (
                <TableEmptyStateRow
                  colSpan={5}
                  description="No plans loaded. Did you run the seed migration?"
                />
              ) : (
                items.map((p) => (
                  <tr key={p.id}>
                    <td style={{ fontFamily: "monospace" }}>{p.code}</td>
                    <td>{p.display_name}</td>
                    <td>{p.is_active ? "yes" : "no"}</td>
                    <td
                      style={{ fontFamily: "monospace", fontSize: "0.78rem" }}
                    >
                      {p.dodo_product_id ?? "—"}
                    </td>
                    <td>{formatDateTime(p.updated_at)}</td>
                  </tr>
                ))
              )}
            </tbody>
          </table>
        </div>
      </SurfaceCard>
    </PageShell>
  );
}
