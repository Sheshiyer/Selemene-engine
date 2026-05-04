"use client";

import Link from "next/link";
import { FormEvent, useCallback, useEffect, useMemo, useRef, useState, useSyncExternalStore } from "react";
import type { BiofieldCaptureResult, BiofieldSession } from "@selemene/biofield-domain";
import { BiofieldClientError } from "@selemene/biofield-api-client";
import type { EngineOutput } from "@selemene/noesis-sdk-ts";
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
import { createBiofieldClient, createNoesisClient } from "@/lib/api";
import { useRouter } from "next/navigation";
import { PIPViewerPanel } from "@/components/pip/PIPViewerPanel";
import type { CompositeScores } from "@/components/pip/types";

// Interval at which live metrics are posted to the Noesis biofield engine.
const METRICS_SUBMIT_INTERVAL_MS = 30_000;

const METRIC_KEYS = [
  "light_quanta_density",
  "normalized_area",
  "average_intensity",
  "fractal_dimension",
  "body_symmetry",
  "pattern_regularity",
] as const;

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
  const [currentSession, setCurrentSession] = useState<BiofieldSession | null>(null);
  const [selectedFile, setSelectedFile] = useState<File | null>(null);
  const [captureResult, setCaptureResult] = useState<BiofieldCaptureResult | null>(null);
  const [statusMessage, setStatusMessage] = useState<string | null>(null);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);
  const [isHydratingSession, setIsHydratingSession] = useState(false);
  const [isStartingSession, setIsStartingSession] = useState(false);
  const [isClosingSession, setIsClosingSession] = useState(false);
  const [isUploading, setIsUploading] = useState(false);
  const [witnessInsight, setWitnessInsight] = useState<EngineOutput | null>(null);

  // Track last metrics submission timestamp to rate-limit engine calls.
  const lastMetricsSubmitRef = useRef<number>(0);

  useEffect(() => {
    if (!authSession) {
      router.replace("/login");
    }
  }, [authSession, router]);

  const client = useMemo(() => {
    if (!authSession) return null;
    return createBiofieldClient(authSession.token);
  }, [authSession]);

  const noesisClient = useMemo(() => {
    if (!authSession) return null;
    return createNoesisClient(authSession.token);
  }, [authSession]);

  function handleAuthFailure() {
    clearStoredActiveSessionId();
    clearStoredAuthSession();
    setCurrentSession(null);
    router.replace("/login");
  }

  useEffect(() => {
    if (!client) return;
    if (!storedSessionId) return;
    if (currentSession?.id === storedSessionId) return;

    let isCancelled = false;

    async function hydrateSession() {
      setIsHydratingSession(true);
      setErrorMessage(null);
      setStatusMessage(null);

      try {
        const session = await client.getSession(storedSessionId);
        if (isCancelled) return;
        setCurrentSession(session);
        if (session.status === "active") {
          setStatusMessage(`Restored session ${session.id}.`);
        } else {
          clearStoredActiveSessionId();
          setStatusMessage(
            `Saved session ${session.id} is ${session.status}; start a new session to continue.`,
          );
        }
      } catch (error) {
        if (isCancelled) return;
        if (error instanceof BiofieldClientError && error.status === 401) {
          handleAuthFailure();
          return;
        }
        clearStoredActiveSessionId();
        setCurrentSession(null);
        setErrorMessage(
          error instanceof Error ? error.message : "Failed to restore your last biofield session.",
        );
      } finally {
        if (!isCancelled) setIsHydratingSession(false);
      }
    }

    void hydrateSession();
    return () => { isCancelled = true; };
  }, [client, currentSession, storedSessionId]);

  async function handleStartSession() {
    if (!client) return;

    setIsStartingSession(true);
    setErrorMessage(null);
    setStatusMessage(null);

    try {
      const session = await client.createSession({
        client_device_id: "browser",
        viewer_version: "biofield-web/0.1.0",
        context: {
          platform: typeof navigator !== "undefined" ? navigator.userAgent : "unknown",
          viewport:
            typeof window !== "undefined"
              ? { width: window.innerWidth, height: window.innerHeight }
              : undefined,
        },
      });
      setCurrentSession(session);
      setStoredActiveSessionId(session.id);
      setCaptureResult(null);
      setWitnessInsight(null);
      setStatusMessage(`Session ${session.id} is active.`);
    } catch (error) {
      if (error instanceof BiofieldClientError && error.status === 401) {
        handleAuthFailure();
        return;
      }
      setErrorMessage(error instanceof Error ? error.message : "Failed to start biofield session.");
    } finally {
      setIsStartingSession(false);
    }
  }

  async function handleCloseSession() {
    if (!client || !currentSession) return;

    setIsClosingSession(true);
    setErrorMessage(null);
    setStatusMessage(null);

    try {
      const session = await client.closeSession(currentSession.id, { reason: "viewer-exit" });
      setCurrentSession(session);
      clearStoredActiveSessionId();
      setStatusMessage(`Session ${session.id} closed.`);
    } catch (error) {
      if (error instanceof BiofieldClientError && error.status === 401) {
        handleAuthFailure();
        return;
      }
      setErrorMessage(error instanceof Error ? error.message : "Failed to close biofield session.");
    } finally {
      setIsClosingSession(false);
    }
  }

  async function handleUpload(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (!client || !currentSession || !selectedFile) return;

    setIsUploading(true);
    setErrorMessage(null);
    setStatusMessage(null);

    const formData = new FormData();
    formData.append("image", selectedFile);
    formData.append("options", JSON.stringify({ mode: "capture" }));
    formData.append(
      "capture_metadata",
      JSON.stringify({
        platform: "web",
        file_name: selectedFile.name,
        file_size: selectedFile.size,
        file_type: selectedFile.type,
        viewport:
          typeof window !== "undefined"
            ? { width: window.innerWidth, height: window.innerHeight }
            : undefined,
      }),
    );

    try {
      const result = await client.uploadCapture(currentSession.id, formData);
      setCaptureResult(result);
      setStatusMessage(`Capture analyzed with ${result.analysis_version}.`);
    } catch (error) {
      if (error instanceof BiofieldClientError && error.status === 401) {
        handleAuthFailure();
        return;
      }
      setErrorMessage(error instanceof Error ? error.message : "Failed to upload capture.");
    } finally {
      setIsUploading(false);
    }
  }

  // BF1-05.2: Wire PIP blob → POST /api/v1/biofield/sessions/:id/captures.
  const handlePIPCapture = useCallback(async (blob: Blob, _dataUrl: string) => {
    if (!client || !currentSession) {
      console.warn("[PIPViewer] no active session — capture not uploaded");
      return;
    }

    setIsUploading(true);
    setErrorMessage(null);
    setStatusMessage(null);

    const file = new File([blob], "pip-capture.png", { type: blob.type || "image/png" });
    const formData = new FormData();
    formData.append("image", file);
    formData.append("options", JSON.stringify({ mode: "capture", source: "pip-camera" }));
    formData.append(
      "capture_metadata",
      JSON.stringify({
        platform: "web",
        source: "pip-camera",
        file_name: "pip-capture.png",
        file_size: blob.size,
        file_type: file.type,
        viewport:
          typeof window !== "undefined"
            ? { width: window.innerWidth, height: window.innerHeight }
            : undefined,
      }),
    );

    try {
      const result = await client.uploadCapture(currentSession.id, formData);
      setCaptureResult(result);
      setStatusMessage(`PIP capture analyzed with ${result.analysis_version}.`);
    } catch (error) {
      if (error instanceof BiofieldClientError && error.status === 401) {
        handleAuthFailure();
        return;
      }
      setErrorMessage(error instanceof Error ? error.message : "Failed to upload PIP capture.");
    } finally {
      setIsUploading(false);
    }
  }, [client, currentSession]);

  // BF1-05.6: Submit live composite scores to Noesis biofield engine.
  // Throttled to METRICS_SUBMIT_INTERVAL_MS (30s) to avoid hammering the API.
  const handleMetrics = useCallback((scores: CompositeScores) => {
    if (!noesisClient) return;

    const now = performance.now();
    if (now - lastMetricsSubmitRef.current < METRICS_SUBMIT_INTERVAL_MS) return;
    lastMetricsSubmitRef.current = now;

    void noesisClient.calculate("biofield", {
      options: {
        source: "pip-live-metrics",
        light_quanta_density: scores.lightQuantaDensity,
        normalized_area: scores.normalizedArea,
        body_symmetry: scores.bodySymmetry,
        pattern_regularity: scores.patternRegularity,
        overall_coherence: scores.overallCoherence,
      },
    }).then((output) => {
      setWitnessInsight(output);
    }).catch((err) => {
      // Non-blocking: engine errors don't interrupt the live viewer.
      console.warn("[BF1-05.6] biofield engine error:", err instanceof Error ? err.message : err);
    });
  }, [noesisClient]);

  const activeMetricRows = captureResult
    ? METRIC_KEYS.map((key) => ({ key, value: captureResult.metrics[key] }))
    : [];
  const hasActiveSession = currentSession?.status === "active";

  return (
    <section className="biofield-stack">
      {/* PIP live camera + WebGL2 shader viewer */}
      <PIPViewerPanel onCapture={handlePIPCapture} onMetrics={handleMetrics} />

      <section className="biofield-grid">
        <article className="biofield-panel">
          <p className="biofield-kicker">Auth state</p>
          <p className="biofield-metric">{authSession ? authSession.email : "Loading…"}</p>
          <p className="biofield-copy">
            Viewer requests use the stored bearer token from the BF1-04 login flow.
          </p>
        </article>
        <article className="biofield-panel">
          <p className="biofield-kicker">Server session</p>
          <p className="biofield-metric">
            {currentSession ? currentSession.status : "No active session"}
          </p>
          <p className="biofield-copy">
            {currentSession
              ? `Session ${currentSession.id} is managed by Noesis.`
              : storedSessionId
                ? `Restoring session ${storedSessionId} from local state…`
                : "Start a real biofield session before uploading a capture."}
          </p>
        </article>
        <article className="biofield-panel">
          <p className="biofield-kicker">Capture path</p>
          <p className="biofield-metric">Browser → Noesis → Python</p>
          <p className="biofield-copy">
            Successful captures now flow into persisted history and reading detail routes in the web surface.
          </p>
        </article>
      </section>

      {/* Witness insight from live metrics → biofield engine */}
      {witnessInsight && (
        <section className="biofield-panel">
          <p className="biofield-eyebrow">Witness insight</p>
          <h2 className="biofield-title" style={{ fontSize: "1.5rem" }}>
            Live field reading
          </h2>
          {witnessInsight.witness_prompt && (
            <p className="biofield-copy" style={{ fontStyle: "italic", opacity: 0.9 }}>
              &ldquo;{witnessInsight.witness_prompt}&rdquo;
            </p>
          )}
          {witnessInsight.consciousness_level !== undefined && (
            <p className="biofield-kicker" style={{ marginTop: "0.5rem" }}>
              Consciousness level: {witnessInsight.consciousness_level}
            </p>
          )}
        </section>
      )}

      <section className="biofield-panel biofield-form-panel">
        <div className="biofield-actions">
          <button
            className="biofield-button"
            disabled={isStartingSession || isHydratingSession || hasActiveSession}
            onClick={handleStartSession}
            type="button"
          >
            {isHydratingSession
              ? "Restoring session…"
              : isStartingSession
                ? "Starting session…"
                : hasActiveSession
                  ? "Session active"
                  : "Start session"}
          </button>
          <button
            className="biofield-link"
            disabled={isClosingSession || !hasActiveSession}
            onClick={handleCloseSession}
            type="button"
          >
            {isClosingSession ? "Closing session…" : "Close session"}
          </button>
        </div>

        {statusMessage ? <p className="biofield-success">{statusMessage}</p> : null}
        {errorMessage ? <p className="biofield-error">{errorMessage}</p> : null}
      </section>

      <section className="biofield-panel biofield-form-panel">
        <p className="biofield-eyebrow">Capture upload</p>
        <h2 className="biofield-title" style={{ fontSize: "2rem" }}>
          Upload a capture
        </h2>
        <p className="biofield-copy">
          Use any local image to exercise the BF1-05 multipart upload and sidecar proxy path.
        </p>

        <form className="biofield-form" onSubmit={handleUpload}>
          <label className="biofield-field" htmlFor="biofield-capture-file">
            <span className="biofield-kicker">Image file</span>
            <input
              accept="image/*"
              className="biofield-input"
              id="biofield-capture-file"
              onChange={(event) => setSelectedFile(event.target.files?.[0] ?? null)}
              type="file"
            />
          </label>

          <div className="biofield-actions">
            <button
              className="biofield-button"
              disabled={isUploading || !hasActiveSession || !selectedFile}
              type="submit"
            >
              {isUploading ? "Uploading…" : "Upload capture"}
            </button>
          </div>
        </form>
      </section>

      {captureResult ? (
        <section className="biofield-panel">
          <p className="biofield-eyebrow">Latest analysis</p>
          <h2 className="biofield-title" style={{ fontSize: "2rem" }}>
            {captureResult.analysis_version}
          </h2>
          <div className="biofield-list-grid">
            <div className="biofield-list-card">
              <p className="biofield-kicker">Reading ID</p>
              <p className="biofield-copy biofield-mono">{captureResult.reading_id}</p>
            </div>
            <div className="biofield-list-card">
              <p className="biofield-kicker">Session ID</p>
              <p className="biofield-copy biofield-mono">{captureResult.session_id}</p>
            </div>
            <div className="biofield-list-card">
              <p className="biofield-kicker">Quality</p>
              <p className="biofield-copy">
                {captureResult.quality_assessment.sufficient_quality ? "Accepted" : "Rejected"}
              </p>
            </div>
          </div>
          <div className="biofield-actions" style={{ marginBottom: "1rem" }}>
            <Link className="biofield-link" href={`/readings/${captureResult.reading_id}`}>
              Open reading detail
            </Link>
            <Link className="biofield-link" href="/history">
              View history
            </Link>
          </div>
          <ul className="biofield-list">
            {activeMetricRows.map((metric) => (
              <li key={metric.key}>
                <p className="biofield-kicker">{metric.key}</p>
                <p className="biofield-metric">{String(metric.value)}</p>
              </li>
            ))}
          </ul>
        </section>
      ) : null}
    </section>
  );
}


const METRIC_KEYS = [
  "light_quanta_density",
  "normalized_area",
  "average_intensity",
  "fractal_dimension",
  "body_symmetry",
  "pattern_regularity",
] as const;

