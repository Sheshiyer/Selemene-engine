"use client";

import { useCallback, useEffect, useState } from "react";
import { useParams } from "next/navigation";
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

export default function EngineDetailPage() {
  const params = useParams<{ id: string }>();
  const id = params?.id;

  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [engine, setEngine] = useState<AdminCapability | null>(null);

  const loadData = useCallback(async () => {
    const token = getAuthToken() ?? undefined;
    if (!id) throw new Error("Missing engine ID.");
    return getAdminCapabilities(token).then((list) => {
      const capability = list.capabilities.find((item) => item.engine_id === id);
      if (!capability) throw new Error("Engine is not present in the canonical capability list.");
      return capability;
    });
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
            <MetricSurface label="Name" value={engine.display_name} />
            <MetricSurface
              label="Category"
              value={
                <span className={categoryBadgeClass(engine.runtime_kind)}>
                  {engine.runtime_kind}
                </span>
              }
            />
            <MetricSurface label="Required Phase" value={engine.required_phase ?? 0} />
            <MetricSurface label="Status" value={capabilityStateLabel(engine)} />
            <MetricSurface label="Public Mirror" value={engine.public_mirror ? "yes" : "no"} />
          </div>

          <div className="grid metrics">
            <MetricSurface
              label="Calculate"
              value={engine.operations?.calculate ?? "unknown"}
              detail={engine.reason_code ?? "no observation"}
            />
            <MetricSurface
              label="Validate"
              value={engine.operations?.validate ?? "unknown"}
              detail={engine.runtime_kind}
            />
          </div>

          <SurfaceCard
            eyebrow="Operations"
            title="Performance Summary"
            summary={`Engine ${engine.display_name} (${engine.engine_id}) with ${capabilityStateLabel(engine)} state. Runtime ${engine.runtime_kind}, phase ${engine.required_phase ?? 0}.`}
          >
            <div className="grid two-col">
              <div>
                <div className="telemetry-caption">Success rate</div>
                <div className="helper">
                  {engine.operations?.calculate === "supported" ? "supported" : "unsupported"}
                </div>
              </div>
              <div>
                <div className="telemetry-caption">Failure rate</div>
                <div className="helper">
                  {engine.dependency_observations?.length ?? 0} observations
                </div>
              </div>
            </div>
          </SurfaceCard>
        </>
      ) : null}
    </PageShell>
  );
}
