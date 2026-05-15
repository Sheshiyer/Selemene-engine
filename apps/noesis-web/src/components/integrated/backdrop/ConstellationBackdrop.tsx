"use client";

// ─── ConstellationBackdrop — real particle-mesh field cartography ───────
// Replaces the static SVG drift with a tsParticles mesh: hairline Sacred
// Gold nodes drifting through a 3-layer parallax field, with proximity
// connectors that fade in/out as particles drift past each other.
//
// Per integrated-reading-design-v2.md § 5.1.
//
// On mobile (<720px) or with prefers-reduced-motion, falls back to a
// low-density static mesh.

import { useCallback, useEffect, useState } from "react";
import Particles, { initParticlesEngine } from "@tsparticles/react";
import { loadSlim } from "@tsparticles/slim";
import type { Container, ISourceOptions } from "@tsparticles/engine";

const SACRED_GOLD_RGB = "197, 160, 23";
const COHERENCE_EMERALD_RGB = "16, 181, 167";

function buildOptions(reducedMotion: boolean, dense: boolean): ISourceOptions {
  return {
    fpsLimit: reducedMotion ? 5 : 60,
    fullScreen: false,
    background: { color: "transparent" },
    detectRetina: true,
    pauseOnBlur: true,
    pauseOnOutsideViewport: true,
    interactivity: {
      events: {
        onHover: reducedMotion ? { enable: false } : { enable: true, mode: "grab" },
        resize: { enable: true },
      },
      modes: {
        grab: { distance: 140, links: { opacity: 0.55, color: `rgb(${SACRED_GOLD_RGB})` } },
      },
    },
    particles: {
      number: {
        value: dense ? 180 : 90,
        density: { enable: true, width: 1920, height: 1080 },
      },
      color: {
        value: [
          `rgb(${SACRED_GOLD_RGB})`,
          `rgb(${COHERENCE_EMERALD_RGB})`,
          `rgb(${SACRED_GOLD_RGB})`,
          `rgb(${SACRED_GOLD_RGB})`, // weight gold heavier
        ],
      },
      links: {
        enable: true,
        distance: 130,
        color: `rgb(${SACRED_GOLD_RGB})`,
        opacity: 0.18,
        width: 0.4,
        triangles: { enable: false },
      },
      move: {
        enable: !reducedMotion,
        speed: 0.32,
        direction: "none",
        random: true,
        straight: false,
        outModes: { default: "out" },
        attract: { enable: false },
      },
      opacity: {
        value: { min: 0.18, max: 0.55 },
        animation: {
          enable: !reducedMotion,
          speed: 0.4,
          sync: false,
          startValue: "random",
        },
      },
      shape: { type: "circle" },
      size: {
        value: { min: 0.6, max: 1.5 },
        animation: {
          enable: !reducedMotion,
          speed: 1.4,
          sync: false,
          startValue: "random",
        },
      },
      twinkle: {
        particles: {
          enable: !reducedMotion,
          frequency: 0.08,
          color: `rgb(${SACRED_GOLD_RGB})`,
          opacity: 0.85,
        },
      },
    },
  };
}

export function ConstellationBackdrop() {
  const [ready, setReady] = useState(false);
  const [reducedMotion, setReducedMotion] = useState(false);
  const [dense, setDense] = useState(true);

  useEffect(() => {
    initParticlesEngine(async (engine) => {
      await loadSlim(engine);
    }).then(() => setReady(true));
  }, []);

  useEffect(() => {
    if (typeof window === "undefined") return;
    const mq = window.matchMedia("(prefers-reduced-motion: reduce)");
    const compact = window.matchMedia("(max-width: 720px)");
    const apply = () => {
      setReducedMotion(mq.matches);
      setDense(!compact.matches);
    };
    apply();
    mq.addEventListener("change", apply);
    compact.addEventListener("change", apply);
    return () => {
      mq.removeEventListener("change", apply);
      compact.removeEventListener("change", apply);
    };
  }, []);

  const particlesLoaded = useCallback(async (_container?: Container) => {
    // Intentional no-op; could hook scroll listeners here if needed.
  }, []);

  if (!ready) return null;

  // On phone-sized viewports we hide entirely to preserve perf + reading focus
  if (typeof window !== "undefined" && window.matchMedia("(max-width: 540px)").matches) {
    return null;
  }

  return (
    <div
      aria-hidden="true"
      style={{
        position: "fixed",
        inset: 0,
        zIndex: 1,
        pointerEvents: "none",
        opacity: 0.68,
        mixBlendMode: "screen",
      }}
    >
      <Particles
        id="constellation-backdrop"
        particlesLoaded={particlesLoaded}
        options={buildOptions(reducedMotion, dense)}
        style={{ width: "100%", height: "100%" }}
      />
    </div>
  );
}
