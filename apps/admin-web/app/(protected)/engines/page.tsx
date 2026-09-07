"use client";

import { useCallback, useEffect, useState } from "react";
import Link from "next/link";
import { ActionRail, MetricSurface, SurfaceCard } from "@/components/admin-primitives";
import { StateBanner, StatePanel } from "@/components/admin-state";
import { PageShell } from "@/components/page-shell";
import { getAuthToken } from "@/lib/auth";
import { ApiClientError, getAdminCapabilities } from "@/lib/api";
import { capabilityStateLabel, type AdminCapability } from "@/lib/engine-capability";

function categoryBadgeClass(category: string): string {
  if (category === "native") return "pill ok";
  if (category === "typescript") return "pill warning";
  if (category === "python") return "pill danger";
  return "pill";
}

function statusIndicator(status: string): string {
  if (status === "available") return "indicator-green";
  if (status === "degraded") return "indicator-yellow";
  return "indicator-gray";
}

export default function EnginesPage() {
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [engines, setEngines] = useState<AdminCapability[]>([]);

  const loadData = useCallback(async () => {
    const token = getAuthToken() ?? undefined;
    return getAdminCapabilities(token);
  }, []);

  useEffect(() => {
    let cancelled = false;
    async function run() {
      setLoading(true);
      setError(null);
      try {
        const result = await loadData();
        if (!cancelled) {
          setEngines(result.capabilities);
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
      summary="Browse the runtime engine registry with category, status, and performance metrics."
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
                title={engine.display_name}
                summary={`Phase ${engine.required_phase ?? 0} · ${engine.public_mirror ? "public mirror" : "runtime identity"}`}
              >
                <div className="grid metrics">
                  <MetricSurface
                    label="Category"
                    value={
                      <span className={categoryBadgeClass(engine.runtime_kind)}>
                        {engine.runtime_kind}
                      </span>
                    }
                  />
                  <MetricSurface
                    label="Status"
                    value={
                      <span>
                        <span className={statusIndicator(engine.availability)} />{" "}
                        {capabilityStateLabel(engine)}
                      </span>
                    }
                  />
                  <MetricSurface
                    label="Calculate"
                    value={engine.operations?.calculate === "supported" ? "supported" : "unsupported"}
                    detail={engine.reason_code ?? "no observation"}
                  />
                  <MetricSurface
                    label="Public Mirror"
                    value={engine.public_mirror ? "yes" : "no"}
                    detail="public mirror"
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
