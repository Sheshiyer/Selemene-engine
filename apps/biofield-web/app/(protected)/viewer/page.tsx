"use client";

import Link from "next/link";
import { useCallback, useEffect, useMemo, useState, useSyncExternalStore } from "react";
import { PIPViewerPanel } from "@/components/pip/PIPViewerPanel";
import { BIOFIELD_ENGINE_ID } from "@selemene/biofield-domain";
import type { CompositeScores, FrameMetrics } from "@/components/pip/MetricsCalculator";
import { BiofieldClientError } from "@selemene/biofield-api-client";
import {
  clearStoredAuthSession,
  getStoredAuthSession,
  subscribeToAuthSession,
} from "@/lib/auth";
import {
  clearStoredActiveSessionId,
  getStoredActiveSessionId,
  setStoredActiveSessionId,
  subscribeToActiveSessionId,
} from "@/lib/session";
import { createBiofieldClient } from "@/lib/api";
import { useRouter } from "next/navigation";

/**
 * ViewerPage — live PIP biofield viewer.
 *
 * Phase: BF1-05 (capture upload path + sidecar proxy)
 * Current wave: camera + WebGL2 shader only (no ML segmentation yet).
 * Next wave: wire capturedDataUrl → POST /api/v1/biofield/sessions/:id/captures
 *            → Python sidecar → Noesis engine seam.
 */
export default function ViewerPage() {
  const router = useRouter();
  const authSession = useSyncExternalStore(
    subscribeToAuthSession,
    getStoredAuthSession,
    () => null,
  );
  const storedSessionId = useSyncExternalStore(
    subscribeToActiveSessionId,
    getStoredActiveSessionId,
    () => null,
  );

  const client = useMemo(() => {
    if (!authSession) {
      return null;
    }
    return createBiofieldClient(authSession.token);
  }, [authSession]);

  const [sessionId, setSessionId] = useState<string | null>(null);
  const [sessionStatus, setSessionStatus] = useState<string>("idle");
  const [isSessionActionLoading, setIsSessionActionLoading] = useState(false);
  const [uploadState, setUploadState] = useState<"idle" | "uploading" | "done" | "error">("idle");
  const [uploadError, setUploadError] = useState<string | null>(null);
  const [lastCapture, setLastCapture] = useState<{
    dataUrl: string;
    maskDataUrl: string | null;
    timestamp: number;
    frameMetrics: FrameMetrics | null;
    compositeScores: CompositeScores | null;
    readingId?: string;
  } | null>(null);

  useEffect(() => {
    if (!authSession) {
      router.replace("/login");
    }
  }, [authSession, router]);

  useEffect(() => {
    if (storedSessionId) {
      setSessionId(storedSessionId);
    }
  }, [storedSessionId]);

  const handleAuthFailure = useCallback(() => {
    clearStoredActiveSessionId();
    clearStoredAuthSession();
    setSessionId(null);
    setSessionStatus("idle");
    router.replace("/login");
  }, [router]);

  const dataUrlToBlob = async (dataUrl: string): Promise<Blob> => {
    const resp = await fetch(dataUrl);
    return resp.blob();
  };

  const ensureSession = useCallback(async (): Promise<string> => {
    if (!client) {
      throw new Error("Please sign in before creating a session");
    }
    if (sessionId) return sessionId;

    const session = await client.createSession({
      client_device_id: typeof navigator !== "undefined" ? navigator.userAgent : "web",
      viewer_version: "biofield-web/pip-v1",
      context: {
        platform: "web",
        viewport:
          typeof window !== "undefined"
            ? { width: window.innerWidth, height: window.innerHeight }
            : undefined,
      },
    });

    setSessionId(session.id);
    setStoredActiveSessionId(session.id);
    setSessionStatus(session.status);
    return session.id;
  }, [client, sessionId]);

  const handleStartSession = useCallback(async () => {
    if (!client) {
      return;
    }

    setIsSessionActionLoading(true);
    setUploadError(null);

    try {
      await ensureSession();
    } catch (err) {
      if (err instanceof BiofieldClientError && err.status === 401) {
        handleAuthFailure();
        return;
      }
      setUploadError(err instanceof Error ? err.message : "Failed to start session");
    } finally {
      setIsSessionActionLoading(false);
    }
  }, [client, ensureSession, handleAuthFailure]);

  const handleCloseSession = useCallback(async () => {
    if (!client || !sessionId) {
      return;
    }

    setIsSessionActionLoading(true);
    setUploadError(null);

    try {
      const closed = await client.closeSession(sessionId, { reason: "viewer-close" });
      setSessionStatus(closed.status);
      setSessionId(null);
      clearStoredActiveSessionId();
    } catch (err) {
      if (err instanceof BiofieldClientError && err.status === 401) {
        handleAuthFailure();
        return;
      }
      setUploadError(err instanceof Error ? err.message : "Failed to close session");
    } finally {
      setIsSessionActionLoading(false);
    }
  }, [client, handleAuthFailure, sessionId]);

  const handleCapture = useCallback(
    async (payload: {
      dataUrl: string;
      maskDataUrl: string | null;
      timestamp: number;
      frameMetrics: FrameMetrics | null;
      compositeScores: CompositeScores | null;
    }) => {
      setLastCapture(payload);
      setUploadState("uploading");
      setUploadError(null);

      try {
        const sid = await ensureSession();
        if (!client) {
          throw new Error("Please sign in before uploading captures");
        }

        const form = new FormData();
        form.append("image", await dataUrlToBlob(payload.dataUrl), `capture-${payload.timestamp}.png`);

        if (payload.maskDataUrl) {
          form.append(
            "segmentation_mask",
            await dataUrlToBlob(payload.maskDataUrl),
            `mask-${payload.timestamp}.png`
          );
        }

        form.append(
          "capture_metadata",
          JSON.stringify({
            engine_id: BIOFIELD_ENGINE_ID,
            capture_timestamp: payload.timestamp,
            realtime_metrics: payload.frameMetrics,
            realtime_scores: payload.compositeScores,
          })
        );
        form.append("options", JSON.stringify({ mode: "capture", source: "biofield-web-viewer" }));

        const result = await client.uploadCapture(sid, form);

        setLastCapture((prev) =>
          prev
            ? {
                ...prev,
                readingId: result.reading_id,
              }
            : prev
        );
        setSessionStatus("active");
        setUploadState("done");
      } catch (err) {
        if (err instanceof BiofieldClientError && err.status === 401) {
          handleAuthFailure();
          return;
        }
        setUploadState("error");
        setUploadError(err instanceof Error ? err.message : "Capture upload failed");
      }
    },
    [client, ensureSession, handleAuthFailure]
  );

  if (!authSession) {
    return (
      <section className="biofield-panel">
        <p className="biofield-eyebrow">Auth gate</p>
        <p className="biofield-copy">Checking your biofield access…</p>
      </section>
    );
  }

  return (
    <section className="biofield-viewer-layout">
      <section className="biofield-panel biofield-form-panel">
        <p className="biofield-eyebrow">Session lifecycle</p>
        <div className="biofield-actions">
          <button
            className="biofield-button"
            disabled={isSessionActionLoading || !!sessionId}
            onClick={() => void handleStartSession()}
            type="button"
          >
            {isSessionActionLoading ? "Starting…" : "Start session"}
          </button>
          <button
            className="biofield-link"
            disabled={isSessionActionLoading || !sessionId}
            onClick={() => void handleCloseSession()}
            type="button"
          >
            {isSessionActionLoading ? "Closing…" : "Close session"}
          </button>
        </div>
        <p className="biofield-copy" style={{ marginTop: 8 }}>
          Session: {sessionId ?? "none"} · status {sessionStatus}
        </p>
      </section>

      <PIPViewerPanel onCapture={handleCapture} />

      {lastCapture && (
        <aside className="biofield-capture-preview">
          <p className="biofield-kicker">
            Last capture — {new Date(lastCapture.timestamp).toLocaleTimeString()}
          </p>
          {/* eslint-disable-next-line @next/next/no-img-element */}
          <img
            src={lastCapture.dataUrl}
            alt="PIP capture"
            className="biofield-capture-thumb"
          />
          <p className="biofield-copy" style={{ marginTop: 8 }}>
            Ready to upload · engine <code>{BIOFIELD_ENGINE_ID}</code>
          </p>
          <p className="biofield-copy" style={{ marginTop: 8 }}>
            Session: {sessionId ?? "pending"}
          </p>
          <p className="biofield-copy" style={{ marginTop: 8 }}>
            Upload: {uploadState}
            {lastCapture.readingId ? ` · reading ${lastCapture.readingId}` : ""}
          </p>
          {lastCapture.readingId ? (
            <p className="biofield-copy" style={{ marginTop: 8 }}>
              <Link className="biofield-link" href={`/readings/${lastCapture.readingId}`}>
                Open reading detail
              </Link>
            </p>
          ) : null}
          {uploadError && <p className="biofield-error-inline">{uploadError}</p>}
        </aside>
      )}
    </section>
  );
}
