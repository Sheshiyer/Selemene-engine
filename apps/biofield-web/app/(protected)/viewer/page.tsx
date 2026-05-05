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

  // ── Witness Dyad data ──────────────────────────────────────────────────────
  const wl = witnessInsight?.result?.witness_layer as {
    aletheios?: { perspective?: string };
    pichet?: { perspective?: string };
    synthesis?: string;
    witness_question?: string;
  } | undefined;
  const aletheios = wl?.aletheios?.perspective;
  const pichet = wl?.pichet?.perspective;
  const synthesis = wl?.synthesis;
  const dyadFallback = witnessInsight?.witness_prompt;
  const hasDyad = aletheios || pichet || synthesis;

  return (
    /* Single non-scrollable viewport — 100dvh × 100vw grid */
    <div style={{
      position: "fixed", inset: 0,
      width: "100vw", height: "100dvh",
      overflow: "hidden",
      display: "grid",
      gridTemplateColumns: "58fr 42fr",
      background: "var(--bg)",
    }}>

      {/* ───── Left: full-height camera / shader panel ───── */}
      <div style={{ height: "100dvh", overflow: "hidden", position: "relative" }}>
        <PIPViewerPanel onCapture={handlePIPCapture} onMetrics={handleMetrics} fillHeight />
      </div>

      {/* ───── Right: data panel ───── */}
      <div style={{
        height: "100dvh",
        display: "flex",
        flexDirection: "column",
        borderLeft: "1px solid var(--line-faint)",
        background: "var(--surface)",
        overflow: "hidden",
      }}>

        {/* ── METRICS (top ~42%) ── */}
        <div style={{
          flex: "0 0 auto",
          padding: "1rem 1.1rem 0.6rem",
          borderBottom: "1px solid var(--line-faint)",
        }}>
          {liveScores ? (
            <BiofieldLiveMetrics scores={liveScores} />
          ) : (
            /* Idle placeholder */
            <div style={{
              padding: "1.5rem 1.2rem",
              borderRadius: "var(--r-xl)",
              background: "var(--panel)",
              border: "1px solid var(--line-faint)",
              display: "flex",
              flexDirection: "column",
              gap: "0.85rem",
            }}>
              <div style={{ display: "flex", alignItems: "center", gap: 6 }}>
                <span style={{ width: 6, height: 6, borderRadius: "50%", background: "var(--muted)" }} />
                <p style={{ margin: 0, fontFamily: "var(--font-mono)", fontSize: "0.62rem", fontWeight: 600, letterSpacing: "0.14em", textTransform: "uppercase", color: "var(--muted)" }}>
                  Live field metrics
                </p>
              </div>
              {[...Array(5)].map((_, i) => (
                <div key={i} style={{ display: "flex", flexDirection: "column", gap: 4 }}>
                  <div style={{ display: "flex", justifyContent: "space-between" }}>
                    <div style={{ width: `${40 + i * 10}px`, height: 8, borderRadius: 4, background: "rgba(255,255,255,0.05)", animation: "pulse-dot 1.8s ease-in-out infinite" }} />
                    <div style={{ width: 28, height: 8, borderRadius: 4, background: "rgba(255,255,255,0.04)" }} />
                  </div>
                  <div style={{ height: 2, borderRadius: 9999, background: "rgba(255,255,255,0.04)" }} />
                </div>
              ))}
              <p style={{ margin: 0, fontSize: "0.7rem", color: "var(--muted)", textAlign: "center" }}>Start camera to stream biofield data</p>
            </div>
          )}
        </div>

        {/* ── WITNESS DYAD (middle, scrollable) ── */}
        <div style={{
          flex: 1,
          minHeight: 0,
          padding: "0.9rem 1.1rem",
          overflowY: "auto",
          display: "flex",
          flexDirection: "column",
          gap: "0.9rem",
        }}>
          {/* Header */}
          <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", flexShrink: 0 }}>
            <div style={{ display: "flex", alignItems: "center", gap: 6 }}>
              <span style={{ width: 5, height: 5, borderRadius: "50%", background: "var(--c-gold)", animation: witnessInsight ? "pulse-dot 1.8s ease-in-out infinite" : "none" }} />
              <p style={{ margin: 0, fontFamily: "var(--font-mono)", fontSize: "0.62rem", fontWeight: 600, letterSpacing: "0.14em", textTransform: "uppercase", color: "var(--muted)" }}>
                Witness Dyad
              </p>
            </div>
            {witnessInsight?.consciousness_level !== undefined && (
              <div style={{ display: "flex", alignItems: "center", gap: 5 }}>
                <span style={{ fontFamily: "var(--font-mono)", fontSize: "0.6rem", fontWeight: 600, letterSpacing: "0.08em", textTransform: "uppercase", color: "var(--muted)" }}>
                  Level
                </span>
                <span style={{ fontFamily: "var(--font-mono)", fontSize: "1.0rem", fontWeight: 700, letterSpacing: "-0.04em", color: "var(--c-indigo)" }}>
                  {witnessInsight.consciousness_level}
                </span>
              </div>
            )}
          </div>

          {witnessInsight && (hasDyad || dyadFallback) ? (
            hasDyad ? (
              <>
                {aletheios && (
                  <div style={{ display: "flex", flexDirection: "column", gap: "0.3rem" }}>
                    <p style={{ margin: 0, fontFamily: "var(--font-mono)", fontSize: "0.58rem", fontWeight: 700, letterSpacing: "0.14em", textTransform: "uppercase", color: "var(--c-indigo)", opacity: 0.85 }}>
                      Aletheios
                    </p>
                    <p style={{ margin: 0, fontSize: "0.88rem", lineHeight: 1.65, color: "var(--text-2)", borderLeft: "2px solid var(--c-indigo)", paddingLeft: "0.85rem", opacity: 0.85 }}>
                      {aletheios}
                    </p>
                  </div>
                )}
                {pichet && (
                  <div style={{ display: "flex", flexDirection: "column", gap: "0.3rem" }}>
                    <p style={{ margin: 0, fontFamily: "var(--font-mono)", fontSize: "0.58rem", fontWeight: 700, letterSpacing: "0.14em", textTransform: "uppercase", color: "var(--c-violet)", opacity: 0.85 }}>
                      Pichet
                    </p>
                    <p style={{ margin: 0, fontSize: "0.88rem", lineHeight: 1.65, color: "var(--text-2)", borderLeft: "2px solid var(--c-violet)", paddingLeft: "0.85rem", opacity: 0.85 }}>
                      {pichet}
                    </p>
                  </div>
                )}
                {synthesis && (
                  <p style={{
                    margin: 0, fontSize: "0.84rem", lineHeight: 1.65,
                    fontStyle: "italic", color: "var(--text-2)", opacity: 0.7,
                    paddingTop: "0.5rem", borderTop: "1px solid var(--line-faint)",
                  }}>
                    {synthesis}
                  </p>
                )}
              </>
            ) : (
              <p style={{
                margin: 0, fontSize: "0.92rem", lineHeight: 1.7,
                fontStyle: "italic", color: "var(--text-2)",
                borderLeft: "2px solid var(--accent-border)",
                paddingLeft: "0.9rem",
              }}>
                &ldquo;{dyadFallback}&rdquo;
              </p>
            )
          ) : (
            <div style={{
              flex: 1,
              display: "flex",
              alignItems: "center",
              justifyContent: "center",
              minHeight: 80,
            }}>
              <p style={{ margin: 0, fontSize: "0.78rem", color: "var(--muted)", textAlign: "center", lineHeight: 1.6, maxWidth: 220 }}>
                Witness perspectives appear here after the first biofield reading
              </p>
            </div>
          )}

          {/* Capture result — compact, inside dyad panel when present */}
          {captureResult && (
            <div style={{
              marginTop: "auto",
              padding: "0.8rem 0.9rem",
              borderRadius: "var(--r-lg)",
              background: "var(--panel)",
              border: "1px solid var(--line-faint)",
              display: "flex",
              flexDirection: "column",
              gap: "0.6rem",
            }}>
              <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
                <p style={{ margin: 0, fontFamily: "var(--font-mono)", fontSize: "0.6rem", fontWeight: 600, letterSpacing: "0.12em", textTransform: "uppercase", color: "var(--muted)" }}>
                  Last capture
                </p>
                <span style={{
                  fontFamily: "var(--font-mono)", fontSize: "0.58rem", fontWeight: 600, letterSpacing: "0.1em",
                  padding: "0.16rem 0.45rem", borderRadius: "var(--r-pill)",
                  background: captureResult.quality_assessment.sufficient_quality ? "rgba(99,102,241,0.1)" : "rgba(255,100,100,0.1)",
                  border: `1px solid ${captureResult.quality_assessment.sufficient_quality ? "rgba(99,102,241,0.22)" : "rgba(255,100,100,0.22)"}`,
                  color: captureResult.quality_assessment.sufficient_quality ? "var(--c-indigo)" : "#ff6464",
                }}>
                  {captureResult.quality_assessment.sufficient_quality ? "Accepted" : "Rejected"}
                </span>
              </div>
              <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: "0.35rem" }}>
                {METRIC_KEYS.map((key) => {
                  const raw = captureResult.metrics[key];
                  const num = typeof raw === "number" ? raw : parseFloat(String(raw));
                  const pct = !isNaN(num) ? Math.round(num * 100) : null;
                  return (
                    <div key={key} style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
                      <span style={{ fontFamily: "var(--font-mono)", fontSize: "0.58rem", color: "var(--muted)", textTransform: "uppercase", letterSpacing: "0.08em" }}>
                        {key.replace(/_/g, " ").replace(/^(light quanta|normalized|average|fractal|body|pattern)/, m => m.slice(0, 3))}
                      </span>
                      <span style={{ fontFamily: "var(--font-mono)", fontSize: "0.72rem", fontWeight: 700, color: "var(--text-2)" }}>
                        {pct !== null ? `${pct}%` : String(raw)}
                      </span>
                    </div>
                  );
                })}
              </div>
              <div style={{ display: "flex", gap: "0.5rem" }}>
                <Link className="biofield-link" href={`/readings/${captureResult.reading_id}`} style={{ fontSize: "0.72rem" }}>
                  Open reading
                </Link>
                <Link className="biofield-link" href="/history" style={{ fontSize: "0.72rem" }}>
                  History
                </Link>
              </div>
            </div>
          )}
        </div>

        {/* ── SESSION STRIP (bottom ~15%) ── */}
        <div style={{
          flex: "0 0 auto",
          padding: "0.75rem 1.1rem",
          borderTop: "1px solid var(--line-faint)",
          background: "var(--surface-2)",
          display: "flex",
          flexDirection: "column",
          gap: "0.55rem",
        }}>
          {/* Account / Session / Tier + Actions */}
          <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", gap: "0.75rem", flexWrap: "wrap" }}>
            <div style={{ display: "flex", alignItems: "center", gap: "1rem" }}>
              <div>
                <p style={{ margin: 0, fontFamily: "var(--font-mono)", fontSize: "0.56rem", fontWeight: 600, letterSpacing: "0.12em", textTransform: "uppercase", color: "var(--muted)" }}>Account</p>
                <p style={{ margin: 0, fontSize: "0.76rem", color: "var(--text)", maxWidth: 140, overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>{authSession?.email ?? "—"}</p>
              </div>
              <div style={{ width: 1, height: 24, background: "var(--line-faint)" }} />
              <div>
                <p style={{ margin: 0, fontFamily: "var(--font-mono)", fontSize: "0.56rem", fontWeight: 600, letterSpacing: "0.12em", textTransform: "uppercase", color: "var(--muted)" }}>Session</p>
                <p style={{ margin: 0, fontFamily: "var(--font-mono)", fontSize: "0.76rem", color: hasActiveSession ? "var(--c-emerald)" : "var(--muted)" }}>
                  {isHydratingSession ? "Restoring…" : currentSession ? currentSession.status : "—"}
                </p>
              </div>
              <div style={{ width: 1, height: 24, background: "var(--line-faint)" }} />
              <div>
                <p style={{ margin: 0, fontFamily: "var(--font-mono)", fontSize: "0.56rem", fontWeight: 600, letterSpacing: "0.12em", textTransform: "uppercase", color: "var(--muted)" }}>Tier</p>
                <p style={{ margin: 0, fontFamily: "var(--font-mono)", fontSize: "0.76rem", color: "var(--c-gold)" }}>{authSession?.tier ?? "—"}</p>
              </div>
            </div>
            <div className="biofield-actions" style={{ margin: 0 }}>
              <button className="biofield-button" style={{ fontSize: "0.72rem", padding: "0.3rem 0.8rem" }}
                disabled={isStartingSession || isHydratingSession || hasActiveSession}
                onClick={handleStartSession} type="button"
              >
                {isStartingSession ? "Starting…" : hasActiveSession ? "Active" : "Start session"}
              </button>
              <button className="biofield-link" style={{ fontSize: "0.72rem" }}
                disabled={isClosingSession || !hasActiveSession}
                onClick={handleCloseSession} type="button"
              >
                {isClosingSession ? "Closing…" : "End session"}
              </button>
            </div>
          </div>

          {/* Status / error messages */}
          {statusMessage && (
            <p style={{ margin: 0, fontFamily: "var(--font-mono)", fontSize: "0.68rem", color: "var(--c-emerald)" }}>{statusMessage}</p>
          )}
          {errorMessage && (
            <p style={{ margin: 0, fontFamily: "var(--font-mono)", fontSize: "0.68rem", color: "#ff6464" }}>{errorMessage}</p>
          )}
        </div>
      </div>
    </div>
  );
}
