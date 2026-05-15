"use client";

// ─── _shared — utilities for the 3D yantra variants ────────────────────
// Per W10 brief: every 3D variant needs WebGL detection (with graceful
// fallback to the W2 2D version) + prefers-reduced-motion observance.
//
// Kept as a thin module so each variant stays focused on its geometry.

import { useEffect, useState } from "react";

/** Detect WebGL2 (or WebGL) availability. SSR-safe. */
export function detectWebGL(): boolean {
  if (typeof window === "undefined") return false;
  try {
    const canvas = document.createElement("canvas");
    const gl = canvas.getContext("webgl2") || canvas.getContext("webgl");
    return !!gl;
  } catch {
    return false;
  }
}

/**
 * Tri-state WebGL hook:
 *   null      → SSR / pre-detect (caller should render the 2D fallback so
 *               hydration matches and there's no blank flash)
 *   false     → WebGL unavailable (caller falls back permanently)
 *   true      → safe to mount the Canvas
 */
export function useWebGL(): boolean | null {
  const [webgl, setWebgl] = useState<boolean | null>(null);
  useEffect(() => {
    setWebgl(detectWebGL());
  }, []);
  return webgl;
}

/** Watch prefers-reduced-motion. Mirrors CoverScene's pattern. */
export function usePrefersReducedMotion(): boolean {
  const [reduced, setReduced] = useState(false);
  useEffect(() => {
    if (typeof window === "undefined") return;
    const mq = window.matchMedia("(prefers-reduced-motion: reduce)");
    setReduced(mq.matches);
    const handler = (e: MediaQueryListEvent) => setReduced(e.matches);
    mq.addEventListener("change", handler);
    return () => mq.removeEventListener("change", handler);
  }, []);
  return reduced;
}

// ─── Brand tokens — bioluminescent palette ─────────────────────────────
// Mirrored from the CSS custom properties since Three.js materials want
// concrete hex strings, not var() references.
export const COLOR = {
  void: "#0A0814",
  parchment: "#F0EDE3",
  gold: "#C5A017", // Sacred Gold
  emerald: "#10B5A7", // Coherence Emerald
  violet: "#7D4BC8", // Witness Violet
  indigo: "#0B50FB",
} as const;
