"use client";

import { RefObject, useCallback, useEffect, useRef, useState } from "react";
import { MetricsCalculator } from "./MetricsCalculator";
import type { MediaPipeMask, MediaPipeFaceResult } from "./useMediaPipe";
import type { FrameMetrics, CompositeScores } from "./types";

export interface LiveMetrics {
  frame: FrameMetrics;
  composite: CompositeScores;
}

export interface UseLiveMetricsResult {
  metrics: LiveMetrics | null;
  /** Call this each RAF frame (or at whatever rate your render loop runs). */
  updateMetrics: (
    canvas: HTMLCanvasElement,
    mask: MediaPipeMask | null,
    face: MediaPipeFaceResult | null,
  ) => void;
}

/**
 * Wraps MetricsCalculator in a React hook.
 * `updateMetrics` should be called each frame from the render loop.
 * State is updated at most every 100 ms (throttled inside MetricsCalculator).
 */
export function useLiveMetrics(
  canvasRef: RefObject<HTMLCanvasElement | null>,
): UseLiveMetricsResult {
  const calcRef = useRef<MetricsCalculator | null>(null);
  const [metrics, setMetrics] = useState<LiveMetrics | null>(null);

  useEffect(() => {
    calcRef.current = new MetricsCalculator();
    return () => {
      calcRef.current?.dispose();
      calcRef.current = null;
    };
  }, []);

  const updateMetrics = useCallback(
    (
      canvas: HTMLCanvasElement,
      mask: MediaPipeMask | null,
      face: MediaPipeFaceResult | null,
    ) => {
      const calc = calcRef.current;
      if (!calc) return;
      const result = calc.compute(canvas, mask, face);
      if (result) {
        setMetrics(result);
      }
    },
    [],
  );

  // Expose a stable no-op when canvas isn't mounted yet so callers don't need to guard.
  const _ = canvasRef; // keep dep to satisfy linter; ref change triggers nothing here

  return { metrics, updateMetrics };
}
