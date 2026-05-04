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
import { BiofieldLiveMetrics } from "@/components/BiofieldLiveMetrics";
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
  const [liveScores, setLiveScores] = useState<CompositeScores | null>(null);
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
    // Always update the live display (no throttle).
    setLiveScores(scores);

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

  const hasActiveSession = currentSession?.status === "active";

  return (
    <section className="biofield-stack">
      {/* PIP live camera + WebGL2 shader viewer */}
      <PIPViewerPanel onCapture={handlePIPCapture} onMetrics={handleMetrics} />

      {/* Live metrics — shown once MediaPipe starts flowing data */}
      {liveScores && <BiofieldLiveMetrics scores={liveScores} />}

      {/* Witness insight from Noesis biofield engine */}
      {witnessInsight && (
        <section style={{
          padding: "1.6rem 1.8rem",
          borderRadius: "var(--r-xl)",
          background: "var(--panel)",
          border: "1px solid var(--line-faint)",
          boxShadow: "inset 0 1px 0 rgba(255,255,255,0.04)",
          display: "flex",
          flexDirection: "column",
          gap: "1rem",
        }}>
          <div style={{ display: "flex", alignItems: "center", gap: "0.6rem" }}>
            <span style={{
              width: 6, height: 6, borderRadius: "50%",
              background: "var(--signal)",
              boxShadow: "0 0 8px rgba(255,179,71,0.5)",
            }} />
            <p style={{ margin: 0, fontSize: "0.7rem", fontWeight: 600, letterSpacing: "0.12em", textTransform: "uppercase", color: "var(--muted)" }}>
              Witness insight
            </p>
          </div>

          {witnessInsight.witness_prompt && (
            <p style={{
              margin: 0,
              fontSize: "1.1rem",
              lineHeight: 1.65,
              fontStyle: "italic",
              color: "var(--text-2)",
              borderLeft: "2px solid var(--accent-border)",
              paddingLeft: "1rem",
            }}>
              &ldquo;{witnessInsight.witness_prompt}&rdquo;
            </p>
          )}

          {witnessInsight.consciousness_level !== undefined && (
            <div style={{ display: "flex", alignItems: "center", gap: "0.75rem" }}>
              <span style={{ fontSize: "0.7rem", fontWeight: 600, letterSpacing: "0.08em", textTransform: "uppercase", color: "var(--muted)" }}>
                Consciousness level
              </span>
              <span style={{
                fontSize: "1.4rem", fontWeight: 700, letterSpacing: "-0.04em",
                color: "var(--accent)",
              }}>
                {witnessInsight.consciousness_level}
              </span>
            </div>
          )}
        </section>
      )}

      {/* Session row — compact single strip */}
      <section style={{
        padding: "1rem 1.4rem",
        borderRadius: "var(--r-xl)",
        background: "var(--panel)",
        border: "1px solid var(--line-faint)",
        display: "flex",
        alignItems: "center",
        justifyContent: "space-between",
        flexWrap: "wrap",
        gap: "0.75rem",
      }}>
        {/* Status */}
        <div style={{ display: "flex", alignItems: "center", gap: "1.2rem" }}>
          <div>
            <p style={{ margin: 0, fontSize: "0.68rem", fontWeight: 600, letterSpacing: "0.1em", textTransform: "uppercase", color: "var(--muted)" }}>Account</p>
            <p style={{ margin: 0, fontSize: "0.82rem", color: "var(--text)" }}>{authSession?.email ?? "—"}</p>
          </div>
          <div style={{ width: 1, height: 28, background: "var(--line-faint)" }} />
          <div>
            <p style={{ margin: 0, fontSize: "0.68rem", fontWeight: 600, letterSpacing: "0.1em", textTransform: "uppercase", color: "var(--muted)" }}>Session</p>
            <p style={{ margin: 0, fontSize: "0.82rem", color: hasActiveSession ? "var(--accent)" : "var(--muted)" }}>
              {isHydratingSession ? "Restoring…" : currentSession ? currentSession.status : storedSessionId ? `Restoring ${storedSessionId}…` : "No session"}
            </p>
          </div>
          <div style={{ width: 1, height: 28, background: "var(--line-faint)" }} />
          <div>
            <p style={{ margin: 0, fontSize: "0.68rem", fontWeight: 600, letterSpacing: "0.1em", textTransform: "uppercase", color: "var(--muted)" }}>Tier</p>
            <p style={{ margin: 0, fontSize: "0.82rem", color: "var(--signal)" }}>{authSession?.tier ?? "—"}</p>
          </div>
        </div>

        {/* Actions */}
        <div className="biofield-actions" style={{ margin: 0 }}>
          <button
            className="biofield-button"
            disabled={isStartingSession || isHydratingSession || hasActiveSession}
            onClick={handleStartSession}
            type="button"
          >
            {isStartingSession ? "Starting…" : hasActiveSession ? "Active" : "Start session"}
          </button>
          <button
            className="biofield-link"
            disabled={isClosingSession || !hasActiveSession}
            onClick={handleCloseSession}
            type="button"
          >
            {isClosingSession ? "Closing…" : "End session"}
          </button>
        </div>

        {statusMessage && <p className="biofield-success" style={{ width: "100%", margin: 0 }}>{statusMessage}</p>}
        {errorMessage  && <p className="biofield-error"  style={{ width: "100%", margin: 0 }}>{errorMessage}</p>}
      </section>

      {/* Manual capture — collapsible-style minimal form */}
      <section className="biofield-panel biofield-form-panel">
        <p className="biofield-eyebrow">Manual capture</p>
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

      {/* Capture result */}
      {captureResult && (
        <section className="biofield-panel">
          <div style={{ display: "flex", alignItems: "baseline", justifyContent: "space-between", marginBottom: "1rem" }}>
            <div>
              <p className="biofield-eyebrow">Analysis complete</p>
              <h2 className="biofield-title" style={{ fontSize: "1.6rem", margin: 0 }}>
                {captureResult.analysis_version}
              </h2>
            </div>
            <span style={{
              fontSize: "0.7rem", fontWeight: 600, letterSpacing: "0.08em", textTransform: "uppercase",
              padding: "0.22rem 0.6rem", borderRadius: "var(--r-pill)",
              background: captureResult.quality_assessment.sufficient_quality ? "rgba(124,124,255,0.12)" : "rgba(255,100,100,0.1)",
              border: `1px solid ${captureResult.quality_assessment.sufficient_quality ? "rgba(124,124,255,0.3)" : "rgba(255,100,100,0.25)"}`,
              color: captureResult.quality_assessment.sufficient_quality ? "var(--accent)" : "#ff6464",
            }}>
              {captureResult.quality_assessment.sufficient_quality ? "Accepted" : "Rejected"}
            </span>
          </div>

          <div style={{
            display: "grid", gridTemplateColumns: "1fr 1fr",
            gap: "0.5rem", marginBottom: "1rem",
          }}>
            {METRIC_KEYS.map((key) => {
              const raw = captureResult.metrics[key];
              const num = typeof raw === "number" ? raw : parseFloat(String(raw));
              const isValid = !isNaN(num);
              const pct = isValid ? Math.round(num * 100) : null;
              return (
                <div key={key} style={{
                  padding: "0.65rem 0.8rem",
                  borderRadius: "var(--r-md)",
                  background: "rgba(255,255,255,0.025)",
                  border: "1px solid var(--line-faint)",
                  display: "flex", flexDirection: "column", gap: "0.35rem",
                }}>
                  <span style={{ fontSize: "0.62rem", fontWeight: 600, letterSpacing: "0.1em", textTransform: "uppercase", color: "var(--muted)" }}>
                    {key.replaceAll("_", " ")}
                  </span>
                  {pct !== null ? (
                    <>
                      <span style={{ fontSize: "1.1rem", fontWeight: 700, letterSpacing: "-0.04em", color: "var(--text)" }}>
                        {pct}<span style={{ fontSize: "0.6rem", opacity: 0.45 }}>%</span>
                      </span>
                      <div style={{ height: 2, borderRadius: 9999, background: "rgba(255,255,255,0.06)" }}>
                        <div style={{
                          width: `${pct}%`, height: "100%", borderRadius: 9999,
                          background: "var(--accent)",
                          transition: "width 0.5s ease",
                        }} />
                      </div>
                    </>
                  ) : (
                    <span style={{ fontSize: "0.82rem", color: "var(--text-2)" }}>{String(raw)}</span>
                  )}
                </div>
              );
            })}
          </div>

          <div className="biofield-actions">
            <Link className="biofield-link" href={`/readings/${captureResult.reading_id}`}>
              Open reading
            </Link>
            <Link className="biofield-link" href="/history">
              History
            </Link>
          </div>
        </section>
      )}
    </section>
  );
}

