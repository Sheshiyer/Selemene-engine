"use client";

import { useCallback, useEffect, useState } from "react";
import { usePathname, useRouter, useSearchParams } from "next/navigation";
import { ActionRail, SurfaceCard } from "@/components/admin-primitives";
import { StateBanner, StatePanel, TableEmptyStateRow } from "@/components/admin-state";
import { PageShell } from "@/components/page-shell";
import { getAuthToken } from "@/lib/auth";
import {
  ApiClientError,
  getAdminBridgeHealth,
  getAdminHermesBridgeStatus,
  getAdminSunoBridgeStatus,
  getAdminLlmProxyStatus
} from "@/lib/api";
import { statusPillClass } from "@/lib/status";
import { buildQueryString, getNumberParam } from "@/lib/url-query";
import type {
  AdminBridgeHealthResponse,
  AdminHermesBridgeStatus,
  AdminSunoBridgeStatus,
  AdminLlmProxyStatus
} from "@/types/admin";

const REFRESH_OPTIONS = [0, 15, 30, 60] as const;

function formatDateTime(value: string | null): string {
  if (!value) {
    return "--";
  }
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) {
    return "--";
  }
  return date.toLocaleString();
}

function circuitPillClass(state: string): string {
  switch (state) {
    case "closed":
      return "pill ok";
    case "open":
      return "pill danger";
    case "half_open":
      return "pill warn";
    default:
      return "pill";
  }
}

export default function BridgePage() {
  const router = useRouter();
  const pathname = usePathname();
  const searchParams = useSearchParams();

  const autoRefreshSec = getNumberParam(searchParams, "refresh", 0, 0, 60);

  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [lastUpdatedAt, setLastUpdatedAt] = useState<string | null>(null);

  const [bridgeHealth, setBridgeHealth] = useState<AdminBridgeHealthResponse | null>(null);
  const [hermesStatus, setHermesStatus] = useState<AdminHermesBridgeStatus | null>(null);
  const [sunoStatus, setSunoStatus] = useState<AdminSunoBridgeStatus | null>(null);
  const [llmProxyStatus, setLlmProxyStatus] = useState<AdminLlmProxyStatus | null>(null);

  const updateQuery = useCallback(
    (updates: { refresh?: number }) => {
      const nextQuery = buildQueryString(searchParams, {
        refresh: updates.refresh && updates.refresh > 0 ? updates.refresh : undefined
      });
      router.replace(nextQuery ? `${pathname}?${nextQuery}` : pathname, { scroll: false });
    },
    [pathname, router, searchParams]
  );

  const loadBridge = useCallback(async () => {
    const token = getAuthToken() ?? undefined;

    const [bridgeResponse, hermesResponse, sunoResponse, llmProxyResponse] = await Promise.all([
      getAdminBridgeHealth(token),
      getAdminHermesBridgeStatus(token),
      getAdminSunoBridgeStatus(token),
      getAdminLlmProxyStatus(token)
    ]);

    return { bridgeResponse, hermesResponse, sunoResponse, llmProxyResponse };
  }, []);

  const handleFetch = useCallback(
    (
      promise: Promise<{
        bridgeResponse: AdminBridgeHealthResponse;
        hermesResponse: AdminHermesBridgeStatus;
        sunoResponse: AdminSunoBridgeStatus;
        llmProxyResponse: AdminLlmProxyStatus;
      }>
    ) => {
      setLoading(true);
      setError(null);
      promise
        .then(({ bridgeResponse, hermesResponse, sunoResponse, llmProxyResponse }) => {
          setBridgeHealth(bridgeResponse);
          setHermesStatus(hermesResponse);
          setSunoStatus(sunoResponse);
          setLlmProxyStatus(llmProxyResponse);
          setLastUpdatedAt(new Date().toISOString());
        })
        .catch((err) => {
          if (err instanceof ApiClientError) {
            setError(err.payload?.error || err.message);
          } else if (err instanceof Error) {
            setError(err.message);
          } else {
            setError("Failed to load bridge health data");
          }
        })
        .finally(() => {
          setLoading(false);
        });
    },
    []
  );

  useEffect(() => {
    let cancelled = false;
    queueMicrotask(() => {
      if (cancelled) return;
      handleFetch(loadBridge());
    });
    return () => {
      cancelled = true;
    };
  }, [handleFetch, loadBridge]);

  useEffect(() => {
    if (autoRefreshSec <= 0) {
      return;
    }
    const interval = window.setInterval(() => {
      handleFetch(loadBridge());
    }, autoRefreshSec * 1000);
    return () => window.clearInterval(interval);
  }, [autoRefreshSec, handleFetch, loadBridge]);

  return (
    <PageShell
      title="Bridge Health"
      summary="Sidecar readiness, circuit breaker states, and external bridge connectivity across all service boundaries."
      actions={
        <ActionRail label="Bridge actions">
          <label>
            <span className="sr-only">Auto refresh</span>
            <select
              value={autoRefreshSec}
              onChange={(event) =>
                updateQuery({ refresh: Number.parseInt(event.target.value, 10) })
              }
            >
              {REFRESH_OPTIONS.map((value) => (
                <option key={value} value={value}>
                  {value === 0 ? "Auto refresh: Off" : `Auto refresh: ${value}s`}
                </option>
              ))}
            </select>
          </label>
          <button type="button" onClick={() => handleFetch(loadBridge())}>
            Refresh
          </button>
        </ActionRail>
      }
    >
      <p className="helper">Last updated: {formatDateTime(lastUpdatedAt)}</p>

      {error ? <StateBanner variant="error" title={error} /> : null}

      {/* Bridge Overview */}
      <SurfaceCard
        eyebrow="Infrastructure"
        title="Bridge Overview"
        summary="Sidecar connectivity posture, overall engine health, and current circuit configuration."
      >
        <div className="grid metrics">
          <article className="metric">
            <div className="label">Base URL</div>
            <div className="value helper">{bridgeHealth?.base_url ?? "--"}</div>
          </article>
          <article className="metric">
            <div className="label">Overall Status</div>
            <div className="value">
              {bridgeHealth ? (
                <span className={statusPillClass(bridgeHealth.overall_status)}>
                  {bridgeHealth.overall_status}
                </span>
              ) : (
                "--"
              )}
            </div>
          </article>
          <article className="metric">
            <div className="label">Sidecar Reachable</div>
            <div className="value">
              {bridgeHealth ? (
                <span className={statusPillClass(bridgeHealth.sidecar_reachable ? "healthy" : "unavailable")}>
                  {bridgeHealth.sidecar_reachable ? "Yes" : "No"}
                </span>
              ) : (
                "--"
              )}
            </div>
          </article>
          <article className="metric">
            <div className="label">Timeout</div>
            <div className="value">{bridgeHealth ? `${bridgeHealth.config.timeout_secs}s` : "--"}</div>
          </article>
          <article className="metric">
            <div className="label">CB Threshold</div>
            <div className="value">{bridgeHealth?.config.cb_threshold ?? "--"}</div>
          </article>
          <article className="metric">
            <div className="label">CB Reset</div>
            <div className="value">{bridgeHealth ? `${bridgeHealth.config.cb_reset_secs}s` : "--"}</div>
          </article>
          <article className="metric">
            <div className="label">Total Engines</div>
            <div className="value">{bridgeHealth?.total_engines ?? "--"}</div>
          </article>
          <article className="metric">
            <div className="label">Healthy / Degraded</div>
            <div className="value">
              {bridgeHealth ? `${bridgeHealth.healthy_engines} / ${bridgeHealth.degraded_engines}` : "--"}
            </div>
          </article>
        </div>
      </SurfaceCard>

      {loading ? (
        <StatePanel
          variant="loading"
          title="Loading bridge telemetry"
          description="Resolving sidecar engine health, circuit breaker states, and bridge connectivity checks."
        />
      ) : (
        <>
          {/* TS Sidecar Engines Table */}
          <article className="panel">
            <h3>TS Sidecar Engines</h3>
            <div className="table-wrap compact">
              <table>
                <thead>
                  <tr>
                    <th>Engine ID</th>
                    <th>Name</th>
                    <th>Healthy</th>
                    <th>Detail</th>
                    <th>Latency (ms)</th>
                    <th>Circuit</th>
                    <th>Failures</th>
                    <th>Phase</th>
                  </tr>
                </thead>
                <tbody>
                  {bridgeHealth?.engines.map((engine) => (
                    <tr key={engine.engine_id}>
                      <td>
                        <div className="table-primary">{engine.engine_id}</div>
                      </td>
                      <td>{engine.engine_name}</td>
                      <td>
                        <span className={statusPillClass(engine.healthy ? "healthy" : "unavailable")}>
                          {engine.healthy ? "healthy" : "degraded"}
                        </span>
                      </td>
                      <td className="cell-wrap">{engine.detail}</td>
                      <td>{engine.latency_ms.toFixed(1)}</td>
                      <td>
                        <span className={circuitPillClass(engine.circuit_state)}>
                          {engine.circuit_state}
                        </span>
                      </td>
                      <td>{engine.circuit_failures}</td>
                      <td>{engine.required_phase}</td>
                    </tr>
                  ))}
                  {(!bridgeHealth || bridgeHealth.engines.length === 0) ? (
                    <TableEmptyStateRow
                      colSpan={8}
                      title="No engine health data"
                      description="Sidecar engine health snapshots will appear after bridge discovery completes."
                    />
                  ) : null}
                </tbody>
              </table>
            </div>
          </article>

          {/* Hermes Bridge Card */}
          <SurfaceCard
            eyebrow="AI Bridge"
            title="Hermes Bridge"
            summary="LLM gateway connectivity, model routing, tool surface, and Noesis API key posture."
          >
            <div className="grid metrics">
              <article className="metric">
                <div className="label">Configured</div>
                <div className="value">
                  <span className={statusPillClass(hermesStatus?.configured ? "healthy" : "unavailable")}>
                    {hermesStatus?.configured ? "Yes" : "No"}
                  </span>
                </div>
              </article>
              <article className="metric">
                <div className="label">Base URL</div>
                <div className="value helper">{hermesStatus?.base_url ?? "--"}</div>
              </article>
              <article className="metric">
                <div className="label">Model</div>
                <div className="value">{hermesStatus?.model ?? "--"}</div>
              </article>
              <article className="metric">
                <div className="label">Mode</div>
                <div className="value">{hermesStatus?.mode ?? "--"}</div>
              </article>
              <article className="metric">
                <div className="label">Noesis Key</div>
                <div className="value">
                  <span className={statusPillClass(hermesStatus?.noesis_api_key_configured ? "healthy" : "unavailable")}>
                    {hermesStatus?.noesis_api_key_configured ? "Configured" : "Missing"}
                  </span>
                </div>
              </article>
              <article className="metric">
                <div className="label">Tools Available</div>
                <div className="value">{hermesStatus?.tools_available ?? "--"}</div>
              </article>
            </div>
            {hermesStatus?.health ? (
              <div className="grid metrics">
                <article className="metric">
                  <div className="label">Health Status</div>
                  <div className="value">
                    <span className={statusPillClass(hermesStatus.health.status)}>
                      {hermesStatus.health.status}
                    </span>
                  </div>
                </article>
                <article className="metric">
                  <div className="label">Model Reachable</div>
                  <div className="value">
                    <span className={statusPillClass(hermesStatus.health.model_reachable ? "healthy" : "unavailable")}>
                      {hermesStatus.health.model_reachable ? "Yes" : "No"}
                    </span>
                  </div>
                </article>
                <article className="metric">
                  <div className="label">Noesis Reachable</div>
                  <div className="value">
                    <span className={statusPillClass(hermesStatus.health.noesis_reachable ? "healthy" : "unavailable")}>
                      {hermesStatus.health.noesis_reachable ? "Yes" : "No"}
                    </span>
                  </div>
                </article>
                <article className="metric">
                  <div className="label">Latency (ms)</div>
                  <div className="value">{hermesStatus.health.latency_ms.toFixed(1)}</div>
                </article>
              </div>
            ) : null}
          </SurfaceCard>

          {/* Suno Bridge Card */}
          <SurfaceCard
            eyebrow="Audio Bridge"
            title="Suno Bridge"
            summary="Music generation gateway connectivity, credit posture, and service reachability."
          >
            <div className="grid metrics">
              <article className="metric">
                <div className="label">Configured</div>
                <div className="value">
                  <span className={statusPillClass(sunoStatus?.configured ? "healthy" : "unavailable")}>
                    {sunoStatus?.configured ? "Yes" : "No"}
                  </span>
                </div>
              </article>
              <article className="metric">
                <div className="label">Base URL</div>
                <div className="value helper">{sunoStatus?.base_url ?? "--"}</div>
              </article>
              {sunoStatus?.health ? (
                <>
                  <article className="metric">
                    <div className="label">Health Status</div>
                    <div className="value">
                      <span className={statusPillClass(sunoStatus.health.status)}>
                        {sunoStatus.health.status}
                      </span>
                    </div>
                  </article>
                  <article className="metric">
                    <div className="label">Reachable</div>
                    <div className="value">
                      <span className={statusPillClass(sunoStatus.health.reachable ? "healthy" : "unavailable")}>
                        {sunoStatus.health.reachable ? "Yes" : "No"}
                      </span>
                    </div>
                  </article>
                </>
              ) : null}
              <article className="metric">
                <div className="label">Credit Info</div>
                <div className="value">
                  {sunoStatus?.credit_info ? "Available" : "--"}
                </div>
              </article>
            </div>
          </SurfaceCard>

          {/* LLM Proxy Card */}
          <SurfaceCard
            eyebrow="Proxy"
            title="LLM Proxy"
            summary="Deployed LLM proxy endpoint, active provider, and provider fleet readiness."
          >
            <div className="grid metrics">
              <article className="metric">
                <div className="label">Deployed</div>
                <div className="value">
                  <span className={statusPillClass(llmProxyStatus?.deployed ? "healthy" : "unavailable")}>
                    {llmProxyStatus?.deployed ? "Yes" : "No"}
                  </span>
                </div>
              </article>
              <article className="metric">
                <div className="label">Endpoint</div>
                <div className="value helper">{llmProxyStatus?.endpoint ?? "--"}</div>
              </article>
              <article className="metric">
                <div className="label">Active Provider</div>
                <div className="value">{llmProxyStatus?.active_provider ?? "--"}</div>
              </article>
              <article className="metric">
                <div className="label">Health Status</div>
                <div className="value">
                  {llmProxyStatus?.health ? (
                    <span className={statusPillClass(llmProxyStatus.health.status)}>
                      {llmProxyStatus.health.status}
                    </span>
                  ) : (
                    "--"
                  )}
                </div>
              </article>
              <article className="metric">
                <div className="label">Reachable</div>
                <div className="value">
                  {llmProxyStatus?.health ? (
                    <span className={statusPillClass(llmProxyStatus.health.reachable ? "healthy" : "unavailable")}>
                      {llmProxyStatus.health.reachable ? "Yes" : "No"}
                    </span>
                  ) : (
                    "--"
                  )}
                </div>
              </article>
            </div>
            {llmProxyStatus?.providers && llmProxyStatus.providers.length > 0 ? (
              <div className="grid metrics">
                <article className="metric">
                  <div className="label">Provider Fleet</div>
                  <div className="value">{llmProxyStatus.providers.join(", ")}</div>
                </article>
              </div>
            ) : null}
          </SurfaceCard>
        </>
      )}
    </PageShell>
  );
}