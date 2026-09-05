"use client";

import { use, useCallback, useEffect, useState } from "react";
import { useRouter } from "next/navigation";
import {
  Bar,
  BarChart,
  CartesianGrid,
  ResponsiveContainer,
  Tooltip,
  XAxis,
  YAxis
} from "recharts";
import { ActionRail, MetricSurface, SurfaceCard } from "@/components/admin-primitives";
import { StateBanner, StatePanel } from "@/components/admin-state";
import { PageShell } from "@/components/page-shell";
import { getAuthToken } from "@/lib/auth";
import { ApiClientError, getAdminWorkflow, getSystemWorkflows } from "@/lib/api";
import { statusPillClass } from "@/lib/status";
import type { AdminSystemWorkflowItem } from "@/types/admin";

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

export default function WorkflowDetailPage({ params }: { params: Promise<{ id: string }> }) {
  const router = useRouter();
  const { id } = use(params);

  const [fetchResult, setFetchResult] = useState<{
    key: string;
    workflow: AdminSystemWorkflowItem | null;
    error: string | null;
    settled: boolean;
  }>({ key: "", workflow: null, error: null, settled: false });

  const loading = !fetchResult.settled || fetchResult.key !== id;
  const error = fetchResult.key === id ? fetchResult.error : null;
  const workflow = fetchResult.key === id ? fetchResult.workflow : null;

  const loadDetail = useCallback(async () => {
    const token = getAuthToken() ?? undefined;

    try {
      const result = await getAdminWorkflow(token, id);
      return result;
    } catch (detailErr) {
      const workflowsResponse = await getSystemWorkflows(token, {
        window_hours: 168,
        limit: 200,
        offset: 0
      });
      const match = workflowsResponse.items.find((w) => w.workflow_id === id);
      if (!match) {
        throw detailErr;
      }
      return match;
    }
  }, [id]);

  useEffect(() => {
    let cancelled = false;

    loadDetail()
      .then((data) => {
        if (!cancelled) {
          setFetchResult({ key: id, workflow: data, error: null, settled: true });
        }
      })
      .catch((err) => {
        if (!cancelled) {
          let message = "Failed to load workflow detail";
          if (err instanceof ApiClientError) {
            message = err.payload?.error || err.message;
          } else if (err instanceof Error) {
            message = err.message;
          }
          setFetchResult({ key: id, workflow: null, error: message, settled: true });
        }
      });

    return () => {
      cancelled = true;
    };
  }, [id, loadDetail]);

  const runChartData =
    workflow
      ? [
          {
            label: "Recent",
            runs: workflow.recent_runs,
            failures: workflow.failure_runs
          }
        ]
      : [];

  return (
    <PageShell
      title={workflow ? workflow.name : `Workflow ${id.slice(0, 8)}…`}
      summary="Full workflow detail including synthesis type, engine composition, cache stats, and run metrics."
    >
      <ActionRail>
        <button type="button" onClick={() => router.push("/workflows")}>
          &larr; Back to workflows
        </button>
      </ActionRail>

      {error ? <StateBanner variant="error" title={error} /> : null}

      {loading ? (
        <StatePanel
          variant="loading"
          title="Loading workflow detail"
          description="Resolving synthesis type, engine composition, cache posture, and run metrics."
        />
      ) : workflow ? (
        <div className="grid detail-section-grid">
          <SurfaceCard
            eyebrow="Identity"
            title={workflow.name}
            summary="Workflow definition and lifecycle posture."
          >
            <div className="grid overlay-detail-grid">
              <div className="helper">ID: {workflow.workflow_id}</div>
              <div className="helper">
                Status: <span className={statusPillClass(workflow.status)}>{workflow.status}</span>
              </div>
              <div className="helper">Last seen: {formatDateTime(workflow.last_seen_at)}</div>
            </div>
          </SurfaceCard>

          <div className="grid metrics">
            <MetricSurface
              label="Engine count"
              value={workflow.engine_count}
              detail="Consciousness engines wired into this workflow."
            />
            <MetricSurface
              label="Recent runs"
              value={workflow.recent_runs}
              detail="Total executions in the selected window."
            />
            <MetricSurface
              label="Failures"
              value={workflow.failure_runs}
              detail="Failed executions in the selected window."
            />
          </div>

          {workflow.synthesis_type ? (
            <SurfaceCard
              eyebrow="Synthesis"
              title="Synthesis configuration"
              summary="Orchestrated synthesis strategy and runtime constraints."
            >
              <div className="grid overlay-detail-grid">
                <div className="helper">
                  Type: <span className="permission-chip">{workflow.synthesis_type}</span>
                </div>
                {workflow.required_phase !== undefined ? (
                  <div className="helper">Required phase: {workflow.required_phase}</div>
                ) : null}
              </div>
            </SurfaceCard>
          ) : null}

          <SurfaceCard
            eyebrow="Engines"
            title="Engine composition"
            summary="Consciousness engines allocated to this workflow execution path."
          >
            <div className="grid overlay-detail-grid">
              {workflow.engine_ids && workflow.engine_ids.length > 0 ? (
                <div className="table-chip-row">
                  {workflow.engine_ids.map((engineId) => (
                    <span key={engineId} className="permission-chip">
                      {engineId}
                    </span>
                  ))}
                </div>
              ) : (
                <div className="helper">No engine list available.</div>
              )}
            </div>
          </SurfaceCard>

          <SurfaceCard
            eyebrow="Cache"
            title="Cache posture"
            summary="L1 in-memory cache metrics for this workflow."
          >
            <div className="grid overlay-detail-grid">
              <div className="helper">
                Cache hits: {workflow.cache_hits ?? "--"}
              </div>
              <div className="helper">
                Cache entries: {workflow.cache_entries ?? "--"}
              </div>
            </div>
          </SurfaceCard>

          <SurfaceCard
            eyebrow="Runs"
            title="Run metrics"
            summary="Recent runs vs failures for the current observation window."
          >
            <ResponsiveContainer width="100%" height={200}>
              <BarChart data={runChartData}>
                <CartesianGrid strokeDasharray="3 3" />
                <XAxis dataKey="label" />
                <YAxis allowDecimals={false} />
                <Tooltip />
                <Bar dataKey="runs" fill="#56d3c2" name="Recent runs" />
                <Bar dataKey="failures" fill="#ef6b73" name="Failures" />
              </BarChart>
            </ResponsiveContainer>
          </SurfaceCard>
        </div>
      ) : (
        <StatePanel
          variant="empty"
          title="Workflow not found"
          description="The requested workflow does not exist or could not be resolved."
        />
      )}
    </PageShell>
  );
}