"use client";

// ─── CursorProximityScene ──────────────────────────────────────────────
// Drives the cursor-proximity tactility layer:
//   1. Auto-tags `<strong>` and `<b>` inside `.verse-flow` with
//      `data-proximity="strong"` (MutationObserver watches for new ones).
//   2. Reads `useCursorProximity()` to find the nearest data-proximity
//      element and toggles `data-proximity-near="true"` on it.
//   3. CSS rules (globals.css addendum) provide the visual response:
//        - strong text → subtle Sacred Gold text-shadow
//        - yantra plates → tiny scale + brightness bump
//
// Respects prefers-reduced-motion: when reduced, attribute is never set
// and transitions in CSS effectively go to zero duration.

import { useEffect, useRef } from "react";
import { useReducedMotion } from "motion/react";
import { useCursorProximity } from "@/lib/integrated/useCursorProximity";

interface Props {
  /** Containing root for verse-flow content. Defaults to `.verse-flow`. */
  verseRootSelector?: string;
  /** Cursor proximity threshold (px). */
  thresholdPx?: number;
}

export function CursorProximityScene({
  verseRootSelector = ".verse-flow",
  thresholdPx = 80,
}: Props) {
  const reduced = useReducedMotion();
  const lastNearRef = useRef<HTMLElement | null>(null);

  // Tag all <strong> / <b> inside verse-flow as proximity targets.
  useEffect(() => {
    if (typeof document === "undefined") return;

    const tag = (root: Element) => {
      const candidates = root.querySelectorAll<HTMLElement>("strong, b");
      candidates.forEach((el) => {
        if (!el.dataset.proximity) {
          el.dataset.proximity = "strong";
        }
      });
    };

    const tagAll = () => {
      document.querySelectorAll(verseRootSelector).forEach(tag);
    };

    tagAll();

    // Observe additions (lazy/scroll-revealed content, hot-replace, etc).
    const observers: MutationObserver[] = [];
    document.querySelectorAll(verseRootSelector).forEach((root) => {
      const mo = new MutationObserver((records) => {
        for (const r of records) {
          r.addedNodes.forEach((n) => {
            if (n.nodeType === 1) tag(n as Element);
          });
        }
      });
      mo.observe(root, { childList: true, subtree: true });
      observers.push(mo);
    });

    return () => observers.forEach((o) => o.disconnect());
  }, [verseRootSelector]);

  // Proximity hook → toggle data-proximity-near on the nearest element.
  const { target, isNear } = useCursorProximity({ thresholdPx });

  useEffect(() => {
    if (reduced) return;
    // Clear previous flag.
    const prev = lastNearRef.current;
    if (prev && prev !== target) {
      prev.removeAttribute("data-proximity-near");
    }
    if (target && isNear) {
      target.setAttribute("data-proximity-near", "true");
      lastNearRef.current = target;
    } else if (target && !isNear && lastNearRef.current === target) {
      target.removeAttribute("data-proximity-near");
      lastNearRef.current = null;
    }
  }, [target, isNear, reduced]);

  // Cleanup the flag on unmount.
  useEffect(() => {
    return () => {
      if (lastNearRef.current) {
        lastNearRef.current.removeAttribute("data-proximity-near");
        lastNearRef.current = null;
      }
    };
  }, []);

  return null;
}
