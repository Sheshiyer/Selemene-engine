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

  // Unified void-field container — centered circle portal
  const containerStyle: React.CSSProperties = fillHeight
    ? { position: "relative", display: "flex", flexDirection: "column", alignItems: "center", justifyContent: "center", height: "100%", width: "100%", background: "#070B1D", overflow: "hidden" }
    : { position: "relative", display: "flex", flexDirection: "column", alignItems: "center", justifyContent: "center", background: "#070B1D", borderRadius: 12, overflow: "hidden" };

  return (
    <section style={containerStyle}>
      {/* ── Circular portal — canvas clipped to circle ── */}
      {/* Outer wrapper: sizes the portal, provides anchor for ring overlay */}
      <div style={{
        position: "relative",
        // circle size: largest square that fits, leaving room for controls + ring halo
        width: "min(80%, calc(100dvh - 80px))",
        aspectRatio: "1 / 1",
        flexShrink: 0,
      }}>
        {/* Concentric ring halo — drawn OUTSIDE the clipped circle */}
        <PortalRings
          streaming={panelState === "streaming"}
          metrics={c ? { coherence: c.overallCoherence, symmetry: c.bodySymmetry, luminance: c.lightQuantaDensity, regularity: c.patternRegularity } : null}
        />

        {/* Clipped circular canvas */}
        <div style={{
          position: "absolute",
          inset: "10%", // ring halo lives in the 10% margin around the circle
          borderRadius: "50%",
          overflow: "hidden",
          background: "#070B1D",
        }}>
          <video
            ref={videoRef}
            autoPlay playsInline muted
            style={{ position: "absolute", opacity: 0, pointerEvents: "none", width: 1, height: 1 }}
          />
          <canvas
            ref={canvasRef}
            style={{ display: "block", width: "100%", height: "100%", objectFit: "cover" }}
          />

          {/* Idle state: sacred geometry placeholder */}
          {panelState === "idle" && <IdleGeometry />}

          {/* Calibrating overlay */}
          {isStreaming && !mpReady && <CalibrationOverlay />}

          {/* Subtle vignette to blend circle edges into void */}
          <div style={{
            position: "absolute", inset: 0,
            borderRadius: "50%",
            background: "radial-gradient(circle, transparent 55%, rgba(7,11,29,0.55) 80%, rgba(7,11,29,0.9) 100%)",
            pointerEvents: "none",
          }} />
        </div>

        {/* Status dots — float at top of the outer ring, centered */}
        <div style={{
          position: "absolute", top: "5%", left: "50%",
          transform: "translateX(-50%)",
          display: "flex", alignItems: "center", gap: 6,
          pointerEvents: "none",
        }}>
          <StatusDot active={glStatus === "ready"} warn={glStatus === "idle"} title={glLabel} />
          <StatusDot active={mpReady} warn={!mpReady && !mpError} title={mpLabel} />
          <StatusDot active={panelState === "streaming"} warn={panelState === "paused"} title={`Camera: ${cameraLabel}`} />
          {captureCount > 0 && (
            <span style={{
              fontFamily: "var(--font-mono)", fontSize: "0.44rem", letterSpacing: "0.12em",
              color: "rgba(16,181,167,0.5)",
            }}>
              {captureCount}×
            </span>
          )}
        </div>
      </div>

      {/* ── Controls — float below the circle, centered ── */}
      <div style={{
        flexShrink: 0,
        display: "flex", alignItems: "center", justifyContent: "center",
        gap: "2rem",
        padding: "0.7rem 1rem 0.5rem",
      }}>
        {panelState === "idle" ? (
          <GeometryButton onClick={() => { void handleStart(); }} label="OPEN FIELD" accent="gold" />
        ) : panelState === "paused" ? (
          <GeometryButton onClick={handleResume} label="RESUME" accent="indigo" />
        ) : null}

        {(panelState === "streaming" || panelState === "paused") && (
          <GeometryButton onClick={handleCapture} label="CAPTURE" accent="emerald" />
        )}

        {panelState === "streaming" && (
          <GeometryButton onClick={handlePause} label="PAUSE" accent="muted" />
        )}

        {panelState !== "idle" && (
          <GeometryButton onClick={handleStop} label="CLOSE" accent="muted" />
        )}
      </div>

      {/* Error */}
      {(cameraError ?? captureError) && (
        <p style={{
          margin: "0 1rem 0.5rem",
          fontFamily: "var(--font-mono)", fontSize: "0.58rem",
          color: "rgba(198,93,59,0.75)", letterSpacing: "0.04em",
          textAlign: "center",
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
        background: color, boxShadow: glow,
        transition: "all 0.4s ease",
      }}
    />
  );
}

/**
 * Portal ring system — SVG overlay that fills 100% of the outer wrapper.
 * The circle canvas lives in the inner 80% (inset 10%); the ring lives in
 * the 10% halo on each side + extends 5% beyond for the outermost arc.
 * Metric arcs are drawn in the ring band between r=42% and r=48% of total size.
 */
function PortalRings({
  streaming,
  metrics,
}: {
  streaming: boolean;
  metrics: { coherence: number; symmetry: number; luminance: number; regularity: number } | null;
}) {
  // SVG coordinate system: 200×200, circle lives at r=72 (inset 10% of 100% → circle r=80 in SVG units of 200/2)
  const C = 100; // center
  const innerR = 72; // edge of the clipped circle portal
  const ringR1 = 78; // inner edge of ring band
  const ringR2 = 90; // outer edge of ring band
  const dotRings = [82, 86, 92]; // concentric dot ring radii

  // Metric arcs — drawn in the ring band, each occupying 90° sectors
  const metricDefs = metrics ? [
    { v: metrics.coherence,  color: "rgba(16,181,167,0.7)",  startDeg: -135, spanDeg: 90 },
    { v: metrics.symmetry,   color: "rgba(197,160,23,0.7)",  startDeg: -45,  spanDeg: 90 },
    { v: metrics.luminance,  color: "rgba(11,80,251,0.7)",   startDeg: 45,   spanDeg: 90 },
    { v: metrics.regularity, color: "rgba(240,237,227,0.4)", startDeg: 135,  spanDeg: 90 },
  ] : [];

  function arcPath(cx: number, cy: number, r: number, startDeg: number, endDeg: number): string {
    const toRad = (d: number) => (d * Math.PI) / 180;
    const x1 = cx + r * Math.cos(toRad(startDeg));
    const y1 = cy + r * Math.sin(toRad(startDeg));
    const x2 = cx + r * Math.cos(toRad(endDeg));
    const y2 = cy + r * Math.sin(toRad(endDeg));
    const large = Math.abs(endDeg - startDeg) > 180 ? 1 : 0;
    return `M ${x1} ${y1} A ${r} ${r} 0 ${large} 1 ${x2} ${y2}`;
  }

  return (
    <svg
      viewBox="0 0 200 200"
      style={{
        position: "absolute", inset: 0, width: "100%", height: "100%",
        pointerEvents: "none", overflow: "visible",
      }}
    >
      {/* Thin circle that frames the portal edge */}
      <circle cx={C} cy={C} r={innerR}
        fill="none"
        stroke={streaming ? "rgba(197,160,23,0.35)" : "rgba(197,160,23,0.12)"}
        strokeWidth="0.5"
      />

      {/* Track ring (full circle outlines in the halo band) */}
      {dotRings.map((r) => (
        <circle key={r} cx={C} cy={C} r={r}
          fill="none"
          stroke="rgba(197,160,23,0.07)"
          strokeWidth="0.4"
        />
      ))}

      {/* Concentric dot pattern on the ring band */}
      {dotRings.map((r) =>
        Array.from({ length: Math.round(r * 0.9) }).map((_, i, arr) => {
          const a = (i / arr.length) * Math.PI * 2;
          return (
            <circle
              key={`${r}-${i}`}
              cx={C + r * Math.cos(a)}
              cy={C + r * Math.sin(a)}
              r={0.5}
              fill="rgba(197,160,23,0.25)"
            />
          );
        })
      )}

      {/* Radial spokes — 12 thin lines from circle edge outward */}
      {Array.from({ length: 12 }).map((_, i) => {
        const a = (i / 12) * Math.PI * 2;
        return (
          <line
            key={i}
            x1={C + (innerR + 1) * Math.cos(a)} y1={C + (innerR + 1) * Math.sin(a)}
            x2={C + (ringR2 + 2) * Math.cos(a)} y2={C + (ringR2 + 2) * Math.sin(a)}
            stroke="rgba(197,160,23,0.12)" strokeWidth="0.4"
          />
        );
      })}

      {/* Metric arc fills — occupy ring band between ringR1 and ringR2 */}
      {metricDefs.map(({ v, color, startDeg, spanDeg }, i) => {
        const sweepDeg = spanDeg * Math.min(1, Math.max(0, v));
        const endDeg = startDeg + sweepDeg;
        const trackEnd = startDeg + spanDeg - 1;
        const arcR = ringR1 + (ringR2 - ringR1) / 2; // midline
        return (
          <g key={i}>
            {/* Track */}
            <path d={arcPath(C, C, arcR, startDeg, trackEnd)} fill="none" stroke="rgba(240,237,227,0.05)" strokeWidth="3.5" />
            {/* Value fill */}
            {sweepDeg > 0.5 && (
              <path d={arcPath(C, C, arcR, startDeg, endDeg - 0.5)} fill="none" stroke={color} strokeWidth="3.5" strokeLinecap="round" />
            )}
          </g>
        );
      })}

      {/* Metric labels at 45° sector midpoints */}
      {metrics && [
        { label: "COH", deg: -90, color: "rgba(16,181,167,0.5)"  },
        { label: "SYM", deg:   0, color: "rgba(197,160,23,0.5)"  },
        { label: "LUM", deg:  90, color: "rgba(11,80,251,0.6)"   },
        { label: "REG", deg: 180, color: "rgba(240,237,227,0.3)" },
      ].map(({ label, deg, color }) => {
        const a = (deg * Math.PI) / 180;
        const lr = ringR2 + 6;
        return (
          <text
            key={label}
            x={C + lr * Math.cos(a)} y={C + lr * Math.sin(a)}
            textAnchor="middle" dominantBaseline="middle"
            style={{ fontFamily: "var(--font-mono)", fontSize: "3.5px", fill: color, letterSpacing: "0.5px" }}
          >
            {label}
          </text>
        );
      })}
    </svg>
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

