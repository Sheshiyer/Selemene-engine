"use client";

import { useCallback, useEffect, useState } from "react";
import Link from "next/link";
import { ActionRail, MetricSurface, SurfaceCard } from "@/components/admin-primitives";
import { StateBanner, StatePanel } from "@/components/admin-state";
import { PageShell } from "@/components/page-shell";
import { getAuthToken } from "@/lib/auth";
import { ApiClientError, getAdminEngines } from "@/lib/api";
import { statusPillClass } from "@/lib/status";
import type { AdminSystemEngineItem } from "@/types/admin";

function categoryBadgeClass(category: string): string {
  if (category === "rust-native") return "pill ok";
  if (category === "ts-bridge") return "pill warning";
  if (category === "python-sidecar") return "pill danger";
  return "pill";
}

function statusIndicator(status: string): string {
  if (status === "healthy") return "indicator-green";
  if (status === "degraded") return "indicator-yellow";
  return "indicator-gray";
}

function formatMs(value: number): string {
  return `${value.toLocaleString()} ms`;
}

export default function EnginesPage() {
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [engines, setEngines] = useState<AdminSystemEngineItem[]>([]);

  const loadData = useCallback(async () => {
    const token = getAuthToken() ?? undefined;
    return getAdminEngines(token);
  }, []);

  useEffect(() => {
    let cancelled = false;
    async function run() {
      setLoading(true);
      setError(null);
      try {
        const result = await loadData();
        if (!cancelled) {
          setEngines(result.items);
          setError(null);
        }
      } catch (err) {
        if (!cancelled) {
          setError(
            err instanceof ApiClientError
              ? err.payload?.error || err.message
              : "Failed to load engine registry"
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

  return (
    <PageShell
      title="Engine Registry"
      summary="View all 16 consciousness engines with category, status, and performance metrics."
      actions={
        <ActionRail label="Engine actions">
          <button type="button" onClick={() => window.location.reload()}>
            Refresh
          </button>
        </ActionRail>
      }
    >
      {error ? <StateBanner variant="error" title={error} /> : null}

      {loading ? (
        <StatePanel
          variant="loading"
          title="Loading engine registry"
          description="Resolving engine status, category mapping, recent runs, and performance metrics."
        />
      ) : (
        <div className="grid two-col">
          {engines.map((engine) => (
            <Link
              key={engine.engine_id}
              href={`/engines/${engine.engine_id}`}
              style={{ textDecoration: "none" }}
            >
              <SurfaceCard
                eyebrow={engine.engine_id}
                title={engine.engine_name}
                summary={`Phase ${engine.required_phase}`}
              >
                <div className="grid metrics">
                  <MetricSurface
                    label="Category"
                    value={
                      <span className={categoryBadgeClass(engine.category)}>
                        {engine.category}
                      </span>
                    }
                  />
                  <MetricSurface
                    label="Status"
                    value={
                      <span>
                        <span className={statusIndicator(engine.status)} />{" "}
                        {engine.status}
                      </span>
                    }
                  />
                  <MetricSurface
                    label="Recent Runs"
                    value={engine.recent_runs}
                    detail={`${engine.failure_runs} failures`}
                  />
                  <MetricSurface
                    label="Avg Duration"
                    value={formatMs(engine.avg_duration_ms)}
                  />
                </div>
              </SurfaceCard>
            </Link>
          ))}
          {engines.length === 0 ? (
            <StatePanel
              variant="empty"
              title="No engines registered"
              description="No engine records are available in the registry."
            />
          ) : null}
        </div>
      )}
    </PageShell>
  );
}