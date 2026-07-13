"use client";

import { useCallback, useEffect, useState } from "react";
import { useParams } from "next/navigation";
import Link from "next/link";
import { ActionRail, MetricSurface, SurfaceCard } from "@/components/admin-primitives";
import { StateBanner, StatePanel } from "@/components/admin-state";
import { PageShell } from "@/components/page-shell";
import { getAuthToken } from "@/lib/auth";
import { ApiClientError, getAdminEngine } from "@/lib/api";
import type { AdminSystemEngineItem } from "@/types/admin";

function categoryBadgeClass(category: string): string {
  if (category === "rust-native") return "pill ok";
  if (category === "ts-bridge") return "pill warning";
  if (category === "python-sidecar") return "pill danger";
  return "pill";
}

function formatMs(value: number): string {
  return `${value.toLocaleString()} ms`;
}

export default function EngineDetailPage() {
  const params = useParams<{ id: string }>();
  const id = params?.id;

  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [engine, setEngine] = useState<AdminSystemEngineItem | null>(null);

  const loadData = useCallback(async () => {
    const token = getAuthToken() ?? undefined;
    if (!id) throw new Error("Missing engine ID.");
    return getAdminEngine(token, id);
  }, [id]);

  useEffect(() => {
    if (!id) return;
    let cancelled = false;
    async function run() {
      setLoading(true);
      setError(null);
      try {
        const result = await loadData();
        if (!cancelled) {
          setEngine(result);
          setError(null);
        }
      } catch (err) {
        if (!cancelled) {
          setError(
            err instanceof ApiClientError
              ? err.payload?.error || err.message
              : "Failed to load engine detail"
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
  }, [loadData, id]);

  return (
    <PageShell
      title="Engine Detail"
      summary={id ?? "--"}
      actions={
        <ActionRail label="Detail actions">
          <Link href="/engines" className="shell-action-link">
            Back to registry
          </Link>
        </ActionRail>
      }
    >
      {error ? <StateBanner variant="error" title={error} /> : null}

      {loading ? (
        <StatePanel
          variant="loading"
          title="Loading engine detail"
          description="Resolving engine status, metrics, and category metadata."
        />
      ) : engine ? (
        <>
          <div className="grid metrics">
            <MetricSurface label="Engine ID" value={engine.engine_id} />
            <MetricSurface label="Name" value={engine.engine_name} />
            <MetricSurface
              label="Category"
              value={
                <span className={categoryBadgeClass(engine.category)}>
                  {engine.category}
                </span>
              }
            />
            <MetricSurface label="Required Phase" value={engine.required_phase} />
            <MetricSurface label="Status" value={engine.status} />
          </div>

          <div className="grid metrics">
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

          <SurfaceCard
            eyebrow="Operations"
            title="Performance Summary"
            summary={`Engine ${engine.engine_name} (${engine.engine_id}) with status ${engine.status}. Category ${engine.category}, phase ${engine.required_phase}.`}
          >
            <div className="grid two-col">
              <div>
                <div className="telemetry-caption">Success rate</div>
                <div className="helper">
                  {engine.recent_runs > 0
                    ? `${(((engine.recent_runs - engine.failure_runs) / engine.recent_runs) * 100).toFixed(1)}%`
                    : "N/A"}
                </div>
              </div>
              <div>
                <div className="telemetry-caption">Failure rate</div>
                <div className="helper">
                  {engine.recent_runs > 0
                    ? `${((engine.failure_runs / engine.recent_runs) * 100).toFixed(1)}%`
                    : "N/A"}
                </div>
              </div>
            </div>
          </SurfaceCard>
        </>
      ) : null}
    </PageShell>
  );
}