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
import { buildApiUrl } from "@/lib/config";
import { useRouter } from "next/navigation";
import { PIPViewerPanel } from "@/components/pip/PIPViewerPanel";
import { BiofieldCosmogram } from "@/components/BiofieldCosmogram";
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

  // Structured Witness Dyad from the multi-engine LLM interpretation endpoint.
  const [witnessDyad, setWitnessDyad] = useState<{
    aletheios: string;
    pichet: string;
    synthesis: string;
    witness_question: string;
    engines_used: string[];
    llm_powered: boolean;
  } | null>(null);

  // User profile (birth data for engine calculations)
  const [userBirthData, setUserBirthData] = useState<{
    birth_date: string | null;
    birth_time: string | null;
    birth_location_lat: number | null;
    birth_location_lng: number | null;
    timezone: string | null;
  } | null>(null);

  // Track last metrics submission timestamp to rate-limit engine calls.
  const lastMetricsSubmitRef = useRef<number>(0);

  useEffect(() => {
    if (!authSession) {
      router.replace("/login");
    }
  }, [authSession, router]);

  // Load user profile once to get birth data for multi-engine witness interpretation.
  useEffect(() => {
    if (!authSession?.token) return;
    fetch(buildApiUrl("/api/v1/users/me"), {
      headers: { Authorization: `Bearer ${authSession.token}` },
    })
      .then((r) => r.ok ? r.json() : null)
      .then((data) => {
        if (data) {
          setUserBirthData({
            birth_date: data.birth_date ?? null,
            birth_time: data.birth_time ?? null,
            birth_location_lat: data.birth_location?.lat ?? null,
            birth_location_lng: data.birth_location?.lng ?? null,
            timezone: data.timezone ?? null,
          });
        }
      })
      .catch(() => { /* profile unavailable — witness still works without birth data */ });
  }, [authSession?.token]);

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

    if (!noesisClient || !authSession?.token) return;

    const now = performance.now();
    if (now - lastMetricsSubmitRef.current < METRICS_SUBMIT_INTERVAL_MS) return;
    lastMetricsSubmitRef.current = now;

    // 1. Biofield engine call (keeps cosmogram data fresh)
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
      console.warn("[BF1-05.6] biofield engine error:", err instanceof Error ? err.message : err);
    });

    // 2. Multi-engine LLM witness interpret — uses birth data + all engines
    const birthData = userBirthData?.birth_date ? {
      date: userBirthData.birth_date,
      latitude: userBirthData.birth_location_lat ?? 0,
      longitude: userBirthData.birth_location_lng ?? 0,
      timezone: userBirthData.timezone ?? "UTC",
      ...(userBirthData.birth_time ? { time: userBirthData.birth_time } : {}),
    } : undefined;

    void fetch(buildApiUrl("/api/v1/witness/interpret"), {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        Authorization: `Bearer ${authSession.token}`,
      },
      body: JSON.stringify({
        birth_data: birthData,
        live_scores: {
          energy: scores.lightQuantaDensity,
          coherence: scores.overallCoherence,
          symmetry: scores.bodySymmetry,
          complexity: scores.patternRegularity,
          regulation: scores.normalizedArea,
          color_balance: (scores.overallCoherence + scores.bodySymmetry) / 2,
        },
        consciousness_level: 0,
      }),
    })
      .then((r) => r.ok ? r.json() : null)
      .then((dyad) => { if (dyad) setWitnessDyad(dyad); })
      .catch((err) => {
        console.warn("[witness-interpret] error:", err instanceof Error ? err.message : err);
      });
  }, [noesisClient, authSession?.token, userBirthData]);

  const hasActiveSession = currentSession?.status === "active";

  // ── Witness Dyad data — prefer LLM result, fall back to rule-based ──────────
  // witnessDyad = rich multi-engine LLM interpretation (when OPENAI_API_KEY set)
  // witnessInsight = biofield engine result (always available, rule-based fallback)
  const wl = witnessInsight?.result?.witness_layer as {
    aletheios?: { perspective?: string };
    pichet?: { perspective?: string };
    synthesis?: string;
    witness_question?: string;
  } | undefined;

  const aletheios = witnessDyad?.aletheios ?? wl?.aletheios?.perspective;
  const pichet = witnessDyad?.pichet ?? wl?.pichet?.perspective;
  const synthesis = witnessDyad?.synthesis ?? wl?.synthesis;
  const witnessQuestion = witnessDyad?.witness_question ?? wl?.witness_question;
  const enginesUsed = witnessDyad?.engines_used ?? [];
  const isLlmPowered = witnessDyad?.llm_powered ?? false;
  const dyadFallback = witnessInsight?.witness_prompt;
  const hasDyad = aletheios || pichet || synthesis;

  return (
    /* ══════════════════════════════════════════════════════════════
       WitnessOS Viewer — void-field, geometry-first layout
       No borders, no cards. Geometry defines space.
       ══════════════════════════════════════════════════════════════ */
    <div style={{
      position: "fixed", inset: 0,
      width: "100vw", height: "100dvh",
      overflow: "hidden",
      display: "grid",
      gridTemplateColumns: "58fr 42fr",
      background: "#070B1D",
    }}>

      {/* ───── Left: full-height camera / biofield portal ───── */}
      <div style={{ height: "100dvh", overflow: "hidden", position: "relative" }}>
        <PIPViewerPanel onCapture={handlePIPCapture} onMetrics={handleMetrics} fillHeight />
      </div>

      {/* ───── Right: consciousness data column — pure void field ───── */}
      {/* Thin geometry rule replaces border-left */}
      <div style={{
        height: "100dvh",
        display: "flex",
        flexDirection: "column",
        position: "relative",
        background: "#070B1D",
        overflow: "hidden",
      }}>
        {/* Vertical divider — thin gold geometry line, not a CSS border */}
        <div style={{
          position: "absolute", left: 0, top: "8%", bottom: "8%", width: 1,
          background: "linear-gradient(180deg, transparent 0%, rgba(197,160,23,0.22) 20%, rgba(197,160,23,0.22) 80%, transparent 100%)",
          pointerEvents: "none",
        }} />

        {/* ── COSMOGRAM — sacred geometry biofield display ── */}
        <div style={{
          flex: "1 1 0",
          minHeight: "42dvh",
          maxHeight: "72dvh",
          overflow: "hidden",
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
        }}>
          <BiofieldCosmogram
            scores={liveScores ?? {
              lightQuantaDensity: 0,
              normalizedArea: 0,
              bodySymmetry: 0.5,
              patternRegularity: 0.5,
              overallCoherence: 0,
            }}
            engineResult={witnessInsight?.result as Record<string, unknown> | null}
          />
        </div>

        {/* ── Thin geometry separator between cosmogram and dyad ── */}
        <div style={{
          flexShrink: 0,
          margin: "0 1.4rem",
          height: 1,
          background: "linear-gradient(90deg, transparent 0%, rgba(197,160,23,0.18) 25%, rgba(16,181,167,0.12) 75%, transparent 100%)",
        }} />

        {/* ── WITNESS DYAD — floating text, no cards ── */}
        <div style={{
          flex: 1,
          minHeight: 0,
          padding: "0.75rem 1.4rem 0.5rem",
          overflowY: "auto",
          display: "flex",
          flexDirection: "column",
          gap: "0.6rem",
        }}>
          {/* Dyad header — label + status badges */}
          <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", flexShrink: 0 }}>
            <div style={{ display: "flex", alignItems: "center", gap: 8 }}>
              {/* Pulsing presence dot */}
              <span style={{
                width: 4, height: 4, borderRadius: "50%",
                background: hasDyad ? "rgba(197,160,23,0.9)" : "rgba(240,237,227,0.12)",
                boxShadow: hasDyad ? "0 0 6px rgba(197,160,23,0.5)" : "none",
                animation: hasDyad ? "pulse-dot 2s ease-in-out infinite" : "none",
                transition: "all 0.5s ease", flexShrink: 0,
              }} />
              <span style={{
                fontFamily: "var(--font-display)", fontSize: "0.52rem", fontWeight: 700,
                letterSpacing: "0.22em", textTransform: "uppercase",
                color: "rgba(240,237,227,0.3)",
              }}>
                Witness Dyad
              </span>
            </div>
            <div style={{ display: "flex", alignItems: "center", gap: 10 }}>
              {isLlmPowered && enginesUsed.length > 0 && (
                <span style={{
                  fontFamily: "var(--font-mono)", fontSize: "0.46rem",
                  color: "rgba(16,181,167,0.6)", letterSpacing: "0.14em",
                  textTransform: "uppercase",
                }}>
                  {enginesUsed.length} engines
                </span>
              )}
              {witnessInsight?.consciousness_level !== undefined && (
                <span style={{
                  fontFamily: "var(--font-mono)", fontSize: "0.76rem", fontWeight: 700,
                  letterSpacing: "-0.04em", color: "rgba(11,80,251,0.75)",
                  textShadow: "0 0 12px rgba(11,80,251,0.4)",
                }}>
                  {witnessInsight.consciousness_level}
                </span>
              )}
            </div>
          </div>

          {witnessInsight && (hasDyad || dyadFallback) ? (
            hasDyad ? (
              <>
                {aletheios && (
                  <div style={{ display: "flex", flexDirection: "column", gap: "0.28rem" }}>
                    {/* Agent label — spaced caps, no box */}
                    <div style={{ display: "flex", alignItems: "center", gap: 7 }}>
                      {/* Diamond glyph */}
                      <svg width="6" height="6" viewBox="0 0 6 6" style={{ flexShrink: 0, opacity: 0.7 }}>
                        <polygon points="3,0 6,3 3,6 0,3" fill="none" stroke="rgba(11,80,251,0.7)" strokeWidth="0.8" />
                      </svg>
                      <span style={{
                        fontFamily: "var(--font-display)", fontSize: "0.5rem", fontWeight: 700,
                        letterSpacing: "0.22em", textTransform: "uppercase",
                        color: "rgba(11,80,251,0.65)",
                      }}>
                        Aletheios · Left Pillar
                      </span>
                    </div>
                    <p style={{
                      margin: 0, fontFamily: "var(--font-body)",
                      fontSize: "0.8rem", lineHeight: 1.72,
                      color: "rgba(240,237,227,0.68)",
                      paddingLeft: "0.85rem",
                    }}>
                      {aletheios}
                    </p>
                  </div>
                )}
                {pichet && (
                  <div style={{ display: "flex", flexDirection: "column", gap: "0.28rem", marginTop: "0.3rem" }}>
                    <div style={{ display: "flex", alignItems: "center", gap: 7 }}>
                      <svg width="6" height="6" viewBox="0 0 6 6" style={{ flexShrink: 0, opacity: 0.7 }}>
                        <polygon points="3,0 6,3 3,6 0,3" fill="none" stroke="rgba(197,160,23,0.7)" strokeWidth="0.8" />
                      </svg>
                      <span style={{
                        fontFamily: "var(--font-display)", fontSize: "0.5rem", fontWeight: 700,
                        letterSpacing: "0.22em", textTransform: "uppercase",
                        color: "rgba(197,160,23,0.65)",
                      }}>
                        Pichet · Right Pillar
                      </span>
                    </div>
                    <p style={{
                      margin: 0, fontFamily: "var(--font-body)",
                      fontSize: "0.8rem", lineHeight: 1.72,
                      color: "rgba(240,237,227,0.68)",
                      paddingLeft: "0.85rem",
                    }}>
                      {pichet}
                    </p>
                  </div>
                )}
                {synthesis && (
                  <>
                    {/* Thin emerald geometry line before synthesis */}
                    <div style={{
                      margin: "0.15rem 0",
                      height: 1,
                      background: "linear-gradient(90deg, rgba(16,181,167,0.12) 0%, rgba(16,181,167,0.06) 100%)",
                    }} />
                    <p style={{
                      margin: 0, fontFamily: "var(--font-body)",
                      fontSize: "0.78rem", lineHeight: 1.65,
                      fontStyle: "italic",
                      color: "rgba(240,237,227,0.44)",
                    }}>
                      {synthesis}
                    </p>
                  </>
                )}
                {witnessQuestion && (
                  <p style={{
                    margin: "0.1rem 0 0",
                    fontFamily: "var(--font-body)",
                    fontSize: "0.76rem", lineHeight: 1.7,
                    color: "rgba(16,181,167,0.72)",
                    fontStyle: "italic",
                  }}>
                    {witnessQuestion}
                  </p>
                )}
              </>
            ) : (
              <p style={{
                margin: 0, fontFamily: "var(--font-body)",
                fontSize: "0.88rem", lineHeight: 1.75,
                fontStyle: "italic",
                color: "rgba(240,237,227,0.5)",
              }}>
                &ldquo;{dyadFallback}&rdquo;
              </p>
            )
          ) : (
            <div style={{ flex: 1, display: "flex", alignItems: "center", justifyContent: "center", minHeight: 80 }}>
              <p style={{
                margin: 0, fontFamily: "var(--font-body)",
                fontSize: "0.72rem", color: "rgba(240,237,227,0.22)",
                textAlign: "center", lineHeight: 1.7, maxWidth: 180,
                letterSpacing: "0.02em",
              }}>
                Witness perspectives emerge after the first biofield reading
              </p>
            </div>
          )}

          {/* Capture result — minimal, no card */}
          {captureResult && (
            <div style={{ marginTop: "auto", paddingTop: "0.5rem", display: "flex", flexDirection: "column", gap: "0.35rem" }}>
              {/* Thin separator */}
              <div style={{ height: 1, background: "linear-gradient(90deg, rgba(11,80,251,0.1) 0%, transparent 100%)" }} />
              <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
                <span style={{
                  fontFamily: "var(--font-display)", fontSize: "0.48rem", fontWeight: 700,
                  letterSpacing: "0.18em", textTransform: "uppercase", color: "rgba(240,237,227,0.25)",
                }}>
                  Last capture
                </span>
                <span style={{
                  fontFamily: "var(--font-mono)", fontSize: "0.52rem", letterSpacing: "0.1em",
                  textTransform: "uppercase",
                  color: captureResult.quality_assessment.sufficient_quality ? "rgba(16,181,167,0.7)" : "rgba(198,93,59,0.7)",
                }}>
                  {captureResult.quality_assessment.sufficient_quality ? "✓ Accepted" : "✗ Rejected"}
                </span>
              </div>
              <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: "0.2rem 0.8rem" }}>
                {METRIC_KEYS.map((key) => {
                  const raw = captureResult.metrics[key];
                  const num = typeof raw === "number" ? raw : parseFloat(String(raw));
                  const pct = !isNaN(num) ? Math.round(num * 100) : null;
                  return (
                    <div key={key} style={{ display: "flex", justifyContent: "space-between", alignItems: "baseline" }}>
                      <span style={{ fontFamily: "var(--font-mono)", fontSize: "0.5rem", color: "rgba(240,237,227,0.28)", textTransform: "uppercase", letterSpacing: "0.07em" }}>
                        {key.replace(/_/g, " ").replace(/^(light quanta|normalized|average|fractal|body|pattern)/, m => m.slice(0, 3))}
                      </span>
                      <span style={{ fontFamily: "var(--font-mono)", fontSize: "0.68rem", fontWeight: 700, color: "rgba(240,237,227,0.6)" }}>
                        {pct !== null ? `${pct}%` : String(raw)}
                      </span>
                    </div>
                  );
                })}
              </div>
              <div style={{ display: "flex", gap: "1rem" }}>
                <Link className="biofield-link" href={`/readings/${captureResult.reading_id}`} style={{ fontSize: "0.66rem" }}>
                  Open reading
                </Link>
                <Link className="biofield-link" href="/history" style={{ fontSize: "0.66rem" }}>
                  History
                </Link>
              </div>
            </div>
          )}
        </div>

        {/* ── SESSION STRIP — minimal anchor, geometry line above ── */}
        <div style={{
          flex: "0 0 auto",
          padding: "0.6rem 1.4rem 0.7rem",
          display: "flex", flexDirection: "column", gap: "0.4rem",
          position: "relative",
        }}>
          {/* Thin gold geometry line — replaces solid border-top */}
          <div style={{
            position: "absolute", top: 0, left: "1.4rem", right: "1.4rem", height: 1,
            background: "linear-gradient(90deg, transparent 0%, rgba(197,160,23,0.18) 30%, rgba(197,160,23,0.18) 70%, transparent 100%)",
          }} />

          <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", gap: "0.75rem", flexWrap: "wrap" }}>
            {/* Account meta — spaced, no divider bars */}
            <div style={{ display: "flex", alignItems: "baseline", gap: "1.2rem" }}>
              <span style={{
                fontFamily: "var(--font-body)", fontSize: "0.7rem",
                color: "rgba(240,237,227,0.55)",
                maxWidth: 140, overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap",
              }}>
                {authSession?.email ?? "—"}
              </span>
              <span style={{
                fontFamily: "var(--font-mono)", fontSize: "0.68rem",
                color: hasActiveSession ? "rgba(16,181,167,0.7)" : "rgba(240,237,227,0.28)",
                textShadow: hasActiveSession ? "0 0 8px rgba(16,181,167,0.3)" : "none",
              }}>
                {isHydratingSession ? "restoring…" : currentSession ? currentSession.status : "—"}
              </span>
              <span style={{
                fontFamily: "var(--font-mono)", fontSize: "0.68rem",
                color: "rgba(197,160,23,0.65)",
                letterSpacing: "0.04em",
              }}>
                {authSession?.tier ?? "—"}
              </span>
            </div>
            {/* Session actions */}
            <div className="biofield-actions" style={{ margin: 0 }}>
              <button
                className="biofield-button" style={{ fontSize: "0.66rem", padding: "0.24rem 0.7rem" }}
                disabled={isStartingSession || isHydratingSession || hasActiveSession}
                onClick={handleStartSession} type="button"
              >
                {isStartingSession ? "Starting…" : hasActiveSession ? "Active" : "Start session"}
              </button>
              <button
                className="biofield-link" style={{ fontSize: "0.66rem" }}
                disabled={isClosingSession || !hasActiveSession}
                onClick={handleCloseSession} type="button"
              >
                {isClosingSession ? "Closing…" : "End session"}
              </button>
            </div>
          </div>

          {statusMessage && (
            <p style={{ margin: 0, fontFamily: "var(--font-mono)", fontSize: "0.6rem", color: "rgba(16,181,167,0.6)" }}>{statusMessage}</p>
          )}
          {errorMessage && (
            <p style={{ margin: 0, fontFamily: "var(--font-mono)", fontSize: "0.6rem", color: "rgba(198,93,59,0.8)" }}>{errorMessage}</p>
          )}
        </div>
      </div>
    </div>
  );
}
