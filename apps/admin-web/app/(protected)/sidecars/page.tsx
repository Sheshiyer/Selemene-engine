"use client";

import { useEffect, useState } from "react";
import { MetricSurface, SurfaceCard } from "@/components/admin-primitives";
import { StateBanner, StatePanel } from "@/components/admin-state";
import { PageShell } from "@/components/page-shell";
import { getAuthToken } from "@/lib/auth";
import { ApiClientError, getAdminSidecarDetail } from "@/lib/api";
import { statusPillClass } from "@/lib/status";
import type { AdminSidecarDetail } from "@/types/admin";

export default function SidecarsPage() {
  const sidecarRequestKey = "sidecar-detail";

  const [fetchResult, setFetchResult] = useState<{
    key: string;
    sidecar: AdminSidecarDetail | null;
    error: string | null;
    settled: boolean;
  }>({ key: "", sidecar: null, error: null, settled: false });

  const loading = fetchResult.key !== sidecarRequestKey || !fetchResult.settled;
  const error = fetchResult.key === sidecarRequestKey ? fetchResult.error : null;
  const sidecar = fetchResult.key === sidecarRequestKey ? fetchResult.sidecar : null;

  useEffect(() => {
    let cancelled = false;
    const token = getAuthToken() ?? undefined;

    getAdminSidecarDetail(token)
      .then((data) => {
        if (!cancelled) {
          setFetchResult({ key: sidecarRequestKey, sidecar: data, error: null, settled: true });
        }
      })
      .catch((err) => {
        if (!cancelled) {
          let message = "Failed to load sidecar health data";
          if (err instanceof ApiClientError) {
            message = err.payload?.error || err.message;
          } else if (err instanceof Error) {
            message = err.message;
          }
          setFetchResult({ key: sidecarRequestKey, sidecar: null, error: message, settled: true });
        }
      });

    return () => {
      cancelled = true;
    };
  }, [sidecarRequestKey]);

  const healthyEngines = sidecar
    ? sidecar.engines.filter((e) => e.healthy).length
    : 0;
  const unhealthyEngines = sidecar
    ? sidecar.engines.length - healthyEngines
    : 0;

  return (
    <PageShell
      title="Sidecar Health"
      summary="Monitor TypeScript engine server and Python biofield CV service health with per-engine detail."
    >
      {error ? <StateBanner variant="error" title={error} /> : null}

      {loading ? (
        <StatePanel
          variant="loading"
          title="Loading sidecar health"
          description="Probing base URLs, engine health, circuit breaker state, and latency posture."
        />
      ) : sidecar ? (
        <>
          <div className="grid metrics">
            <article className="metric">
              <div className="label">Sidecar status</div>
              <div className="value">
                <span className={statusPillClass(sidecar.status)}>{sidecar.status}</span>
              </div>
            </article>
            <article className="metric">
              <div className="label">Base URL</div>
              <div className="value helper" style={{ fontSize: "0.825rem" }}>
                {sidecar.base_url}
              </div>
            </article>
            <article className="metric">
              <div className="label">Healthy engines</div>
              <div className="value">{healthyEngines}</div>
            </article>
            <article className="metric">
              <div className="label">Failed engines</div>
              <div className="value">{unhealthyEngines}</div>
            </article>
          </div>

          <SurfaceCard
            eyebrow="Runtime"
            title="Engine health overview"
            summary="Per-engine health status, detail message, and observed latency for the current sidecar instance."
          >
            <div className="table-wrap compact">
              <table>
                <thead>
                  <tr>
                    <th>Engine ID</th>
                    <th>Health</th>
                    <th>Latency (ms)</th>
                    <th>Detail</th>
                  </tr>
                </thead>
                <tbody>
                  {sidecar.engines.map((engine) => (
                    <tr key={engine.engine_id}>
                      <td>
                        <div className="table-primary">{engine.engine_id}</div>
                      </td>
                      <td>
                        <span
                          className={engine.healthy ? "pill ok" : "pill danger"}
                        >
                          {engine.healthy ? "healthy" : "unhealthy"}
                        </span>
                      </td>
                      <td>{engine.latency_ms.toFixed(1)}</td>
                      <td className="cell-wrap">{engine.detail}</td>
                    </tr>
                  ))}
                  {sidecar.engines.length === 0 ? (
                    <tr>
                      <td colSpan={4}>
                        <div className="state-table-empty">
                          <div className="telemetry-caption">Empty</div>
                          <div className="state-table-empty-title">No engine health data</div>
                          <div className="helper">No engine-level telemetry is available from the sidecar.</div>
                        </div>
                      </td>
                    </tr>
                  ) : null}
                </tbody>
              </table>
            </div>
          </SurfaceCard>

          {sidecar.circuit_breakers && Object.keys(sidecar.circuit_breakers).length > 0 ? (
            <SurfaceCard
              eyebrow="Resilience"
              title="Circuit breakers"
              summary="Active circuit breaker state for engine-to-sidecar communication channels."
            >
              <div className="table-wrap compact">
                <table>
                  <thead>
                    <tr>
                      <th>Engine</th>
                      <th>State</th>
                      <th>Failures</th>
                      <th>Last failure</th>
                    </tr>
                  </thead>
                  <tbody>
                    {Object.entries(sidecar.circuit_breakers).map(([engineId, breaker]) => (
                      <tr key={engineId}>
                        <td>
                          <div className="table-primary">{engineId}</div>
                        </td>
                        <td>
                          <span className={statusPillClass(breaker.state)}>
                            {breaker.state}
                          </span>
                        </td>
                        <td>{breaker.failures}</td>
                        <td>
                          {breaker.last_failure_ts
                            ? new Date(breaker.last_failure_ts).toLocaleString()
                            : "never"}
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            </SurfaceCard>
          ) : null}

          {sidecar.failed_engines.length > 0 ? (
            <SurfaceCard
              eyebrow="Failures"
              title="Failed engine IDs"
              summary="Engine identifiers that are currently reporting unhealthy status from the sidecar probe."
            >
              <div className="table-chip-row">
                {sidecar.failed_engines.map((engineId) => (
                  <span key={engineId} className="permission-chip">
                    {engineId}
                  </span>
                ))}
              </div>
            </SurfaceCard>
          ) : null}
        </>
      ) : (
        <StatePanel
          variant="empty"
          title="No sidecar data"
          description="The sidecar health endpoint returned no data or is unreachable."
        />
      )}
    </PageShell>
  );
}