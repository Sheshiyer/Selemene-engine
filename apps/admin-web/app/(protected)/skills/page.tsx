"use client";

import { useCallback, useEffect, useState } from "react";
import { usePathname, useRouter, useSearchParams } from "next/navigation";
import { ActionRail, SurfaceCard } from "@/components/admin-primitives";
import { StateBanner, StatePanel } from "@/components/admin-state";
import { PageShell } from "@/components/page-shell";
import { getAuthToken } from "@/lib/auth";
import { ApiClientError, getAdminSkillsEcosystemStatus } from "@/lib/api";
import { statusPillClass } from "@/lib/status";
import { buildQueryString, getNumberParam } from "@/lib/url-query";
import type { AdminSkillsEcosystemStatus } from "@/types/admin";

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

export default function SkillsPage() {
  const router = useRouter();
  const pathname = usePathname();
  const searchParams = useSearchParams();

  const autoRefreshSec = getNumberParam(searchParams, "refresh", 0, 0, 60);

  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [lastUpdatedAt, setLastUpdatedAt] = useState<string | null>(null);

  const [ecosystem, setEcosystem] = useState<AdminSkillsEcosystemStatus | null>(null);

  const updateQuery = useCallback(
    (updates: { refresh?: number }) => {
      const nextQuery = buildQueryString(searchParams, {
        refresh: updates.refresh && updates.refresh > 0 ? updates.refresh : undefined
      });
      router.replace(nextQuery ? `${pathname}?${nextQuery}` : pathname, { scroll: false });
    },
    [pathname, router, searchParams]
  );

  const loadEcosystem = useCallback(async () => {
    const token = getAuthToken();
    if (!token) {
      throw new Error("Missing session token. Please sign in again.");
    }

    return getAdminSkillsEcosystemStatus(token);
  }, []);

  const handleFetch = useCallback(
    (promise: Promise<AdminSkillsEcosystemStatus>) => {
      setLoading(true);
      setError(null);
      promise
        .then((data) => {
          setEcosystem(data);
          setLastUpdatedAt(new Date().toISOString());
        })
        .catch((err) => {
          if (err instanceof ApiClientError) {
            setError(err.payload?.error || err.message);
          } else if (err instanceof Error) {
            setError(err.message);
          } else {
            setError("Failed to load skills ecosystem status");
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
      handleFetch(loadEcosystem());
    });
    return () => {
      cancelled = true;
    };
  }, [handleFetch, loadEcosystem]);

  useEffect(() => {
    if (autoRefreshSec <= 0) {
      return;
    }
    const interval = window.setInterval(() => {
      handleFetch(loadEcosystem());
    }, autoRefreshSec * 1000);
    return () => window.clearInterval(interval);
  }, [autoRefreshSec, handleFetch, loadEcosystem]);

  return (
    <PageShell
      title="Skills & Pipelines"
      summary="Skill cluster health, CodeGraph index, witness pipeline status, and bridge deployment posture."
      actions={
        <ActionRail label="Skills actions">
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
          <button type="button" onClick={() => handleFetch(loadEcosystem())}>
            Refresh
          </button>
        </ActionRail>
      }
    >
      <p className="helper">Last updated: {formatDateTime(lastUpdatedAt)}</p>

      {error ? <StateBanner variant="error" title={error} /> : null}

      {loading ? (
        <StatePanel
          variant="loading"
          title="Loading skills ecosystem status"
          description="Resolving cluster system health, CodeGraph index, witness pipeline, and bridge posture."
        />
      ) : (
        <>
          {/* Cluster System */}
          <SurfaceCard
            eyebrow="Infrastructure"
            title="Cluster System"
            summary="Skill cluster directory path, active cluster count, and health check availability."
          >
            <div className="grid metrics">
              <article className="metric">
                <div className="label">Path</div>
                <div className="value helper">{ecosystem?.cluster_system.path ?? "--"}</div>
              </article>
              <article className="metric">
                <div className="label">Active Clusters</div>
                <div className="value">{ecosystem?.cluster_system.active_clusters ?? "--"}</div>
              </article>
              <article className="metric">
                <div className="label">Health Available</div>
                <div className="value">
                  <span
                    className={statusPillClass(
                      ecosystem?.cluster_system.health_available ? "healthy" : "unavailable"
                    )}
                  >
                    {ecosystem?.cluster_system.health_available ? "Yes" : "No"}
                  </span>
                </div>
              </article>
              <article className="metric">
                <div className="label">Skills Indexed</div>
                <div className="value">{ecosystem?.skills_indexed ?? "--"}</div>
              </article>
            </div>
          </SurfaceCard>

          {/* CodeGraph Status */}
          <SurfaceCard
            eyebrow="Index"
            title="CodeGraph"
            summary="Structural code intelligence index readiness, file count, and total indexed symbols."
          >
            <div className="grid metrics">
              <article className="metric">
                <div className="label">Initialized</div>
                <div className="value">
                  <span
                    className={statusPillClass(
                      ecosystem?.codegraph_status?.initialized ? "healthy" : "unavailable"
                    )}
                  >
                    {ecosystem?.codegraph_status?.initialized ? "Yes" : "No"}
                  </span>
                </div>
              </article>
              <article className="metric">
                <div className="label">Files Indexed</div>
                <div className="value">{ecosystem?.codegraph_status?.files_indexed ?? "--"}</div>
              </article>
              <article className="metric">
                <div className="label">Symbols</div>
                <div className="value">{ecosystem?.codegraph_status?.symbols ?? "--"}</div>
              </article>
            </div>
          </SurfaceCard>

          {/* Witness Pipeline */}
          <SurfaceCard
            eyebrow="Pipeline"
            title="Witness Pipeline"
            summary="Witness pipeline version, available pattern count, and vector store posture."
          >
            <div className="grid metrics">
              <article className="metric">
                <div className="label">Version</div>
                <div className="value">{ecosystem?.witness_pipeline.version ?? "--"}</div>
              </article>
              <article className="metric">
                <div className="label">Pattern Count</div>
                <div className="value">{ecosystem?.witness_pipeline.pattern_count ?? "--"}</div>
              </article>
              <article className="metric">
                <div className="label">Vectors Available</div>
                <div className="value">
                  <span
                    className={statusPillClass(
                      ecosystem?.witness_pipeline.vectors_available ? "healthy" : "unavailable"
                    )}
                  >
                    {ecosystem?.witness_pipeline.vectors_available ? "Yes" : "No"}
                  </span>
                </div>
              </article>
            </div>
          </SurfaceCard>

          {/* Bridges Status */}
          <SurfaceCard
            eyebrow="Connectivity"
            title="Bridges Status"
            summary="Bridge integration posture across Hermes, Suno, LLM Proxy, universal tool server, and CLI tooling."
          >
            <div className="grid metrics">
              <article className="metric">
                <div className="label">Hermes</div>
                <div className="value">
                  <span
                    className={statusPillClass(
                      ecosystem?.bridges.hermes_configured ? "healthy" : "unavailable"
                    )}
                  >
                    {ecosystem?.bridges.hermes_configured ? "Configured" : "Missing"}
                  </span>
                </div>
              </article>
              <article className="metric">
                <div className="label">Suno</div>
                <div className="value">
                  <span
                    className={statusPillClass(
                      ecosystem?.bridges.suno_configured ? "healthy" : "unavailable"
                    )}
                  >
                    {ecosystem?.bridges.suno_configured ? "Configured" : "Missing"}
                  </span>
                </div>
              </article>
              <article className="metric">
                <div className="label">LLM Proxy</div>
                <div className="value">
                  <span
                    className={statusPillClass(
                      ecosystem?.bridges.llm_proxy_deployed ? "healthy" : "unavailable"
                    )}
                  >
                    {ecosystem?.bridges.llm_proxy_deployed ? "Deployed" : "Not Deployed"}
                  </span>
                </div>
              </article>
              <article className="metric">
                <div className="label">Universal Tool Server</div>
                <div className="value">
                  <span
                    className={statusPillClass(
                      ecosystem?.bridges.universal_tool_server ? "healthy" : "unavailable"
                    )}
                  >
                    {ecosystem?.bridges.universal_tool_server ? "Yes" : "No"}
                  </span>
                </div>
              </article>
              <article className="metric">
                <div className="label">Bridge CLI</div>
                <div className="value">
                  <span
                    className={statusPillClass(
                      ecosystem?.bridges.bridge_cli_installed ? "healthy" : "unavailable"
                    )}
                  >
                    {ecosystem?.bridges.bridge_cli_installed ? "Installed" : "Missing"}
                  </span>
                </div>
              </article>
            </div>
          </SurfaceCard>
        </>
      )}
    </PageShell>
  );
}