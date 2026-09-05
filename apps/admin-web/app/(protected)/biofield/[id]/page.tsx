"use client";

import { use, useEffect, useState } from "react";
import { useRouter } from "next/navigation";
import { ActionRail, MetricSurface, SurfaceCard } from "@/components/admin-primitives";
import { StateBanner, StatePanel } from "@/components/admin-state";
import { PageShell } from "@/components/page-shell";
import { getAuthToken } from "@/lib/auth";
import { ApiClientError, getAdminBiofieldSession } from "@/lib/api";
import { statusPillClass } from "@/lib/status";
import type { AdminBiofieldSessionItem } from "@/types/admin";

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

function formatDuration(started: string, closed: string | null): string {
  const start = new Date(started).getTime();
  if (Number.isNaN(start)) {
    return "--";
  }
  const end = closed ? new Date(closed).getTime() : Date.now();
  const ms = end - start;
  const seconds = Math.floor(ms / 1000);
  const minutes = Math.floor(seconds / 60);
  const hours = Math.floor(minutes / 60);
  if (hours > 0) {
    return `${hours}h ${minutes % 60}m`;
  }
  return `${minutes}m ${seconds % 60}s`;
}

export default function BiofieldSessionDetailPage({ params }: { params: Promise<{ id: string }> }) {
  const router = useRouter();
  const { id } = use(params);

  const [fetchResult, setFetchResult] = useState<{
    key: string;
    session: AdminBiofieldSessionItem | null;
    error: string | null;
    settled: boolean;
  }>({ key: "", session: null, error: null, settled: false });

  const loading = !fetchResult.settled || fetchResult.key !== id;
  const error = fetchResult.key === id ? fetchResult.error : null;
  const session = fetchResult.key === id ? fetchResult.session : null;

  useEffect(() => {
    let cancelled = false;
    const token = getAuthToken() ?? undefined;

    getAdminBiofieldSession(token, id)
      .then((data) => {
        if (!cancelled) {
          setFetchResult({ key: id, session: data, error: null, settled: true });
        }
      })
      .catch((err) => {
        if (!cancelled) {
          let message = "Failed to load session detail";
          if (err instanceof ApiClientError) {
            message = err.payload?.error || err.message;
          } else if (err instanceof Error) {
            message = err.message;
          }
          setFetchResult({ key: id, session: null, error: message, settled: true });
        }
      });

    return () => {
      cancelled = true;
    };
  }, [id]);

  const sessionDuration = session ? formatDuration(session.started_at, session.closed_at) : "--";

  return (
    <PageShell
      title={`Session ${id.slice(0, 8)}…`}
      summary="Biofield session detail with user context, device metadata, artifact and reading inventory."
    >
      <ActionRail>
        <button type="button" onClick={() => router.push("/biofield")}>
          &larr; Back to sessions
        </button>
      </ActionRail>

      {error ? <StateBanner variant="error" title={error} /> : null}

      {loading ? (
        <StatePanel
          variant="loading"
          title="Loading session detail"
          description="Resolving biofield capture metadata, timing, artifact and reading posture."
        />
      ) : session ? (
        <div className="grid detail-section-grid">
          <SurfaceCard
            eyebrow="Identity"
            title={session.user_email}
            summary="User context and session timing for the selected capture."
          >
            <div className="grid overlay-detail-grid">
              <div className="helper">Session ID: {session.session_id}</div>
              <div className="helper">User ID: {session.user_id}</div>
              <div className="helper">Started at: {formatDateTime(session.started_at)}</div>
              <div className="helper">Closed at: {formatDateTime(session.closed_at)}</div>
              <div className="helper">Latest reading: {formatDateTime(session.latest_reading_at)}</div>
            </div>
          </SurfaceCard>

          <div className="grid metrics">
            <MetricSurface
              label="Status"
              value={session.status}
              detail={<span className={statusPillClass(session.status)}>{session.status}</span>}
            />
            <MetricSurface
              label="Duration"
              value={sessionDuration}
              detail="Elapsed time from session start to close or now."
            />
            <MetricSurface
              label="Artifacts"
              value={session.artifact_count}
              detail="Biofield capture artifacts linked to this session."
            />
            <MetricSurface
              label="Readings"
              value={session.reading_count}
              detail="Total readings generated during this session window."
            />
          </div>

          <SurfaceCard
            eyebrow="Device"
            title="Client device metadata"
            summary="Hardware and viewer context submitted during session initialization."
          >
            <div className="grid overlay-detail-grid">
              <div className="helper">Device ID: {session.client_device_id ?? "--"}</div>
              <div className="helper">Viewer version: {session.viewer_version ?? "--"}</div>
            </div>
          </SurfaceCard>

          <SurfaceCard
            eyebrow="Timeline"
            title="Session lifecycle"
            summary="Status progression from initialization to terminal state."
          >
            <div className="grid overlay-detail-grid">
              <div className="helper">
                <span className="helper-label">Started</span>{" "}
                <span className={statusPillClass("healthy")}>active</span>{" "}
                {formatDateTime(session.started_at)}
              </div>
              {session.closed_at ? (
                <div className="helper">
                  <span className="helper-label">Closed</span>{" "}
                  <span className={statusPillClass(session.status === "abandoned" ? "abandoned" : "healthy")}>
                    {session.status}
                  </span>{" "}
                  {formatDateTime(session.closed_at)}
                </div>
              ) : (
                <div className="helper">
                  <span className="helper-label">Current</span>{" "}
                  <span className={statusPillClass(session.status)}>{session.status}</span>{" "}
                  (open)
                </div>
              )}
            </div>
          </SurfaceCard>

          <SurfaceCard
            eyebrow="Deep links"
            title="User-scoped biofield endpoints"
            summary="Navigate to user-facing biofield surfaces using this session's user context."
          >
            <div className="action-stack">
              <div className="helper">
                Forward to user-scoped endpoints by constructing paths with the session&rsquo;s user_id.
              </div>
              <div className="helper">user_id: {session.user_id}</div>
              <div className="helper">session_id: {session.session_id}</div>
            </div>
          </SurfaceCard>
        </div>
      ) : (
        <StatePanel
          variant="empty"
          title="Session not found"
          description="The requested biofield session does not exist or could not be resolved."
        />
      )}
    </PageShell>
  );
}