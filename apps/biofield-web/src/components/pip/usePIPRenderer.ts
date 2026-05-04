"use client";

import { RefObject, useCallback, useEffect, useRef, useState } from "react";
import { PIPRenderer } from "./PIPRenderer";
import type { MediaPipeMask } from "./useMediaPipe";
import type { PIPSettings } from "./types";

export type RendererStatus = "idle" | "ready" | "error";

export interface UsePIPRendererResult {
  status: RendererStatus;
  startRenderLoop: (
    video: HTMLVideoElement,
    settingsRef: RefObject<PIPSettings>,
    getMask: () => MediaPipeMask | null,
  ) => void;
  stopRenderLoop: () => void;
}

export function usePIPRenderer(
  canvasRef: RefObject<HTMLCanvasElement | null>,
): UsePIPRendererResult {
  const rendererRef = useRef<PIPRenderer | null>(null);
  const rafRef = useRef<number | null>(null);
  const [status, setStatus] = useState<RendererStatus>("idle");

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const renderer = new PIPRenderer(canvas);
    const ok = renderer.init();
    rendererRef.current = renderer;
    setStatus(ok ? "ready" : "error");

    return () => {
      if (rafRef.current !== null) {
        cancelAnimationFrame(rafRef.current);
        rafRef.current = null;
      }
      renderer.dispose();
      rendererRef.current = null;
      setStatus("idle");
    };
  }, [canvasRef]);

  const startRenderLoop = useCallback(
    (
      video: HTMLVideoElement,
      settingsRef: RefObject<PIPSettings>,
      getMask: () => MediaPipeMask | null,
    ) => {
      if (rafRef.current !== null) return;

      const startTime = performance.now();

      function loop() {
        const renderer = rendererRef.current;
        if (!renderer) return;

        const settings = settingsRef.current;
        if (settings) {
          renderer.render(video, performance.now() - startTime, settings, getMask());
        }
        rafRef.current = requestAnimationFrame(loop);
      }

      rafRef.current = requestAnimationFrame(loop);
    },
    [],
  );

  const stopRenderLoop = useCallback(() => {
    if (rafRef.current !== null) {
      cancelAnimationFrame(rafRef.current);
      rafRef.current = null;
    }
  }, []);

  return { status, startRenderLoop, stopRenderLoop };
}
