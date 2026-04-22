"use client";

/**
 * usePIPRenderer — manages PIPRenderer lifecycle tied to a canvas ref.
 * Ported from bv-pip-analysis prototype.
 *
 * Key fix vs prototype: renderer.init() is called in a useEffect that runs
 * AFTER the canvas is mounted (not in the constructor), which resolves the
 * "shader not loading" / blank canvas issue seen in local dev.
 */

import { useState, useEffect, useRef, useCallback } from "react";
import { PIPRenderer } from "./PIPRenderer";
import type { PIPSettings } from "./types";
import { DEFAULT_PIP_SETTINGS } from "./types";

export interface UsePIPRendererReturn {
  canvasRef: React.RefObject<HTMLCanvasElement>;
  isReady: boolean;
  isRunning: boolean;
  error: string | null;
  settings: PIPSettings;
  attach: (video: HTMLVideoElement) => void;
  start: () => void;
  stop: () => void;
  pause: () => void;
  resume: () => void;
  setParameter: <K extends keyof PIPSettings>(key: K, value: PIPSettings[K]) => void;
  setMask: (mask: Uint8Array | null, width: number, height: number) => void;
  captureFrameAsDataURL: () => string | null;
}

export function usePIPRenderer(): UsePIPRendererReturn {
  const canvasRef = useRef<HTMLCanvasElement>(null) as React.RefObject<HTMLCanvasElement>;
  const rendererRef = useRef<PIPRenderer | null>(null);

  const [isReady, setIsReady] = useState(false);
  const [isRunning, setIsRunning] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [settings, setSettings] = useState<PIPSettings>({ ...DEFAULT_PIP_SETTINGS });

  // Initialize renderer once the canvas element is in the DOM.
  // This effect runs after the first paint — safe to call getContext('webgl2').
  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    let renderer: PIPRenderer | null = null;
    try {
      renderer = new PIPRenderer(canvas);
      renderer.init();
      rendererRef.current = renderer;
      setIsReady(true);
      setError(null);
    } catch (e) {
      setError(e instanceof Error ? e.message : "WebGL init failed");
    }

    return () => {
      renderer?.destroy();
      rendererRef.current = null;
      setIsReady(false);
      setIsRunning(false);
    };
  }, []);

  const attach = useCallback((video: HTMLVideoElement) => {
    rendererRef.current?.setVideoSource(video);
  }, []);

  const start = useCallback(() => {
    rendererRef.current?.start();
    setIsRunning(true);
  }, []);

  const stop = useCallback(() => {
    rendererRef.current?.stop();
    setIsRunning(false);
  }, []);

  const pause = useCallback(() => {
    rendererRef.current?.pause();
    setIsRunning(false);
  }, []);

  const resume = useCallback(() => {
    rendererRef.current?.resume();
    setIsRunning(true);
  }, []);

  const setParameter = useCallback(
    <K extends keyof PIPSettings>(key: K, value: PIPSettings[K]) => {
      rendererRef.current?.setParameter(key, value);
      setSettings((prev) => ({ ...prev, [key]: value }));
    },
    []
  );

  const setMask = useCallback((mask: Uint8Array | null, width: number, height: number) => {
    rendererRef.current?.setMask(mask, width, height);
  }, []);

  const captureFrameAsDataURL = useCallback((): string | null => {
    return rendererRef.current?.captureFrameAsDataURL() ?? null;
  }, []);

  return {
    canvasRef,
    isReady,
    isRunning,
    error,
    settings,
    attach,
    start,
    stop,
    pause,
    resume,
    setParameter,
    setMask,
    captureFrameAsDataURL,
  };
}
