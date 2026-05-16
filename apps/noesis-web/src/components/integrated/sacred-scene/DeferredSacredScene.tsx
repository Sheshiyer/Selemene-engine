"use client";

// ─── DeferredSacredScene — viewport-gated SacredScene mount ─────────────
// Browsers cap WebGL contexts (~16 max in Chrome, ~8 in Safari). The
// solo route mounts 24 SacredScene instances (cover + 11 part
// atmospheres + 10 transitions + chapter-0 + closing). Exhausting
// contexts → gl.getContextAttributes() returns null → ".alpha" crash
// in @react-three/fiber's Provider.
//
// Fix: only mount the real SacredScene Canvas when the scene is near
// the viewport. Show a static placeholder (preset-tinted gradient)
// when off-screen. IntersectionObserver hysteresis: mount when within
// 1 viewport, unmount when 2+ viewports away.
//
// At any moment only ~3-5 Canvas instances are alive — well within
// the WebGL context limit.

import { useEffect, useRef, useState } from "react";
import dynamic from "next/dynamic";
import { PRESETS, type SceneKind } from "./presets";

// Lazy-load the real SacredScene so the Three.js bundle only loads
// when actually mounted (further perf win on first paint).
const SacredScene = dynamic(
  () => import("./SacredScene").then((m) => m.SacredScene),
  { ssr: false, loading: () => null },
);

interface DeferredSacredSceneProps {
  kind: SceneKind;
  intensity?: number;
  seed?: number;
  height?: string | number;
  className?: string;
  /** Eager mount (e.g. cover — always visible). Skips IntersectionObserver. */
  eager?: boolean;
}

/** Build a static fallback gradient from the kind's preset so the
 *  placeholder visually matches what the live scene would look like. */
function buildPlaceholderBackground(kind: SceneKind): string {
  const preset = PRESETS[kind];
  return `
    radial-gradient(ellipse at 50% 40%, ${preset.core}33 0%, transparent 55%),
    radial-gradient(ellipse at 50% 70%, ${preset.fogTint}44 0%, transparent 60%),
    radial-gradient(circle at 50% 50%, ${preset.edge}22 0%, transparent 80%),
    var(--c-void)
  `;
}

export function DeferredSacredScene({
  kind,
  intensity = 1,
  seed,
  height = "100%",
  className,
  eager = false,
}: DeferredSacredSceneProps) {
  const ref = useRef<HTMLDivElement>(null);
  const [mounted, setMounted] = useState(eager);

  useEffect(() => {
    if (eager) return;
    if (typeof window === "undefined") return;
    const el = ref.current;
    if (!el) return;

    // Mount when within 1 viewport above OR below the scene.
    // Unmount when 2+ viewports away — hysteresis prevents flicker.
    const mountObserver = new IntersectionObserver(
      (entries) => {
        for (const e of entries) {
          if (e.isIntersecting) setMounted(true);
        }
      },
      { rootMargin: "100% 0% 100% 0%", threshold: 0 },
    );
    const unmountObserver = new IntersectionObserver(
      (entries) => {
        for (const e of entries) {
          if (!e.isIntersecting) setMounted(false);
        }
      },
      { rootMargin: "200% 0% 200% 0%", threshold: 0 },
    );
    mountObserver.observe(el);
    unmountObserver.observe(el);
    return () => {
      mountObserver.disconnect();
      unmountObserver.disconnect();
    };
  }, [eager]);

  const placeholderBg = buildPlaceholderBackground(kind);

  return (
    <div
      ref={ref}
      className={className}
      style={{
        position: "relative",
        width: "100%",
        height,
        overflow: "hidden",
        background: mounted ? "transparent" : placeholderBg,
        transition: "background 0.4s ease",
      }}
    >
      {mounted && (
        <SacredScene
          kind={kind}
          intensity={intensity}
          seed={seed}
          height="100%"
        />
      )}
    </div>
  );
}
