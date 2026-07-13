"use client";

import { useCallback, useEffect, useState } from "react";
import { useParams } from "next/navigation";
import Link from "next/link";
import { ActionRail, MetricSurface, SurfaceCard } from "@/components/admin-primitives";
import { StateBanner, StatePanel } from "@/components/admin-state";
import { PageShell } from "@/components/page-shell";
import { getAuthToken } from "@/lib/auth";
import { ApiClientError, getWitnessDyadExecution } from "@/lib/api";
import { statusPillClass } from "@/lib/status";
import type { AdminWitnessDyadExecutionItem } from "@/types/admin";

function formatDateTime(value: string | null): string {
  if (!value) return "--";
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return "--";
  return date.toLocaleString();
}

function formatMs(value: number | null): string {
  if (value === null) return "--";
  return `${value.toLocaleString()} ms`;
}

export default function WitnessDyadDetailPage() {
  const params = useParams<{ id: string }>();
  const id = params?.id;

  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [execution, setExecution] = useState<AdminWitnessDyadExecutionItem | null>(null);

  const loadData = useCallback(async () => {
    const token = getAuthToken();
    if (!token) throw new Error("Missing session token. Please sign in again.");
    if (!id) throw new Error("Missing execution ID.");
    return getWitnessDyadExecution(token, id);
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
          setExecution(result);
          setError(null);
        }
      } catch (err) {
        if (!cancelled) {
          setError(
            err instanceof ApiClientError
              ? err.payload?.error || err.message
              : "Failed to load execution detail"
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
      title="Witness Dyad Detail"
      summary={`Execution ${id ?? "--"}`}
      actions={
        <ActionRail label="Detail actions">
          <Link href="/witness-dyad" className="shell-action-link">
            Back to list
          </Link>
        </ActionRail>
      }
    >
      {error ? <StateBanner variant="error" title={error} /> : null}

      {loading ? (
        <StatePanel
          variant="loading"
          title="Loading execution detail"
          description="Resolving pillar text, live scores, engine metadata, and LLM configuration."
        />
      ) : execution ? (
        <>
          <div className="grid metrics">
            <MetricSurface label="User" value={execution.user_email} detail={execution.user_id} />
            <MetricSurface label="Tier" value={execution.tier} />
            <MetricSurface
              label="Consciousness"
              value={execution.consciousness_level}
            />
            <MetricSurface
              label="LLM"
              value={
                <span className={statusPillClass(execution.llm_powered ? "healthy" : "idle")}>
                  {execution.llm_powered ? "LLM" : "Rule"}
                </span>
              }
              detail={execution.llm_provider ?? undefined}
            />
            <MetricSurface label="Relationship" value={execution.relationship_mode} />
            <MetricSurface
              label="LLM Duration"
              value={formatMs(execution.llm_duration_ms)}
            />
          </div>

          <div className="grid metrics">
            {Object.entries(execution.live_scores ?? {}).map(([key, value]) => (
              <MetricSurface key={key} label={key} value={value} />
            ))}
          </div>

          <SurfaceCard eyebrow="Engines" title="Available & Used" summary="Engines available and those actually invoked for this execution.">
            <div className="grid two-col">
              <div>
                <div className="telemetry-caption">Available</div>
                <div className="helper">{execution.engines_available.join(", ") || "--"}</div>
              </div>
              <div>
                <div className="telemetry-caption">Used</div>
                <div className="helper">{execution.engines_used.join(", ") || "--"}</div>
              </div>
            </div>
          </SurfaceCard>

          {execution.witness_question ? (
            <SurfaceCard eyebrow="Query" title="Witness Question" summary="The prompt submitted for this dyad execution.">
              <blockquote className="blockquote-text">{execution.witness_question}</blockquote>
            </SurfaceCard>
          ) : null}

          <div className="grid two-col">
            <SurfaceCard eyebrow="Pillar 1" title="Aletheios (Truth)" summary="Rule-determined output.">
              <pre className="overlay-json-block">{execution.aletheios ?? "No output"}</pre>
            </SurfaceCard>
            <SurfaceCard eyebrow="Pillar 2" title="Pichet (Compassion)" summary="Rule-determined output.">
              <pre className="overlay-json-block">{execution.pichet ?? "No output"}</pre>
            </SurfaceCard>
          </div>

          <SurfaceCard eyebrow="Pillar 3" title="Synthesis" summary="Combined interpretation from Aletheios and Pichet.">
            <pre className="overlay-json-block">{execution.synthesis ?? "No output"}</pre>
          </SurfaceCard>

          <SurfaceCard eyebrow="LLM Config" title="Model Assignment" summary="Per-pillar LLM model routing.">
            <div className="grid three-col">
              <div>
                <div className="telemetry-caption">Aletheios Model</div>
                <div className="helper">{execution.llm_model_aletheios ?? "--"}</div>
              </div>
              <div>
                <div className="telemetry-caption">Pichet Model</div>
                <div className="helper">{execution.llm_model_pichet ?? "--"}</div>
              </div>
              <div>
                <div className="telemetry-caption">Synthesis Model</div>
                <div className="helper">{execution.llm_model_synthesis ?? "--"}</div>
              </div>
            </div>
          </SurfaceCard>

          {execution.error_message ? (
            <StateBanner variant="error" title="Execution error" description={execution.error_message} />
          ) : null}

          <p className="helper">Created at: {formatDateTime(execution.created_at)}</p>
        </>
      ) : null}
    </PageShell>
  );
}