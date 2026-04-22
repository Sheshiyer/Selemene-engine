"use client";

/**
 * PIPViewerPanel — live camera feed with WebGL2 PIP biofield shader.
 *
 * Architecture:
 *   <video hidden> → WebGL2 PIPRenderer → <canvas> (visible)
 *
 * The hidden <video> feeds frames into the WebGL texture each rAF.
 * No ML segmentation in this phase — that's BF1-05 wave 2.
 */

import { useEffect, useCallback, useState } from "react";
import { useCamera } from "./useCamera";
import { usePIPRenderer } from "./usePIPRenderer";
import { useSegmentation } from "./useSegmentation";
import { MetricsCalculator, type CompositeScores, type FrameMetrics } from "./MetricsCalculator";

interface PIPViewerPanelProps {
  /** Called with a PNG data-url when the user captures a frame */
  onCapture?: (payload: {
    dataUrl: string;
    maskDataUrl: string | null;
    timestamp: number;
    frameMetrics: FrameMetrics | null;
    compositeScores: CompositeScores | null;
  }) => void;
}

export function PIPViewerPanel({ onCapture }: PIPViewerPanelProps) {
  const camera = useCamera({ width: 640, height: 480 });
  const pip = usePIPRenderer();
  const segmentation = useSegmentation();
  const [capturing, setCapturing] = useState(false);
  const [frameMetrics, setFrameMetrics] = useState<FrameMetrics | null>(null);
  const [compositeScores, setCompositeScores] = useState<CompositeScores | null>(null);

  const metricsRef = useState(() => new MetricsCalculator())[0];

  // Once the camera is playing and the renderer is ready, wire them together
  useEffect(() => {
    if (camera.isPlaying && camera.videoRef.current && pip.isReady) {
      pip.attach(camera.videoRef.current);
      pip.start();
    }
  }, [camera.isPlaying, pip.isReady]); // eslint-disable-line react-hooks/exhaustive-deps

  // Run segmentation loop and feed mask texture into shader
  useEffect(() => {
    if (!camera.isPlaying || !camera.videoRef.current || !segmentation.isReady) return;
    let mounted = true;

    const tick = async () => {
      if (!mounted || !camera.videoRef.current) return;
      await segmentation.process(camera.videoRef.current);
      if (segmentation.mask && segmentation.width && segmentation.height) {
        pip.setMask(segmentation.mask, segmentation.width, segmentation.height);
      }
      setTimeout(tick, 180);
    };

    tick();
    return () => {
      mounted = false;
    };
  }, [camera.isPlaying, segmentation.isReady]); // eslint-disable-line react-hooks/exhaustive-deps

  // Compute real-time metrics from rendered canvas
  useEffect(() => {
    if (!camera.isPlaying || !pip.canvasRef.current) return;
    let mounted = true;

    const canvas = pip.canvasRef.current;
    const ctx = canvas.getContext("2d", { willReadFrequently: true });
    if (!ctx) return;

    const loop = () => {
      if (!mounted) return;
      try {
        const imageData = ctx.getImageData(0, 0, canvas.width, canvas.height);
        const metrics = metricsRef.calculateFromImageData(imageData, segmentation.mask ?? undefined);
        const scores = metricsRef.calculateScores(metrics);
        setFrameMetrics(metrics);
        setCompositeScores(scores);
      } catch {
        // Ignore occasional readback timing errors while canvas resizes.
      }
      setTimeout(loop, 350);
    };

    loop();
    return () => {
      mounted = false;
    };
  }, [camera.isPlaying, segmentation.mask]); // eslint-disable-line react-hooks/exhaustive-deps

  const buildMaskDataUrl = useCallback((): string | null => {
    if (!segmentation.mask || !segmentation.width || !segmentation.height) {
      return null;
    }

    const maskCanvas = document.createElement("canvas");
    maskCanvas.width = segmentation.width;
    maskCanvas.height = segmentation.height;
    const mctx = maskCanvas.getContext("2d");
    if (!mctx) return null;

    const img = mctx.createImageData(segmentation.width, segmentation.height);
    for (let i = 0; i < segmentation.mask.length; i++) {
      const v = segmentation.mask[i];
      const idx = i * 4;
      img.data[idx] = v;
      img.data[idx + 1] = v;
      img.data[idx + 2] = v;
      img.data[idx + 3] = 255;
    }
    mctx.putImageData(img, 0, 0);
    return maskCanvas.toDataURL("image/png", 0.95);
  }, [segmentation.mask, segmentation.width, segmentation.height]);

  const handleStart = useCallback(async () => {
    await camera.start();
  }, [camera]);

  const handlePauseResume = useCallback(() => {
    if (camera.isPlaying) {
      camera.pause();
      pip.pause();
    } else {
      camera.resume();
      pip.resume();
    }
  }, [camera, pip]);

  const handleCapture = useCallback(() => {
    setCapturing(true);
    camera.pause();
    pip.pause();

    const dataUrl = pip.captureFrameAsDataURL();
    const maskDataUrl = buildMaskDataUrl();
    if (dataUrl && onCapture) {
      onCapture({
        dataUrl,
        maskDataUrl,
        timestamp: Date.now(),
        frameMetrics,
        compositeScores,
      });
    }

    setCapturing(false);
  }, [camera, pip, onCapture, buildMaskDataUrl, frameMetrics, compositeScores]);

  const handleStop = useCallback(() => {
    pip.stop();
    camera.stop();
  }, [camera, pip]);

  return (
    <div className="biofield-viewer-panel">
      {/* Hidden live video element feeds WebGL texture */}
      {/* eslint-disable-next-line jsx-a11y/media-has-caption */}
      <video
        ref={camera.videoRef}
        style={{ display: "none" }}
        playsInline
        muted
      />

      {/* PIP shader output canvas */}
      <div className="biofield-canvas-wrapper">
        {pip.error ? (
          <div className="biofield-error">
            <p className="biofield-kicker">WebGL Error</p>
            <p className="biofield-copy">{pip.error}</p>
          </div>
        ) : (
          <canvas
            ref={pip.canvasRef}
            className="biofield-canvas"
            width={640}
            height={480}
          />
        )}

        {!camera.isPlaying && !camera.isLoading && !pip.error && (
          <div className="biofield-canvas-overlay">
            <p className="biofield-kicker">PIP Viewer</p>
            <p className="biofield-copy">
              Camera and segmentation run locally. No video is uploaded.
            </p>
          </div>
        )}
      </div>

      {/* Controls */}
      <div className="biofield-controls">
        {!camera.stream ? (
          <button
            className="biofield-btn biofield-btn--primary"
            onClick={handleStart}
            disabled={camera.isLoading}
          >
            {camera.isLoading ? "Starting…" : "Start Camera"}
          </button>
        ) : (
          <>
            <button
              className="biofield-btn"
              onClick={handlePauseResume}
            >
              {camera.isPlaying ? "Pause" : "Resume"}
            </button>
            <button
              className="biofield-btn biofield-btn--primary"
              onClick={handleCapture}
              disabled={!camera.isPlaying || capturing}
            >
              {capturing ? "Capturing…" : "Capture"}
            </button>
            <button
              className="biofield-btn biofield-btn--ghost"
              onClick={handleStop}
            >
              Stop
            </button>
          </>
        )}

        {camera.error && (
          <p className="biofield-error-inline">{camera.error}</p>
        )}
      </div>

      {/* Device selector (shown once devices are enumerated) */}
      {camera.devices.length > 1 && (
        <div className="biofield-device-select">
          <label className="biofield-kicker" htmlFor="camera-device">
            Camera
          </label>
          <select
            id="camera-device"
            value={camera.selectedDevice ?? ""}
            onChange={(e) => camera.selectDevice(e.target.value)}
          >
            {camera.devices.map((d) => (
              <option key={d.deviceId} value={d.deviceId}>
                {d.label}
              </option>
            ))}
          </select>
        </div>
      )}

      {/* Status strip */}
      <div className="biofield-status-strip">
        <span className={`biofield-status-dot ${pip.isRunning ? "active" : ""}`} />
        <span className="biofield-copy">
          {pip.isRunning
            ? "PIP rendering"
            : camera.stream
            ? "Paused"
            : "Idle"}
        </span>
        {pip.isReady && !pip.error && (
          <span className="biofield-kicker" style={{ marginLeft: "auto" }}>
            WebGL2 ✓
          </span>
        )}
      </div>

      {(frameMetrics || compositeScores) && (
        <div className="biofield-metrics-inline">
          <p className="biofield-kicker">Live Metrics</p>
          <div className="biofield-grid" style={{ gridTemplateColumns: "repeat(3, minmax(0, 1fr))", gap: "0.45rem" }}>
            <div className="biofield-mini-metric">
              <span>LQD</span>
              <strong>{frameMetrics ? frameMetrics.lightQuantaDensity.toFixed(3) : "-"}</strong>
            </div>
            <div className="biofield-mini-metric">
              <span>Symmetry</span>
              <strong>{compositeScores ? compositeScores.symmetry.toFixed(1) : "-"}</strong>
            </div>
            <div className="biofield-mini-metric">
              <span>Coherence</span>
              <strong>{compositeScores ? compositeScores.coherence.toFixed(1) : "-"}</strong>
            </div>
          </div>
          <p className="biofield-copy" style={{ margin: "0.2rem 0 0" }}>
            Mask: {segmentation.bodyDetected ? "body" : "none"} · Face: {segmentation.faceDetected ? "detected" : "none"}
          </p>
        </div>
      )}
    </div>
  );
}
