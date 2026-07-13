"use client";

import { useCallback, useEffect, useState } from "react";
import { usePathname, useRouter, useSearchParams } from "next/navigation";
import { ActionRail, SurfaceCard } from "@/components/admin-primitives";
import { StateBanner, StatePanel, TableEmptyStateRow } from "@/components/admin-state";
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
    const token = getAuthToken() ?? undefined;

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
      title="Selemene Skills & Pipelines"
      summary="Selemene skills, report modes, autoresearch testing, Vectorize pattern memory, and MCP server posture."
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
          title="Loading Selemene skills ecosystem"
          description="Resolving Selemene skills, report modes, autoresearch, Vectorize bindings, and MCP status."
        />
      ) : (
        <>
          {/* Selemene Skills */}
          <SurfaceCard
            eyebrow="Skills"
            title="Selemene Skills"
            summary="Active Selemene skills from project-local .claude/skills and the ~/.agents skills cluster."
          >
            <div className="table-wrap">
              <table>
                <thead>
                  <tr>
                    <th>Name</th>
                    <th>Origin</th>
                    <th>Location</th>
                    <th>Status</th>
                    <th>Description</th>
                  </tr>
                </thead>
                <tbody>
                  {ecosystem?.selemene_skills.map((skill) => (
                    <tr key={skill.name}>
                      <td>
                        <div className="table-primary">{skill.name}</div>
                      </td>
                      <td>
                        <span className={statusPillClass(skill.origin)}>{skill.origin}</span>
                      </td>
                      <td>
                        <div className="helper">{skill.location}</div>
                      </td>
                      <td>
                        <span className={statusPillClass(skill.status)}>{skill.status}</span>
                      </td>
                      <td>
                        <div className="helper">{skill.description}</div>
                      </td>
                    </tr>
                  ))}
                  {ecosystem?.selemene_skills.length === 0 ? (
                    <TableEmptyStateRow
                      colSpan={5}
                      title="No Selemene skills found"
                      description="Check the skills cluster and .claude/skills directory."
                    />
                  ) : null}
                </tbody>
              </table>
            </div>
          </SurfaceCard>

          {/* Autoresearch */}
          <SurfaceCard
            eyebrow="Testing"
            title="Autoresearch"
            summary="Autoresearch loop status and testing grounds for skills, prompts, and agent evaluation."
          >
            <div className="grid metrics">
              <article className="metric">
                <div className="label">Enabled</div>
                <div className="value">
                  <span
                    className={statusPillClass(
                      ecosystem?.autoresearch.enabled ? "healthy" : "unavailable"
                    )}
                  >
                    {ecosystem?.autoresearch.enabled ? "Yes" : "No"}
                  </span>
                </div>
              </article>
              <article className="metric">
                <div className="label">Testing Grounds</div>
                <div className="value">{ecosystem?.autoresearch.testing_grounds ?? "--"}</div>
              </article>
              <article className="metric wide">
                <div className="label">Description</div>
                <div className="value helper">{ecosystem?.autoresearch.description ?? "--"}</div>
              </article>
            </div>
          </SurfaceCard>

          {/* Vectorize */}
          <SurfaceCard
            eyebrow="Pattern Memory"
            title="Vectorize Connection"
            summary="Cloudflare Vectorize binding, AI binding, and durable storage for report pattern memory."
          >
            <div className="grid metrics">
              <article className="metric">
                <div className="label">Status</div>
                <div className="value">
                  <span
                    className={statusPillClass(
                      ecosystem?.vectorize.status === "configured" ? "healthy" : "unavailable"
                    )}
                  >
                    {ecosystem?.vectorize.status ?? "--"}
                  </span>
                </div>
              </article>
              <article className="metric">
                <div className="label">Index Name</div>
                <div className="value helper">{ecosystem?.vectorize.index_name ?? "--"}</div>
              </article>
              <article className="metric">
                <div className="label">Vectorize Binding</div>
                <div className="value helper">{ecosystem?.vectorize.binding ?? "--"}</div>
              </article>
              <article className="metric">
                <div className="label">AI Binding</div>
                <div className="value helper">{ecosystem?.vectorize.ai_binding ?? "--"}</div>
              </article>
              <article className="metric">
                <div className="label">R2 Bucket</div>
                <div className="value helper">{ecosystem?.vectorize.r2_bucket ?? "--"}</div>
              </article>
              <article className="metric">
                <div className="label">D1 Database</div>
                <div className="value helper">{ecosystem?.vectorize.d1_database ?? "--"}</div>
              </article>
            </div>
          </SurfaceCard>

          {/* Report Modes */}
          <SurfaceCard
            eyebrow="Witness Pipeline"
            title="Report Modes"
            summary="Available witness-pipeline report modes, levels, subject counts, and pass architectures."
          >
            <div className="table-wrap">
              <table>
                <thead>
                  <tr>
                    <th>Mode</th>
                    <th>Level</th>
                    <th>Subjects</th>
                    <th>Roles</th>
                    <th>Target Words</th>
                    <th>Architecture</th>
                    <th>Passes</th>
                  </tr>
                </thead>
                <tbody>
                  {ecosystem?.report_modes.map((mode) => (
                    <tr key={mode.mode}>
                      <td>
                        <div className="table-primary">{mode.mode}</div>
                      </td>
                      <td>
                        <span className={statusPillClass(`level-${mode.report_level}`)}>
                          {mode.report_level}
                        </span>
                      </td>
                      <td>
                        {mode.subject_count_min === mode.subject_count_max
                          ? mode.subject_count_min
                          : `${mode.subject_count_min}–${mode.subject_count_max}`}
                      </td>
                      <td>
                        <div className="helper">{mode.roles.join(", ")}</div>
                      </td>
                      <td>
                        {mode.target_words_min.toLocaleString()}–
                        {mode.target_words_max.toLocaleString()}
                      </td>
                      <td>
                        <span className={statusPillClass(mode.architecture)}>
                          {mode.architecture}
                        </span>
                      </td>
                      <td>{mode.pass_count}</td>
                    </tr>
                  ))}
                  {ecosystem?.report_modes.length === 0 ? (
                    <TableEmptyStateRow
                      colSpan={7}
                      title="No report modes found"
                      description="Check packages/witness-pipeline/modes for mode markdown files."
                    />
                  ) : null}
                </tbody>
              </table>
            </div>
          </SurfaceCard>

          {/* MCP Server */}
          <SurfaceCard
            eyebrow="Integration"
            title="Selemene MCP Server"
            summary="MCP server base URL, authentication methods, and exposed tool surface."
          >
            <div className="grid metrics">
              <article className="metric">
                <div className="label">Configured</div>
                <div className="value">
                  <span
                    className={statusPillClass(
                      ecosystem?.mcp.configured ? "healthy" : "unavailable"
                    )}
                  >
                    {ecosystem?.mcp.configured ? "Yes" : "No"}
                  </span>
                </div>
              </article>
              <article className="metric wide">
                <div className="label">Base URL</div>
                <div className="value helper">{ecosystem?.mcp.base_url ?? "--"}</div>
              </article>
              <article className="metric">
                <div className="label">Auth Methods</div>
                <div className="value helper">
                  {ecosystem?.mcp.auth_methods.join(", ") ?? "--"}
                </div>
              </article>
              <article className="metric">
                <div className="label">Tool Count</div>
                <div className="value">{ecosystem?.mcp.tool_count ?? "--"}</div>
              </article>
              <article className="metric wide">
                <div className="label">Tools</div>
                <div className="value helper">{ecosystem?.mcp.tools.join(", ") ?? "--"}</div>
              </article>
            </div>
          </SurfaceCard>
        </>
      )}
    </PageShell>
  );
}
