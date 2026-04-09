"use client";

import Link from "next/link";
import { FormEvent, useEffect, useMemo, useState, useSyncExternalStore } from "react";
import type { BiofieldCaptureResult, BiofieldSession } from "@selemene/biofield-domain";
import { BiofieldClientError } from "@selemene/biofield-api-client";
import {
  clearStoredAuthSession,
  getStoredAuthSession,
  subscribeToAuthSession,
} from "@/lib/auth";
import { createBiofieldClient } from "@/lib/api";
import { useRouter } from "next/navigation";

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
  const [currentSession, setCurrentSession] = useState<BiofieldSession | null>(null);
  const [selectedFile, setSelectedFile] = useState<File | null>(null);
  const [captureResult, setCaptureResult] = useState<BiofieldCaptureResult | null>(null);
  const [statusMessage, setStatusMessage] = useState<string | null>(null);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);
  const [isStartingSession, setIsStartingSession] = useState(false);
  const [isClosingSession, setIsClosingSession] = useState(false);
  const [isUploading, setIsUploading] = useState(false);

  useEffect(() => {
    if (!authSession) {
      router.replace("/login");
    }
  }, [authSession, router]);

  const client = useMemo(() => {
    if (!authSession) {
      return null;
    }
    return createBiofieldClient(authSession.token);
  }, [authSession]);

  function handleAuthFailure() {
    clearStoredAuthSession();
    setCurrentSession(null);
    router.replace("/login");
  }

  async function handleStartSession() {
    if (!client) {
      return;
    }

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
    if (!client || !currentSession) {
      return;
    }

    setIsClosingSession(true);
    setErrorMessage(null);
    setStatusMessage(null);

    try {
      const session = await client.closeSession(currentSession.id, {
        reason: "viewer-exit",
      });
      setCurrentSession(session);
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
    if (!client || !currentSession || !selectedFile) {
      return;
    }

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

  const activeMetricRows = captureResult
    ? METRIC_KEYS.map((key) => ({ key, value: captureResult.metrics[key] }))
    : [];

  return (
    <section className="biofield-stack">
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

      <section className="biofield-panel biofield-form-panel">
        <div className="biofield-actions">
          <button
            className="biofield-button"
            disabled={isStartingSession || !!currentSession}
            onClick={handleStartSession}
            type="button"
          >
            {isStartingSession ? "Starting session…" : currentSession ? "Session active" : "Start session"}
          </button>
          <button
            className="biofield-link"
            disabled={isClosingSession || !currentSession || currentSession.status !== "active"}
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
              disabled={isUploading || !currentSession || currentSession.status !== "active" || !selectedFile}
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
