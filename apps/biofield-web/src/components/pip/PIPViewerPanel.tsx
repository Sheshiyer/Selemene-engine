"use client";

import React, { useCallback, useEffect, useRef, useState } from "react";
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

  // Unified void-field container — same dark field as the right panel
  const containerStyle: React.CSSProperties = fillHeight
    ? { position: "relative", display: "flex", flexDirection: "column", height: "100%", width: "100%", background: "#070B1D", overflow: "hidden" }
    : { position: "relative", display: "flex", flexDirection: "column", background: "#070B1D", borderRadius: 12, overflow: "hidden" };

  return (
    <section style={containerStyle}>
      {/* ── Canvas — fills the void field ── */}
      <div style={{
        position: "relative",
        width: "100%",
        ...(fillHeight ? { flex: 1, minHeight: 0 } : { aspectRatio: "4/3" }),
        overflow: "hidden",
      }}>
        <video
          ref={videoRef}
          autoPlay playsInline muted
          style={{ position: "absolute", opacity: 0, pointerEvents: "none", width: 1, height: 1 }}
        />
        <canvas
          ref={canvasRef}
          style={{ display: "block", width: "100%", height: "100%" }}
        />

        {/* ── Idle state: sacred geometry placeholder ── */}
        {panelState === "idle" && <IdleGeometry />}

        {/* ── Calibrating overlay: spinning geometry ring ── */}
        {isStreaming && !mpReady && <CalibrationOverlay />}

        {/* ── System status — 3 floating geometry dots, top-right ── */}
        <div style={{
          position: "absolute", top: 14, right: 14,
          display: "flex", alignItems: "center", gap: 5,
          pointerEvents: "none",
        }}>
          {/* WebGL dot */}
          <StatusDot active={glStatus === "ready"} warn={glStatus === "idle"} title={glLabel} />
          {/* MediaPipe dot */}
          <StatusDot active={mpReady} warn={!mpReady && !mpError} title={mpLabel} />
          {/* Camera dot */}
          <StatusDot active={panelState === "streaming"} warn={panelState === "paused"} title={`Camera: ${cameraLabel}`} />
          {captureCount > 0 && (
            <span style={{
              fontFamily: "var(--font-mono)", fontSize: "0.48rem", letterSpacing: "0.1em",
              color: "rgba(16,181,167,0.55)", marginLeft: 2,
            }}>
              {captureCount}×
            </span>
          )}
        </div>

        {/* ── Live metric arcs — bottom overlay ── */}
        {c && panelState !== "idle" && (
          <LiveMetricArcs
            coherence={c.overallCoherence}
            symmetry={c.bodySymmetry}
            luminance={c.lightQuantaDensity}
            regularity={c.patternRegularity}
          />
        )}
      </div>

      {/* ── Controls — floating geometry nodes, no action bar ── */}
      <div style={{
        flexShrink: 0,
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        gap: "1.5rem",
        padding: "0.65rem 1rem",
        position: "relative",
      }}>
        {/* Thin gold geometry line above controls */}
        <div style={{
          position: "absolute", top: 0, left: "10%", right: "10%", height: 1,
          background: "linear-gradient(90deg, transparent 0%, rgba(197,160,23,0.15) 30%, rgba(197,160,23,0.15) 70%, transparent 100%)",
        }} />

        {panelState === "idle" ? (
          <GeometryButton onClick={() => { void handleStart(); }} label="OPEN FIELD" accent="gold" />
        ) : panelState === "paused" ? (
          <GeometryButton onClick={handleResume} label="RESUME" accent="indigo" />
        ) : null}

        {(panelState === "streaming" || panelState === "paused") && (
          <GeometryButton
            onClick={handleCapture}
            label="CAPTURE"
            accent="emerald"
          />
        )}

        {panelState === "streaming" && (
          <GeometryButton onClick={handlePause} label="PAUSE" accent="muted" />
        )}

        {panelState !== "idle" && (
          <GeometryButton onClick={handleStop} label="CLOSE" accent="muted" />
        )}
      </div>

      {/* Error — minimal inline, no card */}
      {(cameraError ?? captureError) && (
        <p style={{
          margin: "0 1rem 0.6rem",
          fontFamily: "var(--font-mono)", fontSize: "0.6rem",
          color: "rgba(198,93,59,0.75)", letterSpacing: "0.04em",
        }}>
          {cameraError ?? captureError}
        </p>
      )}
    </section>
  );
}

// ─── Geometry sub-components ─────────────────────────────────────────────────

/** Three-dot system status indicator */
function StatusDot({ active, warn, title }: { active: boolean; warn: boolean; title: string }) {
  const color = active ? "rgba(16,181,167,0.75)" : warn ? "rgba(197,160,23,0.55)" : "rgba(240,237,227,0.15)";
  const glow = active ? "0 0 5px rgba(16,181,167,0.4)" : warn ? "0 0 5px rgba(197,160,23,0.3)" : "none";
  return (
    <span
      title={title}
      style={{
        display: "inline-block",
        width: 4, height: 4, borderRadius: "50%",
        background: color,
        boxShadow: glow,
        transition: "all 0.4s ease",
      }}
    />
  );
}

/** Sacred geometry idle placeholder — animated concentric rings + lotus */
function IdleGeometry() {
  return (
    <div style={{
      position: "absolute", inset: 0,
      display: "flex", alignItems: "center", justifyContent: "center",
      pointerEvents: "none",
    }}>
      <svg
        viewBox="0 0 400 400"
        style={{ width: "min(55%, 220px)", height: "auto", opacity: 0.28 }}
      >
        {/* Concentric dot rings */}
        {[60, 100, 140, 180].map((r, ri) =>
          Array.from({ length: ri * 8 + 8 }).map((_, i, arr) => {
            const a = (i / arr.length) * Math.PI * 2;
            return (
              <circle
                key={`${r}-${i}`}
                cx={200 + r * Math.cos(a)}
                cy={200 + r * Math.sin(a)}
                r={1.2}
                fill="rgba(197,160,23,0.5)"
              />
            );
          })
        )}
        {/* Radial spokes */}
        {Array.from({ length: 12 }).map((_, i) => {
          const a = (i / 12) * Math.PI * 2;
          return (
            <line
              key={i}
              x1={200 + 50 * Math.cos(a)} y1={200 + 50 * Math.sin(a)}
              x2={200 + 185 * Math.cos(a)} y2={200 + 185 * Math.sin(a)}
              stroke="rgba(197,160,23,0.18)" strokeWidth="0.6"
            />
          );
        })}
        {/* Flower of life — 6 circles */}
        {Array.from({ length: 6 }).map((_, i) => {
          const a = (i / 6) * Math.PI * 2;
          return (
            <circle
              key={i}
              cx={200 + 30 * Math.cos(a)}
              cy={200 + 30 * Math.sin(a)}
              r={30}
              fill="none"
              stroke="rgba(197,160,23,0.35)"
              strokeWidth="0.7"
            />
          );
        })}
        <circle cx="200" cy="200" r="30" fill="none" stroke="rgba(197,160,23,0.35)" strokeWidth="0.7" />
        {/* Outer ring */}
        <circle cx="200" cy="200" r="185" fill="none" stroke="rgba(197,160,23,0.12)" strokeWidth="0.6" />
      </svg>
      {/* Label */}
      <p style={{
        position: "absolute", bottom: "18%",
        margin: 0,
        fontFamily: "var(--font-display)",
        fontSize: "0.52rem", fontWeight: 700,
        letterSpacing: "0.26em", textTransform: "uppercase",
        color: "rgba(240,237,227,0.2)",
      }}>
        Open Field to Begin
      </p>
    </div>
  );
}

/** Spinning geometry ring while MediaPipe calibrates */
function CalibrationOverlay() {
  return (
    <div style={{
      position: "absolute", inset: 0,
      display: "flex", flexDirection: "column",
      alignItems: "center", justifyContent: "center",
      gap: "0.85rem",
      background: "rgba(7,11,29,0.65)",
      pointerEvents: "none",
    }}>
      <svg width="48" height="48" viewBox="0 0 48 48" style={{ opacity: 0.7 }}>
        {/* Outer rotating arc */}
        <circle cx="24" cy="24" r="20" fill="none" stroke="rgba(197,160,23,0.15)" strokeWidth="1" />
        <path
          d="M 24 4 A 20 20 0 0 1 44 24"
          fill="none" stroke="rgba(197,160,23,0.6)" strokeWidth="1.2" strokeLinecap="round"
        >
          <animateTransform attributeName="transform" type="rotate" from="0 24 24" to="360 24 24" dur="1.6s" repeatCount="indefinite" />
        </path>
        {/* Inner dot */}
        <circle cx="24" cy="24" r="2.5" fill="rgba(16,181,167,0.6)">
          <animate attributeName="opacity" values="0.4;1;0.4" dur="1.6s" repeatCount="indefinite" />
        </circle>
      </svg>
      <span style={{
        fontFamily: "var(--font-display)",
        fontSize: "0.5rem", fontWeight: 700,
        letterSpacing: "0.26em", textTransform: "uppercase",
        color: "rgba(240,237,227,0.3)",
      }}>
        Calibrating
      </span>
    </div>
  );
}

/** 4 thin arc indicators in bottom-left of canvas — geometry encoding of live metrics */
function LiveMetricArcs({ coherence, symmetry, luminance, regularity }: {
  coherence: number; symmetry: number; luminance: number; regularity: number;
}) {
  const metrics = [
    { v: coherence,  color: "rgba(16,181,167,0.65)",  label: "COH" },
    { v: symmetry,   color: "rgba(197,160,23,0.65)",   label: "SYM" },
    { v: luminance,  color: "rgba(11,80,251,0.65)",    label: "LUM" },
    { v: regularity, color: "rgba(240,237,227,0.4)",   label: "REG" },
  ];
  const R = 14;
  const gap = 38;
  return (
    <svg
      style={{ position: "absolute", bottom: 14, left: 14, pointerEvents: "none", overflow: "visible" }}
      width={gap * 4}
      height={36}
    >
      {metrics.map(({ v, color, label }, i) => {
        const cx = i * gap + R + 2;
        const cy = R + 2;
        const pct = Math.min(1, Math.max(0, v));
        const arc = pct * Math.PI * 1.8; // 0 → 1.8π sweep
        const startA = -Math.PI * 0.9 + Math.PI / 2;
        const endA = startA + arc;
        const x1 = cx + R * Math.cos(startA - Math.PI / 2);
        const y1 = cy + R * Math.sin(startA - Math.PI / 2);
        const x2 = cx + R * Math.cos(endA - Math.PI / 2);
        const y2 = cy + R * Math.sin(endA - Math.PI / 2);
        const large = arc > Math.PI ? 1 : 0;
        return (
          <g key={label}>
            {/* Track ring */}
            <circle cx={cx} cy={cy} r={R} fill="none" stroke="rgba(240,237,227,0.06)" strokeWidth="1.2" />
            {/* Value arc */}
            {pct > 0.01 && (
              <path
                d={`M ${x1} ${y1} A ${R} ${R} 0 ${large} 1 ${x2} ${y2}`}
                fill="none" stroke={color} strokeWidth="1.5" strokeLinecap="round"
              />
            )}
            {/* Label */}
            <text
              x={cx} y={cy + R + 10}
              textAnchor="middle"
              style={{ fontFamily: "var(--font-mono)", fontSize: "0.38rem", fill: "rgba(240,237,227,0.3)", letterSpacing: "0.1em" }}
            >
              {label}
            </text>
          </g>
        );
      })}
    </svg>
  );
}

/** Minimal geometry node button */
function GeometryButton({ onClick, label, accent }: {
  onClick: () => void;
  label: string;
  accent: "gold" | "indigo" | "emerald" | "muted";
}) {
  const colors = {
    gold:    { text: "rgba(197,160,23,0.75)",   line: "rgba(197,160,23,0.3)"   },
    indigo:  { text: "rgba(11,80,251,0.85)",     line: "rgba(11,80,251,0.35)"   },
    emerald: { text: "rgba(16,181,167,0.75)",    line: "rgba(16,181,167,0.3)"   },
    muted:   { text: "rgba(240,237,227,0.28)",   line: "rgba(240,237,227,0.1)"  },
  };
  const { text, line } = colors[accent];
  return (
    <button
      onClick={onClick}
      type="button"
      style={{
        background: "none", border: "none", cursor: "pointer",
        display: "flex", flexDirection: "column", alignItems: "center", gap: 4,
        padding: "4px 8px",
      }}
    >
      {/* Diamond glyph */}
      <svg width="10" height="10" viewBox="0 0 10 10">
        <polygon points="5,0 10,5 5,10 0,5" fill="none" stroke={line} strokeWidth="0.8" />
        <circle cx="5" cy="5" r="1.5" fill={text} />
      </svg>
      <span style={{
        fontFamily: "var(--font-display)",
        fontSize: "0.46rem", fontWeight: 700,
        letterSpacing: "0.22em", textTransform: "uppercase",
        color: text,
      }}>
        {label}
      </span>
    </button>
  );
}

