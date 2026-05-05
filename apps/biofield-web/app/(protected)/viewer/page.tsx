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

      {/* ───── Right: consciousness data panel — Kha Arc background ───── */}
      <div style={{
        height: "100dvh",
        display: "flex",
        flexDirection: "column",
        borderLeft: "1px solid rgba(11,80,251,0.12)",
        background: "linear-gradient(180deg, #070B1D 0%, #0a0e20 60%, #0E1428 100%)",
        overflow: "hidden",
      }}>

        {/* ── COSMOGRAM — sacred geometry biofield display ── */}
        <div style={{
          flex: "1 1 0",
          minHeight: "42dvh",
          maxHeight: "72dvh",
          padding: "0.5rem 0.5rem 0.3rem",
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

        {/* ── WITNESS DYAD — ritual portal section ── */}
        <div style={{
          flex: 1,
          minHeight: 0,
          padding: "0.85rem 1rem",
          overflowY: "auto",
          display: "flex",
          flexDirection: "column",
          gap: "0.85rem",
        }}>
          {/* Section header */}
          <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", flexShrink: 0 }}>
            <div style={{ display: "flex", alignItems: "center", gap: 7 }}>
              <span style={{
                width: 5, height: 5, borderRadius: "50%",
                background: hasDyad ? "var(--c-gold)" : "rgba(240,237,227,0.15)",
                boxShadow: hasDyad ? "0 0 8px rgba(197,160,23,0.6)" : "none",
                animation: hasDyad ? "pulse-dot 1.8s ease-in-out infinite" : "none",
                transition: "all 0.5s ease",
              }} />
              <p style={{
                margin: 0,
                fontFamily: "var(--font-display)",
                fontSize: "0.62rem", fontWeight: 700,
                letterSpacing: "0.18em", textTransform: "uppercase",
                color: "var(--muted)",
              }}>
                Witness Dyad
              </p>
            </div>
            <div style={{ display: "flex", alignItems: "center", gap: 8 }}>
              {isLlmPowered && enginesUsed.length > 0 && (
                <span style={{
                  fontFamily: "var(--font-mono)", fontSize: "0.5rem",
                  color: "var(--c-emerald)", letterSpacing: "0.12em",
                  border: "1px solid rgba(16,181,167,0.3)",
                  borderRadius: 3, padding: "1px 5px",
                  textTransform: "uppercase",
                }}>
                  {enginesUsed.length} engines · AI
                </span>
              )}
              {witnessInsight?.consciousness_level !== undefined && (
                <div style={{ display: "flex", alignItems: "center", gap: 5 }}>
                  <span style={{
                    fontFamily: "var(--font-display)", fontSize: "0.55rem", fontWeight: 600,
                    letterSpacing: "0.12em", textTransform: "uppercase", color: "var(--muted)",
                  }}>
                    Level
                  </span>
                  <span style={{
                    fontFamily: "var(--font-mono)", fontSize: "1.05rem", fontWeight: 700,
                    letterSpacing: "-0.04em", color: "var(--c-indigo)",
                    textShadow: "0 0 14px rgba(11,80,251,0.6)",
                  }}>
                    {witnessInsight.consciousness_level}
                  </span>
                </div>
              )}
            </div>
          </div>

          {witnessInsight && (hasDyad || dyadFallback) ? (
            hasDyad ? (
              <>
                {aletheios && (
                  <div style={{
                    display: "flex", flexDirection: "column", gap: "0.35rem",
                    padding: "0.65rem 0.85rem",
                    borderRadius: "var(--r-md)",
                    background: "linear-gradient(135deg, rgba(11,80,251,0.06) 0%, rgba(7,11,29,0) 100%)",
                    border: "1px solid rgba(11,80,251,0.18)",
                  }}>
                    <div style={{ display: "flex", alignItems: "center", gap: 6 }}>
                      <span style={{ width: 3, height: 12, borderRadius: 2, background: "var(--c-indigo)", boxShadow: "0 0 6px rgba(11,80,251,0.6)", flexShrink: 0 }} />
                      <p style={{
                        margin: 0,
                        fontFamily: "var(--font-display)",
                        fontSize: "0.54rem", fontWeight: 700,
                        letterSpacing: "0.2em", textTransform: "uppercase",
                        color: "var(--c-indigo)",
                        textShadow: "0 0 10px rgba(11,80,251,0.5)",
                      }}>
                        Aletheios · Left Pillar
                      </p>
                    </div>
                    <p style={{
                      margin: 0,
                      fontFamily: "var(--font-body)",
                      fontSize: "0.82rem", lineHeight: 1.72,
                      color: "var(--text-2)",
                    }}>
                      {aletheios}
                    </p>
                  </div>
                )}
                {pichet && (
                  <div style={{
                    display: "flex", flexDirection: "column", gap: "0.35rem",
                    padding: "0.65rem 0.85rem",
                    borderRadius: "var(--r-md)",
                    background: "linear-gradient(135deg, rgba(197,160,23,0.05) 0%, rgba(7,11,29,0) 100%)",
                    border: "1px solid rgba(197,160,23,0.18)",
                  }}>
                    <div style={{ display: "flex", alignItems: "center", gap: 6 }}>
                      <span style={{ width: 3, height: 12, borderRadius: 2, background: "var(--c-gold)", boxShadow: "0 0 6px rgba(197,160,23,0.6)", flexShrink: 0 }} />
                      <p style={{
                        margin: 0,
                        fontFamily: "var(--font-display)",
                        fontSize: "0.54rem", fontWeight: 700,
                        letterSpacing: "0.2em", textTransform: "uppercase",
                        color: "var(--c-gold)",
                        textShadow: "0 0 10px rgba(197,160,23,0.5)",
                      }}>
                        Pichet · Right Pillar
                      </p>
                    </div>
                    <p style={{
                      margin: 0,
                      fontFamily: "var(--font-body)",
                      fontSize: "0.82rem", lineHeight: 1.72,
                      color: "var(--text-2)",
                    }}>
                      {pichet}
                    </p>
                  </div>
                )}
                {synthesis && (
                  <div style={{
                    padding: "0.65rem 0.85rem",
                    borderTop: "1px solid rgba(16,181,167,0.15)",
                  }}>
                    <p style={{
                      margin: 0,
                      fontFamily: "var(--font-body)",
                      fontSize: "0.82rem", lineHeight: 1.65,
                      fontStyle: "italic",
                      color: "var(--text-2)", opacity: 0.75,
                    }}>
                      {synthesis}
                    </p>
                  </div>
                )}
                {witnessQuestion && (
                  <div style={{
                    padding: "0.6rem 0.85rem",
                    borderRadius: "var(--r-md)",
                    background: "rgba(16,181,167,0.05)",
                    border: "1px solid rgba(16,181,167,0.18)",
                    marginTop: "0.25rem",
                  }}>
                    <p style={{
                      margin: 0,
                      fontFamily: "var(--font-body)",
                      fontSize: "0.78rem", lineHeight: 1.7,
                      color: "rgba(16,181,167,0.9)",
                    }}>
                      {witnessQuestion}
                    </p>
                  </div>
                )}
              </>
            ) : (
              <div style={{
                padding: "0.9rem 1rem",
                borderRadius: "var(--r-md)",
                background: "rgba(45,0,80,0.12)",
                border: "1px solid rgba(45,0,80,0.3)",
              }}>
                <p style={{
                  margin: 0,
                  fontFamily: "var(--font-body)",
                  fontSize: "0.9rem", lineHeight: 1.75,
                  fontStyle: "italic",
                  color: "var(--text-2)",
                }}>
                  &ldquo;{dyadFallback}&rdquo;
                </p>
              </div>
            )
          ) : (
            <div style={{
              flex: 1, display: "flex", alignItems: "center", justifyContent: "center", minHeight: 90,
            }}>
              <p style={{
                margin: 0,
                fontFamily: "var(--font-body)",
                fontSize: "0.78rem", color: "var(--muted)",
                textAlign: "center", lineHeight: 1.65, maxWidth: 200,
              }}>
                Witness perspectives appear here after the first biofield reading
              </p>
            </div>
          )}

          {/* Capture result — compact */}
          {captureResult && (
            <div style={{
              marginTop: "auto",
              padding: "0.75rem 0.85rem",
              borderRadius: "var(--r-md)",
              background: "rgba(11,80,251,0.05)",
              border: "1px solid rgba(11,80,251,0.12)",
              display: "flex", flexDirection: "column", gap: "0.5rem",
            }}>
              <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
                <p style={{
                  margin: 0, fontFamily: "var(--font-display)", fontSize: "0.58rem", fontWeight: 700,
                  letterSpacing: "0.14em", textTransform: "uppercase", color: "var(--muted)",
                }}>
                  Last capture
                </p>
                <span style={{
                  fontFamily: "var(--font-display)", fontSize: "0.54rem", fontWeight: 700, letterSpacing: "0.1em",
                  padding: "0.14rem 0.45rem", borderRadius: "var(--r-pill)",
                  background: captureResult.quality_assessment.sufficient_quality ? "rgba(16,181,167,0.1)" : "rgba(198,93,59,0.1)",
                  border: `1px solid ${captureResult.quality_assessment.sufficient_quality ? "rgba(16,181,167,0.25)" : "rgba(198,93,59,0.25)"}`,
                  color: captureResult.quality_assessment.sufficient_quality ? "var(--c-emerald)" : "var(--error)",
                  textShadow: captureResult.quality_assessment.sufficient_quality ? "0 0 8px rgba(16,181,167,0.4)" : "none",
                  textTransform: "uppercase",
                }}>
                  {captureResult.quality_assessment.sufficient_quality ? "Accepted" : "Rejected"}
                </span>
              </div>
              <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: "0.3rem" }}>
                {METRIC_KEYS.map((key) => {
                  const raw = captureResult.metrics[key];
                  const num = typeof raw === "number" ? raw : parseFloat(String(raw));
                  const pct = !isNaN(num) ? Math.round(num * 100) : null;
                  return (
                    <div key={key} style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
                      <span style={{ fontFamily: "var(--font-mono)", fontSize: "0.55rem", color: "var(--muted)", textTransform: "uppercase", letterSpacing: "0.08em" }}>
                        {key.replace(/_/g, " ").replace(/^(light quanta|normalized|average|fractal|body|pattern)/, m => m.slice(0, 3))}
                      </span>
                      <span style={{ fontFamily: "var(--font-mono)", fontSize: "0.7rem", fontWeight: 700, color: "var(--text-2)" }}>
                        {pct !== null ? `${pct}%` : String(raw)}
                      </span>
                    </div>
                  );
                })}
              </div>
              <div style={{ display: "flex", gap: "0.5rem" }}>
                <Link className="biofield-link" href={`/readings/${captureResult.reading_id}`} style={{ fontSize: "0.7rem" }}>
                  Open reading
                </Link>
                <Link className="biofield-link" href="/history" style={{ fontSize: "0.7rem" }}>
                  History
                </Link>
              </div>
            </div>
          )}
        </div>

        {/* ── SESSION STRIP — La Arc — bottom anchor ── */}
        <div style={{
          flex: "0 0 auto",
          padding: "0.7rem 1rem",
          borderTop: "1px solid rgba(11,80,251,0.1)",
          background: "linear-gradient(90deg, rgba(197,160,23,0.04) 0%, rgba(45,0,80,0.08) 50%, #070B1D 100%)",
          display: "flex", flexDirection: "column", gap: "0.5rem",
        }}>
          <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", gap: "0.75rem", flexWrap: "wrap" }}>
            {/* Account meta */}
            <div style={{ display: "flex", alignItems: "center", gap: "0.85rem" }}>
              <div>
                <p style={{ margin: 0, fontFamily: "var(--font-display)", fontSize: "0.5rem", fontWeight: 700, letterSpacing: "0.14em", textTransform: "uppercase", color: "var(--muted)" }}>Account</p>
                <p style={{ margin: 0, fontFamily: "var(--font-body)", fontSize: "0.73rem", color: "var(--text)", maxWidth: 130, overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
                  {authSession?.email ?? "—"}
                </p>
              </div>
              <div style={{ width: 1, height: 22, background: "rgba(11,80,251,0.15)" }} />
              <div>
                <p style={{ margin: 0, fontFamily: "var(--font-display)", fontSize: "0.5rem", fontWeight: 700, letterSpacing: "0.14em", textTransform: "uppercase", color: "var(--muted)" }}>Session</p>
                <p style={{ margin: 0, fontFamily: "var(--font-mono)", fontSize: "0.73rem", color: hasActiveSession ? "var(--c-emerald)" : "var(--muted)", textShadow: hasActiveSession ? "0 0 8px rgba(16,181,167,0.4)" : "none" }}>
                  {isHydratingSession ? "Restoring…" : currentSession ? currentSession.status : "—"}
                </p>
              </div>
              <div style={{ width: 1, height: 22, background: "rgba(11,80,251,0.15)" }} />
              <div>
                <p style={{ margin: 0, fontFamily: "var(--font-display)", fontSize: "0.5rem", fontWeight: 700, letterSpacing: "0.14em", textTransform: "uppercase", color: "var(--muted)" }}>Tier</p>
                <p style={{ margin: 0, fontFamily: "var(--font-mono)", fontSize: "0.73rem", color: "var(--c-gold)", textShadow: "0 0 8px rgba(197,160,23,0.35)" }}>
                  {authSession?.tier ?? "—"}
                </p>
              </div>
            </div>
            {/* Session actions */}
            <div className="biofield-actions" style={{ margin: 0 }}>
              <button
                className="biofield-button" style={{ fontSize: "0.7rem", padding: "0.28rem 0.75rem" }}
                disabled={isStartingSession || isHydratingSession || hasActiveSession}
                onClick={handleStartSession} type="button"
              >
                {isStartingSession ? "Starting…" : hasActiveSession ? "Active" : "Start session"}
              </button>
              <button
                className="biofield-link" style={{ fontSize: "0.7rem" }}
                disabled={isClosingSession || !hasActiveSession}
                onClick={handleCloseSession} type="button"
              >
                {isClosingSession ? "Closing…" : "End session"}
              </button>
            </div>
          </div>

          {statusMessage && (
            <p style={{ margin: 0, fontFamily: "var(--font-mono)", fontSize: "0.65rem", color: "var(--c-emerald)", textShadow: "0 0 8px rgba(16,181,167,0.35)" }}>{statusMessage}</p>
          )}
          {errorMessage && (
            <p style={{ margin: 0, fontFamily: "var(--font-mono)", fontSize: "0.65rem", color: "var(--error)" }}>{errorMessage}</p>
          )}
        </div>
      </div>
    </div>
  );
}
