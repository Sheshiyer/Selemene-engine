"use client";

import { useCallback, useEffect, useRef, useState } from "react";
import { DEFAULT_PIP_SETTINGS } from "./types";
import { useCamera } from "./useCamera";
import { usePIPRenderer } from "./usePIPRenderer";

export interface PIPViewerPanelProps {
  onCapture?: (blob: Blob, dataUrl: string) => void;
}

type PanelState = "idle" | "streaming" | "paused";

export function PIPViewerPanel({ onCapture }: PIPViewerPanelProps) {
  const canvasRef = useRef<HTMLCanvasElement | null>(null);
  const settingsRef = useRef(DEFAULT_PIP_SETTINGS);

  const { videoRef, isStreaming, devices, error: cameraError, startCamera, stopCamera } = useCamera();
  const { status: glStatus, startRenderLoop, stopRenderLoop } = usePIPRenderer(canvasRef);

  const [panelState, setPanelState] = useState<PanelState>("idle");
  const [captureCount, setCaptureCount] = useState(0);
  const [captureError, setCaptureError] = useState<string | null>(null);

  // Start render loop when both camera and renderer are ready.
  useEffect(() => {
    const video = videoRef.current;
    if (isStreaming && glStatus === "ready" && video && panelState === "streaming") {
      startRenderLoop(video, settingsRef);
    }
  }, [isStreaming, glStatus, panelState, startRenderLoop, videoRef]);

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
    if (video && glStatus === "ready") {
      startRenderLoop(video, settingsRef);
    }
    setPanelState("streaming");
  }, [glStatus, startRenderLoop, videoRef]);

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
          // BF1-05.2 will wire this to POST /api/v1/biofield/sessions/:id/captures
          console.info("[PIPViewer] capture", {
            engineId: "biofield-pip",
            size: blob.size,
            width: canvas.width,
            height: canvas.height,
          });
          onCapture?.(blob, dataUrl);
          setCaptureCount((n) => n + 1);
        };
        reader.readAsDataURL(blob);
      },
      "image/png",
    );
  }, [onCapture]);

  const glLabel =
    glStatus === "ready"
      ? "WebGL2 ✓"
      : glStatus === "error"
        ? "WebGL2 ✗"
        : "WebGL2 …";

  const cameraLabel =
    panelState === "streaming"
      ? "streaming"
      : panelState === "paused"
        ? "paused"
        : "off";

  const deviceLabel =
    devices.length > 0 ? `${devices.length} camera${devices.length !== 1 ? "s" : ""}` : "no cameras";

  return (
    <section className="biofield-panel biofield-form-panel">
      <p className="biofield-eyebrow">PIP Viewer</p>
      <h2 className="biofield-title" style={{ fontSize: "2rem" }}>
        Live biofield capture
      </h2>

      {/* Status strip */}
      <div className="biofield-toolbar" style={{ marginBottom: "0.75rem" }}>
        <span
          className={`biofield-status-pill ${glStatus === "ready" ? "biofield-status-pill-good" : glStatus === "error" ? "" : "biofield-status-pill-warn"}`}
        >
          {glLabel}
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

      {/* WebGL2 canvas — hidden video element feeds it each frame */}
      <div
        style={{
          position: "relative",
          width: "100%",
          aspectRatio: "4/3",
          background: "rgba(0,0,0,0.4)",
          borderRadius: 12,
          overflow: "hidden",
          marginBottom: "0.75rem",
        }}
      >
        {/* Hidden video element — camera source for WebGL2 texture */}
        <video
          ref={videoRef}
          autoPlay
          playsInline
          muted
          style={{ position: "absolute", opacity: 0, pointerEvents: "none", width: 1, height: 1 }}
        />
        <canvas
          ref={canvasRef}
          style={{ display: "block", width: "100%", height: "100%" }}
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
      </div>

      {/* Controls */}
      <div className="biofield-actions">
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
        <p className="biofield-error" style={{ marginTop: "0.5rem" }}>
          {cameraError ?? captureError}
        </p>
      )}

      <p className="biofield-copy" style={{ marginTop: "0.75rem" }}>
        Captures save locally as PNG.{" "}
        <span className="biofield-mono" style={{ fontSize: "0.75em", opacity: 0.7 }}>
          API upload wired in BF1-05.2
        </span>
      </p>
    </section>
  );
}
