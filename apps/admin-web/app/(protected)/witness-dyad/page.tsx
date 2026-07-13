"use client";

import { useCallback, useEffect, useMemo, useState } from "react";
import { usePathname, useRouter, useSearchParams } from "next/navigation";
import Link from "next/link";
import { ActionRail } from "@/components/admin-primitives";
import { StateBanner, StatePanel, TableEmptyStateRow } from "@/components/admin-state";
import { PageShell } from "@/components/page-shell";
import { getAuthToken } from "@/lib/auth";
import { ApiClientError, getWitnessDyadExecutions } from "@/lib/api";
import { statusPillClass } from "@/lib/status";
import { buildQueryString, getNumberParam, getStringParam } from "@/lib/url-query";
import type { AdminWitnessDyadExecutionItem } from "@/types/admin";

function formatDateTime(value: string | null): string {
  if (!value) return "--";
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return "--";
  return date.toLocaleString();
}

const TIER_OPTIONS = ["", "free", "practitioner", "adept", "sage"] as const;
const LLM_OPTIONS = [
  { value: "", label: "All" },
  { value: "true", label: "LLM powered" },
  { value: "false", label: "Rule based" }
] as const;

export default function WitnessDyadPage() {
  const router = useRouter();
  const pathname = usePathname();
  const searchParams = useSearchParams();

  const tier = getStringParam(searchParams, "tier");
  const llmPowered = getStringParam(searchParams, "llm_powered");
  const from = getStringParam(searchParams, "from");
  const to = getStringParam(searchParams, "to");
  const limit = getNumberParam(searchParams, "limit", 25, 1, 100);
  const offset = getNumberParam(searchParams, "offset", 0, 0, 10000);

  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [items, setItems] = useState<AdminWitnessDyadExecutionItem[]>([]);
  const [total, setTotal] = useState(0);

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
    const token = getAuthToken() ?? undefined;

    const currentTier = getStringParam(searchParams, "tier");
    const currentLlmPowered = getStringParam(searchParams, "llm_powered");
    const currentFrom = getStringParam(searchParams, "from");
    const currentTo = getStringParam(searchParams, "to");
    const currentLimit = getNumberParam(searchParams, "limit", 25, 1, 100);
    const currentOffset = getNumberParam(searchParams, "offset", 0, 0, 10000);

    return getWitnessDyadExecutions(token, {
      tier: currentTier || undefined,
      llm_powered:
        currentLlmPowered === "true"
          ? true
          : currentLlmPowered === "false"
            ? false
            : undefined,
      from: currentFrom || undefined,
      to: currentTo || undefined,
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
              : "Failed to load witness dyad executions"
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
      title="Witness Dyad Executions"
      summary="Browse LLM-powered and rule-based dyad interpretations with full pillar text and metadata."
      actions={
        <ActionRail label="Witness dyad actions">
          <button type="button" onClick={() => window.location.reload()}>
            Refresh
          </button>
        </ActionRail>
      }
    >
      <div className="panel-inline">
        <label>
          Tier
          <select
            value={tier}
            onChange={(event) =>
              updateQuery({
                tier: event.target.value || undefined,
                offset: undefined
              })
            }
          >
            {TIER_OPTIONS.map((value) => (
              <option key={value} value={value}>
                {value || "All tiers"}
              </option>
            ))}
          </select>
        </label>
        <label>
          LLM Mode
          <select
            value={llmPowered}
            onChange={(event) =>
              updateQuery({
                llm_powered: event.target.value || undefined,
                offset: undefined
              })
            }
          >
            {LLM_OPTIONS.map((opt) => (
              <option key={opt.value} value={opt.value}>
                {opt.label}
              </option>
            ))}
          </select>
        </label>
        <label>
          From
          <input
            type="datetime-local"
            value={from}
            onChange={(event) =>
              updateQuery({ from: event.target.value || undefined, offset: undefined })
            }
          />
        </label>
        <label>
          To
          <input
            type="datetime-local"
            value={to}
            onChange={(event) =>
              updateQuery({ to: event.target.value || undefined, offset: undefined })
            }
          />
        </label>
      </div>

      {error ? <StateBanner variant="error" title={error} /> : null}

      {loading ? (
        <StatePanel
          variant="loading"
          title="Loading witness dyad executions"
          description="Resolving filtered execution rows, tier breakdowns, and LLM metadata."
        />
      ) : (
        <article className="panel">
          <h3>Executions</h3>
          <div className="table-wrap compact">
            <table>
              <thead>
                <tr>
                  <th>User</th>
                  <th>Tier</th>
                  <th>Consciousness</th>
                  <th>LLM</th>
                  <th>Provider</th>
                  <th>Created</th>
                </tr>
              </thead>
              <tbody>
                {items.map((item) => (
                  <tr key={item.id}>
                    <td>
                      <Link href={`/witness-dyad/${item.id}`} className="table-primary link">
                        {item.user_email}
                      </Link>
                    </td>
                    <td>{item.tier}</td>
                    <td>{item.consciousness_level}</td>
                    <td>
                      <span
                        className={statusPillClass(
                          item.llm_powered ? "healthy" : "idle"
                        )}
                      >
                        {item.llm_powered ? "LLM" : "Rule"}
                      </span>
                    </td>
                    <td>{item.llm_provider ?? "--"}</td>
                    <td>{formatDateTime(item.created_at)}</td>
                  </tr>
                ))}
                {items.length === 0 ? (
                  <TableEmptyStateRow
                    colSpan={6}
                    title="No witness dyad executions"
                    description="Adjust filters to widen the execution view."
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
    </PageShell>
  );
}