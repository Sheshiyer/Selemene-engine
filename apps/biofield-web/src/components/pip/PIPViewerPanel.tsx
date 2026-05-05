"use client";

import { useCallback, useEffect, useRef, useState } from "react";
import { DEFAULT_PIP_SETTINGS } from "./types";
import type { CompositeScores } from "./types";
import { useCamera } from "./useCamera";
import { useMediaPipe } from "./useMediaPipe";
import { usePIPRenderer } from "./usePIPRenderer";
import { useLiveMetrics } from "./useLiveMetrics";

export interface PIPViewerPanelProps {
  onCapture?: (blob: Blob, dataUrl: string) => void;
  /** Called each time live metrics are refreshed (~10 fps). */
  onMetrics?: (scores: CompositeScores) => void;
  /** When true the canvas fills the parent container height instead of using 4:3 aspect ratio. */
  fillHeight?: boolean;
}

type PanelState = "idle" | "streaming" | "paused";

export function PIPViewerPanel({ onCapture, onMetrics, fillHeight }: PIPViewerPanelProps) {
  const canvasRef = useRef<HTMLCanvasElement | null>(null);
  const settingsRef = useRef(DEFAULT_PIP_SETTINGS);
  const onMetricsRef = useRef(onMetrics);
  onMetricsRef.current = onMetrics;

  const { videoRef, isStreaming, devices, error: cameraError, startCamera, stopCamera } = useCamera();
  const { status: glStatus, startRenderLoop, stopRenderLoop } = usePIPRenderer(canvasRef);
  const { ready: mpReady, error: mpError, segmentFrame, detectFace } = useMediaPipe();
  const { metrics, updateMetrics } = useLiveMetrics(canvasRef);

  const [panelState, setPanelState] = useState<PanelState>("idle");
  const [captureCount, setCaptureCount] = useState(0);
  const [captureError, setCaptureError] = useState<string | null>(null);

  const getMask = useCallback(() => {
    const video = videoRef.current;
    if (!mpReady || !video) return null;
    return segmentFrame(video);
  }, [mpReady, segmentFrame, videoRef]);

  const getFace = useCallback(() => {
    const video = videoRef.current;
    if (!mpReady || !video) return null;
    return detectFace(video);
  }, [mpReady, detectFace, videoRef]);

  // onFrame: called each RAF iteration; feeds MetricsCalculator and fires onMetrics callback.
  const onFrame = useCallback(
    (canvas: HTMLCanvasElement, mask: ReturnType<typeof getMask>, face: ReturnType<typeof getFace>) => {
      updateMetrics(canvas, mask, face);
    },
    [updateMetrics],
  );

  // Forward composite scores to parent whenever they update.
  useEffect(() => {
    if (metrics && onMetricsRef.current) {
      onMetricsRef.current(metrics.composite);
    }
  }, [metrics]);

  // Start render loop when camera, renderer, and MediaPipe are all ready.
  // stopRenderLoop before each start so the loop always uses the latest getMask/getFace
  // closures (getMask changes when mpReady transitions, and the RAF guard would otherwise
  // prevent the loop from picking up the new closure).
  useEffect(() => {
    const video = videoRef.current;
    if (isStreaming && glStatus === "ready" && mpReady && video && panelState === "streaming") {
      stopRenderLoop();
      startRenderLoop(video, settingsRef, getMask, getFace, onFrame);
    }
  }, [isStreaming, glStatus, mpReady, panelState, startRenderLoop, stopRenderLoop, getMask, getFace, onFrame, videoRef]);

  const handleStart = useCallback(async () => {
    setCaptureError(null);
    await startCamera();
    setPanelState("streaming");
  }, [startCamera]);

  const handlePause = useCallback(() => {
    stopRenderLoop();
    setPanelState("paused");
  }, [stopRenderLoop]);

  const handleResume = useCallback(() => {
    const video = videoRef.current;
    if (video && glStatus === "ready" && mpReady) {
      stopRenderLoop();
      startRenderLoop(video, settingsRef, getMask, getFace, onFrame);
    }
    setPanelState("streaming");
  }, [glStatus, mpReady, getMask, getFace, onFrame, startRenderLoop, stopRenderLoop, videoRef]);

  const handleStop = useCallback(() => {
    stopRenderLoop();
    stopCamera();
    setPanelState("idle");
  }, [stopRenderLoop, stopCamera]);

  const handleCapture = useCallback(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    setCaptureError(null);
    canvas.toBlob(
      (blob) => {
        if (!blob) {
          setCaptureError("Capture failed — canvas is empty.");
          return;
        }
        const reader = new FileReader();
        reader.onloadend = () => {
          const dataUrl = reader.result as string;
          onCapture?.(blob, dataUrl);
          setCaptureCount((n) => n + 1);
        };
        reader.readAsDataURL(blob);
      },
      "image/png",
    );
  }, [onCapture]);

  const glLabel =
    glStatus === "ready" ? "WebGL2 ✓" : glStatus === "error" ? "WebGL2 ✗" : "WebGL2 …";

  const mpLabel = mpError ? "MP ✗" : mpReady ? "MP ✓" : "MP …";

  const cameraLabel =
    panelState === "streaming" ? "streaming" : panelState === "paused" ? "paused" : "off";

  const deviceLabel =
    devices.length > 0 ? `${devices.length} camera${devices.length !== 1 ? "s" : ""}` : "no cameras";

  const c = metrics?.composite;

  return (
    <section
      className="biofield-panel biofield-form-panel"
      style={fillHeight ? { display: "flex", flexDirection: "column", height: "100%", padding: 0, gap: 0, borderRadius: 0, border: "none", background: "transparent" } : undefined}
    >
      {!fillHeight && <p className="biofield-eyebrow">PIP Viewer</p>}
      {!fillHeight && (
        <h2 className="biofield-title" style={{ fontSize: "2rem" }}>
          Live biofield capture
        </h2>
      )}

      {/* Status strip */}
      <div className="biofield-toolbar" style={{ marginBottom: "0.75rem" }}>
        <span
          className={`biofield-status-pill ${glStatus === "ready" ? "biofield-status-pill-good" : glStatus === "error" ? "" : "biofield-status-pill-warn"}`}
        >
          {glLabel}
        </span>
        <span
          className={`biofield-status-pill ${mpReady ? "biofield-status-pill-good" : mpError ? "" : "biofield-status-pill-warn"}`}
          title={mpError ?? undefined}
        >
          {mpLabel}
        </span>
        <span
          className={`biofield-status-pill ${panelState === "streaming" ? "biofield-status-pill-good" : panelState === "paused" ? "biofield-status-pill-warn" : ""}`}
        >
          Camera: {cameraLabel}
        </span>
        <span className="biofield-status-pill">{deviceLabel}</span>
        {captureCount > 0 && (
          <span className="biofield-status-pill biofield-status-pill-good">
            {captureCount} capture{captureCount !== 1 ? "s" : ""}
          </span>
        )}
      </div>

      {/* WebGL2 canvas */}
      <div
        style={{
          position: "relative",
          width: "100%",
          ...(fillHeight ? { flex: 1, minHeight: 0 } : { aspectRatio: "4/3" }),
          background: "rgba(0,0,0,0.4)",
          borderRadius: fillHeight ? 0 : 12,
          overflow: "hidden",
          marginBottom: fillHeight ? 0 : "0.75rem",
        }}
      >
        <video
          ref={videoRef}
          autoPlay
          playsInline
          muted
          style={{ position: "absolute", opacity: 0, pointerEvents: "none", width: 1, height: 1 }}
        />
        <canvas
          ref={canvasRef}
          style={{
            display: "block",
            width: "100%",
            height: "100%",
            transition: "box-shadow 0.5s ease",
            boxShadow: isStreaming && !mpReady
              ? "inset 0 0 0 2px rgba(124,124,255,0.45)"
              : "none",
            animation: isStreaming && !mpReady ? "pulse-canvas-border 1.8s ease-in-out infinite" : "none",
          }}
        />
        {panelState === "idle" && (
          <div
            style={{
              position: "absolute",
              inset: 0,
              display: "flex",
              alignItems: "center",
              justifyContent: "center",
              color: "var(--muted)",
              fontSize: "0.875rem",
            }}
          >
            Start camera to begin PIP visualisation
          </div>
        )}

        {/* Loading overlay while MediaPipe model is initialising (#676) */}
        {isStreaming && !mpReady && (
          <div
            style={{
              position: "absolute",
              inset: 0,
              display: "flex",
              flexDirection: "column",
              alignItems: "center",
              justifyContent: "center",
              gap: "0.5rem",
              background: "rgba(0,0,0,0.55)",
              color: "rgba(180,180,255,0.9)",
              fontSize: "0.8rem",
              pointerEvents: "none",
            }}
          >
            <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round">
              <path d="M12 2v4M12 18v4M4.93 4.93l2.83 2.83M16.24 16.24l2.83 2.83M2 12h4M18 12h4M4.93 19.07l2.83-2.83M16.24 7.76l2.83-2.83">
                <animateTransform attributeName="transform" type="rotate" from="0 12 12" to="360 12 12" dur="1.2s" repeatCount="indefinite" />
              </path>
            </svg>
            Calibrating biofield…
          </div>
        )}

        {/* Live metrics overlay */}
        {c && panelState !== "idle" && (
          <div
            style={{
              position: "absolute",
              bottom: 8,
              left: 8,
              right: 8,
              display: "grid",
              gridTemplateColumns: "1fr 1fr",
              gap: "4px 12px",
              fontSize: "0.7rem",
              color: "rgba(255,255,255,0.85)",
              textShadow: "0 1px 3px rgba(0,0,0,0.8)",
              pointerEvents: "none",
            }}
          >
            <MetricRow label="Coherence"  value={c.overallCoherence} />
            <MetricRow label="Symmetry"   value={c.bodySymmetry} />
            <MetricRow label="Luminance"  value={c.lightQuantaDensity} />
            <MetricRow label="Regularity" value={c.patternRegularity} />
          </div>
        )}
      </div>

      {/* Controls */}
      <div
        className="biofield-actions"
        style={fillHeight ? { padding: "8px 12px", gap: 8, flexShrink: 0, borderTop: "1px solid rgba(255,255,255,0.06)" } : undefined}
      >
        <button
          className="biofield-button"
          disabled={panelState !== "idle"}
          onClick={() => { void handleStart(); }}
          type="button"
        >
          Start camera
        </button>

        {panelState === "streaming" && (
          <button className="biofield-link" onClick={handlePause} type="button">
            Pause
          </button>
        )}

        {panelState === "paused" && (
          <button className="biofield-link" onClick={handleResume} type="button">
            Resume
          </button>
        )}

        <button
          className="biofield-button"
          disabled={panelState !== "streaming" && panelState !== "paused"}
          onClick={handleCapture}
          type="button"
        >
          Capture
        </button>

        <button
          className="biofield-link"
          disabled={panelState === "idle"}
          onClick={handleStop}
          type="button"
        >
          Stop
        </button>
      </div>

      {(cameraError ?? captureError) && (
        <p className="biofield-error" style={{ marginTop: "0.5rem", padding: fillHeight ? "0 12px" : undefined }}>
          {cameraError ?? captureError}
        </p>
      )}

      {!fillHeight && (
        <p className="biofield-copy" style={{ marginTop: "0.75rem" }}>
          Segmentation mask{mpReady ? " active" : " loading"} — biofield overlay is person-gated.
        </p>
      )}
    </section>
  );
}

// ─── Inline metric row ────────────────────────────────────────────────────────
function MetricRow({ label, value }: { label: string; value: number }) {
  const pct = Math.round(value * 100);
  return (
    <div style={{ display: "flex", alignItems: "center", gap: 6 }}>
      <span style={{ opacity: 0.7, minWidth: 70 }}>{label}</span>
      <div
        style={{
          flex: 1,
          height: 3,
          background: "rgba(255,255,255,0.15)",
          borderRadius: 2,
          overflow: "hidden",
        }}
      >
        <div
          style={{
            width: `${pct}%`,
            height: "100%",
            background: "rgba(120,220,180,0.9)",
            borderRadius: 2,
          }}
        />
      </div>
      <span style={{ minWidth: 30, textAlign: "right" }}>{pct}%</span>
    </div>
  );
}

