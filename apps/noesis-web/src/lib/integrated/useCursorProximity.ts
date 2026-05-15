"use client";

// ─── useCursorProximity ────────────────────────────────────────────────
// Tracks the mouse position globally and reports proximity to elements
// flagged with `data-proximity="cursor"`. Returns the *nearest* such
// element's distance and a boolean isNear (within thresholdPx).
//
// Performance: throttles via rAF; passive mousemove; reads
// getBoundingClientRect once per frame; no React state updates unless
// near-state actually flipped (or value crossed a 4px coarse-grain).
//
// Usage:
//   const { distance, isNear, target } = useCursorProximity({ thresholdPx: 80 });
//   if (isNear && target?.dataset.proximityRole === "yantra") { ... }

import { useEffect, useRef, useState } from "react";

export interface ProximityState {
  /** Distance in px from cursor to the nearest data-proximity element. Infinity if none. */
  distance: number;
  /** True if distance <= thresholdPx. */
  isNear: boolean;
  /** The nearest proximity element, if any. */
  target: HTMLElement | null;
}

export interface UseCursorProximityOpts {
  thresholdPx?: number;
  /** Quantize distance reporting to this many pixels (cheap re-renders). Default 8. */
  coarsenPx?: number;
  /** Optional CSS selector for elements to track. Defaults to [data-proximity]. */
  selector?: string;
}

const DEFAULTS: Required<UseCursorProximityOpts> = {
  thresholdPx: 80,
  coarsenPx: 8,
  selector: "[data-proximity]",
};

export function useCursorProximity(
  opts: UseCursorProximityOpts = {},
): ProximityState {
  const { thresholdPx, coarsenPx, selector } = { ...DEFAULTS, ...opts };

  const [state, setState] = useState<ProximityState>({
    distance: Infinity,
    isNear: false,
    target: null,
  });
  const rafRef = useRef<number | null>(null);
  const mouseRef = useRef<{ x: number; y: number } | null>(null);
  const lastReportedDistance = useRef<number>(Infinity);
  const lastTarget = useRef<HTMLElement | null>(null);

  useEffect(() => {
    if (typeof window === "undefined") return;

    const onMove = (e: MouseEvent) => {
      mouseRef.current = { x: e.clientX, y: e.clientY };
      if (rafRef.current !== null) return;
      rafRef.current = requestAnimationFrame(measure);
    };

    const measure = () => {
      rafRef.current = null;
      const mouse = mouseRef.current;
      if (!mouse) return;

      const els = document.querySelectorAll<HTMLElement>(selector);
      let bestDist = Infinity;
      let bestEl: HTMLElement | null = null;

      els.forEach((el) => {
        const r = el.getBoundingClientRect();
        // Skip elements outside the viewport entirely.
        if (
          r.bottom < -200 ||
          r.top > window.innerHeight + 200 ||
          r.right < -200 ||
          r.left > window.innerWidth + 200
        )
          return;
        // Compute distance from mouse to rect.
        const dx = mouse.x < r.left ? r.left - mouse.x : mouse.x > r.right ? mouse.x - r.right : 0;
        const dy = mouse.y < r.top ? r.top - mouse.y : mouse.y > r.bottom ? mouse.y - r.bottom : 0;
        const d = Math.hypot(dx, dy);
        if (d < bestDist) {
          bestDist = d;
          bestEl = el;
        }
      });

      const coarse = Math.round(bestDist / coarsenPx) * coarsenPx;
      const prevCoarse = Math.round(lastReportedDistance.current / coarsenPx) * coarsenPx;
      const targetChanged = bestEl !== lastTarget.current;

      if (coarse !== prevCoarse || targetChanged) {
        lastReportedDistance.current = bestDist;
        lastTarget.current = bestEl;
        setState({
          distance: bestDist,
          isNear: bestDist <= thresholdPx,
          target: bestEl,
        });
      }
    };

    window.addEventListener("mousemove", onMove, { passive: true });
    // Also rerun on scroll because rects move under the cursor.
    const onScroll = () => {
      if (!mouseRef.current) return;
      if (rafRef.current !== null) return;
      rafRef.current = requestAnimationFrame(measure);
    };
    window.addEventListener("scroll", onScroll, { passive: true });

    return () => {
      window.removeEventListener("mousemove", onMove);
      window.removeEventListener("scroll", onScroll);
      if (rafRef.current !== null) cancelAnimationFrame(rafRef.current);
    };
  }, [thresholdPx, coarsenPx, selector]);

  return state;
}
