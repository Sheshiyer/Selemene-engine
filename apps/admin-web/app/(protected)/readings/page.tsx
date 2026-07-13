"use client";

import { useCallback, useEffect, useState } from "react";
import { usePathname, useRouter, useSearchParams } from "next/navigation";
import { ActionRail } from "@/components/admin-primitives";
import { StateBanner, StatePanel, TableEmptyStateRow } from "@/components/admin-state";
import { DrawerSurface } from "@/components/overlay-surface";
import { PageShell } from "@/components/page-shell";
import { getAuthToken } from "@/lib/auth";
import { ApiClientError, getAdminReadings } from "@/lib/api";
import { buildQueryString, getNumberParam, getStringParam } from "@/lib/url-query";
import type { AdminReadingItem } from "@/types/admin";

function formatDateTime(value: string | null): string {
  if (!value) return "--";
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return "--";
  return date.toLocaleString();
}

function truncate(text: string | null, max = 80): string {
  if (!text) return "--";
  return text.length > max ? `${text.slice(0, max)}\u2026` : text;
}

export default function ReadingsPage() {
  const router = useRouter();
  const pathname = usePathname();
  const searchParams = useSearchParams();

  const userId = getStringParam(searchParams, "user_id");
  const engineId = getStringParam(searchParams, "engine_id");
  const limit = getNumberParam(searchParams, "limit", 25, 1, 100);
  const offset = getNumberParam(searchParams, "offset", 0, 0, 10000);

  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [items, setItems] = useState<AdminReadingItem[]>([]);
  const [total, setTotal] = useState(0);
  const [selected, setSelected] = useState<AdminReadingItem | null>(null);

  const updateQuery = useCallback(
    (updates: Record<string, string | number | undefined>) => {
      const nextQuery = buildQueryString(searchParams, updates);
      router.replace(nextQuery ? `${pathname}?${nextQuery}` : pathname, {
        scroll: false
      });
    },
    [pathname, router, searchParams]
  );

  const loadData = useCallback(async () => {
    const token = getAuthToken();
    if (!token) throw new Error("Missing session token. Please sign in again.");

    const currentUserId = getStringParam(searchParams, "user_id");
    const currentEngineId = getStringParam(searchParams, "engine_id");
    const currentLimit = getNumberParam(searchParams, "limit", 25, 1, 100);
    const currentOffset = getNumberParam(searchParams, "offset", 0, 0, 10000);

    return getAdminReadings(token, {
      user_id: currentUserId || undefined,
      engine_id: currentEngineId || undefined,
      limit: currentLimit,
      offset: currentOffset
    });
  }, [searchParams]);

  useEffect(() => {
    let cancelled = false;
    async function run() {
      setLoading(true);
      setError(null);
      try {
        const result = await loadData();
        if (!cancelled) {
          setItems(result.items);
          setTotal(result.total);
          setError(null);
        }
      } catch (err) {
        if (!cancelled) {
          setError(
            err instanceof ApiClientError
              ? err.payload?.error || err.message
              : "Failed to load readings"
          );
        }
      } finally {
        if (!cancelled) setLoading(false);
      }
    }
    void run();
    return () => {
      cancelled = true;
    };
  }, [loadData]);

  const hasPrev = offset > 0;
  const hasNext = offset + items.length < total;

  return (
    <PageShell
      title="Readings Browser"
      summary="Browse completed readings across all users with engine, workflow, and witness prompt context."
      actions={
        <ActionRail label="Readings actions">
          <button type="button" onClick={() => window.location.reload()}>
            Refresh
          </button>
        </ActionRail>
      }
    >
      <div className="panel-inline">
        <label>
          User ID
          <input
            value={userId}
            onChange={(event) =>
              updateQuery({ user_id: event.target.value || undefined, offset: undefined })
            }
            placeholder="user UUID"
          />
        </label>
        <label>
          Engine ID
          <input
            value={engineId}
            onChange={(event) =>
              updateQuery({ engine_id: event.target.value || undefined, offset: undefined })
            }
            placeholder="engine_id"
          />
        </label>
      </div>

      {error ? <StateBanner variant="error" title={error} /> : null}

      {loading ? (
        <StatePanel
          variant="loading"
          title="Loading readings"
          description="Resolving filtered reading rows with engine and workflow context."
        />
      ) : (
        <article className="panel">
          <h3>Readings</h3>
          <div className="table-wrap compact">
            <table>
              <thead>
                <tr>
                  <th>User</th>
                  <th>Engine</th>
                  <th>Workflow</th>
                  <th>Consciousness</th>
                  <th>Witness Prompt</th>
                  <th>Created</th>
                </tr>
              </thead>
              <tbody>
                {items.map((item) => (
                  <tr key={item.id}>
                    <td>
                      <button
                        type="button"
                        className="link-btn table-primary"
                        onClick={() => setSelected(item)}
                      >
                        {item.user_email}
                      </button>
                    </td>
                    <td>{item.engine_id}</td>
                    <td>{item.workflow_id ?? "--"}</td>
                    <td>{item.consciousness_level}</td>
                    <td className="cell-wrap">{truncate(item.witness_prompt, 60)}</td>
                    <td>{formatDateTime(item.created_at)}</td>
                  </tr>
                ))}
                {items.length === 0 ? (
                  <TableEmptyStateRow
                    colSpan={6}
                    title="No readings found"
                    description="Adjust user or engine filters to widen the reading view."
                  />
                ) : null}
              </tbody>
            </table>
          </div>

          <div className="table-pagination">
            <button
              type="button"
              disabled={!hasPrev}
              onClick={() =>
                updateQuery({ offset: Math.max(0, offset - limit) })
              }
            >
              Previous
            </button>
            <span className="helper">
              {offset + 1}&ndash;{Math.min(offset + limit, total)} of {total}
            </span>
            <button
              type="button"
              disabled={!hasNext}
              onClick={() => updateQuery({ offset: offset + limit })}
            >
              Next
            </button>
          </div>
        </article>
      )}

      <DrawerSurface
        open={Boolean(selected)}
        onClose={() => setSelected(null)}
        eyebrow="Reading Detail"
        title={selected?.engine_id ?? "Reading"}
        summary={
          selected
            ? `${selected.user_email} \u00b7 ${selected.workflow_id ?? "no workflow"} \u00b7 L${selected.consciousness_level}`
            : undefined
        }
        footer={
          <button type="button" onClick={() => setSelected(null)}>
            Close
          </button>
        }
      >
        {selected ? (
          <div className="grid overlay-detail-grid">
            <div className="helper">Reading ID: {selected.id}</div>
            <div className="helper">Input hash: {selected.input_hash}</div>
            <div className="helper">
              Calculation time:{" "}
              {selected.calculation_time_ms === null ? "--" : `${selected.calculation_time_ms} ms`}
            </div>
            {selected.witness_prompt ? (
              <div>
                <div className="telemetry-caption">Witness Prompt</div>
                <pre className="overlay-json-block">{selected.witness_prompt}</pre>
              </div>
            ) : null}
            <div>
              <div className="telemetry-caption">Input Data</div>
              <pre className="overlay-json-block">
                {JSON.stringify(selected.input_data, null, 2)}
              </pre>
            </div>
            <div>
              <div className="telemetry-caption">Result Data</div>
              <pre className="overlay-json-block">
                {JSON.stringify(selected.result_data, null, 2)}
              </pre>
            </div>
          </div>
        ) : null}
      </DrawerSurface>
    </PageShell>
  );
}